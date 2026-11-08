import { emit } from "./event";
// Dev-only stand-in for @tauri-apps/api/core, used by `npm run mock` to preview
// the UI in a plain browser with fake data. Never part of the shipped app.

import { databaseCommand, databaseMetas, reorderDatabases, NOT_HANDLED, coordinationItems, removeCoordinatedItems } from "./database";

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
  density: params.get("density") === "compact" ? "compact" : "comfortable",
  unlockBackground: params.get("bg") ?? "midnight",
  idleLockMinutes: 5,
  lockOnMinimize: false,
  screenCaptureProtection: false,
  stickyDefaultColor: "yellow",
  // Dev-only hydration fixture, e.g. ?lastKind=page&lastId=p1. No browser storage.
  lastPlace: params.has("lastId") ? { kind: params.get("lastKind") ?? "page", id: params.get("lastId"), ...(params.has("lastView") ? { viewId: params.get("lastView") } : {}) } : null,
};

const savedPages = new Map<string, { title?: string; content?: string; text?: string; icon?: string; cover?: string }>();
// Pictures added with add_attachment (cover and icon uploads): id -> blob URL, so the mock can show what was picked.
const pictures = new Map<string, string>();
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
    { id: "p1", sectionId: "s1", title: "Morning pages", preview: "Slept badly, but the plan for the week is clear: finish the vault, then the editor polish.", words: 212, order: 0, favorite: true, parent: "", icon: "☀️", cover: "gradient:dusk", trashedAt: null, created: ago(300), updated: ago(5) },
    { id: "p2", sectionId: "s1", title: "Trip notes", preview: "Flights, hotel confirmation numbers and a rough itinerary for the first three days.", words: 96, order: 1, favorite: false, icon: "✈️", cover: "solid:#cdd9f3", trashedAt: null, created: ago(500), updated: ago(120) },
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

