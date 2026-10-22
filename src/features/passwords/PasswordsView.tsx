import { useCallback, useEffect, useMemo, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import {
  Copy,
  ExternalLink,
  Eye,
  EyeOff,
  FileUp,
  History,
  KeyRound,
  Loader2,
  MoreHorizontal,
  Pencil,
  Plus,
  Search,
  ShieldAlert,
  ShieldCheck,
  Star,
  Trash2,
} from "lucide-react";
import { toast } from "sonner";
import { api, errorText } from "@/lib/ipc";
import type { Health, Login, LoginSummary, Strength } from "@/lib/types";
import { cn, domainOf, formatDate, initials, timeAgo } from "@/lib/utils";
import { useApp } from "@/store/app";
import {
  Button,
  EmptyState,
  IconButton,
  Menu,
  MenuContent,
  MenuItem,
  MenuSeparator,
  MenuTrigger,
  StrengthMeter,
  useConfirm,
} from "@/components/ui";
import LoginForm from "./LoginForm";
import TotpRing from "./TotpRing";

type Filter = { kind: "all" } | { kind: "favorites" } | { kind: "totp" } | { kind: "folder"; name: string };

export default function PasswordsView() {
  const logins = useApp((s) => s.logins);
  const setLogins = useApp((s) => s.setLogins);
  const upsertLogin = useApp((s) => s.upsertLogin);
  const activeId = useApp((s) => s.activeLoginId);
  const openLogin = useApp((s) => s.openLogin);
  const intent = useApp((s) => s.intent);
  const setIntent = useApp((s) => s.setIntent);
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<Filter>({ kind: "all" });
  const [formOpen, setFormOpen] = useState(false);
  const [editing, setEditing] = useState<Login | null>(null);
  const [showHealth, setShowHealth] = useState(false);

  useEffect(() => {
    if (intent === "new-login") {
      setIntent(null);
      setEditing(null);
      setFormOpen(true);
    }
  }, [intent, setIntent]);

  const folders = useMemo(
    () => [...new Set(logins.map((l) => l.folder).filter(Boolean))].sort(),
    [logins],
  );

  const visible = useMemo(() => {
    const q = query.trim().toLowerCase();
    return logins.filter((l) => {
      if (filter.kind === "favorites" && !l.favorite) return false;
      if (filter.kind === "totp" && !l.hasTotp) return false;
      if (filter.kind === "folder" && l.folder !== filter.name) return false;
      if (!q) return true;
      return [l.title, l.username, l.url, l.folder, ...l.tags].some((v) => v.toLowerCase().includes(q));
    });
  }, [logins, query, filter]);

  const importCsv = async () => {
    try {
      const r = await api.importCsv();
      if (!r) return;
      setLogins(await api.listLogins());
      toast.success(`Imported ${r.imported} login(s)${r.skipped ? `, skipped ${r.skipped}` : ""}.`, {
        description: "For safety, delete the CSV file now. It contains your passwords in plain text.",
        duration: 10000,
      });
    } catch (e) {
      toast.error(errorText(e));
    }
  };

  const newLogin = () => {
    setEditing(null);
    setFormOpen(true);
  };

  return (
    <div className="flex h-full">
      {/* List */}
      <div className="flex w-[330px] shrink-0 flex-col border-r border-line">
        <div className="space-y-3 p-3">
          <div className="flex gap-2">
            <div className="relative flex-1">
              <Search size={14} className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-faint" />
              <input
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="Filter logins"
                className="h-9 w-full rounded-xl border border-line bg-surface pl-8 pr-3 text-sm outline-none placeholder:text-faint focus:border-accent"
              />
            </div>
            <Button variant="primary" size="sm" className="h-9 px-3" onClick={newLogin}>
              <Plus size={15} /> New
            </Button>
            <Menu>
              <MenuTrigger asChild>
                <button className="grid h-9 w-9 place-items-center rounded-xl border border-line text-muted hover:bg-surface-2 hover:text-fg" aria-label="More">
                  <MoreHorizontal size={16} />
                </button>
              </MenuTrigger>
              <MenuContent>
                <MenuItem icon={<ShieldCheck size={15} />} onSelect={() => setShowHealth(true)}>
                  Security report
                </MenuItem>
                <MenuItem icon={<FileUp size={15} />} onSelect={importCsv}>
                  Import from CSV…
                </MenuItem>
              </MenuContent>
            </Menu>
          </div>
          <div className="flex flex-wrap gap-1.5">
            <Chip active={filter.kind === "all"} onClick={() => setFilter({ kind: "all" })}>
              All {logins.length}
            </Chip>
            <Chip active={filter.kind === "favorites"} onClick={() => setFilter({ kind: "favorites" })}>
              Favorites
            </Chip>
            <Chip active={filter.kind === "totp"} onClick={() => setFilter({ kind: "totp" })}>
              2FA
            </Chip>
            {folders.map((name) => (
              <Chip key={name} active={filter.kind === "folder" && filter.name === name} onClick={() => setFilter({ kind: "folder", name })}>
                {name}
              </Chip>
            ))}
          </div>
        </div>

        <div className="min-h-0 flex-1 space-y-0.5 overflow-y-auto px-2 pb-3">
          {visible.map((l) => (
            <LoginRow
              key={l.id}
              login={l}
              active={!showHealth && l.id === activeId}
              onClick={() => {
                setShowHealth(false);
                openLogin(l.id);
              }}
            />
          ))}
          {visible.length === 0 && logins.length > 0 && <p className="py-10 text-center text-sm text-faint">No matches.</p>}
        </div>
      </div>

      {/* Detail */}
      <div className="min-w-0 flex-1">
        {showHealth ? (
          <HealthReport onOpen={(id) => { setShowHealth(false); openLogin(id); }} />
        ) : logins.length === 0 ? (
          <EmptyState
            icon={<KeyRound size={28} />}
            title="No passwords yet"
            body="Add your first login, or import from your browser's CSV export (Chrome, Edge, Firefox, Bitwarden, 1Password)."
            action={
              <div className="flex gap-2">
                <Button variant="primary" onClick={newLogin}>
                  <Plus size={16} /> Add login
                </Button>
                <Button variant="outline" onClick={importCsv}>
                  <FileUp size={16} /> Import CSV
                </Button>
              </div>
            }
          />
        ) : activeId ? (
          <LoginDetail
            key={activeId}
            id={activeId}
            onEdit={(l) => {
              setEditing(l);
              setFormOpen(true);
            }}
          />
        ) : (
          <EmptyState icon={<KeyRound size={28} />} title="Select a login" body="Pick a login on the left to see its details, copy its password or 2FA code." />
        )}
      </div>

      <LoginForm
        open={formOpen}
        onOpenChange={(o) => {
          setFormOpen(o);
          if (!o) setEditing(null);
        }}
        existing={editing}
        onSaved={(l) => {
          upsertLogin(l);
          openLogin(l.id);
          setShowHealth(false);
        }}
      />
    </div>
  );
}

function Chip({ active, onClick, children }: { active: boolean; onClick: () => void; children: React.ReactNode }) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "rounded-full px-2.5 py-1 text-xs font-medium transition-colors",
        active ? "bg-accent text-white" : "bg-surface-2 text-muted hover:text-fg",
      )}
    >
      {children}
    </button>
  );
}

