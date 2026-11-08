// No browser, disk vault, installs or network. Exercise the production stores and
// signal listener against the in-memory mock, including races during whole saves.
// Run: node dev-mock/c2-check.cjs
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const ts = require('typescript');
const root = path.resolve(__dirname, '..');
const cache = new Map();
global.location = { search: '?db=1' };
global.window = new EventTarget();
const pause = (ms = 0) => new Promise((resolve) => setTimeout(resolve, ms));
let beforeInvoke = async () => {};
const saves = [];
const notices = [];

function load(file) {
  file = path.resolve(file);
  if (!path.extname(file)) file += '.ts';
  if (cache.has(file)) return cache.get(file).exports;
  const module = { exports: {} };
  cache.set(file, module);
  const localRequire = (name) => {
    if (name === 'sonner') return { toast: { error: (text) => notices.push(text) } };
    if (name === '@tauri-apps/api/core') return { invoke: async (cmd, args) => {
      await beforeInvoke(cmd, args);
      if (cmd === 'save_database') saves.push(structuredClone(args));
      return backend.invoke(cmd, args);
    } };
    if (name === '@tauri-apps/api/event') return load(path.join(__dirname, 'event.ts'));
    if (name === '@/lib/storage') return { getKey: () => null, setKey: () => {} };
    if (name.startsWith('@/')) return load(path.join(root, 'src', name.slice(2)));
    if (name.startsWith('.')) return load(path.resolve(path.dirname(file), name));
    return require(name);
  };
  const code = ts.transpileModule(fs.readFileSync(file, 'utf8'), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 }, fileName: file,
  }).outputText;
  vm.runInThisContext(`(function(require,module,exports){${code}\n})`, { filename: file })(localRequire, module, module.exports);
  return module.exports;
}
const backend = load(path.join(__dirname, 'core.ts'));
const { emit } = load(path.join(__dirname, 'event.ts'));
const { watchItemChanges, canReload } = load(path.join(root, 'src/lib/changes.ts'));
const store = load(path.join(root, 'src/features/database/dbStore.ts'));
const { useApp } = load(path.join(root, 'src/store/app.ts'));
const { api } = load(path.join(root, 'src/lib/ipc.ts'));
const { flushAll } = load(path.join(root, 'src/lib/flush.ts'));
let checks = 0;
async function check(name, test) { await test(); checks++; console.log('ok', name); }
const signal = (id, rev, kind = 'page') => emit('items-changed', { ids: [id], kinds: [kind], tree: true, by: 'ai:fixture', revs: { [id]: rev } });
const eventually = async (fn) => { for (let n = 0; n < 100; n++) { if (fn()) return; await pause(10); } assert.ok(fn(), 'condition did not settle'); };

