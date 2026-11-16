// Policy/option and owned-file tests only: no listener, browser, dependency or app is run.
const { test } = require('node:test');
const assert = require('node:assert/strict');
const path = require('node:path');
const fs = require('node:fs');
const crypto = require('node:crypto');
const http = require('node:http');
const manifest = require('../../package.json');
const { PORTS, inside, allowedRun, plainExisting, windowsProfileLayout, dependencyAliases, serverOptions } = require('./baseline-server.cjs');
const { CORE_DISABLED_FEATURES, CHROME_MODEL_MANIFEST, HEADLESS_SHELL, HEADLESS_SHELL_VERSION, INSTALLED_BROWSERS, approvedBrowser, validateBrowserVersion, qualifyBrowserVersion, SUITES, suiteMockUrl, allowedNetwork, safeProxyTarget, proxyHandlers, validateChromeManifest, launchOptions, guardContext, guardedBrowser, runBrowserCheck } = require('./baseline-browser-check.cjs');
const origin = 'http://127.0.0.1:1421';

test('only the exact owned HTTP origin is permitted', () => {
  assert.equal(allowedNetwork(origin + '/?db=1', origin), true);
  for (const url of ['http://127.0.0.1:1423/', 'http://127.0.0.1:19999/', 'http://localhost:1421/',
    'http://127.0.0.1.evil.invalid:1421/', 'http://127.0.0.1:1421@evil.invalid/',
    'http://user:password@127.0.0.1:1421/', 'https://127.0.0.1:1421/', 'file:///private/vault.nota', 'not-a-url']) {
    assert.equal(allowedNetwork(url, origin), false, url);
  }
});
test('only the owned plain WebSocket origin is permitted', () => {
  assert.equal(allowedNetwork('ws://127.0.0.1:1421/?token=synthetic', origin, true), true);
  for (const url of ['ws://127.0.0.1:19999/', 'wss://127.0.0.1:1421/', 'ws://localhost:1421/', 'ws://127.0.0.1.evil.invalid:1421/']) {
    assert.equal(allowedNetwork(url, origin, true), false, url);
  }
});
async function contextFixture() {
  const events = {}, failures = [];
  let httpRoute, socketRoute;
  const context = {
    on(event, callback) { events[event] = callback; }, pages() { return []; },
    async route(pattern, callback) { httpRoute = callback; },
    async routeWebSocket(pattern, callback) { socketRoute = callback; },
  };
  await guardContext(context, origin, message => failures.push(message));
  return { events, failures, httpRoute, socketRoute };
}
test('request observer accepts owned HTTP and WebSocket but marks other-port attempts', async () => {
  const fixture = await contextFixture();
  fixture.events.request({ url: () => origin + '/' });
  fixture.events.request({ url: () => 'ws://127.0.0.1:1421/' });
  assert.equal(fixture.failures.length, 0);
  fixture.events.request({ url: () => 'http://127.0.0.1:19999/' });
  assert.equal(fixture.failures.length, 1);
});
test('context HTTP route continues only the owned origin and aborts another port', async () => {
  const fixture = await contextFixture();
  let continued = 0, aborted = 0;
  const route = url => ({ request: () => ({ url: () => url }), continue() { continued++; }, abort() { aborted++; } });
  await fixture.httpRoute(route(origin + '/'));
  await fixture.httpRoute(route('http://127.0.0.1:19999/'));
  assert.equal(continued, 1); assert.equal(aborted, 1); assert.equal(fixture.failures.length, 1);
});
test('context WebSocket route connects only the owned origin and closes another port', async () => {
  const fixture = await contextFixture();
  let connected = 0, closed = 0;
  const socket = url => ({ url: () => url, connectToServer() { connected++; }, async close(options) { assert.equal(options.code, 1008); closed++; } });
  await fixture.socketRoute(socket('ws://127.0.0.1:1421/'));
  await fixture.socketRoute(socket('ws://127.0.0.1:19999/'));
  assert.equal(connected, 1); assert.equal(closed, 1); assert.equal(fixture.failures.length, 1);
});
for (const protocol of ['request', 'connect', 'upgrade']) {
  test('deny-only proxy refuses ' + protocol + ' without a forwarding operation', () => {
    const failures = [], writes = [], diagnostics = [];
    const handlers = proxyHandlers(message => failures.push(message), message => diagnostics.push(message));
    const response = { writeHead: (status, headers) => writes.push({ status, headers }), end: text => writes.push(text) };
    handlers[protocol]({ url: 'http://127.0.0.1:19999/synthetic' }, response);
    assert.equal(failures.length, 1);
    const reasons = { request: 'HTTP request reached deny-only proxy', connect: 'CONNECT reached deny-only proxy', upgrade: 'Upgrade reached deny-only proxy' };
    assert.equal(failures[0], reasons[protocol]);
    assert.equal(diagnostics.length, 1); assert.ok(!diagnostics[0].includes('/synthetic'));
    if (protocol === 'request') assert.equal(writes[0].status, 403);
    else assert.match(writes[0], /^HTTP\/1\.1 403 Forbidden/);
    assert.equal(Object.keys(handlers).length, 3);
  });
}
test('proxy target diagnostics contain only validated ASCII host and port', () => {
  assert.equal(safeProxyTarget({ url: 'EXAMPLE.invalid:443' }, 'connect'), 'example.invalid:443');
  assert.equal(safeProxyTarget({ url: '[::1]:443' }, 'connect'), '[::1]:443');
  assert.equal(safeProxyTarget({ url: 'http://example.invalid/private/path?key=synthetic#fragment' }, 'request'), 'example.invalid:80');
  assert.equal(safeProxyTarget({ url: 'https://example.invalid:8443/private' }, 'request'), 'example.invalid:8443');
  assert.equal(safeProxyTarget({ url: 'ws://example.invalid/socket?token=synthetic' }, 'upgrade'), 'example.invalid:80');
  assert.equal(safeProxyTarget({ url: 'wss://example.invalid/socket' }, 'upgrade'), 'example.invalid:443');
  assert.equal(safeProxyTarget({ url: '/private?secret=synthetic', headers: { host: 'example.invalid:8080' } }, 'upgrade'), 'example.invalid:8080');
  const longestHost = ['a'.repeat(63), 'b'.repeat(63), 'c'.repeat(63), 'd'.repeat(61)].join('.');
  assert.equal(longestHost.length, 253);
  assert.equal(safeProxyTarget({ url: longestHost + ':65535' }, 'connect'), longestHost + ':65535');
});
test('proxy target diagnostics redact credentials, malformed authorities and log injection', () => {
  for (const url of [undefined, null, 42, 'example.invalid', 'example.invalid:0', 'example.invalid:65536',
    'example.invalid:Infinity', 'example.invalid:443/path', 'user:secret@example.invalid:443',
    'example.invalid:443\r\nInjected: secret', 'example.invalid:443\x1b[31m',
    'ex\u00e4mple.invalid:443', 'example%2einvalid:443', 'example..invalid:443',
    '-example.invalid:443', 'a'.repeat(64) + '.invalid:443', '[not-ip]:443', '[::1%25scope]:443',
    ['a'.repeat(63), 'b'.repeat(63), 'c'.repeat(63), 'd'.repeat(62)].join('.') + ':443', 'a'.repeat(8193)]) {
    assert.equal(safeProxyTarget({ url }, 'connect'), '[redacted]');
  }
  for (const url of ['https://user:secret@example.invalid/private?token=synthetic', 'https://ex\u00e4mple.invalid/private',
    'https://example.invalid\\secret', 'https://example.invalid/private\nINJECTED', 'file:///private/vault.nota',
    'http://example.invalid:/private', 'http://example.invalid:65536/private', '//example.invalid/private']) {
    assert.equal(safeProxyTarget({ url }, 'request'), '[redacted]');
  }
  assert.equal(safeProxyTarget({ url: '/private', headers: { host: ['example.invalid', 'secret'] } }, 'upgrade'), '[redacted]');
  assert.equal(safeProxyTarget({ url: '/private', headers: { host: 'user:secret@example.invalid' } }, 'upgrade'), '[redacted]');
});
test('proxy diagnostics never replace failure accounting or denial', () => {
  for (const [kind, url] of [['request', 'http://example.invalid/private?secret=synthetic'],
    ['connect', 'example.invalid:443'], ['upgrade', 'ws://example.invalid/private?secret=synthetic']]) {
    const failures = [], diagnostics = [], writes = [];
    const handlers = proxyHandlers(reason => failures.push(reason), message => diagnostics.push(message));
    handlers[kind]({ url }, { writeHead: status => writes.push(status), end: body => writes.push(body) });
    assert.equal(failures.length, 1);
    assert.equal(diagnostics.length, 1);
    assert.match(diagnostics[0], /^BASELINE_DENIED_PROXY (REQUEST|CONNECT|UPGRADE) target=example\.invalid:(80|443)$/);
    assert.ok(writes.includes(403) || writes.some(value => /^HTTP\/1\.1 403 Forbidden/.test(value)));
    assert.ok(!diagnostics[0].includes('private')); assert.ok(!diagnostics[0].includes('secret'));
  }
});
test('launch uses an explicit approved Chromium browser and one loopback-port bypass', () => {
  const options = launchOptions({}, 'C:\\owned\\chrome.exe', 'D:\\owned\\run', origin, 'http://127.0.0.1:19998');
  assert.equal(options.executablePath, 'C:\\owned\\chrome.exe');
  assert.equal(options.headless, true);
  assert.equal(options.proxy.server, 'http://127.0.0.1:19998');
  assert.equal(options.proxy.bypass, '<-loopback>,127.0.0.1:1421');
  assert.ok(options.args.includes('--disable-quic'));
  assert.ok(options.downloadsPath.startsWith('D:\\owned\\run'));
});
test('browser allowlist accepts exactly the approved Chrome, Edge and standalone shell paths', () => {
  const expected = ['C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe',
    'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe',
    'G:\\Notaya Dev folder 2\\tools\\chrome-headless-shell-157.0.8103.78-win64\\chrome-headless-shell-win64\\chrome-headless-shell.exe'];
  assert.deepEqual(INSTALLED_BROWSERS, expected);
  for (const file of expected) assert.equal(approvedBrowser(file), true);
  for (const file of [undefined, null, 42, 'chrome', 'msedge.exe', 'C:chrome.exe',
    'C:Program Files\\Google\\Chrome\\Application\\chrome.exe',
    'file:///C:/Program%20Files/Google/Chrome/Application/chrome.exe',
    'https://example.invalid/chrome.exe', 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe.backup',
    'C:\\Program Files\\Google\\Chrome\\Application-old\\chrome.exe',
    'C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe',
    'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe\\child',
    'C:\\Program Files (x86)\\Microsoft\\Edge\\Application-other\\msedge.exe',
    HEADLESS_SHELL + '.backup', HEADLESS_SHELL + '\\child',
    HEADLESS_SHELL.replace('-157.0.8103.78-', '-158.0.0.0-'),
    HEADLESS_SHELL.slice(3), 'file:///' + HEADLESS_SHELL.replaceAll('\\', '/')]) {
    assert.equal(approvedBrowser(file), false, String(file));
  }
});
test('standalone shell runtime version must match exactly without constraining system browsers', () => {
  assert.equal(HEADLESS_SHELL_VERSION, '157.0.8103.78');
  assert.doesNotThrow(() => validateBrowserVersion(HEADLESS_SHELL, '157.0.8103.78'));
  for (const version of ['157.0.8103.77', '157.0.8103.79', '158.0.8150.0', '157.0.8103.78-extra', '', undefined, null, 157, NaN]) {
    assert.throws(() => validateBrowserVersion(HEADLESS_SHELL, version), /runtime version must equal/);
  }
  assert.doesNotThrow(() => validateBrowserVersion(INSTALLED_BROWSERS[0], '157.0.8103.51'));
  assert.doesNotThrow(() => validateBrowserVersion(INSTALLED_BROWSERS[1], '157.0.4341.46'));
});
test('good runtime version keeps fake browser and proxy open and logs the actual version', async () => {
  let browserCloses = 0, proxyCloses = 0;
  const messages = [], originalLog = console.log;
  console.log = message => messages.push(message);
  try {
    await qualifyBrowserVersion({ version: () => '157.0.8103.78', close: async () => { browserCloses++; } },
      { close: async () => { proxyCloses++; } }, HEADLESS_SHELL);
  } finally { console.log = originalLog; }
  assert.equal(browserCloses, 0); assert.equal(proxyCloses, 0);
  assert.deepEqual(messages, ['BASELINE_BROWSER_VERSION executablePath=' + path.resolve(HEADLESS_SHELL) + ' version="157.0.8103.78"']);
});
test('bad or unavailable runtime version closes fake browser and proxy before returning', async () => {
  const originalLog = console.log;
  console.log = () => {};
  try {
    for (const version of [() => '158.0.8150.0', () => { throw new Error('Version unavailable'); }]) {
      const closed = [];
      await assert.rejects(qualifyBrowserVersion({ version, close: async () => { closed.push('browser'); } },
        { close: async () => { closed.push('proxy'); } }, HEADLESS_SHELL), /runtime version must equal|Version unavailable/);
      assert.deepEqual(closed, ['browser', 'proxy']);
    }
  } finally { console.log = originalLog; }
});
test('version failure still closes the proxy if fake browser cleanup fails', async () => {
  let proxyCloses = 0;
  const originalLog = console.log;
  console.log = () => {};
  try {
    await assert.rejects(qualifyBrowserVersion({ version: () => '158.0.8150.0', close: async () => { throw new Error('Cleanup control'); } },
      { close: async () => { proxyCloses++; } }, HEADLESS_SHELL), error => {
      assert.ok(error instanceof AggregateError);
      assert.match(error.errors[0].message, /runtime version must equal/);
      assert.equal(error.errors[1].message, 'Cleanup control');
      return true;
    });
  } finally { console.log = originalLog; }
  assert.equal(proxyCloses, 1);
});
test('late disabled-features switch preserves the pinned Core defaults plus only Autofill suppression', () => {
  const file = path.resolve(__dirname, '../../../../qualification/p6c-node-restore-20261112T014330Z-a0694618/tools/node_modules/playwright-core/lib/coreBundle.js');
  plainExisting(file);
  const bytes = fs.readFileSync(file);
  assert.equal(crypto.createHash('sha256').update(bytes).digest('hex').toUpperCase(), '4EF8330594818D8F464DE939E32BCA62B758099E0D0E94A56095406DA963F7F1');
  const block = /disabledFeatures = \[([\s\S]*?)\]\.filter\(Boolean\);/.exec(bytes.toString('utf8'));
  assert.ok(block, 'Pinned Core disabledFeatures declaration must be found');
  const defaults = Array.from(block[1].matchAll(/^\s*"([A-Za-z0-9_]+)"[,]?\s*$/gm), match => match[1]);
  assert.equal(defaults.length, 16);
  assert.deepEqual(CORE_DISABLED_FEATURES, defaults);
  const options = launchOptions({}, 'chrome', 'G:\\Notaya Dev folder 2\\runs\\synthetic', origin, 'proxy');
  const switches = options.args.filter(arg => arg.startsWith('--disable-features='));
  assert.equal(switches.length, 1);
  const features = switches[0].slice('--disable-features='.length).split(',');
  assert.deepEqual(features, [...defaults, 'AutofillServerCommunication']);
  assert.equal(new Set(features).size, features.length);
  assert.equal(options.args.some(arg => arg.startsWith('--enable-features=')), false);
  assert.equal(options.ignoreDefaultArgs, undefined);
});
test('Chrome manifest option has only the fixed owned filename', () => {
  const run = 'G:\\Notaya Dev folder 2\\runs\\synthetic';
  const options = launchOptions({}, 'chrome', run, origin, 'proxy');
  assert.deepEqual(options.args.filter(arg => arg.startsWith('--optimization-guide-manifest-override=')),
    ['--optimization-guide-manifest-override=' + path.join(run, 'chrome-model-manifest.json')]);
});
function manifestFixture(name, bytes) {
  const root = process.env.NOTAYA_TOOLING_TEST_RUN;
  assert.ok(root && allowedRun(root, path.resolve(__dirname, '../..')), 'Explicit owned fixture run required');
  plainExisting(root);
  const run = path.join(root, name);
  fs.mkdirSync(run); // Preserve every case; no overwrite or cleanup.
  if (bytes !== undefined) fs.writeFileSync(path.join(run, CHROME_MODEL_MANIFEST), bytes, { flag: 'wx' });
  return run;
}
test('Chrome manifest accepts exactly the provided two-byte empty JSON object', () => {
  const run = manifestFixture('chrome-manifest-valid', Buffer.from('{}', 'ascii'));
  assert.equal(validateChromeManifest(run), path.join(run, CHROME_MODEL_MANIFEST));
  assert.equal(fs.readFileSync(path.join(run, CHROME_MODEL_MANIFEST)).length, 2);
});
test('Chrome manifest refuses linked-ancestor metadata before opening the file', () => {
  const run = manifestFixture('chrome-manifest-linked-metadata', Buffer.from('{}', 'ascii'));
  const originalLstat = fs.lstatSync, originalOpen = fs.openSync;
  let opens = 0;
  fs.lstatSync = file => file === run ? { isSymbolicLink: () => true } : originalLstat(file);
  fs.openSync = (...args) => { opens++; return originalOpen(...args); };
  try { assert.throws(() => validateChromeManifest(run), /linked path/); }
  finally { fs.lstatSync = originalLstat; fs.openSync = originalOpen; }
  assert.equal(opens, 0);
});
test('Chrome rejects missing and malformed manifests before creating any proxy or launching', async () => {
  const runs = [manifestFixture('chrome-manifest-missing')];
  for (const [index, bytes] of [Buffer.from(''), Buffer.from('{}\n'), Buffer.from('[]'), Buffer.from('{ }'),
    Buffer.from([0xef, 0xbb, 0xbf, 0x7b, 0x7d])].entries()) runs.push(manifestFixture('chrome-manifest-malformed-' + index, bytes));
  const originalCreateServer = http.createServer;
  let launches = 0, proxies = 0;
  http.createServer = () => { proxies++; throw new Error('A manifest failure must precede proxy creation'); };
  try {
    for (const run of runs) {
      await assert.rejects(guardedBrowser({ launch() { launches++; throw new Error('Unexpected launch'); } }, {}, { run }, () => {}), /ENOENT|exactly two bytes/);
    }
  } finally { http.createServer = originalCreateServer; }
  assert.equal(proxies, 0); assert.equal(launches, 0);
});
test('scrollbar visibility preserves every other launch isolation setting', () => {
  const ordinary = launchOptions({}, 'chrome', 'run', origin, 'proxy');
  const visible = launchOptions({ ignoreDefaultArgs: ['--hide-scrollbars'] }, 'chrome', 'run', origin, 'proxy');
  assert.deepEqual(visible, { ...ordinary, ignoreDefaultArgs: ['--hide-scrollbars'] });
});
test('no other default browser argument may be removed', () => {
  for (const value of [true, false, null, 'synthetic', [], ['--no-sandbox'], ['--disable-background-networking'],
    ['--hide-scrollbars', '--no-sandbox'], ['--hide-scrollbars', '--hide-scrollbars']]) {
    assert.throws(() => launchOptions({ ignoreDefaultArgs: value }, 'chrome', 'run', origin, 'proxy'), /only expose scrollbars/);
  }
});
for (const option of ['userDataDir', 'env', 'proxy', 'channel', 'executablePath']) {
  test('launch refuses a caller override: ' + option, () => {
    assert.throws(() => launchOptions({ [option]: 'synthetic' }, 'chrome', 'run', origin, 'proxy'), /overrides/);
  });
}
for (const value of [NaN, Infinity, -Infinity, 0, 30001]) {
  test('launch refuses invalid timeout ' + value, () => {
    assert.throws(() => launchOptions({ timeout: value }, 'chrome', 'run', origin, 'proxy'), /timeout/);
  });
}
test('launch refuses flags that could replace proxy or profile policy', () => {
  assert.throws(() => launchOptions({ args: ['--proxy-server=direct://'] }, 'chrome', 'run', origin, 'proxy'), /arguments/);
  assert.throws(() => launchOptions({ headless: false }, 'chrome', 'run', origin, 'proxy'), /headless/);
});
test('server configuration has explicit roots and ordinary local client transport', () => {
  const config = serverOptions({ root: 'D:\\owned\\main', modules: 'D:\\owned\\modules', manifest, port: PORTS.views, mode: 'views' }, [], 'D:\\owned\\cache', 'D:\\owned\\out');
  assert.equal(config.configFile, false); assert.equal(config.envDir, false);
  assert.equal(config.server.host, '127.0.0.1'); assert.equal(config.server.port, 1423); assert.equal(config.server.strictPort, true);
  assert.equal(config.server.watch, null); assert.equal(config.server.hmr.clientPort, 1423);
  assert.ok(config.resolve.alias.find(alias => alias.find === '@tauri-apps/api/core').replacement.endsWith('views-core.ts'));
  assert.equal(config.resolve.conditions, undefined); assert.equal(config.resolve.mainFields, undefined);
  assert.equal(config.server.fs.strict, true); assert.equal(config.server.fs.allow.length, 3);
  assert.ok(config.server.fs.deny.includes('**/.git/**'));
});
test('original icons URL composition cannot create a server-refused double slash', () => {
  const icons = suiteMockUrl(origin, SUITES['icons-covers-ui-check.cjs']);
  assert.equal(new URL(`${icons}/?cover=none`).pathname, '/');
  assert.equal(new URL(`${icons}/?cover=none`).search, '?cover=none');
  const page = suiteMockUrl(origin, SUITES['page-look-layers-check.cjs']);
  assert.equal(new URL(page + '?theme=black').href, origin + '/?theme=black');
  for (const name of ['database-ui-check.cjs', 'database-fixes-ui-check.cjs']) {
    assert.equal(new URL(suiteMockUrl(origin, SUITES[name])).href, origin + '/?db=1');
  }
});
test('suite arguments preserve Mermaid URL-first and views screenshot-first contracts', () => {
  assert.equal(Object.keys(SUITES).length, 10);
  assert.equal(SUITES['mermaid-ui-check.cjs'].args, 'url-shots');
  assert.equal(SUITES['database-views-check.cjs'].mode, 'views');
  assert.equal(SUITES['database-views-check.cjs'].args, 'shots');
  assert.equal(SUITES['calc-ui-check.cjs'].args, 'none');
});
test('empty or unknown suite fails before accessing a dependency or server', async () => {
  await assert.rejects(runBrowserCheck({ file: '' }), /selection/);
  await assert.rejects(runBrowserCheck({ file: '../somewhere.cjs' }), /selection/);
});
test('path containment rejects prefix siblings and parent escapes', () => {
  assert.equal(inside('D:\\owned\\run\\temp', 'D:\\owned\\run'), true);
  assert.equal(inside('D:\\owned\\run-other', 'D:\\owned\\run'), false);
  assert.equal(inside('D:\\owned\\run\\..\\other', 'D:\\owned\\run'), false);
});
test('run allowlist accepts only the derivative runs or the exact approved G runs root', () => {
  const root = 'D:\\derivative\\active\\main';
  assert.equal(allowedRun('D:\\derivative\\runs\\fresh\\containment', root), true);
  assert.equal(allowedRun('G:\\Notaya Dev folder 2\\runs\\fresh', root), true);
  for (const run of ['G:\\Notaya Dev folder 2', 'G:\\Notaya Dev folder 2\\runs',
    'G:\\Notaya Dev folder 2\\runs-other\\fresh', 'G:\\other\\runs\\fresh',
    'G:\\Notaya Dev folder 2\\runs\\..\\outside', 'D:\\derivative\\runs-old\\fresh', 'runs\\fresh']) {
    assert.equal(allowedRun(run, root), false, run);
  }
});
test('Windows profile requires owned Local and Roaming AppData within USERPROFILE', () => {
  const USERPROFILE = 'G:\\Notaya Dev folder 2\\runs\\fresh\\home';
  const environment = { USERPROFILE, LOCALAPPDATA: path.join(USERPROFILE, 'AppData/Local'), APPDATA: path.join(USERPROFILE, 'AppData/Roaming') };
  assert.doesNotThrow(() => windowsProfileLayout(environment));
  assert.throws(() => windowsProfileLayout({ ...environment, LOCALAPPDATA: 'G:\\Notaya Dev folder 2\\runs\\fresh\\localappdata' }), /LOCALAPPDATA/);
  assert.throws(() => windowsProfileLayout({ ...environment, APPDATA: 'G:\\Notaya Dev folder 2\\runs\\fresh\\appdata' }), /APPDATA/);
});
test('every declared direct package delegates unchanged bare identifiers and subpaths to Vite', async () => {
  const modules = 'D:\\qualified\\node_modules';
  const aliases = dependencyAliases(manifest, modules);
  const names = [...new Set([...Object.keys(manifest.dependencies), ...Object.keys(manifest.devDependencies)])];
  assert.deepEqual(aliases.map(alias => alias.find), names);
  const options = { attributes: { type: 'synthetic' }, kind: 'import-statement', scan: true, custom: { synthetic: true } };
  for (const alias of aliases) {
    assert.equal(alias.replacement, alias.find);
    for (const source of [alias.find, alias.find + '/synthetic-subpath?raw']) {
      const calls = [];
      const result = await alias.customResolver.call({ resolve(...args) { calls.push(args); return Promise.resolve({ id: 'synthetic-result' }); } }, source, 'D:\\main\\src\\entry.ts', options);
      assert.deepEqual(result, { id: 'synthetic-result' });
      assert.deepEqual(calls, [[source, path.resolve(modules, '../package.json'), { ...options, skipSelf: true }]]);
    }
  }
});
test('dependency aliases preserve nested SDK importers and propagate rejected exported subpaths', async () => {
  const modules = 'D:\\qualified\\node_modules';
  const alias = dependencyAliases({ dependencies: { react: 'synthetic' } }, modules)[0];
  const importer = path.join(modules, 'nested/node_modules/library/index.js');
  let actualImporter;
  await alias.customResolver.call({ resolve(source, from) { actualImporter = from; return null; } }, 'react', importer, {});
  assert.equal(actualImporter, importer);
  await assert.rejects(async () => alias.customResolver.call({ resolve() { throw new Error('Package subpath not exported'); } }, 'react/private', 'D:\\main\\src\\entry.ts', {}), /not exported/);
  assert.throws(() => dependencyAliases({ dependencies: { '../outside': 'synthetic' } }, modules), /package name/);
});
