import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import "@fontsource-variable/inter";
import "@fontsource-variable/jetbrains-mono";
import "./styles/app.css";
import App from "./App";
import StickyWindow from "./features/sticky/StickyWindow";
import CalculatorWindow from "./features/sticky/Calculator";

// In the shipped app, block browser shortcuts that make no sense here
// (reload, print, view source, find, dev tools) and the default context menu.
if (import.meta.env.PROD) {
  document.addEventListener("contextmenu", (e) => {
    const t = e.target as HTMLElement;
    if (!t.closest("input, textarea, [contenteditable='true']")) e.preventDefault();
  });
  document.addEventListener("keydown", (e) => {
    const k = e.key.toLowerCase();
    const blocked =
      k === "f5" ||
      k === "f7" ||
      k === "f12" ||
      (e.ctrlKey && ["r", "p", "u", "g", "j"].includes(k)) ||
      (e.ctrlKey && e.shiftKey && ["i", "j", "c"].includes(k));
    if (blocked) e.preventDefault();
  });
}

const label = getCurrentWebviewWindow().label;
const root = ReactDOM.createRoot(document.getElementById("root") as HTMLElement);

if (label.startsWith("sticky-")) {
  document.documentElement.dataset.window = "sticky";
  root.render(
    <React.StrictMode>
      <StickyWindow id={label.slice("sticky-".length)} />
    </React.StrictMode>,
  );
} else if (label.startsWith("calc-")) {
  document.documentElement.dataset.window = "calc";
  root.render(
    <React.StrictMode>
      <CalculatorWindow />
    </React.StrictMode>,
  );
} else {
  root.render(
    <React.StrictMode>
      <App />
    </React.StrictMode>,
  );
}
