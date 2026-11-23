// Synthetic full-app mock. Run with the local mock server, as in p7-window-check.cjs.
const assert = require('node:assert/strict');
const { chromium } = require(process.env.NOTAYA_PLAYWRIGHT_MODULE || 'playwright');
const base = new URL(process.argv[2] || 'http://127.0.0.1:1421');
assert(['localhost', '127.0.0.1', '[::1]'].includes(base.hostname), 'Use a local throwaway mock');
const counts = process.argv.slice(3).map(Number);
if (!counts.length) counts.push(5_000, 20_000, 200_000);
assert(counts.every((n) => [5_000, 20_000, 200_000].includes(n)), 'Invalid fixture size');

const configs = {
  List: { item: '[data-list-row]', scroll: '[aria-label="List"]', max: 80 },
  Gallery: { item: '[data-card]', scroll: '[aria-label="Gallery"]', max: 100 },
  Board: { item: '[data-card]', scroll: '[data-board-scroll]', max: 120 },
  Timeline: { item: '[data-bar]', scroll: '[data-scroller]', max: 100 },
};
const mark = (page) => page.evaluate(() => window.__p8Windows.length);
const paint = (page) => page.evaluate(() => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve))));

async function inspectReads(page, since, count, phase) {
  const requests = await page.evaluate((i) => window.__p8Windows.slice(i), since);
  const queries = requests.filter((request) => request.command === 'query_view');
  const reads = requests.filter((request) => request.command === 'view_rows');
  assert(queries.every((query) => query.windowed), `${phase} requested a full result`);
  if (phase === 'tail' || phase === 'time') assert.equal(queries.length, 0, `Scrolling re-ran the query: ${JSON.stringify(requests)}`);
  if (phase !== 'time') assert(reads.length > 0, `${phase} never read row values`);
  if (phase === 'open') assert(queries.length > 0, 'View never requested metadata');
  let sum = 0;
  for (const { ranges } of reads) {
    assert(ranges.length > 0 && ranges.length <= 256);
    const length = ranges.reduce((total, { start, count: length }) => {
      assert(Number.isInteger(start) && start >= 0 && Number.isInteger(length) && length > 0 && start + length <= count);
      return total + length;
    }, 0);
    assert(length <= 256, `Read ${length} rows in one request`);
    sum += length;
  }
  assert(sum <= 1024 && sum < count / 4, `${phase} read ${sum} values at size ${count}`);
  return sum;
}

async function bounded(page, config) {
  const items = await page.locator(config.item).count();
  const elements = await page.locator('body *').count();
  assert(items > 0 && items < config.max, `Mounted ${items} items, expected 1..${config.max - 1}`);
  assert(elements < 9_000, `Mounted ${elements} elements`);
  return { items, elements };
}

async function drag(page, source, target) {
  await source.scrollIntoViewIfNeeded();
  await target.scrollIntoViewIfNeeded();
  const a = await source.boundingBox(), b = await target.boundingBox();
  assert(a && b, 'Both drag endpoints are visible');
  const x = a.x + a.width / 2, y = a.y + Math.min(24, a.height / 2);
  await page.mouse.move(x, y);
  await page.mouse.down();
  await page.mouse.move(x + 12, y + 10, { steps: 4 });
  await page.mouse.move(b.x + b.width / 2, b.y + 30, { steps: 16 });
  await page.mouse.up();
}

async function editRow(page, rowId, titleInput, original) {
  const updated = `${original} edited`;
  const saved = (title) => page.waitForFunction(async ({ id, title }) => {
    const { invoke } = await import('/dev-mock/core.ts');
    return (await invoke('get_row', { rowId: id })).row.title === title;
  }, { id: rowId, title }, { polling: 200 });
  await titleInput.fill(updated);
  await saved(updated);
  await titleInput.fill(original);
  await saved(original);
}

