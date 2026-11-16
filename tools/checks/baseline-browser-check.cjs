// One child, one unchanged CJS grader, existing Core and an explicit approved Chromium browser.
// Routing plus an owned deny-only proxy contain HTTP/CONNECT/WebSocket attempts.
// This is not a kernel sandbox or proof about arbitrary native network traffic.
const fs = require('node:fs');
const path = require('node:path');
const http = require('node:http');
const { isIP } = require('node:net');
const crypto = require('node:crypto');
const Module = require('node:module');
const { PORTS, allowedRun, same, plainExisting, need, ownedEnvironment } = require('./baseline-server.cjs');

// Qualified Playwright Core 1.65.0 lib/coreBundle.js:36536-36570,
// SHA256 4EF8330594818D8F464DE939E32BCA62B758099E0D0E94A56095406DA963F7F1.
// Keep every default below when supplying a later merged Chrome switch. Core
// upgrades require review of this pin; enable-features and other defaults remain.
const CORE_DISABLED_FEATURES = Object.freeze([
  'AvoidUnnecessaryBeforeUnloadCheckSync', 'DestroyProfileOnBrowserClose',
  'HttpsUpgrades', 'HttpsFirstBalancedModeAutoEnable', 'LensOverlay', 'MediaRouter',
  'PaintHolding', 'ThirdPartyStoragePartitioning', 'AvoidCorsURLLoaderRestartOnRedirect',
  'TcpPortRandomizationWin', 'OptimizationHints', 'NetworkTimeServiceQuerying',
  'AimEnabled', 'PreconnectToSearch', 'msForceBrowserSignIn',
  'msEdgeUpdateLaunchServicesPreferredVersion',
]);
const CHROME_DISABLED_FEATURES = Object.freeze([...CORE_DISABLED_FEATURES, 'AutofillServerCommunication']);
const CHROME_MODEL_MANIFEST = 'chrome-model-manifest.json';
const HEADLESS_SHELL = 'G:\\Notaya Dev folder 2\\tools\\chrome-headless-shell-157.0.8103.78-win64\\chrome-headless-shell-win64\\chrome-headless-shell.exe';
const HEADLESS_SHELL_VERSION = '157.0.8103.78';
const INSTALLED_BROWSERS = Object.freeze([
  'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe',
  'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe',
  HEADLESS_SHELL,
]);
function approvedBrowser(file) {
  return typeof file === 'string' && path.isAbsolute(file) && INSTALLED_BROWSERS.some(expected => same(file, expected));
}
function validateBrowserVersion(executable, version) {
  if (same(executable, HEADLESS_SHELL)) {
    need(version === HEADLESS_SHELL_VERSION, 'Standalone test browser runtime version must equal ' + HEADLESS_SHELL_VERSION);
  }
}
async function qualifyBrowserVersion(browser, proxy, executable) {
  try {
    const version = browser.version();
    console.log('BASELINE_BROWSER_VERSION executablePath=' + path.resolve(executable) + ' version=' + JSON.stringify(version));
    validateBrowserVersion(executable, version);
  } catch (error) {
    const failures = [error];
    try { await browser.close(); } catch (cleanupError) { failures.push(cleanupError); }
    try { await proxy.close(); } catch (cleanupError) { failures.push(cleanupError); }
    if (failures.length > 1) throw new AggregateError(failures, 'Browser version qualification and cleanup failed');
    throw error;
  }
}

