import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import "@fontsource-variable/inter";
import "@fontsource-variable/jetbrains-mono";
import "./styles/app.css";
import App from "./App";
import StickyWindow from "./features/sticky/StickyWindow";
import CalculatorWindow from "./features/sticky/Calculator";
import ErrorBoundary from "./components/ErrorBoundary";
import { installErrorLog } from "./lib/errorlog";
import { startupMark } from "./lib/startup";

startupMark("bootstrapAtMs");

// Before anything is drawn: errors of this window go to the local error log (a file on this computer).
installErrorLog();

// Keep browser controls out of the desktop windows and production builds.
if (import.meta.env.PROD || "__TAURI_INTERNALS__" in window) {
  document.addEventListener("contextmenu", (e) => {
    const target = e.target instanceof Element ? e.target : e.target instanceof Node ? e.target.parentElement : null;
    const editable = target?.closest("[contenteditable]");
    if (!target?.closest("input, textarea") && !(editable instanceof HTMLElement && editable.isContentEditable)) {
      e.preventDefault();
    }
  });
  window.addEventListener("keydown", (e) => {
    const k = e.key.toLowerCase();
    const blocked =
      k === "f3" ||
      k === "f5" ||
      k === "f7" ||
      (import.meta.env.PROD && k === "f12") ||
      (e.ctrlKey && !e.altKey && (
        ["g", "h", "j", "o", "p", "r", "s", "u", "+", "=", "-", "_", "0"].includes(k) ||
        (e.shiftKey && ["i", "c"].includes(k))
      ));
    if (blocked) e.preventDefault();
  }, { capture: true });
  // Note/row Find and the shell run first. Auxiliary windows and the lock screen have no shell.
  window.addEventListener("keydown", (e) => {
    if (e.ctrlKey && !e.altKey && e.key.toLowerCase() === "f") e.preventDefault();
  });
  window.addEventListener("wheel", (e) => {
    if (e.ctrlKey) e.preventDefault();
  }, { capture: true, passive: false });
  const blockMiddleClick = (e: MouseEvent) => {
    if (e.button === 1) e.preventDefault();
  };
  window.addEventListener("mousedown", blockMiddleClick, { capture: true });
  window.addEventListener("auxclick", blockMiddleClick, { capture: true });
}

const label = getCurrentWebviewWindow().label;
const root = ReactDOM.createRoot(document.getElementById("root") as HTMLElement);

if (label.startsWith("sticky-")) {
  document.documentElement.dataset.window = "sticky";
  root.render(
    <React.StrictMode>
      <ErrorBoundary>
        <StickyWindow id={label.slice("sticky-".length)} />
      </ErrorBoundary>
    </React.StrictMode>,
  );
} else if (label.startsWith("calc-")) {
  document.documentElement.dataset.window = "calc";
  root.render(
    <React.StrictMode>
      <ErrorBoundary>
        <CalculatorWindow />
      </ErrorBoundary>
    </React.StrictMode>,
  );
} else {
  root.render(
    <React.StrictMode>
      <ErrorBoundary>
        <App />
      </ErrorBoundary>
    </React.StrictMode>,
  );
}