(async () => {
  await check('clean reload, dirty typing, own revision and queued save signals', async () => {
    let rev = 1, busy = false, reads = 0, remote = 1;
    const watch = watchItemChanges('fixture', () => rev, () => busy, async (alive) => { await pause(); if (alive()) { rev = remote; reads++; } });
    assert.equal(canReload('fixture', 1, true, { ids: ['fixture'], revs: { fixture: 2 } }), false);
    remote = 2; await signal('fixture', 2); await eventually(() => rev === 2);
    busy = true; remote = 3; await signal('fixture', 3); await pause(); assert.equal(reads, 1);
    // A save response acknowledges this same revision: its own event is ignored.
    rev = 3; busy = false; watch.recheck(); await pause(); assert.equal(reads, 1);
    busy = true; remote = 4; await signal('fixture', 4); rev = 3; busy = false; watch.recheck();
    await eventually(() => rev === 4);
    watch.dispose(); remote = 5; await signal('fixture', 5); await pause(); assert.equal(reads, 2);
  });

  const tree = await api.listNotes();
  const dbId = tree.databases.find((db) => db.title === 'Tasks').id;
  await store.loadDatabase(dbId);
  await check('database clean signals reload, dirty local edits survive and sends base_rev', async () => {
    let db = await api.getDatabase(dbId);
    await api.saveDatabase({ ...db, title: 'Remote definition' }, db.rev);
    await eventually(() => store.peekDatabase(dbId).title === 'Remote definition');
    await store.editDatabase(dbId, (current) => ({ ...current, title: 'Local unsaved definition' }));
    db = await api.getDatabase(dbId);
    await api.saveDatabase({ ...db, title: 'Remote while typing' }, db.rev);
    await pause(80);
    assert.equal(store.peekDatabase(dbId).title, 'Local unsaved definition');
    await store.saveNow(dbId);
    assert.equal((await api.getDatabase(dbId)).title, 'Local unsaved definition');
    assert.equal(typeof saves.at(-1).baseRev, 'number');
  });

  await check('database save chain, failed-save retry and P16 flush keep the final edit', async () => {
    let release, entered;
    const blocked = new Promise((resolve) => { release = resolve; });
    const started = new Promise((resolve) => { entered = resolve; });
    let first = true;
    beforeInvoke = async (cmd) => { if (cmd === 'save_database' && first) { first = false; entered(); await blocked; } };
    const firstSave = store.editDatabase(dbId, (db) => ({ ...db, title: 'First save' }), true);
    await started;
    const lastSave = store.editDatabase(dbId, (db) => ({ ...db, title: 'Last save' }), true);
    const flushing = flushAll();
    release(); await Promise.all([firstSave, lastSave, flushing]);
    assert.equal((await api.getDatabase(dbId)).title, 'Last save');
    assert.equal(store.hasUnsaved(dbId), false);
    let fail = true;
    beforeInvoke = async (cmd) => { if (cmd === 'save_database' && fail) { fail = false; throw new Error('fixture failure'); } };
    await store.editDatabase(dbId, (db) => ({ ...db, title: 'Retry me' }), true);
    assert.equal(store.hasUnsaved(dbId), true);
    await flushAll();
    assert.equal((await api.getDatabase(dbId)).title, 'Retry me');
    assert.equal(store.hasUnsaved(dbId), false);
    assert.equal(notices.length, 1);
    beforeInvoke = async () => {};
  });

  await check('bin groups include every kind and containers hide children from tree/search', async () => {
    await useApp.getState().refreshStatus();
    await useApp.getState().loadVault();
    const db = await api.getDatabase(dbId);
    const rows = await api.queryView({ databaseId: dbId, viewId: db.views[0].id });
    assert.ok(rows.rows.length);
    useApp.getState().openRow(rows.rows[0].id);
    await api.trashPage('p1');
    await api.trashPage(dbId);
    await eventually(() => useApp.getState().activePageId === null);
    if (rows.rows[0]) await api.trashPage(rows.rows[0].id);
    await api.deleteSticky('k1');
    await api.deleteSection('s1');
    await api.deleteNotebook('n1');
    const kinds = new Set((await api.listBin()).map((item) => item.kind));
    for (const kind of ['notebook', 'section', 'page', 'database', 'row', 'sticky']) assert.ok(kinds.has(kind), kind);
    const live = await api.listNotes();
    assert.ok(!live.notebooks.some((item) => item.id === 'n1'));
    assert.ok(!live.pages.some((item) => item.id === 'p1' && item.trashedAt == null));
    assert.ok(!live.databases.some((item) => item.id === dbId && item.trashedAt == null));
    assert.ok(!(await api.search('Morning')).some((item) => item.id === 'p1'));
    assert.ok(!(await api.listStickies()).some((item) => item.id === 'k1'));
    await eventually(() => !useApp.getState().tree.notebooks.some((item) => item.id === 'n1'));
  });

  await check('section restore requires containing-notebook confirmation; empty removes children', async () => {
    await assert.rejects(api.restoreBinItem('s1', false), /parent_in_bin/);
    await api.restoreBinItem('s1', true);
    assert.ok((await api.listNotes()).notebooks.some((item) => item.id === 'n1'));
    await api.restoreBinItem('k1', false);
    assert.equal((await api.getSticky('k1')).open, false);
    await api.deleteNotebook('n1');
    await api.deleteSticky('k1');
    await api.emptyTrash();
    assert.equal((await api.listBin()).length, 0);
    assert.ok(!(await api.listNotes()).sections.some((item) => item.id === 's1'));
    await assert.rejects(api.getSticky('k1'));
  });
  console.log(`${checks} C2 checks passed`);
})().catch((error) => { console.error(error); process.exitCode = 1; });