async function timelineTimeWindow(page, scroller, total) {
  const ticks = page.locator('[data-timeline-tick]');
  const grid = page.locator('[data-timeline-gridline]');
  const start = await scroller.evaluate((el) => el.scrollLeft);
  const before = await ticks.allTextContents();
  assert(before.length > 0 && before.length < 100);
  assert((await grid.count()) > 0 && (await grid.count()) < 100);
  const since = await mark(page);
  await scroller.evaluate((el) => { el.scrollLeft = el.scrollWidth; });
  await page.waitForFunction(({ before }) => {
    const labels = [...document.querySelectorAll('[data-timeline-tick]')].map((el) => el.textContent);
    return labels.length > 0 && labels.join('|') !== before.join('|');
  }, { before });
  assert((await ticks.count()) < 100 && (await grid.count()) < 100);
  assert((await page.locator('[data-bar]').count()) < 100);
  assert((await scroller.evaluate((el) => el.scrollLeft)) > start + 300, 'Timeline has a distinct far time window');
  await inspectReads(page, since, total, 'time');
  await scroller.evaluate((el, position) => { el.scrollLeft = position; }, start);
}

async function waitForBoardRefresh(page, since) {
  // Read from the refreshed generation and let any coalesced refresh from the
  // write or an invalidated row request finish before attributing queries to scroll.
  const settled = await page.evaluateHandle(() => ({ query: null, at: 0 }));
  try {
    await page.waitForFunction(({ since, settled }) => {
      const trace = window.__p8Windows.slice(since);
      const query = trace.findLast((request) => request.command === 'query_view');
      if (!query?.generation || !trace.some((request) =>
        request.command === 'view_rows' && request.generation === query.generation)) return false;
      if (settled.query !== query) {
        settled.query = query;
        settled.at = performance.now();
      }
      // 120ms write debounce, 60ms query debounce, and the mock's 30ms IPC delay.
      return performance.now() - settled.at >= 300;
    }, { since, settled });
  } finally {
    await settled.dispose();
  }
}

async function reorderUnloadedBoardCard(page, scroll, count) {
  const column = scroll.locator('..');
  const readOrder = () => page.evaluate(async () => {
    const { invoke } = await import('/dev-mock/core.ts');
    const tree = await invoke('list_notes');
    const { id } = tree.databases.find((db) => db.title === 'Tasks');
    const db = await invoke('get_database', { id });
    const { id: viewId } = db.views.find((view) => view.kind === 'board');
    const { rowIds, groups } = await invoke('query_view', { request: { databaseId: id, viewId }, windowed: true });
    return { rowIds, members: groups[1].rows };
  });
  const before = await readOrder();
  assert.equal(before.rowIds.length, count);
  await scroll.evaluate((el) => { el.scrollTop = 0; });
  const sourceId = before.members[0];
  const source = column.locator(`[data-card="${sourceId}"]`);
  await source.waitFor();
  const mounted = await column.locator('[data-card]').evaluateAll((cards) => cards.map((card) => card.dataset.card));
  const { x, y, width, height } = await source.boundingBox();
  const start = { x: x + width / 2, y: y + height / 2 };
  const since = await mark(page);
  await page.mouse.move(start.x, start.y);
  await page.mouse.down();
  await page.mouse.move(start.x + 8, start.y + 10, { steps: 3 });
  await page.locator('[data-board-overlay]').waitFor();
  await scroll.evaluate((el) => { el.scrollTop = 22_000; });
  await source.waitFor({ state: 'detached' });
  await page.locator('[data-board-overlay]').waitFor();
  await paint(page);
  const destination = await page.waitForFunction(() => {
    const scroller = document.querySelector('[data-board-scroll]');
    const viewport = scroller.getBoundingClientRect();
    const cards = [...scroller.querySelectorAll('[data-card]')];
    const card = cards.find((candidate) => {
      const rect = candidate.getBoundingClientRect();
      return rect.top > viewport.top + 160 && rect.bottom < viewport.bottom - 100;
    });
    if (!card) return null;
    const rect = card.getBoundingClientRect();
    return { id: card.dataset.card, x: rect.x + rect.width / 2, y: rect.y + rect.height / 2 };
  });
  const target = await destination.jsonValue();
  assert(!mounted.includes(target.id));
  await page.mouse.move(target.x, target.y);
  // Read the selected card before release. The closest corners can select a
  // neighboring card when its height differs from the one under the pointer.
  const cellId = await column.getAttribute('data-cell');
  const selection = await page.waitForFunction(({ cellId, initial }) => {
    const prefix = `over droppable area ${cellId}@`;
    for (const liveRegion of document.querySelectorAll('[id^="DndLiveRegion"]')) {
      const text = liveRegion.textContent;
      const start = text.lastIndexOf(prefix);
      if (start < 0 || !text.endsWith('.')) continue;
      const id = text.slice(start + prefix.length, -1);
      if (initial.includes(id)) continue;
      const card = document.querySelector(`[data-card="${CSS.escape(id)}"]`);
      const viewport = card?.closest('[data-board-scroll]')?.getBoundingClientRect();
      const rect = card?.getBoundingClientRect();
      if (rect && viewport && rect.top >= viewport.top && rect.bottom <= viewport.bottom) return id;
    }
    return false;
  }, { cellId, initial: mounted });
  const selectedId = await selection.jsonValue();
  await page.mouse.up();
  await waitForBoardRefresh(page, since);
  const after = await readOrder();
  const index = before.members.indexOf(selectedId);
  assert(index > 100, `Expected a distant target, found ${index}`);
  const members = new Set(before.members);
  const reordered = before.members.slice(1);
  reordered.splice(index, 0, sourceId);
  let offset = 0;
  assert.deepEqual(after.rowIds, before.rowIds.map((id) => members.has(id) ? reordered[offset++] : id));
  console.log(`${count} Board: overlay retained and full order matched after same-column drag to unloaded index ${index}`);
}

