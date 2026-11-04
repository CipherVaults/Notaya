// Dev-only stand-in for @tauri-apps/api/core, used by `npm run mock` to preview
// the UI in a plain browser with fake data. Never part of the shipped app.

import { databaseCommand, databaseMetas, NOT_HANDLED } from "./database";

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
let importRunning = false;
let importCancelled = false;
const searchHistory: string[] = [];
const calcHistory: { expr: string; result: string }[] = [{ expr: "12.5 × (3 + 4)", result: "87.5" }];

const now = Date.now();
const ago = (m: number) => now - m * 60_000;

const shellPrefs: Record<string, unknown> = {
  closeToTray: true,
  startWithWindows: false,
  startMinimized: true,
  installed: params.get("portable") !== "1",
  trayAvailable: true,
};

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

const savedPages = new Map<string, { title: string; content: string; text: string }>();
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

// ?rtl=1 shows a page mixing Arabic, Hebrew and English (for checking right-to-left text).
const t = (text: string, marks?: unknown[]) => ({ type: "text", text, ...(marks ? { marks } : {}) });
const para = (text: string, attrs?: Record<string, unknown>) => ({ type: "paragraph", ...(attrs ? { attrs } : {}), content: [t(text)] });
const item = (text: string, type = "listItem", attrs?: Record<string, unknown>) => ({ type, ...(attrs ? { attrs } : {}), content: [para(text)] });
const rtlBody = {
  type: "doc",
  content: [
    { type: "heading", attrs: { level: 2 }, content: [t("ملاحظات الصباح")] },
    para("هذه فقرة عربية تبدأ من اليمين، وفيها كلمة English في الوسط وأرقام 123."),
    para("This is an English paragraph, it stays on the left."),
    para("זוהי פסקה בעברית, והיא מיושרת לימין."),
    para("هذه فقرة عربية مع محاذاة صريحة لليسار", { textAlign: "left" }),
    para("English text forced right-to-left", { dir: "rtl" }),
    { type: "heading", attrs: { level: 3 }, content: [t("قائمة المهام")] },
    { type: "bulletList", content: [item("البند الأول"), item("البند الثاني"), item("English item")] },
    { type: "orderedList", content: [item("الخطوة الأولى"), item("الخطوة الثانية")] },
    { type: "taskList", content: [item("إنهاء الخزنة", "taskItem", { checked: true }), item("تحسين المحرر", "taskItem", { checked: false })] },
    { type: "blockquote", content: [para("البساطة أولاً، ثم الأفكار الذكية.")] },
    { type: "callout", attrs: { icon: "💡" }, content: [para("نصيحة: اكتب / في سطر فارغ لإضافة كتلة.")] },
    { type: "toggle", attrs: { open: true, shelf: true }, content: [para("رف الملاحظات"), para("نص داخل الرف.")] },
    { type: "toggle", attrs: { open: false }, content: [para("قسم قابل للطي"), para("مخفي")] },
    { type: "table", content: [
      { type: "tableRow", content: [{ type: "tableHeader", content: [para("الاسم")] }, { type: "tableHeader", content: [para("القيمة")] }, { type: "tableHeader", content: [para("ملاحظة")] }] },
      { type: "tableRow", content: [{ type: "tableCell", content: [para("أحمد")] }, { type: "tableCell", content: [para("١٢٣")] }, { type: "tableCell", content: [para("جيد")] }] },
    ] },
    { type: "table", content: [
      { type: "tableRow", content: [{ type: "tableHeader", content: [para("Name")] }, { type: "tableHeader", content: [para("Value")] }, { type: "tableHeader", content: [para("Note")] }] },
      { type: "tableRow", content: [{ type: "tableCell", content: [para("Ahmed")] }, { type: "tableCell", content: [para("42")] }, { type: "tableCell", content: [para("fine")] }] },
    ] },
    { type: "paragraph" },
  ],
};
if (params.get("rtl")) {
  tree.pages[0].title = "ملاحظات الصباح";
  tree.pages[0].preview = "هذه فقرة عربية تبدأ من اليمين";
}

const stickies = [
  { id: "k1", content: "", text: "Call the bank before Friday\n- ask about the card", color: "yellow", x: 100, y: 100, w: 320, h: 320, onTop: true, open: true, created: ago(100), updated: ago(10) },
  { id: "k2", content: "", text: "Ideas: dark/light toggle, keyboard shortcuts cheat sheet", color: "blue", x: 140, y: 140, w: 320, h: 320, onTop: true, open: false, created: ago(200), updated: ago(60) },
  { id: "k3", content: "", text: "Buy: coffee, batteries, USB stick", color: "pink", x: 180, y: 180, w: 320, h: 320, onTop: false, open: false, created: ago(300), updated: ago(240) },
];

type Args = Record<string, unknown>;

