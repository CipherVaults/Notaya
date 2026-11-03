// Checks right-to-left text in the page editor, in the mock UI. Development only; not part of the app.
// Needs Playwright. From the project root:
//   npm run mock            (leave it running; it serves http://127.0.0.1:1421/)
//   node dev-mock/rtl-ui-check.cjs [screenshot folder]
// Prints ALL OK when every check passes.
const { chromium } = require('playwright');
const shots = process.argv[2];
let fail = 0;
const is = (name, got, want) => { if (got !== want) { fail++; console.log('FAIL', name, JSON.stringify(got), '!==', JSON.stringify(want)); } else console.log('ok  ', name); };
const dirOf = (loc) => loc.first().evaluate((e) => e.getAttribute('dir') + '/' + getComputedStyle(e).direction);
(async () => {
  const b = await chromium.launch();
  const p = await b.newPage({ viewport: { width: 1400, height: 1300 }, deviceScaleFactor: 1 });
  p.on('pageerror', (e) => { fail++; console.log('FAIL page error', e.message); });

  // A page saved before this feature has no "dir" anywhere: it is automatic, so English stays left.
  await p.goto('http://127.0.0.1:1421/?cover=none');
  await p.waitForTimeout(1500);
  await p.getByText('Reading list').first().click();
  await p.waitForTimeout(800);
  const pm = p.locator('.ProseMirror');
  is('an old page: English paragraph is left-to-right', await dirOf(pm.locator('p', { hasText: 'Slept badly' })), 'ltr/ltr');
  is('an old page: the bullet list is left-to-right', await dirOf(pm.locator('li').first()), 'ltr/ltr');

  // A page with Arabic, Hebrew and English.
  await p.goto('http://127.0.0.1:1421/?rtl=1&cover=none');
  await p.waitForTimeout(1500);
  await p.getByText('ملاحظات الصباح').first().click();
  await p.waitForTimeout(800);
  const textEdge = (loc) => loc.first().evaluate((e) => {
    const range = document.createRange(); range.selectNodeContents(e);
    const r = range.getBoundingClientRect(); const pr = e.closest('.ProseMirror').getBoundingClientRect();
    return Math.abs(r.right - pr.right) < Math.abs(r.left - pr.left) ? 'right' : 'left';
  });
  is('Arabic paragraph is right-to-left', await dirOf(pm.locator('p', { hasText: 'هذه فقرة عربية تبدأ' })), 'rtl/rtl');
  is('Arabic paragraph sits on the right', await textEdge(pm.locator('p', { hasText: 'هذه فقرة عربية تبدأ' })), 'right');
  is('English paragraph is left-to-right', await dirOf(pm.locator('p', { hasText: 'This is an English' })), 'ltr/ltr');
  is('English paragraph sits on the left', await textEdge(pm.locator('p', { hasText: 'This is an English' })), 'left');
  is('Hebrew paragraph is right-to-left', await dirOf(pm.locator('p', { hasText: 'זוהי פסקה' })), 'rtl/rtl');
  is('Arabic heading is right-to-left', await dirOf(pm.locator('h2')), 'rtl/rtl');
  is('an explicit "align left" still wins over the direction', await textEdge(pm.locator('p', { hasText: 'محاذاة صريحة' })), 'left');
  is('English text forced right-to-left sits on the right', await textEdge(pm.locator('p', { hasText: 'forced right-to-left' })), 'right');
  is('an Arabic bullet item is right-to-left', await dirOf(pm.locator('ul:not([data-type]) li', { hasText: 'البند الأول' })), 'rtl/rtl');
  is('an English bullet item in the same list is left-to-right', await dirOf(pm.locator('ul:not([data-type]) li', { hasText: 'English item' })), 'ltr/ltr');
  is('the quote follows its text', await dirOf(pm.locator('blockquote')), 'rtl/rtl');
  is('the highlight box follows its text', await dirOf(pm.locator('.callout')), 'rtl/rtl');
  is('the shelf follows its text', await dirOf(pm.locator('.toggle[data-shelf="true"]')), 'rtl/rtl');
  const tableDir = (n) => pm.locator('.tableWrapper').nth(n).evaluate((e) => getComputedStyle(e.querySelector('table')).direction);
  is('the Arabic table runs right-to-left', await tableDir(0), 'rtl');
  is('the English table runs left-to-right', await tableDir(1), 'ltr');
  const firstCellSide = (n) => pm.locator('.tableWrapper').nth(n).evaluate((e) => {
    const c = e.querySelector('th').getBoundingClientRect(); const t = e.querySelector('table').getBoundingClientRect();
    return Math.abs(c.right - t.right) < Math.abs(c.left - t.left) ? 'right' : 'left';
  });
  is('the first column of the Arabic table is on the right', await firstCellSide(0), 'right');
  is('the first column of the English table is on the left', await firstCellSide(1), 'left');
  if (shots) await p.screenshot({ path: `${shots}/editor-rtl-full.png` });

  // Typing Arabic on a new line turns it right-to-left; typing English turns it back.
  await pm.locator(':scope > p').last().click();
  await p.keyboard.type('مرحبا بالعالم');
  is('typing Arabic makes the line right-to-left', await dirOf(pm.locator('p', { hasText: 'مرحبا بالعالم' })), 'rtl/rtl');

  // The ribbon forces a direction, and a second click goes back to automatic.
  const ribbon = p.locator('[role=region][aria-label=Ribbon]');
  const ltrBtn = ribbon.getByRole('button', { name: /^Left-to-right/ });
  const rtlBtn = ribbon.getByRole('button', { name: /^Right-to-left/ });
  await ltrBtn.click();
  is('the ribbon forces left-to-right on Arabic text', await dirOf(pm.locator('p', { hasText: 'مرحبا بالعالم' })), 'ltr/ltr');
  is('Left-to-right shows as pressed', await ltrBtn.getAttribute('aria-pressed'), 'true');
  await ltrBtn.click();
  is('a second click goes back to automatic (Arabic, right-to-left again)', await dirOf(pm.locator('p', { hasText: 'مرحبا بالعالم' })), 'rtl/rtl');
  await rtlBtn.click();
  is('Right-to-left shows as pressed', await rtlBtn.getAttribute('aria-pressed'), 'true');
  await rtlBtn.click();

  // The "/" menu.
  await p.keyboard.press('Enter');
  await p.keyboard.type('English line');
  is('English line is left-to-right', await dirOf(pm.locator('p', { hasText: 'English line' })), 'ltr/ltr');
  await p.keyboard.press('Enter');
  await p.keyboard.type('/Right-to');
  await p.waitForTimeout(300);
  await p.keyboard.press('Enter');
  await p.keyboard.type('forced');
  is('the "/" menu forces right-to-left', await dirOf(pm.locator('p', { hasText: /^forced$/ })), 'rtl/rtl');

  // Several blocks at once.
  await p.keyboard.press('Control+a');
  await ltrBtn.click();
  is('select all + Left-to-right turns the Arabic heading left-to-right', await dirOf(pm.locator('h2')), 'ltr/ltr');
  is('... and the Arabic bullet item, so the bullet flips', await dirOf(pm.locator('ul:not([data-type]) li', { hasText: 'البند الأول' })), 'ltr/ltr');
  if (shots) await p.screenshot({ path: `${shots}/editor-rtl-forced-ltr.png` });

  // The selection toolbar's alignment pop-up has the direction too.
  const line = pm.locator('p', { hasText: 'English line' });
  await line.dblclick();
  await p.getByRole('button', { name: 'Align text' }).click();
  await p.getByRole('button', { name: 'Right-to-left', exact: true }).click();
  is('the selection toolbar forces right-to-left', await dirOf(line), 'rtl/rtl');
  await line.dblclick();
  await p.getByRole('button', { name: 'Align text' }).click();
  await p.getByRole('button', { name: 'Automatic direction' }).click();
  is('... and goes back to automatic', await dirOf(line), 'ltr/ltr');

  // Titles and lists show Arabic right-aligned.
  is('the page title input is automatic', await p.locator('input[placeholder=Untitled]').getAttribute('dir'), 'auto');
  await b.close();
  console.log(fail ? `${fail} FAILED` : 'ALL OK');
  process.exit(fail ? 1 : 0);
})();
