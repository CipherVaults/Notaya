// Browser-free IPC/mock contract checks. No vault, installs or network.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const ts = require('typescript');
const root = path.resolve(__dirname, '..');
const cache = new Map();
global.location = { search: '?db=1' };
global.window = new EventTarget();
function load(file) {
  file = path.resolve(file); if (!path.extname(file)) file += '.ts';
  if (cache.has(file)) return cache.get(file).exports;
  const module = { exports: {} }; cache.set(file, module);
  const localRequire = (name) => {
    if (name === '@tauri-apps/api/core') return backend;
    if (name.startsWith('.')) return load(path.resolve(path.dirname(file), name));
    return require(name);
  };
  const code = ts.transpileModule(fs.readFileSync(file, 'utf8'), { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 }, fileName: file }).outputText;
  vm.runInThisContext(`(function(require,module,exports){${code}\n})`, { filename: file })(localRequire, module, module.exports);
  return module.exports;
}
const backend = load(path.join(__dirname, 'core.ts'));
const { api } = load(path.join(root, 'src/lib/ipc.ts'));
const events = load(path.join(__dirname, 'event.ts'));
(async () => {
  const signals = [];
  await events.listen('items-changed', (event) => signals.push(event.payload));
  const page = await api.getPage('p1');
  const original = await api.historyList('p1');
  assert.equal(original.length, 2);
  assert.equal((await api.historyGet('p1', original[0].versionId)).title, page.title);
  await assert.rejects(api.historyGet('p1', 'missing'));
  console.log('ok history lists metadata and returns only the requested version body');
  await api.savePage('p1', 'Saved after opening history', page.content, 'new body');
  await api.historySnapshot('p1');
  const versions = await api.historyList('p1');
  assert.equal(versions.length, 3);
  await api.historyRestore('p1', original[0].versionId);
  assert.equal((await api.getPage('p1')).title, page.title);
  assert.ok(signals.some((signal) => signal.ids.includes('p1')));
  assert.equal((await api.historyGet('p1', versions[0].versionId)).text, 'new body');
  console.log('ok snapshot and restore preserve the current version and publish a page signal');
  assert.equal((await api.activityList('person')).length, 1);
  await api.activityVerify();
  await assert.rejects(api.activityUndo('mock-person-change'));
  assert.deepEqual(await api.activityUndo('mock-ai-change'), { restored: 1, skipped: 0 });
  console.log('ok activity filters, verifies and undoes an AI change set');
  const historyBeforeClear = (await api.historyList('p1')).length;
  await assert.rejects(api.activityClear('clear'));
  await api.activityClear('CLEAR');
  const remaining = await api.activityList();
  assert.equal(remaining.length, 1); assert.equal(remaining[0].action, 'log.cleared');
  await api.activityVerify(); assert.equal((await api.historyList('p1')).length, historyBeforeClear);
  console.log('ok typed clear leaves an anchor and preserves page history');
  console.log('4 C3 mock/IPC checks passed (browser-free)');
})().catch((error) => { console.error(error); process.exitCode = 1; });
