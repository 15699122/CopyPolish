/// <reference types="vite/client" />
/** Vite 从 frontend/package.json 注入的浏览器预览版本回退值。 */
declare const __APP_VERSION__: string;

interface CopyPolishE2EDiagnostics {
  lastFormatRequest?: unknown;
  lastFormatResult?: string;
  lastFormatError?: string;
  lastSettingsSave?: unknown;
  settingsSaveSequence?: number;
  buildCapabilities?: {
    simplifiedTradConversion: boolean;
  };
}

interface Window {
  __COPYPOLISH_E2E__?: CopyPolishE2EDiagnostics;
}
