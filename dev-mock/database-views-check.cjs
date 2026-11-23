// Checks the database views (board, calendar, timeline, list, gallery, chart) in the harness. Development only.
// Needs Playwright. From the project root:
//   npx vite --config vite.views.config.ts      (leave it running; it serves http://127.0.0.1:1423/)
//   node dev-mock/database-views-check.cjs [screenshot folder] [only]
// Prints ALL OK when every check passes. `only` is a view name (board, calendar, ...) to run just that part.
const { createChecks } = require('../tools/checks/fail-loud.cjs');
const { assertFixtureLine } = require('./grader-oracles.cjs');
const checks = createChecks('database views');
const shots = process.argv[2] || '/mnt/project-files/notaya-build/v150-db';
const only = process.argv[3] || '';
const BASE = 'http://127.0.0.1:1423/dev-mock/database-views-harness.html';
const is = (name, got, want) => checks.equal(name, got, want);
const truthy = (name, value) => checks.check(name,
  (typeof value !== 'number' || Number.isFinite(value)) && !!value);

async function open(b, query, size = { width: 1280, height: 800 }) {
  const p = await b.newPage({ viewport: size, deviceScaleFactor: 1 });
  p.on('pageerror', (e) => checks.check('page error', false, e.message));
  p.on('console', (m) => { if (m.type() === 'error' && !/favicon\.ico$/.test(m.location().url)) checks.check('console error', false, m.text()); });
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

  // A locked database still lets rows change.
  p = await open(b, 'view=board&locked=1');
  await clear(p);
  await drag(p, p.locator('[data-card="r7"]'), p.locator('[data-cell^="s3"]'));
  is('board: a locked database still moves cards', (await calls(p, 'setCells')).length, 1);
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
  const timedTargetVisible = (await dayCell(p, target2).count()) > 0;
  truthy('calendar: required timed-row drag target is visible', timedTargetVisible);
  if (timedTargetVisible) {
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
  } else truthy('calendar: required range-drag target is visible', false);

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

/* ------------------------------------------------------------- timeline */
tests.timeline = async (b) => {
  let p = await open(b, 'view=timeline&zoom=day', { width: 1400, height: 800 });
  const t0 = todayIso();
  is('timeline: a bar per dated row (17 dated of 20)', await p.locator('[data-bar]').count(), 18);
  is('timeline: a today line', await p.locator('[data-today-line]').count(), 1);
  is('timeline: one dependency arrow', await p.locator('[data-dep]').count(), 1);
  is('timeline: sub-items are listed (Demo script under Prepare demo)', await p.locator('[data-title-of="r12"]').count(), 1);
  await shot(p, 'timeline-day');

  const mouseDrag = async (loc, dx, where = 'center') => {
    const bb = await loc.boundingBox();
    const sx = where === 'start' ? bb.x + 3 : where === 'end' ? bb.x + bb.width - 3 : bb.x + bb.width / 2;
    const sy = bb.y + bb.height / 2;
    await p.mouse.move(sx, sy);
    await p.mouse.down();
    await p.mouse.move(sx + dx / 2, sy, { steps: 5 });
    await p.mouse.move(sx + dx, sy, { steps: 5 });
    await p.mouse.up();
    await p.waitForTimeout(200);
  };
  const bar = (id) => p.locator(`[data-bar="${id}"]`);
  // Move r1 (range) by +2 days (44px per day at day zoom).
  const r1 = await p.evaluate(() => window.__h.row('r1').values.due);
  await clear(p);
  await mouseDrag(bar('r1'), 88);
  let sc = await calls(p, 'setCells');
  is('timeline: dragging a bar moves both dates', sc[0] && sc[0].args, ['r1', { due: { start: addDays(r1.start, 2), end: addDays(r1.end, 2) } }]);
  is('timeline: the bar follows', [await bar('r1').getAttribute('data-start'), await bar('r1').getAttribute('data-end')], [addDays(r1.start, 2), addDays(r1.end, 2)]);

  // Resize the end of r4 by +1 day and the start of r4 by -1 day.
  const r4 = await p.evaluate(() => window.__h.row('r4').values.due);
  await clear(p);
  await mouseDrag(p.locator('[data-bar="r4"] [data-handle="end"]'), 44, 'center');
  sc = await calls(p, 'setCells');
  is('timeline: dragging the right edge moves the end', sc[0] && sc[0].args, ['r4', { due: { start: r4.start, end: addDays(r4.end, 1) } }]);
  await clear(p);
  await mouseDrag(p.locator('[data-bar="r4"] [data-handle="start"]'), -44, 'center');
  sc = await calls(p, 'setCells');
  is('timeline: dragging the left edge moves the start', sc[0] && sc[0].args, ['r4', { due: { start: addDays(r4.start, -1), end: addDays(r4.end, 1) } }]);

  // A click opens, an arrow key moves.
  await clear(p);
  await bar('r7').click();
  is('timeline: clicking a bar opens the row', (await calls(p, 'openRow')).map((c) => c.args[0]), ['r7']);
  await clear(p);
  await bar('r7').focus();
  await p.keyboard.press('ArrowRight');
  await p.waitForTimeout(100);
  sc = await calls(p, 'setCells');
  is('timeline: ArrowRight moves a bar by a day', sc[0] && sc[0].args, ['r7', { due: { start: addDays(t0, 1) } }]);

  // Collapse the sub-items of "Prepare demo".
  await p.getByLabel('Collapse sub-items').click();
  is('timeline: collapsing hides sub-items', await p.locator('[data-title-of="r12"]').count(), 0);
  await p.getByLabel('Expand sub-items').click();
  is('timeline: expanding shows them again', await p.locator('[data-title-of="r12"]').count(), 1);

  // Dependencies follow a dragged bar (arrow path changes live).
  const d0 = await p.locator('[data-dep]').getAttribute('d');
  const bb = await bar('r4').boundingBox();
  await p.mouse.move(bb.x + bb.width / 2, bb.y + bb.height / 2);
  await p.mouse.down();
  await p.mouse.move(bb.x + bb.width / 2 + 100, bb.y + bb.height / 2, { steps: 6 });
  truthy('timeline: the arrow moves while dragging', (await p.locator('[data-dep]').getAttribute('d')) !== d0);
  await p.mouse.up();

  // Unscheduled rows: Set dates.
  await clear(p);
  is('timeline: two rows without dates in the footer', await p.locator('[data-unscheduled]').count(), 2);
  await p.locator('[data-unscheduled="r10"]').getByRole('button', { name: 'Set dates' }).click();
  await p.getByLabel('End date', { exact: true }).fill(addDays(t0, 3));
  await p.getByLabel('Start date', { exact: true }).fill(t0);
  await p.getByRole('button', { name: 'Save' }).click();
  await p.waitForTimeout(200);
  sc = await calls(p, 'setCells');
  is('timeline: Set dates saves a range', sc[0] && sc[0].args, ['r10', { due: { start: t0, end: addDays(t0, 3) } }]);
  is('timeline: the row gets a bar', await bar('r10').count(), 1);

  // Zoom.
  await clear(p);
  for (const z of ['Month', 'Quarter', 'Year']) {
    await p.getByRole('button', { name: new RegExp('^' + z + '$', 'i') }).click();
    truthy(`timeline: zoom ${z} saves timelineZoom`, (await calls(p, 'updateView')).pop().args[0].timelineZoom === z.toLowerCase());
    await p.waitForTimeout(250);
    if (z === 'Month') await shot(p, 'timeline-month');
  }
  await p.close();

  // Grouped timeline + end property.
  p = await open(b, 'view=timeline&grouped=1&zoom=week', { width: 1400, height: 800 });
  is('timeline grouped: group headers', await p.locator('button[aria-expanded]').filter({ hasText: /Not started|In progress|Done/ }).count() >= 3, true);
  await shot(p, 'timeline-grouped');
  await clear(p);
  await p.locator('button[aria-expanded]').filter({ hasText: 'Done' }).first().click();
  is('timeline grouped: collapsing a group saves it', (await calls(p, 'updateView'))[0].args[0].group.collapsed, ['s3']);
  await p.close();

  // No date property.
  p = await open(b, 'view=timeline&noprop=1');
  truthy('timeline: prompt when no date property', await p.getByText('Choose a date property for the timeline').isVisible());
  await p.close();

  // Many rows, empty, dark + RTL.
  p = await open(b, 'view=timeline&data=many&theme=sunrise-dark&rtl=1', { width: 1400, height: 800 });
  const manyBars = p.locator('[data-bar]');
  const manyScroller = p.locator('[data-scroller]');
  const visitedBars = new Set();
  const collectBars = async () => {
    const ids = await manyBars.evaluateAll((bars) => bars.map((bar) => bar.getAttribute('data-bar')));
    for (const id of ids) visitedBars.add(id);
    return ids.length;
  };
  const firstBarCount = await collectBars();
  truthy('timeline: the initial bar window is bounded', firstBarCount > 0 && firstBarCount < 100);
  const height = await manyScroller.evaluate((el) => el.clientHeight);
  let tail = false;
  for (let step = 1; height > 0 && step <= 40; step++) {
    const atEnd = await manyScroller.evaluate((el, offset) => {
      el.scrollTop = offset;
      return el.scrollTop + el.clientHeight >= el.scrollHeight - 1;
    }, step * Math.floor(height / 2));
    await p.evaluate(() => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve))));
    const rendered = await collectBars();
    if (rendered <= 0 || rendered >= 100) break;
    if (atEnd) { tail = true; break; }
  }
  truthy('timeline: bounded windows visit 200+ distinct bars from top to bottom', tail && visitedBars.size > 200);
  await shot(p, 'timeline-dark-rtl');
  await p.close();
  p = await open(b, 'view=timeline&data=empty');
  truthy('timeline: empty state', await p.getByText('No rows yet').isVisible());
  await p.close();
};

