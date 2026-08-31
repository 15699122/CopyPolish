import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import "./index.css";

if (
  import.meta.env.VITE_COPYPOLISH_E2E === "true"
  && import.meta.env.VITE_COPYPOLISH_E2E_PROVIDER !== "webdriver"
) {
  await import("@wdio/tauri-plugin").then(({ init }) => init());
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
