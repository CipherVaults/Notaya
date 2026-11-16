// Grader qualification only. Faults wrap the real engine in a child process;
// neither the calculator source nor the historical grader fixture is changed.
const assert = require('node:assert/strict');
const { spawnSync } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');

if (process.argv[2] === '--fixture') {
  const [version, fault] = process.argv.slice(3);
  const real = require('../src/features/sticky/calc.ts');
  const engine = {
    ...real,
    evaluate(expression, options) {
      if (expression === '10 mod 3' && fault === 'wrong-error-type') throw new Error('planted unrelated error');
      if (expression === '10 mod 3' && fault === 'invalid-returns-value') return 0;
      const value = real.evaluate(expression, options);
      if (fault === 'wrong-answer' && expression === '1+2*3') return value + 1;
      if (expression === 'sin(90)') {
        if (fault === 'nan') return NaN;
        if (fault === 'infinity') return Infinity;
        if (fault === 'negative-infinity') return -Infinity;
      }
      return value;
    },
  };
  if (version === 'before') {
    const filename = path.join(__dirname, 'fixtures/calc-check.pre-fail-loud.cjs');
    const source = fs.readFileSync(filename, 'utf8');
    vm.runInNewContext(source, {
      console,
      process,
      require(name) {
        assert.equal(name, '../.calc-check/calc.js');
        return engine;
      },
    }, { filename, timeout: 2000 });
  } else if (version === 'after') {
    const { runCalculatorChecks } = require('./calc-check.cjs');
    const groups = fault === 'empty' ? [] : fault === 'unknown' ? ['not-a-group']
      : fault.startsWith('group:') ? [fault.slice(6)] : undefined;
    runCalculatorChecks({ engine, groups });
  } else if (version === 'helper') {
    const checks = require('../tools/checks/fail-loud.cjs').createChecks('helper contract');
    if (fault === 'nan-expected') checks.near('finite expected', 1, NaN, 1e-9);
    else if (fault === 'infinite-expected') checks.near('finite expected', 1, Infinity, 1e-9);
    else if (fault === 'infinite-tolerance') checks.near('finite tolerance', 1, 1, Infinity);
    else if (fault === 'negative-tolerance') checks.near('nonnegative tolerance', 1, 1, -1);
    else if (fault === 'nan-json') checks.equal('numeric leaves', { value: NaN }, { value: null });
    else if (fault === 'infinite-json') checks.equal('numeric leaves', [Infinity], [null]);
    else if (fault === 'strings') checks.equal('ordinary note strings', ['NaN', 'Infinity'], ['NaN', 'Infinity']);
    else if (fault === 'prior-exit') { process.exitCode = 2; checks.check('good check', true); }
    else if (fault === 'strict-boundary') checks.near('strict boundary', 1, 0, 1, false);
    else if (fault === 'inclusive-boundary') checks.near('inclusive boundary', 1, 0, 1);
    else if (fault !== 'empty') throw new Error('Unknown helper fault');
    checks.finish();
  } else {
    throw new Error('Unknown grader fixture version');
  }
} else {
  const { test } = require('node:test');
  function run(version, fault = 'good') {
    const child = spawnSync(process.execPath, [__filename, '--fixture', version, fault], {
      cwd: path.resolve(__dirname, '..'),
      env: { ...process.env, NODE_OPTIONS: '', NODE_PATH: '' },
      encoding: 'utf8', timeout: 5000, maxBuffer: 128 * 1024,
      windowsHide: true,
    });
    assert.ifError(child.error);
    assert.equal(child.signal, null, child.stderr);
    return child;
  }
  test('before: the unchanged grader accepts the good calculator', (t) => {
    const child = run('before');
    t.diagnostic(`exit=${child.status}\n${child.stdout}`);
    assert.equal(child.status, 0, child.stderr);
    assert.match(child.stdout, /ALL OK/);
  });
  test('before: planted wrong answer prints FAIL but exits successfully', (t) => {
    const child = run('before', 'wrong-answer');
    t.diagnostic(`exit=${child.status}\n${child.stdout}`);
    assert.equal(child.status, 0, child.stderr);
    assert.match(child.stdout, /FAIL 1\+2\*3/);
    assert.match(child.stdout, /FAILURES: 1/);
  });
  test('before: planted secondary NaN prints ALL OK and exits successfully', (t) => {
    const child = run('before', 'nan');
    t.diagnostic(`exit=${child.status}\n${child.stdout}`);
    assert.equal(child.status, 0, child.stderr);
    assert.match(child.stdout, /ALL OK/);
    assert.doesNotMatch(child.stdout, /FAIL/);
  });
  test('after: good calculator passes all original checks', () => {
    const child = run('after');
    assert.equal(child.status, 0, child.stdout + child.stderr);
    assert.match(child.stdout, /ALL OK \(86 checks; calculator\)/);
  });
  for (const fault of ['wrong-answer', 'nan', 'infinity', 'negative-infinity', 'empty', 'unknown', 'wrong-error-type', 'invalid-returns-value']) {
    test(`after: ${fault} cannot pass`, () => {
      const child = run('after', fault);
      assert.equal(child.status, 1, child.stdout + child.stderr);
      assert.match(child.stdout, /FAIL/);
    });
  }
  for (const [group, count] of Object.entries({ expressions: 34, angles: 9, 'angle-dependencies': 9, errors: 16, formatting: 9, negation: 9 })) {
    test(`calculator preserves ${count} original checks in ${group}`, () => {
      const child = run('after', 'group:' + group);
      assert.equal(child.status, 0, child.stdout + child.stderr);
      assert.ok(child.stdout.includes('ALL OK (' + count + ' checks; calculator)'), child.stdout);
    });
  }
  for (const fault of ['nan-expected', 'infinite-expected', 'infinite-tolerance', 'negative-tolerance', 'nan-json', 'infinite-json', 'empty', 'strict-boundary']) {
    test(`shared helper: ${fault} cannot pass`, () => {
      const child = run('helper', fault);
      assert.equal(child.status, 1, child.stdout + child.stderr);
      assert.match(child.stdout, /FAIL/);
    });
  }
  for (const fault of ['strings', 'inclusive-boundary']) {
    test(`shared helper: legitimate ${fault} passes`, () => {
      const child = run('helper', fault);
      assert.equal(child.status, 0, child.stdout + child.stderr);
    });
  }
  test('shared helper: finish preserves an earlier failing exit', () => {
    const child = run('helper', 'prior-exit');
    assert.equal(child.status, 2, child.stdout + child.stderr);
  });
  for (const argument of ['--only=', '--only=not-a-group']) {
    test(`calculator entry point rejects ${argument}`, () => {
      const child = spawnSync(process.execPath, [path.join(__dirname, 'calc-check.cjs'), argument], {
        encoding: 'utf8', timeout: 5000, maxBuffer: 128 * 1024,
        env: { ...process.env, NODE_OPTIONS: '', NODE_PATH: '' }, windowsHide: true,
      });
      assert.ifError(child.error);
      assert.equal(child.status, 1, child.stdout + child.stderr);
      assert.match(child.stdout, /FAIL/);
    });
  }
}
