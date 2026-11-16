// Development checks only. Record failures without interrupting runner cleanup.
function finiteNumbers(value) {
  if (typeof value === 'number') return Number.isFinite(value);
  if (Array.isArray(value)) return value.every(finiteNumbers);
  if (value && typeof value === 'object') return Object.values(value).every(finiteNumbers);
  return true;
}

function createChecks(name) {
  let executed = 0;
  let failed = 0;
  function fail(label, detail = '') {
    failed++;
    if (!process.exitCode) process.exitCode = 1;
    console.log('FAIL', label, detail);
    return false;
  }
  function check(label, condition, detail = '') {
    executed++;
    return condition === true || fail(label, detail);
  }
  function near(label, actual, expected, tolerance, inclusive = true) {
    const finite = [actual, expected, tolerance].every(Number.isFinite) && tolerance >= 0;
    const distance = Math.abs(actual - expected);
    return check(label, finite && (inclusive ? distance <= tolerance : distance < tolerance),
      'actual=' + actual + ' expected=' + expected + ' tolerance=' + tolerance);
  }
  function equal(label, actual, expected) {
    return check(label, finiteNumbers(actual) && finiteNumbers(expected)
      && JSON.stringify(actual) === JSON.stringify(expected));
  }
  function selection(selected, allowed) {
    if (!Array.isArray(selected) || selected.length === 0) {
      fail('Selection must contain at least one case group');
      return [];
    }
    if (new Set(selected).size !== selected.length || selected.some((item) => !allowed.includes(item))) {
      fail('Unknown or repeated case group', selected.join(','));
      return [];
    }
    return selected;
  }
  function finish() {
    if (executed === 0) fail('No checks ran');
    if (!failed && process.exitCode) fail('An earlier operation already failed');
    console.log(failed ? 'FAILURES: ' + failed + ' (' + executed + ' checks; ' + name + ')'
      : 'ALL OK (' + executed + ' checks; ' + name + ')');
    return { executed, failed };
  }
  return { check, near, equal, selection, finish };
}

module.exports = { createChecks };
