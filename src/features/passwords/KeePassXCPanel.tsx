import { useCallback, useEffect, useState } from "react";
import { motion } from "motion/react";
import { AlertTriangle, CheckCircle2, FolderOpen, KeyRound, Loader2, ShieldCheck, WifiOff } from "lucide-react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import type { KeePassXCStatus } from "@/lib/types";
import { Button } from "@/components/ui";

/**
 * Passwords live in the bundled, unmodified KeePassXC (amendment A9). This panel
 * only starts it and shows its state. Cipherleaf never sees KeePassXC's master
 * password or any entry.
 */
export default function KeePassXCPanel() {
  const [status, setStatus] = useState<KeePassXCStatus | null>(null);
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(() => api.keepassxcStatus().then(setStatus).catch(() => {}), []);

  useEffect(() => {
    refresh();
    const t = setInterval(refresh, 2000);
    return () => clearInterval(t);
  }, [refresh]);

  const open = async () => {
    setBusy(true);
    try {
      await api.keepassxcOpen();
      await refresh();
    } catch (e) {
      toast.error(errorText(e));
    } finally {
      setBusy(false);
    }
  };

  if (!status) {
    return (
      <div className="grid h-full place-items-center text-muted">
        <Loader2 className="animate-spin" size={20} />
      </div>
    );
  }

  const ready = status.installed && status.verified;
  const hasDb = status.database !== null;

  return (
    <div className="h-full overflow-y-auto">
      <div className="mx-auto max-w-2xl px-10 py-12">
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          className="relative overflow-hidden rounded-3xl border border-line bg-elevated p-8 shadow-float"
        >
          <div className="pointer-events-none absolute -right-16 -top-16 h-48 w-48 rounded-full opacity-25 blur-3xl" style={{ background: "var(--accent)" }} />
          <div className="relative flex items-start gap-5">
            <div className="grid h-14 w-14 shrink-0 place-items-center rounded-2xl bg-accent-soft text-accent">
              <KeyRound size={28} />
            </div>
            <div className="min-w-0 flex-1">
              <h2 className="text-2xl font-semibold tracking-tight">Passwords</h2>
              <p className="mt-1 text-sm leading-relaxed text-muted">
                Your passwords are kept in <strong className="text-fg">KeePassXC {status.version}</strong>, the well-known open-source
                password manager, bundled with Cipherleaf. It opens in its own window and uses the standard <code className="font-mono text-xs">.kdbx</code> format, so you
                can import, export and use all of its features.
              </p>
            </div>
          </div>

          <div className="relative mt-7">
            {!status.installed && (
              <Notice tone="warn" icon={<AlertTriangle size={17} />}>
                KeePassXC isn't included with this copy of Cipherleaf. Put the KeePassXC folder next to <code className="font-mono text-xs">Cipherleaf.exe</code>.
              </Notice>
            )}
            {status.installed && !status.verified && (
              <Notice tone="danger" icon={<AlertTriangle size={17} />}>
                The KeePassXC found here doesn't match the verified {status.version} release, so Cipherleaf won't start it. Replace it with the original files.
              </Notice>
            )}
            {ready && (
              <div className="space-y-4">
                <Button variant="primary" size="lg" className="w-full" onClick={open} disabled={busy}>
                  {busy ? (
                    <>
                      <Loader2 size={17} className="animate-spin" /> Starting KeePassXC…
                    </>
                  ) : hasDb ? (
                    <>
                      <KeyRound size={17} /> {status.running ? "Bring KeePassXC to the front" : "Open passwords in KeePassXC"}
                    </>
                  ) : (
                    <>
                      <KeyRound size={17} /> Create a password database
                    </>
                  )}
                </Button>

                {!hasDb && (
                  <ol className="list-decimal space-y-1.5 rounded-2xl border border-line bg-surface p-4 pl-9 text-sm leading-relaxed text-muted">
                    <li>
                      In KeePassXC choose <strong className="text-fg">Database → New Database</strong>.
                    </li>
                    <li>
                      Pick a strong master password. In the encryption settings, <strong className="text-fg">Argon2id</strong> is the safest key derivation.
                    </li>
                    <li>
                      Save it as <code className="font-mono text-xs">passwords.kdbx</code> in the folder KeePassXC suggests (Cipherleaf's <code className="font-mono text-xs">data</code> folder).
                    </li>
                  </ol>
                )}

                <div className="flex items-center gap-2 text-sm">
                  {status.running ? (
                    <>
                      <CheckCircle2 size={16} className="text-ok" /> <span className="text-muted">KeePassXC is open. It locks when Cipherleaf locks.</span>
                    </>
                  ) : (
                    <span className="text-faint">KeePassXC isn't running.</span>
                  )}
                </div>
              </div>
            )}
          </div>
        </motion.div>

        <div className="mt-5 grid gap-3 sm:grid-cols-2">
          <Fact icon={<ShieldCheck size={17} />} title="Verified copy">
            Cipherleaf starts KeePassXC only if the program file matches the official {status.version} release.
          </Fact>
          <Fact icon={<WifiOff size={17} />} title="Offline by default">
            Update checks, browser integration, the SSH agent and icon downloads are off until you turn them on in KeePassXC.
          </Fact>
          <Fact icon={<FolderOpen size={17} />} title="Stays in the portable folder">
            Your database and KeePassXC's settings live in Cipherleaf's <code className="font-mono text-xs">data</code> folder, not on the PC.
          </Fact>
          <Fact icon={<KeyRound size={17} />} title="Separate master password">
            KeePassXC has its own master password (and optional keyfile). Cipherleaf never sees it.
          </Fact>
        </div>

        {status.database && (
          <p className="selectable mt-5 break-all text-xs text-faint">Database: {status.database}</p>
        )}
      </div>
    </div>
  );
}

function Notice({ tone, icon, children }: { tone: "warn" | "danger"; icon: React.ReactNode; children: React.ReactNode }) {
  return (
    <div
      className={`flex items-start gap-3 rounded-2xl border p-4 text-sm leading-relaxed ${
        tone === "danger" ? "border-danger/40 bg-danger/10" : "border-warn/40 bg-warn/10"
      }`}
    >
      <span className={tone === "danger" ? "text-danger" : "text-warn"}>{icon}</span>
      <span>{children}</span>
    </div>
  );
}

function Fact({ icon, title, children }: { icon: React.ReactNode; title: string; children: React.ReactNode }) {
  return (
    <div className="rounded-2xl border border-line bg-surface p-4">
      <div className="mb-1.5 flex items-center gap-2 text-sm font-medium">
        <span className="text-accent">{icon}</span>
        {title}
      </div>
      <p className="text-xs leading-relaxed text-muted">{children}</p>
    </div>
  );
}
