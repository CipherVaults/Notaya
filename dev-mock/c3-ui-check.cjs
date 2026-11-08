// npm run mock -- --configLoader native --host 127.0.0.1 --port 1424 --strictPort
// node dev-mock/c3-ui-check.cjs http://127.0.0.1:1424
const assert = require('node:assert/strict');
const fs = require('node:fs');
const { chromium } = require(process.env.NOTAYA_PLAYWRIGHT_MODULE || 'playwright');
const url = new URL(process.argv[2] || 'http://127.0.0.1:1424');
assert.ok(['127.0.0.1', 'localhost'].includes(url.hostname));
const output = 'D:/Notaya-lanes/out/C3';
fs.mkdirSync(output, { recursive: true });
process.env.TEMP = 'D:/Notaya Tests/v1-6-C3u'; process.env.TMP = process.env.TEMP;
fs.mkdirSync(process.env.TEMP, { recursive: true });
let browser;
(async () => {
  browser = await chromium.launch({ executablePath: process.env.NOTAYA_CHROMIUM_PATH, headless: true });
  const page = await browser.newPage({ viewport: { width: 1366, height: 900 } });
  const main = page.getByRole('main');
  const sidebar = page.getByRole('complementary');
  const errors = [];
  page.on('pageerror', (error) => errors.push(error.message));
  // All UI traffic is confined to the local mock.
  await page.route('**/*', (route) => new URL(route.request().url()).origin === url.origin ? route.continue() : route.abort());
  await page.goto(url.origin);
  await sidebar.getByRole('button', { name: 'Home', exact: true }).waitFor();
  await page.evaluate(async () => (await import('/src/store/app.ts')).useApp.getState().openPage('p1'));
  const title = main.getByPlaceholder('Untitled', { exact: true });
  await title.waitFor();
  await title.fill('Latest unsaved title');
  // The editor header is in main; sidebar rows have identically named buttons.
  await main.getByRole('button', { name: 'Page options', exact: true }).click();
  // Radix renders the menu and dialogs in portals outside main.
  await page.getByRole('menu').getByRole('menuitem', { name: 'Version history', exact: true }).click();
  const history = page.getByRole('dialog', { name: 'Version history', exact: true });
  await history.getByRole('heading', { name: 'Latest unsaved title', exact: true }).waitFor();
  console.log('ok opening history flushes the pending title and selects the latest version');
  await history.getByLabel('Versions', { exact: true }).getByRole('button', { name: /\bwith Test AI\b.*before AI edit$/ }).click();
  await history.getByLabel('Version preview', { exact: true }).getByText('Earlier saved text', { exact: true }).waitFor();
  assert.equal(await history.locator('[contenteditable=true]').count(), 0);
  await page.screenshot({ path: `${output}/history.png`, fullPage: true });
  await history.getByRole('button', { name: 'Restore this version', exact: true }).click();
  await history.waitFor({ state: 'hidden' });
  await page.waitForFunction(() => {
    const titles = document.querySelectorAll('main input[placeholder="Untitled"]');
    return titles.length === 1 && titles[0].value === 'Earlier saved title';
  });
  console.log('ok read-only history preview restores the chosen version through items-changed');
  await sidebar.getByRole('button', { name: 'Settings', exact: true }).click();
  const activity = main.locator('section').filter({ has: page.getByRole('heading', { name: 'Activity', exact: true }) });
  await activity.getByRole('heading', { name: 'Activity', exact: true }).waitFor();
  const entries = activity.getByLabel('Activity entries', { exact: true });
  const who = activity.getByLabel('Filter activity by who', { exact: true });
  // Identify the seeded change by content, rather than the order of Undo buttons.
  const aiEntry = entries.locator(':scope > div').filter({ has: page.getByText('Weekly plan', { exact: true }) });
  await activity.getByRole('button', { name: 'Check the log', exact: true }).click();
  await activity.getByText('The log checks out.', { exact: true }).waitFor();
  await who.selectOption('person');
  await entries.getByText('Personal', { exact: true }).waitFor();
  await aiEntry.waitFor({ state: 'hidden' });
  assert.equal(await entries.getByRole('button', { name: 'Undo this change', exact: true }).count(), 0);
  await who.selectOption('ai:Test AI');
  await aiEntry.getByRole('button', { name: 'Undo this change', exact: true }).click();
  const undo = page.getByRole('dialog', { name: 'Undo this change', exact: true });
  await undo.getByRole('button', { name: 'Undo change set', exact: true }).click();
  await undo.waitFor({ state: 'hidden' });
  await page.getByText('1 restored, 0 skipped.', { exact: true }).waitFor();
  await aiEntry.getByRole('button', { name: 'Undo this change', exact: true }).waitFor({ state: 'hidden' });
  console.log('ok Activity filters actors, checks the log and confirms AI change-set undo');
  await activity.getByRole('button', { name: 'Clear activity log', exact: true }).click();
  const clearDialog = page.getByRole('dialog', { name: 'Clear activity log', exact: true });
  const clear = clearDialog.getByRole('button', { name: 'Clear log', exact: true });
  await clear.waitFor();
  assert.equal(await clear.isDisabled(), true);
  await clearDialog.getByLabel('Type CLEAR', { exact: true }).fill('clear');
  assert.equal(await clear.isDisabled(), true);
  await clearDialog.getByLabel('Type CLEAR', { exact: true }).fill('CLEAR');
  await clear.click();
  await clearDialog.waitFor({ state: 'hidden' });
  await entries.getByText('Log cleared', { exact: true }).waitFor();
  await who.selectOption('');
  await activity.getByRole('button', { name: 'Check the log', exact: true }).click();
  await activity.getByText('The log checks out.', { exact: true }).waitFor();
  await page.screenshot({ path: `${output}/activity.png`, fullPage: true });
  console.log('ok typed CLEAR leaves a verifiable anchor; the plain error log remains separate');
  assert.deepEqual(errors, []);
  console.log('4 C3 UI checks passed (mock only)');
})().catch((error) => { console.error(error); process.exitCode = 1; }).finally(async () => { await browser?.close(); });
