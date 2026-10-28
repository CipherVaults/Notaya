// Dev-only stand-in for @tauri-apps/api/core, used by `npm run mock` to preview
// the UI in a plain browser with fake data. Never part of the shipped app.

const params = new URLSearchParams(location.search);
// app = notes with security off | secure = security on, unlocked | lock = security on, locked
// | start = no vault but earlier copies exist (create is an alias of start)
const startState = params.get("s") ?? "app";
// `?s=start&installed` shows the first screen of an installed copy (no notes, nothing to restore).
const installedMock = params.has("installed");
let securityOn = startState === "secure" || startState === "lock";
let unlocked = startState === "app" || startState === "secure";
let exists = startState !== "create" && startState !== "start";
let noticeTaken = false;
const calcHistory: { expr: string; result: string }[] = [{ expr: "12.5 × (3 + 4)", result: "87.5" }];

const now = Date.now();
const ago = (m: number) => now - m * 60_000;

const settings: Record<string, unknown> = {
  theme: params.get("theme") ?? "sunrise",
  accent: params.get("accent") ?? "mono",
  backdrop: "none",
  font: "inter",
  fontSize: Number(params.get("fs") ?? 15),
  editorWidth: "normal",
  density: "comfortable",
  unlockBackground: params.get("bg") ?? "midnight",
  idleLockMinutes: 5,
  lockOnMinimize: false,
  screenCaptureProtection: false,
  stickyDefaultColor: "yellow",
};

const tree = {
  notebooks: [
    { id: "n1", name: "Personal", color: "#a3a3a3", icon: "📓", order: 0, created: ago(900), updated: ago(5) },
    { id: "n2", name: "Work", color: "#737373", icon: "💼", order: 1, created: ago(800), updated: ago(50) },
  ],
  sections: [
    { id: "s1", notebookId: "n1", name: "Journal", color: "", order: 0, created: ago(900), updated: ago(5) },
    { id: "s2", notebookId: "n1", name: "Ideas", color: "", order: 1, created: ago(900), updated: ago(50) },
    { id: "s3", notebookId: "n2", name: "Projects", color: "", order: 0, created: ago(800), updated: ago(50) },
  ],
  pages: [
    { id: "p1", sectionId: "s1", title: "Morning pages", preview: "Slept badly, but the plan for the week is clear: finish the vault, then the editor polish.", words: 212, order: 0, favorite: true, parent: "", icon: "☀️", trashedAt: null, created: ago(300), updated: ago(5) },
    { id: "p2", sectionId: "s1", title: "Trip notes", preview: "Flights, hotel confirmation numbers and a rough itinerary for the first three days.", words: 96, order: 1, favorite: false, icon: "✈️", trashedAt: null, created: ago(500), updated: ago(120) },
    { id: "p4", sectionId: "s1", parent: "p1", title: "Weekly goals", preview: "Three things for this week.", words: 12, order: 0, favorite: false, icon: "🎯", trashedAt: null, created: ago(200), updated: ago(40) },
    { id: "p5", sectionId: "s2", title: "Old draft", preview: "", words: 3, order: 0, favorite: false, parent: "", icon: "", trashedAt: ago(30), created: ago(700), updated: ago(30) },
    { id: "p6", sectionId: "s2", title: "App ideas", preview: "A list.", words: 8, order: 1, favorite: false, parent: "", icon: "", trashedAt: null, created: ago(600), updated: ago(60) },
    { id: "p3", sectionId: "s1", title: "Reading list", preview: "Books to get through before the end of the year.", words: 41, order: 2, favorite: false, icon: "", trashedAt: null, created: ago(900), updated: ago(1500) },
  ],
};

const svg = (w: number, h: number) =>
  "data:image/svg+xml," + encodeURIComponent(`<svg xmlns='http://www.w3.org/2000/svg' width='${w}' height='${h}'><defs><linearGradient id='g' x1='0' y1='0' x2='1' y2='1'><stop offset='0' stop-color='#6a3a8c'/><stop offset='1' stop-color='#f8a577'/></linearGradient></defs><rect width='100%' height='100%' fill='url(#g)'/></svg>`);

