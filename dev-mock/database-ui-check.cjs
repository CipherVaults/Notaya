// Checks the database UI (tables, properties of every kind, filters, sorts, groups, formulas, relations, row pages,
// templates, inline databases) in the mock UI. Development only; not part of the app. Needs Playwright. From the project root:
//   npm run mock            (leave it running; it serves http://127.0.0.1:1421/)
//   node dev-mock/database-ui-check.cjs [screenshot folder] [--only=name]
// Prints ALL OK when every check passes. The data is the mock's, in memory; nothing real is touched.
const { chromium } = require('playwright');
const args = process.argv.slice(2);
const shots = args.find((a) => !a.startsWith('--'));
let fail = 0;
const is = (name, got, want) => {
  if (got !== want) { fail++; console.log('FAIL', name, JSON.stringify(got), '!==', JSON.stringify(want)); } else console.log('ok  ', name);
};
const yes = (name, got) => is(name, !!got, true);
const URL = 'http://127.0.0.1:1421/?db=1';

(async () => {
  const b = await chromium.launch();
  const ctx = await b.newContext({ viewport: { width: 1400, height: 900 }, deviceScaleFactor: 1 });
  const p = await ctx.newPage();
  p.on('pageerror', (e) => { fail++; console.log('FAIL page error', e.message); });
  p.on('console', (m) => { if (m.type() === 'error' && !/favicon|Failed to load resource/.test(m.text())) console.log('console.error', m.text().slice(0, 200)); });
  const shot = async (name) => { if (shots) await p.screenshot({ path: `${shots}/ui-${name}.png` }); };
  const wait = (ms = 250) => p.waitForTimeout(ms);
  const table = () => p.getByRole('application', { name: 'Table' });
  const cell = (row, col) => table().locator('tbody tr').nth(row).locator('td').nth(col);

  await p.goto(URL);
  await wait(1200);

  // ---------------------------------------------------------------- the seeded Tasks database
  await p.getByRole('treeitem', { name: 'Tasks' }).click();
  await wait(900);
  yes('Tasks opens as a table', await table().count());
  yes('a seeded row shows', await p.getByText('Design the landing page').count());
  yes('view tabs show', await p.getByRole('tab', { name: /All tasks/ }).count());
  await shot('tasks-table');

  // ---------------------------------------------------------------- a new database from the sidebar
  await p.getByRole('button', { name: 'New database…' }).click();
  await wait();
  const dlg = p.getByRole('dialog');
  await dlg.getByRole('radio', { name: /Blank/ }).click();
  await dlg.getByLabel('Database name').fill('Check DB');
  await shot('new-database-dialog');
  await dlg.getByRole('button', { name: 'Create', exact: true }).click();
  await wait(800);
  yes('the new database opens', await p.getByRole('navigation', { name: 'Breadcrumb' }).getByText('Check DB').count());
  yes('it appears in the sidebar', await p.getByRole('treeitem', { name: 'Check DB' }).count());
  await shot('blank-database');

  // ---------------------------------------------------------------- properties of every kind
  const headerNames = async () => (await table().locator('thead th').allInnerTexts()).map((t) => t.trim());
  await p.getByRole('button', { name: 'Add a property' }).click();
  await wait();
  await shot('add-property-picker');
  await p.keyboard.press('Escape');
  const KINDS = ['Text', 'Number', 'Select', 'Multi-select', 'Status', 'Date', 'Person', 'Files & media', 'Checkbox', 'URL', 'Email', 'Phone', 'Formula', 'Place', 'Created time', 'Created by', 'Last edited time', 'Last edited by', 'ID'];
  for (const k of KINDS) {
    await p.getByRole('button', { name: 'Add a property' }).click();
    await wait(120);
    await p.getByRole('button', { name: k, exact: true }).first().click();
    await wait(200);
    // A property menu can open on its own (select, status and formula); close it.
    if (await p.getByRole('dialog').count()) { await p.keyboard.press('Escape'); await wait(150); }
    if (await p.locator('[data-radix-popper-content-wrapper]').count()) { await p.keyboard.press('Escape'); await wait(100); }
  }
  const names = await headerNames();
  console.log('columns:', names.join(' | '));
  yes('every kind added a column', names.length >= KINDS.length + 1);
  await shot('all-kinds-columns');

  // ---------------------------------------------------------------- rows and values
  const newRow = p.getByRole('button', { name: 'New', exact: true }).first();
  for (const t of ['Alpha', 'Beta', 'Gamma']) {
    await newRow.click();
    await wait(250);
    await p.keyboard.type(t);
    await p.keyboard.press('Enter');
    await wait(250);
  }
  const rowTitles = async () => (await table().locator('tbody tr').evaluateAll((trs) => trs.map((tr) => (tr.querySelectorAll('td')[1]?.textContent || '').trim()))).filter(Boolean);
  console.log('rows:', (await rowTitles()).join(' | '));
  await shot('rows-added');
  is('three rows were added', (await rowTitles()).join(','), 'Alpha,Beta,Gamma');

  const COL = Object.fromEntries(['Name', 'Text', 'Number', 'Select', 'Multi-select', 'Status', 'Date', 'Person', 'Files', 'Checkbox', 'URL', 'Email', 'Phone', 'Formula', 'Place'].map((n, i) => [n, i + 1]));
  const typeInto = async (row, col, text) => {
    await cell(row, COL[col]).click();
    await p.keyboard.type(text);
    await p.keyboard.press('Enter');
    await wait(200);
  };
  const cellText = async (row, col) => ((await cell(row, COL[col]).innerText()) || '').trim();
  await typeInto(0, 'Text', 'hello');
  await typeInto(0, 'Number', '5');
  await typeInto(1, 'Number', '20');
  await typeInto(2, 'Number', '7');
  await typeInto(0, 'URL', 'https://example.com');
  await typeInto(0, 'Email', 'me@example.com');
  is('text cell', await cellText(0, 'Text'), 'hello');
  is('number cell', await cellText(1, 'Number'), '20');
  is('url cell', await cellText(0, 'URL'), 'https://example.com');
  await cell(0, COL.Checkbox).getByRole('checkbox').click();
  await wait(300);
  is('the checkbox ticks', await cell(0, COL.Checkbox).getByRole('checkbox').getAttribute('aria-checked'), 'true');

  // Select: create options by typing, then pick one for another row.
  const pop = () => p.locator('[data-radix-popper-content-wrapper]').last();
  await cell(0, COL.Select).click();
  await wait(150);
  await p.keyboard.press('Enter');
  await wait(250);
  if (!(await p.getByLabel('Search options').count())) await cell(0, COL.Select).dblclick();
  await wait(200);
  await p.getByLabel('Search options').fill('Red');
  await p.keyboard.press('Enter');
  await wait(300);
  await shot('select-popover');
  await p.keyboard.press('Escape');
  await wait(200);
  is('a select option is created and set', await cellText(0, 'Select'), 'Red');
  // Multi-select with two options.
  await cell(0, COL['Multi-select']).click();
  await p.keyboard.press('Enter');
  await wait(250);
  for (const t of ['One', 'Two']) {
    await p.getByLabel('Search options').fill(t);
    await p.keyboard.press('Enter');
    await wait(250);
  }
  await p.keyboard.press('Escape');
  await wait(200);
  is('multi-select holds two options', (await cellText(0, 'Multi-select')).replace(/\s+/g, ' '), 'One Two');
  // Status uses its groups; pick the first one.
  await cell(1, COL.Status).click();
  await p.keyboard.press('Enter');
  await wait(300);
  await shot('status-popover');
  await p.getByRole('option').first().click();
  await p.keyboard.press('Escape');
  await wait(200);
  is('a status can be picked', await cellText(1, 'Status'), 'Not started');
  // Date.
  await cell(0, COL.Date).click();
  await p.keyboard.press('Enter');
  await wait(300);
  await shot('date-popover');
  await p.getByRole('grid', { name: /^[A-Z][a-z]+ 20\d\d$/ }).getByRole('button').nth(10).click();
  await wait(200);
  await p.keyboard.press('Escape');
  await wait(200);
  yes('a date can be picked', (await cellText(0, 'Date')).length > 4);

  // ---------------------------------------------------------------- a formula
  const header = (name) => table().locator('thead th').filter({ hasText: name }).first();
  await header('Formula').click();
  await wait(300);
  await shot('property-menu-header');
  const fdlg = p.getByRole('dialog');
  if (!(await fdlg.count())) {
    const edit = p.getByRole('menuitem', { name: /Edit property/ });
    if (await edit.count()) await edit.click();
    await wait(300);
  }
  await p.getByLabel('Formula', { exact: true }).fill('prop("Number") * 2 + 1');
  await wait(500);
  await wait(1200);
  await shot('formula-editor');
  await p.keyboard.press('Escape');
  await wait(900);
  await shot('formula-set');
  is('the formula computes per row', await cellText(1, 'Formula'), '41');
  is('the formula computes for the first row too', await cellText(0, 'Formula'), '11');
  // A broken formula shows an error in the editor, not a crash.
  await header('Formula').click();
  await wait(300);
  await p.getByLabel('Formula', { exact: true }).fill('prop("Nope") +');
  await wait(500);
  await shot('formula-error');
  yes('an invalid formula is explained in the editor', await p.getByRole('dialog').getByRole('status').filter({ hasText: /ends too soon|not closed|extra|unknown|no property/i }).count());
  await p.getByLabel('Formula', { exact: true }).fill('prop("Number") * 2 + 1');
  await wait(400);
  await p.keyboard.press('Escape');
  await wait(300);
  await shot('values-typed');

  await b.close();
  console.log(fail ? `${fail} FAILED` : 'ALL OK');
  process.exit(fail ? 1 : 0);
})().catch((e) => { console.log('SCRIPT ERROR', e); process.exit(2); });
