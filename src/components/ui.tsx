// Small, consistent UI primitives built on Radix (accessible, keyboard-friendly).

import * as React from "react";
import { AlertDialog, Dialog as RDialog, DropdownMenu, Slider as RSlider, Switch as RSwitch, Tooltip } from "radix-ui";
import { X } from "lucide-react";
import { motion } from "motion/react";
import { cn } from "@/lib/utils";

/* ---------- Button ---------- */

type ButtonVariant = "primary" | "secondary" | "ghost" | "danger" | "outline";

export const Button = React.forwardRef<
  HTMLButtonElement,
  React.ButtonHTMLAttributes<HTMLButtonElement> & { variant?: ButtonVariant; size?: "sm" | "md" | "lg" }
>(({ className, variant = "secondary", size = "md", ...props }, ref) => (
  <button
    ref={ref}
    className={cn(
      "inline-flex items-center justify-center gap-2 rounded-xl font-medium transition-all duration-150 select-none",
      "disabled:opacity-45 disabled:pointer-events-none active:scale-[0.98]",
      size === "sm" && "h-8 px-3 text-[0.8rem]",
      size === "md" && "h-9 px-4 text-sm",
      size === "lg" && "h-11 px-5 text-[0.95rem]",
      variant === "primary" &&
        "bg-accent text-white shadow-[0_6px_20px_-6px_var(--accent)] hover:brightness-110",
      variant === "secondary" && "bg-surface-2 text-fg hover:bg-line",
      variant === "ghost" && "text-muted hover:text-fg hover:bg-surface-2",
      variant === "outline" && "border border-line text-fg hover:bg-surface-2",
      variant === "danger" && "bg-danger/90 text-white hover:bg-danger",
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
        "h-10 w-full rounded-xl border border-line bg-surface px-3.5 text-sm text-fg placeholder:text-faint",
        "outline-none transition-all focus:border-accent focus:bg-surface-2 focus:ring-4 focus:ring-accent-soft",
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
        "w-full rounded-xl border border-line bg-surface px-3.5 py-2.5 text-sm text-fg placeholder:text-faint",
        "outline-none transition-all focus:border-accent focus:bg-surface-2 focus:ring-4 focus:ring-accent-soft resize-none",
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
          "block h-[18px] w-[18px] rounded-full bg-white shadow transition-transform duration-200",
          checked ? "translate-x-[23px]" : "translate-x-[3px]",
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
      <RSlider.Thumb className="block h-4.5 w-4.5 rounded-full border-2 border-accent bg-white shadow-md transition-transform hover:scale-110 focus:outline-none focus:ring-4 focus:ring-accent-soft" />
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
    <div className="inline-flex rounded-xl bg-surface-2 p-1">
      {options.map((o) => (
        <button
          key={o.value}
          onClick={() => onChange(o.value)}
          className={cn(
            "relative rounded-lg px-3 py-1.5 text-sm transition-colors",
            value === o.value ? "text-fg" : "text-muted hover:text-fg",
          )}
        >
          {value === o.value && (
            <motion.span
              layoutId={`seg-${id}`}
              className="absolute inset-0 rounded-lg bg-elevated shadow-sm"
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
  title,
  description,
  children,
  width = 460,
}: {
  open: boolean;
  onOpenChange: (v: boolean) => void;
  title: string;
  description?: string;
  children: React.ReactNode;
  width?: number;
}) {
  return (
    <RDialog.Root open={open} onOpenChange={onOpenChange}>
      <RDialog.Portal>
        <RDialog.Overlay className="fixed inset-0 z-50 bg-black/45 backdrop-blur-sm" />
        <RDialog.Content
          style={{ width }}
          className="fixed left-1/2 top-1/2 z-50 max-h-[85vh] max-w-[92vw] -translate-x-1/2 -translate-y-1/2 overflow-y-auto rounded-2xl border border-line bg-elevated p-6 shadow-float outline-none"
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
              <button className="rounded-lg p-1.5 text-muted hover:bg-surface-2 hover:text-fg" aria-label="Close">
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
}

const ConfirmContext = React.createContext<(o: ConfirmOptions) => Promise<boolean>>(async () => false);

export function ConfirmProvider({ children }: { children: React.ReactNode }) {
  const [state, setState] = React.useState<(ConfirmOptions & { resolve: (v: boolean) => void }) | null>(null);
  const ask = React.useCallback(
    (o: ConfirmOptions) => new Promise<boolean>((resolve) => setState({ ...o, resolve })),
    [],
  );
  const close = (v: boolean) => {
    state?.resolve(v);
    setState(null);
  };
  return (
    <ConfirmContext.Provider value={ask}>
      {children}
      <AlertDialog.Root open={!!state} onOpenChange={(o) => !o && close(false)}>
        <AlertDialog.Portal>
          <AlertDialog.Overlay className="fixed inset-0 z-[60] bg-black/45 backdrop-blur-sm" />
          <AlertDialog.Content className="fixed left-1/2 top-1/2 z-[60] w-[420px] max-w-[92vw] -translate-x-1/2 -translate-y-1/2 rounded-2xl border border-line bg-elevated p-6 shadow-float">
            <AlertDialog.Title className="text-lg font-semibold">{state?.title}</AlertDialog.Title>
            <AlertDialog.Description className="mt-2 text-sm leading-relaxed text-muted">
              {state?.body}
            </AlertDialog.Description>
            <div className="mt-6 flex justify-end gap-2">
              <AlertDialog.Cancel asChild>
                <Button variant="ghost">Cancel</Button>
              </AlertDialog.Cancel>
              <AlertDialog.Action asChild>
                <Button variant={state?.danger ? "danger" : "primary"} onClick={() => close(true)}>
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
        className="z-[70] min-w-[190px] rounded-xl border border-line bg-elevated p-1.5 shadow-float"
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
        "flex cursor-pointer items-center gap-2.5 rounded-lg px-2.5 py-2 text-sm outline-none",
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
