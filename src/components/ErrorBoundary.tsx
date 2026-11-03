import { Component, type ErrorInfo, type ReactNode } from "react";
import { reportError } from "@/lib/errorlog";

/** Catches a crash while drawing a window, writes it to the local error log (without the component stack,
 * which only has component names but is not needed) and shows a plain message instead of a blank window. */
export default class ErrorBoundary extends Component<{ children: ReactNode }, { failed: boolean }> {
  state = { failed: false };

  static getDerivedStateFromError() {
    return { failed: true };
  }

  componentDidCatch(error: Error, _info: ErrorInfo) {
    reportError("react", `${error?.name ?? "Error"}: ${error?.message ?? ""}`);
  }

  render() {
    if (!this.state.failed) return this.props.children;
    return (
      <div className="flex h-full min-h-screen flex-col items-center justify-center gap-3 bg-bg p-8 text-center text-fg">
        <div className="text-base font-semibold">Something went wrong in this window.</div>
        <p className="max-w-md text-sm leading-relaxed text-muted">
          Notes that were already saved are not affected. Notaya wrote the problem to its error log (Settings, Error log). Close this window and open it
          again, or restart Notaya.
        </p>
        <button
          type="button"
          onClick={() => window.location.reload()}
          className="rounded-lg border border-line bg-surface px-3 py-1.5 text-sm hover:bg-surface-2"
        >
          Reload
        </button>
      </div>
    );
  }
}
