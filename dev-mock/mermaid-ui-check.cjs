// Checks the Mermaid diagram block in the page editor, in the mock UI. Development only; not part of the app.
// Needs Playwright. From the project root, either serve the mock:
//   npm run mock            (serves http://127.0.0.1:1421/)
//   node dev-mock/mermaid-ui-check.cjs
// or build it and serve the folder (a build does not reload when files change):
//   npx vite build --config vite.mock.config.ts --outDir <folder>
//   python3 -m http.server 1461 -d <folder>
//   node dev-mock/mermaid-ui-check.cjs http://127.0.0.1:1461/ [screenshot folder]
// The build served from a page with the app's Content-Security-Policy in a meta tag (see the CSP_PAGE note
// in the report) is checked the same way: pass that page's address. Prints ALL OK when every check passes.
const { chromium } = require('playwright');
const base = process.argv[2] || 'http://127.0.0.1:1421/';
const shots = process.argv[3];
let fail = 0;
const is = (name, got, want) => { if (got !== want) { fail++; console.log('FAIL', name, JSON.stringify(got), '!==', JSON.stringify(want)); } else console.log('ok  ', name); };
const yes = (name, got) => is(name, !!got, true);
const url = (q) => base + (base.includes('?') ? '&' : '?') + q;

const HOSTILE = [
  'flowchart TD',
  '  A["<script>window.__pwned=1</script>Alpha"] --> B["<img src=x onerror=\'window.__pwned=2\'>Beta"]',
  '  B --> C["<a href=\'javascript:window.__pwned=3\'>Gamma</a>"]',
  '  C --> D["<iframe src=\'https://evil.test/\'></iframe><svg onload=\'window.__pwned=4\'></svg>Delta"]',
  '  click A href "javascript:window.__pwned=5" _blank',
  '  click B call window.__pwned=6()',
  '  click C "https://evil.test/click"',
  '  %%{init: {"securityLevel": "loose", "themeCSS": "@import url(https://evil.test/x.css); svg{background:url(https://evil.test/p.png)}"}}%%',
].join('\n');
const HOSTILE_SEQ = [
  'sequenceDiagram',
  '  A->>B: <img src=x onerror="window.__pwned=7"> hi',
  '  Note over A,B: <script>window.__pwned=8</script> note',
  '  link A: Site @ javascript:window.__pwned=9',
].join('\n');
const HOSTILE_FM = '---\nconfig:\n  securityLevel: loose\n  themeCSS: "svg{background:url(https://evil.test/p.png)}"\n---\nflowchart LR\n  X --> Y';

