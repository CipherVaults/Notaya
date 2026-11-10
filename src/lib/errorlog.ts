// Sends problems of the UI to the local error log (a text file on this computer; nothing is sent
// anywhere). Only fixed error classes and bounded locations are sent; error messages stay out.
// The Rust side validates every field again, so this file only keeps the report small and
// makes sure that reporting can never become a problem of its own: it never throws, never reports
// itself and stops after a few dozen reports.

import { invoke } from "@tauri-apps/api/core";

export type ErrorKind = "error" | "unhandledrejection" | "react";

/** The most reports one window sends in its lifetime (the backend has its own, lower limit). */
const MAX_REPORTS = 60;
/** The same report again within this many milliseconds is dropped (a loop of one error). */
const REPEAT_MS = 5000;

let sent = 0;
let busy = false;
let lastKey = "";
let lastAt = 0;
let installed = false;

/** Writes one report. Safe to call from anywhere: it swallows every failure. */
export function reportError(kind: ErrorKind, source?: string | null, line?: number | null): void {
  if (busy || sent >= MAX_REPORTS) return;
  busy = true;
  try {
    const name = source?.split(/[?#]/, 1)[0].split(/[\\/]/).pop() ?? "";
    const basename = /^[A-Za-z0-9._-]{1,60}$/.test(name) ? name : null;
    const key = `${kind}|${basename ?? ""}|${line ?? ""}`;
    const now = Date.now();
    if (key === lastKey && now - lastAt < REPEAT_MS) return;
    lastKey = key;
    lastAt = now;
    sent += 1;
    const report = {
      kind,
      source: basename,
      line: typeof line === "number" && Number.isFinite(line) ? line : null,
    };
    // The answer is not awaited and a failure is ignored: there is nobody to tell.
    Promise.resolve(invoke("log_frontend_error", { report })).catch(() => {});
  } catch {
    // Nothing to do; an error here must not be reported again.
  } finally {
    busy = false;
  }
}

/** Installs the handlers once. Call it before React renders. */
export function installErrorLog(): void {
  if (installed) return;
  installed = true;
  try {
    window.addEventListener("error", (e: ErrorEvent) => {
      try {
        // A failed load of an image or script has no message; the event is still worth one line.
        reportError("error", e.filename, e.lineno);
      } catch {
        // never throw from a handler of errors
      }
    });
    window.addEventListener("unhandledrejection", () => {
      try {
        reportError("unhandledrejection");
      } catch {
        // never throw from a handler of errors
      }
    });
  } catch {
    // No window to listen on; nothing to log.
  }
}
