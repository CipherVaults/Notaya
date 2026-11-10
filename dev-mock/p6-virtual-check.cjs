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

(async () => {
  const browser = await chromium.launch({ ...(process.env.NOTAYA_CHROMIUM_PATH ? { executablePath: process.env.NOTAYA_CHROMIUM_PATH } : {}), ignoreDefaultArgs: ['--hide-scrollbars'] });
  try {
    for (const n of counts) {
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
      await finishPaint(p);
      if (!baseline) assert.equal(await grid.getAttribute('aria-rowcount'), String(n + 3), 'Logical row count includes header, New line and footer');
      if (!baseline) await p.evaluate(() => {
        window.__p6MaxRows = 0;
        const measure = () => { window.__p6MaxRows = Math.max(window.__p6MaxRows, document.querySelectorAll('[aria-label="Table"] tbody tr').length); };
        window.__p6Observer = new MutationObserver(measure);
        window.__p6Observer.observe(document.querySelector('[aria-label="Table"] tbody'), { childList: true }); measure();
      });
      const report = { phase: baseline ? 'before P6 (P1, 300 drawn)' : 'after P6', rows: n, interactions: {} };
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
      const estimateSum = Array.from({ length: n }, (_, i) => [1, 2, 3, 5, 8, 13][i % 6]).reduce((a, b) => a + b, 0);
      const footerText = () => table.locator('tfoot').getByRole('button', { name: 'Calculate Estimate', exact: true }).innerText();
      const totalBefore = await footerText();
      assert.equal(Number(totalBefore.replace(/[^\d.-]/g, '')), estimateSum, 'Full-result Estimate sum');
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

      // Group count and a full-group footer sum, then collapse/expand. Grouping
      // is supplied by the mock engine, never calculated from mounted rows.
      await p.getByRole('button', { name: 'Group', exact: true }).click();
      await p.getByRole('combobox', { name: 'Group', exact: true }).selectOption({ label: 'Status' });
      await p.keyboard.press('Escape');
      await scroll(false);
      const firstGroup = table.locator('tr[data-group-key]').filter({ hasText: 'Not started' }).first();
      await firstGroup.waitFor();
      const groupText = await firstGroup.innerText();
      const groupCount = Math.ceil(n / 4);
      assert(groupText.includes('Not started') && groupText.includes(String(groupCount)), 'First group label/count');
      await table.evaluate((el, count) => { el.scrollTop = count * 35 - 100; }, groupCount);
      const groupFooter = table.locator('tr[data-table-footer]').filter({ has: p.getByRole('button', { name: /Calculate Estimate/ }).filter({ hasText: /[1-9]/ }) }).first().getByRole('button', { name: 'Calculate Estimate', exact: true });
      await groupFooter.waitFor();
      const groupTotal = await groupFooter.innerText();
      const expectedGroupSum = Array.from({ length: n }, (_, i) => i % 4 === 0 ? [1, 2, 3, 5, 8, 13][i % 6] : 0).reduce((a, b) => a + b, 0);
      assert.equal(Number(groupTotal.replace(/[^\d.-]/g, '')), expectedGroupSum, 'Group full-result sum');
      await scroll(true); await scroll(false);
      assert.equal(await firstGroup.innerText(), groupText, 'Group label/count survive scrolling');
      await firstGroup.getByRole('button', { name: 'Collapse group', exact: true }).click();
      await firstGroup.getByRole('button', { name: 'Expand group', exact: true }).waitFor(); await bounded();
      await firstGroup.getByRole('button', { name: 'Expand group', exact: true }).click();
      await firstGroup.getByRole('button', { name: 'Collapse group', exact: true }).waitFor();
      await table.evaluate((el, count) => { el.scrollTop = count * 35 - 100; }, groupCount);
      await groupFooter.waitFor(); assert.equal(await groupFooter.innerText(), groupTotal);
      report.maxTbodyRows = await p.evaluate(() => window.__p6MaxRows);
      assert(report.maxTbodyRows < 120, `Observed maximum tbody rows ${report.maxTbodyRows}`);
      assert.deepEqual(errors, [], 'No page errors');
      console.log(JSON.stringify({ rows: n, maxTbodyRows: report.maxTbodyRows })); console.log(`ALL OK P6 ${n}`);
      await context.close();
    }
  } finally { await browser.close(); }
})().catch((e) => { console.error(e); process.exitCode = 1; });