async function invokeCore<T = unknown>(cmd: string, args: Args = {}, options?: { headers?: Record<string, string> }): Promise<T> {
  await new Promise((r) => setTimeout(r, 30));
  const ok = (v: unknown) => v as T;
  if (!["list_notes", "search", "list_bin", "restore_bin_item", "restore_page", "delete_page", "empty_trash"].includes(cmd)) {
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
      return ok(structuredClone(settings));
    case "save_settings": {
      const lastPlace = settings.lastPlace;
      Object.assign(settings, args.settings as Args);
      settings.lastPlace = lastPlace;
      return ok(structuredClone(settings));
    }
    case "set_last_place":
      if (!unlocked) throw "The vault is locked.";
      settings.lastPlace = structuredClone(args.lastPlace ?? null);
      return ok(null);
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
      return ok(liveTree());
    // The real app makes a verified backup first and fails the delete if that fails; ?failDelete=1 mimics that.
    case "delete_notebook":
    case "delete_section": {
      if (params.get("failDelete") === "1") throw "Could not make a backup copy first (the drive is full), so nothing was deleted.";
      binTimes.set(String(args.id), Date.now());
      return ok(liveTree());
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
      reorderDatabases(args.ids as string[]);
      (args.ids as string[]).forEach((id, i) => {
        const item = [...tree.notebooks, ...tree.sections, ...tree.pages].find((x) => x.id === id);
        if (item) item.order = i;
      });
      for (const k of ["notebooks", "sections", "pages"] as const) (tree[k] as { order: number }[]).sort((a, b) => a.order - b.order);
      return ok({ ...tree, databases: databaseMetas() });
    }
    case "get_page": {
      const saved = savedPages.get(String(args.id));
      const meta = tree.pages.find((x) => x.id === args.id);
      if (!meta) throw "Page not found.";
      // A page the mock has no cover for gets the dusk gradient (the editor checks lean on it); ?cover=none turns it off.
      const cover = params.get("cover") === "none" ? "" : meta?.cover ?? "gradient:dusk";
      return ok({ ...meta, content: JSON.stringify(params.get("rtl") ? rtlBody : pageBody), text: "This week", icon: meta.icon ?? "", cover, ...(saved ?? {}) });
    }
    case "create_subpage":
      return ok({ ...tree.pages[0], id: "p" + Math.floor(Math.random() * 1e6), parent: args.parentId, title: args.title, content: "", text: "", cover: "" });
    case "set_page_look": {
      // The same rules as the real command: no outside addresses, picture icons only for pages.
      const cover = String(args.cover ?? "");
      const icon = String(args.icon ?? "");
      const coverOk = cover === "" || /^gradient:[a-z0-9-]{1,24}$/i.test(cover) || /^solid:#[0-9a-f]{6}$/i.test(cover) || /^http:\/\/clatt\.localhost\/[0-9a-f]{32}(#y=(100|[0-9]{1,2}))?$/i.test(cover);
      if (!coverOk) throw "This cover is not allowed. Use a gradient, a colour or a picture from your notes.";
      if (icon.startsWith("pic:") && !/^pic:[0-9a-f]{32}$/i.test(icon)) throw "This icon is not allowed.";
      if (icon.length > 40) throw "Icon is too long (max 40 characters).";
      const pg = tree.pages.find((x) => x.id === args.id) as { icon: string; cover?: string } | undefined;
      if (pg) {
        pg.icon = icon;
        pg.cover = cover;
      }
      savedPages.set(String(args.id), { ...(savedPages.get(String(args.id)) ?? {}), icon, cover });
      return ok({ ...(pg ?? tree.pages[0]), icon, cover });
    }
    case "add_attachment": {
      const bytes = args as unknown as Uint8Array;
      const png = bytes[0] === 0x89 && bytes[1] === 0x50;
      const jpg = bytes[0] === 0xff && bytes[1] === 0xd8;
      if (!png && !jpg) throw "Only PNG, JPEG, GIF and WebP images are supported.";
      const id = Array.from({ length: 32 }, () => Math.floor(Math.random() * 16).toString(16)).join("");
      pictures.set(id, URL.createObjectURL(new Blob([bytes as BlobPart], { type: png ? "image/png" : "image/jpeg" })));
      return ok(id);
    }
    case "save_page": {
      savedPages.set(String(args.id), { ...(savedPages.get(String(args.id)) ?? {}), title: String(args.title ?? ""), content: String(args.content ?? ""), text: String(args.text ?? "") });
      const pg = tree.pages.find((p) => p.id === args.id);
      if (!pg) throw "That item no longer exists.";
      pg.title = String(args.title); pg.preview = String(args.text).slice(0, 160); pg.updated = Date.now();
      return ok({ ...pg });
    }
    case "editing": return ok(null);
    case "list_bin": return ok(mockBin());
    case "restore_bin_item":
    case "restore_page":
      restoreMockItem(String(args.id), cmd === "restore_bin_item" && !!args.restoreContainers);
      return ok(liveTree());
    case "trash_page": {
      const pg = tree.pages.find((p) => p.id === args.id); if (!pg) throw "That item no longer exists.";
      pg.trashedAt = Date.now(); pg.favorite = false; return ok(liveTree());
    }
    case "delete_page": {
      const id = String(args.id);
      if (!mockItems().some((item) => item.id === id)) throw "That item no longer exists.";
      if (!mockInBin(id)) throw "Only items in the bin can be permanently deleted.";
      removeMockTree(String(args.id)); return ok(liveTree());
    }
    case "empty_trash": {
      for (const item of mockBin()) removeMockTree(item.id);
      return ok(liveTree());
    }
    case "list_stickies": return ok(stickies.filter((s) => !mockInBin(s.id)));
    case "get_sticky": {
      const sticky = stickies.find((s) => s.id === args.id); if (!sticky) throw "That item no longer exists.";
      return ok({ ...sticky, trashedAt: binTimes.get(sticky.id) ?? null });
    }
    case "save_sticky": {
      const sticky = stickies.find((s) => s.id === args.id); if (!sticky) throw "That item no longer exists.";
      sticky.content = String(args.content); sticky.text = String(args.text); sticky.updated = Date.now(); return ok(null);
    }
    case "delete_sticky":
    case "delete_stickies": {
      const gone = new Set((args.ids as string[] | undefined) ?? [String(args.id)]);
      for (const sticky of stickies) if (gone.has(sticky.id)) { binTimes.set(sticky.id, Date.now()); sticky.open = false; }
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
    case "search": {
      const q = String(args.query ?? "").toLowerCase();
      if (!q.trim()) return ok([]);
      const hits = [
        ...tree.pages.filter((p) => (p.title + p.preview).toLowerCase().includes(q)).map((p) => ({ kind: "page", id: p.id, title: p.title, subtitle: "", snippet: p.preview, score: 3 })),
        ...stickies.filter((s) => s.text.toLowerCase().includes(q)).map((s) => ({ kind: "sticky", id: s.id, title: s.text, subtitle: "", snippet: s.text, score: 1 })),
        ...(databaseCommand("search", args, tree) as { id: string }[]),
      ];
      return ok(hits.filter((hit) => !mockInBin(hit.id)));
    }
    default:
      return ok(null);
  }
}

export function convertFileSrc(id: string): string {
  const picked = pictures.get(id);
  if (picked) return picked;
  return `data:image/svg+xml,${encodeURIComponent(`<svg xmlns="http://www.w3.org/2000/svg" width="40" height="30"><rect width="40" height="30" fill="#888"/><text x="4" y="18" font-size="8">${id}</text></svg>`)}`;
}


const binTimes = new Map<string, number>();
const revs = new Map<string, number>();
const mockItems = (): Record<string, unknown>[] => [
  ...tree.notebooks.map((x) => ({ ...x, kind: "notebook" })),
  ...tree.sections.map((x) => ({ ...x, kind: "section" })),
  ...tree.pages.map((x) => ({ ...x, kind: "page" })),
  ...stickies.map((x) => ({ ...x, kind: "sticky" })), ...coordinationItems(),
];
function parentId(id: string): string {
  const item = mockItems().find((x) => x.id === id);
  return String(item?.notebookId ?? item?.db ?? (item?.parent || item?.sectionId) ?? "");
}
function mockWithin(id: string, root: string): boolean {
  const seen = new Set<string>();
  while (id && !seen.has(id)) { if (id === root) return true; seen.add(id); id = parentId(id); }
  return false;
}
function mockInBin(id: string): boolean {
  const seen = new Set<string>();
  while (id && !seen.has(id)) {
    seen.add(id);
    if (binTimes.has(id) || mockItems().find((x) => x.id === id)?.trashedAt != null) return true;
    id = parentId(id);
  }
  return false;
}
function liveTree() {
  if (params.get("nb") === "none") return { notebooks: [], sections: [], pages: [], databases: [] };
  return { notebooks: tree.notebooks.filter((x) => !mockInBin(x.id)), sections: tree.sections.filter((x) => !mockInBin(x.id)), pages: tree.pages.filter((x) => !mockInBin(x.id) || x.trashedAt != null), databases: databaseMetas().filter((x) => !mockInBin(x.id) || x.trashedAt != null) };
}
function mockBin() {
  return mockItems().filter((x) => binTimes.has(String(x.id)) || x.trashedAt != null).map((x) => ({ id: String(x.id), kind: String(x.kind), title: String(x.name ?? x.title ?? x.text ?? ""), icon: String(x.icon ?? ""), trashedAt: binTimes.get(String(x.id)) ?? x.trashedAt, parentInBin: mockInBin(parentId(String(x.id))) }));
}
function restoreMockItem(id: string, restoreContainers: boolean) {
  if (!mockItems().some((item) => item.id === id)) throw "That item no longer exists.";
  const containers: string[] = [];
  const seen = new Set<string>();
  for (let parent = parentId(id); parent && !seen.has(parent); parent = parentId(parent)) {
    seen.add(parent);
    const item = mockItems().find((item) => item.id === parent);
    if ((item?.kind === "notebook" || item?.kind === "section") && binTimes.has(parent)) {
      if (!restoreContainers) throw "parent_in_bin: Restore the containing items too?";
      containers.push(parent);
    }
  }
  for (const container of containers) binTimes.delete(container);
  binTimes.delete(id);
  const page = tree.pages.find((page) => page.id === id);
  if (page) {
    const parent = tree.pages.find((parent) => parent.id === page.parent);
    const children = tree.pages.filter((child) => mockWithin(child.id, id));
    if (parent?.trashedAt != null) page.parent = "";
    for (const child of children) child.trashedAt = null;
  }
  databaseCommand("restore_page", { id }, tree);
}
function removeMockTree(root: string) {
  const ids = new Set(mockItems().filter((x) => mockWithin(String(x.id), root)).map((x) => String(x.id)));
  tree.notebooks = tree.notebooks.filter((x) => !ids.has(x.id)); tree.sections = tree.sections.filter((x) => !ids.has(x.id)); tree.pages = tree.pages.filter((x) => !ids.has(x.id));
  for (let i = stickies.length - 1; i >= 0; i--) if (ids.has(stickies[i].id)) stickies.splice(i, 1);
  removeCoordinatedItems(ids);
  for (const id of ids) { binTimes.delete(id); savedPages.delete(id); }
}

/** Revisions and an event after each mock write, matching the production signal contract. */
export async function invoke<T = unknown>(cmd: string, args: Args = {}, options?: { headers?: Record<string, string> }): Promise<T> {
  const value = await invokeCore<unknown>(cmd, args, options);
  const write = /^(save_page|save_database|save_sticky|set_cells|create_row|delete_|trash_page|restore_|empty_trash|set_page_look|set_page_favorite|set_sticky_|close_sticky|open_sticky|hide_stickies|reorder)/.test(cmd);
  const ids = (args.ids as string[] | undefined) ?? [String(args.id ?? args.rowId ?? (cmd === "save_database" ? JSON.parse(String(args.databaseJson)).id : ""))];
  if (write) {
    for (const id of ids.filter(Boolean)) revs.set(id, (revs.get(id) ?? 0) + 1);
    await emit("items-changed", { ids: ids.filter(Boolean), kinds: ids.map((id) => String(mockItems().find((x) => x.id === id)?.kind ?? "page")), tree: true, by: "person", revs: Object.fromEntries(revs) });
  }
  if (cmd === "save_sticky") return (revs.get(String(args.id)) ?? 0) as T;
  if (value && typeof value === "object" && !Array.isArray(value) && ["get_page", "save_page", "get_database", "save_database", "get_sticky"].includes(cmd)) {
    return structuredClone({ ...value, rev: revs.get(String(args.id ?? (value as { id: string }).id)) ?? 0 }) as T;
  }
  return structuredClone(value) as T;
}
