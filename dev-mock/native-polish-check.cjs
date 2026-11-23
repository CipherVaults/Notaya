// Run against the mock dev server or a production build made with vite.mock.config.ts.
// Usage: node dev-mock/native-polish-check.cjs <dev|desktop-dev|production> <local URL>
const fs = require("node:fs");
const path = require("node:path");
const { chromium } = require(process.env.NOTAYA_PLAYWRIGHT_MODULE || "playwright");
const { createChecks } = require("../tools/checks/fail-loud.cjs");

const [mode, url] = process.argv.slice(2);
const base = new URL(url);
if (!["dev", "desktop-dev", "production"].includes(mode) || !["127.0.0.1", "localhost"].includes(base.hostname)) {
  throw new Error("Choose a test mode and a local mock URL");
}
const checks = createChecks(`native polish: ${mode}`);
const native = mode !== "dev";
const palette = (page) => page.getByRole("dialog", { name: "Command palette" });
const noteFind = (scope) => scope.getByRole("searchbox", { name: "Find in note" });
const read = (name) => fs.readFileSync(path.join(__dirname, "..", name), "utf8");

function versionChecks() {
  const lock = JSON.parse(read("package-lock.json"));
  for (const [name, version] of Object.entries({
    "package.json": JSON.parse(read("package.json")).version,
    "package-lock.json": lock.version,
    "package-lock.json root package": lock.packages[""].version,
    "src-tauri/tauri.conf.json": JSON.parse(read("src-tauri/tauri.conf.json")).version,
    "src-tauri/Cargo.toml": read("src-tauri/Cargo.toml").match(/^version = "([^"]+)"/m)?.[1],
    "bridge/Cargo.toml": read("bridge/Cargo.toml").match(/^version = "([^"]+)"/m)?.[1],
  })) {
    checks.equal(`${name} version`, version, "1.6.0");
  }
  for (const name of ["notaya-app", "notaya-bridge"]) {
    const entry = read("src-tauri/Cargo.lock").split("[[package]]").find((part) => part.includes(`name = "${name}"`));
    checks.equal(`Cargo.lock ${name} version`, entry?.match(/^version = "([^"]+)"/m)?.[1], "1.6.0");
  }
}

function dispatchKey(page, key, modifiers = {}) {
  return page.evaluate(({ key, modifiers }) => !document.activeElement.dispatchEvent(
    new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true, ...modifiers }),
  ), { key, modifiers });
}

async function open(page, query) {
  await page.goto(new URL(query, base).href, { waitUntil: "domcontentloaded" });
}

async function openNote(page) {
  const note = page.getByRole("tree", { name: "Favorites" }).getByRole("treeitem", { name: "Morning pages" });
  await note.click();
  return note;
}