async function checkView(page, name, count) {
  const config = configs[name];
  const initial = await mark(page);
  await page.getByRole('tab', { name, exact: true }).click();
  const items = page.locator(config.item);
  await items.first().waitFor();
  await paint(page);
  const initialDom = await bounded(page, config);
  const initialReads = await inspectReads(page, initial, count, 'open');
  if (name !== 'Timeline') await page.getByText('Website relaunch', { exact: true }).first().waitFor();
  let scroll = page.locator(config.scroll).first();
  const tailTitle = `Task number ${count - 1}`;
  const tailText = page.getByText(tailTitle, { exact: true }).first();

  if (name === 'List' && count === 5_000) {
    await items.first().focus();
    for (let i = 1; i <= 35; i++) {
      const previous = await page.locator(':focus').getAttribute('data-list-row');
      await page.keyboard.press('ArrowDown');
      await page.waitForFunction((id) => {
        const focused = document.activeElement?.getAttribute('data-list-row');
        return focused && focused !== id;
      }, previous);
    }
  }
  if (name === 'Board') {
    const columns = page.locator('[data-column]');
    const cells = page.locator('[data-cell]');
    const counts = await columns.locator('[data-count]').allTextContents();
    assert.deepEqual(counts.map((value) => Number(value.replace(/[^0-9]/g, ''))), [0, count - 18, 6, 6, 6]);
    const estimates = [1, 2, 3, 5, 8, 13];
    let sum = 0;
    for (let i = 0; i < count; i++) if (i >= 25 || i % 4 === 0) sum += estimates[i % estimates.length];
    assert.equal(Number((await columns.nth(1).locator('[data-calc]').textContent()).replace(/[^0-9]/g, '')), sum);
    const source = cells.nth(1).locator(config.item).first();
    const target = cells.nth(2);
    const rowId = await source.getAttribute('data-card');
    const beforeWrite = await mark(page);
    await drag(page, source, target);
    await waitForBoardRefresh(page, beforeWrite);
    await target.locator(`[data-card="${rowId}"]`).waitFor();
    await page.locator('[data-column] [data-count]').filter({ hasText: new RegExp(`^${count - 19}$`) }).waitFor();
    scroll = cells.nth(1).locator(config.scroll);
  }
  if (name === 'Timeline') await timelineTimeWindow(page, scroll, count);

  await paint(page);
  const sinceTail = await mark(page);
  const top = await scroll.evaluate((el) => el.scrollTop);
  await scroll.evaluate((el) => { el.scrollTop = el.scrollHeight; });
  await tailText.waitFor();
  assert((await scroll.evaluate((el) => el.scrollTop)) > top + 1000, 'Tail scroll did not move');
  const tailDom = await bounded(page, config);
  const tailReads = await inspectReads(page, sinceTail, count, 'tail');
  if (name === 'Board' && count === 200_000) await reorderUnloadedBoardCard(page, scroll, count);

  // Find must bring a row outside the current value slice into the view.
  const findTitle = name === 'Board' ? 'Write the API' : 'Design the landing page';
  const sinceFind = await mark(page);
  await page.getByRole('button', { name: 'Search rows', exact: true }).click();
  const search = page.getByPlaceholder('Search…');
  await search.fill(findTitle);
  const found = name === 'Timeline'
    ? page.locator('[data-title-of]').getByText(findTitle, { exact: true }).first()
    : items.filter({ hasText: findTitle }).first();
  await found.waitFor();
  await page.waitForFunction((selector) => document.querySelectorAll(selector).length === 1, config.item);
  const trace = await page.evaluate((i) => window.__p8Windows.slice(i), sinceFind);
  assert(trace.some((request) => request.command === 'query_view' && request.find === findTitle));
  await inspectReads(page, sinceFind, count, 'find');
  const rowId = await items.first().getAttribute(name === 'Timeline' ? 'data-bar' : name === 'List' ? 'data-list-row' : 'data-card');
  if (name === 'List' || name === 'Gallery') await found.press('Enter');
  else await found.click();
  const peek = page.getByRole('complementary', { name: 'Row', exact: true });
  const title = peek.getByPlaceholder('Untitled', { exact: true });
  await title.waitFor();
  assert.equal(await title.inputValue(), findTitle);
  if (name === 'List') await editRow(page, rowId, title, findTitle);
  await peek.getByRole('button', { name: 'Close', exact: true }).click();
  await search.press('Escape');
  console.log(`${count} ${name}: open ${JSON.stringify(initialDom)} / ${initialReads} row values; tail ${JSON.stringify(tailDom)} / ${tailReads} row values; Find and open passed`);
}

