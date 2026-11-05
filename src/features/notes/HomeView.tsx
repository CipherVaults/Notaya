import { useMemo, useState } from "react";
import { BookPlus, FilePlus2, FileText, NotebookPen, Star, StickyNote, Table2, X } from "lucide-react";
import { api } from "@/lib/ipc";
import { timeAgo } from "@/lib/utils";
import { useApp } from "@/store/app";
import { Button, EmptyState } from "@/components/ui";
import { getKey, setKey } from "@/lib/storage";
import { CoverStrip, PageIcon } from "@/lib/pageLook";
import type { NotesTree } from "@/lib/types";

const OPEN_NOTE_KEY = "openVaultNoteDismissed";
/** The most cards a list on Home shows, so Home stays quick with thousands of pages. */
const RECENT_CAP = 8;
const FAVORITES_CAP = 8;
const NOTEBOOKS_CAP = 24;
/** One banner per day, in turn. */
const BANNERS = ["dusk", "aurora", "peach", "ocean", "grape", "citrus", "lagoon", "forest", "candy", "sunrise", "midnight", "rose"];

/** One-time reminder, shown only while security is off. Remembers the dismissal in this browser profile only. */
function OpenVaultNote() {
  const [dismissed, setDismissed] = useState(() => {
    try {
      return getKey(OPEN_NOTE_KEY) === "1";
    } catch {
      return false;
    }
  });
  if (dismissed) return null;
  const dismiss = () => {
    setDismissed(true);
    try {
      setKey(OPEN_NOTE_KEY, "1");
    } catch {
      /* the note simply shows again next time */
    }
  };
  return (
    <div className="flex items-start gap-3 rounded-xl border border-line bg-surface px-4 py-3 text-start text-sm text-muted" role="note">
      <p className="flex-1">Your notes open without a password. Turn on security in Settings, Security, to protect them.</p>
      <button onClick={dismiss} className="rounded-md p-1 text-faint transition-colors hover:bg-surface-2 hover:text-fg" aria-label="Dismiss" title="Dismiss">
        <X size={14} />
      </button>
    </div>
  );
}

interface Card {
  id: string;
  title: string;
  icon: string;
  cover: string;
  preview: string;
  where: string;
  updated: number;
  database: boolean;
}

/** Everything Home shows, worked out in one pass over the tree. */
function homeData(tree: NotesTree) {
  const sections = new Map(tree.sections.map((s) => [s.id, s]));
  const notebooks = new Map(tree.notebooks.map((n) => [n.id, n]));
  const where = (sectionId: string) => {
    const s = sections.get(sectionId);
    return s ? `${notebooks.get(s.notebookId)?.name ?? ""} / ${s.name}`.replace(/^ \/ /, "") : "";
  };
  const live = tree.pages.filter((p) => p.trashedAt === null);
  const page = (p: (typeof live)[number]): Card => ({ id: p.id, title: p.title, icon: p.icon, cover: p.cover ?? "", preview: p.preview, where: where(p.sectionId), updated: p.updated, database: false });
  const db = (d: NotesTree["databases"][number]): Card => ({ id: d.id, title: d.title, icon: d.icon, cover: "", preview: "Database", where: where(d.sectionId), updated: d.updated, database: true });

  const recent = [...live].sort((a, b) => b.updated - a.updated).slice(0, RECENT_CAP).map(page);
  const favorites = [...live.filter((p) => p.favorite).map(page), ...tree.databases.filter((d) => d.trashedAt === null && d.favorite).map(db)]
    .sort((a, b) => b.updated - a.updated)
    .slice(0, FAVORITES_CAP);

  const pagesIn = new Map<string, number>();
  const newest = new Map<string, { id: string; updated: number }>();
  for (const p of live) {
    const s = sections.get(p.sectionId);
    if (!s) continue;
    pagesIn.set(s.notebookId, (pagesIn.get(s.notebookId) ?? 0) + 1);
    const cur = newest.get(s.notebookId);
    if (!cur || p.updated > cur.updated) newest.set(s.notebookId, { id: p.id, updated: p.updated });
  }
  const sectionCount = new Map<string, number>();
  for (const s of tree.sections) sectionCount.set(s.notebookId, (sectionCount.get(s.notebookId) ?? 0) + 1);
  const books = tree.notebooks.slice(0, NOTEBOOKS_CAP).map((n) => ({ nb: n, pages: pagesIn.get(n.id) ?? 0, sections: sectionCount.get(n.id) ?? 0, latest: newest.get(n.id)?.id ?? null }));
  return { recent, favorites, books, hiddenBooks: Math.max(0, tree.notebooks.length - NOTEBOOKS_CAP) };
}

