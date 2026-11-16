// Owned mock-server launcher. Not product code and not an operating-system sandbox.
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const { pathToFileURL } = require('node:url');

const PORTS = Object.freeze({ mock: 1421, views: 1423 });
const ALTERNATE_RUNS = 'G:\\Notaya Dev folder 2\\runs';
const same = (a, b) => path.resolve(a).toLowerCase() === path.resolve(b).toLowerCase();
const inside = (file, directory) => {
  const relative = path.relative(path.resolve(directory), path.resolve(file));
  return relative !== '' && relative !== '..' && !relative.startsWith('..' + path.sep) && !path.isAbsolute(relative);
};
function need(condition, message) { if (!condition) throw new Error(message); }
function allowedRun(run, root) {
  return typeof run === 'string' && path.isAbsolute(run)
    && (inside(run, path.resolve(root, '../../runs')) || inside(run, ALTERNATE_RUNS));
}
function windowsProfileLayout(environment) {
  need(environment.USERPROFILE, 'An explicit owned USERPROFILE is required');
  need(environment.LOCALAPPDATA && same(environment.LOCALAPPDATA, path.join(environment.USERPROFILE, 'AppData/Local')),
    'Owned LOCALAPPDATA must be USERPROFILE\\AppData\\Local');
  need(environment.APPDATA && same(environment.APPDATA, path.join(environment.USERPROFILE, 'AppData/Roaming')),
    'Owned APPDATA must be USERPROFILE\\AppData\\Roaming');
}
function plainExisting(file) {
  let cursor = path.resolve(file);
  while (true) {
    const stat = fs.lstatSync(cursor);
    need(!stat.isSymbolicLink(), 'Refusing linked path: ' + cursor);
    const parent = path.dirname(cursor);
    if (parent === cursor) break;
    cursor = parent;
  }
  return path.resolve(file);
}
function ownedEnvironment(run) {
  for (const name of ['HOME', 'USERPROFILE', 'APPDATA', 'LOCALAPPDATA', 'TEMP', 'TMP', 'NOTAYA_HOME', 'CIPHERLEAF_HOME']) {
    need(process.env[name] && inside(process.env[name], run), 'An explicit owned child ' + name + ' is required');
    plainExisting(process.env[name]);
  }
  // Windows' known-folder routines use the USERPROFILE hierarchy; separate
  // sibling AppData folders are insufficient even when these variables exist.
  windowsProfileLayout(process.env);
  for (const name of ['NODE_OPTIONS', 'NODE_PATH', 'NODE_COMPILE_CACHE', 'ESBUILD_BINARY_PATH',
    'HTTP_PROXY', 'HTTPS_PROXY', 'ALL_PROXY', 'NO_PROXY', 'http_proxy', 'https_proxy', 'all_proxy', 'no_proxy',
    'WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS']) need(!process.env[name], 'Clear inherited ' + name);
  need(process.env.NODE_DISABLE_COMPILE_CACHE === '1', 'Disable compilation cache writes');
}
function layout({ root, modules, run, mode }) {
  need(Object.hasOwn(PORTS, mode), 'Mode must be mock or views');
  for (const value of [root, modules, run]) need(typeof value === 'string' && path.isAbsolute(value), 'Explicit absolute paths are required');
  need(same(root, path.resolve(__dirname, '../..')), 'Root must be this independent checkout');
  need(same(modules, path.resolve(root, '../p6c-evidence/node_modules')), 'Use the explicitly qualified sibling dependency tree');
  need(allowedRun(run, root), 'Run must be below the derivative runs folder or the explicitly approved G:\\Notaya Dev folder 2\\runs');
  for (const entry of [root, modules, run]) plainExisting(entry);
  return { root: path.resolve(root), modules: path.resolve(modules), run: path.resolve(run), mode, port: PORTS[mode] };
}
function dependencyAliases(manifest, modules) {
  const names = [...new Set([...Object.keys(manifest.dependencies || {}), ...Object.keys(manifest.devDependencies || {})])];
  const importerAnchor = path.resolve(modules, '../package.json');
  return names.map(name => {
    need(/^(?:@[a-z0-9._-]+\/)?[a-z0-9][a-z0-9._-]*$/i.test(name), 'Unexpected declared package name');
    return {
      find: name, replacement: name,
      customResolver(source, importer, options) {
        // Keep package names and subpaths intact. Vite, including its CSS and
        // dependency-optimizer resolvers, remains responsible for conditions,
        // browser mappings and exported-subpath checks. Preserve a dependency's
        // own importer so its nested dependencies keep normal resolution.
        const from = importer && inside(importer.split('?')[0], modules) ? importer : importerAnchor;
        return this.resolve(source, from, { ...options, skipSelf: true });
      },
    };
  });
}
function serverOptions(config, plugins, cache, out) {
  const alias = [
    { find: '@tauri-apps/api/core', replacement: path.join(config.root, 'dev-mock', config.mode === 'views' ? 'views-core.ts' : 'core.ts') },
    { find: '@tauri-apps/api/event', replacement: path.join(config.root, 'dev-mock/event.ts') },
    { find: '@tauri-apps/api/window', replacement: path.join(config.root, 'dev-mock/window.ts') },
    { find: '@tauri-apps/api/webviewWindow', replacement: path.join(config.root, 'dev-mock/webviewWindow.ts') },
    { find: '@', replacement: path.join(config.root, 'src') },
    ...dependencyAliases(config.manifest, config.modules),
  ];
  return {
    configFile: false, envDir: false, root: config.root, cacheDir: cache,
    clearScreen: false, plugins, resolve: { alias },
    build: { outDir: out, emptyOutDir: false },
    server: {
      host: '127.0.0.1', port: config.port, strictPort: true, open: false,
      watch: null, hmr: { host: '127.0.0.1', clientPort: config.port },
      proxy: undefined, cors: false,
      fs: {
        strict: true, allow: [config.root, config.modules, cache],
        deny: ['**/.git/**', '**/.env', '**/.env.*', '**/.npmrc', '**/.yarnrc*', '**/.devdata/**',
          '**/*.{nota,clf,kdbx,key,pem,p12,pfx,crt,cer,der}'],
      },
    },
  };
}
async function startServer(options) {
  const config = layout(options);
  ownedEnvironment(config.run);
  const manifestFile = path.join(config.root, 'package.json');
  const sdkManifestFile = path.resolve(config.modules, '../package.json');
  for (const file of [manifestFile, sdkManifestFile]) plainExisting(file);
  config.manifest = JSON.parse(fs.readFileSync(manifestFile, 'utf8'));
  const sdkManifest = JSON.parse(fs.readFileSync(sdkManifestFile, 'utf8'));
  for (const group of ['dependencies', 'devDependencies']) {
    need(JSON.stringify(config.manifest[group]) === JSON.stringify(sdkManifest[group]), 'Qualified SDK declarations do not match Main');
  }
  console.log('BASELINE_RESOLUTION unchanged bare package identifiers; Vite resolver anchored to qualified SDK; no Main dependency link');
  const folder = path.join(config.run, 'server-' + config.mode);
  fs.mkdirSync(folder); // Atomic no-overwrite; never reuse a failed server's cache.
  const cache = path.join(folder, 'cache');
  fs.mkdirSync(cache);
  const entries = {
    vite: path.join(config.modules, 'vite/dist/node/index.js'),
    react: path.join(config.modules, '@vitejs/plugin-react/dist/index.js'),
    tailwind: path.join(config.modules, '@tailwindcss/vite/dist/index.mjs'),
  };
  for (const [label, file] of Object.entries(entries)) {
    plainExisting(file);
    console.log('BASELINE_MODULE ' + label + ' ' + file + ' sha256=' + crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex'));
  }
  const vite = await import(pathToFileURL(entries.vite).href);
  const react = (await import(pathToFileURL(entries.react).href)).default;
  const tailwind = (await import(pathToFileURL(entries.tailwind).href)).default;
  const nonce = crypto.randomBytes(16).toString('hex');
  const origin = 'http://127.0.0.1:' + config.port;
  const identity = { nonce, root: config.root, mode: config.mode, pid: process.pid, origin };
  const boundary = {
    name: 'notaya-owned-baseline-boundary',
    configureServer(instance) {
      instance.middlewares.use((req, res, next) => {
        if (req.headers.host !== '127.0.0.1:' + config.port || !req.url.startsWith('/') || req.url.startsWith('//')) {
          res.statusCode = 403; res.end('Unexpected baseline host or request target'); return;
        }
        if (req.url !== '/__notaya_baseline_identity') { next(); return; }
        if (req.method !== 'GET') { res.statusCode = 405; res.end(); return; }
        res.setHeader('Content-Type', 'application/json');
        res.setHeader('Cache-Control', 'no-store');
        res.end(JSON.stringify(identity));
      });
    },
  };
  let server;
  try {
    server = await vite.createServer(serverOptions(config, [boundary, react(), tailwind()], cache, path.join(folder, 'out')));
    await server.listen();
    const address = server.httpServer.address();
    need(address.address === '127.0.0.1' && address.port === config.port, 'Unexpected resolved listener');
  } catch (error) {
    if (server) await server.close();
    throw error;
  }
  let closing;
  const close = () => closing ??= server.close().then(() => {
    need(!server.httpServer.listening, 'Owned listener did not close');
    console.log('BASELINE_SERVER_CLOSED ' + origin);
  });
  console.log('BASELINE_SERVER_READY ' + origin + ' pid=' + process.pid + ' nonce=' + nonce);
  return { server, identity, close };
}

if (require.main === module) {
  const [root, modules, run, mode, ...extra] = process.argv.slice(2);
  Promise.resolve().then(async () => {
    need(extra.length === 0, 'Unexpected server arguments');
    const owned = await startServer({ root, modules, run, mode });
    const stop = () => owned.close().catch(error => { console.error('FAIL baseline server cleanup:', error.message); process.exitCode = 1; });
    process.once('SIGINT', stop); process.once('SIGTERM', stop);
    process.stdin.setEncoding('utf8');
    process.stdin.on('data', chunk => {
      if (chunk.trim() !== 'stop') { console.error('FAIL unknown server control'); process.exitCode = 1; }
      void stop();
    });
  }).catch(error => { console.error('FAIL baseline server:', error.stack); process.exitCode = 1; });
}
module.exports = { PORTS, ALTERNATE_RUNS, inside, same, allowedRun, windowsProfileLayout, plainExisting, need, ownedEnvironment, layout, dependencyAliases, serverOptions, startServer };
