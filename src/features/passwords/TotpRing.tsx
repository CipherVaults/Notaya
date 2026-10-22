import { useEffect, useState } from "react";
import { motion } from "motion/react";
import type { TotpCode } from "@/lib/types";

/** Shows a live 2FA code with a countdown ring. `fetchCode` is polled each second. */
export default function TotpRing({ fetchCode, onCopy }: { fetchCode: () => Promise<TotpCode>; onCopy?: () => void }) {
  const [code, setCode] = useState<TotpCode | null>(null);
  const [error, setError] = useState(false);

  useEffect(() => {
    let alive = true;
    const tick = () =>
      fetchCode()
        .then((c) => {
          if (alive) {
            setCode(c);
            setError(false);
          }
        })
        .catch(() => alive && setError(true));
    tick();
    const t = setInterval(tick, 1000);
    return () => {
      alive = false;
      clearInterval(t);
    };
  }, [fetchCode]);

  if (error) return <span className="text-sm text-danger">Invalid 2FA secret</span>;
  if (!code) return <span className="text-sm text-faint">…</span>;

  const r = 11;
  const circ = 2 * Math.PI * r;
  const frac = code.remaining / code.period;
  const urgent = code.remaining <= 5;
  const pretty = code.code.length === 6 ? `${code.code.slice(0, 3)} ${code.code.slice(3)}` : code.code;

  return (
    <button onClick={onCopy} className="group flex items-center gap-3" title="Copy code">
      <motion.span
        key={code.code}
        initial={{ opacity: 0, y: 4 }}
        animate={{ opacity: 1, y: 0 }}
        className={`font-mono text-2xl font-semibold tracking-wider ${urgent ? "text-danger" : "text-fg"} group-hover:text-accent`}
      >
        {pretty}
      </motion.span>
      <svg width="28" height="28" viewBox="0 0 28 28" className="-rotate-90">
        <circle cx="14" cy="14" r={r} stroke="var(--surface-2)" strokeWidth="3" fill="none" />
        <circle
          cx="14"
          cy="14"
          r={r}
          stroke={urgent ? "#f43f5e" : "var(--accent)"}
          strokeWidth="3"
          fill="none"
          strokeLinecap="round"
          strokeDasharray={circ}
          strokeDashoffset={circ * (1 - frac)}
          style={{ transition: "stroke-dashoffset 1s linear" }}
        />
      </svg>
      <span className="text-xs tabular-nums text-faint">{code.remaining}s</span>
    </button>
  );
}