/* ---------------------------------------------------------------- chart */
tests.chart = async (b) => {
  let p = await open(b, 'view=chart');
  await p.waitForSelector('[data-chart]');
  is('chart: a bar per status (4 groups)', await p.locator('[data-bar]').count(), 4);
  const label = await p.locator('svg[data-chart]').getAttribute('aria-label');
  truthy('chart: the svg has role img and a summary', (await p.locator('svg[data-chart]').getAttribute('role')) === 'img' && /Bar chart of 20 rows in 4 groups/.test(label) && /Not started: 11/.test(label));
  const hs = await p.locator('[data-bar]').evaluateAll((els) => els.map((e) => Math.round(+e.getAttribute('height'))));
  truthy('chart: the tallest bar is the biggest group (11 vs 4)', hs[0] > hs[1] * 2.5 && hs[1] === hs[2]);
  // tooltip
  const r0 = await p.locator('[data-bar]').first().boundingBox();
  await p.mouse.move(r0.x + r0.width / 2, r0.y + 20);
  await p.waitForTimeout(100);
  truthy('chart: hovering a bar shows a tooltip with the value', /Not started[\s\S]*11/.test((await p.locator('[data-tooltip]').textContent()) || ''));
  await p.mouse.move(5, 500);
  is('chart: the tooltip goes away', await p.locator('[data-tooltip]').count(), 0);
  // table toggle
  await p.getByRole('button', { name: 'Show as table' }).click();
  is('chart: the data table lists every group', await p.locator('[data-chart-table] tbody tr').count(), 4);
  await p.getByRole('button', { name: 'Show chart' }).click();
  // customize: x axis, kind, group
  await clear(p);
  await p.getByRole('button', { name: 'Customize' }).click();
  await p.getByLabel('X axis', { exact: true }).selectOption('prio');
  await p.waitForTimeout(300);
  is('chart: choosing the X axis saves it', (await calls(p, 'updateView')).pop().args[0].chart.xProp, 'prio');
  is('chart: the chart is asked again with the new axis', (await calls(p, 'chartData')).pop().args[0].xProp, 'prio');
  is('chart: bars follow (4 priorities incl. none)', await p.locator('[data-bar]').count(), 4);
  await p.getByLabel('Group by', { exact: true }).selectOption({ label: 'Status' });
  await p.waitForTimeout(300);
  is('chart: grouping splits series (4 x 3 bars)', await p.locator('[data-bar]').count(), 12);
  await p.getByLabel('Stacked').check();
  await p.waitForTimeout(300);
  is('chart: stacked saves the flag', (await calls(p, 'updateView')).pop().args[0].chart.stacked, true);
  await p.getByLabel('Horizontal').check();
  await p.waitForTimeout(200);
  is('chart: horizontal saves the flag', (await calls(p, 'updateView')).pop().args[0].chart.horizontal, true);
  await shot(p, 'chart-config');
  await p.getByRole('button', { name: 'Line' }).click();
  await p.waitForTimeout(300);
  is('chart: line chart draws a polyline per series', await p.locator('svg[data-chart="line"] polyline').count(), 3);
  await p.getByRole('button', { name: 'Donut' }).click();
  await p.waitForTimeout(300);
  truthy('chart: donut draws slices', (await p.locator('[data-slice]').count()) >= 3);
  await p.getByRole('button', { name: 'Number' }).click();
  await p.waitForTimeout(300);
  is('chart: number tile shows the count', (await p.locator('[data-number]').textContent()), '20');
  await p.close();

  p = await open(b, 'view=chart&ck=bar&cx=prio&cg=status&horiz&stack');
  await shot(p, 'chart-stacked');
  await p.close();
  p = await open(b, 'view=chart&ck=line&cx=due&cg=prio&theme=sunrise-dark');
  const rawLineRows = await p.evaluate(() => window.__h.rows());
  const actualLine = await p.evaluate(() => {
    const svg = document.querySelector('svg[data-chart="line"]');
    if (!svg) return null;
    return {
      months: [...svg.children].filter((el) => el.tagName.toLowerCase() === 'text').map((el) => el.textContent),
      legend: [...svg.parentElement.querySelectorAll('[aria-label="Legend"] li')].map((el) => el.textContent.trim()),
      series: [...svg.querySelectorAll('[data-series]')].map((el) => ({
        index: Number(el.getAttribute('data-series')),
        polylines: el.querySelectorAll('polyline').length,
        points: el.querySelectorAll('[data-point]').length,
        labels: [...el.querySelectorAll('[data-point-hit]')].map((point) => point.getAttribute('aria-label')),
      })),
    };
  });
  truthy('chart: exact months, series and ordered point values follow raw fixture rows', assertFixtureLine(rawLineRows, actualLine));
  await shot(p, 'chart-line-dark');
  await p.close();
  p = await open(b, 'view=chart&ck=donut&cx=prio&rtl=1');
  is('chart: donut slices (4 options)', await p.locator('[data-slice]').count(), 4);
  is('chart: donut center total', await p.locator('[data-donut-total]').textContent(), '20');
  is('chart: donut legend rows', await p.locator('[data-legend]').count(), 4);
  await shot(p, 'chart-donut-rtl');
  await p.close();
  p = await open(b, 'view=chart&ck=number&calc=sum&cy=est');
  is('chart: sum of estimates', await p.locator('[data-number]').textContent(), '26');
  await shot(p, 'chart-number');
  await p.close();
  p = await open(b, 'view=chart&chart=error');
  truthy('chart: an error is shown with a retry button', (await p.getByRole('alert').textContent()).includes('could not be calculated') && (await p.getByRole('button', { name: 'Try again' }).isVisible()));
  await shot(p, 'chart-error');
  await p.close();
  p = await open(b, 'view=chart&data=empty');
  truthy('chart: empty state', await p.getByText('Nothing to chart yet').isVisible());
  await p.close();
  p = await open(b, 'view=chart&cx=');
  truthy('chart: no X axis asks to choose one', await p.getByText('Choose what to chart').isVisible());
  await p.close();
};

