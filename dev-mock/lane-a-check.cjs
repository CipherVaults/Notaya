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
let failSave = false;
global.location = { search: '?db=1' };
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
  if (command === 'set_last_place') {
    saves.push(structuredClone(args.lastPlace));
    if (failSave) { failSave = false; throw new Error('Test-only failed save'); }
  }
  return backend.invoke(command, args);
}
const backend = load(path.join(__dirname, 'core.ts'));
const { useApp, restoreLastPlace } = load(path.join(root, 'src/store/app.ts'));
const { startLastPlacePersistence, startNavHistory, goBack, goForward } = load(path.join(root, 'src/lib/navHistory.ts'));
const { flushAll } = load(path.join(root, 'src/lib/flush.ts'));
let checks = 0;
async function check(name, test) { await test(); checks++; console.log('ok ', name); }

(async () => {
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
