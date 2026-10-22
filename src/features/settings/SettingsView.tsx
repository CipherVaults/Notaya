import { useMemo, useState } from "react";
import { Archive, FolderOpen, KeyRound, Lock, Palette, ShieldCheck, Info, FileKey2 } from "lucide-react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import { ACCENTS, STICKY_COLORS } from "@/lib/palette";
import type { Settings } from "@/lib/types";
import { cn, debounce } from "@/lib/utils";
import { useApp } from "@/store/app";
import { Button, Field, Input, Modal, Segmented, Slider, Switch } from "@/components/ui";

export default function SettingsView() {
  const settings = useApp((s) => s.settings);
  const setSettings = useApp((s) => s.setSettings);
  const status = useApp((s) => s.status);
  const [changeOpen, setChangeOpen] = useState(false);

  const persist = useMemo(
    () =>
      debounce((s: Settings) => {
        api.saveSettings(s).catch((e) => toast.error(errorText(e)));
      }, 350),
    [],
  );

  if (!settings) return null;

  const update = (patch: Partial<Settings>) => {
    const next = { ...settings, ...patch };
    setSettings(next); // applies instantly
    persist(next); // saved (encrypted) shortly after
  };

  return (
    <div className="h-full overflow-y-auto">
      <div className="mx-auto max-w-3xl space-y-8 px-10 py-10">
        <h2 className="text-2xl font-semibold tracking-tight">Settings</h2>

        <Section icon={<Palette size={17} />} title="Appearance">
          <Row label="Theme">
            <Segmented
              value={settings.theme}
              onChange={(v) => update({ theme: v })}
              options={[
                { value: "dark", label: "Dark" },
                { value: "light", label: "Light" },
                { value: "system", label: "System" },
              ]}
            />
          </Row>
          <Row label="Accent color">
            <div className="flex items-center gap-2">
              {ACCENTS.map((a) => (
                <button
                  key={a.value}
                  title={a.name}
                  onClick={() => update({ accent: a.value })}
                  className={cn(
                    "h-7 w-7 rounded-full transition-transform hover:scale-110",
                    settings.accent.toLowerCase() === a.value && "ring-2 ring-offset-2 ring-offset-[var(--bg)]",
                  )}
                  style={{ background: a.value, ["--tw-ring-color" as string]: a.value }}
                />
              ))}
              <label className="relative h-7 w-7 cursor-pointer overflow-hidden rounded-full border border-line" title="Custom color">
                <input
                  type="color"
                  value={settings.accent}
                  onChange={(e) => update({ accent: e.target.value })}
                  className="absolute -inset-2 h-12 w-12 cursor-pointer"
                />
              </label>
            </div>
          </Row>
          <Row label="Window material" hint="Mica and Acrylic need Windows 11. Otherwise a solid background is used.">
            <Segmented
              value={settings.backdrop}
              onChange={(v) => update({ backdrop: v })}
              options={[
                { value: "mica", label: "Mica" },
                { value: "acrylic", label: "Acrylic" },
                { value: "none", label: "Solid" },
              ]}
            />
          </Row>
          <Row label="Font">
            <Segmented
              value={settings.font}
              onChange={(v) => update({ font: v })}
              options={[
                { value: "inter", label: "Inter" },
                { value: "system", label: "Segoe" },
                { value: "serif", label: "Serif" },
                { value: "mono", label: "Mono" },
              ]}
            />
          </Row>
          <Row label={`Text size · ${settings.fontSize}px`}>
            <div className="w-56">
              <Slider label="Text size" value={settings.fontSize} min={12} max={20} onChange={(v) => update({ fontSize: v })} />
            </div>
          </Row>
          <Row label="Page width">
            <Segmented
              value={settings.editorWidth}
              onChange={(v) => update({ editorWidth: v })}
              options={[
                { value: "narrow", label: "Narrow" },
                { value: "normal", label: "Normal" },
                { value: "wide", label: "Wide" },
              ]}
            />
          </Row>
          <Row label="Density">
            <Segmented
              value={settings.density}
              onChange={(v) => update({ density: v })}
              options={[
                { value: "comfortable", label: "Comfortable" },
                { value: "compact", label: "Compact" },
              ]}
            />
          </Row>
          <Row label="Lock screen">
            <Segmented
              value={settings.unlockBackground}
              onChange={(v) => update({ unlockBackground: v })}
              options={[
                { value: "aurora", label: "Aurora" },
                { value: "mesh", label: "Mesh" },
                { value: "midnight", label: "Midnight" },
                { value: "plain", label: "Plain" },
              ]}
            />
          </Row>
          <Row label="New sticky note color">
            <div className="flex gap-1.5">
              {Object.entries(STICKY_COLORS).map(([key, c]) => (
                <button
                  key={key}
                  title={c.label}
                  onClick={() => update({ stickyDefaultColor: key })}
                  className={cn(
                    "h-6 w-6 rounded-md ring-1 ring-black/10 transition-transform hover:scale-110",
                    settings.stickyDefaultColor === key && "ring-2 ring-accent",
                  )}
                  style={{ background: c.paper }}
                />
              ))}
            </div>
          </Row>
        </Section>

        <Section icon={<ShieldCheck size={17} />} title="Security">
          <Row label={`Auto-lock after ${settings.idleLockMinutes} min of inactivity`} hint="Also locks on Win+L, sleep, hibernate, and when the USB drive is removed.">
            <div className="w-56">
              <Slider label="Auto-lock minutes" value={settings.idleLockMinutes} min={1} max={60} onChange={(v) => update({ idleLockMinutes: v })} />
            </div>
          </Row>
          <Row label="Lock when minimized">
            <Switch checked={settings.lockOnMinimize} onChange={(v) => update({ lockOnMinimize: v })} />
          </Row>
          <Row label="Block screenshots & screen sharing" hint="Cipherleaf windows appear black in screenshots, recordings and screen shares.">
            <Switch checked={settings.screenCaptureProtection} onChange={(v) => update({ screenCaptureProtection: v })} />
          </Row>
          <Row label="Master password & keyfile">
            <Button variant="outline" size="sm" onClick={() => setChangeOpen(true)}>
              <KeyRound size={14} /> Change…
            </Button>
          </Row>
        </Section>

        <Section icon={<Archive size={17} />} title="Backups">
          <p className="text-sm leading-relaxed text-muted">
            Each time the vault locks after changes, an encrypted backup is saved to <code className="font-mono text-xs">data\backups</code>{" "}
            (the last 10 are kept). Backups open with the same password and keyfile.
          </p>
          <div className="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={async () => {
                try {
                  const p = await api.backupNow();
                  if (p) toast.success("Encrypted backup saved.", { description: p });
                } catch (e) {
                  toast.error(errorText(e));
                }
              }}
            >
              <Archive size={14} /> Save a backup copy…
            </Button>
            <Button variant="ghost" size="sm" onClick={() => api.openDataFolder()}>
              <FolderOpen size={14} /> Open data folder
            </Button>
          </div>
        </Section>

        <Section icon={<Info size={17} />} title="About">
          <dl className="grid grid-cols-[160px_1fr] gap-y-2 text-sm">
            <dt className="text-faint">Version</dt>
            <dd>{status?.version}</dd>
            <dt className="text-faint">Vault location</dt>
            <dd className="selectable break-all font-mono text-xs">{status?.dataDir}</dd>
            <dt className="text-faint">Encryption</dt>
            <dd>XChaCha20-Poly1305 (256-bit) for every item, with tamper detection</dd>
            <dt className="text-faint">Key derivation</dt>
            <dd>Argon2id · 256 MiB · 3 passes · 4 lanes, + HKDF-SHA256 with optional keyfile</dd>
            <dt className="text-faint">Network</dt>
            <dd>None. Cipherleaf makes no network connections.</dd>
            <dt className="text-faint">Passwords</dt>
            <dd>Kept in the bundled KeePassXC 2.8.0 (GPL), which has its own master password and settings</dd>
          </dl>
          <div className="rounded-2xl border border-line bg-surface p-4 text-sm leading-relaxed text-muted">
            <div className="mb-1 font-medium text-fg">Good to know</div>
            <ul className="list-disc space-y-1 pl-5">
              <li>There is no password reset. A forgotten master password or lost keyfile means the data cannot be recovered.</li>
              <li>Malware or a keylogger already on a PC can capture what you type while the vault is unlocked. Only unlock on PCs you trust.</li>
              <li>Cipherleaf's screenshot blocking and lock-when-unplugged don't cover KeePassXC's window. KeePassXC has its own protections.</li>
              <li>Passphrase words: EFF large wordlist, © Electronic Frontier Foundation, CC BY 3.0 US.</li>
            </ul>
          </div>
        </Section>
      </div>

      <ChangeMasterDialog open={changeOpen} onOpenChange={setChangeOpen} />
    </div>
  );
}

