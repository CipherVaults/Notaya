// Regression: opening a legacy page assigns bids without autosaving it.
// Start: npm run mock -- --configLoader native
// Run: node dev-mock/block-ids-check.cjs [http://127.0.0.1:1421/]
// Optional existing tools: NOTAYA_PLAYWRIGHT_MODULE and NOTAYA_CHROMIUM_PATH.
// No install, real app, vault or debug port. The caller owns the mock server.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const target = new URL(process.argv[2] || 'http://127.0.0.1:1421/');
assert(target.protocol === 'http:' && ['127.0.0.1', 'localhost', '[::1]'].includes(target.hostname), 'Use the loopback mock server');
const browserTemp = path.join('D:/Notaya Tests/v1-6-C4b', 'block-ids-browser-temp');
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
    await page.getByText('Reading list', { exact: true }).first().click();
    const editor = page.locator('.ProseMirror').first();
    await editor.waitFor();
    const legacy = await page.evaluate(async () => {
      const { invoke } = await import('/dev-mock/core.ts');
      return JSON.parse((await invoke('get_page', { id: 'p3' })).content);
    });
    const containsBid = (node) => !!node.attrs?.bid || (node.content || []).some(containsBid);
    assert(!containsBid(legacy), 'The fixture must be a legacy page without bids');
    await page.waitForFunction(() => {
      const doc = document.querySelector('.ProseMirror')?.editor?.getJSON();
      return doc?.content?.length && doc.content.every((node) => /^[0-9a-f]{16}$/i.test(node.attrs?.bid));
    });
    const ids = await editor.evaluate((element) => element.editor.getJSON().content.map((node) => node.attrs.bid));
    // PageEditor's save debounce is 500 ms; opening must stay clean past it.
    await page.waitForTimeout(1200);
    assert.equal(await page.evaluate(() => window.__blockIdsSaveCalls.length), 0, 'Opening a legacy page must make NO save_page call');
    console.log('PASS legacy_page_open_assigns_ids_without_save');

    await editor.evaluate((element) => element.editor.commands.focus('end'));
    await page.keyboard.type('block id edit');
    await page.waitForFunction(() => window.__blockIdsSaveCalls.length > 0);
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
    console.log('ALL OK (2 checks)');
  } finally {
    await browser.close();
  }
})().catch((error) => { console.error(error); process.exitCode = 1; });