function PageCard({ card, onOpen }: { card: Card; onOpen: () => void }) {
  return (
    <button
      type="button"
      onClick={onOpen}
      data-testid="home-card"
      data-has-cover={card.cover ? "1" : "0"}
      className="group flex min-w-0 flex-col overflow-hidden rounded-xl border border-line bg-surface text-start transition-colors hover:bg-surface-2"
    >
      {card.cover && <CoverStrip cover={card.cover} className="h-12 w-full shrink-0" />}
      <div className="flex min-w-0 flex-1 flex-col p-3.5">
        <div className="mb-2 flex h-6 items-center text-xl leading-none">
          <PageIcon icon={card.icon} size={22} fallback={card.database ? <Table2 size={20} className="text-faint" /> : <FileText size={20} className="text-faint" />} />
        </div>
        <div dir="auto" className="truncate font-medium">{card.title || "Untitled"}</div>
        <div dir="auto" className="mt-0.5 line-clamp-2 min-h-[2.4em] text-xs leading-relaxed text-muted">{card.preview || "Empty page"}</div>
        <div className="mt-2 truncate text-[0.7rem] text-faint">
          {card.where}
          {card.where && " · "}
          {timeAgo(card.updated)}
        </div>
      </div>
    </button>
  );
}

function SectionTitle({ icon, children }: { icon?: React.ReactNode; children: React.ReactNode }) {
  return (
    <h2 className="mb-3 mt-10 flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wider text-faint">
      {icon}
      {children}
    </h2>
  );
}

const GRID = "grid grid-cols-[repeat(auto-fill,minmax(210px,1fr))] gap-3";

