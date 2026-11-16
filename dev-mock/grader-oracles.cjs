// Independent, test-only observations. No app imports, I/O, or browser launch.
function isMermaidRuntimeResource(value, base, builtRuntimePaths = []) {
  if (!Array.isArray(builtRuntimePaths) || builtRuntimePaths.some((p) => typeof p !== 'string' || !p.startsWith('/') || /[?#]/.test(p) || !/\.m?js$/i.test(p))) {
    throw new Error('Built runtime identities must be verified JavaScript pathnames');
  }
  let path;
  try { path = decodeURIComponent(new URL(value, base).pathname); } catch { return false; }
  if (!/\.m?js$/i.test(path)) return false;
  // The optimized package entry, not the eager editor/templates named mermaid.ts.
  if (/\/deps\/mermaid(?:[._-][^/]*)?\.m?js$/i.test(path) || /\/node_modules\/mermaid\/.*\.m?js$/i.test(path)) return true;
  // A bundle must supply exact runtime identities from its verified emitted files.
  return builtRuntimePaths.includes(path);
}

// Self-contained so Playwright can run the SAME function in the page. A Map handle
// retains element identity; matching text alone must not allow a replacement script.
function inspectScriptBoundary(baseline) {
  const scripts = [...document.querySelectorAll('script')];
  const fingerprint = (el) => JSON.stringify({
    attributes: [...el.attributes].map((a) => [a.name, a.value]),
    body: el.textContent,
  });
  if (baseline === undefined) return new Map(scripts.map((el) => [el, fingerprint(el)]));
  if (!(baseline instanceof Map)) throw new Error('Script baseline is unavailable');
  return scripts.filter((el) => el.closest('.mermaid-block') || !baseline.has(el) || baseline.get(el) !== fingerprint(el)).length;
}

function expectedFixtureLine(rows) {
  if (!Array.isArray(rows) || rows.length !== 20) throw new Error('Expected the complete 20-row chart fixture');
  const ids = rows.map((r) => r?.id);
  if (new Set(ids).size !== 20 || Array.from({ length: 20 }, (_, i) => 'r' + (i + 1)).some((id) => !ids.includes(id))) {
    throw new Error('Chart fixture row identities changed');
  }
  const priorities = [['hi', 'High'], ['md', 'Medium'], ['lo', 'Low']];
  const counts = new Map();
  let dated = 0;
  for (const row of rows) {
    if (!priorities.some(([id]) => id === row.values?.prio)) throw new Error('Chart fixture priority changed');
    const start = row.values?.due?.start;
    if (start === undefined) continue;
    if (typeof start !== 'string') throw new Error('Chart fixture date is not text');
    const parts = /^(\d{4})-(\d{2})-(\d{2})(?:T(?:[01]\d|2[0-3]):[0-5]\d)?$/.exec(start);
    if (!parts) throw new Error('Chart fixture date is malformed');
    const [, year, month, day] = parts;
    const date = new Date(Date.UTC(Number(year), Number(month) - 1, Number(day)));
    if (date.getUTCFullYear() !== Number(year) || date.getUTCMonth() + 1 !== Number(month) || date.getUTCDate() !== Number(day)) {
      throw new Error('Chart fixture date is invalid');
    }
    const key = year + '-' + month;
    if (!counts.has(key)) counts.set(key, { hi: 0, md: 0, lo: 0 });
    counts.get(key)[row.values.prio]++;
    dated++;
  }
  if (dated !== 18 || !counts.size) throw new Error('Expected 18 dated fixture rows');
  const months = [...counts.keys()].sort();
  return {
    months,
    legend: priorities.map(([, name]) => name),
    series: priorities.map(([id], index) => ({
      index,
      polylines: 1,
      points: months.length,
      labels: months.map((month) => month + ': ' + counts.get(month)[id] + '. Click to view data'),
    })),
  };
}

function assertFixtureLine(rows, actual) {
  require('node:assert/strict').deepStrictEqual(actual, expectedFixtureLine(rows));
  return true;
}

module.exports = { isMermaidRuntimeResource, inspectScriptBoundary, expectedFixtureLine, assertFixtureLine };
