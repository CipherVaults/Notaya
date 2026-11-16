// These qualify the actual assertion/summary wiring, not a native application.
// Full real-executable cases are a separate baseline run on throwaway data.
const assert = require('node:assert/strict');
const { spawnSync } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const graders = ['installed', 'restoreflow', 'lockflow', 'tray', 'optionalsecurity'];

// Execute the real phase body against explicit synthetic observations. This is
// grader qualification: it implements no editor, persistence or cryptography.
async function phaseFixture(version, grader, phase, mode, fault) {
  const base = version === 'before' ? path.join(__dirname, 'fixtures/before-fail-loud') : __dirname;
  const filename = path.join(base, grader + '.mjs');
  let source = fs.readFileSync(filename, 'utf8');
  source = source.replace(/^import .* from ["'](?:node:fs|node:sqlite|\.\.\/checks\/fail-loud\.cjs)["'];\r?\n/gm, '');
  assert.doesNotMatch(source, /^import /m, 'new infrastructure imports need explicit fixture support');
  const password = 'SYNTHETIC-ONLY-PASSPHRASE';
  const text = 'OPEN-MODE-NOTE-7731';
  const nonfinite = { nan: NaN, infinity: Infinity, 'negative-infinity': -Infinity };
  const numericFault = Object.hasOwn(nonfinite, fault) ? nonfinite[fault] : undefined;
  const isBad = fault === 'bad';
  const expectedDir = 'synthetic-data';
  const typed = grader === 'tray' ? 'TRAYTEST-' + mode : 'FLUSHTEST-' + mode;
  const state = {
    exists: !(grader === 'installed' && phase === 'first'),
    unlocked: grader === 'installed' ? phase !== 'first' : grader === 'optionalsecurity' ? phase !== 'p4' : grader === 'tray' ? mode === 'open' : true,
    securityOn: grader === 'optionalsecurity' ? phase === 'p4' : grader === 'tray' ? mode === 'secure' : false,
    pageText: grader === 'optionalsecurity' ? text : typed,
    prefs: { closeToTray: true, startWithWindows: false, startMinimized: false },
    backups: new Map(), nextBackup: 0, pageTitle: grader === 'optionalsecurity' ? 'Open note' : grader === 'tray' ? 'Tray page' : 'Flush page',
  };
  if (grader === 'optionalsecurity' && ['p2', 'p3'].includes(phase)) {
    state.backups.set('vault-open.nota', { text, open: true, modifiedMs: 1, size: 100 });
  }
  function protectionFlag(file) {
    const match = state.backups.get(file.split(/[\\/]/).at(-1));
    return match ? match.open : !state.securityOn;
  }
  const badValue = () => numericFault === undefined ? 'PLANTED-WRONG-CONTENT' : numericFault;
  async function rpc(command, args = {}) {
    if (command === 'vault_status') {
      let dataDir = expectedDir;
      if (grader === 'installed' && (isBad || numericFault !== undefined)) dataDir = badValue();
      let unlocked = state.unlocked;
      if (grader === 'tray' && phase === 'status' && (isBad || numericFault !== undefined)) unlocked = isBad ? !unlocked : numericFault;
      return { exists: state.exists, unlocked, securityOn: state.securityOn, offerImport: !state.exists, dataDir };
    }
    if (command === 'get_settings') return { idleLockMinutes: 180, screenCaptureProtection: false };
    if (command === 'save_settings') return args.settings;
    if (command === 'enable_security') {
      if (args.password === 'short') throw new Error('weak password');
      state.securityOn = true; state.unlocked = true;
      let removedCopies = 0;
      if (args.removeUnprotectedCopies) {
        for (const [name, backup] of state.backups) if (backup.open) { state.backups.delete(name); removedCopies++; }
      }
      if (grader === 'optionalsecurity') state.backups.set('vault-protected.nota', { text: state.pageText, open: false, modifiedMs: 2, size: 100 });
      if (grader === 'optionalsecurity' && phase === 'p2' && args.removeUnprotectedCopies && (isBad || numericFault !== undefined)) removedCopies = isBad ? 0 : numericFault;
      return { removedCopies };
    }
    if (command === 'disable_security') {
      if (!state.securityOn || args.currentPassword !== password) throw new Error('bad credentials or security already off');
      state.securityOn = false; state.unlocked = true; return {};
    }
    if (command === 'change_master') throw new Error('security is off');
    if (command === 'unlock_vault') {
      if (args.password !== password) throw new Error('bad credentials');
      state.unlocked = true; return {};
    }
    if (command === 'lock_vault') {
      if (state.securityOn) {
        state.unlocked = false;
        if (grader === 'restoreflow') {
          const index = ++state.nextBackup;
          state.backups.set('vault-' + index + '.nota', { text: state.pageText, open: false, modifiedMs: index, size: 100 });
        }
      }
      return {};
    }
    if (command === 'create_notebook') return { sections: [{ id: 'section' }] };
    if (command === 'create_page') {
      state.pageTitle = args.title; state.pageText = '';
      return { id: grader === 'lockflow' && phase === 'setup' && (isBad || numericFault !== undefined) ? badValue() : 'page' };
    }
    if (command === 'create_sticky') return { id: 'sticky' };
    if (command === 'save_page') { state.pageText = args.text; return {}; }
    if (command === 'list_notes') {
      if (!state.unlocked) throw new Error('locked');
      return { pages: [{ id: 'page', title: state.pageTitle }] };
    }
    if (command === 'get_page') {
      let pageText = state.pageText;
      if ((grader === 'lockflow' && phase === 'check' && mode !== 'sticky') || (grader === 'tray' && phase === 'check')) {
        if (isBad || numericFault !== undefined) pageText = badValue();
      }
      return { text: pageText, content: '' };
    }
    if (command === 'list_stickies') return [{ id: 'sticky' }];
    if (command === 'get_sticky') return { text: mode === 'sticky' && (isBad || numericFault !== undefined) ? badValue() : typed, content: '' };
    if (command === 'list_backups') {
      const list = [...state.backups].map(([name, backup]) => ({ name, ...backup })).sort((a, b) => b.modifiedMs - a.modifiedMs);
      if (numericFault !== undefined) list[0].modifiedMs = numericFault;
      return list;
    }
    if (command === 'restore_vault') {
      const backup = state.backups.get(args.source);
      if (!backup) throw new Error('unpicked or invalid backup path');
      if (backup.open && state.securityOn && !args.allowSecurityOff) throw 'SECURITY_OFF: confirmation required';
      if (!backup.open && args.password !== password) throw new Error('bad credentials');
      state.pageText = grader === 'restoreflow' && isBad ? 'PLANTED-WRONG-BACKUP' : backup.text;
      state.securityOn = !backup.open; state.unlocked = true;
      return { undoFile: 'synthetic-undo.nota' };
    }
    if (command === 'import_portable_notes') throw new Error('only an installed Notaya can import a portable copy');
    if (command === 'get_shell_prefs') {
      if (phase === 'prefs' && state.prefsChanged && (isBad || numericFault !== undefined)) return { ...state.prefs, startWithWindows: isBad ? mode !== 'on' : numericFault };
      return state.prefs;
    }
    if (command === 'set_shell_prefs') { state.prefs = args.prefs; state.prefsChanged = true; return {}; }
    if (command === 'plugin:window|close') return {};
    throw new Error('Unsupported fixture RPC: ' + command);
  }
  async function expressionValue(expression, target) {
    if (expression.includes('metadata?.currentWindow?.label')) return target;
    if (expression === 'location.reload()') return undefined;
    if (expression === 'document.body.innerText') return 'Welcome to Notaya\nImport notes from a portable copy\nStart new notes';
    if (expression.includes('Start new notes')) { state.exists = true; state.unlocked = true; return undefined; }
    if (expression.includes('aside span')) return true;
    if (expression.includes('document.execCommand')) return isBad || numericFault !== undefined ? badValue() : typed;
    if (expression.includes("return 'clicked'")) { state.unlocked = false; return 'clicked'; }
    if (expression.startsWith('window.__TAURI_INTERNALS__.invoke(')) {
      return vm.runInNewContext(expression, { window: { __TAURI_INTERNALS__: { invoke: rpc } } }, { timeout: 1000 });
    }
    throw new Error('Unsupported fixture expression: ' + expression);
  }
  function encode(value) {
    return JSON.stringify(value, (_key, item) => typeof item === 'number' && !Number.isFinite(item)
      ? { __grader_nonfinite__: String(item) } : item);
  }
  // CDP's real JSON cannot encode these values. A test-only decoder injection
  // delivers malformed observation fields directly to the actual predicates.
  const fixtureJSON = {
    stringify: JSON.stringify,
    parse(text) { return JSON.parse(text, (_key, item) => item && typeof item === 'object' && Object.keys(item).length === 1 && Object.hasOwn(item, '__grader_nonfinite__')
      ? ({ NaN, Infinity, '-Infinity': -Infinity })[item.__grader_nonfinite__] : item); },
  };
  class FakeSocket {
    constructor(url) { this.target = url.endsWith('/sticky') ? 'sticky-synthetic' : 'main'; queueMicrotask(() => this.onopen?.()); }
    send(raw) {
      const request = JSON.parse(raw);
      Promise.resolve(request.method === 'Runtime.evaluate' ? expressionValue(request.params.expression, this.target) : undefined)
        .then((value) => this.onmessage?.({ data: encode({ id: request.id, result: { result: { value } } }) }),
          (error) => this.onmessage?.({ data: encode({ id: request.id, result: { exceptionDetails: { exception: { description: String(error) } } } }) }));
    }
    close() {}
  }
  const args = grader === 'installed' ? ['9335', phase, expectedDir]
    : grader === 'restoreflow' ? ['9335', 'synthetic-password-file']
    : grader === 'optionalsecurity' ? ['9335', 'synthetic-password-file', 'synthetic-data', phase]
    : ['9335', phase, mode, 'synthetic-password-file'];
  const fixtureProcess = { argv: [process.execPath, filename, ...args], exit(code) { process.exit(code); } };
  Object.defineProperty(fixtureProcess, 'exitCode', { get: () => process.exitCode, set: (value) => { process.exitCode = value; } });
  const context = {
    console, process: fixtureProcess, JSON: fixtureJSON, Buffer,
    graderChecks: require('../checks/fail-loud.cjs'),
    setTimeout(callback) { queueMicrotask(callback); return 0; }, clearTimeout() {},
    WebSocket: FakeSocket,
    fetch: async () => ({ json: async () => [{ type: 'page', title: 'Synthetic main', webSocketDebuggerUrl: 'ws://synthetic/main' }, { type: 'page', title: 'Synthetic sticky', webSocketDebuggerUrl: 'ws://synthetic/sticky' }] }),
    readFileSync(file) {
      if (file === 'synthetic-password-file') return password;
      return Buffer.from(state.securityOn ? 'SYNTHETIC-ENCRYPTED-FIXTURE' : text);
    },
    readdirSync: () => [...state.backups.keys()], existsSync: () => true,
    DatabaseSync: class {
      constructor(file) { this.file = file; }
      prepare() { return { get: () => { const data = Buffer.alloc(51); data[50] = protectionFlag(this.file) ? 2 : 0; return { data }; } }; }
      close() {}
    },
  };
  await new vm.Script('(async () => {\n' + source + '\n})()', { filename }).runInNewContext(context, { timeout: 2000 });
}

if (process.argv[2] === '--phase-fixture') {
  phaseFixture(...process.argv.slice(3)).catch((error) => { console.error(error); process.exitCode = 1; });
} else if (process.argv[2] === '--fixture') {
  const [version, grader, fault] = process.argv.slice(3);
  assert.ok(graders.includes(grader));
  const base = version === 'before' ? path.join(__dirname, 'fixtures/before-fail-loud') : __dirname;
  const source = fs.readFileSync(path.join(base, grader + '.mjs'), 'utf8');
  const values = { good: true, bad: false, nonfinite: NaN };
  let program;
  if (version === 'before') {
    if (grader === 'optionalsecurity') {
      const definition = source.match(/^const check = .*$/m)[0];
      const end = source.slice(source.indexOf('const failed = results.filter'));
      program = 'const results=[]; const phase="qualification";\n' + definition + '\n'
        + (fault === 'empty' ? '' : 'check("planted observation", value);\n') + end;
    } else if (['installed', 'restoreflow'].includes(grader)) {
      const definition = source.match(/^const check = .*$/m)[0];
      program = definition + '\n' + (fault === 'empty' ? '' : 'check("planted observation", value);\n')
        + source.match(/^process\.exit\(0\).*$/m)[0];
    } else {
      // The old persistence grader only prints this real result, then exits 0.
      const line = source.split('\n').find((s) => s.includes('console.log(`page contains'));
      assert.ok(line);
      program = (fault === 'empty' ? '' : line + '\n') + source.match(/^process\.exit\(0\).*$/m)[0];
    }
  } else {
    assert.match(source, /const check = checks\.check;/);
    assert.match(source, /checks\.finish\(\);/);
    const definition = source.match(/^const check = checks\.check;$/m)[0];
    program = definition + '\n' + (fault === 'empty' ? '' : 'check("planted observation", value);\n')
      + 'checks.finish();';
  }
  vm.runInNewContext(program, {
    console, process, value: values[fault],
    checks: version === 'after' ? require('../checks/fail-loud.cjs').createChecks(grader) : null,
    TYPED: 'EXPECTED', inPage: fault === 'good' ? 'EXPECTED' : String(values[fault]),
    inSticky: fault === 'good' ? 'EXPECTED' : String(values[fault]),
    pg: { text: fault === 'good' ? 'EXPECTED' : String(values[fault]), content: '' },
  }, { timeout: 2000, filename: grader + '-actual-assertion-wiring' });
} else {
  const { test } = require('node:test');
  function runPhase(version, grader, phase, mode = 'page', fault = 'good') {
    const child = spawnSync(process.execPath, [__filename, '--phase-fixture', version, grader, phase, mode, fault], {
      encoding: 'utf8', timeout: 5000, maxBuffer: 128 * 1024, windowsHide: true,
      env: { ...process.env, NODE_OPTIONS: '', NODE_PATH: '', NODE_COMPILE_CACHE: '', NODE_DISABLE_COMPILE_CACHE: '1' },
    });
    assert.ifError(child.error);
    assert.equal(child.signal, null, child.stderr);
    return child;
  }
  const goodPhases = [
    ...['first', 'second', 'portable'].map((phase) => ['installed', phase, 'page']),
    ['restoreflow', 'restore', 'page'], ['lockflow', 'setup', 'page'],
    ...['type', 'check'].flatMap((phase) => ['page', 'sticky', 'close'].map((mode) => ['lockflow', phase, mode])),
    ...['setup', 'close', 'status', 'check'].flatMap((phase) => ['secure', 'open'].map((mode) => ['tray', phase, mode])),
    ...['on', 'off'].map((mode) => ['tray', 'prefs', mode]),
    ...['p1', 'p2', 'p3', 'p4', 'p5'].map((phase) => ['optionalsecurity', phase, 'page']),
  ];
  for (const [grader, phase, mode] of goodPhases) {
    test(`phase before good: ${grader}/${phase}/${mode}`, () => {
      const child = runPhase('before', grader, phase, mode);
      assert.equal(child.status, 0, child.stdout + child.stderr);
      assert.doesNotMatch(child.stdout, /FAIL/);
    });
    test(`phase after good: ${grader}/${phase}/${mode}`, () => {
      const child = runPhase('after', grader, phase, mode);
      assert.equal(child.status, 0, child.stdout + child.stderr);
      assert.match(child.stdout, /ALL OK \([1-9]\d* checks/);
    });
  }
  const faultPhases = [
    ['installed', 'first', 'page'], ['restoreflow', 'restore', 'page'],
    ['lockflow', 'check', 'page'], ['lockflow', 'check', 'sticky'],
    ['tray', 'check', 'open'], ['tray', 'status', 'secure'], ['tray', 'prefs', 'on'],
    ['optionalsecurity', 'p2', 'page'],
  ];
  for (const [grader, phase, mode] of faultPhases) {
    for (const fault of ['bad', 'nan', 'infinity', 'negative-infinity']) {
      test(`phase before fault: ${grader}/${phase}/${mode}/${fault}`, (t) => {
        const child = runPhase('before', grader, phase, mode, fault);
        const expected = grader === 'optionalsecurity' && fault !== 'infinity' ? 1 : 0;
        assert.equal(child.status, expected, child.stdout + child.stderr);
        t.diagnostic('actual old phase exit=' + child.status + '\n' + child.stdout);
      });
      test(`phase after fault: ${grader}/${phase}/${mode}/${fault}`, () => {
        const child = runPhase('after', grader, phase, mode, fault);
        assert.equal(child.status, 1, child.stdout + child.stderr);
        assert.match(child.stdout, /FAIL/);
      });
    }
  }
  for (const grader of ['installed', 'lockflow', 'tray', 'optionalsecurity']) {
    test(`phase before empty: ${grader}/unknown`, (t) => {
      const child = runPhase('before', grader, 'unknown', grader === 'tray' ? 'open' : 'page');
      assert.equal(child.status, grader === 'installed' ? 1 : 0, child.stdout + child.stderr);
      t.diagnostic('actual old unknown phase exit=' + child.status + '\n' + child.stdout);
    });
    test(`phase after empty: ${grader}/unknown`, () => {
      const child = runPhase('after', grader, 'unknown', grader === 'tray' ? 'open' : 'page');
      assert.equal(child.status, 1, child.stdout + child.stderr);
      assert.match(child.stdout, /FAIL/);
    });
  }
  function numericObservation(version, grader, input) {
    const base = version === 'before' ? path.join(__dirname, 'fixtures/before-fail-loud') : __dirname;
    const source = fs.readFileSync(path.join(base, grader + '.mjs'), 'utf8');
    const label = grader === 'restoreflow' ? 'list_backups works while locked, newest first' : 'security turned on; earlier open backups were deleted';
    const statement = source.split('\n').find((line) => line.includes('check("' + label + '"'));
    assert.ok(statement, 'actual numeric assertion must be present: ' + grader);
    let observed;
    vm.runInNewContext(statement, {
      check(_name, value) { observed = value; },
      list: grader === 'restoreflow' ? input.map((modifiedMs) => ({ modifiedMs })) : undefined,
      on: grader === 'optionalsecurity' ? { ok: { removedCopies: input } } : undefined,
    }, { timeout: 2000, filename: grader + '-actual-numeric-predicate' });
    return observed;
  }
  for (const [label, values] of [
    ['newer infinity', [Infinity, 1]], ['older negative infinity', [2, -Infinity]],
    ['both positive infinity', [Infinity, Infinity]], ['both negative infinity', [-Infinity, -Infinity]],
  ]) {
    test('numeric before: restore accepts ' + label, (t) => {
      const observed = numericObservation('before', 'restoreflow', values);
      assert.equal(observed, true);
      t.diagnostic('retained old numeric predicate accepts the planted non-finite timestamps');
    });
  }
  test('numeric before: optional security accepts infinite removed-copy count', (t) => {
    assert.equal(numericObservation('before', 'optionalsecurity', Infinity), true);
    t.diagnostic('retained old numeric predicate accepts the planted infinite count');
  });
  for (const [label, values, expected] of [
    ['ordered', [2, 1], true], ['equal', [1, 1], true], ['finite pre-epoch', [0, -1], true], ['wrong order', [1, 2], false],
    ['newer NaN', [NaN, 1], false], ['older NaN', [2, NaN], false],
    ['newer infinity', [Infinity, 1], false], ['older infinity', [2, Infinity], false],
    ['newer negative infinity', [-Infinity, 1], false], ['older negative infinity', [2, -Infinity], false],
    ['both positive infinity', [Infinity, Infinity], false], ['both negative infinity', [-Infinity, -Infinity], false],
  ]) {
    test('numeric after: restore ' + label, () => {
      assert.equal(numericObservation('after', 'restoreflow', values), expected);
    });
  }
  for (const [label, count, expected] of [
    ['one', 1, true], ['several', 2, true], ['zero', 0, false], ['negative', -1, false],
    ['NaN', NaN, false], ['infinity', Infinity, false], ['negative infinity', -Infinity, false],
  ]) {
    test('numeric after: optional security ' + label, () => {
      assert.equal(numericObservation('after', 'optionalsecurity', count), expected);
    });
  }
  function run(version, grader, fault) {
    const child = spawnSync(process.execPath, [__filename, '--fixture', version, grader, fault], {
      encoding: 'utf8', timeout: 5000, maxBuffer: 65536, windowsHide: true,
      env: { ...process.env, NODE_OPTIONS: '', NODE_PATH: '', NODE_COMPILE_CACHE: '', NODE_DISABLE_COMPILE_CACHE: '1' },
    });
    assert.ifError(child.error);
    assert.equal(child.signal, null, child.stderr);
    return child;
  }
  for (const grader of graders) {
    for (const fault of ['good', 'bad', 'empty', 'nonfinite']) {
      test(`before: ${grader} ${fault} observed exit`, (t) => {
        const child = run('before', grader, fault);
        const expected = grader === 'optionalsecurity' && ['bad', 'nonfinite'].includes(fault) ? 1 : 0;
        assert.equal(child.status, expected, child.stdout + child.stderr);
        t.diagnostic('observed exit=' + child.status + '\n' + child.stdout);
      });
      test(`after: ${grader} ${fault} fail-loud wiring`, () => {
        const child = run('after', grader, fault);
        assert.equal(child.status, fault === 'good' ? 0 : 1, child.stdout + child.stderr);
        if (fault !== 'good') assert.match(child.stdout, /FAIL/);
      });
    }
  }
  const invalidArguments = {
    installed: ['9335', 'unknown', 'synthetic-data'],
    restoreflow: [],
    lockflow: ['9335', 'unknown', 'page', 'must-not-be-read'],
    tray: ['9335', 'unknown', 'open', 'must-not-be-read'],
    optionalsecurity: ['9335', 'must-not-be-read', 'synthetic-data', 'unknown'],
  };
  for (const [grader, args] of Object.entries(invalidArguments)) {
    test(`entry point: ${grader} rejects empty/unknown selection before access`, () => {
      const child = spawnSync(process.execPath, ['--require', path.join(__dirname, 'fixtures/no-network.cjs'), path.join(__dirname, grader + '.mjs'), ...args], {
        encoding: 'utf8', timeout: 5000, maxBuffer: 65536, windowsHide: true,
        env: { ...process.env, NODE_OPTIONS: '', NODE_PATH: '', NODE_COMPILE_CACHE: '', NODE_DISABLE_COMPILE_CACHE: '1' },
      });
      assert.ifError(child.error);
      assert.equal(child.status, 1, child.stdout + child.stderr);
      assert.match(child.stdout, /FAIL/);
      assert.doesNotMatch(child.stderr, /ENOENT|NETWORK_FORBIDDEN/);
    });
  }
}
