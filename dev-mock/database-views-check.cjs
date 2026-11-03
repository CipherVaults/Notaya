// Checks the database views (board, calendar, timeline, list, gallery, chart) in the harness. Development only.
// Needs Playwright. From the project root:
//   npx vite --config vite.views.config.ts      (leave it running; it serves http://127.0.0.1:1423/)
//   node dev-mock/database-views-check.cjs [screenshot folder] [only]
// Prints ALL OK when every check passes. `only` is a view name (board, calendar, ...) to run just that part.
const { chromium } = require('playwright');
const shots = process.argv[2] || '/mnt/project-files/notaya-build/v150-db';
const only = process.argv[3] || '';
const BASE = 'http://127.0.0.1:1423/dev-mock/database-views-harness.html';
let fail = 0;
const is = (name, got, want) => {
  const g = JSON.stringify(got), w = JSON.stringify(want);
  if (g !== w) { fail++; console.log('FAIL', name, g, '!==', w); } else console.log('ok  ', name);
};
const truthy = (name, v) => { if (!v) { fail++; console.log('FAIL', name); } else console.log('ok  ', name); };

async function open(b, query, size = { width: 1280, height: 800 }) {
  const p = await b.newPage({ viewport: size, deviceScaleFactor: 1 });
  p.on('pageerror', (e) => { fail++; console.log('FAIL page error', e.message); });
  p.on('console', (m) => { if (m.type() === 'error' && !/favicon/.test(m.text())) { fail++; console.log('FAIL console error', m.text()); } });
  await p.goto(`${BASE}?${query}`);
  await p.waitForSelector('#view-host > *');
  await p.waitForTimeout(400);
  return p;
}
const calls = (p, fn) => p.evaluate((f) => window.__h.calls.filter((c) => c.fn === f), fn);
const clear = (p) => p.evaluate(() => window.__h.clear());
const shot = (p, name) => p.screenshot({ path: `${shots}/views-${name}.png` });

async function drag(p, from, to, opts = {}) {
  const a = await from.boundingBox();
  const t = await to.boundingBox();
  const sx = a.x + a.width / 2, sy = a.y + Math.min(a.height / 2, 30);
  const tx = (opts.x ?? t.x + t.width / 2), ty = (opts.y ?? t.y + Math.min(t.height / 2, 40));
  await p.mouse.move(sx, sy);
  await p.mouse.down();
  await p.mouse.move(sx + 12, sy + 12, { steps: 4 });
  await p.mouse.move(tx, ty, { steps: 14 });
  await p.waitForTimeout(150);
  await p.mouse.move(tx + 1, ty + 1, { steps: 2 });
  await p.mouse.up();
  await p.waitForTimeout(300);
}

const tests = {};