/* ------------------------------------------------------------ list, gallery */
tests.list = async (b) => {
  let p = await open(b, 'view=list&check=1');
  is('list: a line per row', await p.locator('[data-list-row]').count(), 20);
  await shot(p, 'list');
  await clear(p);
  await p.getByRole('checkbox', { name: 'Done: Write release notes' }).click();
  let sc = await calls(p, 'setCells');
  is('list: a checkbox toggles in place (checked -> unchecked)', sc[0] && sc[0].args, ['r3', { done: false }]);
  is('list: toggling does not open the row', (await calls(p, 'openRow')).length, 0);
  await clear(p);
  await p.getByRole('checkbox', { name: 'Done: Review pull requests' }).click();
  sc = await calls(p, 'setCells');
  is('list: unchecked -> checked', sc[0] && sc[0].args, ['r7', { done: true }]);
  await clear(p);
  await p.locator('[data-list-row="r5"]').click();
  is('list: click opens', (await calls(p, 'openRow')).map((c) => c.args[0]), ['r5']);
  await clear(p);
  await p.locator('[data-list-row="r1"]').focus();
  await p.keyboard.press('ArrowDown');
  is('list: ArrowDown moves focus', await p.evaluate(() => document.activeElement.dataset.listRow), 'r2');
  await p.keyboard.press('Enter');
  is('list: Enter opens the focused row', (await calls(p, 'openRow')).map((c) => c.args[0]), ['r2']);
  await clear(p);
  await p.getByRole('button', { name: 'New' }).click();
  await p.getByLabel('Title of the new row').fill('Listed');
  await p.keyboard.press('Enter');
  await p.waitForTimeout(150);
  is('list: + New creates a row', (await calls(p, 'createRow'))[0].args, [{}, 'Listed']);
  await p.close();

  p = await open(b, 'view=list&grouped=1&theme=sunrise-dark&rtl=1');
  is('list grouped: three status groups and none', await p.locator('[data-group]').count(), 4);
  await clear(p);
  await p.locator('[data-group="s3"]').getByRole('button', { name: 'New' }).click();
  await p.getByLabel('Title of the new row').fill('Finished');
  await p.keyboard.press('Enter');
  await p.waitForTimeout(150);
  is('list grouped: + New in a group sets its value', (await calls(p, 'createRow'))[0].args, [{ status: 's3' }, 'Finished']);
  await p.keyboard.press('Escape');
  await clear(p);
  await p.locator('[data-group="s2"] > button').click();
  is('list grouped: a group collapses', (await calls(p, 'updateView'))[0].args[0].group.collapsed, ['s2']);
  await shot(p, 'list-dark-rtl');
  await p.close();
  p = await open(b, 'view=list&data=empty');
  truthy('list: empty state', await p.getByText('No rows yet').isVisible());
  await p.close();
};

