// Qualifies actual assertion/summary wiring without launching a browser.
// This deliberately does NOT qualify the UI, browser cleanup, or product effect.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const { stripTypeScriptTypes } = require('node:module');
const { spawnSync } = require('node:child_process');
const FILES = ['page-look-layers-check.cjs', 'database-views-check.cjs', 'database-ui-check.cjs', 'icons-covers-ui-check.cjs', 'database-mock-check.mts',
  'automations-ui-check.cjs', 'calc-ui-check.cjs', 'database-fixes-ui-check.cjs', 'mermaid-ui-check.cjs', 'ribbon-ui-check.cjs', 'rtl-ui-check.cjs'];

if (process.argv[2] === '--probe') {
  const [version, name, fault] = process.argv.slice(3);
  assert.ok(FILES.includes(name));
  const filename = version === 'before' ? path.join(__dirname, 'fixtures/graders-before', name) : path.join(__dirname, name);
  const source = fs.readFileSync(filename, 'utf8');
  const fixed = version === 'after';
  if (fault === 'unknown-view') {
    assert.equal(name, 'database-views-check.cjs');
    process.argv = [process.execPath, filename, 'unused-shots', 'not-a-view'];
    vm.runInNewContext(source, {
      console, process,
      require(name) {
        if (name === './grader-oracles.cjs') return require('./grader-oracles.cjs');
        if (name === 'playwright') return { chromium: { async launch() {
          console.log('LAUNCH_REQUESTED'); return { async close() {} };
        } } };
        assert.equal(name, '../tools/checks/fail-loud.cjs');
        return require('../tools/checks/fail-loud.cjs');
      },
    }, { filename, timeout: 2000 });
    return;
  }
  let prefix;
  if (name === 'page-look-layers-check.cjs' && !fixed) {
    prefix = 'let fails = 0;\n' + source.split('\n').find((line) => line.includes('const ok='));
  } else if (name === 'database-mock-check.mts') {
    const begin = fixed ? source.indexOf('const checks =') : source.indexOf('let fail =');
    const end = source.indexOf('const call =');
    assert.ok(begin >= 0 && end > begin);
    prefix = (fixed ? "const failLoud = require('../tools/checks/fail-loud.cjs');\n" : '') + source.slice(begin, end);
    prefix = stripTypeScriptTypes(prefix);
  } else {
    const end = source.indexOf(name === 'database-views-check.cjs' ? 'async function open' : '(async');
    assert.ok(end > 0);
    prefix = source.slice(0, end);
  }
  let exercise = '';
  const compare = name === 'page-look-layers-check.cjs' ? 'ok' : 'is';
  if (fault === 'good') exercise = compare === 'ok' ? "ok('known good', true);" : "is('known good', 3, 3);";
  else if (fault === 'wrong') exercise = compare === 'ok' ? "ok('planted wrong effect', false);" : "is('planted wrong effect', 4, 3);";
  else if (fault === 'nonfinite') exercise = compare === 'ok' ? "ok('nonfinite', Infinity);" : "is('nonfinite', Infinity, Infinity);";
  else if (fault === 'nan-json') exercise = "is('nonfinite', NaN, null);";
  else if (fault === 'wrong-sum' || fault === 'good-sum') {
    assert.equal(name, 'database-ui-check.cjs');
    const line = source.split('\n').find((value) => value.includes("'the rollup sums the Estimate of the linked rows'"));
    assert.ok(line);
    exercise = "const roll = '" + (fault === 'good-sum' ? '3' : '1') + "';\n" + line;
  } else assert.equal(fault, 'empty');
  let finish;
  if (fixed) {
    assert.ok(source.includes('checks.finish()'));
    finish = 'checks.finish();';
  } else if (compare === 'ok') {
    finish = "console.log(fails ? 'FAILED ' + fails : 'ALL OK');";
  } else {
    assert.ok(source.includes('process.exit(fail ? 1 : 0)'));
    finish = "console.log(fail ? 'FAILED ' + fail : 'ALL OK'); process.exit(fail ? 1 : 0);";
  }
  vm.runInNewContext(prefix + '\n' + exercise + '\n' + finish, {
    console, process, Buffer,
    require(name) {
      if (name === './grader-oracles.cjs') return require('./grader-oracles.cjs');
      if (name === 'playwright') return { chromium: {} };
      if (name === '../tools/checks/fail-loud.cjs') return require('../tools/checks/fail-loud.cjs');
      assert.ok(['fs', 'os', 'path'].includes(name), name);
      return require(name);
    },
  }, { filename, timeout: 2000 });
} else {
  const { test } = require('node:test');
  function run(version, file, fault) {
    const child = spawnSync(process.execPath, [__filename, '--probe', version, file, fault], {
      env: { ...process.env, NODE_OPTIONS: '', NODE_PATH: '', NODE_COMPILE_CACHE: '', NODE_DISABLE_COMPILE_CACHE: '1' },
      encoding: 'utf8', windowsHide: true, timeout: 5000, maxBuffer: 128 * 1024,
    });
    assert.ifError(child.error);
    assert.equal(child.signal, null);
    return child;
  }
  for (const file of FILES) {
    test('before: good assertion wiring accepts ' + file, () => {
      const child = run('before', file, 'good');
      assert.equal(child.status, 0, child.stdout + child.stderr);
    });
    test('before: empty assertion execution succeeds in ' + file, (t) => {
      const child = run('before', file, 'empty');
      t.diagnostic('Old empty wiring exit=' + child.status + ' ' + child.stdout.trim());
      assert.equal(child.status, 0, child.stdout + child.stderr);
    });
    for (const fault of ['good', 'wrong', 'empty', 'nonfinite']) {
      test('after: ' + file + ' / ' + fault, () => {
        const child = run('after', file, fault);
        assert.equal(child.status, fault === 'good' ? 0 : 1, child.stdout + child.stderr);
      });
    }
  }
  for (const [file, fault] of [['page-look-layers-check.cjs', 'wrong'], ...FILES.map((file) => [file, 'nonfinite']), ['database-views-check.cjs', 'nan-json'], ['icons-covers-ui-check.cjs', 'nan-json'], ['database-mock-check.mts', 'nan-json'], ['database-ui-check.cjs', 'wrong-sum']]) {
    test('before: planted blind spot ' + file + ' / ' + fault, (t) => {
      const child = run('before', file, fault);
      t.diagnostic('Old planted-defect exit=' + child.status + ' ' + child.stdout.trim());
      assert.equal(child.status, 0, child.stdout + child.stderr);
    });
  }
  for (const fault of ['wrong-sum', 'good-sum']) {
    test('after: rollup checks independently seeded exact sum / ' + fault, () => {
      const child = run('after', 'database-ui-check.cjs', fault);
      assert.equal(child.status, fault === 'good-sum' ? 0 : 1, child.stdout + child.stderr);
    });
  }
  test('before: actual unknown-view entry launches but runs no suite and succeeds', (t) => {
    const child = run('before', 'database-views-check.cjs', 'unknown-view');
    t.diagnostic('Old entry exit=' + child.status + ' ' + child.stdout.trim());
    assert.equal(child.status, 0, child.stdout + child.stderr);
    assert.match(child.stdout, /LAUNCH_REQUESTED/);
    assert.match(child.stdout, /ALL OK/);
  });
  test('after: actual unknown-view entry fails before requesting a browser', () => {
    const child = run('after', 'database-views-check.cjs', 'unknown-view');
    assert.equal(child.status, 1, child.stdout + child.stderr);
    assert.doesNotMatch(child.stdout, /LAUNCH_REQUESTED/);
  });
}
