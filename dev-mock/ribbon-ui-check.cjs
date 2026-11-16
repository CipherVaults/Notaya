// Checks the page editor's ribbon, and that the "/" menu still works next to it, in the mock UI.
// Development only; not part of the app. Needs Playwright. From the project root:
//   npm run mock            (leave it running; it serves http://127.0.0.1:1421/)
//   node dev-mock/ribbon-ui-check.cjs [screenshot folder]
// Prints ALL OK when every check passes.
const { chromium } = require('playwright');
const { createChecks } = require('../tools/checks/fail-loud.cjs');
const checks = createChecks('ribbon-ui');
const shots = process.argv[2];
const is = (name, got, want) => {
  if (typeof got === 'number' || typeof want === 'number') return checks.near(name, got, want, 0);
  return checks.check(name, got === want, JSON.stringify(got) + ' !== ' + JSON.stringify(want));
};
(async () => {
  const b = await chromium.launch();
  try {
  const p = await b.newPage({ viewport: { width: 1400, height: 900 } });
  p.on('pageerror', (e) => checks.check('page error', false, e.message));
  await p.goto('http://127.0.0.1:1421/');
  await p.waitForTimeout(1500);
  await p.getByText('Reading list').first().click();
  await p.waitForTimeout(800);
  const ribbon = p.locator('[role=region][aria-label=Ribbon]');
  const pm = p.locator('.ProseMirror');
  const btn = (name) => ribbon.getByRole('button', { name, exact: true });
  const tab = (name) => ribbon.getByRole('tab', { name, exact: true });
  is('the ribbon shows on a page', await ribbon.count(), 1);
  is('Home is the open tab', await tab('Home').getAttribute('aria-selected'), 'true');
  is('no Table tab outside a table', await tab('Table').count(), 0);
  if (shots) await p.screenshot({ path: `${shots}/ribbon-home.png` });

  // Type a new line, then make it bold and a heading from the ribbon.
  await pm.click();
  await p.keyboard.press('Control+End');
  await p.keyboard.press('Enter');
  await p.keyboard.type('Ribbon line');
  await p.keyboard.press('Shift+Home');
  await btn('Bold (Ctrl+B)').click();
  is('Bold makes the selection bold', await pm.locator('strong', { hasText: 'Ribbon line' }).count(), 1);
  is('Bold shows as pressed', await btn('Bold (Ctrl+B)').getAttribute('aria-pressed'), 'true');
  await btn('Large heading').click();
  is('the Large heading style makes it a heading', await pm.locator('h1', { hasText: 'Ribbon line' }).count(), 1);
  await btn('Undo (Ctrl+Z)').click();
  is('Undo takes the heading back off', await pm.locator('h1', { hasText: 'Ribbon line' }).count(), 0);
  await btn('Redo (Ctrl+Y)').click();
  is('Redo puts it back', await pm.locator('h1', { hasText: 'Ribbon line' }).count(), 1);

  // A pick from a drop-down leaves the cursor in the page.
  await p.keyboard.press('End');
  await btn('Text size').click();
  await p.getByRole('button', { name: '24', exact: true }).click();
  await p.keyboard.type('!');
  is('after picking a size, typing still goes into the page', await pm.locator('h1', { hasText: 'Ribbon line!' }).count(), 1);

  // Lists and indent.
  await p.keyboard.press('End');
  await p.keyboard.press('Enter');
  await btn('Bullet list').click();
  await p.keyboard.type('one');
  await p.keyboard.press('Enter');
  await p.keyboard.type('two');
  await btn('Increase indent (Tab)').click();
  is('Increase indent nests the item', await pm.locator('ul ul li', { hasText: 'two' }).count(), 1);
  await btn('Decrease indent (Shift+Tab)').click();
  is('Decrease indent brings it back', await pm.locator('ul ul li', { hasText: 'two' }).count(), 0);
  await p.keyboard.press('Enter');
  await p.keyboard.press('Enter');

  // Insert tab: a table, then the contextual Table tab.
  await tab('Insert').click();
  if (shots) await p.screenshot({ path: `${shots}/ribbon-insert.png` });
  const tables = await pm.locator('table').count();
  await btn('Insert a 3 by 3 table').click();
  is('Insert > Table adds a table', await pm.locator('table').count(), tables + 1);
  is('the Table tab appears inside a table', await tab('Table').count(), 1);
  await tab('Table').click();
  const t = pm.locator('table').last();
  const rows = await t.locator('tr').count();
  await btn('Insert a row below').click();
  is('Insert row below adds a row', await t.locator('tr').count(), rows + 1);
  const cols = await t.locator('tr').first().locator('th,td').count();
  await btn('Insert a column to the right').click();
  is('Insert column right adds a column', await t.locator('tr').first().locator('th,td').count(), cols + 1);
  if (shots) await p.screenshot({ path: `${shots}/ribbon-table.png` });
  is('Header row shows on from a body cell', await btn('Turn the top row\'s header style on or off').getAttribute('aria-pressed'), 'true');
  await btn('Delete table').click();
  is('Delete table removes it', await pm.locator('table').count(), tables);
  is('and the Table tab goes away', await tab('Table').count(), 0);
  is('falling back to Home', await tab('Home').getAttribute('aria-selected'), 'true');

  // The "/" menu still works with the ribbon on screen.
  await p.keyboard.type('/check');
  await p.waitForTimeout(200);
  is('typing / opens the menu', await p.getByRole('button', { name: /Checklist/ }).count() > 0, true);
  if (shots) await p.screenshot({ path: `${shots}/ribbon-slash.png` });
  await p.keyboard.press('Enter');
  is('Enter in the / menu runs the command', await pm.locator('ul[data-type=taskList]').count() > 0, true);

  // Hiding the ribbon keeps the tabs and is remembered.
  await ribbon.getByRole('button', { name: 'Hide the ribbon' }).click();
  is('Hide leaves only the tab row', await btn('Bold (Ctrl+B)').count(), 0);
  await p.reload();
  await p.waitForTimeout(1200);
  await p.getByText('Reading list').first().click();
  await p.waitForTimeout(800);
  is('the hidden state is remembered', await btn('Bold (Ctrl+B)').count(), 0);
  await tab('Insert').click();
  is('clicking a tab shows the ribbon again', await btn('Add an image from your PC').count(), 1);

  } catch (error) {
    checks.check('ribbon-ui scenario', false, error.message);
  } finally {
    await b.close();
    checks.finish();
  }
})().catch((error) => { checks.check('ribbon-ui runner', false, error.message); checks.finish(); });
