// Checks the Deg/Rad switch of the big calculator in the mock UI. Development only; not part of the app.
// Needs Playwright (npm i -g playwright, or any copy that require('playwright') finds). From the project root:
//   npm run mock            (leave it running; it serves http://127.0.0.1:1421/)
//   node dev-mock/calc-ui-check.cjs
// Prints ALL OK when every check passes.
const { chromium } = require('playwright');
let fail = 0;
const is = (name, got, want) => { if (got !== want) { fail++; console.log('FAIL', name, JSON.stringify(got), '!==', JSON.stringify(want)); } else console.log('ok  ', name); };
(async () => {
  const b = await chromium.launch();
  const p = await b.newPage({ viewport: { width: 1400, height: 900 } });
  await p.goto('http://127.0.0.1:1421/');
  await p.waitForTimeout(1500);
  await p.getByText('Sticky notes', { exact: false }).first().click();
  await p.waitForTimeout(800);
  await p.getByRole('button', { name: 'Scientific' }).click();
  const expr = () => p.getByLabel('Expression').inputValue();
  const line = () => p.locator('section[aria-label=Calculator] [aria-live=polite]').innerText();
  const unit = () => p.locator('section[aria-label=Calculator] span[aria-label^="Angles in"]').innerText();
  const click = (name) => p.getByRole('button', { name, exact: true }).click();

  is('the unit label starts as DEG', await unit(), 'DEG');
  await click('Rad'); is('toggling with an empty display changes the label', await unit(), 'RAD');
  await click('Deg'); is('and back', await unit(), 'DEG');

  await p.getByLabel('Expression').fill('sin(30)');
  is('typed sin(30) in Deg', await line(), '= 0.5');
  await click('Rad'); is('typed sin(30), switched to Rad', await line(), '= -0.988031624093');
  await click('Deg'); is('typed sin(30), switched back', await line(), '= 0.5');

  await click('='); is('after = the display holds the answer', await expr(), '0.5');
  await click('Rad');
  is('after =, switching to Rad brings the expression back', await expr(), 'sin(30)');
  is('after =, switching to Rad shows the Rad answer', await line(), '= -0.988031624093');
  await click('='); is('= again keeps the Rad answer', await expr(), '-0.988031624093');
  await click('Deg');
  is('switching to Deg again recalls it', await expr(), 'sin(30)');
  is('and shows the Deg answer', await line(), '= 0.5');

  await p.getByLabel('Expression').fill('asin(0.5)');
  await click('='); is('asin(0.5) in Deg', await expr(), '30');
  await click('Rad'); is('asin(0.5) in Rad', await line(), '= 0.523598775598');

  await p.getByLabel('Expression').fill('2+3');
  await click('='); await click('Deg');
  is('an answer without sin, cos or tan is left alone', await expr(), '5');
  await b.close();
  console.log(fail ? 'FAILURES: ' + fail : 'ALL OK');
  process.exit(fail ? 1 : 0);
})().catch(e => { console.error(e); process.exit(1); });
