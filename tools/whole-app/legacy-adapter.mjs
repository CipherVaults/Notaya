// Runs the reviewed legacy JS with credentials in this child's environment, never in files/argv.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath, pathToFileURL } from 'node:url';
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
// Keep a real source filename in stack traces; data: stacks used to print the
// entire base64 script, burying the actual exception. Resolve its one local import.
source = source.replace('from "./legacy-cdp.mjs"', `from ${JSON.stringify(pathToFileURL(path.join(root, 'tools/real-exe-tests/legacy-cdp.mjs')).href)}`);
fs.mkdirSync(artifactDir, { recursive: true });
fs.writeFileSync(path.join(artifactDir, 'adapter.json'), JSON.stringify({ sourcePath, sourceHash, args, changes: ['Read ephemeral password from child environment instead of temp file; assertion output/exit guard.', 'Resolve shared CDP import to its file URL; execute preserved source as a real file for readable stack traces.'] }, null, 2));
fs.writeFileSync(path.join(artifactDir, 'executed-source.mjs'), source);
process.argv = [process.execPath, sourcePath, ...args];
let printed = [];
const log = console.log.bind(console);
console.log = (...values) => { printed.push(format(...values)); log(...values); };
function guardedExitCode(code) {
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
  return bad ? 1 : 0;
}
// Legacy suites close their sockets in finally and exit naturally on every route.
// Apply assertion guards after every suite has drained its socket close handshake.
process.once('beforeExit', () => { process.exitCode = guardedExitCode(process.exitCode); });
await import(pathToFileURL(path.join(artifactDir, 'executed-source.mjs')).href);