async function openTasks(page, count, configure) {
  const url = new URL(base);
  url.searchParams.set('db', '1');
  url.searchParams.set('p8', '1');
  url.searchParams.set('rows', String(count));
  await page.goto(url.href);
  const fixture = configure && await configure();
  await page.getByRole('treeitem', { name: 'Tasks', exact: true }).click();
  await page.locator('tr[data-row-id]').first().waitFor();
  return fixture;
}

async function configureFixture(page, fixture) {
  return page.evaluate(async ({ kind, count, manyUndated }) => {
    const { invoke } = await import('/dev-mock/core.ts');
    const { databaseCommand } = await import('/dev-mock/database.ts');
    const tree = await invoke('list_notes');
    const { id } = tree.databases.find((db) => db.title === 'Tasks');
    const db = await invoke('get_database', { id });
    const view = db.views.find((v) => v.kind === (kind === 'timeline' ? 'timeline' : 'board'));
    const idProp = db.props.find((p) => p.kind === 'id').id;
    const status = db.props.find((p) => p.kind === 'status');
    const group = (prop) => ({ prop, sort: 'manual', dateBy: '', numberStep: 0, hideEmpty: false, hidden: [], collapsed: [] });
    view.group = group(kind === 'columns' ? idProp : status.id);
    if (kind === 'lanes') view.subGroup = group(idProp);
    if (kind === 'columns') view.calcs = [];
    const endProp = 'p8-test-end';
    if (kind === 'timeline') db.props.push({ ...db.props.find((p) => p.id === view.dateProp), id: endProp, name: 'Finish' });
    await invoke('save_database', { databaseJson: JSON.stringify(db) });
    const request = { databaseId: id, viewId: view.id, view };
    const query = (find = '') => databaseCommand('query_view', { request, windowed: true, find }, {});
    const initial = query();
    if (kind === 'columns') return { groups: initial.groups.map(({ key }) => key) };
    if (kind === 'lanes') return { lanes: [...new Set(initial.groups.flatMap((g) => g.subs.map(({ key }) => key)))] };

    const patchDate = (rowId, date, prop = view.dateProp) => databaseCommand('set_cells', { rowId, patch: JSON.stringify({ [prop]: date }) }, {});
    for (const rowId of initial.timeline.unscheduledIds) patchDate(rowId, { start: '2026-11-23' });
    const future = initial.rowIds[count - 2];
    const tail = initial.rowIds[count - 1];
    patchDate(future, { start: '2099-12-31' });
    patchDate(tail, null);
    if (manyUndated) for (const rowId of initial.rowIds.slice(-1001, -2)) patchDate(rowId, null);
    const nearTitle = 'Design the landing page';
    const near = query(nearTitle).timeline;
    patchDate(initial.rowIds[0], { start: '2099-12-30', end: '2099-12-31' }, endProp);
    const withEnd = databaseCommand('query_view', { request: { ...request, view: { ...view, endProp } }, windowed: true, find: nearTitle }, {}).timeline;
    return { future, tail, near, withEnd, undated: query(`Task number ${count - 1}`).timeline };
  }, fixture);
}

