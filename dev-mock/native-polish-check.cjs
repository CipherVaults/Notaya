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
  await page.goto(new URL(query, base).href);
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

  const note = page.getByRole("tree", { name: "Favorites" }).getByRole("treeitem", { name: "Morning pages" });
  await note.click();
  const editor = page.locator(".ProseMirror");
  await editor.waitFor();
  const find = page.getByRole("searchbox", { name: "Find in note" });
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
  await find.press("Escape");
  await find.waitFor({ state: "hidden" });
  checks.equal("Note text survives Find", await editor.innerText(), text);
  checks.check("Note focus returns to the editor", await editor.evaluate((el) => el === document.activeElement));
  const styles = await page.evaluate(() => ({
    pageOverscroll: getComputedStyle(document.documentElement).overscrollBehavior,
    chromeSelection: getComputedStyle(document.body).userSelect,
    noteSelection: getComputedStyle(document.querySelector(".ProseMirror p")).userSelect,
    imageDrag: getComputedStyle(document.querySelector(".ProseMirror img")).webkitUserDrag,
  }));
  checks.equal("Page overscroll containment", styles.pageOverscroll, "none");
  checks.equal("No text selection in chrome", styles.chromeSelection, "none");
  checks.equal("Note text stays selectable", styles.noteSelection, "text");
  checks.equal("Images do not generate browser drag previews", styles.imageDrag, "none");
  await editor.blur();
  checks.equal("Note owns Find when the page background has focus", await dispatchKey(page, "f", ctrl), true);
  await find.waitFor();
  await find.press("Escape");
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

async function browserChecks(page) {
  await open(page, "?s=lock");
  const password = page.getByLabel("Master password", { exact: true });
  await password.waitFor();
  await password.focus();
  for (const key of ["f", "g", "h", "j", "o", "p", "r", "s", "u", "+", "=", "-", "_", "0"]) {
    checks.equal(`Ctrl+${key} browser default`, await dispatchKey(page, key, { ctrlKey: true }), native);
  }
  for (const key of ["g", "i", "j", "c", "r"]) {
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
    await findChecks(page);
    await browserChecks(page);
  } finally {
    await browser.close();
  }
})().catch((error) => checks.check("Native polish runner", false, error.message)).finally(() => checks.finish());