(async () => {
  const b = await chromium.launch();
  const p = await b.newPage({ viewport: { width: 1400, height: 1300 }, deviceScaleFactor: 1 });
  const requests = [];
  const navigations = [];
  const violations = [];
  p.on('pageerror', (e) => { fail++; console.log('FAIL page error', e.message); });
  p.on('request', (r) => { const u = r.url(); if (!/^(http:\/\/127\.0\.0\.1|data:|blob:)/.test(u)) requests.push(u); });
  p.on('framenavigated', (f) => { if (f === p.mainFrame()) navigations.push(f.url()); });
  await p.addInitScript(() => {
    window.__csp = [];
    document.addEventListener('securitypolicyviolation', (e) => window.__csp.push(e.violatedDirective + ' ' + e.blockedURI));
    window.addEventListener('beforeunload', () => { window.__unloaded = true; });
  });
  p.on('dialog', async (d) => { fail++; console.log('FAIL a dialog opened:', d.message()); await d.dismiss(); });

  await p.goto(url('cover=none'));
  await p.waitForTimeout(1500);
  const startUrl = p.url();
  await p.getByText('Reading list').first().click();
  await p.waitForTimeout(800);
  const pm = p.locator('.ProseMirror');
  const block = pm.locator('.mermaid-block');
  const area = block.first().locator('textarea.mermaid-code');
  const svgs = () => block.locator('.mermaid-out svg').count();
  const settle = (ms = 1500) => p.waitForTimeout(ms);

  // The library is not loaded until a diagram is first shown.
  const scripts = async () => p.evaluate(() => performance.getEntriesByType('resource').map((r) => r.name).filter((n) => /mermaid|Diagram|cytoscape|elk/i.test(n)).length);
  is('the library is not loaded before a diagram is shown', await scripts(), 0);

  // 1. Insert from the "/" menu.
  await pm.waitFor();
  await p.evaluate(() => document.querySelector('.ProseMirror').editor.commands.focus('end'));
  await p.waitForTimeout(300);
  await p.keyboard.press('Enter');
  await p.keyboard.type('/Diagram');
  await p.waitForTimeout(400);
  if (shots) await p.screenshot({ path: `${shots}/mermaid-slash.png` });
  await p.keyboard.press('Enter');
  await p.waitForFunction(() => document.querySelector('.mermaid-block .mermaid-out svg'), null, { timeout: 20000 });
  is('a diagram block is inserted', await block.count(), 1);
  is('it shows a picture', await svgs(), 1);
  yes('the picture holds the sample text', (await block.locator('.mermaid-out').textContent()).includes('Ship it'));
  yes('the code box is open and has the sample', (await area.inputValue()).startsWith('flowchart TD'));
  is('the code box is left to right and monospace', await area.evaluate((e) => getComputedStyle(e).direction + '/' + /mono|consolas/i.test(getComputedStyle(e).fontFamily)), 'ltr/true');
  yes('the library is now loaded', (await scripts()) > 0);
  is('no script, handler or link in the picture', await p.evaluate(() => document.querySelectorAll('.mermaid-out script, .mermaid-out a, .mermaid-out iframe, .mermaid-out img, .mermaid-out [onclick], .mermaid-out [onload], .mermaid-out [onerror]').length), 0);
  is('the picture is bounded by the page width', await block.locator('.mermaid-out svg').evaluate((e) => e.getBoundingClientRect().width <= e.closest('.ProseMirror').getBoundingClientRect().width), true);
  if (shots) await p.screenshot({ path: `${shots}/mermaid-flowchart.png` });

  // 2. Tab inserts spaces; the sample picker works for every kind.
  await area.click();
  await p.keyboard.press('Control+Home');
  await p.keyboard.press('Tab');
  yes('Tab typed spaces, not a focus change', (await area.inputValue()).startsWith('  flowchart'));
  await p.keyboard.press('Control+z');
  yes('undo works inside the code box', (await area.inputValue()).startsWith('flowchart'));
  for (const [key, label, text] of [
    ['sequence', 'Sequence', 'Open a page'], ['class', 'Class', 'Notebook'], ['state', 'State', 'Review'], ['gantt', 'Gantt', 'Develop'],
    ['pie', 'Pie', 'Meetings'], ['er', 'ER', 'NOTEBOOK'], ['mindmap', 'Mindmap', 'Diagrams'], ['git', 'Git graph', 'release'], ['flowchart', 'Flowchart', 'Ship it'],
  ]) {
    await block.locator('select.mermaid-templates').selectOption(key);
    try {
      await p.waitForFunction((t) => document.querySelector('.mermaid-block .mermaid-out')?.textContent.includes(t), text, { timeout: 20000 });
      is('sample ' + label + ' draws', true, true);
    } catch { is('sample ' + label + ' draws', false, true); }
    is('sample ' + label + ' has no error', await block.locator('.mermaid-error').isVisible(), false);
    if (shots) await p.screenshot({ path: `${shots}/mermaid-${key}.png`, clip: await block.boundingBox() });
  }

  // 3. Invalid code shows an error and the page keeps working.
  await area.fill('flowchart TD\n  A --> \n  B[[[ oops');
  await p.waitForFunction(() => document.querySelector('.mermaid-block .mermaid-error:not([hidden])'), null, { timeout: 10000 });
  yes('invalid code shows an error message', (await block.locator('.mermaid-error').textContent()).length > 5);
  if (shots) await p.screenshot({ path: `${shots}/mermaid-error.png`, clip: await block.boundingBox() });
  is('the old picture stays, dimmed', await block.locator('.mermaid-out').getAttribute('data-stale'), 'true');
  await area.fill('this is not a diagram at all');
  await p.waitForFunction(() => document.querySelector('.mermaid-block .mermaid-error:not([hidden])')?.textContent.length > 3, null, { timeout: 10000 });
  yes('non-diagram text shows an error', await block.locator('.mermaid-error').isVisible());
  await area.fill('');
  await settle(1200);
  is('empty code shows neither a picture nor an error', (await svgs()) + (await block.locator('.mermaid-error').isVisible() ? 1 : 0), 0);

  // 4. Hostile code.
  const hostile = async (name, code) => {
    await area.fill(code);
    await settle(2500);
    const r = await p.evaluate(() => ({
      pwned: window.__pwned ?? null,
      bad: document.querySelectorAll('script:not([src]), .mermaid-block script, .mermaid-block iframe, .mermaid-block object, .mermaid-block embed, .mermaid-block img, .mermaid-block a, .mermaid-block foreignObject iframe').length,
      handlers: [...document.querySelectorAll('.mermaid-out *')].filter((e) => [...e.attributes].some((a) => /^on/i.test(a.name))).length,
      hrefs: [...document.querySelectorAll('.mermaid-out *')].filter((e) => ['href', 'xlink:href', 'src'].some((n) => e.hasAttribute(n) && !(e.getAttribute(n) || '').startsWith('#'))).length,
      outside: [...document.querySelectorAll('.mermaid-out style')].filter((s) => /@import|url\(\s*['"]?(?!#)/i.test(s.textContent)).length + [...document.querySelectorAll('.mermaid-out [style]')].filter((e) => /url\(\s*['"]?(?!#)/i.test(e.getAttribute('style'))).length,
      drawn: document.querySelectorAll('.mermaid-out svg').length,
    }));
    is(name + ': nothing ran', r.pwned, null);
    is(name + ': no script/iframe/img/link elements', r.bad, 0);
    is(name + ': no event-handler attributes', r.handlers, 0);
    is(name + ': no outside addresses', r.hrefs + r.outside, 0);
    return r;
  };
  const h1 = await hostile('hostile flowchart', HOSTILE);
  is('hostile flowchart still draws something (labels as text)', h1.drawn, 1);
  yes('the label text is shown as plain text', (await block.locator('.mermaid-out').textContent()).includes('Alpha'));
  yes('the settings lines were reported as ignored', await block.locator('.mermaid-note').isVisible());
  // Click everything in the picture: nothing may navigate, open a dialog or run.
  for (const el of await block.locator('.mermaid-out svg *').elementHandles()) { try { await el.click({ timeout: 300, force: true }); } catch { /* not clickable */ } }
  await settle(500);
  is('clicking the picture ran nothing', await p.evaluate(() => window.__pwned ?? null), null);
  is('and went nowhere', p.url(), startUrl);
  await hostile('hostile sequence', HOSTILE_SEQ);
  await hostile('hostile front matter', HOSTILE_FM);
  is('front matter with settings still draws', await svgs(), 1);
  is('no request left this machine', requests.join(' '), '');
  is('the window was never navigated or unloaded', navigations.length + ((await p.evaluate(() => window.__unloaded)) ? 1 : 0), 1);

  // 5. A diagram that is too long is refused, with a message.
  await p.evaluate((n) => {
    const ed = document.querySelector('.ProseMirror').editor;
    ed.commands.insertContentAt(ed.state.doc.content.size, { type: 'mermaidBlock', attrs: { code: 'flowchart TD\n' + 'A-->B\n'.repeat(n), showCode: false } });
  }, 10000);
  await p.waitForFunction(() => [...document.querySelectorAll('.mermaid-error')].some((e) => !e.hidden && /longer than/.test(e.textContent)), null, { timeout: 10000 });
  is('code over the limit is refused with a message', true, true);
  await p.evaluate(() => { const ed = document.querySelector('.ProseMirror').editor; let at = -1; ed.state.doc.descendants((n, o) => { if (n.type.name === 'mermaidBlock' && n.attrs.code.length > 50000) at = o; }); if (at >= 0) ed.view.dispatch(ed.state.tr.delete(at, at + ed.state.doc.nodeAt(at).nodeSize)); });
  await p.waitForFunction(() => document.querySelectorAll('.mermaid-block').length === 1, null, { timeout: 5000 });
  is('the textarea itself stops at the limit', await area.getAttribute('maxlength'), '50000');

  // 6. Typed fence, and an imported-style block (code hidden).
  await area.fill('flowchart LR\n  S --> T');
  await settle(1000);
  await p.evaluate(() => { const ed = document.querySelector('.ProseMirror').editor; ed.commands.focus('end'); });
  await p.waitForTimeout(300); // the editor takes the focus a moment later
  await p.keyboard.press('Enter');
  await p.keyboard.type('```mermaid ');
  await p.waitForFunction(() => document.querySelectorAll('.mermaid-block').length === 2, null, { timeout: 5000 });
  is('typing ```mermaid and a space makes a diagram block', await block.count(), 2);
  await p.evaluate(() => {
    const ed = document.querySelector('.ProseMirror').editor;
    ed.commands.insertContentAt(ed.state.doc.content.size, { type: 'mermaidBlock', attrs: { code: 'pie title Imported\n  "A" : 3\n  "B" : 1', showCode: false } });
  });
  await p.waitForFunction(() => document.querySelectorAll('.mermaid-block .mermaid-out svg').length >= 2, null, { timeout: 15000 });
  const imported = block.last();
  is('an imported diagram starts with its code hidden', await imported.locator('textarea').isHidden(), true);
  is('its button says Show code', await imported.locator('.mermaid-toggle').textContent(), 'Show code');
  await imported.locator('.mermaid-toggle').click();
  is('Show code opens the code', await imported.locator('textarea').isVisible(), true);
  is('then the button says Hide code', await imported.locator('.mermaid-toggle').textContent(), 'Hide code');
  await imported.locator('.mermaid-toggle').click();
  is('Hide code closes it again', await imported.locator('textarea').isHidden(), true);

  // 7. The look changes: the picture is drawn again in the new colours.
  const fillOf = () => block.first().locator('.mermaid-out svg').evaluate((s) => {
    const n = s.querySelector('.node rect, .node polygon, .node path, rect');
    return n ? getComputedStyle(n).fill : '';
  });
  const before = await fillOf();
  const svgBefore = await block.first().locator('.mermaid-out svg').elementHandle();
  await p.evaluate(() => { document.documentElement.dataset.theme = 'black'; document.documentElement.dataset.mode = 'dark'; });
  await p.waitForFunction((el) => document.querySelector('.mermaid-block .mermaid-out svg') !== el, svgBefore, { timeout: 15000 });
  const after = await fillOf();
  yes('a new look redraws the diagram (' + before + ' -> ' + after + ')', before && after && before !== after);
  if (shots) await p.screenshot({ path: `${shots}/mermaid-dark-page.png` });

  // 8. Saving: the code is kept in the page, and is in its plain text (so search finds it).
  await p.evaluate(() => { document.documentElement.dataset.theme = 'sunrise'; document.documentElement.dataset.mode = 'light'; });
  await settle(2500);
  const text = await p.evaluate(() => document.querySelector('.ProseMirror').editor.getText());
  yes('the plain-text copy holds the diagram code', text.includes('flowchart LR') && text.includes('S --> T'));
  await p.getByText('Trip notes').first().click();
  await settle(1500);
  is('another page has no diagram', await block.count(), 0);
  await p.getByText('Reading list').first().click();
  await p.waitForFunction(() => document.querySelectorAll('.mermaid-block .mermaid-out svg').length >= 2, null, { timeout: 20000 });
  is('the diagrams come back after leaving and returning', await block.count() >= 3, true);
  yes('the code is intact', (await block.first().locator('textarea').inputValue()).includes('S --> T'));

  // 9. Strictly nothing was blocked by the policy (when the page has one) and nothing left the machine.
  const csp = await p.evaluate(() => window.__csp);
  is('no Content-Security-Policy violation was reported', csp.join(' | '), '');
  is('no request left this machine (end)', requests.join(' '), '');

  await b.close();
  console.log(fail ? `FAILED: ${fail}` : 'ALL OK');
  process.exit(fail ? 1 : 0);
})();
