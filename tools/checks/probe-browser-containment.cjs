// Runtime qualification using only two owned synthetic loopback endpoints.
// This does not launch Notaya, read a vault or contact an external service.
const assert = require('node:assert/strict');
const http = require('node:http');
const path = require('node:path');
const { approvedBrowser, guardedBrowser } = require('./baseline-browser-check.cjs');
const { allowedRun, same, need, plainExisting, ownedEnvironment } = require('./baseline-server.cjs');

async function listen(server) {
  await new Promise((resolve, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', resolve); });
  return 'http://127.0.0.1:' + server.address().port;
}
async function close(server) {
  if (!server.listening) return;
  server.closeAllConnections();
  await new Promise((resolve, reject) => server.close(error => error ? reject(error) : resolve()));
  assert.equal(server.listening, false);
}
async function probe(core, chrome, run) {
  need(same(core, path.resolve(__dirname, '../../../../qualification/p6c-node-restore-20261112T014330Z-a0694618/tools/node_modules/playwright-core/index.js')), 'Use the qualified Core');
  need(approvedBrowser(chrome), 'Use an explicitly approved Chromium browser');
  need(allowedRun(run, path.resolve(__dirname, '../..')), 'Use an explicitly approved runs folder');
  for (const file of [core, chrome, run]) plainExisting(file);
  ownedEnvironment(run);
  console.log('BASELINE_BROWSER_EXECUTABLE ' + path.resolve(chrome));
  const positive = http.createServer((req, res) => { res.writeHead(200); res.end('owned positive'); });
  let decoyConnections = 0;
  const decoy = http.createServer((req, res) => { res.writeHead(200); res.end('unexpected decoy contact'); });
  decoy.on('connection', () => decoyConnections++);
  const observations = [];
  let browser;
  try {
    const origin = await listen(positive);
    const decoyOrigin = await listen(decoy);
    const raw = require(core);
    browser = await guardedBrowser(raw.chromium, { timeout: 30000 }, { chrome, run, origin }, message => observations.push(message));
    const page = await browser.newPage();
    const response = await page.goto(origin, { timeout: 15000 });
    assert.equal(response.status(), 200);
    assert.equal(await page.textContent('body'), 'owned positive');
    assert.deepEqual(observations, []);
    console.log('PASS explicit approved Chromium browser reaches only the explicitly allowed owned HTTP endpoint');

    // Deliberately bypass the context HTTP route. The request observer and
    // deny-only proxy must still prevent contact with the second owned server.
    await page.route('**/*', route => route.continue());
    let blocked;
    try { blocked = await page.goto(decoyOrigin, { timeout: 10000 }); } catch {}
    assert.equal(decoyConnections, 0);
    assert.ok(observations.includes('Unexpected page request origin'));
    assert.ok(observations.includes('HTTP request reached deny-only proxy'));
    if (blocked) assert.equal(blocked.status(), 403);
    console.log('PASS page.route override cannot reach the owned foreign-port HTTP decoy');

    observations.length = 0;
    try { await page.goto(decoyOrigin.replace('http:', 'https:'), { timeout: 10000 }); } catch {}
    assert.equal(decoyConnections, 0);
    assert.ok(observations.includes('CONNECT reached deny-only proxy'));
    console.log('PASS CONNECT is rejected before any decoy TCP connection');
  } finally {
    try { if (browser) await browser.close(); }
    finally { await close(positive); await close(decoy); }
  }
  console.log('ALL OK (3 actual browser containment probes; no Notaya/product qualification)');
}

if (require.main === module) {
  const [core, chrome, run, ...extra] = process.argv.slice(2);
  Promise.resolve().then(() => { need(extra.length === 0, 'Unexpected probe arguments'); return probe(core, chrome, run); })
    .catch(error => { console.error('FAIL browser containment probe:', error.stack); process.exitCode = 1; });
}
module.exports = { probe };
