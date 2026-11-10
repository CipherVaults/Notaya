// Synthetic mock only. Adds at the END of a 5,000-row table and uses real UI keys.
// node dev-mock/p6-added-row-undo-check.cjs http://127.0.0.1:1421
// Existing installed Playwright/Chromium can be selected with the P6 env variables.
const assert = require('node:assert/strict');
const { chromium } = require(process.env.NOTAYA_PLAYWRIGHT_MODULE || 'playwright');
const url = new URL(process.argv[2] || 'http://127.0.0.1:1421');
assert(['localhost', '127.0.0.1', '[::1]'].includes(url.hostname), 'Use a local throwaway mock');
url.searchParams.set('db', '1');
url.searchParams.set('rows', '5000');

(async () => {
  const browser = await chromium.launch(process.env.NOTAYA_CHROMIUM_PATH ? { executablePath: process.env.NOTAYA_CHROMIUM_PATH } : {});
  try {
    const p = await browser.newPage({ viewport: { width: 1100, height: 700 } });
    const errors = [];
    p.on('pageerror', (e) => errors.push(e.message));
    await p.goto(url.href);
    await p.getByRole('treeitem', { name: 'Tasks', exact: true }).click();
    const table = p.getByRole('application', { name: 'Table', exact: true });
    const grid = table.getByRole('grid');
    await table.locator('tbody tr[aria-selected]').first().waitFor();
    const databaseId = await p.locator('[data-database]').getAttribute('data-database');
    const snapshot = () => p.evaluate(async (databaseId) => {
      const { api } = await import('/src/lib/ipc.ts');
      const db = await api.getDatabase(databaseId);
      const result = await api.queryView({ databaseId, viewId: db.views[0].id });
      return result.rows.map(({ id, title }) => ({ id, title }));
    }, databaseId);
    const before = await snapshot();
    assert.equal(before.length, 5000);
    await table.evaluate((el) => { el.scrollTop = el.scrollHeight; });
    const newAtEnd = table.getByRole('button', { name: 'New', exact: true });
    await newAtEnd.click();
    const editor = table.getByRole('textbox', { name: 'Task', exact: true });
    await editor.waitFor();
    const addedId = await editor.evaluate((el) => el.closest('tr').dataset.rowId);
    assert(addedId && !before.some((row) => row.id === addedId), 'New has a fresh stable identity');
    assert.equal(await editor.evaluate((el) => el.closest('tr').querySelector('[role="checkbox"]').getAttribute('aria-label')), 'Select row 5001', 'New is appended at the end');
    assert(await editor.evaluate((el) => el === document.activeElement), 'The new title editor owns focus');
    await editor.fill('P6b end row undo');
    await p.keyboard.press('Enter');

    const expectState = async (label, count, title) => {
      await p.waitForFunction(async ({ databaseId, addedId, count, title }) => {
        const { api } = await import('/src/lib/ipc.ts');
        const { undoDepth } = await import('/src/features/database/undo.ts');
        const db = await api.getDatabase(databaseId);
        const result = await api.queryView({ databaseId, viewId: db.views[0].id });
        const added = result.rows.find((row) => row.id === addedId);
        // Wait for the view's requery too: a backend-only pass could hide stale UI.
        const grid = document.querySelector('[aria-label="Table"] [role="grid"]');
        return result.rows.length === count && (title === null ? !added : added?.title === title)
          && Number(grid?.getAttribute('aria-rowcount')) === count + 3
          && undoDepth(databaseId).undo + undoDepth(databaseId).redo === 2;
      }, { databaseId, addedId, count, title }, { polling: 200 });
      const rows = await snapshot();
      assert.equal(rows.length, count, `${label}: full-result count`);
      assert.equal(rows.find((row) => row.id === addedId)?.title ?? null, title, `${label}: added row identity/name`);
      assert.deepEqual(rows.filter((row) => row.id !== addedId), before, `${label}: existing rows unchanged`);
      assert.equal(await grid.getAttribute('aria-rowcount'), String(count + 3), `${label}: UI logical count`);
      console.log('ok  ', label, count, JSON.stringify(title));
    };
    await expectState('added and named', 5001, 'P6b end row undo');
    // No refocusing between the two undos/redos: removal must not lose the keys.
    await p.keyboard.press('Control+z');
    await expectState('undo name', 5001, '');
    await p.keyboard.press('Control+z');
    await expectState('undo row', 5000, null);
    await p.keyboard.press('Control+Shift+z');
    await expectState('redo row', 5001, '');
    await p.keyboard.press('Control+Shift+z');
    await expectState('redo name', 5001, 'P6b end row undo');
    await table.evaluate((el) => { el.scrollTop = el.scrollHeight; });
    const restored = table.locator(`tbody tr[aria-selected][data-row-id="${addedId}"]`);
    await restored.getByRole('gridcell').nth(1).filter({ hasText: 'P6b end row undo' }).waitFor();
    assert.equal(await restored.getByRole('checkbox', { name: 'Select row 5001', exact: true }).count(), 1);
    assert((await table.locator('tbody tr').count()) < 120, 'Virtual DOM stays bounded');
    assert.deepEqual(errors, [], 'No page errors');
    console.log('ALL OK P6b added row undo 5000');
  } finally { await browser.close(); }
})().catch((e) => { console.error(e); process.exitCode = 1; });
