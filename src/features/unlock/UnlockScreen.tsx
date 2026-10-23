import { useState } from "react";
import { AnimatePresence, motion, useAnimationControls } from "motion/react";
import { ArrowRight, FileKey2, Loader2, NotebookPen, TriangleAlert } from "lucide-react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import { cachedAppearance } from "@/lib/theme";
import { useApp } from "@/store/app";
import { Button } from "@/components/ui";
import Logo from "@/components/Logo";
import LockBackdrop from "./Backdrop";
import { KeyfileChip, PasswordInput } from "./fields";
import RestoreDialog from "./RestoreDialog";

export default function UnlockScreen() {
  const status = useApp((s) => s.status);
  const background = cachedAppearance().unlockBackground;
  if (!status) return null;
  return (
    <div className="relative flex h-full items-center justify-center overflow-hidden">
      <LockBackdrop kind={background} />
      <motion.div
        initial={{ opacity: 0, y: 18, scale: 0.97 }}
        animate={{ opacity: 1, y: 0, scale: 1 }}
        transition={{ duration: 0.5, ease: [0.22, 1, 0.36, 1] }}
        className="relative w-[440px] max-w-[92vw]"
      >
        {!status.exists ? (
          <StartCard />
        ) : status.securityOn ? (
          <UnlockCard keyfileRequired={status.keyfileRequired} />
        ) : (
          <ProblemCard message={status.openError} />
        )}
        {status.exists && status.securityOn && (
          <p className="mt-6 text-center text-xs text-faint">
            Encrypted with XChaCha20-Poly1305 · Argon2id · Stored only on this drive
          </p>
        )}
      </motion.div>
    </div>
  );
}

function Card({ children }: { children: React.ReactNode }) {
  return (
    <div className="rounded-3xl border border-line bg-elevated/80 p-8 shadow-float backdrop-blur-2xl">{children}</div>
  );
}

function UnlockCard({ keyfileRequired }: { keyfileRequired: boolean }) {
  const [password, setPassword] = useState("");
  const [keyfile, setKeyfile] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [restoreOpen, setRestoreOpen] = useState(false);
  const controls = useAnimationControls();
  const refreshStatus = useApp((s) => s.refreshStatus);

  const unlock = async () => {
    if (!password || busy) return;
    if (keyfileRequired && !keyfile) {
      setError("This vault needs its keyfile. Choose it below.");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      await api.unlockVault(password, keyfile);
      setPassword("");
      await refreshStatus();
    } catch (e) {
      setError(errorText(e));
      controls.start({ x: [0, -10, 10, -7, 7, -3, 0], transition: { duration: 0.45 } });
    } finally {
      setBusy(false);
    }
  };

  return (
    <motion.div animate={controls}>
      <Card>
        <div className="mb-7 flex flex-col items-center text-center">
          <motion.div
            initial={{ rotate: -12, scale: 0.8 }}
            animate={{ rotate: 0, scale: 1 }}
            transition={{ type: "spring", stiffness: 200, damping: 14 }}
            className="mb-4 grid h-16 w-16 place-items-center rounded-2xl bg-accent-soft"
          >
            <Logo size={34} />
          </motion.div>
          <h1 className="text-2xl font-semibold tracking-tight">Welcome back</h1>
          <p className="mt-1 text-sm text-muted">Enter your master password to unlock your notes.</p>
        </div>

        <div className="space-y-3">
          <PasswordInput value={password} onChange={setPassword} placeholder="Master password" autoFocus onEnter={unlock} />
          {keyfileRequired &&
            (keyfile ? (
              <KeyfileChip path={keyfile} onClear={() => setKeyfile(null)} />
            ) : (
              <Button
                variant="outline"
                className="w-full"
                onClick={async () => setKeyfile(await api.pickKeyfile())}
              >
                <FileKey2 size={15} /> Choose keyfile
              </Button>
            ))}
          <AnimatePresence>
            {error && (
              <motion.p
                initial={{ opacity: 0, height: 0 }}
                animate={{ opacity: 1, height: "auto" }}
                exit={{ opacity: 0, height: 0 }}
                className="text-sm text-danger"
              >
                {error}
              </motion.p>
            )}
          </AnimatePresence>
          <Button variant="primary" size="lg" className="w-full" onClick={unlock} disabled={!password || busy}>
            {busy ? (
              <>
                <Loader2 size={17} className="animate-spin" /> Deriving key…
              </>
            ) : (
              <>
                Unlock <ArrowRight size={17} />
              </>
            )}
          </Button>
          <div className="pt-1 text-center">
            <button
              type="button"
              onClick={() => setRestoreOpen(true)}
              className="rounded-md px-2 py-1 text-sm font-medium text-fg underline-offset-4 hover:underline focus-visible:underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-soft"
            >
              Restore from a backup…
            </button>
          </div>
        </div>
      </Card>
      <RestoreDialog open={restoreOpen} onOpenChange={setRestoreOpen} vaultExists />
    </motion.div>
  );
}

