import { Delete } from "lucide-react";
import { useCalc } from "./useCalc";

type Cell = { label: React.ReactNode; insert?: string; action?: "clear" | "back" | "equals" | "negate"; wide?: boolean; strong?: boolean; title?: string };

const KEYS: Cell[] = [
  { label: "AC", action: "clear", title: "Clear (Esc)" },
  { label: <Delete size={14} />, action: "back", title: "Delete the last character" },
  { label: "(", insert: "(" },
  { label: ")", insert: ")" },
  { label: "7", insert: "7" },
  { label: "8", insert: "8" },
  { label: "9", insert: "9" },
  { label: "÷", insert: "÷", strong: true },
  { label: "4", insert: "4" },
  { label: "5", insert: "5" },
  { label: "6", insert: "6" },
  { label: "×", insert: "×", strong: true },
  { label: "1", insert: "1" },
  { label: "2", insert: "2" },
  { label: "3", insert: "3" },
  { label: "−", insert: "−", strong: true },
  { label: "0", insert: "0" },
  { label: ".", insert: "." },
  { label: "%", insert: "%", title: "Percent: 200 + 10% is 220" },
  { label: "+", insert: "+", strong: true },
  { label: "Ans", insert: "Ans", title: "The last result" },
  { label: "±", action: "negate", title: "Change sign" },
  { label: "=", action: "equals", wide: true, strong: true, title: "Calculate (Enter)" },
];

/**
 * A small calculator that covers the body of a sticky note. It takes the note's own colors, so it looks
 * like part of the note, and it keeps nothing: closing it (or locking the vault) clears it.
 */
export default function MiniCalculator({ ink, paper, onClose }: { ink: string; paper: string; onClose: () => void }) {
  const calc = useCalc(true);
  const { expr, input, preview, shown } = calc;

  const press = (k: Cell) => {
    if (k.insert) calc.insert(k.insert);
    else if (k.action === "clear") calc.clear();
    else if (k.action === "back") calc.back();
    else if (k.action === "equals") calc.equals();
    else if (k.action === "negate") calc.negate();
    input.current?.focus();
  };

  return (
    <div className="flex h-full flex-col gap-1.5 overflow-y-auto px-3 py-2" aria-label="Calculator">
      <div className="rounded-lg px-2.5 py-1.5" style={{ background: `color-mix(in oklab, ${ink} 9%, transparent)` }}>
        <input
          ref={input}
          autoFocus
          value={expr}
          onChange={(e) => calc.type(e.target.value)}
          onKeyDown={(e) => {
            e.stopPropagation();
            if (e.key === "Enter") {
              e.preventDefault();
              calc.equals();
            } else if (e.key === "Escape") {
              // First Esc clears; Esc on an empty display closes the calculator.
              if (expr) calc.clear();
              else onClose();
            }
          }}
          aria-label="Expression"
          placeholder="Type or tap"
          spellCheck={false}
          autoComplete="off"
          className="w-full bg-transparent text-right font-mono text-base tabular-nums outline-none placeholder:opacity-40"
          style={{ color: ink, outline: "none", boxShadow: "none" }}
        />
        <div
          className="h-4 truncate text-right font-mono text-xs tabular-nums"
          style={{ color: shown?.error ? "#c0243a" : ink, opacity: shown?.error ? 1 : 0.65 }}
          aria-live="polite"
        >
          {shown ? (shown.error ? shown.text : `= ${shown.text}`) : preview ? `= ${preview}` : ""}
        </div>
      </div>

      <div className="grid grid-cols-4 gap-1">
        {KEYS.map((k, i) => (
          <button
            key={i}
            title={k.title}
            onMouseDown={(e) => e.preventDefault()}
            onClick={() => press(k)}
            className="grid h-7 place-items-center rounded-md text-[0.8rem] font-medium tabular-nums transition active:scale-95"
            style={{
              gridColumn: k.wide ? "span 2" : undefined,
              color: k.wide ? paper : ink,
              background: k.wide ? ink : `color-mix(in oklab, ${ink} ${k.strong ? 16 : 8}%, transparent)`,
            }}
          >
            {k.label}
          </button>
        ))}
      </div>
    </div>
  );
}
