import { useCallback, useEffect, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { Copy, RefreshCw, ShieldAlert, ShieldCheck } from "lucide-react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import type { Generated, PassphraseOptions, PasswordOptions } from "@/lib/types";
import { useApp } from "@/store/app";
import { Button, Input, Segmented, Slider, Switch } from "@/components/ui";
import { ColoredPassword } from "@/features/passwords/PasswordsView";

type Mode = "password" | "passphrase";

/** Rough, conservative labels for entropy in bits. */
function entropyLabel(bits: number) {
  if (bits < 40) return { text: "Weak", color: "#f43f5e" };
  if (bits < 60) return { text: "Fair", color: "#f59e0b" };
  if (bits < 80) return { text: "Strong", color: "#22c55e" };
  if (bits < 128) return { text: "Very strong", color: "#10b981" };
  return { text: "Overkill (in a good way)", color: "var(--accent)" };
}

export default function GeneratorView() {
  const [mode, setMode] = useState<Mode>("password");
  const [pw, setPw] = useState<PasswordOptions>({ length: 24, lower: true, upper: true, digits: true, symbols: true, avoidAmbiguous: false });
  const [pp, setPp] = useState<PassphraseOptions>({ words: 6, separator: "-", capitalize: true, includeNumber: true });
  const [result, setResult] = useState<Generated | null>(null);
  const [nonce, setNonce] = useState(0);
  const [breach, setBreach] = useState<number | null>(null);
  const breachEnabled = useApp((s) => s.settings?.breachCheckEnabled ?? false);

  const generate = useCallback(async () => {
    try {
      setBreach(null);
      setResult(mode === "password" ? await api.generatePassword(pw) : await api.generatePassphrase(pp));
      setNonce((n) => n + 1);
    } catch (e) {
      toast.error(errorText(e));
    }
  }, [mode, pw, pp]);

  useEffect(() => {
    generate();
  }, [generate]);

  const copy = async () => {
    if (!result) return;
    const secs = await api.copySecretText(result.value);
    toast.success("Copied", { description: `Clears from the clipboard in ${secs}s.` });
  };

  const label = result ? entropyLabel(result.entropy) : null;

  return (
    <div className="h-full overflow-y-auto">
      <div className="mx-auto max-w-2xl px-10 py-10">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-semibold tracking-tight">Generator</h2>
            <p className="mt-1 text-sm text-muted">Made with your operating system's secure random generator. Nothing is saved.</p>
          </div>
          <Segmented<Mode>
            value={mode}
            onChange={setMode}
            options={[
              { value: "password", label: "Password" },
              { value: "passphrase", label: "Passphrase" },
            ]}
          />
        </div>

        <div className="relative mt-8 overflow-hidden rounded-3xl border border-line bg-elevated p-7 shadow-float">
          <div className="pointer-events-none absolute -right-20 -top-20 h-56 w-56 rounded-full opacity-25 blur-3xl" style={{ background: "var(--accent)" }} />
          <AnimatePresence mode="wait">
            <motion.div
              key={nonce}
              initial={{ opacity: 0, filter: "blur(6px)" }}
              animate={{ opacity: 1, filter: "blur(0px)" }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.22 }}
              className="selectable relative min-h-[4.5rem] break-all font-mono text-[1.55rem] leading-snug tracking-wide"
            >
              {result && <ColoredPassword value={result.value} />}
            </motion.div>
          </AnimatePresence>
          <div className="relative mt-6 flex items-center justify-between">
            {label && result && (
              <div className="text-sm">
                <span className="font-semibold" style={{ color: label.color }}>
                  {label.text}
                </span>
                <span className="text-faint"> · {Math.round(result.entropy)} bits of entropy</span>
              </div>
            )}
            <div className="flex gap-2">
              {breachEnabled && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={async () => {
                    try {
                      setBreach(await api.checkBreachText(result?.value ?? ""));
                    } catch (e) {
                      toast.error(errorText(e));
                    }
                  }}
                >
                  {breach === null ? <ShieldAlert size={15} /> : breach > 0 ? <ShieldAlert size={15} className="text-danger" /> : <ShieldCheck size={15} className="text-ok" />}
                  {breach === null ? "Breach check" : breach > 0 ? `Found ${breach}×` : "Not breached"}
                </Button>
              )}
              <Button variant="outline" size="sm" onClick={generate}>
                <RefreshCw size={15} /> Regenerate
              </Button>
              <Button variant="primary" size="sm" onClick={copy}>
                <Copy size={15} /> Copy
              </Button>
            </div>
          </div>
        </div>

        <div className="mt-6 space-y-5 rounded-3xl border border-line bg-surface p-7">
          {mode === "password" ? (
            <>
              <OptionRow label="Length" value={String(pw.length)}>
                <Slider label="Length" value={pw.length} min={8} max={128} onChange={(v) => setPw({ ...pw, length: v })} />
              </OptionRow>
              <Toggle label="Lowercase (a–z)" checked={pw.lower} onChange={(v) => setPw({ ...pw, lower: v })} />
              <Toggle label="Uppercase (A–Z)" checked={pw.upper} onChange={(v) => setPw({ ...pw, upper: v })} />
              <Toggle label="Numbers (0–9)" checked={pw.digits} onChange={(v) => setPw({ ...pw, digits: v })} />
              <Toggle label="Symbols (!@#$…)" checked={pw.symbols} onChange={(v) => setPw({ ...pw, symbols: v })} />
              <Toggle label="Avoid look-alikes (I l 1 O 0 o)" checked={pw.avoidAmbiguous} onChange={(v) => setPw({ ...pw, avoidAmbiguous: v })} />
            </>
          ) : (
            <>
              <OptionRow label="Words" value={String(pp.words)}>
                <Slider label="Words" value={pp.words} min={3} max={12} onChange={(v) => setPp({ ...pp, words: v })} />
              </OptionRow>
              <div className="flex items-center justify-between">
                <span className="text-sm">Separator</span>
                <Input value={pp.separator} maxLength={3} onChange={(e) => setPp({ ...pp, separator: e.target.value })} className="h-9 w-20 text-center font-mono" />
              </div>
              <Toggle label="Capitalize words" checked={pp.capitalize} onChange={(v) => setPp({ ...pp, capitalize: v })} />
              <Toggle label="Include a number" checked={pp.includeNumber} onChange={(v) => setPp({ ...pp, includeNumber: v })} />
              <p className="text-xs text-faint">Words come from the EFF large wordlist (7,776 words, CC BY 3.0 US, Electronic Frontier Foundation).</p>
            </>
          )}
        </div>
      </div>
    </div>
  );
}

function OptionRow({ label, value, children }: { label: string; value: string; children: React.ReactNode }) {
  return (
    <div className="space-y-3">
      <div className="flex justify-between text-sm">
        <span>{label}</span>
        <span className="font-mono font-semibold text-accent">{value}</span>
      </div>
      {children}
    </div>
  );
}

function Toggle({ label, checked, onChange }: { label: string; checked: boolean; onChange: (v: boolean) => void }) {
  return (
    <div className="flex items-center justify-between">
      <span className="text-sm">{label}</span>
      <Switch checked={checked} onChange={onChange} label={label} />
    </div>
  );
}
