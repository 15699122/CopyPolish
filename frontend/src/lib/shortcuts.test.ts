import { describe, expect, it } from "vitest";

import {
  DEFAULT_SHORTCUT_BINDINGS,
  SHORTCUT_ACTION_LABELS,
  comboFromEvent,
  eventMatchesShortcut,
  formatComboForDisplay,
  parseCombo,
  serializeCombo,
  validateBinding,
} from "./shortcuts";

function keyEvent(
  init: Partial<KeyboardEvent> & { code: string },
): KeyboardEvent {
  return {
    ctrlKey: false,
    metaKey: false,
    shiftKey: false,
    altKey: false,
    isComposing: false,
    keyCode: 0,
    ...init,
  } as unknown as KeyboardEvent;
}

describe("shortcuts 序列化与解析", () => {
  it("默认绑定均可解析且 round-trip 一致", () => {
    for (const binding of Object.values(DEFAULT_SHORTCUT_BINDINGS)) {
      const parsed = parseCombo(binding);
      expect(parsed).not.toBeNull();
      expect(parsed!.ctrlOrCmd).toBe(true);
      expect(serializeCombo(parsed!)).toBe(binding);
    }
  });

  it("拒绝非法格式", () => {
    expect(parseCombo("")).toBeNull();
    expect(parseCombo("Ctrl+KeyA")).toBeNull(); // 不支持裸 Ctrl 语义修饰
    expect(validateBinding("CtrlOrCmd+Bogus1", {})).not.toBeNull(); // 未知按键由校验拒绝
  });
});

describe("eventMatchesShortcut", () => {
  it("CtrlOrCmd 同时匹配 ctrlKey 与 metaKey", () => {
    const binding = "CtrlOrCmd+Enter";
    expect(
      eventMatchesShortcut(keyEvent({ code: "Enter", ctrlKey: true }), binding),
    ).toBe(true);
    expect(
      eventMatchesShortcut(keyEvent({ code: "Enter", metaKey: true }), binding),
    ).toBe(true);
    expect(eventMatchesShortcut(keyEvent({ code: "Enter" }), binding)).toBe(
      false,
    );
  });

  it("多余的修饰键视为不匹配", () => {
    const binding = "CtrlOrCmd+Enter";
    expect(
      eventMatchesShortcut(
        keyEvent({ code: "Enter", ctrlKey: true, shiftKey: true }),
        binding,
      ),
    ).toBe(false);
  });

  it("按 KeyboardEvent.code 匹配，不受 event.key 影响", () => {
    const event = keyEvent({
      code: "KeyC",
      ctrlKey: true,
      shiftKey: true,
      key: "ç", // 某些键盘布局下 key 不同
    });
    expect(eventMatchesShortcut(event, "CtrlOrCmd+Shift+KeyC")).toBe(true);
  });
});

describe("comboFromEvent 与 IME 防护", () => {
  it("仅按下修饰键时返回 null", () => {
    expect(comboFromEvent(keyEvent({ code: "ControlLeft", ctrlKey: true }))).toBeNull();
  });

  it("isComposing 或 keyCode 229 时返回 null", () => {
    expect(
      comboFromEvent(keyEvent({ code: "Enter", ctrlKey: true, isComposing: true })),
    ).toBeNull();
    expect(
      comboFromEvent(keyEvent({ code: "Enter", ctrlKey: true, keyCode: 229 })),
    ).toBeNull();
  });
});

describe("validateBinding", () => {
  it("缺少 Ctrl/Cmd 修饰键被拒绝", () => {
    expect(validateBinding("Shift+KeyA", {})).toContain("Ctrl/Cmd");
  });

  it("重复绑定被拒绝并指出冲突动作", () => {
    const error = validateBinding("CtrlOrCmd+Enter", {
      copy_output: "CtrlOrCmd+Enter",
    });
    expect(error).toBe(`该组合键已被「${SHORTCUT_ACTION_LABELS.copy_output}」占用`);
  });

  it("系统黑名单组合被拒绝", () => {
    expect(validateBinding("CtrlOrCmd+KeyW", {})).not.toBeNull();
    expect(validateBinding("CtrlOrCmd+KeyQ", {})).not.toBeNull();
  });

  it("不支持的按键（如 Tab/Escape/标点）被拒绝；Comma 作为默认兼容例外", () => {
    expect(validateBinding("CtrlOrCmd+Tab", {})).not.toBeNull();
    expect(validateBinding("CtrlOrCmd+Escape", {})).not.toBeNull();
    expect(validateBinding("CtrlOrCmd+Semicolon", {})).not.toBeNull();
    expect(validateBinding("CtrlOrCmd+Comma", {})).toBeNull();
  });

  it("字母、数字、功能键、方向键合法", () => {
    expect(validateBinding("CtrlOrCmd+Shift+KeyF", {})).toBeNull();
    expect(validateBinding("CtrlOrCmd+Digit5", {})).toBeNull();
    expect(validateBinding("CtrlOrCmd+F2", {})).toBeNull();
    expect(validateBinding("CtrlOrCmd+ArrowDown", {})).toBeNull();
  });
});

describe("formatComboForDisplay", () => {
  it("输出人类可读文本", () => {
    expect(formatComboForDisplay("CtrlOrCmd+Enter")).toBe("Ctrl/Cmd + Enter");
    expect(formatComboForDisplay("CtrlOrCmd+Shift+KeyC")).toBe("Ctrl/Cmd + Shift + C");
    expect(formatComboForDisplay("CtrlOrCmd+Comma")).toBe("Ctrl/Cmd + ,");
  });

  it("非法字符串原样返回", () => {
    expect(formatComboForDisplay("nonsense")).toBe("nonsense");
  });
});