async function editorShortcutChecks(page) {
  for (const [name, query, selector] of [
    ["Page", "?db=1&s=secure", ".ProseMirror"],
    ["Sticky", "?label=sticky-k1", ".sticky-editor .ProseMirror"],
  ]) {
    await open(page, query);
    if (name === "Page") await openNote(page);
    const editor = page.locator(selector);
    const sample = "Keyboard formatting sample";
    await editor.focus();
    const selected = await editor.evaluate((el, text) => {
      el.editor.chain()
        .setContent({ type: "doc", content: [{ type: "paragraph", content: [{ type: "text", text }] }] }, { emitUpdate: false })
        .setTextSelection({ from: 1, to: text.length + 1 })
        .run();
      const { doc, selection } = el.editor.state;
      return doc.textBetween(selection.from, selection.to);
    }, sample);
    checks.equal(`${name} shortcut sample is selected`, selected, sample);
    for (const [shortcut, tag] of [["Control+u", "u"], ["Control+Shift+S", "s"], ["Control+Shift+H", "mark"]]) {
      const mark = editor.locator(tag).filter({ hasText: sample });
      await page.keyboard.press(shortcut);
      checks.equal(`${name} ${shortcut} adds <${tag}>`, await mark.count(), 1);
      await page.keyboard.press(shortcut);
      checks.equal(`${name} ${shortcut} removes <${tag}>`, await mark.count(), 0);
    }
    if (name === "Page") {
      const block = editor.getByText(sample, { exact: true });
      for (const [key, alignment] of [["J", "justify"], ["E", "center"], ["R", "right"]]) {
        await page.keyboard.press(`Control+Shift+${key}`);
        checks.equal(`Page Ctrl+Shift+${key} aligns text ${alignment}`,
          await block.evaluate((el) => el.style.textAlign), alignment);
      }
      // The protected mock emits vault-locked after locking, and App reloads.
      // Allow for the two-second save flush before judging that no reload occurred.
      const reload = (timeout) => page.waitForEvent("framenavigated", {
        predicate: (frame) => frame === page.mainFrame() && frame.url() === new URL(query, base).href,
        timeout,
      });
      const shiftedLock = reload(2500).then(() => true, (error) => {
        if (error.name !== "TimeoutError") throw error;
        return false;
      });
      await page.keyboard.press("Control+Shift+L");
      checks.equal("Page Ctrl+Shift+L aligns text left", await block.evaluate((el) => el.style.textAlign), "left");
      const locked = await shiftedLock;
      checks.equal("Protected page alignment leaves the vault unlocked", locked, false);
      if (locked) {
        await openNote(page);
        await editor.focus();
      }
      const plainLock = reload(4000);
      await page.keyboard.press("Control+l");
      const lockedFrame = await plainLock;
      checks.check("Plain Ctrl+L locks the protected vault", !!lockedFrame);
    } else {
      await stickyImageChecks(page, editor);
    }
  }
}

async function stickyImageChecks(page, editor) {
  const src = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACklEQVR4nGMAAQAABQABDQottAAAAABJRU5ErkJggg==";
  await editor.evaluate((el, src) => {
    el.editor.commands.setContent({
      type: "doc",
      content: [{ type: "image", attrs: { src, alt: "Sticky drag fixture" } }],
    }, { emitUpdate: false });
  }, src);
  const image = editor.getByAltText("Sticky drag fixture");
  checks.check("Sticky image remains draggable",
    await image.evaluate(async (el) => {
      await el.decode();
      return el.draggable && getComputedStyle(el).webkitUserDrag !== "none";
    }));
  const chromeDrag = await page.evaluate((src) => {
    const image = new Image();
    image.src = src;
    document.body.append(image);
    try {
      const regular = getComputedStyle(image).webkitUserDrag;
      image.draggable = true;
      return { regular, explicit: getComputedStyle(image).webkitUserDrag };
    } finally {
      image.remove();
    }
  }, src);
  checks.equal("Chrome image suppresses browser dragging", chromeDrag.regular, "none");
  checks.check("Explicitly draggable chrome image remains draggable", chromeDrag.explicit !== "none");
}