export function Avatar({ title, color, size = 36 }: { title: string; color: string; size?: number }) {
  return (
    <div
      className="grid shrink-0 place-items-center rounded-xl font-semibold text-white"
      style={{
        width: size,
        height: size,
        fontSize: size * 0.36,
        background: `linear-gradient(135deg, ${color || "var(--accent)"}, color-mix(in oklab, ${color || "var(--accent)"} 55%, black))`,
      }}
    >
      {initials(title)}
    </div>
  );
}

function LoginRow({ login, active, onClick }: { login: LoginSummary; active: boolean; onClick: () => void }) {
  return (
    <div
      onClick={onClick}
      className={cn(
        "flex cursor-pointer items-center gap-3 rounded-xl px-2.5 transition-colors",
        active ? "bg-accent-soft" : "hover:bg-surface-2",
      )}
      style={{ paddingTop: "var(--row-pad)", paddingBottom: "var(--row-pad)" }}
    >
      <Avatar title={login.title} color={login.color} />
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-1.5">
          <span className="truncate text-sm font-medium">{login.title}</span>
          {login.favorite && <Star size={11} className="shrink-0 text-warn" fill="currentColor" />}
        </div>
        <div className="truncate text-xs text-muted">{login.username || domainOf(login.url) || "—"}</div>
      </div>
      {login.hasTotp && <span className="rounded-md bg-surface-2 px-1.5 py-0.5 text-[0.65rem] font-semibold text-muted">2FA</span>}
    </div>
  );
}

