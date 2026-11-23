// Small, consistent UI primitives built on Radix (accessible, keyboard-friendly).

import * as React from "react";
import { AlertDialog, Dialog as RDialog, DropdownMenu, Slider as RSlider, Switch as RSwitch, Tooltip } from "radix-ui";
import { Loader2, X } from "lucide-react";
import { motion } from "motion/react";
import { cn } from "@/lib/utils";
import { errorText } from "@/lib/ipc";

/* ---------- Button ---------- */

type ButtonVariant = "primary" | "secondary" | "ghost" | "danger" | "outline";

export const Button = React.forwardRef<
  HTMLButtonElement,
  React.ButtonHTMLAttributes<HTMLButtonElement> & { variant?: ButtonVariant; size?: "sm" | "md" | "lg" }
>(({ className, variant = "secondary", size = "md", ...props }, ref) => (
  <button
    ref={ref}
    className={cn(
      "inline-flex items-center justify-center gap-2 rounded-lg font-medium transition-all duration-150 select-none",
      "disabled:opacity-45 disabled:pointer-events-none active:scale-[0.98]",
      size === "sm" && "h-8 px-3 text-[0.8rem]",
      size === "md" && "h-9 px-4 text-sm",
      size === "lg" && "h-11 px-5 text-[0.95rem]",
      variant === "primary" &&
        "bg-accent text-accent-fg hover:opacity-90",
      variant === "secondary" && "bg-surface-2 text-fg hover:bg-line",
      variant === "ghost" && "text-muted hover:text-fg hover:bg-surface-2",
      variant === "outline" && "border border-line text-fg hover:bg-surface-2",
      variant === "danger" && "bg-danger text-white hover:opacity-90",
      className,
    )}
    {...props}
  />
));
Button.displayName = "Button";

export const IconButton = React.forwardRef<
  HTMLButtonElement,
  React.ButtonHTMLAttributes<HTMLButtonElement> & { label: string; active?: boolean; size?: "sm" | "md" }
>(({ className, label, active, size = "md", children, ...props }, ref) => (
  <Tip label={label}>
    <button
      ref={ref}
      aria-label={label}
      className={cn(
        "inline-flex items-center justify-center rounded-lg transition-colors duration-150",
        size === "md" ? "h-8 w-8" : "h-7 w-7",
        active ? "bg-accent-soft text-accent" : "text-muted hover:text-fg hover:bg-surface-2",
        "disabled:opacity-40 disabled:pointer-events-none",
        className,
      )}
      {...props}
    >
      {children}
    </button>
  </Tip>
));
IconButton.displayName = "IconButton";

/* ---------- Inputs ---------- */

export const Input = React.forwardRef<HTMLInputElement, React.InputHTMLAttributes<HTMLInputElement>>(
  ({ className, ...props }, ref) => (
    <input
      ref={ref}
      spellCheck={false}
      autoComplete="off"
      className={cn(
        "h-9 w-full rounded-lg border border-line bg-surface px-3.5 text-sm text-fg placeholder:text-faint",
        "outline-none transition-all focus:border-accent focus:bg-surface-2 focus:ring-2 focus:ring-accent-soft",
        className,
      )}
      {...props}
    />
  ),
);
Input.displayName = "Input";

export const Textarea = React.forwardRef<HTMLTextAreaElement, React.TextareaHTMLAttributes<HTMLTextAreaElement>>(
  ({ className, ...props }, ref) => (
    <textarea
      ref={ref}
      className={cn(
        "w-full rounded-lg border border-line bg-surface px-3.5 py-2.5 text-sm text-fg placeholder:text-faint",
        "outline-none transition-all focus:border-accent focus:bg-surface-2 focus:ring-2 focus:ring-accent-soft resize-none",
        className,
      )}
      {...props}
    />
  ),
);
Textarea.displayName = "Textarea";

export function Field({ label, hint, children }: { label: string; hint?: React.ReactNode; children: React.ReactNode }) {
  return (
    <label className="block space-y-1.5">
      <span className="text-xs font-medium uppercase tracking-wider text-faint">{label}</span>
      {children}
      {hint && <span className="block text-xs text-faint">{hint}</span>}
    </label>
  );
}