export default function Home() {
  const tree = useApp((s) => s.tree);
  const setIntent = useApp((s) => s.setIntent);
  const openPage = useApp((s) => s.openPage);
  const statusKnown = useApp((s) => s.status !== null);
  const securityOn = useApp((s) => s.status?.securityOn ?? false);
  const showNote = statusKnown && !securityOn;
  const data = useMemo(() => homeData(tree), [tree]);

  if (tree.notebooks.length === 0) {
    return (
      <EmptyState
        icon={<NotebookPen size={28} />}
        title="Your first notebook"
        body={
          securityOn
            ? "Notebooks hold sections, and sections hold pages. Everything is encrypted the moment you type it."
            : "Notebooks hold sections, and sections hold pages. Your notes are saved on this drive, in the app's vault file. They are not private until you turn on security in Settings."
        }
        action={
          <div className="flex max-w-md flex-col items-center gap-4">
            <Button variant="primary" onClick={() => setIntent("new-notebook")}>
              <BookPlus size={16} /> Create notebook
            </Button>
            {showNote && <OpenVaultNote />}
          </div>
        }
      />
    );
  }

  const now = new Date();
  const hour = now.getHours();
  const greeting = hour < 5 ? "Working late" : hour < 12 ? "Good morning" : hour < 18 ? "Good afternoon" : "Good evening";
  const glyph = hour < 5 ? "🌙" : hour < 12 ? "🌅" : hour < 18 ? "☀️" : "🌆";
  const dayOfYear = Math.floor((Date.UTC(now.getFullYear(), now.getMonth(), now.getDate()) - Date.UTC(now.getFullYear(), 0, 0)) / 86400000);
  const banner = BANNERS[dayOfYear % BANNERS.length];
  const open = (id: string, database: boolean) => {
    const s = useApp.getState();
    const sectionId = database ? s.tree.databases.find((d) => d.id === id)?.sectionId : s.tree.pages.find((p) => p.id === id)?.sectionId;
    const section = s.tree.sections.find((x) => x.id === sectionId);
    if (section) useApp.setState({ activeNotebookId: section.notebookId, activeSectionId: section.id });
    openPage(id);
  };

  return (
    <div className="h-full overflow-y-auto" data-testid="home">
      <div className={`cover-gradient-${banner} h-[168px] w-full dark:brightness-[.8]`} data-testid="home-banner" aria-hidden />
      <div className="mx-auto px-10 pb-16" style={{ maxWidth: "calc(var(--editor-width) + 80px)" }}>
        <div className="relative z-10 -mt-9 mb-1 text-[3.2rem] leading-none" aria-hidden>{glyph}</div>
        <h1 className="text-[2rem] font-bold tracking-tight">{greeting}</h1>
        <p className="mt-0.5 text-sm text-faint">{now.toLocaleDateString(undefined, { weekday: "long", day: "numeric", month: "long" })}</p>
        <p className="mt-2 text-muted">
          {securityOn
            ? "Everything here is encrypted. Pick a page from the sidebar, or start a new one."
            : "Your notes are saved on this drive. Pick a page from the sidebar, or start a new one."}
        </p>
        {showNote && (
          <div className="mt-5">
            <OpenVaultNote />
          </div>
        )}

        <div className="mt-6 flex gap-2">
          <Button variant="primary" onClick={() => setIntent("new-page")}>
            <FilePlus2 size={16} /> New page
          </Button>
          <Button variant="outline" onClick={() => api.createSticky(true).then(async () => useApp.getState().setStickies(await api.listStickies()))}>
            <StickyNote size={15} /> New sticky note
          </Button>
        </div>

        {data.favorites.length > 0 && (
          <>
            <SectionTitle icon={<Star size={12} />}>Favorites</SectionTitle>
            <div className={GRID} data-testid="home-favorites">
              {data.favorites.map((c) => (
                <PageCard key={c.id} card={c} onOpen={() => open(c.id, c.database)} />
              ))}
            </div>
          </>
        )}

        {data.recent.length > 0 && (
          <>
            <SectionTitle>Recent pages</SectionTitle>
            <div className={GRID} data-testid="home-recent">
              {data.recent.map((c) => (
                <PageCard key={c.id} card={c} onOpen={() => open(c.id, false)} />
              ))}
            </div>
          </>
        )}

        <SectionTitle>Notebooks</SectionTitle>
        <div className="divide-y divide-line overflow-hidden rounded-xl border border-line" data-testid="home-notebooks">
          {data.books.map(({ nb, pages, sections, latest }) => (
            <button
              key={nb.id}
              type="button"
              onClick={() => {
                const s = useApp.getState();
                const first = s.tree.sections.find((x) => x.notebookId === nb.id);
                useApp.setState({ activeNotebookId: nb.id, activeSectionId: first?.id ?? null });
                if (latest) open(latest, false);
              }}
              className="flex w-full items-center gap-3 px-4 py-2.5 text-start transition-colors hover:bg-surface-2"
            >
              <span className="grid h-7 w-7 shrink-0 place-items-center rounded-lg text-lg" style={{ background: `color-mix(in oklab, ${nb.color || "var(--faint)"} 18%, transparent)` }}>
                {nb.icon}
              </span>
              <span dir="auto" className="min-w-0 flex-1 truncate text-sm font-medium">{nb.name}</span>
              <span className="shrink-0 text-xs text-faint">
                {sections} {sections === 1 ? "section" : "sections"} · {pages} {pages === 1 ? "page" : "pages"}
              </span>
            </button>
          ))}
        </div>
        {data.hiddenBooks > 0 && <p className="mt-2 text-xs text-faint">And {data.hiddenBooks} more in the sidebar.</p>}
      </div>
    </div>
  );
}
