// Lane A logic checks without a browser or real vault. Uses the locked TypeScript compiler
// to load the production store/navigation modules against the existing in-memory mock core.
// Run: node dev-mock/lane-a-check.cjs
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const ts = require('typescript');
const root = path.resolve(__dirname, '..');
const cache = new Map();
const saves = [];
const notices = [];
// The sidebar deliberately omits these pages; existing get_page still reads them (and rows).
const hiddenPages = new Map();
let failSave = false;
global.location = { search: '?db=1' };
const browserKeys = new Map([
  ['dbview:old-database', 'old-view'], ['dbview:another-database', 'another-view'],
  ['notaya.appearance', '{"theme":"paper"}'], ['notaya.layout', '{"open":false,"width":300}'],
  ['cipherleaf.layout', 'legacy-layout'], ['dbview', 'keep'], ['other:dbview:database', 'keep'],
]);
const unrelatedKeys = new Map([...browserKeys].filter(([key]) => !key.startsWith('dbview:')));
const browserStorage = {
  get length() { return browserKeys.size; },
  key: (i) => [...browserKeys.keys()][i] ?? null,
  removeItem: (key) => { assert(key.startsWith('dbview:'), 'Cleanup must only remove dbview: keys'); browserKeys.delete(key); },
};
global.localStorage = browserStorage;
const pause = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

function load(file) {
  file = path.resolve(file);
  if (!path.extname(file)) file += '.ts';
  if (cache.has(file)) return cache.get(file).exports;
  const module = { exports: {} };
  cache.set(file, module);
  const requireLocal = (name) => {
    if (name === 'sonner') return { toast: { error: (message) => notices.push(message) } };
    if (name === '@/lib/storage') return { getKey: () => null, setKey: () => { throw new Error('Navigation must not write browser storage'); } };
    if (name === '@tauri-apps/api/event') return load(path.join(__dirname, 'event.ts'));
    if (name === '@tauri-apps/api/core') return { invoke };
    if (name === '@/lib/ipc') return { api: {
      getSettings: () => invoke('get_settings'),
      listNotes: () => invoke('list_notes'),
      listStickies: () => invoke('list_stickies'),
      getPage: (id) => invoke('get_page', { id }),
      getDatabase: (id) => invoke('get_database', { id }),
    } };
    if (name.startsWith('@/')) return load(path.join(root, 'src', name.slice(2)));
    if (name.startsWith('.')) return load(path.resolve(path.dirname(file), name));
    return require(name);
  };
  const compiled = ts.transpileModule(fs.readFileSync(file, 'utf8'), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 }, fileName: file,
  }).outputText;
  vm.runInThisContext(`(function(require,module,exports){${compiled}\n})`, { filename: file })(requireLocal, module, module.exports);
  return module.exports;
}
async function invoke(command, args = {}) {
  if (command === 'get_page' && hiddenPages.has(args.id)) return structuredClone(hiddenPages.get(args.id));
  if (command === 'set_last_place') {
    saves.push(structuredClone(args.lastPlace));
    if (failSave) { failSave = false; throw new Error('Test-only failed save'); }
  }
  return backend.invoke(command, args);
}
const backend = load(path.join(__dirname, 'core.ts'));
const { useApp, restoreLastPlace, rowExportNotebook, cleanupLegacyDatabaseViews } = load(path.join(root, 'src/store/app.ts'));
const { startLastPlacePersistence, startNavHistory, goBack, goForward } = load(path.join(root, 'src/lib/navHistory.ts'));
const { flushAll } = load(path.join(root, 'src/lib/flush.ts'));
let checks = 0;
async function check(name, test) { await test(); checks++; console.log('ok ', name); }

