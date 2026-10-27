import { useEffect, useMemo, useState } from "react";
import { Archive, FileKey2, FolderOpen, Info, KeyRound, Lock, LockOpen, Palette, RotateCcw, ShieldCheck } from "lucide-react";
import { toast } from "sonner";
import { registerFlush } from "@/lib/flush";
import { api, errorText } from "@/lib/ipc";
import { ACCENTS, STICKY_COLORS } from "@/lib/palette";
import type { Settings, ThemeMode } from "@/lib/types";
import { cn, debounce } from "@/lib/utils";
import { useApp } from "@/store/app";
import { Button, Field, Input, Modal, Segmented, Slider, Switch } from "@/components/ui";
import RestoreDialog from "@/features/unlock/RestoreDialog";
import OpenSourceDialog from "./OpenSourceDialog";
import { DisableSecurityDialog, EnableSecurityDialog } from "./SecurityDialogs";

const THEMES: { value: ThemeMode; label: string; bg: string; side: string; fg: string; line: string }[] = [
  { value: "black", label: "Black", bg: "#000000", side: "#0b0b0b", fg: "#f4f4f4", line: "rgba(255,255,255,0.14)" },
  { value: "graphite", label: "Graphite", bg: "#191919", side: "#202020", fg: "#ececec", line: "rgba(255,255,255,0.12)" },
  { value: "white", label: "White", bg: "#ffffff", side: "#f7f7f5", fg: "#1f1f1f", line: "rgba(0,0,0,0.12)" },
  { value: "paper", label: "Paper", bg: "#faf6ee", side: "#f1e9da", fg: "#28231c", line: "rgba(70,45,10,0.18)" },
  { value: "system", label: "Match Windows", bg: "linear-gradient(90deg,#000 50%,#fff 50%)", side: "transparent", fg: "#888", line: "rgba(128,128,128,0.4)" },
];

