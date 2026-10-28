import { useEffect, useMemo, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api } from "@/lib/ipc";
import type { CalcEntry } from "@/lib/types";
import { CalcError, dependsOnAngleUnit, evaluate, formatNumber, negateExpr } from "./calc";

/**
 * The state and actions of a calculator window. The display and "Ans" belong to this window; the history
 * is shared by every calculator window and saved in the vault, so it is still there after closing them.
 */
export function useCalc(degrees: boolean) {
  const [expr, setExpr] = useState("");
  const [ans, setAns] = useState(0);
  const [history, setHistory] = useState<CalcEntry[]>([]);

  // Load the shared history, and reload it whenever any calculator window changes it.
  useEffect(() => {
    const load = () => api.calcHistory().then(setHistory).catch(() => {});
    load();
    const unlisten = listen("calc-history-changed", load);
    return () => {
      unlisten.then((f) => f());
    };
  }, []);
  const [shown, setShown] = useState<{ text: string; error: boolean } | null>(null);
  const [copied, setCopied] = useState(false);
  const input = useRef<HTMLInputElement>(null);
  // The last expression calculated in this window (the shared history may have newer lines from other windows).
  const lastExpr = useRef("");

  // A live preview of the answer while typing.
  const preview = useMemo(() => {
    if (!expr.trim()) return null;
    try {
      return formatNumber(evaluate(expr, { degrees, ans }));
    } catch {
      return null;
    }
  }, [expr, degrees, ans]);

  const type = (value: string) => {
    setExpr(value);
    setShown(null);
  };

  const insert = (text: string) => {
    const el = input.current;
    const start = el?.selectionStart ?? expr.length;
    const end = el?.selectionEnd ?? expr.length;
    setExpr((cur) => cur.slice(0, start) + text + cur.slice(end));
    setShown(null);
    requestAnimationFrame(() => {
      el?.focus();
      el?.setSelectionRange(start + text.length, start + text.length);
    });
  };

  const clear = () => {
    setExpr("");
    setShown(null);
  };
  const back = () => {
    setExpr((x) => x.slice(0, -1));
    setShown(null);
  };
  const negate = () => {
    setExpr(negateExpr);
    setShown(null);
  };

  const equals = () => {
    if (!expr.trim()) return;
    try {
      const value = evaluate(expr, { degrees, ans });
      const text = formatNumber(value);
      lastExpr.current = expr.trim();
      api.addCalcEntry(expr.trim(), text).then(setHistory).catch(() => {});
      setAns(value);
      setExpr(text);
      setShown({ text, error: false });
    } catch (e) {
      setShown({ text: e instanceof CalcError ? e.message : "Check the expression", error: true });
    }
  };

  // After "=" the display holds only the answer, so switching Deg/Rad would seem to do nothing. Bring the
  // last expression back (if it has sin, cos or tan) so the preview shows its answer in the new unit.
  const recallForAngleUnit = () => {
    const last = lastExpr.current;
    if (!shown || shown.error || !last || !dependsOnAngleUnit(last)) return;
    setExpr(last);
    setShown(null);
  };

  const copy = async () => {
    const text = shown && !shown.error ? shown.text : preview;
    if (!text) return;
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      // Fall back to the older way if the page isn't allowed to use the clipboard API.
      const ta = document.createElement("textarea");
      ta.value = text;
      document.body.append(ta);
      ta.select();
      document.execCommand("copy");
      ta.remove();
    }
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  };

  const clearHistory = () => {
    setHistory([]);
    // If it couldn't be cleared, show what is still saved.
    api.clearCalcHistory().catch(() => api.calcHistory().then(setHistory).catch(() => {}));
  };

  return { expr, type, input, preview, shown, copied, history, clearHistory, insert, clear, back, negate, equals, recallForAngleUnit, copy };
}