const pageBody = {
  type: "doc",
  content: [
    { type: "heading", attrs: { level: 2 }, content: [{ type: "text", text: "This week" }] },
    { type: "paragraph", content: [{ type: "text", text: "Slept badly, but the plan for the week is clear. Everything is encrypted the moment it is typed, and nothing leaves this drive." }] },
    { type: "callout", attrs: { icon: "💡" }, content: [{ type: "paragraph", content: [{ type: "text", text: "Tip: type / on an empty line to add a block, or drag the grip on the left to move one." }] }] },
    { type: "image", attrs: { src: svg(900, 340), width: 70, align: "center", caption: "A picture, encrypted in the vault" } },
    {
      type: "taskList",
      content: [
        { type: "taskItem", attrs: { checked: true }, content: [{ type: "paragraph", content: [{ type: "text", text: "Finish the vault" }] }] },
        { type: "taskItem", attrs: { checked: false }, content: [{ type: "paragraph", content: [{ type: "text", text: "Polish the editor" }] }] },
      ],
    },
    { type: "toggle", attrs: { open: true }, content: [{ type: "paragraph", content: [{ type: "text", text: "More details" }] }, { type: "paragraph", content: [{ type: "text", text: "Hidden text lives inside a toggle." }] }] },
    { type: "heading", attrs: { level: 4 }, content: [{ type: "text", text: "A small heading" }] },
    { type: "columns", content: [
      { type: "column", content: [{ type: "paragraph", content: [{ type: "text", text: "Left column: plans" }] }] },
      { type: "column", content: [{ type: "paragraph", content: [{ type: "text", text: "Right column: notes" }] }] },
    ] },
    { type: "toggle", attrs: { open: true }, content: [{ type: "heading", attrs: { level: 2 }, content: [{ type: "text", text: "Toggle heading" }] }, { type: "paragraph", content: [{ type: "text", text: "Inside a toggle heading." }] }] },
    { type: "pageLink", attrs: { pageId: "p4" } },
    { type: "mathBlock", attrs: { latex: String.raw`x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}` } },
    { type: "paragraph", content: [{ type: "text", text: "Energy is " }, { type: "mathInline", attrs: { latex: "E = mc^2" } }, { type: "text", text: " in every frame." }] },
    { type: "blockquote", content: [{ type: "paragraph", content: [{ type: "text", text: "Simple things first, then the clever ones." }] }] },
    { type: "paragraph" },
  ],
};

const stickies = [
  { id: "k1", content: "", text: "Call the bank before Friday\n- ask about the card", color: "yellow", x: 100, y: 100, w: 320, h: 320, onTop: true, open: true, created: ago(100), updated: ago(10) },
  { id: "k2", content: "", text: "Ideas: dark/light toggle, keyboard shortcuts cheat sheet", color: "blue", x: 140, y: 140, w: 320, h: 320, onTop: true, open: false, created: ago(200), updated: ago(60) },
  { id: "k3", content: "", text: "Buy: coffee, batteries, USB stick", color: "pink", x: 180, y: 180, w: 320, h: 320, onTop: false, open: false, created: ago(300), updated: ago(240) },
];

type Args = Record<string, unknown>;

