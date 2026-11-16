// Checks the automations dialog and the Button property in the mock UI. Development only; needs Playwright. From the project root:
//   npm run mock            (leave it running; it serves http://127.0.0.1:1421/)
//   node dev-mock/automations-ui-check.cjs [screenshot folder]
// Prints ALL OK when every check passes. The data is the mock's, in memory; nothing real is touched.
const { chromium } = require('playwright');
const { createChecks } = require('../tools/checks/fail-loud.cjs');
const checks = createChecks('automations-ui');
const shots = process.argv.slice(2).find((a) => !a.startsWith('--'));
const is = (name, got, want) => {
  if (typeof got === 'number' || typeof want === 'number') return checks.near(name, got, want, 0);
  return checks.check(name, got === want, JSON.stringify(got) + ' !== ' + JSON.stringify(want));
};
const yes = (name, got) => is(name, (typeof got !== 'number' || Number.isFinite(got)) && !!got, true);

(async () => {
  const b = await chromium.launch();
  try {
  const ctx = await b.newContext({ viewport: { width: 1400, height: 900 }, deviceScaleFactor: 1 });
  const p = await ctx.newPage();
  p.on('pageerror', (e) => checks.check('page error', false, e.message));
  const shot = async (name) => { if (shots) await p.screenshot({ path: `${shots}/auto-${name}.png`, caret: 'initial' }); };
  const wait = (ms = 250) => p.waitForTimeout(ms);
  const table = () => p.getByRole('application', { name: 'Table' });

  await p.goto('http://127.0.0.1:1421/?db=1');
  await wait(1200);
  await p.getByRole('treeitem', { name: 'Tasks' }).click();
  await wait(900);

  // ---- an automation: a new row gets a note and a message
  await p.getByRole('button', { name: 'Database menu' }).click();
  await p.getByRole('menuitem', { name: /Automations/ }).click();
  await wait();
  const dlg = p.getByRole('dialog');
  yes('the dialog opens', await dlg.getByText('Automations').count());
  await shot('empty');
  await dlg.getByRole('button', { name: 'New automation' }).click();
  await wait();
  await dlg.getByLabel('Automation name').fill('Greet new rows');
  // The starter action is "Change a property"; make it show a message instead.
  await dlg.getByLabel('Action', { exact: true }).selectOption('notify');
  await dlg.getByLabel('Message').fill('New task: {title}');
  await shot('rowAdded-notify');
  yes('the trigger says a row is added', await dlg.getByLabel('Trigger').inputValue() === 'rowAdded');
  await dlg.getByRole('button', { name: 'Add an action' }).click();
  await wait();
  const actionSel = dlg.getByLabel('Action', { exact: true });
  await actionSel.nth(1).selectOption('setProp');
  await wait(150);
  const propSel = dlg.getByLabel('Property', { exact: true }).first();
  const propNames = await propSel.locator('option').allInnerTexts();
  yes('the property list holds Status', propNames.includes('Status'));
  await propSel.selectOption({ label: 'Status' });
  await dlg.getByLabel('Option').selectOption({ label: 'In progress' });
  await shot('rowAdded-two-actions');
  await p.keyboard.press('Escape');
  await wait(1000);

  // add a row; the automation sets Status and shows the message
  await p.getByRole('button', { name: 'New', exact: true }).first().click();
  await wait(900);
  yes('a message from the automation is shown', await p.getByText(/Greet new rows/).count());
  await shot('after-new-row');

  // ---- a Button property
  await p.getByRole('button', { name: 'Add a property' }).click();
  await wait();
  await p.getByRole('menuitem', { name: /^Button/ }).or(p.getByText('Button', { exact: true })).first().click();
  await wait(500);
  await shot('button-menu');
  yes('the button menu has a label field', await p.getByLabel('Button label').count());
  await p.getByLabel('Button label').fill('Start');
  await p.getByRole('button', { name: 'Add an action' }).click();
  await wait();
  await p.getByLabel('Action', { exact: true }).last().selectOption('setProp');
  await wait(150);
  await p.getByLabel('Property', { exact: true }).last().selectOption({ label: 'Done' });
  await p.getByLabel('Checkbox').last().selectOption('true');
  await shot('button-configured');
  await p.keyboard.press('Escape');
  await wait(1200);
  const btn = table().getByRole('button', { name: 'Start' }).first();
  yes('the button shows in the table', await btn.count());
  await btn.click();
  await wait(900);
  await shot('after-press');
  } catch (error) {
    checks.check('automations-ui scenario', false, error.message);
  } finally {
    await b.close();
    checks.finish();
  }
})().catch((error) => { checks.check('automations-ui runner', false, error.message); checks.finish(); });
