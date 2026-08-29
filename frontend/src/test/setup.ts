import "@testing-library/jest-dom/vitest";

// React 19 需要显式标记测试运行在 act 支持的环境中。
(globalThis as typeof globalThis & { IS_REACT_ACT_ENVIRONMENT: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

// Node 24+ 在未提供 --localstorage-file 时访问内置 localStorage getter 会输出 warning；
// 测试只需要进程内隔离的存储，因此始终替换为轻量内存实现。
const localStorageStore = new Map<string, string>();
Object.defineProperty(window, "localStorage", {
  configurable: true,
  value: {
    getItem: (k: string) => localStorageStore.get(k) ?? null,
    setItem: (k: string, v: string) => void localStorageStore.set(k, String(v)),
    removeItem: (k: string) => void localStorageStore.delete(k),
    clear: () => void localStorageStore.clear(),
  },
});

// jsdom 未实现的 API（复制按钮走 clipboard）。
Object.assign(navigator, {
  clipboard: {
    writeText: async () => undefined,
  },
});

window.scrollTo = () => {};

// Radix ScrollArea 依赖 ResizeObserver。
if (typeof globalThis.ResizeObserver === "undefined") {
  class RO {
    observe() {}
    unobserve() {}
    disconnect() {}
  }
  Object.defineProperty(globalThis, "ResizeObserver", { configurable: true, value: RO });
}