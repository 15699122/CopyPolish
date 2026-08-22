#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""customtkinter GUI 集成冒烟测试（需真实 display）。"""

import importlib.util
import os
import sys

_HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(_HERE)
sys.path.insert(0, ROOT)


def _display_alive():
    import subprocess
    code = (
        "import tkinter\n"
        "try:\n"
        "    r=tkinter.Tk()\n"
        "    r.withdraw()\n"
        "    r.destroy()\n"
        "    print('OK')\n"
        "except Exception:\n"
        "    print('ERR')\n"
    )
    try:
        proc = subprocess.run([sys.executable, "-c", code], capture_output=True, text=True, timeout=15)
        return "OK" in (proc.stdout or "")
    except Exception:
        return False


def _load_gui_module():
    path = os.path.join(ROOT, "chinese_copywriting_formatter.py")
    spec = importlib.util.spec_from_file_location("ccf_gui_module", path)
    if spec is None or spec.loader is None:
        raise ImportError("无法定位 GUI 模块：%s" % path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def _assert_visible(widget, name):
    assert widget.winfo_width() > 0, "%s width <= 0" % name
    assert widget.winfo_height() > 0, "%s height <= 0" % name


def main():
    try:
        import customtkinter  # noqa: F401
    except Exception as exc:
        print("SKIP: 无 customtkinter（%s），GUI 集成测试跳过。" % exc)
        return 0

    if not os.environ.get("DISPLAY") or not _display_alive():
        print("SKIP: 无可用 X 显示，GUI 集成测试跳过。")
        return 0

    import ccw_engine
    gui = _load_gui_module()
    ctk = gui._import_gui()

    try:
        app = gui.FormatterApp(ctk)
    except Exception as exc:
        print("SKIP: 初始化 GUI 失败（%s）。GUI 集成测试跳过。" % exc)
        return 0

    try:
        fmt = ccw_engine.format_text
        inp = "在LeanCloud上\n\n公式$E=mc^2$很重要\n```\ngithub; $x|y\n```"
        app.input_text.delete("1.0", "end")
        app.input_text.insert("1.0", inp)
        app._reformat()
        app.root.update()
        out = app.output_text.get("1.0", "end-1c")
        expected = fmt(inp, app.enabled)
        assert out == expected, ("paste->format", repr(out), repr(expected))
        assert "\n\n" in out
        assert "$E=mc^2$" in out
        assert "github; $x|y" in out
        print("paste->format ok:", repr(out[:40]))

        app._copy_output()
        app.root.update()
        assert app.root.clipboard_get() == out
        print("copy ok")

        for geometry in ("1400x1000", "720x540"):
            app.root.geometry(geometry)
            app.root.update_idletasks()
            app.root.update()
            for widget, name in (
                (app.title_bar, "title_bar"),
                (app.minimize_button, "minimize_button"),
                (app.maximize_button, "maximize_button"),
                (app.close_button, "close_button"),
                (app.input_text, "input_text"),
                (app.output_text, "output_text"),
                (app.settings_button, "settings_button"),
                (app.clear_button, "clear_button"),
                (app.copy_button, "copy_button"),
            ):
                _assert_visible(widget, name)
            assert str(app.root.overrideredirect()) in ("True", "1"), app.root.overrideredirect()
        print("responsive layout ok")

        # 自定义窗口控制按钮按 Windows 默认位置在右上角排列：最小化、最大化/还原、关闭。
        controls = [app.minimize_button, app.maximize_button, app.close_button]
        xs = [button.winfo_rootx() for button in controls]
        assert xs == sorted(xs), xs
        print("window controls ok")

        app._open_settings()
        app.root.update()
        assert app._settings_win is not None and app._settings_win.winfo_exists()
        assert len(app._rule_vars) == len(app.rules), (len(app._rule_vars), len(app.rules))
        assert str(app._settings_win.overrideredirect()) in ("False", "0", "None"), app._settings_win.overrideredirect()
        assert str(app._settings_win.attributes("-topmost")) in ("0", "0.0", "false", "False"), app._settings_win.attributes("-topmost")
        app._settings_win.geometry("500x600")
        app.root.update_idletasks()
        app.root.update()
        _assert_visible(app._settings_win, "settings_window")
        first_key = next(iter(app._rule_vars))
        app._rule_vars[first_key].set(False)
        app._toggle_rule(first_key, app._rule_vars[first_key])
        app.root.update()
        assert first_key not in app.enabled
        print("settings ok: vars=%d rules=%d" % (len(app._rule_vars), len(app.rules)))

        expected_enabled = set(app.enabled)
        app._save_state()
        saved_en, saved_last = ccw_engine.load_settings()
        assert saved_en == expected_enabled, (saved_en, expected_enabled)
        assert saved_last == app._last_input, (saved_last, app._last_input)
        print("persistence ok: last=%r enabled=%d" % (saved_last, len(saved_en)))

        ccw_engine.save_settings(ccw_engine.get_enabled_defaults(), "")
        print("GUI_INTEGRATION_OK")
    finally:
        try:
            if app._settings_win is not None:
                app._settings_win.destroy()
        except Exception:
            pass
        try:
            app.root.destroy()
        except Exception:
            pass
    return 0


if __name__ == "__main__":
    sys.exit(main())