async function findChecks(page) {
  await open(page, "?db=1");
  await page.getByRole("main").getByRole("heading", { level: 1 }).waitFor();
  const ctrl = { ctrlKey: true };
  checks.equal("Home cancels browser Find", await dispatchKey(page, "f", ctrl), true);
  await palette(page).waitFor();
  checks.check("Home opens Search", await palette(page).isVisible());
  await page.keyboard.press("Escape");
  await palette(page).waitFor({ state: "hidden" });

  const note = await openNote(page);
  const editor = page.locator(".ProseMirror");
  await editor.waitFor();
  const find = noteFind(page);
  await editor.focus();
  const text = await editor.innerText();
  checks.equal("Note cancels browser Find", await dispatchKey(page, "f", ctrl), true);
  await find.waitFor();
  checks.equal("Note focuses its Find instead of Search", await page.evaluate(() => document.activeElement.getAttribute("aria-label")), "Find in note");
  await find.fill("WEEK");
  await page.getByText("1 of 2", { exact: true }).waitFor();
  checks.equal("Note highlights a case-insensitive match", await editor.locator("[data-note-find-match=active]").first().textContent(), "week");
  await find.press("Enter");
  await page.getByText("2 of 2", { exact: true }).waitFor();
  checks.check("Next match advances", await page.getByText("2 of 2", { exact: true }).isVisible());
  await find.press("Shift+Enter");
  await page.getByText("1 of 2", { exact: true }).waitFor();
  checks.check("Previous match goes back", await page.getByText("1 of 2", { exact: true }).isVisible());
  checks.equal("Note Find leaves composing Escape available", await dispatchKey(page, "Escape", { isComposing: true }), false);
  checks.check("Note Find stays open while composing", await find.isVisible());
  await find.press("Escape");
  await find.waitFor({ state: "hidden" });
  checks.equal("Note text survives Find", await editor.innerText(), text);
  checks.check("Note focus returns to the editor", await editor.evaluate((el) => el === document.activeElement));
  const styles = await page.evaluate(() => ({
    pageOverscroll: getComputedStyle(document.documentElement).overscrollBehavior,
    chromeSelection: getComputedStyle(document.body).userSelect,
    noteSelection: getComputedStyle(document.querySelector(".ProseMirror p")).userSelect,
  }));
  checks.equal("Page overscroll containment", styles.pageOverscroll, "none");
  checks.equal("No text selection in chrome", styles.chromeSelection, "none");
  checks.equal("Note text stays selectable", styles.noteSelection, "text");
  checks.check("Full-page note leaves dragging to the image block",
    await editor.locator(".image-block").first().evaluate((el) => el.draggable && !el.querySelector("img").draggable));
  await editor.blur();
  checks.equal("Note owns Find when the page background has focus", await dispatchKey(page, "f", ctrl), true);
  await find.waitFor();
  await page.getByRole("button", { name: "Search", exact: true }).click();
  const foreground = palette(page);
  await foreground.getByPlaceholder("Search notes and sticky notes, or type a command").waitFor();
  checks.check("Search takes focus over Note Find", await foreground.evaluate((el) => el.contains(document.activeElement)));
  await page.keyboard.press("Escape");
  await foreground.waitFor({ state: "hidden" });
  checks.check("Escape closes Search and leaves the background Note Find open", await find.isVisible());
  await editor.press("Escape");
  await find.waitFor({ state: "hidden" });
  checks.check("Escape in the full-page editor closes its Find", await editor.isVisible());
  await note.focus();
  checks.equal("Sidebar uses the Shell Find fallback", await dispatchKey(page, "f", ctrl), true);
  await palette(page).waitFor();
  checks.check("Sidebar opens Search", await palette(page).isVisible());
  await page.keyboard.press("Escape");
  await palette(page).waitFor({ state: "hidden" });

  await page.getByRole("treeitem", { name: "Tasks" }).click();
  const table = page.getByRole("application", { name: "Table" });
  await table.waitFor();
  await table.focus();
  checks.equal("Database cancels browser Find", await dispatchKey(page, "f", ctrl), true);
  const rows = page.getByRole("textbox", { name: "Search rows" });
  await rows.waitFor();
  checks.equal("Database focuses row Find instead of Search", await page.evaluate(() => document.activeElement.getAttribute("aria-label")), "Search rows");
  await rows.fill("Design the landing page");
  await table.getByText("Design the landing page").waitFor();
  await rows.press("Escape");
  await rows.waitFor({ state: "hidden" });
  checks.equal("Database owns Find when the page background has focus", await dispatchKey(page, "f", ctrl), true);
  await rows.waitFor();
}

