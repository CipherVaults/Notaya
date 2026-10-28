import { useEffect, useRef, useState } from "react";
import { CheckCircle2, FileKey2, FolderOpen, Loader2, RotateCcw, TriangleAlert } from "lucide-react";
import { api, errorText } from "@/lib/ipc";
import type { BackupInfo } from "@/lib/types";
import { cn, formatDate } from "@/lib/utils";
import { Button, Modal } from "@/components/ui";
import { KeyfileChip, PasswordInput } from "./fields";

const fileName = (p: string) => p.split(/[\\/]/).pop() ?? p;

function formatSize(bytes: number): string {
  if (bytes < 1024 * 1024) return `${Math.max(1, Math.round(bytes / 1024))} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

/**
 * Restore the vault from a backup file. Shared by the lock screen, the first-run screen and Settings.
 * The password lives only in this component's state and is cleared on close and on success.
 */
export default function RestoreDialog({
  open,
  onOpenChange,
  vaultExists,
}: {
  open: boolean;
  onOpenChange: (o: boolean) => void;
  vaultExists: boolean;
}) {
  const [backups, setBackups] = useState<BackupInfo[] | null>(null);
  const [listError, setListError] = useState<string | null>(null);
  const [source, setSource] = useState<string | null>(null);
  const [pickedPath, setPickedPath] = useState<string | null>(null);
  const [password, setPassword] = useState("");
  const [keyfile, setKeyfile] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [done, setDone] = useState<{ undoFile: string | null } | null>(null);
  // The backup was made while security was off: restoring it turns security off, so ask first.
  const [confirmOff, setConfirmOff] = useState(false);
  const [askConfirmOff, setAskConfirmOff] = useState(false);
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
      setConfirmOff(false);
      setAskConfirmOff(false);
      return;
    }
    let cancelled = false;
    api.listBackups()
      .then((list) => !cancelled && setBackups(list))
      .catch((e) => {
        if (cancelled) return;
        setBackups([]);
        setListError(errorText(e));
      });
    return () => {
      cancelled = true;
    };
  }, [open]);

  // The password may stay empty: a backup made while security was off needs none.
  const canRestore = !!source && !busy && !done && (!askConfirmOff || confirmOff);

  const restore = async () => {
    if (!source || busy || done) return;
    setBusy(true);
    setError(null);
    try {
      const result = await api.restoreVault(source, password, keyfile, confirmOff);
      setPassword("");
      setDone(result);
      // The backend has unlocked the restored vault. A reload makes the screen start fresh from it.
      reloadTimer.current = setTimeout(() => window.location.reload(), 1200);
    } catch (e) {
      const text = errorText(e);
      if (text.startsWith("SECURITY_OFF:")) {
        // Not a failure: the person has to confirm before the restore turns security off.
        setAskConfirmOff(true);
        setError(text.replace(/^SECURITY_OFF:\s*/, ""));
      } else {
        setError(text);
        focusPassword();
      }
    } finally {
      setBusy(false);
    }
  };

  const chooseFile = async () => {
    setError(null);
    try {
      const path = await api.pickBackupFile();
      if (path) {
        setPickedPath(path);
        setSource(path);
        focusPassword();
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
        if (!o && (busy || done)) return;
        onOpenChange(o);
      }}
      title="Restore from a backup"
      description="Go back to an earlier copy of your vault."
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
              {listError ?? "No automatic backups found in the data folder."}
            </p>
          )}

          {(pickedPath || (backups && backups.length > 0)) && (
            <div className="max-h-52 space-y-1.5 overflow-y-auto pr-0.5">
              {pickedPath && (
                <BackupRow
                  checked={source === pickedPath}
                  onSelect={() => setSource(pickedPath)}
                  onMouseChosen={focusPassword}
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
                  onMouseChosen={focusPassword}
                  title={formatDate(b.modifiedMs)}
                  detail={formatSize(b.size)}
                  tooltip={b.name}
                />
              ))}
            </div>
          )}

          <Button type="button" variant="outline" size="sm" onClick={chooseFile} className="disabled:opacity-70">
            <FolderOpen size={14} /> Choose another backup file…
          </Button>
        </fieldset>

        {vaultExists && (
          <div className="flex items-start gap-2.5 rounded-xl border border-warn/30 bg-warn/5 p-3.5 text-sm leading-relaxed text-muted">
            <TriangleAlert size={16} className="mt-0.5 shrink-0 text-warn" />
            <span>
              This replaces your current vault with the backup. Your current vault is not deleted. It is kept in the
              data folder in a file whose name starts with "restore-undo".
            </span>
          </div>
        )}

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

        {error && (
          <p role="alert" className="text-sm text-danger">
            {error}
          </p>
        )}

        {askConfirmOff && !done && (
          <label className="flex cursor-pointer items-start gap-3 rounded-2xl border border-warn/30 bg-warn/5 p-4 text-sm leading-relaxed">
            <input
              type="checkbox"
              checked={confirmOff}
              onChange={(e) => setConfirmOff(e.target.checked)}
              className="mt-1 h-4 w-4 accent-[var(--accent)]"
            />
            <span className="text-muted">
              Turn security off and restore this backup. My notes will open without a password.
            </span>
          </label>
        )}

        {done && (
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
                <RotateCcw size={14} /> {vaultExists ? "Replace my vault with this backup" : "Restore backup"}
              </>
            )}
          </Button>
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
