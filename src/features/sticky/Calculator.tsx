import { useState } from "react";
import { Calculator as CalcIcon, Check, Copy, Delete, History, Trash2 } from "lucide-react";
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

/**
 * A calculator that lives under the sticky notes. It parses what you type itself (nothing is run as code)
 * and keeps its history only in memory, so closing the app or locking the vault clears it.
 */
export default function Calculator() {
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

  const keys = (list: Key[], cols: number) => (
    <div className="grid gap-1.5" style={{ gridTemplateColumns: `repeat(${cols}, minmax(0, 1fr))` }}>
      {list.map((k, i) => (
        <button
          key={i}
          title={k.title}
          onMouseDown={(e) => e.preventDefault()}
          onClick={() => press(k)}
          className={cn(
            "grid h-11 place-items-center rounded-lg text-[0.95rem] font-medium tabular-nums transition-colors active:scale-[0.97]",
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
    <section aria-label="Calculator" className="rounded-2xl border border-line bg-surface p-5">
      <div className="mb-4 flex items-center gap-3">
        <CalcIcon size={17} className="text-muted" />
        <h3 className="text-sm font-semibold">Calculator</h3>
        <div className="flex-1" />
        {mode === "scientific" && (
          <Segmented
            value={degrees ? "deg" : "rad"}
            onChange={(v) => setDegrees(v === "deg")}
            options={[
              { value: "deg", label: "Deg" },
              { value: "rad", label: "Rad" },
            ]}
          />
        )}
        <Segmented
          value={mode}
          onChange={setMode}
          options={[
            { value: "basic", label: "Basic" },
            { value: "scientific", label: "Scientific" },
          ]}
        />
      </div>

      <div className="flex flex-col gap-5 lg:flex-row">
        <div className={cn("shrink-0", mode === "scientific" ? "w-full lg:w-[520px]" : "w-full lg:w-[300px]")}>
          <div className="mb-3 rounded-xl border border-line bg-bg px-3 py-2.5">
            <input
              ref={input}
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
              <button
                onClick={calc.copy}
                disabled={!preview && !(shown && !shown.error)}
                className="inline-flex items-center gap-1 rounded-md px-1.5 py-0.5 text-xs text-faint hover:bg-surface-2 hover:text-fg disabled:pointer-events-none disabled:opacity-40"
                aria-label="Copy the result"
                title="Copy the result"
              >
                {copied ? <Check size={12} /> : <Copy size={12} />} {copied ? "Copied" : "Copy"}
              </button>
              <div className={cn("truncate text-right font-mono text-sm tabular-nums", shown?.error ? "text-danger" : "text-muted")} aria-live="polite">
                {shown ? (shown.error ? shown.text : `= ${shown.text}`) : preview ? `= ${preview}` : ""}
              </div>
            </div>
          </div>

          <div className="flex gap-3">
            {mode === "scientific" && <div className="w-[210px] shrink-0">{keys(SCIENTIFIC, 3)}</div>}
            <div className="min-w-0 flex-1">{keys(BASIC, 4)}</div>
          </div>
        </div>

        <div className="min-w-0 flex-1 lg:border-l lg:border-line lg:pl-5">
          <div className="mb-2 flex items-center gap-2 text-xs font-medium uppercase tracking-wider text-faint">
            <History size={13} /> History
            <div className="flex-1" />
            {history.length > 0 && (
              <button onClick={() => calc.setHistory([])} className="inline-flex items-center gap-1 normal-case tracking-normal hover:text-fg">
                <Trash2 size={12} /> Clear
              </button>
            )}
          </div>
          {history.length === 0 ? (
            <p className="text-sm leading-relaxed text-faint">
              Your calculations show up here. Click one to use it again. History isn't saved: it is cleared when you lock the vault or close the app.
            </p>
          ) : (
            <ul className="max-h-[300px] space-y-0.5 overflow-y-auto">
              {history.map((h, i) => (
                <li key={i}>
                  <button
                    onClick={() => {
                      calc.type(h.expr);
                      input.current?.focus();
                    }}
                    className="flex w-full items-baseline justify-between gap-3 rounded-lg px-2.5 py-1.5 text-left hover:bg-surface-2"
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