export function Switch({ checked, onChange, label }: { checked: boolean; onChange: (v: boolean) => void; label?: string }) {
  return (
    <RSwitch.Root
      checked={checked}
      onCheckedChange={onChange}
      aria-label={label}
      className={cn(
        "relative h-6 w-11 shrink-0 rounded-full transition-colors duration-200",
        checked ? "bg-accent" : "bg-surface-2 border border-line",
      )}
    >
      <RSwitch.Thumb
        className={cn(
          "block h-[18px] w-[18px] rounded-full shadow transition-transform duration-200",
          checked ? "translate-x-[23px] bg-accent-fg" : "translate-x-[3px] bg-muted",
        )}
      />
    </RSwitch.Root>
  );
}

export function Slider({
  value,
  min,
  max,
  step = 1,
  onChange,
  label,
}: {
  value: number;
  min: number;
  max: number;
  step?: number;
  onChange: (v: number) => void;
  label: string;
}) {
  return (
    <RSlider.Root
      className="relative flex h-5 w-full touch-none items-center select-none"
      value={[value]}
      min={min}
      max={max}
      step={step}
      onValueChange={(v) => onChange(v[0])}
      aria-label={label}
    >
      <RSlider.Track className="relative h-1.5 grow rounded-full bg-surface-2">
        <RSlider.Range className="absolute h-full rounded-full bg-accent" />
      </RSlider.Track>
      <RSlider.Thumb className="block h-4.5 w-4.5 rounded-full border-2 border-accent bg-bg shadow-md transition-transform hover:scale-110 focus:outline-none focus:ring-2 focus:ring-accent-soft" />
    </RSlider.Root>
  );
}

export function Segmented<T extends string>({
  value,
  options,
  onChange,
}: {
  value: T;
  options: { value: T; label: React.ReactNode }[];
  onChange: (v: T) => void;
}) {
  const id = React.useId();
  return (
    <div className="inline-flex rounded-lg bg-surface-2 p-0.5">
      {options.map((o) => (
        <button
          key={o.value}
          type="button"
          aria-pressed={value === o.value}
          onClick={() => onChange(o.value)}
          className={cn(
            "relative rounded-md px-3 py-1 text-sm transition-colors",
            value === o.value ? "text-fg" : "text-muted hover:text-fg",
          )}
        >
          {value === o.value && (
            <motion.span
              layoutId={`seg-${id}`}
              className="absolute inset-0 rounded-md bg-elevated shadow-sm"
              transition={{ type: "spring", stiffness: 500, damping: 38 }}
            />
          )}
          <span className="relative">{o.label}</span>
        </button>
      ))}
    </div>
  );
}

export function Kbd({ children }: { children: React.ReactNode }) {
  return (
    <kbd className="rounded-md border border-line bg-surface px-1.5 py-0.5 font-mono text-[0.7rem] text-muted">
      {children}
    </kbd>
  );
}

/* ---------- Tooltip ---------- */

export function Tip({ label, children, side = "bottom" }: { label: string; children: React.ReactNode; side?: "top" | "bottom" | "left" | "right" }) {
  return (
    <Tooltip.Root delayDuration={350}>
      <Tooltip.Trigger asChild>{children}</Tooltip.Trigger>
      <Tooltip.Portal>
        <Tooltip.Content
          side={side}
          sideOffset={6}
          className="z-[100] rounded-lg bg-elevated px-2.5 py-1.5 text-xs text-fg shadow-float border border-line"
        >
          {label}
        </Tooltip.Content>
      </Tooltip.Portal>
    </Tooltip.Root>
  );
}

export const TooltipProvider = Tooltip.Provider;

/* ---------- Dialog ---------- */