export async function invoke<T = unknown>(cmd: string, args: Args = {}): Promise<T> {
  await new Promise((r) => setTimeout(r, 30));
  const ok = (v: unknown) => v as T;
  switch (cmd) {
    case "vault_status":
      return ok({
        exists,
        unlocked,
        keyfileRequired: false,
        headerOk: true,
        securityOn,
        earlierCopies: !exists && !installedMock,
        openError: null,
        dataDir: "E:\\Notaya\\data",
        version: "1.0.0",
        offerImport: !exists && installedMock,
      });
    case "import_portable_notes":
      exists = true;
      unlocked = true;
      securityOn = false;
      return ok({ backupsCopied: 3, backupsSkipped: 0 });
    case "start_notes":
      exists = true;
      unlocked = true;
      securityOn = false;
      return ok(null);
    case "enable_security":
      if (String(args.password ?? "").length < 10) throw "Use at least 10 characters for your master password.";
      securityOn = true;
      return ok({ removedCopies: args.removeUnprotectedCopies ? 3 : 0, backupFailed: false });
    case "disable_security":
      if (args.currentPassword === "wrong") throw "Wrong password or keyfile.";
      securityOn = false;
      return ok(null);
    case "unlock_vault":
      if (args.password === "wrong") throw "Wrong master password or keyfile for this Notaya vault.";
      unlocked = true;
      return ok(null);
    case "start_over":
      exists = false;
      return ok("set-aside-1792700000000.clf");
    case "lock_vault":
      if (!securityOn) return ok(null);
      unlocked = false;
      return ok(null);
    case "list_backups":
      return ok([
        { name: "vault-2026-10-22-18-40.clf", modifiedMs: ago(60 * 20), size: 482_304 },
        { name: "vault-2026-10-21-09-05.clf", modifiedMs: ago(60 * 44), size: 1_572_864 },
      ]);
    case "pick_backup_file":
      return ok("E:\\Backups\\my-old-vault.clf");
    case "restore_vault":
      if (args.password === "wrong") throw "That password (or keyfile) doesn't open this backup.";
    {
      const undoFile = exists ? "restore-undo-1792700000000.clf" : null;
      exists = true;
      unlocked = true;
      securityOn = !!args.password;
      return ok({ undoFile });
    }
    case "take_notice": {
      // ?notice=<text> shows that message once, as the backend would after a failed backup.
      const text = params.get("notice");
      if (!text || noticeTaken) return ok(null);
      noticeTaken = true;
      return ok(text);
    }
    case "flush_done":
      return ok(null);
    case "get_settings":
      return ok({ ...settings });
    case "save_settings":
      Object.assign(settings, args.settings as Args);
      return ok({ ...settings });
    case "set_backdrop":
      return ok(false);
    case "list_notes":
      return ok(params.get("nb") === "none" ? { ...tree, notebooks: [], sections: [], pages: [] } : tree);
    // The real app makes a verified backup first and fails the delete if that fails; ?failDelete=1 mimics that.
    case "delete_notebook":
    case "delete_section": {
      if (params.get("failDelete") === "1") throw "Could not make a backup copy first (the drive is full), so nothing was deleted.";
      const secs = new Set(cmd === "delete_notebook" ? tree.sections.filter((x) => x.notebookId === args.id).map((x) => x.id) : [String(args.id)]);
      tree.pages = tree.pages.filter((p) => !secs.has(p.sectionId));
      tree.sections = tree.sections.filter((x) => !secs.has(x.id));
      if (cmd === "delete_notebook") tree.notebooks = tree.notebooks.filter((n) => n.id !== args.id);
      return ok({ ...tree });
    }
    case "update_section": {
      const sec = tree.sections.find((x) => x.id === args.id);
      if (sec) sec.name = String(args.name);
      return ok({ ...tree, sections: [...tree.sections] });
    }
    case "set_page_favorite": {
      const pg = tree.pages.find((x) => x.id === args.id);
      if (pg) pg.favorite = !!args.favorite;
      return ok({ ...tree, pages: [...tree.pages] });
    }
    case "reorder": {
      (args.ids as string[]).forEach((id, i) => {
        const item = [...tree.notebooks, ...tree.sections, ...tree.pages].find((x) => x.id === id);
        if (item) item.order = i;
      });
      for (const k of ["notebooks", "sections", "pages"] as const) (tree[k] as { order: number }[]).sort((a, b) => a.order - b.order);
      return ok({ ...tree });
    }
    case "get_page":
      return ok({ ...tree.pages[0], id: args.id, sectionId: "s1", content: JSON.stringify(pageBody), text: "This week", trashedAt: null, cover: params.get("cover") === "none" ? "" : "gradient:dusk" });
    case "create_subpage":
      return ok({ ...tree.pages[0], id: "p" + Math.floor(Math.random() * 1e6), parent: args.parentId, title: args.title, content: "", text: "", cover: "" });
    case "set_page_look":
      return ok({ ...tree.pages[0], icon: args.icon });
    case "save_page":
      return ok({ ...tree.pages[0] });
    case "list_stickies":
      return ok([...stickies]);
    case "get_sticky":
      return ok(stickies[0]);
    case "delete_sticky":
    case "delete_stickies": {
      const gone = new Set((args.ids as string[] | undefined) ?? [String(args.id)]);
      for (let i = stickies.length - 1; i >= 0; i--) if (gone.has(stickies[i].id)) stickies.splice(i, 1);
      return ok(null);
    }
    // The real app opens a calculator window; here, open ?label=calc-1 in a new tab to see one.
    case "open_calculator":
      return ok(null);
    case "calc_history":
      return ok([...calcHistory]);
    case "add_calc_entry": {
      const expr = String(args.expr), result = String(args.result);
      if (!(calcHistory[0]?.expr === expr && calcHistory[0]?.result === result)) calcHistory.unshift({ expr, result });
      calcHistory.length = Math.min(calcHistory.length, 200);
      return ok([...calcHistory]);
    }
    case "clear_calc_history":
      calcHistory.length = 0;
      return ok(null);
    case "password_strength": {
      const len = String(args.password ?? "").length;
      const score = len > 24 ? 4 : len > 14 ? 3 : len > 8 ? 2 : 1;
      return ok({ score, guessesLog10: len, crackTime: score > 2 ? "centuries" : "2 minutes", warning: null, suggestions: [] });
    }
    case "search":
      return ok([
        { kind: "page", id: "p1", title: "Morning pages", subtitle: "", snippet: "Slept badly, but the plan for the week is clear", score: 3 },
        { kind: "sticky", id: "k1", title: "Call the bank before Friday", subtitle: "", snippet: "ask about the card", score: 1 },
      ]);
    default:
      return ok(null);
  }
}

export function convertFileSrc(id: string): string {
  return `data:image/svg+xml,${encodeURIComponent(`<svg xmlns="http://www.w3.org/2000/svg" width="40" height="30"><rect width="40" height="30" fill="#888"/><text x="4" y="18" font-size="8">${id}</text></svg>`)}`;
}