async function highCardinalityBoard(browser, kind, count) {
  const page = await browser.newPage({ viewport: { width: 1400, height: 900 } });
  try {
    const fixture = await openTasks(page, count, () => configureFixture(page, { kind, count }));
    const keys = fixture.groups ?? fixture.lanes;
    assert.equal(keys.length, count);
    if (kind === 'columns') await page.evaluate(() => { document.documentElement.dir = 'rtl'; });
    await page.getByRole('tab', { name: 'Board', exact: true }).click();
    const board = page.getByRole('region', { name: 'Board', exact: true });
    const attr = kind === 'columns' ? 'data-column' : 'data-board-lane';
    const head = board.locator(`[${attr}="${keys[0]}"]`);
    const tail = board.locator(`[${attr}="${keys.at(-1)}"]`);
    await head.waitFor();
    await board.locator('[data-card]').first().waitFor();
    assert.equal(await tail.count(), 0);
    const measure = () => board.evaluate((el) => ({
      elements: el.querySelectorAll('*').length,
      columns: el.querySelectorAll('[data-column]').length,
      lanes: el.querySelectorAll('[data-board-lane]').length,
      cells: el.querySelectorAll('[data-cell]').length,
    }));
    const top = await measure();
    const since = await mark(page);
    await board.evaluate((el, axis) => {
      if (axis === 'columns') el.scrollLeft = -el.scrollWidth;
      else el.scrollTop = el.scrollHeight;
    }, kind);
    await tail.waitFor();
    await head.waitFor({ state: 'detached' });
    if (kind === 'lanes') await board.evaluate((el) => { el.scrollLeft = el.scrollWidth; });
    const tailCard = kind === 'columns' ? board.getByText(`Task number ${count - 1}`, { exact: true }) : tail.locator('[data-card]').first();
    await tailCard.waitFor();
    const end = await measure();
    for (const { elements, columns, lanes, cells } of [top, end]) {
      assert(columns > 0 && columns <= 12 && lanes <= 16 && cells > 0 && cells <= 80 && elements <= 2500,
        `Unbounded Board ${kind}: ${JSON.stringify({ elements, columns, lanes, cells })}`);
      if (kind === 'lanes') assert(lanes > 0);
    }
    await inspectReads(page, since, count, 'tail');
    console.log(`${count} Board ${kind}: top ${JSON.stringify(top)}; tail ${JSON.stringify(end)}${kind === 'columns' ? ' (RTL)' : ''}`);
  } finally {
    await page.close();
  }
}

async function timelineMetadata(browser, manyUndated = false) {
  const page = await browser.newPage({ viewport: { width: 1400, height: 900 } });
  try {
    const count = 5_000;
    const { future, tail, near, withEnd, undated } = await openTasks(page, count, () => configureFixture(page, { kind: 'timeline', count, manyUndated }));
    const farDay = Date.UTC(2099, 11, 31) / 86_400_000;
    assert(near.maxDay < farDay && near.minDay != null);
    assert.deepEqual(withEnd, { minDay: near.minDay, maxDay: farDay, unscheduledIds: [] });
    assert.deepEqual(undated, { minDay: null, maxDay: null, unscheduledIds: [tail] });
    await page.getByRole('tab', { name: 'Timeline', exact: true }).click();
    const timeline = page.getByRole('region', { name: 'Timeline', exact: true });
    const tracks = timeline.locator('[data-scroller]');
    const footer = timeline.locator('[data-unscheduled-box]');
    const toggle = footer.getByRole('button', { name: manyUndated ? 'No dates 1000' : 'No dates 1' });
    const last = footer.locator(`[data-unscheduled="${tail}"]`).getByRole('button', { name: 'Task number 4999' });
    await toggle.waitFor();
    const start = await tracks.evaluate((el) => ({ y: el.scrollTop, width: el.scrollWidth }));
    const today = Math.floor(Date.now() / 86_400_000);
    assert.equal(start.y, 0);
    assert(start.width >= (farDay - today) * 16, `Initial Timeline axis is only ${start.width}px wide`);
    assert.equal(await tracks.locator(`[data-line="${future}"]`).count(), 0);
    if (manyUndated) {
      assert.equal(await last.count(), 0);
      const footerScroll = footer.locator('.overflow-y-auto');
      await footerScroll.evaluate((el) => { el.scrollTop = el.scrollHeight; });
    }
    await last.waitFor();
    const mounted = await footer.locator('[data-unscheduled]').count();
    assert(mounted > 0 && mounted <= 12, `Mounted ${mounted} undated rows`);
    assert.equal(await tracks.evaluate((el) => el.scrollTop), 0);
    const since = await mark(page);
    await tracks.evaluate((el) => { el.scrollLeft = el.scrollWidth; });
    await tracks.locator('[data-timeline-tick]').getByText('December 2099', { exact: true }).waitFor();
    await inspectReads(page, since, count, 'time');
    await toggle.click();
    assert.equal(await toggle.getAttribute('aria-expanded'), 'false');
    assert.equal(await footer.locator('[data-unscheduled]').count(), 0);
    assert.equal(await tracks.evaluate((el) => el.scrollWidth), start.width);
    console.log(`5000 Timeline: initial axis ${start.width}px; ${manyUndated ? '1000' : '1'} undated, ${mounted} mounted; tail and collapse passed`);
  } finally {
    await page.close();
  }
}

