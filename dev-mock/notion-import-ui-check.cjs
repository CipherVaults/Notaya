// Exercise the import dialog, sidebar, editor and bin with synthetic data.
// Run against a local server built with vite.mock.config.ts.
const { chromium } = require(process.env.NOTAYA_PLAYWRIGHT_MODULE || "playwright");
const { createChecks } = require("../tools/checks/fail-loud.cjs");

const address = new URL(process.argv[2] || "http://127.0.0.1:1421/");
if (!["127.0.0.1", "localhost"].includes(address.hostname) || !["http:", "https:"].includes(address.protocol)) {
  throw new Error("Use a local mock URL");
}
address.searchParams.set("importScale", "1");
address.searchParams.set("holdBinCommand", "delete_page");
const checks = createChecks("large Notion import UI");
const imported = { notebooks: 1, sections: 2, pages: 3032 };

async function openTreeRow(row) {
  await row.waitFor();
  if (await row.getAttribute("aria-expanded") !== "true") await row.press("ArrowRight");
}

async function run(page) {
  await page.goto(address.href, { waitUntil: "domcontentloaded" });
  const notebookTree = page.getByRole("tree", { name: "Notebooks" });
  const row = (name) => notebookTree.getByRole("treeitem", { name, exact: true });
  const notebook = row("Notion import");
  const readback = () => page.evaluate(() => window.__notayaImportScaleReadback());
  await page.getByRole("button", { name: "Import pages…", exact: true }).click();
  await page.getByRole("dialog", { name: "Import pages", exact: true }).getByRole("button", { name: "Choose files…" }).click();
  const done = page.getByRole("dialog", { name: "Import finished" });
  await done.getByText("3032 pages in 2 sections", { exact: false }).waitFor();
  checks.equal("Import creates the notebook, sections and all 3,032 pages", await readback(), imported);
  await done.getByRole("button", { name: "Done", exact: true }).click();

  await openTreeRow(notebook);
  await openTreeRow(row("Imported pages"));
  const home = row("صفحة البداية");
  const child = row("صفحة فرعية");
  await home.click();
  const editor = page.locator(".ProseMirror");
  const homeText = editor.getByText("هذا نص تجريبي في صفحة البداية.", { exact: true });
  await homeText.waitFor();
  checks.equal("Home title and body have right-to-left direction", await page.evaluate(() => [
    getComputedStyle(document.querySelector('input[placeholder="Untitled"]')).direction,
    getComputedStyle(document.querySelector(".ProseMirror p")).direction,
  ]), ["rtl", "rtl"]);
  checks.equal("The subpage starts inside the collapsed home branch", await child.count(), 0);
  await home.press("ArrowRight");
  await child.waitFor();
  checks.equal("The subpage sits one level below its home", Number(await child.getAttribute("aria-level")), Number(await home.getAttribute("aria-level")) + 1);
  await editor.getByRole("link", { name: /صفحة فرعية/ }).click();
  const childText = editor.getByText("هذا نص تجريبي في الصفحة الفرعية.", { exact: true });
  await childText.waitFor();
  checks.equal("The imported page link opens the correct RTL subpage", await childText.evaluate((node) => node.getAttribute("dir")), "rtl");
  checks.equal("The sidebar selects the linked subpage", await child.getAttribute("aria-selected"), "true");

  await openTreeRow(row("Bulk pages"));
  await openTreeRow(row("Batch 30"));
  await row("Sample 3000").waitFor();
  checks.equal("The final large batch has 100 sibling pages", await row("Sample 3000").getAttribute("aria-setsize"), "100");

  await notebook.press("Shift+F10");
  await page.getByRole("menuitem", { name: "Move to bin", exact: true }).click();
  const confirm = page.getByRole("alertdialog", { name: 'Move to bin "Notion import"?' });
  await confirm.getByRole("button", { name: "Move to bin", exact: true }).click();
  await notebook.waitFor({ state: "detached" });
  await confirm.waitFor({ state: "hidden" });
  checks.equal("Moving the notebook to the bin retains the imported subtree", await readback(), imported);

  await page.getByRole("button", { name: /^Trash/ }).click();
  const binnedNotebooks = page.locator("section").filter({ has: page.getByRole("heading", { name: "Notebooks", exact: true }) });
  await binnedNotebooks.getByText("Notion import", { exact: true }).waitFor();
  await binnedNotebooks.getByRole("button", { name: "Delete for good", exact: true }).click();
  const deleting = binnedNotebooks.getByRole("button", { name: "Deleting…", exact: true });
  await deleting.waitFor();
  checks.check("The bin stays responsive and disables repeat actions while deletion is pending",
    await deleting.isDisabled() && await binnedNotebooks.getByRole("button", { name: "Restore", exact: true }).isDisabled());
  await page.evaluate(() => window.dispatchEvent(new Event("notaya-test-release-bin-action")));
  await binnedNotebooks.waitFor({ state: "detached" });
  checks.equal("Delete for good removes every imported record", await readback(), { notebooks: 0, sections: 0, pages: 0 });
  checks.check("Other notebooks survive the purge", await row("Personal").isVisible() && await row("Work").isVisible());
  checks.equal("Delete for good goes through one backend command", await page.evaluate(() => window.__inv.filter(([cmd]) => cmd === "delete_page").length), 1);
}

(async () => {
  const executablePath = process.env.NOTAYA_CHROMIUM_PATH;
  const browser = await chromium.launch(executablePath ? { executablePath } : {});
  try {
    const page = await browser.newPage({ viewport: { width: 1400, height: 900 } });
    page.setDefaultTimeout(12_000);
    page.on("pageerror", (error) => checks.check("Uncaught page error", false, error.message));
    await run(page);
  } finally {
    await browser.close();
  }
})().catch((error) => checks.check("UI journey completed", false, error.message)).finally(() => checks.finish());
