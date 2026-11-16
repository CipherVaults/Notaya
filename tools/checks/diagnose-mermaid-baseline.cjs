// Diagnostic only: identify the unchanged grader's matches; no qualification pass.
const path = require('node:path');
const http = require('node:http');
const { guardedBrowser, approvedBrowser } = require('./baseline-browser-check.cjs');
const { need, same, allowedRun, plainExisting, ownedEnvironment } = require('./baseline-server.cjs');

async function diagnose(core, chrome, run, nonce) {
  const root = path.resolve(__dirname, '../..');
  need(same(core, path.resolve(root, '../../qualification/p6c-node-restore-20261112T014330Z-a0694618/tools/node_modules/playwright-core/index.js')), 'Unexpected Core');
  need(approvedBrowser(chrome) && allowedRun(run, root), 'Unapproved browser/run');
  need(/^[a-f0-9]{32}$/.test(nonce), 'Owned server nonce required');
  for (const file of [root, core, chrome, run]) plainExisting(file);
  ownedEnvironment(run);
  const origin = 'http://127.0.0.1:1421';
  const identity = await new Promise((resolve, reject) => {
    const request = http.get(origin + '/__notaya_baseline_identity', { agent: false }, response => {
      let text = '';
      response.on('data', chunk => { text += chunk; if (text.length > 4096) request.destroy(new Error('Identity too large')); });
      response.on('end', () => { try { need(response.statusCode === 200, 'Bad identity status'); resolve(JSON.parse(text)); } catch (error) { reject(error); } });
      response.on('error', reject);
    });
    request.setTimeout(5000, () => request.destroy(new Error('Identity timeout')));
    request.on('error', reject);
  });
  need(identity.nonce === nonce && identity.mode === 'mock' && same(identity.root, root) && identity.origin === origin, 'Foreign server');
  const denied = [];
  let browser;
  try {
    browser = await guardedBrowser(require(core).chromium, {}, { chrome, run, origin }, reason => denied.push(reason));
    const page = await browser.newPage({ viewport: { width: 1400, height: 1300 } });
    await page.goto(origin + '/?cover=none');
    await page.getByText('Reading list').first().click();
    await page.locator('.ProseMirror').waitFor();
    const inspect = async label => {
      const evidence = await page.evaluate(() => ({
        resourceMatches: performance.getEntriesByType('resource').map(r => r.name).filter(n => /mermaid|Diagram|cytoscape|elk/i.test(n)).map(n => new URL(n).pathname),
        badMatches: [...document.querySelectorAll('script:not([src]), .mermaid-block script, .mermaid-block iframe, .mermaid-block object, .mermaid-block embed, .mermaid-block img, .mermaid-block a, .mermaid-block foreignObject iframe')].map(e => ({
          tag: e.tagName, type: e.getAttribute('type'), insideDiagram: !!e.closest('.mermaid-block'),
          reactRefreshPreamble: e.tagName === 'SCRIPT' && /injectIntoGlobalHook|RefreshRuntime/.test(e.textContent),
        })),
        diagramCount: document.querySelectorAll('.mermaid-out svg').length,
        hostileMarker: window.__pwned ?? null,
      }));
      console.log('DIAGNOSTIC ' + label + ' ' + JSON.stringify(evidence));
    };
    await inspect('before-diagram');
    await page.evaluate(() => {
      const editor = document.querySelector('.ProseMirror').editor;
      editor.commands.insertContentAt(editor.state.doc.content.size, { type: 'mermaidBlock', attrs: {
        code: '---\nconfig:\n  securityLevel: loose\n  themeCSS: "svg{background:url(https://evil.test/p.png)}"\n---\nflowchart LR\n  X --> Y', showCode: false,
      } });
    });
    await page.waitForSelector('.mermaid-out svg', { timeout: 20000 });
    await inspect('after-frontmatter-diagram');
  } finally {
    if (browser) await browser.close();
  }
  need(denied.length === 0, 'Diagnostic observed denied network attempts: ' + JSON.stringify(denied));
  console.log('DIAGNOSTIC_COMPLETE; unchanged failed baseline remains FAILED; no security or feature acceptance');
}
if (require.main === module) {
  const [core, chrome, run, nonce, ...extra] = process.argv.slice(2);
  Promise.resolve().then(() => { need(extra.length === 0, 'Unexpected arguments'); return diagnose(core, chrome, run, nonce); })
    .catch(error => { console.error('FAIL diagram diagnostic:', error.stack); process.exitCode = 1; });
}
module.exports = { diagnose };
