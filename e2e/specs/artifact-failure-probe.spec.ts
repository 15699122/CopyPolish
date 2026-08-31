import { formatText, waitForApp } from "../support/app.js";

const enabled = process.env.COPYPOLISH_E2E_ARTIFACT_PROBE === "1";

describe("CopyPolish E2E 失败 artifact probe", () => {
  if (!enabled) {
    it.skip("需要通过 test:artifact-probe runner 显式启用", () => {});
    return;
  }

  before(async () => {
    await waitForApp();
  });

  it("先完成真实 Rust IPC，再按预期失败以验证诊断包", async () => {
    const source = "在LeanCloud上，花了5000元";
    const output = await formatText(source);
    expect(output).toBe("在 LeanCloud 上，花了 5000 元");

    throw new Error("expected artifact failure probe error");
  });
});