export async function invoke<T = unknown>(cmd: string, args: Args = {}, options?: { headers?: Record<string, string> }): Promise<T> {
  await new Promise((r) => setTimeout(r, 30));
  const ok = (v: unknown) => v as T;
  if (cmd !== "list_notes" && cmd !== "search") {
    const dbr = databaseCommand(cmd, args, tree, options);
    if (dbr !== NOT_HANDLED) return ok(dbr);
  }
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
      return ok("set-aside-1792700000000.nota");
    case "lock_vault":
      if (!securityOn) return ok(null);
      unlocked = false;
      return ok(null);
    case "list_backups":
      return ok([
        { name: "vault-2026-10-22-18-40.nota", modifiedMs: ago(60 * 20), size: 482_304 },
        { name: "vault-2026-10-21-09-05.nota", modifiedMs: ago(60 * 44), size: 1_572_864 },
      ]);
    case "pick_backup_file":
      return ok("E:\\Backups\\my-old-vault.nota");
    case "restore_vault":
      if (args.password === "wrong") throw "That password (or keyfile) doesn't open this backup.";
    {
      const undoFile = exists ? "restore-undo-1792700000000.nota" : null;
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
    // The error log: nothing is recorded in the mock; the numbers are made up.
    case "log_frontend_error":
      return ok(null);
    case "log_info":
      return ok({ folder: "E:\\Notaya\\data\\logs", bytes: 18432, lastEntry: "2026-10-29T07:45:00Z" });
    case "open_log_folder":
      return ok(null);
    case "get_shell_prefs":
      return ok({ ...shellPrefs });
    case "set_shell_prefs":
      Object.assign(shellPrefs, args.prefs as Args);
      return ok({ ...shellPrefs });
    case "list_notes":
      return ok({ ...(params.get("nb") === "none" ? { ...tree, notebooks: [], sections: [], pages: [] } : tree), databases: databaseMetas() });
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
    case "get_page": {
      const saved = savedPages.get(String(args.id));
      return ok({ ...tree.pages[0], id: args.id, sectionId: "s1", content: JSON.stringify(params.get("rtl") ? rtlBody : pageBody), text: "This week", trashedAt: null, cover: params.get("cover") === "none" ? "" : "gradient:dusk", ...(saved ?? {}) });
    }
    case "create_subpage":
      return ok({ ...tree.pages[0], id: "p" + Math.floor(Math.random() * 1e6), parent: args.parentId, title: args.title, content: "", text: "", cover: "" });
    case "set_page_look":
      return ok({ ...tree.pages[0], icon: args.icon });
    case "save_page":
      // Remember what was saved, so a page keeps its content while the mock runs (the first page is the fixed sample otherwise).
      savedPages.set(String(args.id), { title: String(args.title ?? ""), content: String(args.content ?? ""), text: String(args.text ?? "") });
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
    case "open_sticky":
      for (const k of stickies) if (k.id === args.id) k.open = true;
      return ok(null);
    case "close_sticky":
      for (const k of stickies) if (k.id === args.id) k.open = false;
      return ok(null);
    case "hide_stickies":
      for (const k of stickies) k.open = false;
      return ok(null);
    // The last five searches (the real app keeps them in the encrypted vault).
    case "search_history":
      return ok([...searchHistory]);
    case "add_search_term": {
      const term = String(args.term).trim().replace(/\s+/g, " ");
      if (term) {
        const at = searchHistory.findIndex((t) => t.toLowerCase() === term.toLowerCase());
        if (at >= 0) searchHistory.splice(at, 1);
        searchHistory.unshift(term);
        searchHistory.length = Math.min(searchHistory.length, 5);
      }
      return ok([...searchHistory]);
    }
    case "remove_search_term": {
      const at = searchHistory.indexOf(String(args.term));
      if (at >= 0) searchHistory.splice(at, 1);
      return ok([...searchHistory]);
    }
    case "clear_search_history":
      searchHistory.length = 0;
      return ok(null);
    case "calc_history":
      return ok([...calcHistory]);
    case "add_calc_entry": {
      const expr = String(args.expr), result = String(args.result);
      if (!(calcHistory[0]?.expr === expr && calcHistory[0]?.result === result)) calcHistory.unshift({ expr, result });
      calcHistory.length = Math.min(calcHistory.length, 500);
      return ok([...calcHistory]);
    }
    case "clear_calc_history":
      calcHistory.length = 0;
      return ok(null);
    // The real app opens a save dialog and writes the file; here it pretends the file was saved.
    case "export_notes": {
      const req = args.request as { scope: string; ids: string[]; format: string; includeSubpages: boolean; output?: string };
      await new Promise((r) => setTimeout(r, 400));
      const zip = req.output === "zip";
      const ext = req.format === "docx" ? "docx" : req.format === "pdf" ? "pdf" : req.format === "html" ? "html" : "md";
      const live = tree.pages.filter((p) => p.trashedAt === null);
      const inSections = (ids: string[]) => live.filter((p) => ids.includes(p.sectionId)).length;
      const pages =
        req.scope === "pages"
          ? req.ids.length
          : req.scope === "section" || req.scope === "sections"
            ? inSections(req.ids)
            : inSections(tree.sections.filter((s) => req.ids.includes(s.notebookId)).map((s) => s.id));
      return ok({
        path: `C:\\Users\\you\\Documents\\Notaya export.${zip ? "zip" : ext}`,
        pages,
        images: 1,
        missingImages: 0,
        imagesLeftOut: 0,
        charsReplaced: req.format === "pdf" ? 2 : 0,
        assetsFolder: !zip && ext === "md" ? "C:\\Users\\you\\Documents\\Notaya export.assets" : null,
        files: zip ? pages + (ext === "md" ? 1 : 0) : 1,
        archive: zip,
      });
    }
    case "import_cancel": {
      const running = importRunning;
      importCancelled = running;
      return ok(running);
    }
    // The real app opens a file dialog and imports the files; here it pretends two files were picked.
    // ?importFail=1 makes it fail, ?importCancel=1 cancels the dialog.
    case "import_notes": {
      if (params.get("importFail") === "1") throw "No pages were found. Choose Markdown (.md), HTML, Word (.docx) or PDF files, a folder of them, or a ZIP file made by Notion.";
      // The import takes a moment (longer with ?importSlow=1) and stops at once when "import_cancel" arrives.
      importCancelled = false;
      importRunning = true;
      const steps = params.get("importSlow") === "1" ? 100 : 15;
      for (let i = 0; i < steps && !importCancelled; i++) await new Promise((r) => setTimeout(r, 100));
      importRunning = false;
      if (importCancelled) {
        importCancelled = false;
        return ok(null);
      }
      if (params.get("importCancel") === "1") return ok(null);
      const n = tree.notebooks.length + 1;
      const nb = { id: `n${n + 10}`, name: "Notion import", color: "#6366f1", icon: "📓", order: n, created: ago(0), updated: ago(0) };
      const secs = [
        { id: `s${n + 20}`, notebookId: nb.id, name: "Imported pages", color: "", order: 0, created: ago(0), updated: ago(0) },
        { id: `s${n + 21}`, notebookId: nb.id, name: "Recipes", color: "", order: 1, created: ago(0), updated: ago(0) },
      ];
      const mk = (id: string, sectionId: string, title: string, order: number, parent = "", icon = "") => ({
        id, sectionId, title, preview: "Imported from Notion.", words: 40, order, favorite: false, parent, icon, trashedAt: null, created: ago(0), updated: ago(0),
      });
      tree.notebooks.push(nb);
      tree.sections.push(...secs);
      tree.pages.push(
        mk(`p${n + 30}`, secs[0].id, "Project plan", 0, "", "🚀"),
        mk(`p${n + 31}`, secs[0].id, "Sub page", 1, `p${n + 30}`),
        mk(`p${n + 32}`, secs[1].id, "Pancakes", 0, "", "🥞"),
      );
      // ?importTwo=1: a Notaya ZIP of two notebooks, which comes back as two notebooks.
      const notebooks = [{ id: nb.id, name: nb.name }];
      if (params.get("importTwo") === "1") {
        const nb2 = { ...nb, id: `n${n + 11}`, name: "Second notebook", order: n + 1 };
        const sec2 = { ...secs[0], id: `s${n + 22}`, notebookId: nb2.id, name: "Work" };
        tree.notebooks.push(nb2);
        tree.sections.push(sec2);
        tree.pages.push(mk(`p${n + 33}`, sec2.id, "Tasks", 0));
        notebooks.push({ id: nb2.id, name: nb2.name });
      }
      return ok({
        notebookId: nb.id,
        notebookName: nb.name,
        notebooks,
        sections: notebooks.length > 1 ? 3 : 2,
        pages: notebooks.length > 1 ? 4 : 3,
        images: 2,
        skipped: [
          { item: "Tasks 0123abcd.csv", reason: "A Notion database (table) view. Tables are not imported; the pages inside a database are, when they were exported." },
          { item: "Picture holiday.heic (Project plan 0123abcd.html)", reason: "Not a PNG, JPEG, GIF or WebP picture." },
        ],
        notes: ["PDF import reads text only: pictures, scanned pages and the page layout are not imported."],
      });
    }
    case "password_strength": {
      const len = String(args.password ?? "").length;
      const score = len > 24 ? 4 : len > 14 ? 3 : len > 8 ? 2 : 1;
      return ok({ score, guessesLog10: len, crackTime: score > 2 ? "centuries" : "2 minutes", warning: null, suggestions: [] });
    }
    case "search":
      return ok([
        { kind: "page", id: "p1", title: "Morning pages", subtitle: "", snippet: "Slept badly, but the plan for the week is clear", score: 3 },
        { kind: "sticky", id: "k1", title: "Call the bank before Friday", subtitle: "", snippet: "ask about the card", score: 1 },
        ...(databaseCommand("search", args, tree) as unknown[]),
      ]);
    default:
      return ok(null);
  }
}

export function convertFileSrc(id: string): string {
  return `data:image/svg+xml,${encodeURIComponent(`<svg xmlns="http://www.w3.org/2000/svg" width="40" height="30"><rect width="40" height="30" fill="#888"/><text x="4" y="18" font-size="8">${id}</text></svg>`)}`;
}
