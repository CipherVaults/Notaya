// Pure fixtures only. No real TypeScript/Vite import, browser, server, build or app.
const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { ALTERNATE_RUNS, inside, plainExisting } = require('./baseline-server.cjs');
const { layout, InputReads, equalManifests, resolutionImporter, installResolution, typecheck, buildOptions, observeModuleGraph, outputManifest } = require('./baseline-build.cjs');
const parent = process.env.NOTAYA_BASELINE_BUILD_FIXTURE_ROOT;
assert.ok(parent && path.isAbsolute(parent) && inside(parent, ALTERNATE_RUNS), 'Explicit owned G fixture parent required');
plainExisting(parent);
const fixture = fs.mkdtempSync(path.join(parent, 'pure-fixture-')); // Retain every fixture, including failed checks.
const root = path.join(fixture, 'main'), modules = path.join(fixture, 'p6c', 'node_modules');
fs.mkdirSync(root); fs.mkdirSync(modules, { recursive: true });
const manifest = { dependencies: { react: '19.3.1', '@tauri-apps/api': '2.12.4' }, devDependencies: { vite: '8.3.5' } };

test('build layout refuses another source and a D drive output', () => {
  assert.throws(() => layout({ root, modules, run: fixture }), /independent Main/);
  const main = path.resolve(__dirname, '../..'), sdk = path.resolve(main, '../p6c-evidence/node_modules');
  assert.throws(() => layout({ root: main, modules: sdk, run: 'D:\\not-admitted-build' }), /G drive/);
});
test('bare source modules use the SDK anchor while relative and @ paths stay local', () => {
  const source = path.join(root, 'src', 'main.tsx'), anchor = path.resolve(modules, '../package.json');
  for (const id of ['react', 'react/jsx-runtime', '@tauri-apps/api/core', 'vite/client']) assert.equal(resolutionImporter(id, source, modules), anchor);
  for (const id of ['./App', '../styles/app.css', '@/App', '#private']) assert.equal(resolutionImporter(id, source, modules), source);
  const nested = path.join(modules, 'outer', 'node_modules', 'inner', 'index.d.ts');
  assert.equal(resolutionImporter('react', nested, modules), nested);
});
test('TypeScript delegation preserves options, redirected reference and explicit modes', () => {
  const calls = [], host = {}, options = { moduleResolution: 'bundler' }, redirected = {}, source = { impliedNodeFormat: 99 };
  const ts = { getModeForUsageLocation: () => 99, getModeForFileReference: (reference, mode) => reference.resolutionMode ?? mode,
    resolveModuleName(...args) { calls.push(args); return { resolvedModule: { resolvedFileName: 'synthetic' } }; },
    resolveTypeReferenceDirective(...args) { calls.push(args); return { resolvedTypeReferenceDirective: { resolvedFileName: 'synthetic' } }; } };
  installResolution(ts, host, modules);
  assert.equal(host.resolveModuleNameLiterals([{ text: 'react/jsx-runtime' }], path.join(root, 'src', 'x.tsx'), redirected, options, source).length, 1);
  assert.equal(calls[0][0], 'react/jsx-runtime'); assert.equal(calls[0][2], options); assert.equal(calls[0][3], host);
  assert.equal(calls[0][5], redirected); assert.equal(calls[0][6], 99);
  host.resolveTypeReferenceDirectiveReferences([{ fileName: 'vite/client', resolutionMode: 1 }], path.join(root, 'src', 'vite-env.d.ts'), redirected, options, source);
  assert.equal(calls[1][0], 'vite/client'); assert.equal(calls[1][1], path.resolve(modules, '../package.json'));
  assert.equal(calls[1][2], options); assert.equal(calls[1][4], redirected); assert.equal(calls[1][6], 1);
});
test('production build keeps actual plugins/options and has no mock Tauri aliases', async () => {
  const plugins = [{ name: 'actual-react' }, { name: 'actual-tailwind' }], cache = path.join(fixture, 'cache'), dist = path.join(fixture, 'dist');
  const options = buildOptions({ root, modules }, manifest, plugins, cache, dist);
  assert.equal(options.configFile, false); assert.equal(options.envDir, false); assert.equal(options.plugins, plugins);
  assert.deepEqual(options.build, { outDir: dist, emptyOutDir: false, target: 'chrome120', sourcemap: false, chunkSizeWarningLimit: 2000, watch: null });
  assert.deepEqual(options.resolve.alias.map(alias => alias.find), ['@', 'react', '@tauri-apps/api', 'vite']);
  const alias = options.resolve.alias.find(item => item.find === '@tauri-apps/api');
  const calls = []; const condition = { custom: 'browser', isEntry: false };
  await alias.customResolver.call({ resolve(...args) { calls.push(args); } }, '@tauri-apps/api/core', path.join(root, 'src', 'x.ts'), condition);
  assert.deepEqual(calls[0], ['@tauri-apps/api/core', path.resolve(modules, '../package.json'), { ...condition, skipSelf: true }]);
  // The actual Rolldown context provides no getWatchFiles. Exercise only its
  // supported graph methods, including a virtual module without reading it.
  const ids = [path.join(root, 'src', 'x.ts'), '\0synthetic-runtime'], observed = [];
  const context = { getModuleIds: () => ids.values(), getModuleInfo: id => ({ id }) };
  assert.equal(observeModuleGraph(context, id => observed.push(id)), 2); assert.deepEqual(observed, ids);
  assert.throws(() => observeModuleGraph({}, () => {}), /Missing supported module graph API/);
  assert.throws(() => observeModuleGraph({ ...context, getModuleIds: () => [].values() }, () => {}), /zero module graph/);
  assert.throws(() => observeModuleGraph({ ...context, getModuleInfo: () => null }, () => {}), /Missing or mismatched/);
  assert.throws(() => observeModuleGraph({ ...context, getModuleInfo: () => ({ id: 'wrong' }) }, () => {}), /Missing or mismatched/);
});
test('typecheck retains configuration and references, forces no emit, and rejects diagnostics or empty roots', () => {
  const references = [{ path: path.join(root, 'tsconfig.node.json') }];
  const parsed = { fileNames: [path.join(root, 'src', 'x.ts')], options: { strict: true, paths: { '@/*': ['./src/*'] } }, projectReferences: references, errors: [] };
  let captured, diagnostics = [];
  const ts = { version: '6.0.3', sys: {}, getParsedCommandLineOfConfigFile: () => parsed,
    createCompilerHost: () => ({}), createProgram(options) { captured = options; return { getSourceFiles: () => [{ fileName: parsed.fileNames[0] }] }; },
    getPreEmitDiagnostics: () => diagnostics, formatDiagnosticsWithColorAndContext: () => 'planted compiler failure' };
  const reads = new InputReads(root, modules), declarations = { devDependencies: { typescript: '6.0.3' } };
  typecheck(ts, { root, modules }, reads, declarations);
  assert.equal(captured.projectReferences, references); assert.equal(captured.options.strict, true);
  assert.equal(captured.options.paths, parsed.options.paths); assert.equal(captured.options.noEmit, true);
  assert.deepEqual(captured.options.typeRoots, [path.join(modules, '@types')]);
  assert.throws(() => captured.host.writeFile('anything'), /writes are forbidden/);
  diagnostics = [{ category: 1 }]; assert.throws(() => typecheck(ts, { root, modules }, reads, declarations), /planted compiler failure/);
  parsed.fileNames = []; assert.throws(() => typecheck(ts, { root, modules }, reads, declarations), /zero source files/);
});
test('Main and SDK locks must match actual bytes', () => {
  for (const directory of [root, path.dirname(modules)]) {
    fs.writeFileSync(path.join(directory, 'package.json'), JSON.stringify(manifest)); fs.writeFileSync(path.join(directory, 'package-lock.json'), '{"lockfileVersion":3}');
  }
  assert.deepEqual(equalManifests(new InputReads(root, modules), { root, modules }), manifest);
  fs.writeFileSync(path.join(root, 'package-lock.json'), '{"lockfileVersion":2}');
  assert.throws(() => equalManifests(new InputReads(root, modules), { root, modules }), /byte-identical/);
});
test('read set rejects outside input and notices a changed admitted input', () => {
  const reads = new InputReads(root, modules), file = path.join(root, 'source.ts'); fs.writeFileSync(file, 'before'); reads.bytes(file, 'fixture');
  assert.throws(() => reads.bytes(path.join(fixture, 'outside.ts'), 'fixture'), /outside Main/);
  assert.equal(reads.existing(path.join(fixture, 'outside.ts')), false);
  assert.equal(reads.allowed(path.join(root, '.env.local')), false);
  fs.writeFileSync(file, 'after'); assert.throws(() => reads.verify(), /changed during build/);
});
test('empty, incomplete and zero-byte frontend selections fail; complete output has hashes', () => {
  const dist = path.join(fixture, 'output-proof'); fs.mkdirSync(dist);
  assert.throws(() => outputManifest(dist), /requires nonempty/);
  fs.writeFileSync(path.join(dist, 'index.html'), '<main>fixture</main>'); fs.writeFileSync(path.join(dist, 'app.js'), 'void 0;');
  assert.throws(() => outputManifest(dist), /requires nonempty/);
  fs.writeFileSync(path.join(dist, 'app.css'), ''); assert.throws(() => outputManifest(dist), /Empty frontend/);
  fs.writeFileSync(path.join(dist, 'app.css'), 'main{}'); const rows = outputManifest(dist);
  assert.equal(rows.length, 3); assert.ok(rows.every(row => /^[a-f0-9]{64}$/.test(row.sha256) && row.bytes > 0));
});