export default function SettingsView() {
  const settings = useApp((s) => s.settings);
  const setSettings = useApp((s) => s.setSettings);
  const status = useApp((s) => s.status);
  const [licensesOpen, setLicensesOpen] = useState(false);
  const [changeOpen, setChangeOpen] = useState(false);
  const [restoreOpen, setRestoreOpen] = useState(false);
  const [enableOpen, setEnableOpen] = useState(false);
  const [disableOpen, setDisableOpen] = useState(false);
  const securityOn = status?.securityOn ?? false;

  const persist = useMemo(
    () =>
      debounce(
        (s: Settings) => api.saveSettings(s).then(() => {}, (e) => void toast.error(errorText(e))),
        350,
      ),
    [],
  );

  // Write a pending change before the vault locks or the app closes.
  useEffect(() => registerFlush(() => persist.flush()), [persist]);

  if (!settings) return null;

  const update = (patch: Partial<Settings>) => {
    const next = { ...settings, ...patch };
    setSettings(next); // applies instantly
    persist(next); // saved (encrypted) shortly after
  };

  return (
    <div className="h-full overflow-y-auto">
      <div className="mx-auto max-w-3xl px-10 pb-20 pt-14">
        <h1 className="mb-10 text-[2rem] font-bold tracking-tight">Settings</h1>

        <Section icon={<Palette size={16} />} title="Appearance">
          <div className="pb-5">
            <div className="mb-3 text-sm">Theme</div>
            <div className="grid grid-cols-5 gap-3">
              {THEMES.map((t) => (
                <button
                  key={t.value}
                  onClick={() => update({ theme: t.value })}
                  className={cn(
                    "group rounded-xl border p-1.5 text-left transition-colors hover:bg-surface",
                    settings.theme === t.value ? "border-fg" : "border-line",
                  )}
                >
                  <div className="flex h-16 overflow-hidden rounded-lg border" style={{ background: t.bg, borderColor: t.line }}>
                    {t.value !== "system" && (
                      <>
                        <div className="w-1/4" style={{ background: t.side, borderRight: `1px solid ${t.line}` }} />
                        <div className="flex-1 space-y-1.5 p-2">
                          <div className="h-1.5 w-2/3 rounded-full" style={{ background: t.fg, opacity: 0.85 }} />
                          <div className="h-1 w-full rounded-full" style={{ background: t.fg, opacity: 0.25 }} />
                          <div className="h-1 w-4/5 rounded-full" style={{ background: t.fg, opacity: 0.25 }} />
                        </div>
                      </>
                    )}
                  </div>
                  <div className="px-1 pb-0.5 pt-1.5 text-xs">{t.label}</div>
                </button>
              ))}
            </div>
          </div>
          <Row label="Accent color" hint="Used for buttons, checkboxes and highlights. Black & white follows the theme.">
            <div className="flex flex-wrap items-center justify-end gap-2">
              {ACCENTS.map((a) => (
                <button
                  key={a.value}
                  title={a.name}
                  aria-label={a.name}
                  onClick={() => update({ accent: a.value })}
                  className={cn(
                    "h-6 w-6 rounded-full border border-line transition-transform hover:scale-110",
                    settings.accent.toLowerCase() === a.value && "ring-2 ring-fg ring-offset-2 ring-offset-[var(--bg)]",
                  )}
                  style={{ background: a.value === "mono" ? "linear-gradient(135deg, #000 50%, #fff 50%)" : a.value }}
                />
              ))}
              <label className="relative h-6 w-6 cursor-pointer overflow-hidden rounded-full border border-line" title="Custom color">
                <input
                  type="color"
                  value={/^#[0-9a-f]{6}$/i.test(settings.accent) ? settings.accent : "#888888"}
                  onChange={(e) => update({ accent: e.target.value })}
                  className="absolute -inset-2 h-12 w-12 cursor-pointer"
                />
              </label>
            </div>
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
          <Row label={`Text size · ${settings.fontSize}px`} hint="Scales the whole app. Change one piece of text inside a note by selecting it.">
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
                { value: "full", label: "Full" },
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
          <Row label="Window material" hint="Mica and Acrylic need Windows 11. Otherwise a solid background is used.">
            <Segmented
              value={settings.backdrop}
              onChange={(v) => update({ backdrop: v })}
              options={[
                { value: "none", label: "Solid" },
                { value: "mica", label: "Mica" },
                { value: "acrylic", label: "Acrylic" },
              ]}
            />
          </Row>
          <Row label="Lock screen">
            <Segmented
              value={settings.unlockBackground}
              onChange={(v) => update({ unlockBackground: v })}
              options={[
                { value: "midnight", label: "Glow" },
                { value: "plain", label: "Plain" },
                { value: "mesh", label: "Mesh" },
                { value: "aurora", label: "Aurora" },
              ]}
            />
          </Row>
          <Row label="New sticky note color">
            <div className="flex gap-1.5">
              {Object.entries(STICKY_COLORS).map(([key, c]) => (
                <button
                  key={key}
                  title={c.label}
                  aria-label={c.label}
                  onClick={() => update({ stickyDefaultColor: key })}
                  className={cn(
                    "h-6 w-6 rounded-md ring-1 ring-black/10 transition-transform hover:scale-110",
                    settings.stickyDefaultColor === key && "ring-2 ring-fg ring-offset-2 ring-offset-[var(--bg)]",
                  )}
                  style={{ background: c.paper }}
                />
              ))}
            </div>
          </Row>
        </Section>

        <Section icon={<ShieldCheck size={16} />} title="Security (optional)">
          {securityOn ? (
            <>
              <Row label="Security is on" hint="Your notes need the master password to open. Once you have unlocked them, you can turn security off here at any time (it asks for the password again).">
                <Button variant="outline" size="sm" onClick={() => setDisableOpen(true)}>
                  <LockOpen size={14} /> Turn off security…
                </Button>
              </Row>
              <Row label={`Auto-lock after ${settings.idleLockMinutes} min of inactivity`} hint="Also locks on Win+L, sleep, hibernate, and when the USB drive is removed.">
                <div className="w-56">
                  <Slider label="Auto-lock minutes" value={settings.idleLockMinutes} min={1} max={60} onChange={(v) => update({ idleLockMinutes: v })} />
                </div>
              </Row>
              <Row label="Lock when minimized">
                <Switch checked={settings.lockOnMinimize} onChange={(v) => update({ lockOnMinimize: v })} />
              </Row>
              <Row label="Block screenshots & screen sharing" hint="Off by default. When on, Cipherleaf windows appear black in screenshots, recordings and screen shares.">
                <Switch checked={settings.screenCaptureProtection} onChange={(v) => update({ screenCaptureProtection: v })} />
              </Row>
              <Row label="Master password & keyfile">
                <Button variant="outline" size="sm" onClick={() => setChangeOpen(true)}>
                  <KeyRound size={14} /> Change…
                </Button>
              </Row>
            </>
          ) : (
            <Row
              label="Security is off"
              hint="Your notes open straight away and nothing locks. Anyone who can open this folder can read them. Turn security on to add a master password, an optional keyfile, auto-lock and screenshot blocking."
            >
              <Button variant="primary" size="sm" onClick={() => setEnableOpen(true)}>
                <Lock size={14} /> Turn on…
              </Button>
            </Row>
          )}
        </Section>

        <Section icon={<Archive size={16} />} title="Backups">
          <p className="pb-3 text-sm leading-relaxed text-muted">
            Each time Cipherleaf locks or closes after changes, a backup is saved to <code className="font-mono text-xs">data\backups</code> (the last 10 are kept).
            Backups open with the password and keyfile they had when they were made; a backup made while security was off needs no password.
            Restoring puts a backup in place of your current vault, and keeps the current vault as a separate file.
          </p>
          <div className="flex flex-wrap gap-2 pb-2">
            <Button
              variant="outline"
              size="sm"
              onClick={async () => {
                try {
                  const p = await api.backupNow();
                  if (p) toast.success("Backup saved.", { description: p });
                } catch (e) {
                  toast.error(errorText(e));
                }
              }}
            >
              <Archive size={14} /> Save a backup copy…
            </Button>
            <Button variant="outline" size="sm" onClick={() => setRestoreOpen(true)}>
              <RotateCcw size={14} /> Restore from a backup…
            </Button>
            <Button variant="ghost" size="sm" onClick={() => api.openDataFolder()}>
              <FolderOpen size={14} /> Open data folder
            </Button>
          </div>
        </Section>

        <Section icon={<Info size={16} />} title="About">
          <dl className="grid grid-cols-[160px_1fr] gap-y-2.5 pb-4 text-sm">
            <dt className="text-faint">Version</dt>
            <dd>{status?.version}</dd>
            <dt className="text-faint">Vault location</dt>
            <dd className="selectable break-all font-mono text-xs">{status?.dataDir}</dd>
            <dt className="text-faint">Encryption</dt>
            <dd>
              {securityOn
                ? "XChaCha20-Poly1305 (256-bit) for every item, with tamper detection"
                : "XChaCha20-Poly1305 (256-bit) file format with tamper detection. With security off the key is built into the program, so this does not keep anyone out"}
            </dd>
            <dt className="text-faint">Key derivation</dt>
            <dd>{securityOn ? "Argon2id · 256 MiB · 3 passes · 4 lanes, + HKDF-SHA256 with optional keyfile" : "None while security is off: the notes use a built-in key, so they are not secret"}</dd>
            <dt className="text-faint">Network</dt>
            <dd>Cipherleaf's own code makes none. The Windows WebView2 component that draws the window may contact Microsoft at startup.</dd>
            <dt className="text-faint">Credits</dt>
            <dd>
              Built on open source software.{" "}
              <button type="button" onClick={() => setLicensesOpen(true)} className="font-medium text-accent underline-offset-4 hover:underline">
                Open source licenses…
              </button>
            </dd>
          </dl>
          <div className="rounded-xl border border-line bg-surface p-4 text-sm leading-relaxed text-muted">
            <div className="mb-1 font-medium text-fg">Good to know</div>
            <ul className="list-disc space-y-1 pl-5">
              <li>With security on, there is no password reset. A forgotten master password or lost keyfile means the data cannot be recovered.</li>
              <li>Malware or a keylogger already on a PC can capture what you type while the vault is unlocked. Only unlock on PCs you trust.</li>
              <li>Cipherleaf no longer includes KeePassXC. A passwords file made with an earlier version (a .kdbx file in the data folder, and its copies in data\backups) is left as it is and still opens in KeePassXC itself.</li>
              <li>Equations are drawn by KaTeX 0.19.0 (MIT licence, fonts under the SIL Open Font License), running on this device only.</li>
            </ul>
          </div>
        </Section>
      </div>

      <OpenSourceDialog open={licensesOpen} onOpenChange={setLicensesOpen} />
      <ChangeMasterDialog open={changeOpen} onOpenChange={setChangeOpen} />
      <EnableSecurityDialog open={enableOpen} onOpenChange={setEnableOpen} />
      <DisableSecurityDialog open={disableOpen} onOpenChange={setDisableOpen} keyfileUsed={status?.keyfileRequired ?? false} />
      <RestoreDialog open={restoreOpen} onOpenChange={setRestoreOpen} vaultExists />
    </div>
  );
}

function Section({ icon, title, children }: { icon: React.ReactNode; title: string; children: React.ReactNode }) {
  return (
    <section className="mb-10">
      <h2 className="mb-1 flex items-center gap-2 border-b border-line pb-3 text-sm font-semibold">
        <span className="text-muted">{icon}</span>
        {title}
      </h2>
      <div className="divide-y divide-line/60 [&>*]:py-3.5">{children}</div>
    </section>
  );
}

function Row({ label, hint, children }: { label: string; hint?: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-8">
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
      description="Changing the password also makes a new data key and re-encrypts your notes, which takes a moment. Notes saved from now on can't be opened with your old password. Older backups still open with the old password, so keep them somewhere safe or delete them."
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
