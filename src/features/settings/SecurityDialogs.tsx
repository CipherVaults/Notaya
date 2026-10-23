import { useEffect, useRef, useState } from "react";
import { FileKey2, Lock, LockOpen, Sparkles, TriangleAlert } from "lucide-react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import type { Strength } from "@/lib/types";
import { useApp } from "@/store/app";
import { Button, Modal, StrengthMeter } from "@/components/ui";
import { KeyfileChip, PasswordInput, SuggestedPassphrase } from "@/features/unlock/fields";

/** Turn security on: choose a master password (and an optional keyfile). The notes are not re-encrypted. */
export function EnableSecurityDialog({ open, onOpenChange }: { open: boolean; onOpenChange: (v: boolean) => void }) {
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [strength, setStrength] = useState<Strength | null>(null);
  const [keyfile, setKeyfile] = useState<string | null>(null);
  const [understood, setUnderstood] = useState(false);
  const [removeCopies, setRemoveCopies] = useState(true);
  const [suggested, setSuggested] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const refreshStatus = useApp((s) => s.refreshStatus);
  const setSettings = useApp((s) => s.setSettings);
  const seq = useRef(0);

  // Wipe everything when the dialog closes, so no password stays in memory.
  useEffect(() => {
    if (open) return;
    setPassword("");
    setConfirm("");
    setStrength(null);
    setKeyfile(null);
    setUnderstood(false);
    setRemoveCopies(true);
    setSuggested(null);
    setBusy(false);
  }, [open]);

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
  const ready = strongEnough && matches && understood && !busy;

  const suggest = async () => {
    const g = await api.generatePassphrase({ words: 6, separator: "-", capitalize: true, includeNumber: true });
    setPassword(g.value);
    setConfirm("");
    setSuggested(g.value);
  };

  const changePassword = (v: string) => {
    setPassword(v);
    if (v !== suggested) setSuggested(null);
  };

  const submit = async () => {
    if (!ready) return;
    setBusy(true);
    try {
      const result = await api.enableSecurity(password, keyfile, removeCopies);
      setSettings(await api.getSettings());
      await refreshStatus();
      toast.success("Security is on.", {
        description:
          result.removedCopies > 0
            ? `${result.removedCopies} earlier unprotected ${result.removedCopies === 1 ? "copy was" : "copies were"} deleted.`
            : undefined,
      });
      onOpenChange(false);
    } catch (e) {
      toast.error(errorText(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Modal
      open={open}
      onOpenChange={(o) => {
        if (!o && busy) return;
        onOpenChange(o);
      }}
      title="Turn on security"
      description="Your notes are kept. They get a new key and a master password; this takes a moment. The security options then appear in Settings."
      width={520}
    >
      <div className="space-y-4">
        <div className="space-y-3">
          <PasswordInput value={password} onChange={changePassword} placeholder="Master password" autoFocus />
          {strength && <StrengthMeter score={strength.score} label={`~${strength.crackTime} to crack`} />}
          {strength?.warning && <p className="text-xs text-warn">{strength.warning}</p>}
          <PasswordInput value={confirm} onChange={setConfirm} placeholder="Type it again" onEnter={submit} />
          {confirm && !matches && <p className="text-xs text-danger">The passwords don't match yet.</p>}
          <button type="button" onClick={suggest} className="flex items-center gap-1.5 text-xs font-medium text-accent hover:underline">
            <Sparkles size={13} /> Suggest a strong passphrase
          </button>
          {suggested && <SuggestedPassphrase value={suggested} />}
          {!strongEnough && password && (
            <p className="text-xs text-faint">Use at least 10 characters and reach "Strong".</p>
          )}
        </div>

        <div className="space-y-2 rounded-2xl border border-line bg-surface p-4 text-sm leading-relaxed text-muted">
          <div className="flex items-center gap-2 font-medium text-fg">
            <FileKey2 size={16} className="text-accent" /> Keyfile (optional, a second factor)
          </div>
          <p>Someone who has your notes file and learns your password still can't open it without the keyfile. Keep it on a different drive.</p>
          {keyfile ? (
            <KeyfileChip path={keyfile} onClear={() => setKeyfile(null)} />
          ) : (
            <div className="grid grid-cols-2 gap-2">
              <Button type="button" variant="outline" size="sm" onClick={async () => setKeyfile(await api.createKeyfile().catch(() => null))}>
                Create new
              </Button>
              <Button type="button" variant="outline" size="sm" onClick={async () => setKeyfile(await api.pickKeyfile())}>
                Use existing
              </Button>
            </div>
          )}
        </div>

        <label className="flex cursor-pointer items-start gap-3 rounded-2xl border border-line p-4 text-sm leading-relaxed">
          <input
            type="checkbox"
            checked={removeCopies}
            onChange={(e) => setRemoveCopies(e.target.checked)}
            className="mt-1 h-4 w-4 accent-[var(--accent)]"
          />
          <span className="text-muted">
            <strong className="text-fg">Delete earlier backups made while security was off.</strong> Anyone with the
            folder can open those without a password. A new backup under your new password is saved first. Old vaults that a restore kept (restore-undo files) are never deleted.
          </span>
        </label>

        <label className="flex cursor-pointer items-start gap-3 rounded-2xl border border-warn/30 bg-warn/5 p-4 text-sm leading-relaxed">
          <input
            type="checkbox"
            checked={understood}
            onChange={(e) => setUnderstood(e.target.checked)}
            className="mt-1 h-4 w-4 accent-[var(--accent)]"
          />
          <span className="text-muted">
            I understand there is <strong className="text-fg">no password reset</strong>. If I forget my master
            password{keyfile ? " or lose my keyfile" : ""}, my notes cannot be recovered by anyone.
          </span>
        </label>

        <div className="flex justify-end gap-2 pt-1">
          <Button type="button" variant="ghost" onClick={() => onOpenChange(false)} disabled={busy}>
            Cancel
          </Button>
          <Button type="button" variant="primary" onClick={submit} disabled={!ready}>
            <Lock size={14} /> {busy ? "Turning on…" : "Turn on security"}
          </Button>
        </div>
      </div>
    </Modal>
  );
}

/** Turn security off: needs the current master password (and keyfile). Shows what changes first. */
export function DisableSecurityDialog({
  open,
  onOpenChange,
  keyfileUsed,
}: {
  open: boolean;
  onOpenChange: (v: boolean) => void;
  keyfileUsed: boolean;
}) {
  const [current, setCurrent] = useState("");
  const [keyfile, setKeyfile] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const refreshStatus = useApp((s) => s.refreshStatus);
  const setSettings = useApp((s) => s.setSettings);

  useEffect(() => {
    if (open) return;
    setCurrent("");
    setKeyfile(null);
    setBusy(false);
    setError(null);
  }, [open]);

  const submit = async () => {
    if (!current || busy) return;
    setBusy(true);
    setError(null);
    try {
      await api.disableSecurity(current, keyfile);
      setSettings(await api.getSettings());
      await refreshStatus();
      toast.success("Security is off. Your notes open without a password.");
      onOpenChange(false);
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Modal
      open={open}
      onOpenChange={(o) => {
        if (!o && busy) return;
        onOpenChange(o);
      }}
      title="Turn off security"
      description="Your notes are kept. Cipherleaf will open them without asking for a password."
      width={500}
    >
      <div className="space-y-4">
        <div className="flex items-start gap-2.5 rounded-xl border border-warn/30 bg-warn/5 p-3.5 text-sm leading-relaxed text-muted">
          <TriangleAlert size={16} className="mt-0.5 shrink-0 text-warn" />
          <ul className="list-disc space-y-1 pl-4">
            <li>Anyone who can open this folder, or copy the notes file, will be able to read your notes.</li>
            <li>Nothing locks automatically any more, and screenshots are no longer blocked.</li>
            <li>Earlier backups keep the password they were made with. New backups will have no password.</li>
            <li>Your KeePassXC passwords file is not changed. It keeps its own master password.</li>
          </ul>
        </div>
        <PasswordInput value={current} onChange={setCurrent} placeholder="Current master password" autoFocus onEnter={submit} />
        {keyfileUsed &&
          (keyfile ? (
            <KeyfileChip path={keyfile} onClear={() => setKeyfile(null)} />
          ) : (
            <Button type="button" variant="outline" size="sm" onClick={async () => setKeyfile(await api.pickKeyfile())}>
              <FileKey2 size={14} /> Choose your keyfile
            </Button>
          ))}
        {error && (
          <p role="alert" className="text-sm text-danger">
            {error}
          </p>
        )}
        <div className="flex justify-end gap-2 pt-1">
          <Button type="button" variant="ghost" onClick={() => onOpenChange(false)} disabled={busy}>
            Cancel
          </Button>
          <Button type="button" variant="primary" onClick={submit} disabled={!current || busy}>
            <LockOpen size={14} /> {busy ? "Turning off…" : "Turn off security"}
          </Button>
        </div>
      </div>
    </Modal>
  );
}
