// Development only, synthetic mock data. Caller owns the mock server; no real app/vault.
// node dev-mock/p6-virtual-check.cjs http://127.0.0.1:1421 [5000 20000] [--baseline]
// --baseline runs the SAME timing routine against P1 (300 rows drawn), no virtual assertions.
// NOTAYA_PLAYWRIGHT_MODULE / NOTAYA_CHROMIUM_PATH select already installed tools. No install.
const assert = require('node:assert/strict');
const { chromium } = require(process.env.NOTAYA_PLAYWRIGHT_MODULE || 'playwright');
const args = process.argv.slice(2);
const base = new URL(args.find((a) => /^https?:/.test(a)) || 'http://127.0.0.1:1421');
assert(['localhost', '127.0.0.1', '[::1]'].includes(base.hostname), 'Use a local throwaway mock');
const counts = args.filter((a) => /^\d+$/.test(a)).map(Number);
if (!counts.length) counts.push(5000, 20000);
assert(counts.every((n) => n >= 300 && n <= 50000), 'Row counts must be 300..50000');
const baseline = args.includes('--baseline');
const finishPaint = (p) => p.evaluate(() => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => setTimeout(resolve, 0)))));

// TEST-ONLY: independent seed arithmetic; query calcs and mounted windows are checked outputs.
const expectedSeed = (n, fixture) => {
  const values = new Map(), pattern = [1, 2, 3, 5, 8, 13];
  for (let status = 0; status < 4; status++) {
    const ordinals = [];
    for (let i = status; i < n; i += 4) ordinals.push(i);
    for (let member = 0; member < ordinals.length; member += 2) {
      const magnitude = pattern[(member / 2) % pattern.length];
      for (let j = member; j < Math.min(member + 2, ordinals.length); j++) {
        const i = ordinals[j];
        values.set(i + 1, { status, estimate: fixture === 'ordinary' ? pattern[i % pattern.length] : member + 1 === ordinals.length ? 0 : j === member ? magnitude : -magnitude });
      }
    }
  }
  return values;
};
const mockSnapshot = (p) => p.evaluate(async () => {
  const { invoke } = await import('/dev-mock/core.ts');
  const tree = await invoke('list_notes');
  const tasks = tree.databases.filter((d) => d.title === 'Tasks');
  if (tasks.length !== 1) throw new Error('Expected exactly one synthetic Tasks database');
  const database = await invoke('get_database', { id: tasks[0].id });
  const tables = database.views.filter((v) => v.name === 'All tasks' && v.kind === 'table');
  if (tables.length !== 1) throw new Error('Expected one synthetic All tasks table');
  const view = tables[0];
  const result = await invoke('query_view', { request: { databaseId: database.id, viewId: view.id } });
  return { database, view, result };
});
const checkSeed = (snapshot, n, fixture, prior) => {
  const { database, view, result } = snapshot;
  const oneProp = (name, kind) => {
    const props = database.props.filter((prop) => prop.name === name && prop.kind === kind);
    assert.equal(props.length, 1, 'Unique property ' + name); return props[0];
  };
  const status = oneProp('Status', 'status'), estimate = oneProp('Estimate', 'number');
  assert.deepEqual(status.options.map((o) => o.name), ['Not started', 'In progress', 'Review', 'Done']);
  assert.equal(estimate.numberFormat, 'number');
  assert.equal(result.total, n); assert.equal(result.filtered, n); assert.equal(result.truncated, false);
  assert.equal(result.rows.length, n); assert.equal(new Set(result.rows.map((r) => r.id)).size, n);
  const reference = expectedSeed(n, fixture), byId = new Map(), byNumber = new Map();
  for (const row of result.rows) {
    assert(!byNumber.has(row.number), 'Unique seeded row number');
    const expected = reference.get(row.number); assert(expected, 'Seeded row ' + row.number);
    assert.equal(row.values[status.id], status.options[expected.status].id, 'Seeded Status ' + row.number);
    assert.equal(row.values[estimate.id], expected.estimate, 'Seeded Estimate ' + row.number);
    byId.set(row.id, { ...expected, number: row.number }); byNumber.set(row.number, row.id);
  }
  if (prior) {
    assert.equal(database.id, prior.databaseId); assert.equal(view.id, prior.viewId);
    assert.equal(status.id, prior.status.id); assert.equal(estimate.id, prior.estimate.id);
    assert.deepEqual([...byNumber].sort((a,b) => a[0]-b[0]), [...prior.byNumber].sort((a,b) => a[0]-b[0]), 'Same identities after edit, sort and drag');
  }
  return { databaseId: database.id, viewId: view.id, status, estimate, byId, byNumber, total: [...reference.values()].reduce((sum, row) => sum + row.estimate, 0) };
};
const digestIds = (ids) => require('node:crypto').createHash('sha256').update([...ids].sort().join('\n')).digest('hex');
// Observe a completed saved generation; never force a save or move focus.
const waitForSavedView = async (p, seed, collapsed, previousRevision = -1) => {
  const observe = () => p.evaluate(async ({ databaseId, viewId, statusId, collapsed, previousRevision }) => {
    const { invoke } = await import('/dev-mock/core.ts');
    const { peekDatabase, hasUnsaved } = await import('/src/features/database/dbStore.ts');
    if (hasUnsaved(databaseId)) return false;
    const local = peekDatabase(databaseId), localView = local?.views.find((view) => view.id === viewId);
    if (!localView || localView.group?.prop !== statusId) return false;
    if (JSON.stringify(localView.group.collapsed) !== JSON.stringify(collapsed)) return false;
    const viewKey = JSON.stringify(localView);
    const saved = await invoke('get_database', { id: databaseId });
    const savedView = saved.views.find((view) => view.id === viewId);
    const current = peekDatabase(databaseId), currentView = current?.views.find((view) => view.id === viewId);
    if (hasUnsaved(databaseId) || saved.id !== databaseId || !savedView || !currentView) return false;
    if (!Number.isInteger(saved.rev) || saved.rev <= previousRevision || current.rev !== saved.rev) return false;
    if (JSON.stringify(savedView) !== viewKey || JSON.stringify(currentView) !== viewKey) return false;
    return { databaseId, viewId, revision: saved.rev, viewKey };
  }, { databaseId: seed.databaseId, viewId: seed.viewId, statusId: seed.status.id, collapsed, previousRevision });
  const timeoutMs = seed.byId.size >= 20000 ? 120000 : 30000;
  let stopped = false, timeout;
  const ensureActive = () => {
    if (stopped) throw new Error('Saved-generation observation cancelled');
  };
  const poll = async () => {
    for (;;) {
      ensureActive();
      const completion = await observe();
      ensureActive();
      if (completion !== false) {
        assert(completion && typeof completion === 'object' && !Array.isArray(completion), 'Saved generation returns a completion object');
        assert.equal(completion.databaseId, seed.databaseId, 'Saved completion database identity');
        assert.equal(completion.viewId, seed.viewId, 'Saved completion view identity');
        assert(Number.isInteger(completion.revision) && completion.revision > previousRevision, 'Saved completion revision advances');
        assert.equal(typeof completion.viewKey, 'string', 'Saved completion has an exact view key');
        return completion;
      }
      // Preserve waitForFunction's default RAF cadence after an awaited false observation.
      await p.evaluate(() => new Promise((resolve) => requestAnimationFrame(resolve)));
      ensureActive();
    }
  };
  const deadline = new Promise((_, reject) => {
    timeout = setTimeout(() => {
      stopped = true;
      reject(new Error('Saved generation did not complete within ' + timeoutMs + 'ms'));
    }, timeoutMs);
  });
  try {
    // Promise.race installs rejection handlers on both branches, including late settlement.
    return await Promise.race([poll(), deadline]);
  } finally {
    stopped = true;
    clearTimeout(timeout);
  }
};
const verifyGroups = async (p, table, seed, n, fixture, bounded, finishPaint, completion) => {
  const snapshot = await mockSnapshot(p);
  checkSeed(snapshot, n, fixture, seed);
  assert.equal(snapshot.database.id, completion.databaseId); assert.equal(snapshot.view.id, completion.viewId);
  assert.equal(snapshot.database.rev, completion.revision, 'Snapshot uses completed saved revision');
  assert.equal(JSON.stringify(snapshot.view), completion.viewKey, 'Snapshot uses exact completed view definition');
  const { view, result } = snapshot;
  assert.equal(view.group?.prop, seed.status.id); assert.equal(view.group.sort, 'manual');
  assert.equal(view.group.dateBy, ''); assert.equal(view.group.hideEmpty, false);
  assert.deepEqual(view.group.hidden, []); assert.deepEqual(view.group.collapsed, []);
  assert.equal(view.subGroup, null); assert.deepEqual(view.sorts, []); assert.equal(view.wrap, false);
  assert.equal(view.calcs.filter((c) => c.prop === seed.estimate.id && c.calc === 'sum').length, 1);
  const keys = ['', ...seed.status.options.map((o) => o.id)];
  assert.deepEqual(result.groups.map((g) => g.key), keys, 'Exact group order, including empty No Status');
  const descriptors = []; let index = 0, pixels = 0;
  for (const [position, group] of result.groups.entries()) {
    const label = position === 0 ? 'No Status' : seed.status.options[position - 1].name;
    const members = [...seed.byId].filter(([, row]) => position > 0 && row.status === position - 1);
    const expectedIds = members.map(([id]) => id), expectedSet = new Set(expectedIds);
    const sum = members.reduce((value, [, row]) => value + row.estimate, 0);
    assert.equal(group.label, label); assert.equal(group.count, expectedIds.length); assert.deepEqual(group.subs, []);
    assert.equal(new Set(group.rows).size, group.rows.length);
    assert.deepEqual([...group.rows].sort(), [...expectedIds].sort(), label + ': complete membership independent of DOM');
    assert.deepEqual(group.rows, result.rows.filter((row) => expectedSet.has(row.id)).map((row) => row.id), label + ': ordered membership');
    assert.equal(group.calcs[seed.estimate.id], String(sum), label + ': calc versus independent seed arithmetic');
    const headerIndex = index, headerOffset = pixels, addIndex = index + 1 + expectedIds.length, footerIndex = addIndex + 1;
    const footerOffset = pixels + 37 + expectedIds.length * 35 + 33;
    const header = table.locator('tbody tr[data-group-key="' + group.key + '"][data-index="' + headerIndex + '"]');
    const mount = async (offset, locator) => {
      await table.evaluate((el, offset) => { el.scrollTop = Math.max(0, offset - 50); }, offset);
      await locator.waitFor({ state: 'attached' }); await finishPaint(p); await bounded();
      assert.equal(await locator.count(), 1, 'Exact logical row identity');
    };
    await mount(headerOffset, header);
    assert.equal(await header.getAttribute('aria-rowindex'), String(headerIndex + 2));
    assert.equal(await header.getByText(label, { exact: true }).count(), 1);
    assert.equal(await header.locator('td > div > span.text-xs.tabular-nums.text-faint').innerText(), String(expectedIds.length));
    const footer = table.locator('tbody tr[data-table-footer][data-index="' + footerIndex + '"]');
    await mount(footerOffset, footer);
    assert.equal(await footer.getAttribute('aria-rowindex'), String(footerIndex + 2));
    const add = table.locator('tbody tr[data-index="' + addIndex + '"]');
    assert.equal(await add.getByRole('button', { name: 'New', exact: true }).count(), 1);
    if (expectedIds.length) {
      const lastId = group.rows.at(-1);
      const last = table.locator('tbody tr[data-row-id="' + lastId + '"][data-index="' + (addIndex - 1) + '"]');
      // A group footer has metadata on hand; its preceding row arrives via IPC.
      await last.waitFor({ state: 'attached' });
      assert.equal(await last.count(), 1);
    } else assert.equal(footerIndex, headerIndex + 2, 'Empty header/New/zero footer');
    const button = footer.getByRole('button', { name: 'Calculate Estimate', exact: true });
    assert.equal(await button.count(), 1);
    assert.equal(await button.locator(':scope > span.text-faint').innerText(), 'Sum');
    assert.equal(await button.locator(':scope > span.font-medium.tabular-nums.text-fg').innerText(), String(sum), 'Exact footer value; zero is valid');
    descriptors.push({ key: group.key, label, count: expectedIds.length, sum, headerIndex, footerIndex, headerOffset, footerOffset, memberIdsSHA256: digestIds(expectedIds) });
    index = footerIndex + 1; pixels = footerOffset + 33;
  }
  assert.equal(await table.getByRole('grid').getAttribute('aria-rowcount'), String(index + 1));
  if (fixture === 'zero-sum') for (const group of descriptors) {
    assert.equal(group.sum, 0);
    if (group.count) {
      const members = [...seed.byId.values()].filter((row) => row.status === seed.status.options.findIndex((o) => o.id === group.key));
      assert(members.some((row) => row.estimate > 0) && members.some((row) => row.estimate < 0), 'Nonempty zero-sum has both signs');
    }
  }
  return descriptors;
};

