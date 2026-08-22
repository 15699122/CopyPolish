import "@testing-library/jest-dom/vitest";

// 部分 jsdom 配置下 localStorage 未随 window 提供，补一个内存实现。
if (typeof window.localStorage === "undefined") {
  const store = new Map<string, string>();
  Object.defineProperty(window, "localStorage", {
    configurable: true,
    value: {
      getItem: (k: string) => store.get(k) ?? null,
      setItem: (k: string, v: string) => void store.set(k, String(v)),
      removeItem: (k: string) => void store.delete(k),
      clear: () => void store.clear(),
    },
  });
}

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