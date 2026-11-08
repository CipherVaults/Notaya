// Legacy hostile keys must survive the actual row-to-template consumer as own
// JSON properties. No app, vault, network or new dependency is involved.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
const ts = require('typescript');
const source = fs.readFileSync('src/features/database/rowOps.ts', 'utf8');
const js = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2020 } }).outputText;
const db = { props: ['__proto__', 'constructor', 'prototype', 'old-id'].map((id) => ({ id, kind: 'text' })), templates: [] };
let saved;
const exportsObject = {};
vm.runInNewContext(js, {
  exports: exportsObject,
  require(name) {
    if (name === 'sonner') return { toast: { success() {} } };
    if (name === '@/lib/ipc') return {};
    if (name === './dbStore') return { peekDatabase: () => db, editDatabase: async (_id, update) => { saved = update(db); } };
    if (name === './model') return { newId: () => 'template-id', isComputedKind: () => false };
    throw new Error('Unexpected test dependency');
  },
});
(async () => {
  const values = JSON.parse('{"__proto__":"\u200f\u0639\u05d0  ","constructor":"ctor","prototype":"proto","old-id":"e\u0301\u200e  "}');
  await exportsObject.saveRowAsTemplate('db', { values, icon: '' }, 'Template', '', '');
  const cells = JSON.parse(saved.templates[0].cells);
  for (const key of Object.keys(values)) {
    assert(Object.hasOwn(cells, key));
    assert.equal(cells[key], values[key]);
  }
  console.log('PASS legacy_template_keys_are_own_properties_with_exact_values (4 keys)');
})().catch((error) => { console.error(error); process.exitCode = 1; });
