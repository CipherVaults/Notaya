import { useEffect, useRef, useState } from "react";
import { AnimatePresence, motion, useAnimationControls } from "motion/react";
import { ArrowRight, Check, Copy, FileKey2, Loader2, ShieldCheck, Sparkles } from "lucide-react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import { cachedAppearance } from "@/lib/theme";
import type { Strength } from "@/lib/types";
import { useApp } from "@/store/app";
import { Button, StrengthMeter } from "@/components/ui";
import Logo from "@/components/Logo";
import LockBackdrop from "./Backdrop";
import { copyText, KeyfileChip, PasswordInput } from "./fields";
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
        {status.exists ? <UnlockCard keyfileRequired={status.keyfileRequired} /> : <CreateCard />}
        <p className="mt-6 text-center text-xs text-faint">
          Encrypted with XChaCha20-Poly1305 · Argon2id · Stored only on this drive
        </p>
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
          <p className="mt-1 text-sm text-muted">Enter your master password to unlock your vault.</p>
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

/** Shows a suggested passphrase in plain sight so it can be read, written down or copied. */
function SuggestedPassphrase({ value }: { value: string }) {
  const [copyState, setCopyState] = useState<"idle" | "copied" | "failed">("idle");
  const timer = useRef<ReturnType<typeof setTimeout>>(undefined);
  const copied = copyState === "copied";

  useEffect(() => () => clearTimeout(timer.current), []);
  // A new suggestion starts fresh.
  useEffect(() => {
    clearTimeout(timer.current);
    setCopyState("idle");
  }, [value]);

  const copy = async () => {
    const ok = await copyText(value);
    setCopyState(ok ? "copied" : "failed");
    clearTimeout(timer.current);
    timer.current = setTimeout(() => setCopyState("idle"), ok ? 2000 : 3000);
  };

  return (
    <div className="space-y-2 rounded-2xl border border-accent/40 bg-accent-soft p-4">
      <div className="text-xs font-medium uppercase tracking-wider text-faint">Your suggested passphrase</div>
      <div className="flex items-start gap-3">
        {/* May wrap only after a hyphen. The <wbr /> adds no characters, so copying and selecting give the exact phrase. */}
        <div
          className="selectable min-w-0 flex-1 font-mono text-lg font-semibold leading-snug text-fg"
          style={{ overflowWrap: "normal", wordBreak: "normal" }}
        >
          {value.split("-").map((part, i, all) => (
            <span key={i}>
              {part}
              {i < all.length - 1 && (
                <>
                  -<wbr />
                </>
              )}
            </span>
          ))}
        </div>
        <Button variant="outline" size="sm" onClick={copy} className="shrink-0">
          {copied ? <Check size={14} /> : <Copy size={14} />} {copied ? "Copied" : "Copy"}
        </Button>
      </div>
      <span role="status" aria-live="polite" className={copyState === "failed" ? "block text-xs text-danger" : "sr-only"}>
        {copied ? "Copied to the clipboard" : copyState === "failed" ? "Couldn't copy - select the text and press Ctrl+C" : ""}
      </span>
      <p className="text-sm leading-relaxed text-fg">
        Write this down on paper or save it in a password manager. Then type it again in the second box.
      </p>
      <p className="text-xs leading-relaxed text-muted">
        Copying puts it on the clipboard. Windows may keep clipboard history (Win+V), so clear it after you have pasted it.
      </p>
    </div>
  );
}