function LoginDetail({ id, onEdit }: { id: string; onEdit: (l: Login) => void }) {
  const [login, setLogin] = useState<Login | null>(null);
  const [reveal, setReveal] = useState(false);
  const [strength, setStrength] = useState<Strength | null>(null);
  const [breach, setBreach] = useState<number | null | "loading">(null);
  const [showHistory, setShowHistory] = useState(false);
  const upsertLogin = useApp((s) => s.upsertLogin);
  const removeLogin = useApp((s) => s.removeLogin);
  const openLogin = useApp((s) => s.openLogin);
  const summary = useApp((s) => s.logins.find((l) => l.id === id));
  const breachEnabled = useApp((s) => s.settings?.breachCheckEnabled ?? false);
  const confirm = useConfirm();

  useEffect(() => {
    api
      .getLogin(id)
      .then((l) => {
        setLogin(l);
        if (l.password) api.passwordStrength(l.password, [l.username, l.title]).then(setStrength);
      })
      .catch((e) => toast.error(errorText(e)));
  }, [id, summary?.updated]);

  const copy = async (field: string, label: string) => {
    try {
      const secs = await api.copyLoginField(id, field);
      toast.success(`${label} copied`, { description: `Clipboard clears in ${secs}s and is kept out of clipboard history.` });
    } catch (e) {
      toast.error(errorText(e));
    }
  };

  const totpFetch = useCallback(() => api.totpCode(id), [id]);

  if (!login) {
    return (
      <div className="grid h-full place-items-center text-muted">
        <Loader2 className="animate-spin" size={20} />
      </div>
    );
  }

  const del = async () => {
    const ok = await confirm({
      title: `Delete "${login.title}"?`,
      body: "This login will be permanently deleted from your vault.",
      confirm: "Delete",
      danger: true,
    });
    if (!ok) return;
    await api.deleteLogin(id);
    removeLogin(id);
    openLogin(null);
    toast.success("Login deleted.");
  };

  const checkBreach = async () => {
    setBreach("loading");
    try {
      setBreach(await api.checkBreach(id));
    } catch (e) {
      setBreach(null);
      toast.error(errorText(e));
    }
  };

  return (
    <div className="h-full overflow-y-auto">
      <div className="mx-auto max-w-2xl px-10 py-10">
        <div className="mb-8 flex items-center gap-4">
          <Avatar title={login.title} color={login.color} size={56} />
          <div className="min-w-0 flex-1">
            <h2 className="truncate text-2xl font-semibold tracking-tight">{login.title}</h2>
            {login.urls[0] && <div className="truncate text-sm text-muted">{domainOf(login.urls[0])}</div>}
          </div>
          <IconButton
            label={login.favorite ? "Remove from favorites" : "Add to favorites"}
            onClick={async () => {
              const s = await api.setLoginFavorite(id, !login.favorite);
              upsertLogin(s);
              setLogin({ ...login, favorite: !login.favorite });
            }}
          >
            <Star size={17} className={login.favorite ? "text-warn" : ""} fill={login.favorite ? "currentColor" : "none"} />
          </IconButton>
          <Button variant="outline" size="sm" onClick={() => onEdit(login)}>
            <Pencil size={14} /> Edit
          </Button>
          <Menu>
            <MenuTrigger asChild>
              <button className="rounded-lg p-2 text-muted hover:bg-surface-2 hover:text-fg" aria-label="More">
                <MoreHorizontal size={17} />
              </button>
            </MenuTrigger>
            <MenuContent>
              {breachEnabled && (
                <MenuItem icon={<ShieldAlert size={15} />} onSelect={checkBreach}>
                  Check for breaches
                </MenuItem>
              )}
              <MenuSeparator />
              <MenuItem danger icon={<Trash2 size={15} />} onSelect={del}>
                Delete login
              </MenuItem>
            </MenuContent>
          </Menu>
        </div>

        <div className="divide-y divide-line overflow-hidden rounded-2xl border border-line bg-surface">
          {login.username && (
            <Row label="Username">
              <span className="selectable truncate">{login.username}</span>
              <CopyButton onClick={() => copy("username", "Username")} />
            </Row>
          )}
          {login.password && (
            <Row label="Password">
              <span className="selectable truncate font-mono text-[0.95rem] tracking-wide">
                {reveal ? <ColoredPassword value={login.password} /> : "•".repeat(Math.min(login.password.length, 18))}
              </span>
              <div className="flex shrink-0 items-center">
                <IconButton size="sm" label={reveal ? "Hide" : "Reveal"} onClick={() => setReveal(!reveal)}>
                  {reveal ? <EyeOff size={15} /> : <Eye size={15} />}
                </IconButton>
                <CopyButton onClick={() => copy("password", "Password")} />
              </div>
            </Row>
          )}
          {login.totp && (
            <Row label="One-time code">
              <TotpRing fetchCode={totpFetch} onCopy={() => copy("totp", "Code")} />
              <CopyButton onClick={() => copy("totp", "Code")} />
            </Row>
          )}
          {login.urls.map((u, i) => (
            <Row key={i} label={i === 0 ? "Website" : `Website ${i + 1}`}>
              <span className="selectable truncate text-accent">{u}</span>
              <IconButton size="sm" label="Open in browser" onClick={() => api.openUrl(u).catch((e) => toast.error(errorText(e)))}>
                <ExternalLink size={15} />
              </IconButton>
            </Row>
          ))}
          {login.fields.map((f, i) => (
            <Row key={`f${i}`} label={f.name || "Field"}>
              <span className="selectable truncate">{f.secret ? "•".repeat(12) : f.value}</span>
              <CopyButton onClick={() => copy(`field:${i}`, f.name || "Field")} />
            </Row>
          ))}
        </div>

        {strength && login.password && (
          <div className="mt-5 rounded-2xl border border-line bg-surface p-4">
            <StrengthMeter score={strength.score} label={`~${strength.crackTime} to crack`} />
            {strength.warning && <p className="mt-2 text-xs text-warn">{strength.warning}</p>}
          </div>
        )}

        <AnimatePresence>
          {breach !== null && (
            <motion.div
              initial={{ opacity: 0, y: 6 }}
              animate={{ opacity: 1, y: 0 }}
              className={cn(
                "mt-4 flex items-center gap-3 rounded-2xl border p-4 text-sm",
                breach === "loading" ? "border-line" : breach > 0 ? "border-danger/40 bg-danger/10" : "border-ok/40 bg-ok/10",
              )}
            >
              {breach === "loading" ? (
                <>
                  <Loader2 size={16} className="animate-spin" /> Checking (only a 5-character hash prefix is sent)…
                </>
              ) : breach > 0 ? (
                <>
                  <ShieldAlert size={18} className="text-danger" /> This password appeared in {breach.toLocaleString()} known data
                  breaches. Change it now.
                </>
              ) : (
                <>
                  <ShieldCheck size={18} className="text-ok" /> Not found in any known breach.
                </>
              )}
            </motion.div>
          )}
        </AnimatePresence>

        {login.notes && (
          <div className="mt-5">
            <div className="mb-2 text-xs font-medium uppercase tracking-wider text-faint">Notes</div>
            <p className="selectable whitespace-pre-wrap rounded-2xl border border-line bg-surface p-4 text-sm leading-relaxed">{login.notes}</p>
          </div>
        )}

        {login.tags.length > 0 && (
          <div className="mt-5 flex flex-wrap gap-1.5">
            {login.tags.map((t) => (
              <span key={t} className="rounded-full bg-surface-2 px-2.5 py-1 text-xs text-muted">
                #{t}
              </span>
            ))}
          </div>
        )}

        <div className="mt-8 space-y-1 text-xs text-faint">
          <div>Created {formatDate(login.created)}</div>
          <div>Updated {formatDate(login.updated)}</div>
          <div>Password changed {timeAgo(login.passwordChanged)}</div>
        </div>

        {login.history.length > 0 && (
          <div className="mt-5">
            <button onClick={() => setShowHistory(!showHistory)} className="flex items-center gap-1.5 text-xs font-medium text-muted hover:text-fg">
              <History size={13} /> Password history ({login.history.length})
            </button>
            {showHistory && (
              <div className="mt-2 divide-y divide-line rounded-xl border border-line">
                {login.history.map((h, i) => (
                  <div key={i} className="flex items-center justify-between px-3 py-2 text-xs">
                    <span className="selectable font-mono">{h.password}</span>
                    <span className="text-faint">{formatDate(h.changed)}</span>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

function Row({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="group flex items-center gap-4 px-4 py-3">
      <span className="w-28 shrink-0 text-xs font-medium text-faint">{label}</span>
      <div className="flex min-w-0 flex-1 items-center justify-between gap-2">{children}</div>
    </div>
  );
}

function CopyButton({ onClick }: { onClick: () => void }) {
  return (
    <IconButton size="sm" label="Copy" onClick={onClick}>
      <Copy size={14} />
    </IconButton>
  );
}

/** Digits and symbols in a different color, so passwords are easier to read out. */
export function ColoredPassword({ value }: { value: string }) {
  return (
    <>
      {[...value].map((ch, i) => (
        <span key={i} className={/[0-9]/.test(ch) ? "text-sky-400" : /[^a-zA-Z0-9]/.test(ch) ? "text-rose-400" : ""}>
          {ch}
        </span>
      ))}
    </>
  );
}

function HealthReport({ onOpen }: { onOpen: (id: string) => void }) {
  const [health, setHealth] = useState<Health | null>(null);
  useEffect(() => {
    api.passwordHealth().then(setHealth).catch((e) => toast.error(errorText(e)));
  }, []);
  if (!health) {
    return (
      <div className="grid h-full place-items-center text-muted">
        <Loader2 className="animate-spin" />
      </div>
    );
  }
  const color = health.score >= 80 ? "#10b981" : health.score >= 50 ? "#f59e0b" : "#f43f5e";
  const r = 52;
  const circ = 2 * Math.PI * r;
  return (
    <div className="h-full overflow-y-auto">
      <div className="mx-auto max-w-2xl px-10 py-10">
        <h2 className="text-2xl font-semibold tracking-tight">Security report</h2>
        <p className="mt-1 text-sm text-muted">Checked locally on this PC. Nothing was sent anywhere.</p>
        <div className="mt-8 flex items-center gap-8 rounded-2xl border border-line bg-surface p-6">
          <div className="relative h-32 w-32">
            <svg viewBox="0 0 120 120" className="h-full w-full -rotate-90">
              <circle cx="60" cy="60" r={r} stroke="var(--surface-2)" strokeWidth="10" fill="none" />
              <motion.circle
                cx="60"
                cy="60"
                r={r}
                stroke={color}
                strokeWidth="10"
                fill="none"
                strokeLinecap="round"
                strokeDasharray={circ}
                initial={{ strokeDashoffset: circ }}
                animate={{ strokeDashoffset: circ * (1 - health.score / 100) }}
                transition={{ duration: 1, ease: "easeOut" }}
              />
            </svg>
            <div className="absolute inset-0 grid place-items-center">
              <span className="text-3xl font-bold" style={{ color }}>
                {health.score}
              </span>
            </div>
          </div>
          <div className="grid flex-1 grid-cols-3 gap-4 text-center">
            <Stat n={health.weak.length} label="Weak" />
            <Stat n={health.reused.length} label="Reused" />
            <Stat n={health.old.length} label="Older than 1 year" />
          </div>
        </div>
        <IssueList title="Weak passwords" items={health.weak} onOpen={onOpen} />
        <IssueList title="Reused passwords" items={health.reused} onOpen={onOpen} />
        <IssueList title="Old passwords" items={health.old} onOpen={onOpen} />
        {health.total > 0 && health.weak.length + health.reused.length + health.old.length === 0 && (
          <p className="mt-8 text-center text-sm text-ok">Excellent. Every password is strong, unique and fresh.</p>
        )}
      </div>
    </div>
  );
}

function Stat({ n, label }: { n: number; label: string }) {
  return (
    <div>
      <div className={cn("text-2xl font-semibold", n > 0 ? "text-fg" : "text-faint")}>{n}</div>
      <div className="text-xs text-muted">{label}</div>
    </div>
  );
}

function IssueList({ title, items, onOpen }: { title: string; items: Health["weak"]; onOpen: (id: string) => void }) {
  if (items.length === 0) return null;
  return (
    <div className="mt-8">
      <h3 className="mb-2 text-sm font-semibold">{title}</h3>
      <div className="divide-y divide-line overflow-hidden rounded-2xl border border-line bg-surface">
        {items.map((it, i) => (
          <button key={`${it.id}-${i}`} onClick={() => onOpen(it.id)} className="flex w-full items-center gap-3 px-4 py-3 text-left hover:bg-surface-2">
            <Avatar title={it.title} color="" size={30} />
            <div className="min-w-0 flex-1">
              <div className="truncate text-sm font-medium">{it.title}</div>
              <div className="truncate text-xs text-muted">{it.detail}</div>
            </div>
          </button>
        ))}
      </div>
    </div>
  );
}
