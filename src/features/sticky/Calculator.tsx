import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Calculator as CalcIcon, Check, Copy, Delete, History, Minus, Trash2, X } from "lucide-react";
import { api } from "@/lib/ipc";
import { applyAppearance, cachedAppearance } from "@/lib/theme";
import { cn } from "@/lib/utils";
import { Segmented } from "@/components/ui";
import { useCalc } from "./useCalc";

type Mode = "basic" | "scientific";

/** What a key puts into the expression, and how it looks. */
interface Key {
  label: React.ReactNode;
  insert?: string;
  action?: "clear" | "back" | "equals" | "negate";
  tone?: "op" | "main" | "fn";
  span?: number;
  title?: string;
}

const BASIC: Key[] = [
  { label: "AC", action: "clear", tone: "fn", title: "Clear (Esc)" },
  { label: <Delete size={17} />, action: "back", tone: "fn", title: "Delete the last character (Backspace)" },
  { label: "(", insert: "(", tone: "fn" },
  { label: ")", insert: ")", tone: "fn" },
  { label: "7", insert: "7" },
  { label: "8", insert: "8" },
  { label: "9", insert: "9" },
  { label: "÷", insert: "÷", tone: "op" },
  { label: "4", insert: "4" },
  { label: "5", insert: "5" },
  { label: "6", insert: "6" },
  { label: "×", insert: "×", tone: "op" },
  { label: "1", insert: "1" },
  { label: "2", insert: "2" },
  { label: "3", insert: "3" },
  { label: "−", insert: "−", tone: "op" },
  { label: "0", insert: "0" },
  { label: ".", insert: "." },
  { label: "%", insert: "%", tone: "fn", title: "Percent: 200 + 10% is 220" },
  { label: "+", insert: "+", tone: "op" },
  { label: "Ans", insert: "Ans", tone: "fn", title: "The last result" },
  { label: "±", action: "negate", tone: "fn", title: "Change sign" },
  { label: "=", action: "equals", tone: "main", span: 2, title: "Calculate (Enter)" },
];

const SCIENTIFIC: Key[] = [
  { label: "sin", insert: "sin(" },
  { label: "cos", insert: "cos(" },
  { label: "tan", insert: "tan(" },
  { label: "sin⁻¹", insert: "asin(" },
  { label: "cos⁻¹", insert: "acos(" },
  { label: "tan⁻¹", insert: "atan(" },
  { label: "ln", insert: "ln(" },
  { label: "log", insert: "log(" },
  { label: "eˣ", insert: "exp(" },
  { label: "√", insert: "√(" },
  { label: "x²", insert: "^2" },
  { label: "xʸ", insert: "^" },
  { label: "π", insert: "π" },
  { label: "e", insert: "e" },
  { label: "n!", insert: "!" },
  { label: "|x|", insert: "abs(" },
  { label: "1/x", insert: "1÷(" },
  { label: "10ˣ", insert: "10^" },
];

const toneClass = (tone: Key["tone"]) =>
  tone === "main"
    ? "bg-accent text-accent-fg hover:opacity-90"
    : tone === "op"
      ? "bg-surface-2 text-fg hover:bg-line"
      : tone === "fn"
        ? "bg-surface text-muted hover:bg-surface-2 hover:text-fg"
        : "bg-surface text-fg hover:bg-surface-2";

const win = getCurrentWindow();

/**
 * A calculator in its own small window, opened from the sticky notes. Any number can be open at once.
 * It parses what you type itself (nothing is run as code). The history is shared by every calculator
 * window and saved in the vault, so it is still there after the windows close; Clear empties it.
 */
