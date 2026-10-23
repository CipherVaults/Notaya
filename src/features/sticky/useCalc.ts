import { useMemo, useRef, useState } from "react";
import { CalcError, evaluate, formatNumber, negateExpr } from "./calc";

export interface CalcEntry {
  expr: string;
  result: string;
}

/**
 * The state and actions of a calculator display, shared by the big calculator under the sticky notes and
 * the small one inside a sticky note. Nothing is stored: it all lives in memory and is gone when the
 * screen reloads (for example when the vault locks).
 */
export function useCalc(degrees: boolean) {
  const [expr, setExpr] = useState("");
  const [ans, setAns] = useState(0);
  const [history, setHistory] = useState<CalcEntry[]>([]);
  const [shown, setShown] = useState<{ text: string; error: boolean } | null>(null);
  const [copied, setCopied] = useState(false);
  const input = useRef<HTMLInputElement>(null);

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
  const negate = () => setExpr(negateExpr);

  const equals = () => {
    if (!expr.trim()) return;
    try {
      const value = evaluate(expr, { degrees, ans });
      const text = formatNumber(value);
      setHistory((h) => [{ expr: expr.trim(), result: text }, ...h].slice(0, 30));
      setAns(value);
      setExpr(text);
      setShown({ text, error: false });
    } catch (e) {
      setShown({ text: e instanceof CalcError ? e.message : "Check the expression", error: true });
    }
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

  return { expr, type, input, preview, shown, copied, history, setHistory, insert, clear, back, negate, equals, copy };
}
