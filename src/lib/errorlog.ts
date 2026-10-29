// Sends problems of the UI to the local error log (a text file on this computer; nothing is sent
// anywhere). The Rust side cleans every field again, so this file only keeps the report small and
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
export function reportError(kind: ErrorKind, message: unknown, source?: string | null, line?: number | null): void {
  if (busy || sent >= MAX_REPORTS) return;
  busy = true;
  try {
    const text = String(message ?? "").slice(0, 400);
    const key = `${kind}|${text}|${source ?? ""}|${line ?? ""}`;
    const now = Date.now();
    if (key === lastKey && now - lastAt < REPEAT_MS) return;
    lastKey = key;
    lastAt = now;
    sent += 1;
    const report = {
      kind,
      message: text,
      source: source ? String(source).slice(0, 300) : null,
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

/** What to write about a rejected promise. A text rejection is an error message from a command of the
 * app (already logged by the backend if it was a real fault), so only the fact is written. */
function describeReason(reason: unknown): string {
  if (reason instanceof Error) return `${reason.name}: ${reason.message}`;
  return `A promise was rejected with a value of type ${typeof reason}`;
}

/** Installs the handlers once. Call it before React renders. */
export function installErrorLog(): void {
  if (installed) return;
  installed = true;
  try {
    window.addEventListener("error", (e: ErrorEvent) => {
      try {
        // A failed load of an image or script has no message; the event is still worth one line.
        reportError("error", e.message || "An error without a message", e.filename, e.lineno);
      } catch {
        // never throw from a handler of errors
      }
    });
    window.addEventListener("unhandledrejection", (e: PromiseRejectionEvent) => {
      try {
        reportError("unhandledrejection", describeReason(e.reason));
      } catch {
        // never throw from a handler of errors
      }
    });
  } catch {
    // No window to listen on; nothing to log.
  }
}
