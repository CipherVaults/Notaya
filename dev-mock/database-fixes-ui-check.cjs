// Checks the V1.5.1 database fixes in the mock UI: visible scrollbars, undo and redo, clicking a tag or a date in a table
// opens its picker, adding an action in the automations dialog (it stays open and keeps the action), clickable charts with a
// list of the rows behind a bar / slice / point / number, and readable native dropdowns in every look. Development only;
// needs Playwright. From the project root:
//   npm run mock            (leave it running; it serves http://127.0.0.1:1421/)
//   node dev-mock/database-fixes-ui-check.cjs [screenshot folder]
// Prints ALL OK when every check passes. The data is the mock's, in memory; nothing real is touched.
const { chromium } = require('playwright');
const shots = process.argv.slice(2).find((a) => !a.startsWith('--'));
let fail = 0;
const is = (name, got, want) => {
  if (got !== want) { fail++; console.log('FAIL', name, JSON.stringify(got), '!==', JSON.stringify(want)); } else console.log('ok  ', name);
};
const yes = (name, got) => is(name, !!got, true);
const URL = process.env.MOCK_URL || 'http://127.0.0.1:1421/?db=1';

(async () => {
  // Playwright hides scrollbars by default; the point here is that they show.
  const b = await chromium.launch({ ignoreDefaultArgs: ['--hide-scrollbars'] });
  const ctx = await b.newContext({ viewport: { width: 1100, height: 700 }, deviceScaleFactor: 1 });
  const p = await ctx.newPage();
  p.on('pageerror', (e) => { fail++; console.log('FAIL page error', e.message); });
  const shot = async (name) => { if (shots) await p.screenshot({ path: `${shots}/fix-${name}.png`, caret: 'initial' }); };
  const wait = (ms = 250) => p.waitForTimeout(ms);
  const table = () => p.getByRole('application', { name: 'Table' });
  // A DOM index is a window position, not a row identity after New scrolls.
  const row = (i) => table().locator('tbody tr[aria-selected]').filter({ has: p.getByRole('checkbox', { name: `Select row ${i + 1}`, exact: true }) });
  const cell = (r, c) => row(r).locator('td').nth(c);
  const readRows = () => p.evaluate(async () => {
    const { api } = await import('/src/lib/ipc.ts');
    const databaseId = document.querySelector('[data-database]').dataset.database;
    const db = await api.getDatabase(databaseId);
    return (await api.queryView({ databaseId, viewId: db.views[0].id })).rows;
  });
  const backToFirstRow = async () => {
    await table().evaluate((el) => { el.scrollTop = 0; });
    await row(0).waitFor();
  };
  const pop = () => p.locator('[data-radix-popper-content-wrapper]');
  const look = (t) => p.evaluate((x) => { document.documentElement.dataset.theme = x; document.documentElement.dataset.mode = /black|graphite|sunrise-dark/.test(x) ? 'dark' : 'light'; }, t);
  const col = async (name) => (await table().locator('thead th').allInnerTexts()).map((t) => t.trim()).indexOf(name) + 0;

  await p.goto(URL);
  await wait(1200);
  await p.getByRole('treeitem', { name: 'Tasks' }).click();
  await wait(900);
  const cStatus = await col('Status'), cDue = await col('Due'), cTags = await col('Tags'), cDone = await col('Done'), cTask = await col('Task');

  // ---------------------------------------------------------------- 1. scrollbars
  const bars = await p.evaluate(() => {
    const w = document.querySelector('[aria-label=Table]');
    const r = w.getBoundingClientRect();
    const th = getComputedStyle(w, '::-webkit-scrollbar-thumb').backgroundColor;
    return { vBar: w.offsetWidth - w.clientWidth, hBar: w.offsetHeight - w.clientHeight, bottom: r.bottom, vh: innerHeight, thumb: th };
  });
  yes('the table has a vertical bar at the right (10px or more)', bars.vBar >= 10);
  yes('the table has a horizontal bar (10px or more)', bars.hBar >= 10);
  yes('the horizontal bar is inside the window, not far below it', bars.bottom <= bars.vh + 1);
  yes('the scrollbar thumb is not transparent at rest', bars.thumb && !/rgba\(0, 0, 0, 0\)|transparent/.test(bars.thumb));
  for (const t of ['sunrise', 'black']) {
    await look(t);
    await wait(150);
    await shot(`table-bars-${t}`);
  }
  await look('sunrise');

  // ---------------------------------------------------------------- 3. tags and dates open on a click
  await cell(0, cStatus).click({ position: { x: 14, y: 16 } });
  await wait(350);
  is('one click on a status cell opens its picker', await pop().count(), 1);
  yes('the picker lists the options', await pop().getByRole('option', { name: /In progress/ }).count());
  await shot('status-picker');
  await pop().getByRole('option', { name: /In progress/ }).first().click();
  await wait(500);
  is('the picker closes after a pick', await pop().count(), 0);
  yes('the cell shows the new status', (await cell(0, cStatus).innerText()).includes('In progress'));

  await cell(0, cDue).click({ position: { x: 14, y: 16 } });
  await wait(350);
  is('one click on a date cell opens the date picker', await pop().count(), 1);
  yes('it shows the calendar', await pop().getByRole('grid').count());
  await shot('date-picker');
  await p.keyboard.press('Escape');
  await wait(250);
  is('Escape closes the date picker', await pop().count(), 0);
  await cell(0, cDue).click({ position: { x: 14, y: 16 } });
  await wait(350);
  is('a click on the same date cell opens it again', await pop().count(), 1);
  await p.keyboard.press('Escape');
  await wait(250);

  await cell(0, cTags).click({ position: { x: 14, y: 16 } });
  await wait(350);
  is('one click on a multi-select cell opens its picker', await pop().count(), 1);
  await p.keyboard.press('Escape');
  await wait(250);
  // Click right on a tag of an active cell.
  const chip = cell(1, cStatus).locator('span[title]').first();
  await chip.click();
  await wait(350);
  is('a click directly on a tag opens its picker', await pop().count(), 1);
  await p.keyboard.press('Escape');
  await wait(250);
  await chip.click();
  await wait(350);
  is('a second click on the (now active) tag opens it again', await pop().count(), 1);
  await p.keyboard.press('Escape');
  await wait(250);

  // ---------------------------------------------------------------- 2. undo and redo
  const toastText = async () => (await p.locator('[data-sonner-toast]').allInnerTexts()).join(' | ');
  const statusNow = async () => (await cell(0, cStatus).innerText()).trim();
  is('the status is In progress before the undo', await statusNow(), 'In progress');
  await cell(0, cTask).click(); // takes the focus into the table, no editing
  await p.keyboard.press('Control+z');
  await wait(700);
  is('Ctrl+Z puts the old status back', await statusNow(), 'Not started');
  yes('a toast says what was undone', /Undone: edit of Status/.test(await toastText()));
  await p.keyboard.press('Control+Shift+z');
  await wait(700);
  is('Ctrl+Shift+Z does it again', await statusNow(), 'In progress');
  yes('a toast says what was redone', /Redone: edit of Status/.test(await toastText()));
  await p.keyboard.press('Control+z');
  await wait(600);
  await p.keyboard.press('Control+y');
  await wait(700);
  is('Ctrl+Y redoes too', await statusNow(), 'In progress');

  // a cleared value (the mock date follows today's clock, not a fixed month)
  const dateBefore = (await cell(0, cDue).innerText()).trim();
  await cell(0, cDue).click({ position: { x: 3, y: 3 } });
  await p.keyboard.press('Escape');
  await wait(200);
  await cell(0, cTask).click();
  await cell(0, cDue).click({ position: { x: 14, y: 16 } });
  await wait(300);
  await p.getByRole('button', { name: 'Clear' }).click();
  await wait(600);
  await p.keyboard.press('Escape');
  await wait(300);
  is('the date was cleared', (await cell(0, cDue).innerText()).trim(), '');
  await cell(0, cTask).click();
  await p.keyboard.press('Control+z');
  await wait(700);
  is('undo brings the date back', (await cell(0, cDue).innerText()).trim(), dateBefore);

  // a checkbox
  const done = () => cell(2, cDone).getByRole('checkbox');
  const was = await done().getAttribute('aria-checked');
  await done().click();
  await wait(600);
  yes('the checkbox flipped', (await done().getAttribute('aria-checked')) !== was);
  await cell(2, cTask).click();
  await p.keyboard.press('Control+z');
  await wait(700);
  is('undo flips the checkbox back', await done().getAttribute('aria-checked'), was);

  // a new row and its undo
  const beforeAdd = await readRows();
  const rowsBefore = beforeAdd.length;
  await p.getByRole('button', { name: 'New', exact: true }).first().click();
  await wait(500);
  await p.keyboard.type('Undo me');
  await p.keyboard.press('Enter');
  await wait(600);
  const afterAdd = await readRows();
  const addedRow = afterAdd.find((r) => !beforeAdd.some((before) => before.id === r.id));
  is('a row was added', afterAdd.length, rowsBefore + 1);
  is('the new row has the typed name', addedRow?.title, 'Undo me');
  await backToFirstRow();
  // typing in a text field keeps its own undo: Ctrl+Z there must not undo the database
  await cell(1, cTask).click();
  await p.keyboard.press('Enter');
  await wait(150);
  await p.keyboard.type('xyz');
  const undoneBefore = await p.locator('[data-sonner-toast]', { hasText: /Undone|Nothing to undo/ }).count();
  await p.keyboard.press('Control+z');
  await wait(400);
  is('Ctrl+Z inside a text cell is the browser\'s (no database undo)', await p.locator('[data-sonner-toast]', { hasText: /Undone|Nothing to undo/ }).count(), undoneBefore);
  await p.keyboard.press('Escape');
  await wait(300);
  await cell(1, cTask).click();
  await p.keyboard.press('Control+z'); // the name typed into the new row
  await wait(700);
  const afterNameUndo = await readRows();
  is('the first Ctrl+Z keeps the new row', afterNameUndo.length, rowsBefore + 1);
  is('the first Ctrl+Z takes the typed name back', afterNameUndo.find((r) => r.id === addedRow?.id)?.title, '');
  await p.keyboard.press('Control+z'); // the new row itself
  await wait(900);
  const afterRowUndo = await readRows();
  is('the next Ctrl+Z takes the new row away again', afterRowUndo.length, rowsBefore);
  yes('the added row ID is absent after undo', !afterRowUndo.some((r) => r.id === addedRow?.id));
  await p.keyboard.press('Control+Shift+z');
  await wait(900);
  const afterRowRedo = await readRows();
  is('Ctrl+Shift+Z brings it back', afterRowRedo.length, rowsBefore + 1);
  is('redo restores the same row with its empty name', afterRowRedo.find((r) => r.id === addedRow?.id)?.title, '');
  await p.keyboard.press('Control+Shift+z');
  await wait(700);
  is('and again the typed name', (await readRows()).find((r) => r.id === addedRow?.id)?.title, 'Undo me');
  await shot('after-undo');

  // The row page.
  await backToFirstRow();
  await cell(0, cTask).getByRole('button', { name: 'Open row' }).click({ force: true });
  await wait(900);
  const pageStatus = p.getByRole('list', { name: 'Properties' }).getByText('In progress').first();
  yes('the row page shows the Status property', await pageStatus.count());
  await pageStatus.click();
  await wait(400);
  is('a click on a tag on the row page opens its picker', await pop().count(), 1);
  await p.keyboard.press('Escape');
  await wait(250);
  await p.getByRole('treeitem', { name: 'Tasks' }).click();
  await wait(900);

  // ---------------------------------------------------------------- 4. automations: adding an action
  await p.getByRole('button', { name: 'Database menu' }).click();
  await p.getByRole('menuitem', { name: /Automations/ }).click();
  await wait(400);
  const dlg = p.getByRole('dialog');
  await dlg.getByRole('button', { name: 'New automation' }).click();
  await wait(900);
  await dlg.getByRole('button', { name: 'Add an action' }).click();
  await wait(2500); // longer than the delay before a change is saved
  is('the dialog stays open after Add an action', await dlg.count(), 1);
  is('both actions are still there after the save', await dlg.getByLabel('Action', { exact: true }).count(), 2);
  is('the new action already names a property', (await dlg.getByLabel('Property', { exact: true }).nth(1).inputValue()) !== '', true);
  await shot('automation-add-action');
  await p.keyboard.press('Escape');
  await wait(400);

  // ---------------------------------------------------------------- 6. dropdowns in the dark looks
  await p.getByRole('button', { name: 'Sort' }).click();
  await wait(300);
  await p.getByRole('button', { name: /Add sort/ }).click();
  await wait(300);
  const optCols = {};
  for (const t of ['sunrise', 'white', 'paper', 'sunrise-dark', 'black', 'graphite']) {
    await look(t);
    await wait(100);
    optCols[t] = await p.evaluate(() => {
      const sel = document.querySelector('select');
      if (!sel) return null;
      const o = sel.querySelector('option');
      const cs = getComputedStyle(o);
      const root = getComputedStyle(document.documentElement);
      return { bg: cs.backgroundColor, fg: cs.color, scheme: root.colorScheme };
    });
  }
  console.log(JSON.stringify(optCols));
  const lum = (c) => { const m = /rgba?\((\d+), (\d+), (\d+)(?:, ([\d.]+))?/.exec(c || ''); return m ? (0.2126 * +m[1] + 0.7152 * +m[2] + 0.0722 * +m[3]) / 255 : -1; };
  for (const t of ['sunrise-dark', 'black', 'graphite']) {
    const o = optCols[t];
    yes(`${t}: option background is dark and opaque, text is light`, o && lum(o.bg) < 0.25 && lum(o.fg) > 0.6 && !/rgba\(.*, 0\)$/.test(o.bg));
    is(`${t}: native controls are dark`, o && o.scheme, 'dark');
  }
  for (const t of ['sunrise', 'white', 'paper']) {
    const o = optCols[t];
    yes(`${t}: option background is light and opaque, text is dark`, o && lum(o.bg) > 0.75 && lum(o.fg) < 0.4);
  }
  await look('black');
  await p.evaluate(() => { document.querySelectorAll('select').forEach((s) => { s.size = Math.min(5, s.options.length); s.style.height = 'auto'; }); });
  await wait(150);
  await shot('sort-select-black');
  await p.keyboard.press('Escape');
  await look('sunrise');
  await p.getByRole('treeitem', { name: 'Tasks' }).click();
  await wait(700);

  // ---------------------------------------------------------------- 5. charts
  await p.getByRole('tab', { name: /Chart/ }).click();
  await wait(1200);
  await look('sunrise');
  yes('the chart shows bars', (await p.locator('[data-bar]').count()) > 0);
  const firstBar = p.locator('[data-bar-point]').first();
  await firstBar.hover();
  await wait(250);
  const tip = (await p.locator('[data-tooltip]').textContent()) || '';
  yes('the tooltip shows the group name', tip.length > 0);
  yes('the tooltip shows a percent', /%/.test(tip));
  yes('the tooltip says "Click to view data"', /Click to view data/.test(tip));
  await shot('chart-tooltip');
  const label = await firstBar.getAttribute('aria-label');
  await firstBar.click();
  await wait(900);
  const drill = p.locator('[data-chart-drill]');
  is('a click on a bar opens the list of rows', await drill.count(), 1);
  const group = (label || '').split(':')[0];
  yes('the title is the group name', (await drill.getByRole('heading').innerText()).trim() === group);
  yes('a chip shows the condition', /is/.test((await drill.locator('[data-drill-chip]').first().innerText()) || ''));
  const nRows = await drill.locator('[data-drill-row]').count();
  yes('the list holds rows', nRows > 0);
  const countInBar = Number((/: (\d+)/.exec(label || '') || [])[1]);
  is('the number of rows equals the value of the bar', nRows, countInBar);
  yes('a close button is there', await drill.getByRole('button', { name: 'Close' }).count());
  await shot('chart-drill');
  await drill.getByRole('button', { name: 'Close' }).click();
  await wait(300);
  is('the close button closes it', await drill.count(), 0);
  await firstBar.click();
  await wait(700);
  await drill.locator('[data-drill-row]').first().click();
  await wait(700);
  is('a click on a row closes the list', await drill.count(), 0);
  yes('and opens the row', (await p.getByRole('dialog').count()) > 0 || (await p.getByRole('navigation', { name: 'Breadcrumb' }).innerText()).split('\n').length > 0);
  await p.getByRole('treeitem', { name: 'Tasks' }).click();
  await wait(900);
  await p.getByRole('tab', { name: /Chart/ }).click();
  await wait(900);
  // the other chart types
  const kinds = { Donut: '[data-slice]', Line: '[data-point-hit]' };
  for (const [name, sel] of Object.entries(kinds)) {
    await p.getByRole('button', { name: 'Customize' }).click().catch(() => undefined);
    await wait(200);
    if (!(await p.getByRole('group', { name: 'Chart type' }).count())) { await p.getByRole('button', { name: 'Customize' }).click(); await wait(200); }
    await p.getByRole('group', { name: 'Chart type' }).getByRole('button', { name }).click();
    await wait(900);
    const first = p.locator(sel).first();
    if (name === 'Donut') {
      // the slice is a stroke only: aim at the middle of its arc, not at the middle of its box (that is the hole)
      const at = await first.evaluate((el) => {
        const dash = parseFloat(el.getAttribute('stroke-dasharray'));
        const off = parseFloat(el.getAttribute('stroke-dashoffset'));
        const pt = el.getPointAtLength(-off + dash / 2).matrixTransform(el.getScreenCTM());
        return { x: pt.x, y: pt.y };
      });
      await p.mouse.move(at.x, at.y);
    } else await first.hover({ force: true });
    await wait(250);
    yes(`${name}: the tooltip says "Click to view data"`, /Click to view data/.test((await p.locator('[data-tooltip]').textContent().catch(() => '')) || ''));
    if (name === 'Donut') await p.mouse.down().then(() => p.mouse.up()); else await first.click({ force: true });
    await wait(900);
    is(`${name}: a click opens the list of rows`, await drill.count(), 1);
    yes(`${name}: it lists rows`, (await drill.locator('[data-drill-row]').count()) > 0);
    if (name === 'Donut') await shot('chart-donut-drill');
    await p.keyboard.press('Escape');
    await wait(300);
    is(`${name}: Escape closes the list`, await drill.count(), 0);
  }
  await p.getByRole('group', { name: 'Chart type' }).getByRole('button', { name: 'Number' }).click();
  await wait(900);
  await p.locator('[data-chart="number"]').click();
  await wait(900);
  is('Number: a click opens the list of all rows', await drill.count(), 1);
  yes('Number: it lists rows', (await drill.locator('[data-drill-row]').count()) > 5);
  await p.keyboard.press('Escape');
  await wait(300);
  await look('black');
  await p.getByRole('group', { name: 'Chart type' }).getByRole('button', { name: 'Bar' }).click();
  await wait(900);
  await p.locator('[data-bar-point]').first().click();
  await wait(800);
  await shot('chart-drill-black');

  console.log(fail ? `${fail} FAILED` : 'ALL OK');
  await b.close();
  process.exit(fail ? 1 : 0);
})();