/** Shown when there is no vault file but earlier copies exist: never start empty without asking. */
function StartCard() {
  const [restoreOpen, setRestoreOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const refreshStatus = useApp((s) => s.refreshStatus);
  const earlierCopies = useApp((s) => s.status?.earlierCopies ?? false);
  const openError = useApp((s) => s.status?.openError ?? null);

  const start = async () => {
    setBusy(true);
    try {
      await api.startNotes();
      await refreshStatus();
    } catch (e) {
      toast.error(errorText(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Card>
      <div className="mb-6 flex flex-col items-center text-center">
        <div className="mb-4 grid h-16 w-16 place-items-center rounded-2xl bg-accent-soft">
          <Logo size={34} />
        </div>
        <h1 className="text-2xl font-semibold tracking-tight">{earlierCopies ? "No notes found here" : "Notes could not be started"}</h1>
        <p className="mt-1 text-sm text-muted">
          {earlierCopies
            ? "This folder has earlier backups but no current notes file. Restore a backup to get your notes back, or start new notes."
            : (openError ?? "A new notes file could not be created in this folder. Check that the folder can be written to.")}
        </p>
      </div>
      <div className="space-y-3">
        <Button variant="primary" size="lg" className="w-full" onClick={() => setRestoreOpen(true)}>
          Restore from a backup…
        </Button>
        <Button variant="outline" size="lg" className="w-full" onClick={start} disabled={busy}>
          {busy ? (
            <>
              <Loader2 size={17} className="animate-spin" /> Starting…
            </>
          ) : (
            <>
              <NotebookPen size={17} /> Start new notes
            </>
          )}
        </Button>
        <p className="text-center text-xs text-faint">Your backups are not deleted either way.</p>
      </div>
      <RestoreDialog open={restoreOpen} onOpenChange={setRestoreOpen} vaultExists={false} />
    </Card>
  );
}

/** A notes file that opens without a password could not be opened (for example it is damaged). */
function ProblemCard({ message }: { message: string | null }) {
  const [restoreOpen, setRestoreOpen] = useState(false);
  return (
    <Card>
      <div className="mb-6 flex flex-col items-center text-center">
        <div className="mb-4 grid h-16 w-16 place-items-center rounded-2xl bg-accent-soft">
          <TriangleAlert size={30} className="text-warn" />
        </div>
        <h1 className="text-2xl font-semibold tracking-tight">Your notes could not be opened</h1>
        <p className="mt-1 text-sm text-muted">{message ?? "The notes file could not be read."}</p>
      </div>
      <Button variant="primary" size="lg" className="w-full" onClick={() => setRestoreOpen(true)}>
        Restore from a backup…
      </Button>
      <RestoreDialog open={restoreOpen} onOpenChange={setRestoreOpen} vaultExists />
    </Card>
  );
}