tests.gallery = async (b) => {
  let p = await open(b, 'view=gallery');
  is('gallery: a card per row', await p.locator('[data-card]').count(), 20);
  is('gallery: covers come from the files property', await p.locator('[data-cover]').count(), 4);
  truthy('gallery: cards without a picture get a placeholder', (await p.locator('[data-cover-empty]').count()) === 16);
  const w0 = (await p.locator('[data-card]').first().boundingBox()).width;
  await clear(p);
  await p.locator('[data-card="r4"]').click();
  is('gallery: click opens', (await calls(p, 'openRow')).map((c) => c.args[0]), ['r4']);
  await p.locator('[data-card="r5"]').focus();
  await clear(p);
  await p.keyboard.press('Enter');
  is('gallery: Enter opens', (await calls(p, 'openRow')).map((c) => c.args[0]), ['r5']);
  await clear(p);
  await p.locator('[data-new-card]').click();
  await p.getByLabel('Title of the new row').fill('Fresh');
  await p.keyboard.press('Enter');
  await p.waitForTimeout(150);
  is('gallery: the + New card creates a row', (await calls(p, 'createRow'))[0].args, [{}, 'Fresh']);
  await shot(p, 'gallery');
  await p.close();
  p = await open(b, 'view=gallery&cover=none');
  is('gallery: cover "none" shows no pictures', (await p.locator('[data-cover]').count()) + (await p.locator('[data-cover-empty]').count()), 0);
  await p.close();
  p = await open(b, 'view=gallery&grouped=1&theme=sunrise-dark&rtl=1');
  is('gallery grouped: sections', await p.locator('[data-group]').count(), 4);
  await shot(p, 'gallery-dark-rtl');
  await p.close();
  p = await open(b, 'view=gallery', { width: 500, height: 800 });
  const ys = await p.locator('[data-card]').evaluateAll((els) => els.map((e) => Math.round(e.getBoundingClientRect().top)));
  truthy('gallery: a narrow window has fewer columns', ys.filter((y) => y === ys[0]).length <= 2 && w0 < 260);
  await p.close();
  p = await open(b, 'view=gallery&data=empty');
  truthy('gallery: empty state', await p.getByText('No rows yet').isVisible());
  await p.close();
};

(async () => {
  const names = Object.keys(tests);
  const selected = checks.selection(only ? [only] : names, names);
  if (!selected.length) { checks.finish(); return; }
  const { chromium } = require('playwright');
  const b = await chromium.launch();
  try {
    for (const name of selected) {
      console.log('--', name);
      try { await tests[name](b); }
      catch (e) { checks.check(name + ' threw', false, e.message.split('\n').slice(0, 12).join(' | ')); }
    }
  } finally {
    await b.close();
    checks.finish();
  }
})().catch((error) => { checks.check('database views runner', false, error.message); checks.finish(); });
