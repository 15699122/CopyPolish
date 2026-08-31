import { assertAppDidNotPolluteRepository, formatText, waitForApp } from "../support/app.js";

describe("CopyPolish 真实 Tauri 启动链路", () => {
  before(async () => {
    await waitForApp();
  });

  after(async () => {
    await assertAppDidNotPolluteRepository();
  });

  it("通过真实 WebView 和 Rust IPC 格式化默认示例", async () => {
    await expect(await formatText("在LeanCloud上，花了5000元")).toBe(
      "在 LeanCloud 上，花了 5000 元",
    );
  });
});