async function rowPeekFindChecks(page) {
  await open(page, "?db=1");
  await page.getByRole("treeitem", { name: "Tasks" }).click();
  const table = page.getByRole("application", { name: "Table" });
  const title = "Design the landing page";
  const row = table.getByRole("row").filter({ hasText: title });
  for (const [peekMode, role] of [["Center peek", "dialog"], ["Side peek", "complementary"]]) {
    await page.getByRole("button", { name: "View settings" }).click();
    const openIn = page.getByLabel("Open rows in");
    await openIn.selectOption({ label: peekMode });
    await page.keyboard.press("Escape");
    await openIn.waitFor({ state: "hidden" });

    const peek = page.getByRole(role, { name: "Row" });
    const editor = peek.locator(".ProseMirror");
    const find = noteFind(peek);
    const openRow = async () => {
      await row.getByText(title, { exact: true }).hover();
      await row.getByRole("button", { name: "Open row" }).click();
      await editor.waitFor();
    };

    await openRow();
    const estimate = peek.getByRole("listitem").filter({ has: page.getByText("Estimate", { exact: true }) });
    const value = estimate.getByText("1", { exact: true });
    const input = estimate.getByRole("textbox", { name: "Estimate" });
    await value.click();
    await input.fill("999");
    checks.equal(`${peekMode} inline editor holds the draft`, await input.inputValue(), "999");
    await input.press("Escape");
    await input.waitFor({ state: "hidden" });
    const staysOpen = await peek.isVisible();
    checks.check(`${peekMode} inline Escape cancels the edit and leaves the row open`, staysOpen);
    if (!staysOpen) await openRow();
    await value.waitFor();
    checks.check(`${peekMode} inline Escape preserves the saved estimate`, await value.isVisible());
    await editor.locator("p").first().click();
    await page.keyboard.press("Escape");
    await peek.waitFor({ state: "hidden" });
    checks.check(`${peekMode} Escape from a body paragraph closes the row`, !(await peek.isVisible()));

    const titleInput = peek.getByPlaceholder("Untitled", { exact: true });
    for (const [focus, target] of [["Find input", find], ["row editor", editor], ["row title", titleInput]]) {
      await openRow();
      await editor.press("Control+f");
      await find.waitFor();
      if (target === titleInput) {
        await titleInput.click();
      } else {
        await target.focus();
      }
      await page.keyboard.press("Escape");
      await find.waitFor({ state: "hidden" });
      const staysOpen = await peek.isVisible();
      checks.check(`${peekMode} first Escape from ${focus} closes only Find`, staysOpen);
      if (target === titleInput) {
        checks.check(`${peekMode} closing Find keeps focus on the title`,
          staysOpen && await titleInput.evaluate((el) => el === document.activeElement));
      }
      if (staysOpen) {
        await page.keyboard.press("Escape");
        await peek.waitFor({ state: "hidden" });
        checks.check(`${peekMode} second Escape after ${focus} closes the row`, !(await peek.isVisible()));
      }
    }

    if (role === "complementary") {
      await openRow();
      await editor.press("Control+f");
      await find.waitFor();
      await value.click();
      await input.fill("999");
      await page.keyboard.press("Escape");
      await input.waitFor({ state: "hidden" });
      checks.check("Side peek inline Escape with Find open keeps the row open", await peek.isVisible());
      checks.check("Side peek inline Escape with Find open preserves the saved estimate", await value.isVisible());
      checks.check("Side peek inline Escape leaves Find open", await find.isVisible());
    }
  }
}

