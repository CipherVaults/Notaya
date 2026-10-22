import { useEffect, useRef, useState } from "react";
import { AnimatePresence, motion, useAnimationControls } from "motion/react";
import { ArrowRight, Eye, EyeOff, FileKey2, KeyRound, Loader2, ShieldCheck, Sparkles, X } from "lucide-react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import { cachedAppearance } from "@/lib/theme";
import type { Strength } from "@/lib/types";
import { useApp } from "@/store/app";
import { Button, StrengthMeter } from "@/components/ui";
import Logo from "@/components/Logo";
import LockBackdrop from "./Backdrop";

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

function PasswordInput({
  value,
  onChange,
  placeholder,
  autoFocus,
  onEnter,
}: {
  value: string;
  onChange: (v: string) => void;
  placeholder: string;
  autoFocus?: boolean;
  onEnter?: () => void;
}) {
  const [show, setShow] = useState(false);
  return (
    <div className="relative">
      <KeyRound size={16} className="pointer-events-none absolute left-3.5 top-1/2 -translate-y-1/2 text-faint" />
      <input
        type={show ? "text" : "password"}
        value={value}
        autoFocus={autoFocus}
        spellCheck={false}
        autoComplete="off"
        placeholder={placeholder}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={(e) => e.key === "Enter" && onEnter?.()}
        className="h-12 w-full rounded-2xl border border-line bg-surface pl-10 pr-11 text-[0.95rem] text-fg outline-none transition-all placeholder:text-faint focus:border-accent focus:ring-4 focus:ring-accent-soft"
      />
      <button
        type="button"
        tabIndex={-1}
        onClick={() => setShow(!show)}
        className="absolute right-2 top-1/2 -translate-y-1/2 rounded-lg p-2 text-faint hover:text-fg"
        aria-label={show ? "Hide password" : "Show password"}
      >
        {show ? <EyeOff size={16} /> : <Eye size={16} />}
      </button>
    </div>
  );
}

function KeyfileChip({ path, onClear }: { path: string; onClear: () => void }) {
  const name = path.split(/[\\/]/).pop();
  return (
    <div className="flex items-center gap-2 rounded-xl border border-accent/40 bg-accent-soft px-3 py-2 text-sm">
      <FileKey2 size={15} className="text-accent" />
      <span className="flex-1 truncate" title={path}>
        {name}
      </span>
      <button onClick={onClear} className="rounded p-0.5 text-muted hover:text-fg" aria-label="Remove keyfile">
        <X size={14} />
      </button>
    </div>
  );
}

function UnlockCard({ keyfileRequired }: { keyfileRequired: boolean }) {
  const [password, setPassword] = useState("");
  const [keyfile, setKeyfile] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
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
        </div>
      </Card>
    </motion.div>
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
    toast.info("Suggested a 6-word passphrase. Write it down somewhere safe, then type it again to confirm.");
  };

  const create = async () => {
    setBusy(true);
    try {
      await api.createVault(password, keyfile);
      setPassword("");
      setConfirm("");
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
            <PasswordInput value={password} onChange={setPassword} placeholder="Master password" autoFocus />
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
    </Card>
  );
}