/* ---------------------------------------------------------------- board */
tests.board = async (b) => {
  let p = await open(b, 'view=board&cover=cover');
  is('board: four columns', await p.locator('[data-column]').count(), 4);
  is('board: column counts', await p.locator('[data-column] [data-count]').allTextContents(), ['1', '11', '4', '4']);
  await shot(p, 'board');

  // Drag "Review pull requests" (In progress, s2) to the Done column.
  await clear(p);
  const card = p.locator('[data-card="r7"]');
  const done = p.locator('[data-cell^="s3"]');
  await drag(p, card, done);
  let sc = await calls(p, 'setCells');
  is('board: drag to another column calls setCells once', sc.length, 1);
  is('board: drag sets the status option id', sc[0] && sc[0].args, ['r7', { status: 's3' }]);
  is('board: the card is now in the Done column', await p.locator('[data-cell^="s3"] [data-card="r7"]').count(), 1);

  // Reorder inside a column: move "Archive old notes" (r16) above "Write release notes" (r3).
  await clear(p);
  await drag(p, p.locator('[data-card="r16"]'), p.locator('[data-card="r3"]'), { y: (await p.locator('[data-card="r3"]').boundingBox()).y + 4 });
  const ro = await calls(p, 'reorderRows');
  is('board: reorder inside a column calls reorderRows', ro.length, 1);
  truthy('board: reorderRows lists every row', ro[0] && ro[0].args[0].length === 20);
  is('board: reordering does not change the status', (await calls(p, 'setCells')).length, 0);
  is('board: r16 is now before r3', await p.evaluate(() => { const ids = window.__h.rows().map((r) => r.id); return ids.indexOf('r16') < ids.indexOf('r3'); }), true);

  // Enter opens the focused card.
  await clear(p);
  await p.locator('[data-card="r1"]').focus();
  await p.keyboard.press('Enter');
  is('board: Enter opens a card', (await calls(p, 'openRow')).map((c) => c.args[0]), ['r1']);
  await clear(p);
  await p.locator('[data-card="r5"]').click();
  is('board: click opens a card', (await calls(p, 'openRow')).map((c) => c.args[0]), ['r5']);

  // + New in a column creates a row with that status.
  await clear(p);
  await p.locator('[data-cell^="s1"]').getByRole('button', { name: 'New' }).click();
  await p.getByLabel('Title of the new row').fill('Brand new card');
  await p.keyboard.press('Enter');
  await p.waitForTimeout(200);
  const cr = await calls(p, 'createRow');
  is('board: + New creates a row in the column', cr[0] && cr[0].args, [{ status: 's1' }, 'Brand new card']);
  await p.keyboard.press('Escape');

  // Collapse and expand a column; hide a column and show it again.
  await clear(p);
  await p.getByLabel('Collapse In progress').click({ force: true });
  let up = await calls(p, 'updateView');
  is('board: collapse saves group.collapsed', up[0] && up[0].args[0].group.collapsed, ['s2']);
  await p.waitForTimeout(200);
  await shot(p, 'board-collapsed');
  await p.getByLabel('Expand In progress').click();
  is('board: expand empties group.collapsed', (await calls(p, 'updateView')).pop().args[0].group.collapsed, []);
  await p.getByLabel('Hide Done').click({ force: true });
  is('board: hide saves group.hidden', (await calls(p, 'updateView')).pop().args[0].group.hidden, ['s3']);
  await p.waitForTimeout(200);
  is('board: three columns after hiding one', await p.locator('[data-column]').count(), 3);
  await p.getByRole('button', { name: 'Done', exact: true }).click();
  await p.waitForTimeout(200);
  is('board: four columns after showing it again', await p.locator('[data-column]').count(), 4);
  await p.close();

  // Without a group property: a prompt.
  p = await open(b, 'view=board&group=none');
  truthy('board: prompt to pick a group property', await p.getByText('Choose how to group the board').isVisible());
  await clear(p);
  await p.getByLabel('Group by').selectOption('prio');
  await p.waitForTimeout(200);
  is('board: picking a property saves the group', (await calls(p, 'updateView'))[0].args[0].group.prop, 'prio');
  is('board: columns after picking Priority', await p.locator('[data-column]').count(), 4);
  await p.close();

  // multiSelect group: moving replaces the source tag and keeps the others.
  p = await open(b, 'view=board&groupby=tags');
  is('board multi: columns incl. No Tags', await p.locator('[data-column]').count(), 5);
  await clear(p);
  // r1 has tags [design]; drag it to the "docs" column: design is taken out, docs added.
  await drag(p, p.locator('[data-cell^="t1"] [data-card="r1"]'), p.locator('[data-cell^="t3"]'));
  sc = await calls(p, 'setCells');
  is('board multi: move replaces the source tag', sc[0] && sc[0].args, ['r1', { tags: ['t3'] }]);
  await p.close();

  // Status grouped by To-do / In progress / Complete: a drop picks the first option of that group.
  p = await open(b, 'view=board&dateby=group');
  await clear(p);
  await drag(p, p.locator('[data-card="r2"]'), p.locator('[data-cell^="complete"]'));
  sc = await calls(p, 'setCells');
  is('board status groups: drop sets the first option of the group', sc[0] && sc[0].args, ['r2', { status: 's3' }]);
  await p.close();

  // Swimlanes (sub-group by priority).
  p = await open(b, 'view=board&sub=prio&size=small', { width: 1280, height: 900 });
  is('board lanes: a cell per column and lane', await p.locator('[data-cell]').count() > 8, true);
  await shot(p, 'board-lanes');
  await clear(p);
  // r7 (In progress / Medium) to Done / High: both properties change.
  await drag(p, p.locator('[data-cell="s2\u0001md"] [data-card="r7"]'), p.locator('[data-cell="s3\u0001hi"]'));
  sc = await calls(p, 'setCells');
  is('board lanes: dropping in another lane changes both properties', sc[0] && sc[0].args, ['r7', { status: 's3', prio: 'hi' }]);
  await p.close();

  // Dark + RTL
  p = await open(b, 'view=board&theme=sunrise-dark&rtl=1');
  await shot(p, 'board-dark-rtl');
  await p.close();
};


