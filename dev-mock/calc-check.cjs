// Checks for the calculator engine (src/features/sticky/calc.ts). Development only; not part of the app.
// Run from the project root in PowerShell:
//   node dev-mock/calc-check.cjs
// Uses the installed TypeScript (repo or parent); preserves compiled evidence in TEMP.
const fs = require('node:fs');
const path = require('node:path');
const ts = require('typescript');
const compiled = fs.mkdtempSync(path.join(require('node:os').tmpdir(), 'notaya-calc-'));
const result = ts.transpileModule(fs.readFileSync(path.join(__dirname, '../src/features/sticky/calc.ts'), 'utf8'), {
  compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2020 },
});
const entry = path.join(compiled, 'calc.cjs');
fs.writeFileSync(entry, result.outputText);
const { evaluate, formatNumber, negateExpr, dependsOnAngleUnit, CalcError } = require(entry);
const cases = [
 ['1+2*3', 7], ['(1+2)*3', 9], ['2^3^2', 512], ['-2^2', -4], ['2^-1', 0.5], ['10/4', 2.5],
 ['0.1+0.2', 0.3], ['200+10%', 220], ['200-10%', 180], ['50%', 0.5], ['10%*200', 20], ['200*10%', 20],
 ['5!', 120], ['0!', 1], ['3(4+1)', 15], ['2π', 6.283185307179586], ['2pi', 6.283185307179586], ['sqrt(16)', 4], ['√9', 3], ['sqrt 16', 4],
 ['abs(-3)', 3], ['ln(e)', 1], ['log(1000)', 3], ['exp(0)', 1], ['2e3', 2000], ['2e', 2*Math.E], ['1.5e-3', 0.0015],
 ['7÷2', 3.5], ['3×4', 12], ['9−4', 5], ['  1 +  1 ', 2], ['(2+3)(4+5)', 45], ['2sin(0)', 0], ['10 mod 3', null],
];
let fail=0;
for (const [src, want] of cases) {
  try { const got = evaluate(src, {degrees:true, ans:0}); const ok = want!==null && Math.abs(got-want) < 1e-9*Math.max(1,Math.abs(want)); if(!ok){fail++; console.log('FAIL', src, got, want);} }
  catch(e){ if(want===null){ console.log('ok (error as expected)', src, '->', e.message);} else {fail++; console.log('FAIL (threw)', src, e.message);} }
}
// degrees / radians
const d=(s,o)=>evaluate(s,Object.assign({degrees:true,ans:0},o||{}));
const chk=(name,got,want)=>{ if(Math.abs(got-want)>1e-9){fail++;console.log('FAIL',name,got,want);} };
chk('sin90deg',d('sin(90)'),1); chk('cos90deg',d('cos(90)'),0); chk('sin180deg',d('sin(180)'),0); chk('tan45',d('tan(45)'),1);
chk('sin rad',d('sin(pi/2)',{degrees:false}),1); chk('asin1 deg',d('asin(1)'),90); chk('atan1 deg',d('atan(1)'),45); chk('acos0 rad',d('acos(0)',{degrees:false}),Math.PI/2);
chk('ans',d('ans*2',{ans:21}),42);
// which expressions the Deg/Rad switch can change (the display brings them back after "=")
for (const [x,w] of [['sin(30)',true],['asin(0.5)',true],['2cos(60)+1',true],['TAN(45)',true],['2+3',false],['sqrt(4)',false],['ln(e)',false],['sin(ans)',false],['',false]]) { if (dependsOnAngleUnit(x)!==w) { fail++; console.log('FAIL dependsOnAngleUnit',JSON.stringify(x),!w,w); } }
// errors
for (const bad of ['1/0','sqrt(-1)','ln(0)','1+','(1+2','1+2)','foo','2**3','abs','1..2','5.5!','171!','tan(90)','asin(2)','','3 $ 4']) {
  try { const v=evaluate(bad,{degrees:true,ans:0}); fail++; console.log('FAIL should error:',JSON.stringify(bad),'got',v); } catch(e){ if(!(e instanceof CalcError)){fail++;console.log('FAIL non-CalcError',bad,e);} else console.log('error ok:',JSON.stringify(bad),'->',e.message); }
}
// formatting
const fmt=[[0.1+0.2,'0.3'],[1/3,'0.333333333333'],[1e21,'1e21'],[123456789012345,'123456789012345'],[2.5,'2.5'],[-0,'0'],[1e-10,'1e-10'],[100,'100'],[1234.5678,'1234.5678']];
for(const [v,w] of fmt){ const g=formatNumber(v); if(g!==w){fail++;console.log('FAIL fmt',v,g,w);} }
// negateExpr (the +/- key)
for(const [x,w] of [['-2+3','-(-2+3)'],['-2','2'],['-2.5','2.5'],['-(2+3)','2+3'],['-(2)+(3)','-(-(2)+(3))'],['5','-(5)'],['','-'],['-','']]){ const g=negateExpr(x); if(g!==w){fail++;console.log('FAIL negateExpr',JSON.stringify(x),JSON.stringify(g),JSON.stringify(w));} }
chk('negate -2+3 evaluates',d(negateExpr('-2+3')),-1);
console.log(fail? ('FAILURES: '+fail) : 'ALL OK');
process.exitCode = fail ? 1 : 0;
