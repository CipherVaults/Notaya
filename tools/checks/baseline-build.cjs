// Production frontend qualification using the existing public SDK, with owned outputs.
// Path checks and observed read sets are operational guards, not an OS sandbox.
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const { pathToFileURL } = require('node:url');
const { ALTERNATE_RUNS, inside, same, allowedRun, plainExisting, ownedEnvironment,
  dependencyAliases, need } = require('./baseline-server.cjs');

const digest = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
function layout({ root, modules, run }) {
  for (const value of [root, modules, run]) need(typeof value === 'string' && path.isAbsolute(value), 'Explicit absolute paths are required');
  need(same(root, path.resolve(__dirname, '../..')), 'Root must be this independent Main checkout');
  need(same(modules, path.resolve(root, '../p6c-evidence/node_modules')), 'Use the qualified sibling public SDK');
  need(allowedRun(run, root) && inside(run, ALTERNATE_RUNS), 'Build outputs require an approved G drive run descendant');
  for (const entry of [root, modules, run]) plainExisting(entry);
  return { root: path.resolve(root), modules: path.resolve(modules), run: path.resolve(run) };
}
class InputReads {
  constructor(root, modules) {
    this.root = root; this.modules = modules; this.rows = new Map();
    this.sdkMetadata = ['package.json', 'package-lock.json'].map(name => path.resolve(modules, '..', name));
  }
  allowed(file) {
    const resolved = path.resolve(file);
    return (same(resolved, this.root) || inside(resolved, this.root) || same(resolved, this.modules)
      || inside(resolved, this.modules) || this.sdkMetadata.some(meta => same(meta, resolved)))
      && !/(?:^|[\\/])(?:\.git|\.devdata)(?:[\\/]|$)|(?:^|[\\/])\.env(?:\.|$)/i.test(resolved);
  }
  existing(file, directory = false) {
    if (!this.allowed(file)) return false;
    try { plainExisting(file); return directory ? fs.statSync(file).isDirectory() : fs.statSync(file).isFile(); }
    catch (error) { if (error.code === 'ENOENT' || error.code === 'ENOTDIR') return false; throw error; }
  }
  bytes(file, reason) {
    need(this.allowed(file), 'Input outside Main/public SDK: ' + file);
    plainExisting(file); need(fs.statSync(file).isFile(), 'Expected ordinary input file: ' + file);
    const resolved = path.resolve(file), bytes = fs.readFileSync(resolved), sha256 = digest(bytes);
    const previous = this.rows.get(resolved.toLowerCase());
    if (previous) { need(previous.sha256 === sha256, 'Input changed during build: ' + resolved); previous.reasons.add(reason); }
    else this.rows.set(resolved.toLowerCase(), { path: resolved, bytes: bytes.length, sha256, reasons: new Set([reason]) });
    return bytes;
  }
  observe(file, reason) { if (this.existing(file)) this.bytes(file, reason); }
  verify() {
    for (const row of this.rows.values()) need(digest(this.bytes(row.path, 'final-readback')) === row.sha256, 'Input changed: ' + row.path);
  }
  manifest() { return [...this.rows.values()].map(row => ({ ...row, reasons: [...row.reasons] })).sort((a, b) => a.path.localeCompare(b.path)); }
}
function equalManifests(reads, config) {
  const mainPackage = reads.bytes(path.join(config.root, 'package.json'), 'manifest');
  const sdkPackage = reads.bytes(path.resolve(config.modules, '../package.json'), 'manifest');
  const mainLock = reads.bytes(path.join(config.root, 'package-lock.json'), 'lock');
  const sdkLock = reads.bytes(path.resolve(config.modules, '../package-lock.json'), 'lock');
  need(mainPackage.equals(sdkPackage) && mainLock.equals(sdkLock), 'Main and qualified SDK manifests/locks must be byte-identical');
  return JSON.parse(mainPackage.toString('utf8'));
}
function resolutionImporter(id, containingFile, modules) {
  const local = id.startsWith('.') || id.startsWith('@/') || id.startsWith('#') || path.isAbsolute(id);
  return local || inside(containingFile, modules) ? containingFile : path.resolve(modules, '../package.json');
}
function installResolution(ts, host, modules) {
  host.resolveModuleNameLiterals = (literals, containingFile, redirected, options, sourceFile) => literals.map(literal =>
    ts.resolveModuleName(literal.text, resolutionImporter(literal.text, containingFile, modules), options, host,
      undefined, redirected, sourceFile ? ts.getModeForUsageLocation(sourceFile, literal, options) : undefined));
  host.resolveTypeReferenceDirectiveReferences = (references, containingFile, redirected, options, sourceFile) => references.map(reference => {
    const name = typeof reference === 'string' ? reference : reference.fileName;
    return ts.resolveTypeReferenceDirective(name, resolutionImporter(name, containingFile, modules), options, host,
      redirected, undefined, ts.getModeForFileReference(reference, sourceFile?.impliedNodeFormat));
  });
}
function typecheck(ts, config, reads, manifest) {
  need(ts.version === manifest.devDependencies.typescript, 'Installed TypeScript version differs from the locked declaration');
  const io = {
    ...ts.sys, getCurrentDirectory: () => config.root,
    readFile: file => reads.existing(file) ? reads.bytes(file, 'typescript-readFile').toString('utf8') : undefined,
    fileExists: file => reads.existing(file), directoryExists: directory => reads.existing(directory, true),
    readDirectory: (directory, ...args) => reads.existing(directory, true) ? ts.sys.readDirectory(directory, ...args) : [],
    getDirectories: directory => reads.existing(directory, true) ? ts.sys.getDirectories(directory) : [],
    realpath: file => { need(reads.allowed(file), 'Unexpected TypeScript realpath: ' + file); plainExisting(file); return ts.sys.realpath(file); },
    writeFile() { throw new Error('TypeScript writes are forbidden'); },
  };
  const formatHost = { getCanonicalFileName: file => file, getCurrentDirectory: () => config.root, getNewLine: () => '\n' };
  const format = diagnostics => ts.formatDiagnosticsWithColorAndContext(diagnostics, formatHost);
  const parsed = ts.getParsedCommandLineOfConfigFile(path.join(config.root, 'tsconfig.json'), {}, {
    ...io, onUnRecoverableConfigFileDiagnostic(diagnostic) { throw new Error(format([diagnostic])); },
  });
  need(parsed && parsed.fileNames.length > 0, 'TypeScript selected zero source files');
  if (parsed.errors.length) throw new Error(format(parsed.errors));
  const options = { ...parsed.options, noEmit: true, typeRoots: [path.join(config.modules, '@types')] };
  const host = ts.createCompilerHost(options);
  for (const key of ['getCurrentDirectory', 'readFile', 'fileExists', 'directoryExists', 'readDirectory', 'getDirectories', 'realpath', 'writeFile']) host[key] = io[key];
  host.createDirectory = () => { throw new Error('TypeScript directory writes are forbidden'); };
  installResolution(ts, host, config.modules);
  const program = ts.createProgram({ rootNames: parsed.fileNames, options, host,
    projectReferences: parsed.projectReferences, configFileParsingDiagnostics: parsed.errors });
  const sources = program.getSourceFiles().map(source => source.fileName);
  need(sources.length > 0 && sources.every(file => reads.allowed(file)), 'Unexpected or empty TypeScript source set');
  const diagnostics = ts.getPreEmitDiagnostics(program);
  console.log('BASELINE_TYPESCRIPT ' + JSON.stringify({ version: ts.version, rootFiles: parsed.fileNames.length,
    sourceFiles: sources.length, projectReferences: parsed.projectReferences, diagnostics: diagnostics.length, noEmit: true }));
  if (diagnostics.length) throw new Error(format(diagnostics));
}
function buildOptions(config, manifest, plugins, cache, dist) {
  return { configFile: false, envDir: false, root: config.root, cacheDir: cache, clearScreen: false, plugins,
    resolve: { alias: [{ find: '@', replacement: path.join(config.root, 'src') }, ...dependencyAliases(manifest, config.modules)] },
    build: { outDir: dist, emptyOutDir: false, target: 'chrome120', sourcemap: false, chunkSizeWarningLimit: 2000, watch: null } };
}
function observeModuleGraph(context, observe) {
  // Rolldown's public PluginContext has these methods, but no getWatchFiles.
  need(typeof context.getModuleIds === 'function' && typeof context.getModuleInfo === 'function', 'Missing supported module graph API');
  let count = 0;
  for (const id of context.getModuleIds()) {
    const info = context.getModuleInfo(id);
    need(typeof id === 'string' && info && info.id === id, 'Missing or mismatched module graph information');
    observe(info.id); count++;
  }
  need(count > 0, 'Vite selected zero module graph entries');
  return count;
}
function outputManifest(dist) {
  plainExisting(dist); const rows = [], directories = [dist];
  while (directories.length) for (const entry of fs.readdirSync(directories.pop(), { withFileTypes: true })) {
    const file = path.join(entry.parentPath, entry.name); plainExisting(file);
    if (entry.isDirectory()) directories.push(file);
    else { need(entry.isFile(), 'Unexpected frontend output type'); const bytes = fs.readFileSync(file);
      need(bytes.length > 0, 'Empty frontend output: ' + file); rows.push({ path: path.relative(dist, file), bytes: bytes.length, sha256: digest(bytes) }); }
  }
  need(rows.some(row => row.path === 'index.html') && rows.some(row => /\.js$/.test(row.path))
    && rows.some(row => /\.css$/.test(row.path)), 'Frontend output requires nonempty index.html, JavaScript and CSS');
  return rows.sort((a, b) => a.path.localeCompare(b.path));
}
async function runBuild(options) {
  const config = layout(options); ownedEnvironment(config.run);
  const reads = new InputReads(config.root, config.modules);
  let failure, outputs;
  try {
    reads.bytes(__filename, 'controller'); reads.bytes(require.resolve('./baseline-server.cjs'), 'shared-helper');
    const manifest = equalManifests(reads, config);
    const cache = path.join(config.run, 'frontend-cache'), dist = path.join(config.run, 'frontend-dist');
    fs.mkdirSync(cache); fs.mkdirSync(dist); // Atomic no-overwrite: failed output is retained.
    const entries = { typescript: 'typescript/lib/typescript.js', vite: 'vite/dist/node/index.js',
      react: '@vitejs/plugin-react/dist/index.js', tailwind: '@tailwindcss/vite/dist/index.mjs' };
    for (const [name, relative] of Object.entries(entries)) {
      entries[name] = path.join(config.modules, relative); reads.bytes(entries[name], 'public-tool-entry');
    }
    typecheck(require(entries.typescript), config, reads, manifest);
    const vite = await import(pathToFileURL(entries.vite).href);
    const react = (await import(pathToFileURL(entries.react).href)).default;
    const tailwind = (await import(pathToFileURL(entries.tailwind).href)).default;
    const observe = file => {
      if (file.includes('\0')) return;
      const plain = file.split(/[?#]/, 1)[0];
      if (path.isAbsolute(plain)) { need(reads.allowed(plain), 'Unexpected Vite input: ' + plain); reads.observe(plain, 'vite-load-or-module-graph'); }
    };
    reads.bytes(path.join(config.root, 'index.html'), 'vite-html');
    const boundary = { name: 'notaya-owned-production-build-inputs', enforce: 'pre',
      configResolved(resolved) {
        need(resolved.configFile === undefined && resolved.configFileDependencies.length === 0 && resolved.envDir === false, 'Implicit Vite config/environment discovery');
        need(same(resolved.root, config.root) && same(resolved.cacheDir, cache) && same(resolved.build.outDir, dist)
          && resolved.build.watch === null && resolved.build.emptyOutDir === false, 'Unexpected resolved build paths/options');
      },
      load(id) { observe(id); return null; },
      generateBundle() {
        console.log('BASELINE_VITE_MODULE_GRAPH ' + JSON.stringify({ moduleIds: observeModuleGraph(this, observe), ancillaryWatchFilesCollected: false }));
      },
    };
    const result = await vite.build(buildOptions(config, manifest, [boundary, react(), tailwind()], cache, dist));
    if (result && typeof result.close === 'function') { await result.close(); throw new Error('Unexpected Vite watcher was closed'); }
    outputs = outputManifest(dist);
  } catch (error) { failure = error; throw error; }
  finally {
    let closureFailure;
    try { reads.verify(); } catch (error) { closureFailure = error; }
    console.log('BASELINE_INPUTS ' + JSON.stringify({ coverage: 'Controller entries, TypeScript readFile, physical Vite load/module-graph files; excludes ancillary addWatchFile dependencies and complete runtime filesystem tracing', files: reads.manifest() }));
    if (closureFailure) { if (failure) console.error('FAIL input closure:', closureFailure.stack); else throw closureFailure; }
  }
  console.log('BASELINE_FRONTEND_OUTPUTS ' + JSON.stringify({ root: path.join(config.run, 'frontend-dist'), files: outputs, manifestSHA256: digest(JSON.stringify(outputs)) }));
  console.log('BASELINE_FRONTEND_BUILD_OK');
  return outputs;
}
if (require.main === module) {
  const [root, modules, run, ...extra] = process.argv.slice(2);
  Promise.resolve().then(() => { need(extra.length === 0, 'Unexpected build arguments'); return runBuild({ root, modules, run }); })
    .catch(error => { console.error('FAIL baseline frontend build:', error.stack); process.exitCode = 1; });
}
module.exports = { layout, InputReads, equalManifests, resolutionImporter, installResolution, typecheck, buildOptions, observeModuleGraph, outputManifest, runBuild };