async function browserChecks(page) {
  await open(page, "?s=lock");
  const password = page.getByLabel("Master password", { exact: true });
  await password.waitFor();
  await password.focus();
  for (const key of ["f", "g", "h", "j", "o", "p", "r", "s", "u", "+", "=", "-", "_", "0"]) {
    checks.equal(`Ctrl+${key} browser default`, await dispatchKey(page, key, { ctrlKey: true }), native);
  }
  for (const key of ["g", "h", "i", "j", "c", "r", "s"]) {
    checks.equal(`Ctrl+Shift+${key} browser default`, await dispatchKey(page, key, { ctrlKey: true, shiftKey: true }), native);
  }
  for (const key of ["F3", "F5", "F7", "F12"]) {
    checks.equal(`${key} browser default`, await dispatchKey(page, key), key === "F12" ? mode === "production" : native);
  }
  checks.equal("Shift+F3 browser default", await dispatchKey(page, "F3", { shiftKey: true }), native);
  for (const key of ["a", "c", "v", "x", "z"]) {
    checks.equal(`Ctrl+${key} still reaches the focused field`, await dispatchKey(page, key, { ctrlKey: true }), false);
  }
  checks.check("Locked window has no Command Palette", !(await palette(page).count()));

  const behaviors = await page.evaluate(() => {
    const sandbox = document.createElement("div");
    sandbox.innerHTML = '<button>Chrome</button><input><textarea></textarea><div contenteditable>Editable <span contenteditable="false">Embedded UI</span></div>';
    document.body.append(sandbox);
    const [chrome, input, textarea, editor] = sandbox.children;
    const fire = (target, event) => !target.dispatchEvent(event);
    const wheel = (ctrlKey) => fire(chrome, new WheelEvent("wheel", { bubbles: true, cancelable: true, ctrlKey, deltaY: 10 }));
    const mouse = (target, type, button) => fire(target, new MouseEvent(type, { bubbles: true, cancelable: true, button }));
    try {
      return {
        "Ctrl+wheel": wheel(true),
        "Middle mousedown": mouse(chrome, "mousedown", 1),
        "Middle auxclick": mouse(chrome, "auxclick", 1),
        "Chrome context menu": mouse(chrome, "contextmenu", 2),
        "Chrome Text node context menu": mouse(chrome.firstChild, "contextmenu", 2),
        "Non-editable island context menu": mouse(editor.lastChild.firstChild, "contextmenu", 2),
        "Normal wheel": wheel(false),
        "Left mousedown": mouse(chrome, "mousedown", 0),
        "Input context menu": mouse(input, "contextmenu", 2),
        "Textarea context menu": mouse(textarea, "contextmenu", 2),
        "Editable Text node context menu": mouse(editor.firstChild, "contextmenu", 2),
      };
    } finally {
      sandbox.remove();
    }
  });
  const blockedInDesktop = new Set([
    "Ctrl+wheel", "Middle mousedown", "Middle auxclick", "Chrome context menu",
    "Chrome Text node context menu", "Non-editable island context menu",
  ]);
  for (const [name, blocked] of Object.entries(behaviors)) {
    checks.equal(name, blocked, native && blockedInDesktop.has(name));
  }

  for (const [label, focus] of [["sticky-k1", ".sticky-editor .ProseMirror"], ["calc-1", 'input[aria-label="Expression"]']]) {
    await open(page, `?label=${label}`);
    await page.locator(focus).focus();
    checks.equal(`${label} browser Find`, await dispatchKey(page, "f", { ctrlKey: true }), native);
    checks.check(`${label} has no Command Palette`, !(await palette(page).count()));
  }
}

(async () => {
  versionChecks();
  const executablePath = process.env.NOTAYA_CHROMIUM_PATH;
  const browser = await chromium.launch(executablePath ? { executablePath } : {});
  try {
    const page = await browser.newPage({ viewport: { width: 1400, height: 900 } });
    page.setDefaultTimeout(12_000);
    if (mode === "desktop-dev") {
      await page.addInitScript(() => { window.__TAURI_INTERNALS__ = {}; });
    }
    page.on("pageerror", (error) => checks.check("Uncaught page error", false, error.message));
    await editorShortcutChecks(page);
    await rowPeekFindChecks(page);
    await findChecks(page);
    await browserChecks(page);
  } finally {
    await browser.close();
  }
})().catch((error) => checks.check("Native polish runner", false, error.message)).finally(() => checks.finish());