(async () => {
  await check('startup removes all legacy dbview: keys and preserves unrelated keys byte-for-byte', async () => {
    assert.deepEqual(browserKeys, unrelatedKeys);
    cleanupLegacyDatabaseViews(); // Reloading does not alter any remaining key.
    assert.deepEqual(browserKeys, unrelatedKeys);
  });
  await check('unavailable storage does not prevent startup cleanup', async () => {
    Object.defineProperty(global, 'localStorage', { configurable: true, get() { throw new Error('Test-only blocked storage'); } });
    try { assert.doesNotThrow(cleanupLegacyDatabaseViews); }
    finally { Object.defineProperty(global, 'localStorage', { configurable: true, writable: true, value: browserStorage }); }
    let attempted = false;
    const lockedStorage = { ...browserStorage, get length() { return browserKeys.size; }, removeItem: () => { attempted = true; throw new Error('Test-only denied removal'); } };
    browserKeys.set('dbview:blocked', 'old-view');
    global.localStorage = lockedStorage;
    try { assert.doesNotThrow(cleanupLegacyDatabaseViews); }
    finally { global.localStorage = browserStorage; cleanupLegacyDatabaseViews(); }
    assert(attempted, 'The removal failure was exercised');
    assert.deepEqual(browserKeys, unrelatedKeys);
  });
  const tree = await invoke('list_notes');
  const tasks = tree.databases.find((d) => d.title === 'Tasks');
  const db = await invoke('get_database', { id: tasks.id });
  const rows = await invoke('query_view', { request: { databaseId: tasks.id, viewId: db.views[0].id } });
  const rowId = rows.rows[0].id;
  await check('old settings without lastPlace open Home', async () => {
    assert.equal((await restoreLastPlace(undefined, tree)).activePageId, null);
  });
  await check('page and nested page restore with their notebook and section', async () => {
    for (const id of ['p1', 'p4']) {
      const state = await restoreLastPlace({ kind: 'page', id }, tree);
      assert.equal(state.activePageId, id);
      assert.equal(state.activeNotebookId, 'n1');
      assert.equal(state.activeSectionId, 's1');
    }
  });
  await check('database restores with its parents', async () => {
    const state = await restoreLastPlace({ kind: 'database', id: tasks.id }, tree);
    assert.equal(state.activePageId, tasks.id);
    assert.equal(state.activeKind, 'database');
    assert.equal(state.activeSectionId, tasks.sectionId);
  });
  await check('row restores as a row, with its containing notebook', async () => {
    const state = await restoreLastPlace({ kind: 'row', id: rowId }, tree);
    assert.equal(state.activeKind, 'row');
    assert.equal(state.activePageId, rowId);
    assert.equal(state.activeSectionId, tasks.sectionId);
  });
  await check('search-to-row across notebooks exports the row database notebook', async () => {
    const row = await invoke('get_page', { id: rowId });
    const hit = (await invoke('search', { query: row.title })).find((h) => h.kind === 'row' && h.id === rowId);
    assert(hit, 'Row is reachable through search');
    useApp.setState({ tree });
    useApp.getState().openNotebook('n2');
    useApp.getState().openRow(hit.id); // The same navigation as CommandPalette's row result.
    assert.equal(useApp.getState().activeNotebookId, 'n2', 'Previously selected notebook stays selected');
    const notebook = await rowExportNotebook(useApp.getState().tree, useApp.getState().activePageId);
    assert.equal(notebook.id, 'n1');
    assert.notEqual(notebook.id, useApp.getState().activeNotebookId);
  });
  await check('row export has no fallback for missing or binned row/database or missing parents', async () => {
    assert.equal(await rowExportNotebook(tree, 'gone'), null);
    for (const mutate of [
      (t) => { t.databases = t.databases.filter((d) => d.id !== tasks.id); },
      (t) => { t.databases.find((d) => d.id === tasks.id).trashedAt = 1; },
      (t) => { t.databases.find((d) => d.id === tasks.id).sectionId = 's3'; },
      (t) => { t.sections = []; },
      (t) => { t.notebooks = []; },
    ]) {
      const changed = structuredClone(tree); mutate(changed);
      assert.equal(await rowExportNotebook(changed, rowId), null);
    }
    const row = await invoke('get_page', { id: rowId });
    hiddenPages.set(rowId, { ...row, trashedAt: 1 });
    try { assert.equal(await rowExportNotebook(tree, rowId), null); }
    finally { hiddenPages.delete(rowId); }
  });
  await check('live subpages beneath a row restore, including a nested descendant', async () => {
    const row = await invoke('get_page', { id: rowId });
    hiddenPages.set('row-child', { ...row, id: 'row-child', parent: rowId });
    hiddenPages.set('row-grandchild', { ...row, id: 'row-grandchild', parent: 'row-child' });
    for (const id of ['row-child', 'row-grandchild']) {
      assert(!tree.pages.some((p) => p.id === id));
      const state = await restoreLastPlace({ kind: 'page', id }, tree);
      assert.equal(state.activePageId, id);
      assert.equal(state.activeKind, 'page');
      assert.equal(state.activeNotebookId, tree.sections.find((s) => s.id === tasks.sectionId).notebookId);
      assert.equal(state.activeSectionId, tasks.sectionId);
    }
  });
  await check('row descendants reject missing, binned, cross-section and cyclic ancestors', async () => {
    const child = structuredClone(hiddenPages.get('row-child'));
    const row = await invoke('get_page', { id: rowId });
    try {
      for (const patch of [{ parent: 'gone' }, { trashedAt: 1 }, { sectionId: 's3' }, { parent: 'row-grandchild' }]) {
        hiddenPages.set('row-child', { ...child, ...patch });
        assert.equal((await restoreLastPlace({ kind: 'page', id: 'row-grandchild' }, tree)).activePageId, null);
      }
      hiddenPages.set('row-child', child);
      for (const patch of [{ trashedAt: 1 }, { parent: 'gone' }, { parent: 'row-grandchild' }]) {
        hiddenPages.set(rowId, { ...row, ...patch });
        assert.equal((await restoreLastPlace({ kind: 'page', id: 'row-grandchild' }, tree)).activePageId, null);
      }
      hiddenPages.delete(rowId);
      for (const patch of [{ trashedAt: 1 }, { parent: 'gone' }, { parent: 'row-grandchild' }]) {
        const changed = structuredClone(tree);
        Object.assign(changed.databases.find((d) => d.id === tasks.id), patch);
        assert.equal((await restoreLastPlace({ kind: 'page', id: 'row-grandchild' }, changed)).activePageId, null);
      }
      const changed = structuredClone(tree);
      changed.databases = changed.databases.filter((d) => d.id !== tasks.id);
      assert.equal((await restoreLastPlace({ kind: 'page', id: 'row-grandchild' }, changed)).activePageId, null);
    } finally {
      hiddenPages.set('row-child', child);
      hiddenPages.delete(rowId);
    }
  });
  await check('saved database view resolves; a deleted view falls back to Home', async () => {
    const selected = db.views[1];
    const state = await restoreLastPlace({ kind: 'database', id: tasks.id, viewId: selected.id }, tree);
    assert.equal(state.activeDatabaseViewId, selected.id);
    assert.equal((await restoreLastPlace({ kind: 'database', id: tasks.id, viewId: 'gone' }, tree)).activePageId, null);
  });
  await check('missing, trashed, orphaned and cyclic locations fall back to Home', async () => {
    for (const place of [{ kind: 'page', id: 'gone' }, { kind: 'page', id: 'p5' }, { kind: 'database', id: 'gone' }, { kind: 'settings', id: 'p1' }]) {
      assert.equal((await restoreLastPlace(place, tree)).activePageId, null);
    }
    for (const mutate of [
      (t) => { t.databases.find((d) => d.id === tasks.id).trashedAt = 1; },
      (t) => { t.sections = []; },
      (t) => { t.notebooks = []; },
    ]) {
      const changed = structuredClone(tree); mutate(changed);
      assert.equal((await restoreLastPlace({ kind: 'database', id: tasks.id }, changed)).activePageId, null);
      assert.equal((await restoreLastPlace({ kind: 'row', id: rowId }, changed)).activePageId, null);
    }
    const changed = structuredClone(tree);
    changed.pages.find((p) => p.id === 'p1').parent = 'p4';
    assert.equal((await restoreLastPlace({ kind: 'page', id: 'p4' }, changed)).activePageId, null);
  });
  await invoke('set_last_place', { lastPlace: { kind: 'page', id: 'p4' } });
  useApp.setState({ status: { unlocked: true }, settings: null });
  const stop = startLastPlacePersistence();
  await check('initial load restores before any debounced save can overwrite it', async () => {
    saves.length = 0;
    await useApp.getState().loadVault();
    await pause(500);
    assert.equal(useApp.getState().activePageId, 'p4');
    assert.equal(saves.length, 0);
  });
  await check('rapid navigation saves only the final place', async () => {
    useApp.getState().openPage('p1');
    useApp.getState().openPage('p2');
    await pause(500);
    assert.deepEqual(saves, [{ kind: 'page', id: 'p2' }]);
  });
  await check('Settings and Sticky notes keep the note; Home clears it', async () => {
    useApp.getState().setView('settings');
    await pause(450);
    useApp.getState().setView('stickies');
    await pause(450);
    assert.equal(saves.length, 1);
    useApp.setState({ view: 'notes', activePageId: null });
    await flushAll();
    assert.equal((await invoke('get_settings')).lastPlace, null);
  });
  await check('lock/quit flush saves the latest place before the debounce', async () => {
    useApp.getState().openPage('p1');
    useApp.getState().openPage('p4');
    await flushAll();
    assert.deepEqual((await invoke('get_settings')).lastPlace, { kind: 'page', id: 'p4' });
  });
  await check('appearance saves cannot overwrite lastPlace', async () => {
    await invoke('save_settings', { settings: { theme: 'paper', lastPlace: null } });
    assert.deepEqual((await invoke('get_settings')).lastPlace, { kind: 'page', id: 'p4' });
  });
  await check('back/forward changes are included in persisted navigation', async () => {
    const stopHistory = startNavHistory();
    useApp.getState().openPage('p2');
    goBack(); await flushAll();
    assert.equal((await invoke('get_settings')).lastPlace.id, 'p4');
    goForward(); await flushAll();
    assert.equal((await invoke('get_settings')).lastPlace.id, 'p2');
    stopHistory();
  });
  await check('a failed save is retried by the lock flush, without note data in the error', async () => {
    failSave = true;
    useApp.getState().openPage('p3');
    await pause(500);
    await flushAll();
    assert.equal((await invoke('get_settings')).lastPlace.id, 'p3');
    assert.deepEqual(notices, ["Couldn't remember the last open place."]);
  });
  await check('database view changes persist and old database callbacks are ignored', async () => {
    useApp.getState().openDatabase(tasks.id);
    useApp.getState().setDatabaseView(tasks.id, db.views[1].id);
    useApp.getState().setDatabaseView('another-database', 'another-view');
    await flushAll();
    assert.deepEqual((await invoke('get_settings')).lastPlace, { kind: 'database', id: tasks.id, viewId: db.views[1].id });
    useApp.getState().openPage('p3');
    await flushAll();
  });
  stop();
  await check('reload/unlock loads the stored location again', async () => {
    useApp.setState({ settings: null, activePageId: null, activeNotebookId: null, activeSectionId: null });
    await useApp.getState().loadVault();
    assert.equal(useApp.getState().activePageId, 'p3');
  });
  await check('page/database mixed reordering persists and leaves parents unchanged', async () => {
    const ids = ['p2', tasks.id, 'p1', 'p3'];
    const result = await invoke('reorder', { ids });
    const items = [...result.pages, ...result.databases].filter((x) => ids.includes(x.id)).sort((a, b) => a.order - b.order);
    assert.deepEqual(items.map((x) => x.id), ids);
    const updated = result.databases.find((d) => d.id === tasks.id);
    assert.equal(updated.sectionId, tasks.sectionId);
    assert.equal(updated.parent, tasks.parent);
  });
  console.log(`ALL OK (${checks} checks; logic only, no browser or real vault)`);
})().catch((error) => { console.error(error); process.exitCode = 1; });
