// Times the Table view of a big database in the mock UI (EX-2026-11-09-65). Development only; not part of the app.
// It measures the screen work (React and the browser). The mock's own "backend" is JavaScript, so its time is
// listed apart (calls) and is NOT the Rust engine's time: for that use tools/db-bench.
//   npm run mock             (leave it running)
//   node dev-mock/perf-table-check.cjs 5000 [showMoreClicks]
// Prints, for one interaction at a time: wall time and the main-thread "long task" time. Numbers depend on the
// computer; compare before and after on the SAME computer. Throwaway data only.
const { chromium } = require('playwright');
(async () => {
  const n = Number(process.argv[2] || 5000);
  const more = Number(process.argv[3] || 0);
  const b = await chromium.launch();
  const p = await (await b.newContext({ viewport: { width: 1400, height: 900 } })).newPage();
  await p.addInitScript(() => { window.__long = 0; new PerformanceObserver((l) => { for (const e of l.getEntries()) window.__long += e.duration; }).observe({ entryTypes: ['longtask'] }); });
  await p.goto(`http://127.0.0.1:1421/?db=1&rows=${n}`);
  await p.waitForTimeout(2000);
  await p.getByRole('treeitem', { name: 'Tasks' }).click();
  const table = p.getByRole('application', { name: 'Table' });
  const rows = () => table.locator('tbody tr[aria-selected]');
  await rows().first().waitFor();
  await p.waitForTimeout(1500);
  const settle = () => p.evaluate(() => new Promise((r) => setTimeout(() => requestAnimationFrame(() => requestAnimationFrame(r)), 500)));
  for (let i = 0; i < more; i++) { await p.getByRole('button', { name: /Show more rows/ }).click(); await settle(); }
  const dom = await p.evaluate(() => ({ nodes: document.querySelectorAll('*').length, rows: document.querySelectorAll('[aria-label="Table"] tbody tr[aria-selected]').length }));
  console.log(`rows in the database ${n}, rows on screen ${dom.rows}, DOM nodes ${dom.nodes}`);
  const timeIt = async (label, fn) => {
    await p.evaluate(() => { window.__long = 0; window.__inv = []; });
    const t = Date.now(); await fn(); await settle();
    console.log(label.padEnd(30), 'wall', Date.now() - t - 500, 'ms | main-thread long tasks', await p.evaluate(() => Math.round(window.__long)), 'ms | mock calls', JSON.stringify(await p.evaluate(() => window.__inv.map((x) => x[0] + ':' + Math.round(x[1])))));
  };
  await timeIt('select a row (checkbox)', () => rows().nth(2).getByRole('checkbox').first().click());
  await timeIt('deselect', () => rows().nth(2).getByRole('checkbox').first().click());
  await timeIt('click a cell', () => rows().nth(3).locator('td').nth(1).click());
  await p.keyboard.press('Escape');
  await timeIt('edit a title, Enter', async () => { await rows().nth(3).locator('td').nth(1).click(); await p.keyboard.press('Enter'); await table.getByRole('textbox', { name: 'Task', exact: true }).fill('X'); await p.keyboard.press('Enter'); });
  await timeIt('sort by a column header', () => table.locator('thead th').nth(3).click());
  console.log('JS heap MB', await p.evaluate(() => (performance.memory ? Math.round(performance.memory.usedJSHeapSize / 1e6) : null)));
  await b.close();
})();