(async () => {
  const browser = await chromium.launch({ ...(process.env.NOTAYA_CHROMIUM_PATH ? { executablePath: process.env.NOTAYA_CHROMIUM_PATH } : {}), ignoreDefaultArgs: ['--hide-scrollbars'] });
  try {
    for (const [n, fixture] of counts.flatMap((n) => (baseline ? ['ordinary'] : ['ordinary', 'zero-sum']).map((fixture) => [n, fixture]))) {
      const context = await browser.newContext({ viewport: { width: 1400, height: 900 } });
      const p = await context.newPage();
      p.setDefaultTimeout(n >= 20000 ? 120000 : 30000);
      const errors = [];
      p.on('pageerror', (e) => errors.push(e.message));
      await p.addInitScript(() => {
        window.__p6Long = [];
        new PerformanceObserver((list) => { for (const e of list.getEntries()) window.__p6Long.push({ start: e.startTime, duration: e.duration }); }).observe({ entryTypes: ['longtask'] });
      });
      const url = new URL(base);
      url.searchParams.set('db', '1'); url.searchParams.set('rows', String(n));
      if (!baseline) url.searchParams.set('p6Oracle', fixture);
      await p.goto(url.href);
      await p.getByRole('treeitem', { name: 'Tasks', exact: true }).click();
      const table = p.getByRole('application', { name: 'Table', exact: true });
      const grid = table.getByRole('grid');
      // Also works on P1, which does not yet have data-row-id.
      const rows = () => table.locator('tbody tr[aria-selected]');
      const title = (i) => rows().nth(i).getByRole('gridcell').nth(1);
      const rowNumber = (number) => rows().filter({ has: p.getByRole('checkbox', { name: `Select row ${number}`, exact: true }) });
      const activeRowNumber = () => table.locator('tbody tr[aria-selected]').filter({ has: p.locator('.db-cell-active') }).getByRole('checkbox', { name: /^Select row / }).getAttribute('aria-label').then((s) => Number(s.replace('Select row ', '')));
      await title(0).waitFor();
      const seed = baseline ? null : checkSeed(await mockSnapshot(p), n, fixture);
      await finishPaint(p);
      if (!baseline) assert.equal(await grid.getAttribute('aria-rowcount'), String(n + 3), 'Logical row count includes header, New line and footer');
      if (!baseline) await p.evaluate(() => {
        window.__p6MaxRows = 0;
        const measure = () => { window.__p6MaxRows = Math.max(window.__p6MaxRows, document.querySelectorAll('[aria-label="Table"] tbody tr').length); };
        window.__p6Observer = new MutationObserver(measure);
        window.__p6Observer.observe(document.querySelector('[aria-label="Table"] tbody'), { childList: true }); measure();
      });
      const report = { phase: baseline ? 'before P6 (P1, 300 drawn)' : 'after P6', rows: n, fixture, interactions: {} };
      const timeIt = async (name, action) => {
        const start = await p.evaluate(() => performance.now());
        await action(); await finishPaint(p);
        const timing = await p.evaluate(({ start }) => ({ end: performance.now(), tasks: window.__p6Long.filter((e) => e.start >= start) }), { start });
        report.interactions[name] = { wallMs: +(timing.end - start).toFixed(1), longTasks: timing.tasks.length, longTaskMs: +timing.tasks.reduce((s, e) => s + e.duration, 0).toFixed(1) };
      };
      const checkbox = () => rowNumber(3).getByRole('checkbox', { name: 'Select row 3', exact: true });
      await timeIt('select', async () => { await checkbox().click({ force: true }); await p.getByText('1 selected', { exact: true }).waitFor(); });
      await timeIt('deselect', async () => { await checkbox().click({ force: true }); await p.getByRole('button', { name: 'Clear selection' }).waitFor({ state: 'hidden' }); });
      await timeIt('click cell', () => title(3).click({ position: { x: 14, y: 16 } }));
      const edited = 'P6 retained title';
      await timeIt('edit title and Enter', async () => {
        const queries = await p.evaluate(() => (window.__inv || []).filter(([cmd]) => cmd === 'query_view').length);
        await p.keyboard.press('Enter');
        await table.getByRole('textbox', { name: 'Task', exact: true }).fill(edited);
        await p.keyboard.press('Enter');
        await rowNumber(4).getByRole('gridcell').nth(1).filter({ hasText: edited }).waitFor();
        await p.waitForFunction((queries) => (window.__inv || []).filter(([cmd]) => cmd === 'query_view').length > queries, queries);
      });
      const sort = async () => {
        await p.getByRole('button', { name: 'Sort', exact: true }).click();
        const panel = p.locator('[data-radix-popper-content-wrapper]').last();
        await panel.getByRole('button', { name: 'Add sort', exact: true }).click();
        await panel.getByLabel('Sort by', { exact: true }).selectOption({ label: 'Task' });
        await p.keyboard.press('Escape');
        await title(0).filter({ hasText: 'Accessibility audit' }).waitFor();
      };
      await timeIt('sort by title (toolbar)', sort);
      report.memory = await p.evaluate(() => ({ jsHeapBytes: performance.memory?.usedJSHeapSize ?? null, domNodes: document.querySelectorAll('*').length, domRows: document.querySelectorAll('[aria-label="Table"] tbody tr[aria-selected]').length }));
      // Preserve timings even if a later interaction assertion fails.
      console.log(JSON.stringify(report));
      if (baseline) { assert.deepEqual(errors, []); await context.close(); continue; }
      assert.equal(await table.getByRole('button', { name: 'Drag to reorder' }).count(), 0, 'Sorted rows cannot be dragged');
      await p.getByRole('button', { name: 'Sort', exact: true }).click();
      await p.getByRole('button', { name: 'Remove sort', exact: true }).click();
      await p.keyboard.press('Escape');
      await title(0).filter({ hasText: 'Design the landing page' }).waitFor();
      const bounded = async () => {
        const count = await table.locator('tbody tr').count();
        assert(count < 120, `${n} loaded: ${count} tbody rows, must be <120 including spacers`);
      };
      const scroll = async (end) => {
        await table.evaluate((el, end) => { el.scrollTop = end ? el.scrollHeight : 0; }, end);
        await p.waitForFunction((number) => {
          const wrap = document.querySelector('[aria-label="Table"]');
          const row = wrap.querySelector(`[aria-label="Select row ${number}"]`)?.closest('tr');
          if (!row) return false;
          const r = row.getBoundingClientRect(), v = wrap.getBoundingClientRect();
          return r.top >= v.top + 32 && r.bottom <= v.bottom;
        }, end ? n : 1);
        await finishPaint(p); await bounded();
      };
      const estimateSum = seed.total;
      const footerText = () => table.locator('tfoot').getByRole('button', { name: 'Calculate Estimate', exact: true }).innerText();
      const totalBefore = await footerText();
      assert.equal(await table.locator('tfoot').getByRole('button', { name: 'Calculate Estimate', exact: true }).locator(':scope > span.font-medium.tabular-nums.text-fg').innerText(), String(estimateSum), 'Independent full-result sum; zero is valid');
      await scroll(true);
      assert((await rowNumber(n).innerText()).includes(`Task number ${n - 1}`), 'Last row is scrollable');
      assert.equal(await rowNumber(n).getAttribute('aria-rowindex'), String(n + 1), 'Last data row has its logical position');
      await scroll(false);
      assert.equal(await footerText(), totalBefore, 'Total survives scrolling');
      assert.equal(await p.getByRole('button', { name: /Show more rows/ }).count(), 0);

      // Native sticky header and frozen title: compare geometry after both axes move.
      const geometry = () => table.evaluate((el) => {
        const th = el.querySelector('thead th[data-col="0"]');
        const r = th.getBoundingClientRect(), viewport = el.getBoundingClientRect();
        const body = [...el.querySelectorAll('tbody tr[data-row-id]')].find((tr) => { const b = tr.getBoundingClientRect(); return b.top > viewport.top + 40 && b.bottom < viewport.bottom; });
        const cell = body?.querySelector('[role="gridcell"]')?.getBoundingClientRect();
        return { headerTop: r.top, headerX: r.x, headerWidth: r.width, bodyX: cell?.x, bodyWidth: cell?.width };
      });
      const before = await geometry();
      await table.evaluate((el) => { el.scrollTop = 8000; el.scrollLeft = 700; });
      await p.waitForFunction(() => [...document.querySelectorAll('[aria-label="Table"] tbody tr[data-row-id]')].some((row) => Number(row.dataset.index) > 100));
      await finishPaint(p);
      const after = await geometry();
      for (const key of ['headerTop', 'headerX', 'headerWidth', 'bodyX', 'bodyWidth']) assert(Math.abs(before[key] - after[key]) <= 1.5, `Sticky geometry ${key}: ${before[key]} -> ${after[key]}`);
      await table.evaluate((el) => { el.scrollLeft = 0; });
      await scroll(false);

      await title(0).click({ position: { x: 14, y: 16 } });
      for (const [key, expected] of [['End', n], ['ArrowUp', n - 1], ['Home', 1], ['ArrowDown', 2]]) {
        await p.keyboard.press(key);
        await p.waitForFunction((expected) => document.activeElement?.closest('tr')?.querySelector(`[aria-label="Select row ${expected}"]`), expected);
        assert.equal(await activeRowNumber(), expected, key);
        const cell = table.locator('.db-cell-active');
        const bounds = await cell.boundingBox(), viewport = await table.boundingBox();
        assert(bounds.y >= viewport.y + 32 && bounds.y + bounds.height <= viewport.y + viewport.height, `${key} scrolls focused cell into view`);
        await bounded();
      }
      await p.keyboard.press('PageDown'); await finishPaint(p);
      const pageRow = await activeRowNumber(); assert(pageRow > 2, 'PageDown advances');
      await p.keyboard.press('PageUp'); await finishPaint(p); assert.equal(await activeRowNumber(), 2, 'PageUp returns');
      await p.keyboard.press('ArrowRight'); await finishPaint(p); assert.equal(await table.locator('.db-cell-active').getAttribute('aria-colindex'), '3');
      await p.keyboard.press('Tab'); await finishPaint(p); assert.equal(await table.locator('.db-cell-active').getAttribute('aria-colindex'), '4');
      await p.keyboard.press('Escape');

      // Shift range includes rows absent from the DOM. Active row remains ID-based.
      await scroll(false);
      await rowNumber(1).getByRole('checkbox', { name: 'Select row 1', exact: true }).click({ force: true });
      await scroll(true);
      await rowNumber(n).getByRole('checkbox', { name: `Select row ${n}`, exact: true }).click({ modifiers: ['Shift'], force: true });
      await p.getByText(`${n} selected`, { exact: true }).waitFor();
      await p.getByRole('button', { name: 'Clear selection', exact: true }).click();
      await table.getByRole('checkbox', { name: 'Select all rows', exact: true }).click({ force: true });
      await p.getByText(`${n} selected`, { exact: true }).waitFor();
      await table.getByRole('checkbox', { name: 'Select all rows', exact: true }).click({ force: true });
      await p.getByRole('button', { name: 'Clear selection' }).waitFor({ state: 'hidden' });
      await scroll(false);
      assert((await rowNumber(4).innerText()).includes(edited), 'Committed edit retained away and back');
      await title(3).click({ position: { x: 14, y: 16 } }); await p.keyboard.press('Enter');
      const editor = table.getByRole('textbox', { name: 'Task', exact: true });
      await editor.fill('Uncommitted P6 draft');
      const handle = await editor.elementHandle();
      await table.evaluate((el) => { el.scrollTop = 10000; });
      await finishPaint(p); await bounded();
      assert.equal(await editor.inputValue(), 'Uncommitted P6 draft');
      assert(await handle.evaluate((el) => el === document.activeElement), 'Pinned editor keeps its focus and draft');
      await table.evaluate((el) => { el.scrollTop = 0; }); await finishPaint(p);
      await p.keyboard.press('Escape');
      assert((await rowNumber(4).innerText()).includes(edited), 'Escape cancels the draft');

      // Drag from first row to a mounted target, then from first into an initially
      // undrawn window. Pointer movement at the lower edge must drive dnd auto-scroll.
      const settleScroll = () => table.evaluate(async (el) => {
        let previous = el.scrollTop, stable = 0;
        const deadline = performance.now() + 5000;
        while (stable < 8) {
          await new Promise(requestAnimationFrame);
          if (performance.now() > deadline) throw new Error('Drag auto-scroll did not settle away from the edge');
          stable = el.scrollTop === previous ? stable + 1 : 0;
          previous = el.scrollTop;
        }
        return el.scrollTop;
      });
      const drag = async (autoScroll) => {
        await scroll(false);
        const sourceId = await rowNumber(1).getAttribute('data-row-id');
        const source = await rowNumber(1).getByRole('button', { name: 'Drag to reorder', exact: true }).boundingBox();
        await p.mouse.move(source.x + source.width / 2, source.y + source.height / 2); await p.mouse.down();
        await p.mouse.move(source.x + source.width / 2, source.y + 12, { steps: 4 });
        let targetId;
        if (autoScroll) {
          const viewport = await table.boundingBox();
          await p.mouse.move(source.x + source.width / 2, viewport.y + viewport.height - 12, { steps: 12 });
          await p.waitForFunction(() => document.querySelector('[aria-label="Table"]').scrollTop > 1800);
          // Leave the auto-scroll zone BEFORE choosing a row. The old script read
          // its coordinates at the edge, then spent eight pointer steps scrolling
          // past them. Sortable siblings also have a temporary one-row transform;
          // use their layout boxes, the geometry dnd-kit measures for collisions.
          await p.mouse.move(source.x + source.width / 2, viewport.y + viewport.height / 2);
          await settleScroll();
          const target = await table.locator('tbody tr[data-row-id]').evaluateAll((trs) => {
            const wrap = document.querySelector('[aria-label="Table"]').getBoundingClientRect();
            const candidates = trs.map((tr) => {
              const b = tr.getBoundingClientRect();
              const dy = new DOMMatrixReadOnly(getComputedStyle(tr).transform).m42;
              return { id: tr.dataset.rowId, x: b.x + 45, y: b.top - dy + b.height / 2, height: b.height };
            }).filter((r) => r.y - r.height / 2 > wrap.top + wrap.height * 0.3 && r.y + r.height / 2 < wrap.bottom - wrap.height * 0.3);
            candidates.sort((a, b) => Math.abs(a.y - (wrap.top + wrap.height / 2)) - Math.abs(b.y - (wrap.top + wrap.height / 2)));
            if (!candidates.length) throw new Error('No settled drop row in the middle of the viewport');
            return candidates[0];
          });
          targetId = target.id;
          await p.mouse.move(target.x, target.y);
          await settleScroll();
        } else {
          const target = rowNumber(4); targetId = await target.getAttribute('data-row-id');
          const b = await target.boundingBox(); await p.mouse.move(source.x + source.width / 2, b.y + b.height / 2, { steps: 12 });
        }
        // Capture the label at release, after the final scrolling and pointer move.
        const originalLabel = await table.locator(`tr[data-row-id="${targetId}"]`).getByRole('checkbox', { name: /^Select row / }).getAttribute('aria-label');
        assert(originalLabel, 'Drop target still exists immediately before release');
        await p.mouse.up();
        // Source replaces target's former order position; query_view must finish.
        await p.waitForFunction(({ sourceId, originalLabel }) => document.querySelector(`tr[data-row-id="${sourceId}"]`)?.querySelector(`[aria-label="${originalLabel}"]`), { sourceId, originalLabel });
        await bounded();
      };
      await drag(false); await drag(true);

      // Exact key/header index, complete seed membership and independent sum bind every footer.
      await p.getByRole('button', { name: 'Group', exact: true }).click();
      await p.getByRole('combobox', { name: 'Group', exact: true }).selectOption({ label: 'Status' });
      await p.keyboard.press('Escape');
      const grouped = await waitForSavedView(p, seed, []);
      const beforeGroups = await verifyGroups(p, table, seed, n, fixture, bounded, finishPaint, grouped);
      const notStarted = beforeGroups.find((group) => group.key === seed.status.options[0].id);
      assert(notStarted);
      const firstGroup = table.locator('tbody tr[data-group-key="' + notStarted.key + '"][data-index="' + notStarted.headerIndex + '"]');
      await scroll(true); await scroll(false);
      await firstGroup.waitFor();
      const groupText = await firstGroup.innerText();
      await firstGroup.getByRole('button', { name: 'Collapse group', exact: true }).click();
      await firstGroup.getByRole('button', { name: 'Expand group', exact: true }).waitFor(); await bounded();
      const collapsed = await waitForSavedView(p, seed, [notStarted.key], grouped.revision);
      await firstGroup.getByRole('button', { name: 'Expand group', exact: true }).click();
      await firstGroup.getByRole('button', { name: 'Collapse group', exact: true }).waitFor();
      const expanded = await waitForSavedView(p, seed, [], collapsed.revision);
      assert.equal(await firstGroup.innerText(), groupText, 'Exact header/count survives scrolling and collapse/expand');
      const afterGroups = await verifyGroups(p, table, seed, n, fixture, bounded, finishPaint, expanded);
      assert.deepEqual(afterGroups, beforeGroups, 'All exact group identities/counts/footer sums survive collapse/expand');
      console.log(JSON.stringify({ rows: n, fixture, groupOracle: afterGroups, groupSaveProof: { grouped, collapsed, expanded } }));
      report.maxTbodyRows = await p.evaluate(() => window.__p6MaxRows);
      assert(report.maxTbodyRows < 120, `Observed maximum tbody rows ${report.maxTbodyRows}`);
      assert.deepEqual(errors, [], 'No page errors');
      console.log(JSON.stringify({ rows: n, fixture, maxTbodyRows: report.maxTbodyRows })); console.log(`ALL OK P6 ${n} ${fixture}`);
      await context.close();
    }
  } finally { await browser.close(); }
})().catch((e) => { console.error(e); process.exitCode = 1; });
