import { useState } from "react";
import { Eye, EyeOff, FileKey2, KeyRound, X } from "lucide-react";

/** A large password box with a show/hide eye. Used on the lock screen and in the restore dialog. */
export function PasswordInput({
  value,
  onChange,
  placeholder,
  autoFocus,
  onEnter,
  label,
  inputRef,
}: {
  value: string;
  onChange: (v: string) => void;
  placeholder: string;
  autoFocus?: boolean;
  onEnter?: () => void;
  /** Accessible name for screen readers (defaults to the placeholder). */
  label?: string;
  inputRef?: React.Ref<HTMLInputElement>;
}) {
  const [show, setShow] = useState(false);
  return (
    <div className="relative">
      <KeyRound size={16} className="pointer-events-none absolute left-3.5 top-1/2 -translate-y-1/2 text-faint" />
      <input
        ref={inputRef}
        type={show ? "text" : "password"}
        value={value}
        autoFocus={autoFocus}
        spellCheck={false}
        autoComplete="off"
        placeholder={placeholder}
        aria-label={label ?? placeholder}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={(e) => e.key === "Enter" && !e.nativeEvent.isComposing && onEnter?.()}
        className="h-12 w-full rounded-2xl border border-line bg-surface pl-10 pr-11 text-[0.95rem] text-fg outline-none transition-all placeholder:text-faint focus:border-accent focus:ring-4 focus:ring-accent-soft"
      />
      <button
        type="button"
        onClick={() => setShow(!show)}
        className="absolute right-2 top-1/2 -translate-y-1/2 rounded-lg p-2 text-faint hover:text-fg focus-visible:text-fg focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-soft"
        aria-label={show ? "Hide password" : "Show password"}
      >
        {show ? <EyeOff size={16} /> : <Eye size={16} />}
      </button>
    </div>
  );
}

/** Files that change on their own or belong to an app: a keyfile like that stops working once it changes. */
const CHANGING_FILE = /\.(nota|clf|kdbx|ini|exe|dll|tmp|log|lnk|db|journal)$/i;

/**
 * A warning for a keyfile choice that is likely a mistake, or null. Any file can be a
 * keyfile, but only its exact contents count: if the file changes, the vault won't open.
 */
export function keyfileWarning(path: string, dataDir?: string): string | null {
  const norm = (p: string) => p.replace(/[\\/]+/g, "\\").replace(/\\$/, "").toLowerCase();
  if (dataDir && norm(path).startsWith(norm(dataDir) + "\\")) {
    return "This file is inside the vault's data folder. Files there change as the app runs, so it may stop working as a keyfile. Keep the keyfile somewhere else, ideally on another drive.";
  }
  if (CHANGING_FILE.test(path)) {
    return "This kind of file changes on its own or belongs to an app. A keyfile only works while its contents stay exactly the same, so this may be the wrong file.";
  }
  return null;
}

/** Shows the chosen keyfile's name with a small remove button, and a warning if the choice looks wrong. */
export function KeyfileChip({
  path,
  onClear,
  disabled,
  dataDir,
}: {
  path: string;
  onClear: () => void;
  disabled?: boolean;
  dataDir?: string;
}) {
  const name = path.split(/[\\/]/).pop();
  const warning = keyfileWarning(path, dataDir);
  return (
    <div className="space-y-1.5">
      <div className="flex items-center gap-2 rounded-xl border border-accent/40 bg-accent-soft px-3 py-2 text-sm">
        <FileKey2 size={15} className="text-accent" />
        <span className="flex-1 truncate" title={path}>
          {name}
        </span>
        <button
          onClick={onClear}
          disabled={disabled}
          className="rounded p-0.5 text-muted hover:text-fg disabled:pointer-events-none disabled:opacity-45"
          aria-label="Remove keyfile"
          type="button"
        >
          <X size={14} />
        </button>
      </div>
      {warning && <p className="text-xs leading-relaxed text-warn">{warning}</p>}
    </div>
  );
}
