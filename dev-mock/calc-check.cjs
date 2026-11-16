// Development only. Node 24+ reads the current standalone TypeScript engine;
// no stale .calc-check output or dependency installation is needed.
// Run: node dev-mock/calc-check.cjs [--only=expressions,angles,...]
// W already supplied fail-on-exit and fresh-source loading. Preserve its intent
// while using Node's built-in TypeScript support and the shared check helper.
const { createChecks } = require('../tools/checks/fail-loud.cjs');
const GROUPS = ['expressions', 'angles', 'angle-dependencies', 'errors', 'formatting', 'negation'];

function runCalculatorChecks({ engine, groups = GROUPS } = {}) {
  const checks = createChecks('calculator');
  const selected = checks.selection(groups, GROUPS);
  if (!selected.length) return checks.finish();
  const { evaluate, formatNumber, negateExpr, dependsOnAngleUnit, CalcError } =
    engine ?? require('../src/features/sticky/calc.ts');
  const d = (s, o) => evaluate(s, { degrees: true, ans: 0, ...o });
  const chk = (name, got, want) => checks.near(name, got, want, 1e-9);
  if (selected.includes('expressions')) {
    const cases = [
      ['1+2*3', 7], ['(1+2)*3', 9], ['2^3^2', 512], ['-2^2', -4], ['2^-1', 0.5], ['10/4', 2.5],
      ['0.1+0.2', 0.3], ['200+10%', 220], ['200-10%', 180], ['50%', 0.5], ['10%*200', 20], ['200*10%', 20],
      ['5!', 120], ['0!', 1], ['3(4+1)', 15], ['2π', 6.283185307179586], ['2pi', 6.283185307179586], ['sqrt(16)', 4], ['√9', 3], ['sqrt 16', 4],
      ['abs(-3)', 3], ['ln(e)', 1], ['log(1000)', 3], ['exp(0)', 1], ['2e3', 2000], ['2e', 2*Math.E], ['1.5e-3', 0.0015],
      ['7÷2', 3.5], ['3×4', 12], ['9−4', 5], ['  1 +  1 ', 2], ['(2+3)(4+5)', 45], ['2sin(0)', 0], ['10 mod 3', null],
    ];
    for (const [src, want] of cases) {
      let got;
      let error;
      try { got = d(src); } catch (caught) { error = caught; }
      if (want === null) checks.check(src + ': expected calculator error', error instanceof CalcError);
      else if (error) checks.check(src, false, error.message);
      else checks.near(src, got, want, 1e-9 * Math.max(1, Math.abs(want)), false);
    }
  }
  if (selected.includes('angles')) {
    chk('sin90deg', d('sin(90)'), 1); chk('cos90deg', d('cos(90)'), 0);
    chk('sin180deg', d('sin(180)'), 0); chk('tan45', d('tan(45)'), 1);
    chk('sin rad', d('sin(pi/2)', { degrees: false }), 1);
    chk('asin1 deg', d('asin(1)'), 90); chk('atan1 deg', d('atan(1)'), 45);
    chk('acos0 rad', d('acos(0)', { degrees: false }), Math.PI / 2);
    chk('ans', d('ans*2', { ans: 21 }), 42);
  }
  if (selected.includes('angle-dependencies')) {
    for (const [x, want] of [['sin(30)',true], ['asin(0.5)',true], ['2cos(60)+1',true], ['TAN(45)',true], ['2+3',false], ['sqrt(4)',false], ['ln(e)',false], ['sin(ans)',false], ['',false]]) {
      checks.equal('dependsOnAngleUnit ' + x, dependsOnAngleUnit(x), want);
    }
  }
  if (selected.includes('errors')) {
    for (const bad of ['1/0','sqrt(-1)','ln(0)','1+','(1+2','1+2)','foo','2**3','abs','1..2','5.5!','171!','tan(90)','asin(2)','','3 $ 4']) {
      let error;
      try { d(bad); } catch (caught) { error = caught; }
      checks.check('calculator error ' + JSON.stringify(bad), error instanceof CalcError);
    }
  }
  if (selected.includes('formatting')) {
    for (const [v, want] of [[0.1+0.2,'0.3'], [1/3,'0.333333333333'], [1e21,'1e21'], [123456789012345,'123456789012345'], [2.5,'2.5'], [-0,'0'], [1e-10,'1e-10'], [100,'100'], [1234.5678,'1234.5678']]) {
      checks.equal('format ' + v, formatNumber(v), want);
    }
  }
  if (selected.includes('negation')) {
    for (const [x, want] of [['-2+3','-(-2+3)'], ['-2','2'], ['-2.5','2.5'], ['-(2+3)','2+3'], ['-(2)+(3)','-(-(2)+(3))'], ['5','-(5)'], ['','-'], ['-','']]) {
      checks.equal('negateExpr ' + JSON.stringify(x), negateExpr(x), want);
    }
    chk('negate -2+3 evaluates', d(negateExpr('-2+3')), -1);
  }
  return checks.finish();
}

if (require.main === module) {
  const args = process.argv.slice(2);
  const groups = args.length === 0 ? GROUPS
    : args.length === 1 && args[0].startsWith('--only=') ? args[0].slice(7).split(',').filter(Boolean) : [];
  runCalculatorChecks({ groups });
}
module.exports = { runCalculatorChecks };
