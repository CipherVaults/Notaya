import { useEffect, useRef, useState } from "react";
import { CheckCircle2, FileKey2, FolderOpen, Loader2, RotateCcw, TriangleAlert } from "lucide-react";
import { api, errorText } from "@/lib/ipc";
import type { BackupInfo } from "@/lib/types";
import { cn, formatDate } from "@/lib/utils";
import { Button, Modal } from "@/components/ui";
import { KeyfileChip, PasswordInput } from "./fields";

const noop = () => {};
const fileName = (p: string) => p.split(/[\\/]/).pop() ?? p;

function formatSize(bytes: number): string {
  if (bytes < 1024 * 1024) return `${Math.max(1, Math.round(bytes / 1024))} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

/**
 * Restore the vault from a backup file. Shared by the lock screen, the first-run screen and Settings.
 * The password lives only in this component's state and is cleared on close and on success.
 *
 * In "passwords" mode (Settings only, while unlocked) it restores the KeePassXC passwords file instead.
 * That needs no password here: Cipherleaf never opens that file.
 */
export default function RestoreDialog({
  open,
  onOpenChange,
  vaultExists,
  mode = "vault",
}: {
  open: boolean;
  onOpenChange: (o: boolean) => void;
  vaultExists: boolean;
  mode?: "vault" | "passwords";
}) {
  const isPasswords = mode === "passwords";
  const [backups, setBackups] = useState<BackupInfo[] | null>(null);
  const [listError, setListError] = useState<string | null>(null);
  const [source, setSource] = useState<string | null>(null);
  const [pickedPath, setPickedPath] = useState<string | null>(null);
  const [password, setPassword] = useState("");
  const [keyfile, setKeyfile] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [done, setDone] = useState<{ undoFile: string | null } | null>(null);
  const reloadTimer = useRef<ReturnType<typeof setTimeout>>(undefined);
  const passwordRef = useRef<HTMLInputElement>(null);

  // Move to the password box once a backup is chosen with the mouse or the file dialog.
  // (Arrow keys on the list keep their focus there, so the list stays usable from the keyboard.)
  const focusPassword = () => requestAnimationFrame(() => passwordRef.current?.focus());

  // Never leave the page-reload timer running after this dialog goes away.
  useEffect(() => () => clearTimeout(reloadTimer.current), []);

  // Load the automatic backups each time the dialog opens; wipe everything when it closes.
  useEffect(() => {
    if (!open) {
      setDone(null);
      setBackups(null);
      setListError(null);
      setSource(null);
      setPickedPath(null);
      setPassword("");
      setKeyfile(null);
      setBusy(false);
      setError(null);
      return;
    }
    let cancelled = false;
    (isPasswords ? api.listPasswordBackups() : api.listBackups())
      .then((list) => !cancelled && setBackups(list))
      .catch((e) => {
        if (cancelled) return;
        setBackups([]);
        setListError(errorText(e));
      });
    return () => {
      cancelled = true;
    };
  }, [open, isPasswords]);

  // The password may stay empty: a backup made while security was off needs none.
  const canRestore = !!source && !busy && !done;

  const restore = async () => {
    if (!source || busy || done) return;
    setBusy(true);
    setError(null);
    try {
      if (isPasswords) {
        // No page reload here: Cipherleaf stays unlocked and the person goes on to Passwords.
        setDone(await api.restorePasswords(source));
        return;
      }
      const result = await api.restoreVault(source, password, keyfile);
      setPassword("");
      setDone(result);
      // The backend has unlocked the restored vault. A reload makes the screen start fresh from it.
      reloadTimer.current = setTimeout(() => window.location.reload(), 1200);
    } catch (e) {
      setError(errorText(e));
      focusPassword();
    } finally {
      setBusy(false);
    }
  };

  const chooseFile = async () => {
    setError(null);
    try {
      const path = await (isPasswords ? api.pickPasswordsBackupFile() : api.pickBackupFile());
      if (path) {
        setPickedPath(path);
        setSource(path);
        if (!isPasswords) focusPassword();
      }
    } catch (e) {
      setError(errorText(e));
    }
  };

  const chooseKeyfile = async () => {
    setError(null);
    try {
      const path = await api.pickKeyfile();
      if (path) setKeyfile(path);
    } catch (e) {
      setError(errorText(e));
    }
  };

  return (
    <Modal
      open={open}
      onOpenChange={(o) => {
        // Don't close halfway through a restore. (The vault restore reloads the page by itself when done.)
        if (!o && (busy || (done && !isPasswords))) return;
        onOpenChange(o);
      }}
      title={isPasswords ? "Restore the passwords file" : "Restore from a backup"}
      description={
        isPasswords
          ? "Go back to an earlier copy of your KeePassXC passwords."
          : "Go back to an earlier copy of your vault."
      }
      width={520}
    >
      <div className="space-y-4">
        <fieldset className="space-y-2" disabled={busy || !!done}>
          <legend className="mb-1.5 text-xs font-medium uppercase tracking-wider text-faint">Choose a backup</legend>

          {backups === null && (
            <p className="flex items-center gap-2 text-sm text-muted">
              <Loader2 size={14} className="animate-spin" /> Looking for backups…
            </p>
          )}
          {backups?.length === 0 && !pickedPath && (
            <p className="rounded-xl border border-line bg-surface px-3.5 py-3 text-sm text-muted">
              {listError ??
                (isPasswords
                  ? "No automatic passwords backups found yet. They are made when the vault locks and the passwords file has changed."
                  : "No automatic backups found in the data folder.")}
            </p>
          )}

          {(pickedPath || (backups && backups.length > 0)) && (
            <div className="max-h-52 space-y-1.5 overflow-y-auto pr-0.5">
              {pickedPath && (
                <BackupRow
                  checked={source === pickedPath}
                  onSelect={() => setSource(pickedPath)}
                  onMouseChosen={isPasswords ? noop : focusPassword}
                  title={fileName(pickedPath)}
                  detail="A file you chose"
                  tooltip={pickedPath}
                />
              )}
              {backups?.map((b) => (
                <BackupRow
                  key={b.name}
                  checked={source === b.name}
                  onSelect={() => setSource(b.name)}
                  onMouseChosen={isPasswords ? noop : focusPassword}
                  title={formatDate(b.modifiedMs)}
                  detail={formatSize(b.size)}
                  tooltip={b.name}
                />
              ))}
            </div>
          )}

          <Button type="button" variant="outline" size="sm" onClick={chooseFile} className="disabled:opacity-70">
            <FolderOpen size={14} /> {isPasswords ? "Choose another file…" : "Choose another backup file…"}
          </Button>
        </fieldset>

        {isPasswords && (
          <div className="flex items-start gap-2.5 rounded-xl border border-warn/30 bg-warn/5 p-3.5 text-sm leading-relaxed text-muted">
            <TriangleAlert size={16} className="mt-0.5 shrink-0 text-warn" />
            <span>
              This replaces your KeePassXC passwords file with the backup. KeePassXC will be closed first, so save
              anything you are doing there. The backup opens with the KeePassXC master password it had when it was
              made. Your current file is not deleted: it is kept in the data folder in a file whose name starts with
              "passwords-undo".
            </span>
          </div>
        )}

        {!isPasswords && vaultExists && (
          <div className="flex items-start gap-2.5 rounded-xl border border-warn/30 bg-warn/5 p-3.5 text-sm leading-relaxed text-muted">
            <TriangleAlert size={16} className="mt-0.5 shrink-0 text-warn" />
            <span>
              This replaces your current vault with the backup. Your current vault is not deleted. It is kept in the
              data folder in a file whose name starts with "restore-undo".
            </span>
          </div>
        )}

        {!isPasswords && (
        <div className="space-y-2">
          <div className="text-xs font-medium uppercase tracking-wider text-faint">Password of this backup (if it has one)</div>
          <PasswordInput
            value={password}
            onChange={setPassword}
            placeholder="Password of this backup"
            inputRef={passwordRef}
            onEnter={restore}
          />
          {keyfile ? (
            <KeyfileChip path={keyfile} onClear={() => setKeyfile(null)} disabled={busy || !!done} />
          ) : (
            <Button type="button" variant="outline" size="sm" onClick={chooseKeyfile} disabled={busy || !!done} className="disabled:opacity-70">
              <FileKey2 size={14} /> Add keyfile…
            </Button>
          )}
          <p className="text-xs leading-relaxed text-faint">
            Use the master password (and keyfile) the backup had when it was made. If you changed your password since
            then, use the old one. A backup made while security was off has no password: leave the box empty, and your
            notes will open without a password afterwards (you can turn security on again in Settings).
          </p>
        </div>
        )}

        {error && (
          <p role="alert" className="text-sm text-danger">
            {error}
          </p>
        )}

        {done && isPasswords && (
          <p
            role="status"
            className="flex items-start gap-2.5 rounded-xl border border-ok/50 bg-ok/10 p-3.5 text-sm font-medium leading-relaxed text-fg"
          >
            <CheckCircle2 size={18} className="mt-0.5 shrink-0 text-ok" />
            <span>
              Done.{" "}
              {done.undoFile ? (
                <>
                  Your previous passwords file is kept as <span className="font-mono text-xs">{done.undoFile}</span>.
                </>
              ) : (
                "There was no earlier passwords file to keep."
              )}{" "}
              Open Passwords to use the restored one.
            </span>
          </p>
        )}

        {done && !isPasswords && (
          <p
            role="status"
            className="flex items-start gap-2.5 rounded-xl border border-ok/50 bg-ok/10 p-3.5 text-sm font-medium leading-relaxed text-fg"
          >
            <CheckCircle2 size={18} className="mt-0.5 shrink-0 text-ok" />
            <span>
              Your backup is restored.
              {done.undoFile && (
                <>
                  {" "}Your old vault was kept as <span className="font-mono text-xs">{done.undoFile}</span>.
                </>
              )}{" "}
              Opening it now…
            </span>
          </p>
        )}

        <div className="flex justify-end gap-2 pt-1">
          {isPasswords && done ? (
            <Button type="button" variant="primary" onClick={() => onOpenChange(false)}>
              Close
            </Button>
          ) : (
            <>
              <Button type="button" variant="ghost" onClick={() => onOpenChange(false)} disabled={busy || !!done} className="disabled:opacity-70">
                Cancel
              </Button>
              <Button type="button" variant="primary" onClick={restore} disabled={!canRestore}>
                {busy ? (
                  <>
                    <Loader2 size={14} className="animate-spin" /> Restoring…
                  </>
                ) : (
                  <>
                    <RotateCcw size={14} />{" "}
                    {isPasswords
                      ? "Replace passwords file"
                      : vaultExists
                        ? "Replace my vault with this backup"
                        : "Restore backup"}
                  </>
                )}
              </Button>
            </>
          )}
        </div>
      </div>
    </Modal>
  );
}

function BackupRow({
  checked,
  onSelect,
  onMouseChosen,
  title,
  detail,
  tooltip,
}: {
  checked: boolean;
  onSelect: () => void;
  onMouseChosen: () => void;
  title: string;
  detail: string;
  tooltip: string;
}) {
  return (
    <label
      title={tooltip}
      // detail > 0 means a real mouse click (keyboard-made clicks have detail 0).
      onClick={(e) => e.detail > 0 && onMouseChosen()}
      className={cn(
        "flex cursor-pointer items-center gap-3 rounded-xl border px-3.5 py-2.5 text-sm transition-colors",
        "has-[:focus-visible]:ring-2 has-[:focus-visible]:ring-accent-soft",
        checked ? "border-accent bg-accent-soft" : "border-line hover:bg-surface",
      )}
    >
      <input
        type="radio"
        name="restore-source"
        checked={checked}
        onChange={onSelect}
        className="h-4 w-4 shrink-0 accent-[var(--accent)]"
      />
      <span className="min-w-0 flex-1 truncate">{title}</span>
      <span className="shrink-0 text-xs text-muted">{detail}</span>
    </label>
  );
}