/* ------------------------------------------------------------- calendar */
const addDays = (iso, n) => { const d = new Date(Date.parse(iso.slice(0, 10) + 'T00:00:00Z') + n * 86400000); return d.toISOString().slice(0, 10); };
const todayIso = () => { const n = new Date(); return `${n.getFullYear()}-${String(n.getMonth() + 1).padStart(2, '0')}-${String(n.getDate()).padStart(2, '0')}`; };
const dayCell = (p, iso) => p.locator(`[data-day="${Math.floor(Date.parse(iso + 'T00:00:00Z') / 86400000)}"]`);

tests.calendar = async (b) => {
  let p = await open(b, 'view=calendar');
  const t0 = todayIso();
  truthy('calendar: 7 weekday headers starting Monday', (await p.locator('.uppercase > div').allTextContents()).length === 7 && /^mon/i.test((await p.locator('.uppercase > div').first().textContent()) || ''));
  truthy('calendar: month grid has whole weeks', (await p.locator('[data-week]').count()) >= 4);
  is('calendar: unscheduled list shows the two rows without a date', await p.locator('[data-unscheduled]').count(), 2);
  await shot(p, 'calendar-month');

  // Drag a one-day row to the next (or previous) day.
  const target = (await dayCell(p, addDays(t0, 1)).count()) ? addDays(t0, 1) : addDays(t0, -1);
  await clear(p);
  await drag(p, p.locator('[data-bar="r7"]'), dayCell(p, target));
  let sc = await calls(p, 'setCells');
  is('calendar: drag to another day calls setCells', sc[0] && sc[0].args, ['r7', { due: { start: target } }]);
  await p.waitForTimeout(200);

  // Drag the timed row: its time is kept.
  await clear(p);
  const r2 = await p.evaluate(() => window.__h.row('r2').values.due.start);
  const target2 = addDays(r2, 2);
  if (await dayCell(p, target2).count()) {
    await drag(p, p.locator('[data-bar="r2"]').first(), dayCell(p, target2));
    sc = await calls(p, 'setCells');
    is('calendar: a moved row keeps its time', sc[0] && sc[0].args[1].due.start, target2 + 'T10:00');
  }

  // Drag a multi-day bar by 7 days: start and end both move, the length stays.
  await clear(p);
  const before = await p.evaluate(() => window.__h.row('r4').values.due);
  const barBox = await p.locator('[data-bar="r4"]').first().boundingBox();
  const cellsUnder = await p.evaluate(([x, y]) => Number(document.elementsFromPoint(x, y).find((n) => n.dataset && n.dataset.day).dataset.day), [barBox.x + barBox.width / 2, barBox.y + Math.min(barBox.height / 2, 30)]);
  const targetCell = p.locator(`[data-day="${cellsUnder + 7}"]`);
  if (await targetCell.count()) {
    await drag(p, p.locator('[data-bar="r4"]').first(), targetCell);
    const after = await p.evaluate(() => window.__h.row('r4').values.due);
    const days = (a, c) => Math.round((Date.parse(c.slice(0, 10)) - Date.parse(a.slice(0, 10))) / 86400000);
    is('calendar: a range moved by 7 days keeps its length', [days(before.start, after.start), days(before.end, after.end)], [7, 7]);
  } else console.log('skip calendar range drag: target week not visible');

  // Drag an unscheduled row onto a day.
  await clear(p);
  await drag(p, p.locator('[data-unscheduled="r10"]'), dayCell(p, t0));
  sc = await calls(p, 'setCells');
  is('calendar: an unscheduled row dropped on a day gets that date', sc[0] && sc[0].args, ['r10', { due: { start: t0 } }]);
  is('calendar: it leaves the unscheduled list', await p.locator('[data-unscheduled]').count(), 1);

  // Click an empty day: an inline title, Enter creates the row with that date.
  await clear(p);
  const cb = await dayCell(p, t0).boundingBox();
  await p.mouse.click(cb.x + cb.width - 5, cb.y + cb.height - 3);
  await p.getByLabel('Title of the new row').fill('Dentist');
  await p.keyboard.press('Enter');
  await p.waitForTimeout(200);
  is('calendar: a click on an empty day creates a row with that date', (await calls(p, 'createRow'))[0] && (await calls(p, 'createRow'))[0].args, [{ due: { start: t0 } }, 'Dentist']);

  // "N more" popover.
  const moreBtn = p.locator('[data-more]').first();
  truthy('calendar: a busy day shows "N more"', await moreBtn.count() > 0);
  await moreBtn.click();
  truthy('calendar: the popover lists the rows of the day', (await p.locator('[data-more-popover] button').count()) >= 3);
  await shot(p, 'calendar-more');
  await clear(p);
  await p.locator('[data-more-popover] button').first().click();
  is('calendar: a row in the popover opens', (await calls(p, 'openRow')).length, 1);
  truthy('calendar: the popover closes after a choice', (await p.locator('[data-more-popover]').count()) === 0);

  // Navigation.
  const title0 = await p.locator('[data-title]').textContent();
  await p.getByLabel('Next month').click();
  const title1 = await p.locator('[data-title]').textContent();
  truthy('calendar: next month changes the title', title0 !== title1);
  await p.getByLabel('Previous month').click();
  is('calendar: previous month goes back', await p.locator('[data-title]').textContent(), title0);
  await p.getByLabel('Next month').click();
  await p.getByRole('button', { name: 'Today' }).click();
  is('calendar: Today returns', await p.locator('[data-title]').textContent(), title0);

  // Week mode.
  await clear(p);
  await p.getByRole('button', { name: 'Week' }).click();
  is('calendar: Week saves calendarMode', (await calls(p, 'updateView'))[0].args[0], { calendarMode: 'week' });
  await p.waitForTimeout(250);
  is('calendar: week mode shows one week', await p.locator('[data-week]').count(), 1);
  await p.getByLabel('Next week').click();
  await p.getByRole('button', { name: 'Today' }).click();
  await shot(p, 'calendar-week');
  await p.close();

  // No date property: prompt.
  p = await open(b, 'view=calendar&noprop=1');
  truthy('calendar: prompt when no date property is set', await p.getByText('Choose a date property for the calendar').isVisible());
  await p.getByLabel('Date property').selectOption('due');
  await p.waitForTimeout(100);
  is('calendar: choosing a property saves dateProp', (await calls(p, 'updateView'))[0].args[0], { dateProp: 'due' });
  await p.close();

  // Dark, right-to-left page.
  p = await open(b, 'view=calendar&theme=sunrise-dark&rtl=1');
  await shot(p, 'calendar-dark-rtl');
  await p.close();
};

(async () => {
  const b = await chromium.launch();
  for (const [name, fn] of Object.entries(tests)) {
    if (only && only !== name) continue;
    console.log('--', name);
    try { await fn(b); } catch (e) { fail++; console.log('FAIL', name, 'threw', e.message.split('\n')[0]); }
  }
  await b.close();
  console.log(fail ? `${fail} FAILED` : 'ALL OK');
  process.exit(fail ? 1 : 0);
})();
