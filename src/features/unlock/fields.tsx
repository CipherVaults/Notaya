import { useEffect, useRef, useState } from "react";
import { Check, Copy, Eye, EyeOff, FileKey2, KeyRound, X } from "lucide-react";
import { Button } from "@/components/ui";

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

/** Shows the chosen keyfile's name with a small remove button. */
export function KeyfileChip({ path, onClear, disabled }: { path: string; onClear: () => void; disabled?: boolean }) {
  const name = path.split(/[\\/]/).pop();
  return (
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
  );
}

/**
 * Puts text on the clipboard, with the older fallback if the page may not use the clipboard API.
 * Returns true only if the copy really worked.
 */
export async function copyText(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    // Fall through to the older way.
  }
  const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  const ta = document.createElement("textarea");
  ta.value = text;
  ta.setAttribute("aria-hidden", "true");
  ta.setAttribute("readonly", "");
  ta.tabIndex = -1;
  ta.style.position = "fixed";
  ta.style.top = "0";
  ta.style.left = "0";
  ta.style.opacity = "0";
  ta.style.pointerEvents = "none";
  document.body.append(ta);
  let ok = false;
  try {
    ta.select();
    ok = document.execCommand("copy");
  } catch {
    ok = false;
  } finally {
    ta.remove();
    previous?.focus();
  }
  return ok;
}

/** Shows a suggested passphrase in plain sight so it can be read, written down or copied. */
export function SuggestedPassphrase({ value }: { value: string }) {
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
