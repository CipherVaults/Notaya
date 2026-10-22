import { useCallback, useEffect, useState } from "react";
import { Eye, EyeOff, Plus, ShieldAlert, Sparkles, Trash2, X } from "lucide-react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import { TAG_COLORS } from "@/lib/palette";
import type { CustomField, Login, LoginInput, LoginSummary, Strength } from "@/lib/types";
import { cn } from "@/lib/utils";
import { Button, Field, Input, Modal, StrengthMeter, Switch, Textarea } from "@/components/ui";
import TotpRing from "./TotpRing";

const EMPTY: LoginInput = {
  id: null,
  title: "",
  username: "",
  password: "",
  urls: [""],
  notes: "",
  totp: "",
  fields: [],
  tags: [],
  folder: "",
  favorite: false,
  color: "",
};

export default function LoginForm({
  open,
  onOpenChange,
  existing,
  onSaved,
}: {
  open: boolean;
  onOpenChange: (v: boolean) => void;
  existing: Login | null;
  onSaved: (l: LoginSummary) => void;
}) {
  const [f, setF] = useState<LoginInput>(EMPTY);
  const [tagText, setTagText] = useState("");
  const [show, setShow] = useState(false);
  const [strength, setStrength] = useState<Strength | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (!open) return;
    if (existing) {
      setF({
        id: existing.id,
        title: existing.title,
        username: existing.username,
        password: existing.password,
        urls: existing.urls.length ? existing.urls : [""],
        notes: existing.notes,
        totp: existing.totp,
        fields: existing.fields.map((x) => ({ ...x })),
        tags: existing.tags,
        folder: existing.folder,
        favorite: existing.favorite,
        color: existing.color,
      });
      setTagText(existing.tags.join(", "));
    } else {
      setF(EMPTY);
      setTagText("");
    }
    setShow(false);
  }, [open, existing]);

  useEffect(() => {
    if (!f.password) {
      setStrength(null);
      return;
    }
    const t = setTimeout(() => api.passwordStrength(f.password, [f.username, f.title]).then(setStrength), 150);
    return () => clearTimeout(t);
  }, [f.password, f.username, f.title]);

  const set = <K extends keyof LoginInput>(k: K, v: LoginInput[K]) => setF((x) => ({ ...x, [k]: v }));

  const generate = async () => {
    const g = await api.generatePassword({ length: 20, lower: true, upper: true, digits: true, symbols: true, avoidAmbiguous: true });
    set("password", g.value);
    setShow(true);
  };

  const totpFetch = useCallback(() => api.totpPreview(f.totp), [f.totp]);

  const submit = async () => {
    setBusy(true);
    try {
      const input: LoginInput = {
        ...f,
        tags: tagText.split(",").map((t) => t.trim()).filter(Boolean),
        urls: f.urls.map((u) => u.trim()).filter(Boolean),
        fields: f.fields.filter((x) => x.name.trim() || x.value),
      };
      const saved = await api.saveLogin(input);
      onSaved(saved);
      onOpenChange(false);
      toast.success(existing ? "Login updated." : "Login saved.");
    } catch (e) {
      toast.error(errorText(e));
    } finally {
      setBusy(false);
    }
  };

  const updateField = (i: number, patch: Partial<CustomField>) =>
    set("fields", f.fields.map((x, j) => (j === i ? { ...x, ...patch } : x)));

  return (
    <Modal open={open} onOpenChange={onOpenChange} title={existing ? "Edit login" : "New login"} width={580}>
      <form
        className="space-y-4"
        onSubmit={(e) => {
          e.preventDefault();
          submit();
        }}
      >
        <div className="grid grid-cols-2 gap-3">
          <Field label="Name">
            <Input autoFocus value={f.title} onChange={(e) => set("title", e.target.value)} placeholder="e.g. GitHub" />
          </Field>
          <Field label="Username or email">
            <Input value={f.username} onChange={(e) => set("username", e.target.value)} placeholder="you@example.com" />
          </Field>
        </div>

        <Field label="Password">
          <div className="flex gap-2">
            <div className="relative flex-1">
              <Input
                type={show ? "text" : "password"}
                value={f.password}
                onChange={(e) => set("password", e.target.value)}
                className="pr-10 font-mono"
              />
              <button
                type="button"
                onClick={() => setShow(!show)}
                className="absolute right-2 top-1/2 -translate-y-1/2 rounded p-1.5 text-faint hover:text-fg"
                aria-label={show ? "Hide" : "Show"}
              >
                {show ? <EyeOff size={15} /> : <Eye size={15} />}
              </button>
            </div>
            <Button type="button" variant="outline" onClick={generate}>
              <Sparkles size={15} /> Generate
            </Button>
          </div>
        </Field>
        {strength && <StrengthMeter score={strength.score} label={`~${strength.crackTime} to crack`} />}

        <Field label="Website">
          <div className="space-y-2">
            {f.urls.map((u, i) => (
              <div key={i} className="flex gap-2">
                <Input
                  value={u}
                  onChange={(e) => set("urls", f.urls.map((x, j) => (j === i ? e.target.value : x)))}
                  placeholder="https://"
                />
                {f.urls.length > 1 && (
                  <Button type="button" variant="ghost" className="px-2.5" onClick={() => set("urls", f.urls.filter((_, j) => j !== i))}>
                    <X size={15} />
                  </Button>
                )}
              </div>
            ))}
            <button type="button" onClick={() => set("urls", [...f.urls, ""])} className="flex items-center gap-1 text-xs font-medium text-accent hover:underline">
              <Plus size={12} /> Add another website
            </button>
          </div>
        </Field>

        <Field label="Two-factor (TOTP) secret" hint="Paste the setup key or otpauth:// link from the site's 2FA setup page.">
          <Input value={f.totp} onChange={(e) => set("totp", e.target.value)} placeholder="JBSW Y3DP EHPK 3PXP" className="font-mono" />
        </Field>
        {f.totp.trim().length >= 16 && (
          <div className="rounded-xl border border-line bg-surface px-4 py-3">
            <TotpRing fetchCode={totpFetch} />
          </div>
        )}

        <div className="grid grid-cols-2 gap-3">
          <Field label="Folder">
            <Input value={f.folder} onChange={(e) => set("folder", e.target.value)} placeholder="e.g. Work" />
          </Field>
          <Field label="Tags">
            <Input value={tagText} onChange={(e) => setTagText(e.target.value)} placeholder="comma, separated" />
          </Field>
        </div>

        <Field label="Notes">
          <Textarea rows={3} value={f.notes} onChange={(e) => set("notes", e.target.value)} placeholder="Security questions, recovery codes…" />
        </Field>

        <div className="space-y-2">
          <span className="text-xs font-medium uppercase tracking-wider text-faint">Custom fields</span>
          {f.fields.map((x, i) => (
            <div key={i} className="flex items-center gap-2">
              <Input value={x.name} onChange={(e) => updateField(i, { name: e.target.value })} placeholder="Label" className="w-36" />
              <Input
                type={x.secret ? "password" : "text"}
                value={x.value}
                onChange={(e) => updateField(i, { value: e.target.value })}
                placeholder="Value"
              />
              <button
                type="button"
                title={x.secret ? "Hidden field" : "Visible field"}
                onClick={() => updateField(i, { secret: !x.secret })}
                className={cn("rounded-lg p-2", x.secret ? "text-accent" : "text-faint hover:text-fg")}
              >
                <ShieldAlert size={15} />
              </button>
              <button type="button" onClick={() => set("fields", f.fields.filter((_, j) => j !== i))} className="rounded-lg p-2 text-faint hover:text-danger">
                <Trash2 size={15} />
              </button>
            </div>
          ))}
          <button
            type="button"
            onClick={() => set("fields", [...f.fields, { name: "", value: "", secret: true }])}
            className="flex items-center gap-1 text-xs font-medium text-accent hover:underline"
          >
            <Plus size={12} /> Add field
          </button>
        </div>

        <div className="flex items-center justify-between rounded-xl border border-line bg-surface px-4 py-3">
          <div className="flex gap-1.5">
            {["", ...TAG_COLORS].map((c) => (
              <button
                type="button"
                key={c || "none"}
                onClick={() => set("color", c)}
                className={cn(
                  "h-6 w-6 rounded-full border border-line transition-transform hover:scale-110",
                  f.color === c && "ring-2 ring-accent ring-offset-2 ring-offset-[var(--elevated)]",
                )}
                style={{ background: c || "var(--surface-2)" }}
                aria-label={c || "No color"}
              />
            ))}
          </div>
          <label className="flex items-center gap-2 text-sm text-muted">
            Favorite <Switch checked={f.favorite} onChange={(v) => set("favorite", v)} />
          </label>
        </div>

        <div className="flex justify-end gap-2 pt-1">
          <Button type="button" variant="ghost" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button type="submit" variant="primary" disabled={busy || !f.title.trim()}>
            {existing ? "Save changes" : "Save login"}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
