// Regression: opening a legacy page assigns bids without autosaving it.
// Start: npm run mock -- --configLoader native
// Run: node dev-mock/block-ids-check.cjs [http://127.0.0.1:1421/]
// Optional existing tools: NOTAYA_PLAYWRIGHT_MODULE and NOTAYA_CHROMIUM_PATH.
// No install, real app, vault or debug port. The caller owns the mock server.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { waitForMockState, installEquationFixture } = require('./block-ids-waits.cjs');

const target = new URL(process.argv[2] || 'http://127.0.0.1:1421/');
assert(target.protocol === 'http:' && ['127.0.0.1', 'localhost', '[::1]'].includes(target.hostname), 'Use the loopback mock server');
const browserTemp = path.join('D:/Notaya Tests/v1-6-C4d', 'block-ids-browser-temp');
fs.mkdirSync(browserTemp, { recursive: true });
process.env.TEMP = browserTemp;
process.env.TMP = browserTemp;
const { chromium } = require(process.env.NOTAYA_PLAYWRIGHT_MODULE || 'playwright');

(async () => {
  const browser = await chromium.launch({
    ...(process.env.NOTAYA_CHROMIUM_PATH ? { executablePath: process.env.NOTAYA_CHROMIUM_PATH } : {}),
  });
  try {
    const context = await browser.newContext();
    // Keep this check on the mock server; no external requests.
    await context.route('**/*', (route) => new URL(route.request().url()).origin === target.origin ? route.continue() : route.abort());
    const page = await context.newPage();
    const errors = [];
    page.on('pageerror', (error) => errors.push(error.message));
    await page.addInitScript(() => { window.__blockIdsSaveCalls = []; });
    // Observe actual mock saves without changing the app or its fake backend.
    await page.route('**/dev-mock/core.ts*', async (route) => {
      const response = await route.fetch();
      const source = await response.text();
      const marker = 'switch (cmd) {';
      assert(source.includes(marker), 'The mock invoke seam must be instrumented');
      await route.fulfill({ response, body: source.replace(marker,
        'if (cmd === "save_page") window.__blockIdsSaveCalls.push(structuredClone(args));\n' + marker) });
    });
    await page.goto(target.href);
    // A first import of pm/history can make Vite optimize a new dependency and
    // reload the document. Do it BEFORE observing saves or changing the page.
    // Reacquire the module in the current document if that reload interrupts us.
    const preload = () => waitForMockState(page, 'the mock and equation fixture modules to load', async () => {
      const mock = await import('/dev-mock/core.ts');
      const fixture = await import('/dev-mock/equation-fixture.ts');
      window.__blockIdsModules = { ...mock, ...fixture };
      return document.readyState === 'complete' && !!document.querySelector('#root')?.childElementCount;
    });
    await preload();
    // Start measurements in a fresh document after dependency warmup. The mock
    // data and the save-call recorder are both reset by this explicit reload.
    try {
      await page.reload();
    } catch (error) {
      // Vite may already be performing the reload we requested. The following
      // readiness probe must still find the modules in the current document.
      if (!/interrupted by another navigation|net::ERR_ABORTED/i.test(error.message)) throw error;
    }
    await preload();
    await page.getByText('Reading list', { exact: true }).first().click();
    const editor = page.locator('.ProseMirror').first();
    await waitForMockState(page, 'the editable Reading list page', () => {
      const element = document.querySelector('.ProseMirror');
      return document.readyState === 'complete' && element?.isConnected && element.editor?.isEditable
        && document.querySelector('input[placeholder="Untitled"]')?.value === 'Reading list';
    });
    const legacy = await waitForMockState(page, 'the legacy Reading list fixture', async () => {
      const { invoke } = window.__blockIdsModules;
      return JSON.parse((await invoke('get_page', { id: 'p3' })).content);
    });
    const containsBid = (node) => !!node.attrs?.bid || (node.content || []).some(containsBid);
    assert(!containsBid(legacy), 'The fixture must be a legacy page without bids');
    const ids = await waitForMockState(page, 'all top-level block ids on the Reading list page', () => {
      const doc = document.querySelector('.ProseMirror')?.editor?.getJSON();
      return doc?.content?.length && doc.content.every((node) => /^[0-9a-f]{16}$/i.test(node.attrs?.bid))
        && doc.content.map((node) => node.attrs.bid);
    });
    // PageEditor's save debounce is 500 ms; opening must stay clean past it.
    await page.waitForTimeout(1200);
    assert.equal(await page.evaluate(() => window.__blockIdsSaveCalls.length), 0, 'Opening a legacy page must make NO save_page call');
    console.log('PASS legacy_page_open_assigns_ids_without_save');

    await editor.evaluate((element) => element.editor.commands.focus('end'));
    await page.keyboard.type('block id edit');
    await waitForMockState(page, 'the real edit to reach save_page for p3', () =>
      window.__blockIdsSaveCalls?.some((save) => save.id === 'p3' && save.text.endsWith('block id edit')));
    await page.waitForTimeout(700);
    const saves = await page.evaluate(() => window.__blockIdsSaveCalls);
    assert.equal(saves.length, 1, 'A real edit must save once');
    assert.equal(saves[0].id, 'p3');
    assert(saves[0].text.endsWith('block id edit'), 'The real edit must reach the save');
    const saved = JSON.parse(saves[0].content);
    assert.deepEqual(saved.content.map((node) => node.attrs.bid), ids, 'The assigned ids must ride the next real edit unchanged');
    const allIds = [];
    const collect = (node) => {
      if (node.attrs?.bid) allIds.push(node.attrs.bid);
      (node.content || []).forEach(collect);
    };
    collect(saved);
    assert.equal(new Set(allIds).size, allIds.length, 'Saved ids must be unique');
    assert.deepEqual(errors, [], 'No browser errors');
    console.log('PASS real_edit_saves_content_and_stable_ids');

    // Exercise the equation's real node-view editor and history transactions.
    const equationId = 'eeeeeeeeeeeeeeee';
    // setContent followed by a quick edit shares one undo event and restores
    // the old Reading list page. Keep fixture setup out of edit history.
    await waitForMockState(page, 'the saved Reading list editor and isolated equation fixture',
      installEquationFixture, { bid: equationId, previousContent: saves[0].content });
    const equation = () => waitForMockState(page, 'the single equation in the current editor', () => {
      const nodes = document.querySelector('.ProseMirror')?.editor?.getJSON().content.filter((node) => node.type === 'mathBlock');
      return nodes?.length === 1 && nodes[0];
    });
    assert.deepEqual((await equation()).attrs, { bid: equationId, latex: 'x^2' }, 'The fixture equation must be installed before editing');
    await page.locator('.math-block').click();
    const area = page.locator('.math-editor textarea');
    await area.waitFor();
    assert.equal(await area.inputValue(), 'x^2', 'The node-view editor must open on the fixture equation');
    const exact = '\u200f\u0639\u05d0 e\u0301\u200e  ';
    await area.fill(exact);
    assert.deepEqual((await equation()).attrs, { bid: equationId, latex: exact });
    await area.press('Escape');
    await area.waitFor({ state: 'detached' });
    assert.equal(await editor.evaluate((element) => element.editor.commands.undo()), true, 'The equation edit must be undoable');
    assert.deepEqual((await equation()).attrs, { bid: equationId, latex: 'x^2' });
    assert.equal(await editor.evaluate((element) => element.editor.commands.redo()), true, 'The equation edit must be redoable');
    assert.deepEqual((await equation()).attrs, { bid: equationId, latex: exact });
    assert.deepEqual(errors, [], 'No browser errors during equation edits or history');
    console.log('PASS equation_edit_undo_redo_keeps_id_and_exact_text');
    console.log('ALL OK (3 checks)');
  } finally {
    await browser.close();
  }
})().catch((error) => { console.error(error); process.exitCode = 1; });