export default function CalculatorWindow() {
  useEffect(() => {
    applyAppearance(cachedAppearance(), false);
    // Rust asks every window to save before the vault locks. A calculator has nothing pending: answer at once.
    const unlisten = listen<number>("vault-locking", (event) => {
      api.flushDone(event.payload).catch(() => {});
    });
    // Ready only once it can answer, so a lock never waits for this window.
    unlisten.then(() => requestAnimationFrame(() => requestAnimationFrame(() => api.windowReady())));
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  return (
    <div className="h-full p-2.5">
      <div className="relative flex h-full flex-col overflow-hidden rounded-2xl border border-line bg-bg shadow-[0_12px_32px_-8px_rgba(0,0,0,0.45)]">
        <Calculator />
        <div
          className="absolute bottom-1 right-1 h-3 w-3 cursor-se-resize opacity-40"
          onMouseDown={(e) => {
            e.preventDefault();
            win.startResizeDragging("SouthEast");
          }}
          style={{ background: "linear-gradient(135deg, transparent 50%, var(--text) 50%)", borderBottomRightRadius: 4 }}
        />
      </div>
    </div>
  );
}

function Calculator() {
  const [mode, setMode] = useState<Mode>("basic");
  const [degrees, setDegrees] = useState(true);
  const calc = useCalc(degrees);
  const { expr, input, preview, shown, copied, history } = calc;

  const press = (k: Key) => {
    if (k.insert) return calc.insert(k.insert);
    if (k.action === "clear") calc.clear();
    else if (k.action === "back") calc.back();
    else if (k.action === "equals") calc.equals();
    else if (k.action === "negate") calc.negate();
    input.current?.focus();
  };

  const keys = (list: Key[], cols: number, small?: boolean) => (
    <div className="grid gap-1.5" style={{ gridTemplateColumns: `repeat(${cols}, minmax(0, 1fr))` }}>
      {list.map((k, i) => (
        <button
          key={i}
          title={k.title}
          onMouseDown={(e) => e.preventDefault()}
          onClick={() => press(k)}
          className={cn(
            "grid place-items-center rounded-lg font-medium tabular-nums transition-colors active:scale-[0.97]",
            small ? "h-8 text-[0.8rem]" : "h-9 text-[0.95rem]",
            toneClass(k.tone),
          )}
          style={k.span ? { gridColumn: `span ${k.span}` } : undefined}
        >
          {k.label}
        </button>
      ))}
    </div>
  );

  return (
    <section aria-label="Calculator" className="flex min-h-0 flex-1 flex-col">
      <div data-tauri-drag-region className="flex h-10 shrink-0 items-center gap-2 border-b border-line px-3">
        <CalcIcon size={15} className="pointer-events-none text-muted" />
        <h1 data-tauri-drag-region className="text-sm font-semibold">
          Calculator
        </h1>
        <div data-tauri-drag-region className="flex-1" />
        <button
          onClick={() => win.minimize()}
          className="grid h-6 w-6 place-items-center rounded-md text-muted hover:bg-surface-2 hover:text-fg"
          aria-label="Minimize"
          title="Minimize"
        >
          <Minus size={14} />
        </button>
        <button
          onClick={() => win.close()}
          className="grid h-6 w-6 place-items-center rounded-md text-muted hover:bg-surface-2 hover:text-fg"
          aria-label="Close (the history stays saved)"
          title="Close (the history stays saved)"
        >
          <X size={15} />
        </button>
      </div>

      <div className="flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto p-3">
        <div className="flex items-center gap-2">
          <Segmented
            value={mode}
            onChange={setMode}
            options={[
              { value: "basic", label: "Basic" },
              { value: "scientific", label: "Scientific" },
            ]}
          />
          <div className="flex-1" />
          {mode === "scientific" && (
            <Segmented
              value={degrees ? "deg" : "rad"}
              onChange={(v) => {
                if ((v === "deg") === degrees) return; // the unit already shown: nothing to change
                setDegrees(v === "deg");
                calc.recallForAngleUnit();
              }}
              options={[
                { value: "deg", label: "Deg" },
                { value: "rad", label: "Rad" },
              ]}
            />
          )}
        </div>

        <div className="shrink-0 rounded-xl border border-line bg-surface px-3 py-2.5">
          <input
            ref={input}
            autoFocus
            value={expr}
            onChange={(e) => calc.type(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.preventDefault();
                calc.equals();
              } else if (e.key === "Escape") calc.clear();
              else if (e.key === "ArrowUp" && history[0]) {
                e.preventDefault();
                calc.type(history[0].expr);
              }
            }}
            aria-label="Expression"
            placeholder="Type or tap: 12.5 × (3 + 4)"
            spellCheck={false}
            autoComplete="off"
            className="w-full bg-transparent text-right font-mono text-lg tabular-nums text-fg outline-none placeholder:text-faint"
            style={{ outline: "none", boxShadow: "none" }}
          />
          <div className="mt-1 flex h-6 items-center justify-between gap-2">
            <div className="flex items-center gap-2">
              <button
                onClick={calc.copy}
                disabled={!preview && !(shown && !shown.error)}
                className="inline-flex items-center gap-1 rounded-md px-1.5 py-0.5 text-xs text-faint hover:bg-surface-2 hover:text-fg disabled:pointer-events-none disabled:opacity-40"
                aria-label="Copy the result"
                title="Copy the result"
              >
                {copied ? <Check size={12} /> : <Copy size={12} />} {copied ? "Copied" : "Copy"}
              </button>
              {mode === "scientific" && (
                <span className="text-xs font-medium text-faint" title="Angles for sin, cos and tan are in this unit">
                  <span aria-hidden="true">{degrees ? "DEG" : "RAD"}</span>
                  <span className="sr-only">{degrees ? "Angles in degrees" : "Angles in radians"}</span>
                </span>
              )}
            </div>
            <div className={cn("truncate text-right font-mono text-sm tabular-nums", shown?.error ? "text-danger" : "text-muted")} aria-live="polite">
              {shown ? (shown.error ? shown.text : `= ${shown.text}`) : preview ? `= ${preview}` : ""}
            </div>
          </div>
        </div>

        <div className="shrink-0 space-y-1.5">
          {mode === "scientific" && keys(SCIENTIFIC, 6, true)}
          {keys(BASIC, 4)}
        </div>

        <div className="flex min-h-[120px] flex-1 flex-col border-t border-line pt-2">
          <div className="mb-1 flex items-center gap-2 text-xs font-medium uppercase tracking-wider text-faint">
            <History size={13} /> History
            <div className="flex-1" />
            {history.length > 0 && (
              <button onClick={calc.clearHistory} className="inline-flex items-center gap-1 normal-case tracking-normal hover:text-fg">
                <Trash2 size={12} /> Clear
              </button>
            )}
          </div>
          {history.length === 0 ? (
            <p className="text-xs leading-relaxed text-faint">
              Your calculations show up here, in every calculator window. Click one to use it again. They stay saved in your vault until you clear them.
            </p>
          ) : (
            <ul className="min-h-0 flex-1 space-y-0.5 overflow-y-auto">
              {history.map((h, i) => (
                <li key={i}>
                  <button
                    onClick={() => {
                      calc.type(h.expr);
                      input.current?.focus();
                    }}
                    className="flex w-full items-baseline justify-between gap-3 rounded-lg px-2 py-1 text-left hover:bg-surface-2"
                  >
                    <span className="truncate font-mono text-sm text-muted">{h.expr}</span>
                    <span className="shrink-0 font-mono text-sm tabular-nums text-fg">= {h.result}</span>
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>
    </section>
  );
}
