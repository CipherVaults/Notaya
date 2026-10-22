import type { Settings } from "@/lib/types";

/** Animated backgrounds for the lock screen. Pure CSS, GPU-friendly. */
export default function LockBackdrop({ kind }: { kind: Settings["unlockBackground"] }) {
  if (kind === "plain") return null;
  if (kind === "midnight") {
    return (
      <div className="pointer-events-none absolute inset-0 overflow-hidden">
        <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_top,color-mix(in_oklab,var(--accent)_22%,transparent),transparent_60%)]" />
        <div className="grid-fade absolute inset-0 opacity-60" />
      </div>
    );
  }
  if (kind === "mesh") {
    return (
      <div className="pointer-events-none absolute inset-0 overflow-hidden">
        <div className="absolute inset-0 bg-[radial-gradient(at_20%_20%,color-mix(in_oklab,var(--accent)_30%,transparent)_0,transparent_45%),radial-gradient(at_80%_30%,color-mix(in_oklab,var(--text)_14%,transparent)_0,transparent_45%),radial-gradient(at_50%_90%,color-mix(in_oklab,var(--text)_10%,transparent)_0,transparent_50%)]" />
      </div>
    );
  }
  return (
    <div className="pointer-events-none absolute inset-0 overflow-hidden">
      <div className="aurora-blob drift-a left-[8%] top-[10%] h-[42vh] w-[42vh]" style={{ background: "var(--accent)", opacity: 0.2 }} />
      <div className="aurora-blob drift-b right-[6%] top-[22%] h-[38vh] w-[38vh]" style={{ background: "var(--text)", opacity: 0.16 }} />
      <div className="aurora-blob drift-c bottom-[2%] left-[34%] h-[34vh] w-[34vh]" style={{ background: "var(--accent-strong)", opacity: 0.14 }} />
      <div className="grid-fade absolute inset-0 opacity-40" />
    </div>
  );
}
