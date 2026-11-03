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
