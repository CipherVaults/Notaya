// Pure oracle fixtures. No Playwright, app, listener, network, or user data.
const assert = require('node:assert/strict');
const { test } = require('node:test');
const { resolve } = require('node:path');
const source = process.env.NOTAYA_GRADER_ORACLES_TEST_FILE || resolve(__dirname, 'grader-oracles.cjs');
const { isMermaidRuntimeResource, inspectScriptBoundary, expectedFixtureLine, assertFixtureLine } = require(source);
const base = 'http://127.0.0.1:1421/';

test('eager editor sources and query text are not the lazy package', () => {
  for (const url of ['/src/features/notes/editor/MermaidBlock.ts', '/src/features/notes/editor/mermaid.ts', '/src/main.js?name=mermaid.js']) {
    assert.equal(isMermaidRuntimeResource(url, base), false);
  }
});
test('actual Vite cache and package runtime entries are counted', () => {
  for (const url of ['/node_modules/.vite/deps/mermaid.js?v=1', '/@fs/G:/owned/cache/deps/mermaid.js?v=2', '/node_modules/mermaid/dist/mermaid.esm.min.mjs']) {
    assert.equal(isMermaidRuntimeResource(url, base), true);
  }
});
test('built JavaScript requires its explicit verified runtime identity', () => {
  assert.equal(isMermaidRuntimeResource('/assets/mermaid-ABC.js', base), false);
  assert.equal(isMermaidRuntimeResource('/assets/mermaid-ABC.js?x=1', base, ['/assets/mermaid-ABC.js']), true);
  assert.equal(isMermaidRuntimeResource('/assets/editor-ABC.js', base, ['/assets/mermaid-ABC.js']), false);
});
test('invalid runtime identity lists are errors', () => {
  for (const paths of [null, [NaN], [Infinity], ['relative.js'], ['/assets/mermaid.js?guess=1']]) {
    assert.throws(() => isMermaidRuntimeResource('/assets/mermaid.js', base, paths));
  }
});

const script = (type = 'module', body = 'trusted refresh') => ({
  attributes: [{ name: 'type', value: type }], textContent: body, insideDiagram: false,
  closest() { return this.insideDiagram ? {} : null; },
  getAttribute(name) { return this.attributes.find((a) => a.name === name)?.value ?? null; },
});
function withScripts(run) {
  const previous = global.document;
  const elements = [script(), script('module', '')];
  elements[1].attributes.push({ name: 'src', value: '/src/main.tsx' });
  global.document = { querySelectorAll(selector) { assert.equal(selector, 'script'); return elements; } };
  try { run(elements); } finally {
    if (previous === undefined) delete global.document;
    else global.document = previous;
  }
}
test('unchanged shell scripts are accepted by identity, full attributes and body', () => withScripts(() => {
  const baseline = inspectScriptBoundary();
  assert.equal(inspectScriptBoundary(baseline), 0);
}));
test('a new inline module is rejected, including the same trusted text', () => withScripts((els) => {
  const baseline = inspectScriptBoundary(); els.push(script());
  assert.equal(inspectScriptBoundary(baseline), 1);
}));
test('changing a baseline module body is rejected', () => withScripts((els) => {
  const baseline = inspectScriptBoundary(); els[0].textContent = 'changed';
  assert.equal(inspectScriptBoundary(baseline), 1);
}));
test('changing full attributes, including adding src, is rejected', () => withScripts((els) => {
  const baseline = inspectScriptBoundary(); els[0].attributes.push({ name: 'src', value: '/payload.js' });
  assert.equal(inspectScriptBoundary(baseline), 1);
}));
test('a replacement script cannot borrow a baseline fingerprint', () => withScripts((els) => {
  const baseline = inspectScriptBoundary(); els[0] = script();
  assert.equal(inspectScriptBoundary(baseline), 1);
}));
test('a baseline script moved into a diagram is rejected', () => withScripts((els) => {
  const baseline = inspectScriptBoundary(); els[0].insideDiagram = true;
  assert.equal(inspectScriptBoundary(baseline), 1);
}));
test('an unavailable or non-finite script baseline fails loudly', () => withScripts(() => {
  for (const value of [null, NaN, Infinity, []]) assert.throws(() => inspectScriptBoundary(value));
}));

function rowsAt(start = '2026-11-15') {
  return Array.from({ length: 20 }, (_, i) => ({
    id: 'r' + (i + 1),
    values: { prio: ['hi', 'md', 'lo'][i % 3], ...([9, 13].includes(i) ? {} : { due: { start } }) },
  }));
}
const november = {
  months: ['2026-11'], legend: ['High', 'Medium', 'Low'],
  series: [0, 1, 2].map((index) => ({ index, polylines: 1, points: 1, labels: ['2026-11: 6. Click to view data'] })),
};
test('midmonth fixture has exactly one month and six dated rows per priority', () => {
  assert.deepEqual(expectedFixtureLine(rowsAt()), november);
  assert.equal(assertFixtureLine(rowsAt(), november), true);
});
test('year boundary uses raw start dates, not a fixed two-month total', () => {
  const rows = rowsAt('2027-01-01'); rows[0].values.due = { start: '2026-12-31', end: '2027-12-31' };
  assert.deepEqual(expectedFixtureLine(rows), {
    months: ['2026-12', '2027-01'], legend: ['High', 'Medium', 'Low'],
    series: [0, 1, 2].map((index) => ({
      index, polylines: 1, points: 2,
      labels: index === 0 ? ['2026-12: 1. Click to view data', '2027-01: 5. Click to view data'] : ['2026-12: 0. Click to view data', '2027-01: 6. Click to view data'],
    })),
  });
});
test('leap-day timestamps stay in their stated calendar month', () => {
  assert.deepEqual(expectedFixtureLine(rowsAt('2028-02-29T10:00')).months, ['2028-02']);
  assert.throws(() => expectedFixtureLine(rowsAt('2027-02-29')));
});
test('empty, incomplete and duplicate row fixtures fail', () => {
  assert.throws(() => expectedFixtureLine([]));
  assert.throws(() => expectedFixtureLine(rowsAt().slice(1)));
  const rows = rowsAt(); rows[1].id = 'r1'; assert.throws(() => expectedFixtureLine(rows));
});
test('non-finite, missing dated values and unknown priorities fail', () => {
  for (const date of [NaN, Infinity, '2026-13-01']) assert.throws(() => expectedFixtureLine(rowsAt(date)));
  const missing = rowsAt(); delete missing[0].values.due; assert.throws(() => expectedFixtureLine(missing));
  const unknown = rowsAt(); unknown[0].values.prio = 'other'; assert.throws(() => expectedFixtureLine(unknown));
});
test('the actual line grader rejects missing or extra points', () => {
  for (const delta of [-1, 1]) {
    const bad = structuredClone(november); bad.series[0].points += delta;
    assert.throws(() => assertFixtureLine(rowsAt(), bad));
  }
});
test('the actual line grader rejects disabled, reordered or wrong-valued output', () => {
  assert.throws(() => assertFixtureLine(rowsAt(), null));
  const reordered = structuredClone(november); reordered.legend.reverse();
  assert.throws(() => assertFixtureLine(rowsAt(), reordered));
  const wrong = structuredClone(november); wrong.series[0].labels[0] = '2026-11: 5. Click to view data';
  assert.throws(() => assertFixtureLine(rowsAt(), wrong));
});