function Section({ icon, title, children }: { icon: React.ReactNode; title: string; children: React.ReactNode }) {
  return (
    <section className="space-y-5 rounded-3xl border border-line bg-surface p-7">
      <h3 className="flex items-center gap-2 text-base font-semibold">
        <span className="text-accent">{icon}</span>
        {title}
      </h3>
      {children}
    </section>
  );
}

function Row({ label, hint, children }: { label: string; hint?: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-6">
      <div className="min-w-0">
        <div className="text-sm">{label}</div>
        {hint && <div className="mt-0.5 text-xs leading-relaxed text-faint">{hint}</div>}
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  );
}

function ChangeMasterDialog({ open, onOpenChange }: { open: boolean; onOpenChange: (v: boolean) => void }) {
  const [current, setCurrent] = useState("");
  const [currentKey, setCurrentKey] = useState<string | null>(null);
  const [next, setNext] = useState("");
  const [confirmPw, setConfirmPw] = useState("");
  const [nextKey, setNextKey] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const reset = () => {
    setCurrent("");
    setNext("");
    setConfirmPw("");
    setCurrentKey(null);
    setNextKey(null);
  };

  const submit = async () => {
    if (next !== confirmPw) {
      toast.error("The new passwords don't match.");
      return;
    }
    setBusy(true);
    try {
      await api.changeMaster(current, currentKey, next, nextKey);
      toast.success("Master password updated.");
      reset();
      onOpenChange(false);
    } catch (e) {
      toast.error(errorText(e));
    } finally {
      setBusy(false);
    }
  };

  const keyName = (p: string | null) => p?.split(/[\\/]/).pop();

  return (
    <Modal
      open={open}
      onOpenChange={(o) => {
        if (!o) reset();
        onOpenChange(o);
      }}
      title="Change master password"
      description="Your data isn't re-encrypted; only the key that protects it changes. It takes a second."
      width={500}
    >
      <form
        className="space-y-4"
        onSubmit={(e) => {
          e.preventDefault();
          submit();
        }}
      >
        <Field label="Current password">
          <Input type="password" value={current} onChange={(e) => setCurrent(e.target.value)} autoFocus />
        </Field>
        <Field label="Current keyfile (if you use one)">
          <Button type="button" variant="outline" size="sm" onClick={async () => setCurrentKey(await api.pickKeyfile())}>
            <FileKey2 size={14} /> {keyName(currentKey) ?? "Choose keyfile"}
          </Button>
        </Field>
        <Field label="New password">
          <Input type="password" value={next} onChange={(e) => setNext(e.target.value)} />
        </Field>
        <Field label="Confirm new password">
          <Input type="password" value={confirmPw} onChange={(e) => setConfirmPw(e.target.value)} />
        </Field>
        <Field label="New keyfile (optional)" hint="Leave empty to stop using a keyfile.">
          <div className="flex gap-2">
            <Button type="button" variant="outline" size="sm" onClick={async () => setNextKey(await api.createKeyfile().catch(() => null))}>
              Create new
            </Button>
            <Button type="button" variant="outline" size="sm" onClick={async () => setNextKey(await api.pickKeyfile())}>
              {keyName(nextKey) ?? "Use existing"}
            </Button>
          </div>
        </Field>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button type="submit" variant="primary" disabled={busy || !current || !next}>
            <Lock size={14} /> {busy ? "Updating…" : "Update"}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
