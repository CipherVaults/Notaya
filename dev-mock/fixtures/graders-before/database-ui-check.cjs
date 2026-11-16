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
const URL = process.env.MOCK_URL || 'http://127.0.0.1:1421/?db=1';

(async () => {
  const b = await chromium.launch();
  const ctx = await b.newContext({ viewport: { width: 1400, height: 900 }, deviceScaleFactor: 1 });
  const p = await ctx.newPage();
  p.on('pageerror', (e) => { fail++; console.log('FAIL page error', e.message); });
  p.on('console', (m) => { if (m.type() === 'error' && !/favicon|Failed to load resource/.test(m.text())) console.log('console.error', m.text().slice(0, 200)); });
  const shot = async (name) => { if (shots) await p.screenshot({ path: `${shots}/ui-${name}.png`, caret: 'initial' }); }; // (Playwright's default caret hiding redraws the page's React blocks)
  const wait = (ms = 250) => p.waitForTimeout(ms);
  const table = () => p.getByRole('application', { name: 'Table' });
  const cell = (row, col) => table().locator('tbody tr[aria-selected]').nth(row).locator('td').nth(col);

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
  const rowTitles = async () => (await table().locator('tbody tr[aria-selected]').evaluateAll((trs) => trs.map((tr) => (tr.querySelectorAll('td')[1]?.textContent || '').trim()))).filter(Boolean);
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
  await wait(300);
  if (!(await p.locator('[data-radix-popper-content-wrapper]').count())) await p.keyboard.press('Enter'); // a click opens the picker itself since V1.5.1
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
  await wait(300);
  if (!(await p.locator('[data-radix-popper-content-wrapper]').count())) await p.keyboard.press('Enter');
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
  await wait(300);
  if (!(await p.locator('[data-radix-popper-content-wrapper]').count())) await p.keyboard.press('Enter');
  await wait(300);
  await shot('status-popover');
  await p.getByRole('option').first().click();
  await p.keyboard.press('Escape');
  await wait(200);
  is('a status can be picked', await cellText(1, 'Status'), 'Not started');
  // Date.
  await cell(0, COL.Date).click();
  await wait(300);
  if (!(await p.locator('[data-radix-popper-content-wrapper]').count())) await p.keyboard.press('Enter');
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
  await wait(1400);
  await shot('formula-error');
  yes('an invalid formula is explained in the editor', await p.getByRole('dialog').getByRole('status').filter({ hasText: /ends too soon|not closed|extra|unknown|no property/i }).count());
  await p.getByLabel('Formula', { exact: true }).fill('prop("Number") * 2 + 1');
  await wait(400);
  await p.keyboard.press('Escape');
  await wait(300);

  // ---------------------------------------------------------------- sort, filter, group
  const panel = () => p.locator('[data-radix-popper-content-wrapper]').last();
  await p.getByRole('button', { name: 'Sort', exact: true }).click();
  await wait(200);
  await panel().getByRole('button', { name: 'Add sort' }).click();
  await panel().getByLabel('Sort by').selectOption({ label: 'Number' });
  await panel().getByLabel('Direction').selectOption('desc');
  await wait(500);
  await shot('sort-panel');
  is('sorting by Number descending reorders the rows', (await rowTitles()).join(','), 'Beta,Gamma,Alpha');
  await p.keyboard.press('Escape');
  await wait(200);
  yes('the Sort button shows how many sorts are on', await p.getByRole('button', { name: 'Sort', exact: true }).filter({ hasText: '1' }).count());
  await p.getByRole('button', { name: 'Sort', exact: true }).click();
  await wait(200);
  await panel().getByRole('button', { name: 'Remove sort' }).click();
  await wait(400);
  await p.keyboard.press('Escape');
  is('without a sort the rows keep their own order', (await rowTitles()).join(','), 'Alpha,Beta,Gamma');

  await p.getByRole('button', { name: 'Filter', exact: true }).click();
  await wait(200);
  await panel().getByRole('button', { name: 'Add filter', exact: true }).click();
  await panel().getByLabel('Value').fill('et');
  await wait(600);
  await shot('filter-panel');
  is('a "contains" filter on the name narrows the rows', (await rowTitles()).join(','), 'Beta');
  await p.keyboard.press('Escape');
  await wait(200);
  yes('the Filter button shows the count', await p.getByRole('button', { name: 'Filter', exact: true }).filter({ hasText: '1' }).count());
  await p.getByRole('button', { name: 'Filter', exact: true }).click();
  await wait(200);
  await panel().getByRole('button', { name: 'Remove filter' }).click();
  await wait(500);
  await p.keyboard.press('Escape');
  is('removing the filter brings the rows back', (await rowTitles()).join(','), 'Alpha,Beta,Gamma');

  await p.getByRole('button', { name: 'Group', exact: true }).click();
  await wait(200);
  await panel().getByLabel('Group', { exact: true }).selectOption({ label: 'Checkbox' });
  await wait(600);
  await shot('group-panel');
  await p.keyboard.press('Escape');
  await wait(300);
  yes('grouping by a checkbox makes two groups', (await table().getByRole('button', { name: /Collapse group/ }).count()) === 2);
  await shot('grouped-table');
  await p.getByRole('button', { name: 'Group', exact: true }).click();
  await wait(200);
  await panel().getByLabel('Group', { exact: true }).selectOption({ label: 'None' });
  await wait(400);
  await p.keyboard.press('Escape');
  await wait(300);
  is('ungrouping shows no group headers', await table().getByRole('button', { name: /Collapse group/ }).count(), 0);
  await shot('values-typed');


  // ---------------------------------------------------------------- a relation and a rollup
  const colOf = async (name) => (await headerNames()).findIndex((t) => t.startsWith(name));
  const cellBy = async (row, name) => cell(row, await colOf(name));
  const addProp = async (kind) => {
    await p.getByRole('button', { name: 'Add a property' }).click();
    await wait(150);
    await p.getByRole('button', { name: kind, exact: true }).first().click();
    await wait(200);
  };
  await addProp('Relation');
  await p.getByLabel('Related database').selectOption({ label: '✅ Tasks' });
  await shot('relation-new');
  await p.getByRole('button', { name: 'Create', exact: true }).click();
  await wait(900);
  yes('the relation column exists', (await colOf('Relation')) > 0);
  const relCell = await cellBy(0, 'Relation');
  await relCell.click();
  await p.keyboard.press('Enter');
  await wait(300);
  await p.getByRole('button', { name: 'Design the landing page' }).click();
  await wait(200);
  await p.getByRole('button', { name: 'Write the API' }).click();
  await wait(300);
  await shot('relation-picker');
  await p.keyboard.press('Escape');
  await wait(300);
  const relText = (await (await cellBy(0, 'Relation')).innerText()).replace(/\s+/g, ' ');
  yes('the relation cell shows both linked rows', relText.includes('Design the landing page') && relText.includes('Write the API'));
  await addProp('Rollup');
  await p.getByLabel('Rolled up property').selectOption({ label: 'Estimate' });
  await p.getByLabel('Calculation').selectOption({ label: 'Sum' });
  await shot('rollup-new');
  await p.getByRole('button', { name: 'Create', exact: true }).click();
  await wait(1200);
  const roll = ((await (await cellBy(0, 'Rollup')).innerText()) || '').trim();
  console.log('rollup:', roll);
  yes('the rollup sums the Estimate of the linked rows', /^\d+(\.\d+)?$/.test(roll) && Number(roll) > 0);
  await shot('relation-rollup-table');

  // ---------------------------------------------------------------- a row as a page
  await cell(0, 1).hover();
  await table().getByRole('button', { name: 'Open row' }).first().click();
  await wait(900);
  yes('the row opens as a page with its properties', await p.getByRole('textbox', { name: /Title/i }).count() || await p.locator('.ProseMirror').count());
  yes('the breadcrumb leads back to the database', await p.getByRole('navigation', { name: 'Breadcrumb' }).getByText('Check DB').count());
  yes('the property list shows the row values', await p.getByText('hello').count());
  await shot('row-page');
  await p.locator('.ProseMirror').click();
  await p.keyboard.type('Notes about Alpha');
  await wait(1500);
  await p.getByRole('navigation', { name: 'Breadcrumb' }).getByText('Check DB').first().click();
  await wait(900);
  yes('back in the table', await table().count());
  await cell(0, 1).hover();
  await table().getByRole('button', { name: 'Open row' }).first().click();
  await wait(900);
  yes('the page body of the row was saved', await p.locator('.ProseMirror', { hasText: 'Notes about Alpha' }).count());
  await p.getByRole('navigation', { name: 'Breadcrumb' }).getByText('Check DB').first().click();
  await wait(700);

  // ---------------------------------------------------------------- peeks
  const openRowsIn = async (label) => {
    await p.getByRole('button', { name: 'View settings' }).click();
    await wait(200);
    await panel().getByLabel('Open rows in').selectOption({ label });
    await wait(500);
    await p.keyboard.press('Escape');
    await wait(200);
  };
  await openRowsIn('Side peek');
  await cell(1, 1).hover();
  await table().getByRole('button', { name: 'Open row' }).nth(1).click();
  await wait(1000);
  yes('a side peek opens next to the table', await p.getByRole('complementary', { name: 'Row' }).count());
  yes('the side peek shows the row', await p.getByRole('complementary', { name: 'Row' }).getByText('Beta').count());
  await shot('side-peek');
  await p.getByRole('complementary', { name: 'Row' }).getByRole('button', { name: 'Close' }).click();
  await wait(300);
  is('the side peek closes', await p.getByRole('complementary', { name: 'Row' }).count(), 0);
  await openRowsIn('Center peek');
  await cell(1, 1).hover();
  await table().getByRole('button', { name: 'Open row' }).nth(1).click();
  await wait(1000);
  yes('a center peek opens as a dialog', await p.getByRole('dialog').getByText('Beta').count());
  await shot('center-peek');
  await p.keyboard.press('Escape');
  await wait(300);
  await openRowsIn('Full page');

  // ---------------------------------------------------------------- duplicate, trash and undo
  await cell(0, 1).hover();
  await table().getByRole('button', { name: 'Row options' }).first().click();
  await p.getByRole('menuitem', { name: 'Duplicate' }).click();
  await wait(700);
  is('duplicating a row adds one', (await rowTitles()).length, 4);
  await cell(0, 1).hover();
  await table().getByRole('button', { name: 'Row options' }).first().click();
  await p.getByRole('menuitem', { name: 'Move to trash' }).click();
  await wait(600);
  is('trashing a row removes it', (await rowTitles()).length, 3);
  await shot('row-trashed-toast');
  await p.getByRole('button', { name: 'Undo' }).click();
  await wait(800);
  is('Undo brings the row back', (await rowTitles()).length, 4);

  // ---------------------------------------------------------------- templates
  await p.getByRole('button', { name: 'New with a template' }).click();
  await p.getByRole('menuitem', { name: /Manage templates/ }).click();
  await wait(400);
  const tdlg = p.getByRole('dialog', { name: 'Templates' });
  await tdlg.getByRole('button', { name: 'New template' }).click();
  await wait(300);
  await tdlg.getByLabel('Template name').fill('Weekly');
  await wait(300);
  await shot('templates-dialog');
  await p.keyboard.press('Escape');
  await wait(700);
  const rowsBefore = await table().locator('tbody tr[aria-selected]').count();
  await p.getByRole('button', { name: 'New with a template' }).click();
  yes('the new template is offered in the New menu', await p.getByRole('menuitem', { name: /Weekly/ }).count());
  await shot('new-menu');
  await p.getByRole('menuitem', { name: /Weekly/ }).click();
  await wait(900);
  is('a row can be made from a template', await table().locator('tbody tr[aria-selected]').count(), rowsBefore + 1);

  // ---------------------------------------------------------------- inline database and linked view inside a page
  await p.getByRole('treeitem', { name: 'Trip notes' }).click();
  await wait(900);
  const toEnd = () => p.locator('div.cursor-text').last().click(); // the empty space under the text puts the cursor on a new last line
  await toEnd();
  await p.keyboard.type('/database');
  await wait(300);
  await shot('slash-database');
  await p.keyboard.press('Enter');
  await wait(1200);
  const block = p.locator('[data-database-block]');
  is('an inline database block is inserted', await block.count(), 1);
  yes('the inline database shows a table', await block.getByRole('application', { name: 'Table' }).count());
  await block.getByRole('button', { name: 'New', exact: true }).first().click();
  await wait(400);
  await p.keyboard.type('Inline row');
  await p.keyboard.press('Enter');
  await wait(500);
  yes('a row can be added inside the page', await block.getByText('Inline row').count());
  await shot('inline-database');
  // The block survives leaving the page and coming back.
  await p.getByRole('treeitem', { name: 'Reading list' }).click();
  await wait(900);
  await p.getByRole('treeitem', { name: 'Trip notes' }).click();
  await wait(1200);
  is('the block is still in the page', await p.locator('[data-database-block]').count(), 1);
  yes('and still has its row', await p.locator('[data-database-block]').getByText('Inline row').count());
  // A linked view of Tasks.
  await toEnd();
  await p.keyboard.type('/linked');
  await wait(300);
  await p.keyboard.press('Enter');
  await wait(300);
  await p.getByRole('dialog', { name: 'Choose a database' }).getByRole('button', { name: /Tasks/ }).click();
  await p.locator('[data-database-block]', { hasText: 'Design the landing page' }).waitFor({ timeout: 20000 }).catch(() => undefined);
  await wait(300);
  is('a linked view adds a second block', await p.locator('[data-database-block]').count(), 2);
  yes('the linked view shows the Tasks rows', await p.locator('[data-database-block]').nth(1).getByText('Design the landing page').count());
  yes('the linked view got a view of its own', await p.locator('[data-database-block]').nth(1).getByRole('tab', { name: /linked/ }).count());
  await shot('linked-view');

  yes('the inline database is not listed twice in the sidebar tree', (await p.getByRole('treeitem', { name: 'Untitled database' }).count()) === 0);

  // ---------------------------------------------------------------- footers, views, lock
  await p.getByRole('treeitem', { name: 'Check DB' }).click();
  await wait(1000);
  await table().getByRole('button', { name: 'Calculate Number' }).click({ force: true });
  await p.getByRole('menuitem', { name: 'Sum', exact: true }).click();
  await wait(700);
  yes('a footer calculation shows the sum of the column', await table().locator('tfoot').getByText('37').count());
  await shot('footer-sum');
  await p.getByRole('button', { name: 'Add a view' }).click();
  await p.getByRole('menuitem', { name: 'Table', exact: true }).click();
  await wait(700);
  is('a second view is added', await p.getByRole('tab').count(), 2);
  await p.getByRole('tab').nth(0).click();
  await wait(500);
  // Lock: the table can be used but its structure cannot change.
  await p.getByRole('button', { name: 'Database menu' }).click();
  await wait(300);
  await p.getByRole('menuitem', { name: 'Lock database' }).click();
  await wait(700);
  is('a locked database offers no way to add a property', await p.getByRole('button', { name: 'Add a property' }).count(), 0);
  await shot('locked');
  await p.getByRole('button', { name: 'Database menu' }).click();
  await p.getByRole('menuitem', { name: 'Unlock database' }).click();
  await wait(700);
  is('unlocking brings it back', await p.getByRole('button', { name: 'Add a property' }).count(), 1);

  // ---------------------------------------------------------------- search and trash
  await p.keyboard.press('Control+k');
  await wait(300);
  await p.keyboard.type('Check');
  await wait(600);
  yes('the palette finds the database', await p.getByRole('option', { name: /Check DB/ }).count());
  await shot('palette-database');
  await p.keyboard.press('Escape');
  await wait(200);
  await p.getByRole('treeitem', { name: 'Check DB' }).hover();
  await p.getByRole('treeitem', { name: 'Check DB' }).getByRole('button', { name: 'Database options' }).click();
  await p.getByRole('menuitem', { name: 'Move to bin', exact: true }).click();
  await wait(700);
  is('a trashed database leaves the sidebar', await p.getByRole('treeitem', { name: 'Check DB' }).count(), 0);
  await p.getByRole('button', { name: /^Trash/ }).click();
  await wait(700);
  yes('the trash lists the database', await p.getByText('Check DB').count());
  await shot('trash');

  // ---------------------------------------------------------------- panels of the seeded Tasks database (looked at in screenshots)
  await p.getByRole('treeitem', { name: 'Tasks' }).click();
  await wait(900);
  await table().locator('thead th').filter({ hasText: 'Status' }).first().click();
  await wait(400);
  await shot('property-status');
  await p.keyboard.press('Escape');
  await wait(300);
  await p.getByRole('button', { name: 'Properties', exact: true }).click();
  await wait(300);
  await shot('properties-panel');
  await p.keyboard.press('Escape');
  await p.getByRole('button', { name: 'View settings' }).click();
  await wait(300);
  await shot('view-settings');
  await p.keyboard.press('Escape');
  await p.getByRole('button', { name: 'Database menu' }).click();
  await wait(300);
  await shot('database-menu');
  await p.keyboard.press('Escape');
  await wait(200);
  await p.getByRole('button', { name: /^Options for/ }).first().click();
  await wait(300);
  await shot('view-menu');
  await p.keyboard.press('Escape');

  // ---------------------------------------------------------------- looks: dark theme, right-to-left, a 900px window
  const look = async (name, url, size, extra) => {
    const pg = await ctx.newPage();
    pg.on('pageerror', (e) => { fail++; console.log('FAIL page error', e.message); });
    await pg.setViewportSize(size);
    await pg.goto(url);
    await pg.waitForTimeout(1200);
    await pg.getByRole('treeitem', { name: 'Tasks' }).click();
    await pg.waitForTimeout(1000);
    const noScroll = await pg.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1);
    yes(`${name}: the page itself does not scroll sideways`, noScroll);
    if (extra) await extra(pg);
    if (shots) await pg.screenshot({ path: `${shots}/ui-${name}.png`, caret: 'initial' });
    await pg.close();
  };
  await look('dark-table', 'http://127.0.0.1:1421/?db=1&theme=sunrise-dark', { width: 1400, height: 900 });
  await look('narrow-table', 'http://127.0.0.1:1421/?db=1', { width: 900, height: 700 });
  await look('narrow-dark-table', 'http://127.0.0.1:1421/?db=1&theme=sunrise-dark', { width: 900, height: 700 });
  // The app's own text runs left to right, but text in Arabic or Hebrew must read right to left in titles and cells,
  // and the layout must hold if the whole window is mirrored.
  await look('rtl-text', 'http://127.0.0.1:1421/?db=1', { width: 1400, height: 900 }, async (pg) => {
    await pg.getByRole('button', { name: 'New', exact: true }).first().click();
    await pg.waitForTimeout(300);
    await pg.keyboard.type('مهمة جديدة بالعربية');
    await pg.keyboard.press('Enter');
    await pg.waitForTimeout(500);
    const t = pg.getByRole('application', { name: 'Table' }).locator('tbody tr[aria-selected]', { hasText: 'مهمة جديدة' }).first();
    yes('an Arabic row title is saved and shown', await t.count());
    const dir = await t.locator('td').nth(1).evaluate((td) => { const el = td.querySelector('[dir]'); return el ? getComputedStyle(el).direction : 'none'; });
    is('the Arabic title reads right to left', dir, 'rtl');
    // ...and it lines up with the right edge of its cell.
    const edge = await t.locator('td').nth(1).evaluate((td) => { const el = td.querySelector('[dir=auto]'); const range = document.createRange(); range.selectNodeContents(el); const r = range.getBoundingClientRect(); const c = td.getBoundingClientRect(); return Math.round(c.right - r.right); });
    yes('the Arabic title ends at the right side of its cell (the space for the row buttons stays free)', edge >= 0 && edge <= 64);
  });
  await look('rtl-mirrored', 'http://127.0.0.1:1421/?db=1', { width: 1400, height: 900 }, async (pg) => {
    await pg.evaluate(() => document.documentElement.setAttribute('dir', 'rtl'));
    await pg.waitForTimeout(500);
    const taskRight = await pg.evaluate(() => { const th = [...document.querySelectorAll('thead th')].find((t) => t.textContent.trim() === 'Task'); return th ? Math.round(th.getBoundingClientRect().right) : -1; });
    yes('mirrored: the Task column sits at the right edge of the table', taskRight > 900);
    await pg.getByRole('button', { name: 'Filter', exact: true }).click();
    await pg.waitForTimeout(300);
    await pg.getByRole('button', { name: 'Add filter', exact: true }).click();
    await pg.waitForTimeout(300);
    if (shots) await pg.screenshot({ path: `${shots}/ui-rtl-mirrored-filter.png`, caret: 'initial' });
    await pg.keyboard.press('Escape');
  });
  await b.close();
  console.log(fail ? `${fail} FAILED` : 'ALL OK');
  process.exit(fail ? 1 : 0);
})().catch((e) => { console.log('SCRIPT ERROR', e); process.exit(2); });