const SUITES = Object.freeze({
  'page-look-layers-check.cjs': { mode: 'mock', args: 'shots' },
  'database-ui-check.cjs': { mode: 'mock', args: 'shots', database: true },
  'database-views-check.cjs': { mode: 'views', args: 'shots' },
  'icons-covers-ui-check.cjs': { mode: 'mock', args: 'shots', bareOrigin: true },
  'automations-ui-check.cjs': { mode: 'mock', args: 'shots' },
  'calc-ui-check.cjs': { mode: 'mock', args: 'none' },
  'database-fixes-ui-check.cjs': { mode: 'mock', args: 'shots', database: true },
  'mermaid-ui-check.cjs': { mode: 'mock', args: 'url-shots' },
  'export-warning-check.cjs': { mode: 'mock', args: 'url-shots' },
  'ribbon-ui-check.cjs': { mode: 'mock', args: 'shots' },
  'rtl-ui-check.cjs': { mode: 'mock', args: 'shots' },
});
function suiteMockUrl(origin, suite) {
  // The icons grader appends its own slash; preserve each original URL contract.
  return origin + (suite.database ? '/?db=1' : suite.bareOrigin ? '' : '/');
}
function allowedNetwork(value, origin, websocket = false) {
  try {
    const url = new URL(value);
    return !url.username && !url.password && url.hostname === '127.0.0.1'
      && url.protocol === (websocket ? 'ws:' : 'http:')
      && url.port === new URL(origin).port;
  } catch { return false; }
}
const localContent = value => value === 'about:blank' || value.startsWith('data:') || value.startsWith('blob:');
function safeProxyTarget(req, kind) {
  const redacted = '[redacted]';
  const raw = req?.url;
  if (typeof raw !== 'string' || raw.length > 8192 || /[\\\r\n\t]/.test(raw)) return redacted;
  let authority, defaultPort;
  if (kind === 'connect') authority = raw;
  else {
    const absolute = /^(https?|wss?):\/\/([^/?#]*)(?:[/?#]|$)/i.exec(raw);
    if (absolute) {
      authority = absolute[2];
      defaultPort = /^(https|wss)$/i.test(absolute[1]) ? 443 : 80;
    } else if (raw.startsWith('/') && !raw.startsWith('//')) {
      authority = req.headers?.host;
      defaultPort = 80;
    } else return redacted;
  }
  // Never emit URL paths, queries, user information, escapes or non-ASCII hosts.
  if (typeof authority !== 'string' || authority.length > 261) return redacted;
  const parts = /^(\[[0-9a-f:.]+\]|[a-z0-9.-]+)(?::([0-9]{1,5}))?$/i.exec(authority);
  if (!parts || (kind === 'connect' && !parts[2])) return redacted;
  const host = parts[1].toLowerCase();
  if (host.startsWith('[')) {
    if (isIP(host.slice(1, -1)) !== 6) return redacted;
  } else {
    const labels = host.replace(/\.$/, '').split('.');
    if (host.length > 253 || labels.some(label => !/^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?$/.test(label))) return redacted;
  }
  const port = parts[2] === undefined ? defaultPort : Number(parts[2]);
  if (!Number.isInteger(port) || port < 1 || port > 65535) return redacted;
  return host + ':' + port;
}
function proxyHandlers(fail, diagnostic = message => console.error(message)) {
  const report = (kind, req) => diagnostic('BASELINE_DENIED_PROXY ' + kind.toUpperCase() + ' target=' + safeProxyTarget(req, kind));
  return {
    request(req, res) { fail('HTTP request reached deny-only proxy'); res.writeHead(403, { Connection: 'close' }); res.end('Blocked test network request'); report('request', req); },
    connect(req, socket) { fail('CONNECT reached deny-only proxy'); socket.end('HTTP/1.1 403 Forbidden\r\nConnection: close\r\n\r\n'); report('connect', req); },
    upgrade(req, socket) { fail('Upgrade reached deny-only proxy'); socket.end('HTTP/1.1 403 Forbidden\r\nConnection: close\r\n\r\n'); report('upgrade', req); },
  };
}
async function denyProxy(fail) {
  const handlers = proxyHandlers(fail);
  const server = http.createServer(handlers.request);
  const sockets = new Set();
  server.on('connection', socket => { sockets.add(socket); socket.on('close', () => sockets.delete(socket)); });
  server.on('connect', handlers.connect); server.on('upgrade', handlers.upgrade);
  await new Promise((resolve, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', resolve); });
  server.unref();
  let closing;
  return {
    origin: 'http://127.0.0.1:' + server.address().port,
    close: () => closing ??= new Promise((resolve, reject) => {
      for (const socket of sockets) socket.destroy();
      server.close(error => error ? reject(error) : resolve());
    }),
  };
}
function validateChromeManifest(run) {
  need(allowedRun(run, path.resolve(__dirname, '../..')), 'Chrome manifest must be in an approved owned run');
  const file = path.join(run, CHROME_MODEL_MANIFEST);
  plainExisting(file);
  const descriptor = fs.openSync(file, 'r');
  try {
    const before = fs.fstatSync(descriptor);
    need(before.isFile() && before.size === 2, 'Owned Chrome manifest must contain exactly two bytes: {}');
    const bytes = Buffer.alloc(3);
    const count = fs.readSync(descriptor, bytes, 0, 3, 0);
    need(count === 2 && bytes[0] === 0x7b && bytes[1] === 0x7d && fs.fstatSync(descriptor).size === 2,
      'Owned Chrome manifest must contain exactly two bytes: {}');
  } finally { fs.closeSync(descriptor); }
  return file;
}
function launchOptions(options, chrome, run, origin, proxy) {
  need(options && typeof options === 'object', 'Invalid browser launch options');
  const allowed = new Set(['headless', 'timeout', 'slowMo', 'args', 'ignoreDefaultArgs']);
  need(Object.keys(options).every(key => allowed.has(key)), 'Profile, environment, proxy or channel overrides are not allowed');
  // Two unchanged graders must see scrollbars. Do not allow callers to drop
  // any other Playwright defaults, including its security/network switches.
  need(options.ignoreDefaultArgs === undefined || (Array.isArray(options.ignoreDefaultArgs)
    && options.ignoreDefaultArgs.length === 1 && options.ignoreDefaultArgs[0] === '--hide-scrollbars'),
  'Default browser arguments may only expose scrollbars');
  need(options.headless === undefined || options.headless === true, 'Baseline browser must be headless');
  need(options.args === undefined || (Array.isArray(options.args) && options.args.length === 0), 'Unexpected browser arguments');
  need(options.timeout === undefined || (Number.isFinite(options.timeout) && options.timeout > 0 && options.timeout <= 30000), 'Launch timeout must be finite and bounded');
  need(options.slowMo === undefined || (Number.isFinite(options.slowMo) && options.slowMo >= 0), 'Invalid launch delay');
  const bypass = '<-loopback>,127.0.0.1:' + new URL(origin).port;
  return { ...options, executablePath: chrome, headless: true, downloadsPath: path.join(run, 'downloads'),
    proxy: { server: proxy, bypass }, env: { ...process.env },
    args: ['--disable-background-networking', '--disable-component-update', '--disable-default-apps',
      '--disable-sync', '--no-first-run', '--no-default-browser-check', '--disable-quic',
      '--force-webrtc-ip-handling-policy=disable_non_proxied_udp',
      '--host-resolver-rules=MAP * ~NOTFOUND, EXCLUDE 127.0.0.1, EXCLUDE localhost',
      '--disable-features=' + CHROME_DISABLED_FEATURES.join(','),
      '--optimization-guide-manifest-override=' + path.join(run, CHROME_MODEL_MANIFEST)] };
}
async function guardContext(context, origin, fail) {
  context.on('request', request => {
    if (!allowedNetwork(request.url(), origin) && !allowedNetwork(request.url(), origin, true)
      && !localContent(request.url())) fail('Unexpected page request origin');
  });
  await context.route('**/*', route => {
    if (allowedNetwork(route.request().url(), origin)) return route.continue();
    fail('Blocked unexpected page request'); return route.abort('blockedbyclient');
  });
  await context.routeWebSocket('**/*', socket => {
    if (allowedNetwork(socket.url(), origin, true)) { socket.connectToServer(); return; }
    fail('Blocked unexpected WebSocket'); return socket.close({ code: 1008, reason: 'Outside owned baseline origin' });
  });
  const observePage = page => page.on('websocket', socket => {
    if (!allowedNetwork(socket.url(), origin, true)) fail('Unexpected WebSocket origin');
  });
  context.on('page', observePage);
  for (const page of context.pages()) observePage(page);
}
async function guardedBrowser(rawChromium, options, config, fail) {
  // Root supplies this exact empty JSON file; never fall back to a missing file.
  // This changes Chrome's own on-device AI setup, not Notaya's AI behavior.
  // Edge handling of the Chrome-specific option requires separate qualification.
  validateChromeManifest(config.run);
  const proxy = await denyProxy(fail);
  let browser;
  try { browser = await rawChromium.launch(launchOptions(options, config.chrome, config.run, config.origin, proxy.origin)); }
  catch (error) { await proxy.close(); throw error; }
  await qualifyBrowserVersion(browser, proxy, config.chrome); // Before exposing any context or page.
  const contexts = new WeakSet();
  const guard = async context => {
    if (!contexts.has(context)) { await guardContext(context, config.origin, fail); contexts.add(context); }
    return context;
  };
  let closing;
  const close = () => closing ??= (async () => { try { await browser.close(); } finally { await proxy.close(); } })();
  browser.once('disconnected', () => { void proxy.close().catch(() => {}); });
  return new Proxy(browser, { get(target, property) {
    if (property === 'newContext') return async contextOptions => guard(await target.newContext(contextOptions));
    if (property === 'newPage') return async pageOptions => { const page = await target.newPage(pageOptions); await guard(page.context()); return page; };
    if (property === 'close') return close;
    const value = Reflect.get(target, property); return typeof value === 'function' ? value.bind(target) : value;
  } });
}
async function identity(origin, nonce, root, mode) {
  const result = await new Promise((resolve, reject) => {
    const request = http.get(origin + '/__notaya_baseline_identity', { agent: false }, response => {
      const chunks = []; let size = 0;
      response.on('data', chunk => { size += chunk.length; if (size > 4096) request.destroy(new Error('Identity response too large')); else chunks.push(chunk); });
      response.on('end', () => resolve({ code: response.statusCode, text: Buffer.concat(chunks).toString('utf8') }));
      response.on('error', reject);
    });
    request.setTimeout(5000, () => request.destroy(new Error('Identity timeout'))); request.on('error', reject);
  });
  const value = JSON.parse(result.text);
  need(result.code === 200 && value.nonce === nonce && value.mode === mode && same(value.root, root) && value.origin === origin,
    'Owned server identity did not match');
}
async function runBrowserCheck({ root, playwright, chrome, run, file, nonce }) {
  need(Object.hasOwn(SUITES, file), 'Unknown or empty browser-check selection');
  need(same(root, path.resolve(__dirname, '../..')), 'Root must be this independent checkout');
  need(allowedRun(run, root), 'Run must be inside an explicitly approved runs folder');
  const expectedCore = path.resolve(root, '../../qualification/p6c-node-restore-20261112T014330Z-a0694618/tools/node_modules/playwright-core/index.js');
  need(same(playwright, expectedCore), 'Use the explicitly qualified Playwright Core');
  need(approvedBrowser(chrome), 'Use an explicitly approved Chromium browser');
  for (const entry of [root, playwright, chrome, run]) plainExisting(entry);
  ownedEnvironment(run);
  need(typeof nonce === 'string' && /^[a-f0-9]{32}$/.test(nonce), 'Explicit server nonce required');
  const suite = SUITES[file], origin = 'http://127.0.0.1:' + PORTS[suite.mode];
  await identity(origin, nonce, root, suite.mode);
  const out = path.join(run, 'browser-' + file.replace(/\.cjs$/, ''));
  fs.mkdirSync(out); // Never overwrite prior output, even after a failed check.
  const original = path.join(root, 'dev-mock', file);
  plainExisting(original);
  const bytes = fs.readFileSync(original), source = bytes.toString('utf8');
  const digest = crypto.createHash('sha256').update(bytes).digest('hex');
  console.log('BASELINE_GRADER_SOURCE ' + original + ' originalSHA256=' + digest + ' executedSHA256=' + digest);
  console.log('BASELINE_INFRA existing-playwright-core explicit approved Chromium browser owned-deny-proxy; assertions/selectors/fixtures/tolerances unchanged');
  console.log('BASELINE_BROWSER_EXECUTABLE ' + path.resolve(chrome));
  const raw = require(playwright), owned = new Set();
  const fail = reason => { console.error('FAIL baseline infrastructure:', reason); if (!process.exitCode) process.exitCode = 1; };
  const chromium = new Proxy(raw.chromium, { get(target, property) {
    if (property === 'launch') return async (options = {}) => { const browser = await guardedBrowser(target, options, { chrome, run, origin }, fail); owned.add(browser); return browser; };
    if (['connect', 'connectOverCDP', 'launchPersistentContext'].includes(property)) return () => { throw new Error('Attaching or reusing a profile is forbidden'); };
    const value = Reflect.get(target, property); return typeof value === 'function' ? value.bind(target) : value;
  } });
  const closeAll = async () => { for (const browser of owned) { try { await browser.close(); } catch (error) { fail('Browser cleanup: ' + error.message); } } };
  const fatal = error => { fail(error?.stack || String(error)); void closeAll(); };
  process.once('uncaughtException', fatal); process.once('unhandledRejection', fatal);
  process.once('SIGINT', () => { process.exitCode = 130; void closeAll(); });
  process.once('SIGTERM', () => { process.exitCode = 143; void closeAll(); });
  process.env.MOCK_URL = suiteMockUrl(origin, suite);
  const args = suite.args === 'none' ? [] : suite.args === 'url-shots' ? [origin + '/', out] : [out];
  process.argv = [process.execPath, original, ...args];
  const originalLoad = Module._load;
  Module._load = function(request, parent, isMain) {
    if (request === 'playwright' && parent?.filename === original) return { ...raw, chromium };
    return originalLoad.call(this, request, parent, isMain);
  };
  try {
    const instance = new Module(original, module);
    instance.filename = original; instance.paths = Module._nodeModulePaths(path.dirname(original));
    instance._compile(source, original);
  } finally { Module._load = originalLoad; }
}

if (require.main === module) {
  const [root, playwright, chrome, run, file, nonce, ...extra] = process.argv.slice(2);
  Promise.resolve().then(() => { need(extra.length === 0, 'Unexpected browser-check arguments'); return runBrowserCheck({ root, playwright, chrome, run, file, nonce }); })
    .catch(error => { console.error('FAIL baseline browser launcher:', error.stack); process.exitCode = 1; });
}
module.exports = { CORE_DISABLED_FEATURES, CHROME_DISABLED_FEATURES, CHROME_MODEL_MANIFEST, HEADLESS_SHELL, HEADLESS_SHELL_VERSION, INSTALLED_BROWSERS, approvedBrowser, validateBrowserVersion, qualifyBrowserVersion, SUITES, suiteMockUrl, allowedNetwork, safeProxyTarget, proxyHandlers, validateChromeManifest, launchOptions, guardContext, guardedBrowser, environment: ownedEnvironment, runBrowserCheck };
