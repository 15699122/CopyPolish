/// <reference types="vitest/config" />
import path from "path";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import packageJson from "./package.json" with { type: "json" };

// Tauri 期望固定的 dev 端口，且使用自定义协议加载前端资源。
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  // Tauri 使用 custom protocol/asset URL 加载打包后的前端资源；相对基路径
  // 避免生成 `/assets/...` 这类只适用于 HTTP 根路径的绝对资源地址。
  base: "./",
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": path.resolve(import.meta.dirname, "./src"),
    },
  },
  clearScreen: false,
  define: {
    __APP_VERSION__: JSON.stringify(packageJson.version),
  },
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? { protocol: "ws", host, port: 1421 }
      : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  // Vitest（组件交互测试，jsdom 环境）
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./src/test/setup.ts"],
    css: false,
  },
});
