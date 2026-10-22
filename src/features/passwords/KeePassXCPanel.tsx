import { useCallback, useEffect, useRef, useState } from "react";
import { AlertTriangle, ExternalLink, KeyRound, Loader2, RefreshCw, ShieldCheck, WifiOff } from "lucide-react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import type { KeePassXCStatus } from "@/lib/types";
import { useApp } from "@/store/app";
import { Button, IconButton } from "@/components/ui";

type Phase = "starting" | "embedded" | "closed" | "failed";

/**
 * Passwords live in the bundled, unmodified KeePassXC (amendments A9 and A10). Its
 * window is shown right here, over the host area below. Cipherleaf never sees the
 * master password or any entry; it only tells Windows where to put the window.
 */
export default function KeePassXCPanel() {
  const [status, setStatus] = useState<KeePassXCStatus | null>(null);
  const [phase, setPhase] = useState<Phase>("starting");
  const [error, setError] = useState<string | null>(null);
  const host = useRef<HTMLDivElement>(null);
  const paletteOpen = useApp((s) => s.paletteOpen);

  /** The host area in physical pixels, relative to the window. */
  const rect = useCallback(() => {
    const r = host.current?.getBoundingClientRect();
    if (!r) return null;
    const d = window.devicePixelRatio || 1;
    return {
      x: Math.round(r.left * d),
      y: Math.round(r.top * d),
      w: Math.max(1, Math.round(r.width * d)),
      h: Math.max(1, Math.round(r.height * d)),
    };
  }, []);

  const refresh = useCallback(() => api.keepassxcStatus().then(setStatus).catch(() => {}), []);

  const start = useCallback(async () => {
    const r = rect();
    if (!r) return;
    setPhase("starting");
    setError(null);
    try {
      await api.keepassxcEmbed(r.x, r.y, r.w, r.h);
      setPhase("embedded");
    } catch (e) {
      setError(errorText(e));
      setPhase("failed");
    }
    refresh();
  }, [rect, refresh]);

  // Show KeePassXC when this panel appears; hide it when the panel goes away.
  useEffect(() => {
    let cancelled = false;
    api.keepassxcStatus().then((s) => {
      if (cancelled) return;
      setStatus(s);
      if (s.installed && s.verified) start();
      else setPhase("failed");
    });
    return () => {
      cancelled = true;
      api.keepassxcHide().catch(() => {});
    };
  }, [start]);

  // Keep the window glued to the host area while the app is resized.
  useEffect(() => {
    if (phase !== "embedded" || paletteOpen) return;
    let frame = 0;
    const move = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        const r = rect();
        if (r) api.keepassxcPlace(r.x, r.y, r.w, r.h).catch(() => {});
      });
    };
    move();
    const ro = new ResizeObserver(move);
    if (host.current) ro.observe(host.current);
    window.addEventListener("resize", move);
    return () => {
      cancelAnimationFrame(frame);
      ro.disconnect();
      window.removeEventListener("resize", move);
    };
  }, [phase, paletteOpen, rect]);

  // The command palette is drawn by the web page, so KeePassXC must step aside for it.
  useEffect(() => {
    if (phase === "embedded" && paletteOpen) api.keepassxcHide().catch(() => {});
  }, [phase, paletteOpen]);

  // If KeePassXC is closed from inside (File → Quit), offer to open it again.
  useEffect(() => {
    if (phase !== "embedded") return;
    const t = setInterval(async () => {
      const s = await api.keepassxcStatus().catch(() => null);
      if (s) {
        setStatus(s);
        if (!s.embedded) setPhase("closed");
      }
    }, 2000);
    return () => clearInterval(t);
  }, [phase]);

  const separateWindow = async () => {
    try {
      await api.keepassxcOpen();
      setPhase("closed");
      refresh();
    } catch (e) {
      toast.error(errorText(e));
    }
  };

  const version = status?.version ?? "2.8.0";
  const unavailable = status && (!status.installed || !status.verified);

  return (
    <div className="flex h-full flex-col">
      <div className="flex h-12 shrink-0 items-center gap-3 border-b border-line px-5">
        <KeyRound size={17} className="text-muted" />
        <span className="text-sm font-semibold">Passwords</span>
        <span className="text-xs text-faint">KeePassXC {version}</span>
        <div className="flex-1" />
        {phase === "embedded" && (
          <IconButton label="Open in a separate window" size="sm" onClick={separateWindow}>
            <ExternalLink size={15} />
          </IconButton>
        )}
      </div>

      <div ref={host} data-kpxc-host="" className="relative min-h-0 flex-1">
        {/* KeePassXC's own window is drawn over this area by Windows. */}
        {phase === "starting" && !unavailable && (
          <div className="grid h-full place-items-center text-muted">
            <div className="flex items-center gap-2 text-sm">
              <Loader2 size={16} className="animate-spin" /> Starting KeePassXC…
            </div>
          </div>
        )}

        {unavailable && (
          <Message tone={status && !status.installed ? "warn" : "danger"} title={status && !status.installed ? "KeePassXC isn't included" : "KeePassXC doesn't match the verified release"}>
            {status && !status.installed
              ? "Put the KeePassXC folder next to Cipherleaf.exe."
              : `The KeePassXC found here isn't the verified ${version} release, so Cipherleaf won't start it. Replace it with the original files.`}
          </Message>
        )}

        {!unavailable && (phase === "failed" || phase === "closed") && (
          <div className="grid h-full place-items-center overflow-y-auto p-8">
            <div className="w-full max-w-md space-y-5 text-center">
              <div className="mx-auto grid h-14 w-14 place-items-center rounded-2xl bg-accent-soft text-accent">
                {phase === "failed" ? <AlertTriangle size={26} /> : <KeyRound size={26} />}
              </div>
              <div>
                <h2 className="text-lg font-semibold tracking-tight">{phase === "failed" ? "Couldn't show KeePassXC here" : "KeePassXC is closed"}</h2>
                <p className="mt-1.5 text-sm leading-relaxed text-muted">
                  {phase === "failed" ? (error ?? "KeePassXC didn't open in time.") : "Open it again to keep working with your passwords."}
                </p>
              </div>
              <div className="flex justify-center gap-2">
                <Button variant="primary" onClick={start}>
                  <RefreshCw size={15} /> {phase === "failed" ? "Try again" : "Open KeePassXC"}
                </Button>
                <Button variant="outline" onClick={separateWindow}>
                  <ExternalLink size={15} /> Separate window
                </Button>
              </div>
              <div className="grid gap-2 pt-2 text-left text-xs text-muted sm:grid-cols-2">
                <Hint icon={<ShieldCheck size={14} />}>Only a verified {version} copy is started.</Hint>
                <Hint icon={<WifiOff size={14} />}>Offline by default; your database stays in the data folder.</Hint>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function Message({ tone, title, children }: { tone: "warn" | "danger"; title: string; children: React.ReactNode }) {
  return (
    <div className="grid h-full place-items-center p-8">
      <div className={`max-w-md rounded-2xl border p-5 text-sm leading-relaxed ${tone === "danger" ? "border-danger/40 bg-danger/10" : "border-warn/40 bg-warn/10"}`}>
        <div className="mb-1 flex items-center gap-2 font-medium">
          <AlertTriangle size={16} className={tone === "danger" ? "text-danger" : "text-warn"} />
          {title}
        </div>
        <p className="text-muted">{children}</p>
      </div>
    </div>
  );
}

function Hint({ icon, children }: { icon: React.ReactNode; children: React.ReactNode }) {
  return (
    <div className="flex items-start gap-2 rounded-xl border border-line bg-surface p-3">
      <span className="mt-0.5 text-accent">{icon}</span>
      <span>{children}</span>
    </div>
  );
}
