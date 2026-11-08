// Start: npm run mock -- --configLoader native --host 127.0.0.1 --port 1423 --strictPort
// Then: node dev-mock/c2-ui-check.cjs http://127.0.0.1:1423
// Uses only the already-installed Playwright/Chrome named in the lane environment.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { chromium } = require(process.env.NOTAYA_PLAYWRIGHT_MODULE || 'playwright');
const target = new URL(process.argv[2] || 'http://127.0.0.1:1423');
assert.equal(target.protocol, 'http:');
assert.ok(['127.0.0.1', 'localhost', '[::1]'].includes(target.hostname));
const output = 'D:/Notaya-lanes/out/C2';
const browserTemp = 'D:/Notaya Tests/v1-6-C2/browser';
fs.mkdirSync(output, { recursive: true });
fs.mkdirSync(browserTemp, { recursive: true });
process.env.TEMP = browserTemp;
process.env.TMP = browserTemp;
let browser;
(async () => {
  browser = await chromium.launch({ executablePath: process.env.NOTAYA_CHROMIUM_PATH, headless: true });
  const page = await browser.newPage({ viewport: { width: 1366, height: 900 } });
  const errors = [];
  page.on('pageerror', (error) => errors.push(error.message));
  await page.goto(`${target.origin}/?db=1`);
  await page.getByRole('button', { name: 'Home', exact: true }).waitFor();
  await page.evaluate(async () => (await import('/src/store/app.ts')).useApp.getState().openPage('p1'));
  const title = page.getByPlaceholder('Untitled', { exact: true });
  await title.waitFor();
  await page.evaluate(async () => {
    const { invoke } = await import('/dev-mock/core.ts');
    await invoke('save_page', { id: 'p1', title: 'Remote clean title', content: '{"type":"doc","content":[{"type":"paragraph"}]}', text: '' });
  });
  await page.waitForFunction(() => document.querySelector('input[placeholder="Untitled"]')?.value === 'Remote clean title');
  console.log('ok clean open page reloads on items-changed');
  await title.click();
  await title.fill('Person typing');
  await page.evaluate(async () => {
    const { invoke } = await import('/dev-mock/core.ts');
    await invoke('save_page', { id: 'p1', title: 'Remote while dirty', content: '{"type":"doc","content":[{"type":"paragraph"}]}', text: '' });
  });
  assert.equal(await title.inputValue(), 'Person typing');
  await page.waitForFunction(async () => (await (await import('/dev-mock/core.ts')).invoke('get_page', { id: 'p1' })).title === 'Person typing');
  console.log('ok dirty typing survives remote signal and lands in save order');
  await title.fill('Flush on leaving');
  await page.getByRole('button', { name: 'Home', exact: true }).click();
  await page.waitForFunction(async () => (await (await import('/dev-mock/core.ts')).invoke('get_page', { id: 'p1' })).title === 'Flush on leaving');
  console.log('ok leaving page flushes pending typing');
  await page.evaluate(async () => {
    const { invoke } = await import('/dev-mock/core.ts');
    await invoke('delete_section', { id: 's1' });
    await invoke('delete_notebook', { id: 'n1' });
    await invoke('delete_sticky', { id: 'k1' });
  });
  await page.getByRole('button', { name: /Trash/ }).click();
  await page.getByRole('heading', { name: 'Notebooks', exact: true }).waitFor();
  await page.getByRole('heading', { name: 'Sections', exact: true }).waitFor();
  await page.getByRole('heading', { name: 'Sticky notes', exact: true }).waitFor();
  await page.screenshot({ path: path.join(output, 'bin-groups.png'), fullPage: true });
  const section = page.locator('section').filter({ has: page.getByRole('heading', { name: 'Sections', exact: true }) });
  await section.getByRole('button', { name: 'Restore', exact: true }).first().click();
  await page.getByRole('button', { name: 'Restore together', exact: true }).click();
  await page.getByRole('heading', { name: 'Notebooks', exact: true }).waitFor({ state: 'hidden' });
  console.log('ok grouped bin restores section with its notebook after confirmation');
  assert.deepEqual(errors, []);
  console.log('4 C2 UI checks passed (mock only)');
})().catch((error) => { console.error(error); process.exitCode = 1; }).finally(async () => { await browser?.close(); });