async function timelineSubItems(browser, count, { undated = false, grouped = false } = {}) {
  const page = await browser.newPage({ viewport: { width: 1400, height: 900 } });
  const scenario = grouped ? 'grouped' : undated ? 'ungrouped, undated child' : 'ungrouped, dated child';
  try {
    const fixture = await openTasks(page, count, () => page.evaluate(async ({ grouped, undated }) => {
      const { invoke } = await import('/dev-mock/core.ts');
      const { databaseCommand } = await import('/dev-mock/database.ts');
      const { id } = (await invoke('list_notes')).databases.find((db) => db.title === 'Tasks');
      const db = await invoke('enable_sub_items', { id });
      const view = db.views.find((v) => v.kind === 'timeline');
      view.showSubItems = true;
      if (grouped) {
        const prop = db.props.find((p) => p.kind === 'status').id;
        view.group = { prop, sort: 'manual', dateBy: '', numberStep: 0, hideEmpty: true, hidden: [], collapsed: [] };
      }
      await invoke('save_database', { databaseJson: JSON.stringify(db) });
      const command = (name, args) => databaseCommand(name, args, {});
      const snapshot = command('query_view', { request: { databaseId: id, viewId: view.id, view }, windowed: true });
      const ids = grouped ? snapshot.groups[0].rows : snapshot.rowIds;
      if (ids.length <= 65) throw new Error('Child must start outside the top row window');
      const parent = ids[0], child = ids.at(-1), tail = ids.at(-2);
      const patch = (rowId, cells) => command('set_cells', { rowId, patch: JSON.stringify(cells) });
      if (undated) {
        // Keep the child outside the initial No dates footer window as well as the track window.
        for (const rowId of ids.slice(1, 65)) patch(rowId, { [view.dateProp]: null });
      }
      const date = undated ? null : { start: '2026-11-23' };
      patch(child, { [db.subParentProp]: [parent], [view.dateProp]: date });
      const parentRow = command('get_row', { rowId: parent }).row;
      const childRow = command('get_row', { rowId: child }).row;
      return {
        parent, child, tail, childTitle: childRow.title,
        childDate: childRow.values[view.dateProp] ?? null,
        parentLink: childRow.values[db.subParentProp], childLink: parentRow.values[db.subChildProp],
      };
    }, { grouped, undated }));
    assert.deepEqual(fixture.parentLink, [fixture.parent]);
    assert.deepEqual(fixture.childLink, [fixture.child]);
    assert.equal(fixture.childDate?.start ?? null, undated ? null : '2026-11-23');

    await page.getByRole('tab', { name: 'Timeline', exact: true }).click();
    const timeline = page.getByRole('region', { name: 'Timeline', exact: true });
    const tracks = timeline.locator('[data-scroller]');
    const parent = tracks.locator(`[data-line="${fixture.parent}"]`);
    const child = tracks.locator(`[data-line="${fixture.child}"]`);
    const childTitle = child.getByRole('button', { name: fixture.childTitle, exact: true });
    const collapse = parent.getByRole('button', { name: 'Collapse sub-items', exact: true });
    const expand = parent.getByRole('button', { name: 'Expand sub-items', exact: true });
    const checkAtTop = async () => {
      await collapse.waitFor();
      await childTitle.waitFor();
      assert.equal(await tracks.evaluate((el) => el.scrollTop), 0);
      assert.equal(await parent.getAttribute('data-track-index'), grouped ? '1' : '0');
      assert.equal(await child.getAttribute('data-track-index'), grouped ? '2' : '1');
      assert.equal(await timeline.locator(`[data-unscheduled="${fixture.child}"]`).count(), 0);
    };
    await checkAtTop();
    await collapse.click();
    await child.waitFor({ state: 'detached' });
    await expand.click();
    await checkAtTop();

    if (!grouped) {
      const since = await mark(page);
      await tracks.evaluate((el) => { el.scrollTop = el.scrollHeight; });
      await tracks.locator(`[data-title-of="${fixture.tail}"]`).waitFor();
      await child.waitFor({ state: 'detached' });
      await paint(page);
      await inspectReads(page, since, count, 'tail');
    }
    console.log(`${count} Timeline sub-items (${scenario}): top placement and collapse/expand passed${grouped ? '' : '; tail scroll issued zero query_view calls'}`);
  } catch (error) {
    throw new Error(`${count} Timeline sub-items (${scenario})`, { cause: error });
  } finally {
    await page.close();
  }
}

