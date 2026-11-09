// Runs the reviewed legacy JS with credentials in this child's environment, never in files/argv.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { format } from 'node:util';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const [file, artifactDir, ...args] = process.argv.slice(2);
if (!['optionalsecurity.mjs', 'restoreflow.mjs', 'lockflow.mjs', 'installed.mjs'].includes(file)) throw Error('Unsupported legacy suite');
if (file !== 'installed.mjs' && !process.env.NOTAYA_TEST_PASSWORD) throw Error('Missing disposable credential');
const sourcePath = path.join(root, 'tools/real-exe-tests', file);
let source = fs.readFileSync(sourcePath, 'utf8');
const sourceHash = crypto.createHash('sha256').update(source).digest('hex');
const needle = 'readFileSync(pwFile, "utf8").trim()';
if (file !== 'installed.mjs') {
  if (!source.includes(needle)) throw Error('Legacy password seam changed; review adapter before running');
  source = source.replace(needle, 'process.env.NOTAYA_TEST_PASSWORD');
}
fs.mkdirSync(artifactDir, { recursive: true });
fs.writeFileSync(path.join(artifactDir, 'adapter.json'), JSON.stringify({ sourcePath, sourceHash, args, changes: ['Read ephemeral password from child environment instead of temp file; assertion output/exit guard.'] }, null, 2));
fs.writeFileSync(path.join(artifactDir, 'executed-source.mjs'), source);
process.argv = [process.execPath, sourcePath, ...args];
let printed = [];
const log = console.log.bind(console);
console.log = (...values) => { printed.push(format(...values)); log(...values); };
const exit = process.exit.bind(process);
process.exit = code => {
  let bad = Number(code || 0) !== 0 || printed.some(line => /^\s*FAIL\b/.test(line));
  if (file !== 'lockflow.mjs') {
    if (!printed.some(line => /^PASS\b/.test(line))) { bad = true; log('FAIL no assertion output'); }
  } else {
    const [, phase, scenario] = args;
    if (phase === 'type') {
      if (!printed.some(line => line.startsWith('typed into editor:') && line.includes(`FLUSHTEST-${scenario}`))) { bad = true; log('FAIL editor did not receive probe text'); }
      if (scenario !== 'close' && !printed.includes('vault locked after click: true')) { bad = true; log('FAIL explicit lock was not observed'); }
    }
    if (phase === 'check') {
      const page = printed.includes(`page contains "FLUSHTEST-${scenario}": true`);
      const sticky = printed.includes(`sticky contains "FLUSHTEST-${scenario}": true`);
      if ((scenario === 'sticky' && !sticky) || (scenario !== 'sticky' && !page)) { bad = true; log('FAIL pending text missing in expected target'); }
    }
    if (phase === 'setup' && (!printed.includes('page: created') || !printed.includes('sticky: created'))) { bad = true; log('FAIL setup did not create both probes'); }
  }
  delete process.env.NOTAYA_TEST_PASSWORD;
  exit(bad ? 1 : 0);
};
await import(`data:text/javascript;base64,${Buffer.from(source).toString('base64')}`);
