// Run against the local full mock: node dev-mock/p7-window-check.cjs [mock URL]
const assert = require('node:assert/strict');
const { chromium } = require(process.env.NOTAYA_PLAYWRIGHT_MODULE || 'playwright');
const url = new URL(process.argv[2] || 'http://127.0.0.1:1421');
assert(['localhost', '127.0.0.1', '[::1]'].includes(url.hostname));
const count = 200_000;
url.searchParams.set('db', '1');
url.searchParams.set('rows', String(count));

(async () => {
  const browser = await chromium.launch(process.env.NOTAYA_CHROMIUM_PATH ? { executablePath: process.env.NOTAYA_CHROMIUM_PATH } : {});
  try {
    const page = await browser.newPage({ viewport: { width: 1400, height: 900 } });
    page.setDefaultTimeout(60_000);
    const errors = [];
    page.on('pageerror', (error) => errors.push(error.message));
    await page.goto(url.href);
    await page.getByRole('treeitem', { name: 'Tasks', exact: true }).click();
    const table = page.getByRole('application', { name: 'Table', exact: true });
    const row = (index) => table.locator('tr[data-row-id]').filter({
      has: page.getByRole('checkbox', { name: `Select row ${index}`, exact: true }),
    });
    await row(1).waitFor();
    assert.equal(await table.getByRole('grid').getAttribute('aria-rowcount'), String(count + 3));
    assert.match(await row(1).innerText(), /Design the landing page/);
    await page.evaluate(() => { window.__inv = []; });
    const title = row(1).getByRole('gridcell').nth(1);
    await title.click({ position: { x: 14, y: 15 } });
    await page.keyboard.press('End');
    await page.waitForFunction((n) => document.activeElement?.closest('tr')?.querySelector(`[aria-label="Select row ${n}"]`), count);
    assert.match(await row(count).innerText(), /Task number 199999/);
    assert((await table.locator('tr[data-row-id]').count()) < 120);
    const calls = await page.evaluate(() => window.__inv.map(([command]) => command));
    assert(calls.includes('view_rows'));
    assert(!calls.includes('query_view'), 'A keyboard jump reads the existing generation');

    await page.getByRole('button', { name: 'Search rows', exact: true }).click();
    await page.getByPlaceholder('Search…').fill('Task number 199999');
    await page.waitForFunction(() => document.querySelector('[aria-label="Table"] [role="grid"]')?.getAttribute('aria-rowcount') === '4');
    await row(1).getByText('Task number 199999', { exact: true }).waitFor();
    assert.equal(await table.locator('tr[data-row-id]').count(), 1);

    const details = await page.evaluate(async () => {
      const { invoke } = await import('/dev-mock/core.ts');
      const tree = await invoke('list_notes');
      const db = tree.databases.find((d) => d.title === 'Tasks');
      const { views } = await invoke('get_database', { id: db.id });
      const request = { databaseId: db.id, viewId: views[0].id };
      const meta = await invoke('query_view', { request, windowed: true });
      const range = { ...request, generation: meta.generation, start: meta.rowIds.length - 2, count: 2 };
      const read = await invoke('view_rows', { request: range });
      const foreign = await invoke('view_rows', { request: { ...range, viewId: 'different view' } });
      let oversizeRejected = false;
      try { await invoke('view_rows', { request: { ...range, count: 257 } }); }
      catch { oversizeRejected = true; }
      const last = read[1];
      await invoke('set_cells', { rowId: last.id, title: 'Updated at the end', patch: '{}' });
      const stale = await invoke('view_rows', { request: range });
      const next = await invoke('query_view', { request, windowed: true });
      const fresh = await invoke('view_rows', { request: { ...range, generation: next.generation } });
      return {
        fullRowsSent: 'rows' in meta, count: meta.rowIds.length,
        tailMatches: read.map((r) => r.id).join() === meta.rowIds.slice(-2).join(),
        foreign, oversizeRejected, stale, changed: next.generation !== meta.generation, title: fresh[1].title,
      };
    });
    assert.deepEqual(details, {
      fullRowsSent: false, count, tailMatches: true, foreign: null,
      oversizeRejected: true, stale: null, changed: true, title: 'Updated at the end',
    });

    // Use a fresh small fixture whose matching group cannot request any row values.
    const fixtureUrl = new URL(url);
    fixtureUrl.searchParams.set('rows', '25');
    await page.goto(fixtureUrl.href);
    const tasks = page.getByRole('treeitem', { name: 'Tasks', exact: true });
    await tasks.waitFor();
    const groupKey = await page.evaluate(async () => {
      const { invoke } = await import('/dev-mock/core.ts');
      const tree = await invoke('list_notes');
      const { id } = tree.databases.find((db) => db.title === 'Tasks');
      const db = await invoke('get_database', { id });
      const status = db.props.find((prop) => prop.kind === 'status');
      const cost = db.props.find((prop) => prop.name === 'Cost');
      cost.numberFormat = 'numberWithCommas';
      const view = db.views[0];
      view.group = { ...db.views.find((candidate) => candidate.kind === 'board').group, collapsed: ['', ...status.options.map((option) => option.id)] };
      view.filter = { op: 'and', conds: [{ prop: db.props[0].id, op: 'contains', value: 'Design the landing page' }], groups: [] };
      await invoke('save_database', { databaseJson: JSON.stringify(db) });
      const meta = await invoke('query_view', { request: { databaseId: id, viewId: view.id }, windowed: true });
      await invoke('set_cells', { rowId: meta.rowIds[0], patch: JSON.stringify({ [cost.id]: 0.0000001 }) });
      window.__inv = [];
      return status.options[0].id;
    });
    await tasks.click();
    const group = table.locator(`tr[data-group-key="${groupKey}"]`);
    const collapsed = group.getByRole('button', { name: 'Expand group', exact: true });
    const noMatches = page.getByText('No rows match.');
    await collapsed.waitFor();
    await group.getByText('1', { exact: true }).waitFor();
    assert.equal(await table.locator('tr[data-row-id]').count(), 0);
    assert.equal(await noMatches.count(), 0, 'A filter match in a collapsed group has no empty banner');

    await page.getByRole('button', { name: 'Search rows', exact: true }).click();
    const search = page.getByPlaceholder('Search\u2026');
    await search.fill('no matching task');
    await group.getByText('0', { exact: true }).waitFor();
    await noMatches.waitFor();

    // Start from a settled zero-match result so the count must change before
    // this assertion. With the group collapsed, no window slice can load.
    await search.fill('1e-7');
    await group.getByText('1', { exact: true }).waitFor();
    assert.equal(await noMatches.count(), 0, 'Find must honor the count before any values are loaded');
    assert.equal(await table.locator('tr[data-row-id]').count(), 0);
    assert.equal(await page.evaluate(() => window.__inv.some(([cmd]) => cmd === 'view_rows')), false);

    await search.fill('1,e-7');
    await noMatches.waitFor();
    await page.getByRole('button', { name: 'Clear the filter and search', exact: true }).click();
    await noMatches.waitFor({ state: 'hidden' });
    assert.equal(await search.inputValue(), '');
    await page.getByRole('button', { name: 'Filter', exact: true }).click();
    await page.getByRole('button', { name: 'Add filter', exact: true }).click();
    await page.getByLabel('Value', { exact: true }).fill('no matching task');
    await page.keyboard.press('Escape');
    await noMatches.waitFor();
    await page.getByRole('button', { name: 'Clear the filter and search', exact: true }).click();
    await noMatches.waitFor({ state: 'hidden' });

    await search.fill('1e-7');
    await group.getByText('1', { exact: true }).waitFor();
    await collapsed.click();
    await row(1).getByText('1e-7', { exact: true }).waitFor();
    assert.equal(await noMatches.count(), 0);
    console.log('ALL OK P7 collapsed filter and Find, zero matches, Clear, and scientific number display');
    assert.deepEqual(errors, []);
    console.log('ALL OK P7 200000-row table, tail focus, Find, window bounds, identity and invalidation');
  } finally {
    await browser.close();
  }
})().catch((error) => { console.error(error); process.exitCode = 1; });