export function Modal({
  open,
  onOpenChange,
  onCloseAutoFocus,
  title,
  description,
  children,
  width = 460,
  busy = false,
}: {
  open: boolean;
  onOpenChange: (v: boolean) => void;
  onCloseAutoFocus?: (event: Event) => void;
  title: string;
  description?: string;
  children: React.ReactNode;
  width?: number;
  busy?: boolean;
}) {
  return (
    <RDialog.Root open={open} onOpenChange={(value) => !busy && onOpenChange(value)}>
      <RDialog.Portal>
        <RDialog.Overlay className="fixed inset-0 z-50 bg-black/45 backdrop-blur-sm" />
        <RDialog.Content
          onCloseAutoFocus={onCloseAutoFocus}
          aria-busy={busy || undefined}
          style={{ width }}
          className="fixed left-1/2 top-1/2 z-50 max-h-[85vh] max-w-[92vw] -translate-x-1/2 -translate-y-1/2 overflow-y-auto rounded-xl border border-line bg-elevated p-6 shadow-float outline-none"
        >
          <div className="mb-5 flex items-start justify-between gap-4">
            <div>
              <RDialog.Title className="text-lg font-semibold tracking-tight">{title}</RDialog.Title>
              {description ? (
                <RDialog.Description className="mt-1 text-sm text-muted">{description}</RDialog.Description>
              ) : (
                <RDialog.Description className="sr-only">{title}</RDialog.Description>
              )}
            </div>
            <RDialog.Close asChild>
              <button className="rounded-lg p-1.5 text-muted hover:bg-surface-2 hover:text-fg disabled:opacity-45" aria-label="Close" disabled={busy}>
                <X size={16} />
              </button>
            </RDialog.Close>
          </div>
          {children}
        </RDialog.Content>
      </RDialog.Portal>
    </RDialog.Root>
  );
}

/* ---------- Confirm ---------- */

interface ConfirmOptions {
  title: string;
  body: string;
  confirm?: string;
  danger?: boolean;
  onConfirm?: () => Promise<void>;
  busyLabel?: string;
}

type Confirmation = ConfirmOptions & { resolve: (v: boolean) => void };
const ConfirmContext = React.createContext<(o: ConfirmOptions) => Promise<boolean>>(async () => false);

export function ConfirmProvider({ children }: { children: React.ReactNode }) {
  const pending = React.useRef<Confirmation | null>(null);
  const running = React.useRef(false);
  const [state, setState] = React.useState<Confirmation | null>(null);
  const [busy, setBusy] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);
  const ask = React.useCallback((o: ConfirmOptions) => new Promise<boolean>((resolve) => {
    if (pending.current) {
      resolve(false);
      return;
    }
    pending.current = { ...o, resolve };
    setError(null);
    setState(pending.current);
  }), []);
  const close = (accepted: boolean) => {
    const request = pending.current;
    pending.current = null;
    setState(null);
    request?.resolve(accepted);
  };
  const accept = async () => {
    const request = pending.current;
    if (!request || running.current) return;
    if (!request.onConfirm) {
      close(true);
      return;
    }
    running.current = true;
    setBusy(true);
    setError(null);
    try {
      await request.onConfirm();
      close(true);
    } catch (cause) {
      setError(errorText(cause));
    } finally {
      running.current = false;
      setBusy(false);
    }
  };
  return (
    <ConfirmContext.Provider value={ask}>
      {children}
      <AlertDialog.Root open={!!state} onOpenChange={(o) => !o && !running.current && close(false)}>
        <AlertDialog.Portal>
          <AlertDialog.Overlay className="fixed inset-0 z-[60] bg-black/45 backdrop-blur-sm" />
          <AlertDialog.Content aria-busy={busy || undefined} className="fixed left-1/2 top-1/2 z-[60] w-[420px] max-w-[92vw] -translate-x-1/2 -translate-y-1/2 rounded-xl border border-line bg-elevated p-6 shadow-float">
            <AlertDialog.Title className="text-lg font-semibold">{state?.title}</AlertDialog.Title>
            <AlertDialog.Description className="mt-2 whitespace-pre-line text-sm leading-relaxed text-muted">
              {state?.body}
            </AlertDialog.Description>
            {error && <p role="alert" className="mt-3 text-sm text-danger">{error}</p>}
            <div className="mt-6 flex items-center justify-end gap-2">
              {busy && <span role="status" className="mr-auto text-sm text-muted">{state?.busyLabel ?? "Working…"}</span>}
              <AlertDialog.Cancel asChild>
                <Button variant="ghost" disabled={busy}>Cancel</Button>
              </AlertDialog.Cancel>
              <AlertDialog.Action asChild>
                <Button variant={state?.danger ? "danger" : "primary"} disabled={busy} onClick={(e) => {
                  e.preventDefault();
                  void accept();
                }}>
                  {busy && <Loader2 size={15} className="animate-spin" aria-hidden="true" />}
                  {state?.confirm ?? "Confirm"}
                </Button>
              </AlertDialog.Action>
            </div>
          </AlertDialog.Content>
        </AlertDialog.Portal>
      </AlertDialog.Root>
    </ConfirmContext.Provider>
  );
}