function CreateCard() {
  const [step, setStep] = useState<0 | 1>(0);
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [strength, setStrength] = useState<Strength | null>(null);
  const [keyfile, setKeyfile] = useState<string | null>(null);
  const [understood, setUnderstood] = useState(false);
  const [busy, setBusy] = useState(false);
  // The suggested passphrase, shown so it can be read and copied. Kept in this component only.
  const [suggested, setSuggested] = useState<string | null>(null);
  const [restoreOpen, setRestoreOpen] = useState(false);
  const refreshStatus = useApp((s) => s.refreshStatus);
  const seq = useRef(0);

  useEffect(() => {
    const id = ++seq.current;
    if (!password) {
      setStrength(null);
      return;
    }
    const t = setTimeout(async () => {
      const s = await api.passwordStrength(password, ["cipherleaf"]);
      if (id === seq.current) setStrength(s);
    }, 120);
    return () => clearTimeout(t);
  }, [password]);

  const strongEnough = password.length >= 10 && (strength?.score ?? 0) >= 3;
  const matches = password === confirm;

  const suggest = async () => {
    const g = await api.generatePassphrase({ words: 6, separator: "-", capitalize: true, includeNumber: true });
    setPassword(g.value);
    setConfirm("");
    setSuggested(g.value);
    toast.info("Suggested a passphrase. It is shown below; write it down first.");
  };

  const changePassword = (v: string) => {
    setPassword(v);
    // Hide the box as soon as the password is edited to something else.
    if (v !== suggested) setSuggested(null);
  };

  const create = async () => {
    setBusy(true);
    try {
      await api.createVault(password, keyfile);
      setPassword("");
      setConfirm("");
      setSuggested(null);
      await refreshStatus();
      toast.success("Your vault is ready.");
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
        <h1 className="text-2xl font-semibold tracking-tight">Create your vault</h1>
        <p className="mt-1 text-sm text-muted">
          {step === 0
            ? "Choose one strong master password. It is the only key to everything inside."
            : "Optional: add a keyfile as a second factor."}
        </p>
      </div>

      <AnimatePresence mode="wait" initial={false}>
        {step === 0 ? (
          <motion.div
            key="pw"
            initial={{ opacity: 0, x: -16 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -16 }}
            className="space-y-3"
          >
            <PasswordInput value={password} onChange={changePassword} placeholder="Master password" autoFocus />
            {strength && <StrengthMeter score={strength.score} label={`~${strength.crackTime} to crack`} />}
            {strength?.warning && <p className="text-xs text-warn">{strength.warning}</p>}
            <PasswordInput
              value={confirm}
              onChange={setConfirm}
              placeholder="Type it again"
              onEnter={() => strongEnough && matches && setStep(1)}
            />
            {confirm && !matches && <p className="text-xs text-danger">The passwords don't match yet.</p>}
            <button onClick={suggest} className="flex items-center gap-1.5 text-xs font-medium text-accent hover:underline">
              <Sparkles size={13} /> Suggest a strong passphrase
            </button>
            {suggested && <SuggestedPassphrase value={suggested} />}
            <Button
              variant="primary"
              size="lg"
              className="w-full"
              disabled={!strongEnough || !matches}
              onClick={() => setStep(1)}
            >
              Continue <ArrowRight size={17} />
            </Button>
            {!strongEnough && password && (
              <p className="text-center text-xs text-faint">Use at least 10 characters and reach "Strong".</p>
            )}
            <div className="text-center">
              <button
                type="button"
                onClick={() => setRestoreOpen(true)}
                className="rounded-md px-2 py-1 text-sm font-medium text-fg underline-offset-4 hover:underline focus-visible:underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-soft"
              >
                Already have a backup? Restore it
              </button>
            </div>
          </motion.div>
        ) : (
          <motion.div
            key="kf"
            initial={{ opacity: 0, x: 16 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: 16 }}
            className="space-y-4"
          >
            <div className="rounded-2xl border border-line bg-surface p-4 text-sm leading-relaxed text-muted">
              <div className="mb-2 flex items-center gap-2 font-medium text-fg">
                <FileKey2 size={16} className="text-accent" /> Keyfile (recommended for maximum security)
              </div>
              With a keyfile, someone who steals your vault <em>and</em> learns your password still can't open it
              without the file. Keep it on a different drive than the vault.
            </div>
            {keyfile ? (
              <KeyfileChip path={keyfile} onClear={() => setKeyfile(null)} />
            ) : (
              <div className="grid grid-cols-2 gap-2">
                <Button variant="outline" onClick={async () => setKeyfile(await api.createKeyfile().catch(() => null))}>
                  Create new
                </Button>
                <Button variant="outline" onClick={async () => setKeyfile(await api.pickKeyfile())}>
                  Use existing
                </Button>
              </div>
            )}
            <label className="flex cursor-pointer items-start gap-3 rounded-2xl border border-warn/30 bg-warn/5 p-4 text-sm leading-relaxed">
              <input
                type="checkbox"
                checked={understood}
                onChange={(e) => setUnderstood(e.target.checked)}
                className="mt-1 h-4 w-4 accent-[var(--accent)]"
              />
              <span className="text-muted">
                I understand there is <strong className="text-fg">no password reset</strong>. If I forget my master
                password{keyfile ? " or lose my keyfile" : ""}, my data cannot be recovered by anyone.
              </span>
            </label>
            <div className="flex gap-2">
              <Button variant="ghost" onClick={() => setStep(0)}>
                Back
              </Button>
              <Button variant="primary" size="lg" className="flex-1" disabled={!understood || busy} onClick={create}>
                {busy ? (
                  <>
                    <Loader2 size={17} className="animate-spin" /> Creating vault…
                  </>
                ) : (
                  <>
                    <ShieldCheck size={17} /> Create vault
                  </>
                )}
              </Button>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
      <RestoreDialog open={restoreOpen} onOpenChange={setRestoreOpen} vaultExists={false} />
    </Card>
  );
}