async function narrowGallery(browser) {
  const page = await browser.newPage({ viewport: { width: 1400, height: 900 } });
  page.setDefaultTimeout(60_000);
  try {
    // Leave two slots below the supported row limit for the two creates.
    await openTasks(page, 199_998);
    await page.getByRole('tab', { name: 'Gallery', exact: true }).click();
    const gallery = page.getByRole('region', { name: 'Gallery', exact: true });
    await gallery.locator('[data-card]').first().waitFor();
    await page.setViewportSize({ width: 640, height: 900 });
    await page.waitForFunction(() => {
      const grid = document.querySelector('[data-grid] [data-index]');
      return grid && getComputedStyle(grid).gridTemplateColumns.trim().split(/\s+/).length === 1;
    });
    await gallery.evaluate((el) => { el.scrollTop = el.scrollHeight; });
    await gallery.getByText('Task number 199997', { exact: true }).waitFor();
    await gallery.locator('[data-new-card]').click();
    const editor = gallery.getByRole('textbox', { name: 'Title of the new row' });
    for (const title of ['P8 narrow gallery first row', 'P8 narrow gallery second row']) {
      await editor.fill(title);
      await editor.press('Enter');
      await gallery.getByRole('button', { name: title, exact: true }).waitFor();
      await page.waitForFunction(() => {
        const input = document.querySelector('[aria-label="Title of the new row"]');
        const viewport = input?.closest('[aria-label="Gallery"]')?.getBoundingClientRect();
        const rect = input?.getBoundingClientRect();
        return input === document.activeElement && rect && viewport && rect.top >= viewport.top && rect.bottom <= viewport.bottom;
      });
    }
    await bounded(page, configs.Gallery);
    console.log('199998 → 200000 narrow Gallery: tail and two consecutive in-viewport creates passed');
  } finally {
    await page.close();
  }
}

(async () => {
  const browser = await chromium.launch(process.env.NOTAYA_CHROMIUM_PATH ? { executablePath: process.env.NOTAYA_CHROMIUM_PATH } : {});
  try {
    for (const count of counts) {
      const page = await browser.newPage({ viewport: { width: 1400, height: 900 } });
      page.setDefaultTimeout(60_000);
      const errors = [];
      page.on('pageerror', (error) => errors.push(error.message));
      try {
        await openTasks(page, count);
        for (const name of Object.keys(configs)) {
          try {
            await checkView(page, name, count);
          } catch (error) {
            throw new Error(`${count} rows in ${name}`, { cause: error });
          }
        }
        assert.deepEqual(errors, []);
      } finally {
        await page.close();
      }
    }
    if (counts.includes(200_000)) await narrowGallery(browser);
    await highCardinalityBoard(browser, 'columns', 1000);
    await highCardinalityBoard(browser, 'lanes', 500);
    await timelineMetadata(browser);
    await timelineMetadata(browser, true);
    for (const count of [400, 5_000]) {
      await timelineSubItems(browser, count);
      await timelineSubItems(browser, count, { undated: true });
    }
    await timelineSubItems(browser, 5_000, { grouped: true });
    console.log('ALL OK P8 virtual List, Gallery, Board, and Timeline');
  } finally {
    await browser.close();
  }
})().catch((error) => { console.error(error); process.exitCode = 1; });