export const useConfirm = () => React.useContext(ConfirmContext);

/* ---------- Dropdown menu ---------- */

export const Menu = DropdownMenu.Root;
export const MenuTrigger = DropdownMenu.Trigger;

export function MenuContent({ children, align = "end" }: { children: React.ReactNode; align?: "start" | "end" | "center" }) {
  return (
    <DropdownMenu.Portal>
      <DropdownMenu.Content
        align={align}
        sideOffset={6}
        className="z-[70] min-w-[190px] rounded-lg border border-line bg-elevated p-1.5 shadow-float"
      >
        {children}
      </DropdownMenu.Content>
    </DropdownMenu.Portal>
  );
}

export function MenuItem({
  children,
  onSelect,
  danger,
  icon,
}: {
  children: React.ReactNode;
  onSelect: () => void;
  danger?: boolean;
  icon?: React.ReactNode;
}) {
  return (
    <DropdownMenu.Item
      onSelect={onSelect}
      className={cn(
        "flex cursor-pointer items-center gap-2.5 rounded-md px-2.5 py-1.5 text-sm outline-none",
        danger ? "text-danger data-[highlighted]:bg-danger/10" : "text-fg data-[highlighted]:bg-surface-2",
      )}
    >
      {icon && <span className="opacity-70">{icon}</span>}
      {children}
    </DropdownMenu.Item>
  );
}

export function MenuSeparator() {
  return <DropdownMenu.Separator className="my-1 h-px bg-line" />;
}

/* ---------- Misc ---------- */

export function EmptyState({ icon, title, body, action }: { icon: React.ReactNode; title: string; body: string; action?: React.ReactNode }) {
  return (
    <div className="flex h-full flex-col items-center justify-center px-8 text-center">
      <motion.div
        initial={{ scale: 0.85, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ type: "spring", stiffness: 260, damping: 20 }}
        className="mb-5 grid h-16 w-16 place-items-center rounded-2xl bg-accent-soft text-accent"
      >
        {icon}
      </motion.div>
      <h3 className="text-lg font-semibold tracking-tight">{title}</h3>
      <p className="mt-1.5 max-w-sm text-sm leading-relaxed text-muted">{body}</p>
      {action && <div className="mt-6">{action}</div>}
    </div>
  );
}

export function StrengthMeter({ score, label }: { score: number; label?: string }) {
  const colors = ["#f43f5e", "#f97316", "#eab308", "#22c55e", "#10b981"];
  const names = ["Very weak", "Weak", "Fair", "Strong", "Very strong"];
  return (
    <div className="space-y-1.5">
      <div className="flex gap-1.5">
        {[0, 1, 2, 3, 4].map((i) => (
          <motion.div
            key={i}
            className="h-1.5 flex-1 rounded-full"
            animate={{ backgroundColor: i <= score ? colors[score] : "var(--surface-2)" }}
            transition={{ duration: 0.25 }}
          />
        ))}
      </div>
      <div className="flex justify-between text-xs">
        <span style={{ color: colors[score] }} className="font-medium">
          {names[score]}
        </span>
        {label && <span className="text-faint">{label}</span>}
      </div>
    </div>
  );
}
