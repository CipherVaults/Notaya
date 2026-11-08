// Start: npm run mock -- --configLoader native --port <free port>
// Then: node dev-mock/lane-a-shots.cjs http://127.0.0.1:<port>
// Uses the installed Playwright/Chrome named by NOTAYA_PLAYWRIGHT_MODULE and
// NOTAYA_CHROMIUM_PATH. No installs, real vault, or real app. The caller owns the server.
// "Before" reconstructs only pre-lane icon styles from c2ead6e on today's mock UI;
// it is not a historical screenshot of the old application or its layout.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { createHash } = require('node:crypto');

const root = path.resolve(__dirname, '..');
const target = new URL(process.argv[2] || 'http://127.0.0.1:1421');
assert(['127.0.0.1', 'localhost', '[::1]'].includes(target.hostname) && target.protocol === 'http:', 'Use the loopback mock server');
const output = path.join('D:/Notaya-lanes/out/A-shots', new Date().toISOString().replace(/[:.]/g, '-'));
fs.mkdirSync(output, { recursive: true });
// Chrome's temporary profile and Playwright artifacts stay with this lane's throwaway data.
const browserTemp = path.join(process.env.NOTAYA_HOME || 'D:/Notaya Tests/v1-6-AF', 'lane-a-browser-temp');
fs.mkdirSync(browserTemp, { recursive: true });
process.env.TEMP = browserTemp;
process.env.TMP = browserTemp;
// Values read from c2ead6e:src/styles/app.css. Other theme colors were unchanged by lane A.
const beforeFaint = { sunrise: '#8f7766', 'sunrise-dark': '#8c7c6e', black: '#737373', graphite: '#777777', white: '#939393', paper: '#85796a' };
const themes = ['sunrise', 'sunrise-dark', 'black', 'graphite', 'white', 'paper'];
const report = {
  started: new Date().toISOString(), head: process.env.NOTAYA_SHOTS_HEAD || 'uncommitted working tree; see file hashes', beforeRef: 'c2ead6e',
  beforeMeaning: 'Pre-lane faint tokens, inherited star color and menu-icon opacity reconstructed on current mock UI. Layout and star fill behavior are current code.',
  target: target.href, files: {}, checks: [], screenshots: [], pageErrors: [], status: 'running',
};
for (const file of ['src/styles/app.css', 'src/features/database/DatabasePage.tsx', 'src/features/database/DatabaseView.tsx', 'src/components/Sidebar.tsx', 'src/components/TitleBar.tsx', 'src/store/app.ts', 'src/lib/navHistory.ts', 'src-tauri/src/commands/vault.rs', 'dev-mock/lane-a-shots.cjs']) {
  report.files[file] = createHash('sha256').update(fs.readFileSync(path.join(root, file))).digest('hex');
}
let browser;
const manifest = () => fs.writeFileSync(path.join(output, 'manifest.json'), JSON.stringify(report, null, 2));

async function snapshot(page, name) {
  await page.mouse.move(1100, 750); // Do not reveal hover-only controls in the resting screenshots.
  for (const [region, locator] of [['window', null], ['sidebar', page.locator('aside')], ['topbar', page.locator('.title-bar')]]) {
    const file = `${name}-${region}.png`;
    if (locator) await locator.screenshot({ path: path.join(output, file), animations: 'disabled' });
    else await page.screenshot({ path: path.join(output, file), animations: 'disabled' });
    report.screenshots.push(file);
  }
  manifest();
}

async function verifyStar(page, on) {
  const button = page.getByRole('button', { name: on ? 'Remove from favorites' : 'Add to favorites', exact: true });
  await button.waitFor();
  assert.equal(await button.getAttribute('aria-pressed'), String(on));
  assert.equal(await button.locator('svg').getAttribute('fill'), on ? 'currentColor' : 'none');
  const matches = await button.locator('svg').evaluate((svg, active) => {
    const probe = document.createElement('span');
    probe.style.color = active ? 'var(--favorite)' : 'var(--muted)';
    svg.parentElement.append(probe);
    const match = getComputedStyle(svg).color === getComputedStyle(probe).color;
    probe.remove();
    return match;
  }, on);
  assert(matches, `Favorite ${on ? 'gold fill' : 'muted outline'} color`);
}

async function functionalChecks(page, databaseId) {
  const top = page.locator('.title-bar');
  for (const name of ['Import', 'Export']) {
    const button = top.getByRole('button', { name, exact: true });
    assert.equal(await button.getAttribute('data-tauri-drag-region'), null);
    await button.click();
    await page.getByRole('dialog').waitFor();
    if (name === 'Import') assert(await page.getByText('Comments and page history are not imported.', { exact: true }).isVisible());
    await page.keyboard.press('Escape');
    await page.getByRole('dialog').waitFor({ state: 'hidden' });
  }
  const newNotebook = page.getByRole('button', { name: 'New notebook', exact: true });
  await newNotebook.focus();
  assert(await newNotebook.evaluate((el) => document.activeElement === el));
  await page.keyboard.press('Enter');
  await page.getByRole('dialog').waitFor();
  await page.keyboard.press('Escape');
  const tabs = page.getByRole('tablist', { name: 'Views' }).getByRole('tab');
  const chosen = await tabs.nth(1).innerText();
  await tabs.nth(1).click();
  await page.waitForFunction(async () => {
    const { invoke } = await import('/dev-mock/core.ts');
    const { useApp } = await import('/src/store/app.ts');
    const saved = (await invoke('get_settings')).lastPlace;
    return saved?.viewId && saved.viewId === useApp.getState().activeDatabaseViewId;
  });
  const place = await page.evaluate(async () => (await (await import('/dev-mock/core.ts')).invoke('get_settings')).lastPlace);
  assert.equal(place.id, databaseId);
  await top.getByRole('button', { name: 'Back', exact: true }).click();
  assert.equal(await tabs.nth(0).getAttribute('aria-selected'), 'true');
  await top.getByRole('button', { name: 'Forward', exact: true }).click();
  assert.equal(await tabs.nth(1).getAttribute('aria-selected'), 'true');
  // Use the mock's in-memory settings and production hydration, as used after unlock.
  await page.evaluate(async () => {
    const { useApp } = await import('/src/store/app.ts');
    useApp.setState({ settings: null, activePageId: null, activeDatabaseViewId: null });
    await useApp.getState().loadVault();
  });
  assert.equal(await page.getByRole('tab', { name: chosen, exact: true }).getAttribute('aria-selected'), 'true');
  assert.equal(await page.evaluate(() => Object.keys(localStorage).some((key) => key.startsWith('dbview:'))), false);
  const rowContext = await page.evaluate(async (id) => {
    const { invoke } = await import('/dev-mock/core.ts');
    const { useApp } = await import('/src/store/app.ts');
    const db = await invoke('get_database', { id });
    const result = await invoke('query_view', { request: { databaseId: id, viewId: db.views[0].id } });
    const tree = useApp.getState().tree;
    const notebookId = tree.sections.find((s) => s.id === db.sectionId).notebookId;
    const other = tree.notebooks.find((n) => n.id !== notebookId);
    useApp.getState().openNotebook(other.id);
    return { tree, title: result.rows[0].title, notebookId, oldNotebookId: other.id, notebookName: tree.notebooks.find((n) => n.id === notebookId).name };
  }, databaseId);
  await top.getByRole('button', { name: 'Search', exact: true }).click();
  await page.getByPlaceholder('Search notes and sticky notes, or type a command').fill(rowContext.title);
  await page.getByRole('option').filter({ hasText: rowContext.title }).first().click();
  const exportButton = top.getByRole('button', { name: 'Export', exact: true });
  await exportButton.click(); // Also waits for row -> database -> notebook resolution.
  const exportState = await page.evaluate(async () => {
    const { useApp } = await import('/src/store/app.ts');
    const s = useApp.getState();
    return { kind: s.activeKind, notebookId: s.activeNotebookId, target: s.exportTarget };
  });
  assert.equal(exportState.kind, 'row');
  assert.equal(exportState.notebookId, rowContext.oldNotebookId);
  assert.deepEqual(exportState.target, { notebookId: rowContext.notebookId, initial: 'notebook' });
  const wholeNotebook = page.getByRole('radio', { name: /Whole notebook/ });
  assert(await wholeNotebook.isChecked());
  assert((await wholeNotebook.locator('..').innerText()).includes(rowContext.notebookName), 'Dialog names the row notebook');
  await page.keyboard.press('Escape');
  await page.evaluate(async (id) => {
    const { useApp } = await import('/src/store/app.ts');
    const tree = useApp.getState().tree;
    useApp.getState().setTree({ ...tree, databases: tree.databases.filter((d) => d.id !== id) });
  }, databaseId);
  await page.waitForFunction(() => document.querySelector('.title-bar button[title^="Export unavailable"]')?.disabled);
  assert(await exportButton.isDisabled(), 'An unresolved row never offers the previously selected notebook');
  await page.evaluate(async ({ tree, id }) => {
    const { useApp } = await import('/src/store/app.ts');
    useApp.getState().setTree(tree);
    useApp.getState().openDatabase(id);
  }, { tree: rowContext.tree, id: databaseId });
  await page.getByRole('tablist', { name: 'Views' }).waitFor();
  const controls = await top.locator('[data-tauri-drag-region]').count();
  assert(controls > 0, 'Drag regions remain in the top bar');
  for (const name of ['Import', 'Export', 'Minimize', 'Maximize', 'Close']) {
    assert(await top.getByRole('button', { name, exact: true }).isVisible());
  }
  report.checks.push({ theme: await page.locator('html').getAttribute('data-theme'), result: 'Import/Export dialogs, cross-notebook search-to-row export scope, unresolved row disabled, keyboard New notebook, database view save/hydration/back/forward, no dbview browser key, top-bar controls and drag attributes passed (mock only)' });
}

(async () => {
  const { chromium } = require(process.env.NOTAYA_PLAYWRIGHT_MODULE || 'playwright-core');
  browser = await chromium.launch({ executablePath: process.env.NOTAYA_CHROMIUM_PATH, headless: true });
  for (const theme of themes) {
    const context = await browser.newContext({ viewport: { width: 1280, height: 800 }, reducedMotion: 'reduce' });
    // Only this local mock origin and in-page resources are needed.
    await context.route('**/*', (route) => new URL(route.request().url()).origin === target.origin ? route.continue() : route.abort());
    const page = await context.newPage();
    await page.addInitScript(() => {
      if (sessionStorage.getItem('af-upgrade-seeded')) return;
      sessionStorage.setItem('af-upgrade-seeded', '1');
      localStorage.setItem('dbview:legacy-one', 'legacy-view');
      localStorage.setItem('dbview:legacy-two', 'another-view');
      localStorage.setItem('notaya.layout', '{"open":true,"width":264}');
      localStorage.setItem('cipherleaf.layout', 'keep-legacy-layout');
      localStorage.setItem('other:dbview:keep', 'keep-unrelated');
    });
    page.on('pageerror', (error) => report.pageErrors.push({ theme, message: error.message }));
    const url = new URL(target);
    url.search = new URLSearchParams({ db: '1', theme }).toString();
    await page.goto(url.href);
    await page.getByRole('button', { name: 'New notebook', exact: true }).waitFor();
    const checkStorage = async () => {
      const keys = await page.evaluate(() => Object.fromEntries(Object.entries(localStorage)));
      assert(!Object.keys(keys).some((key) => key.startsWith('dbview:')), 'Upgrade removes legacy database/view associations');
      assert.equal(keys['notaya.layout'], '{"open":true,"width":264}');
      assert.equal(keys['cipherleaf.layout'], 'keep-legacy-layout');
      assert.equal(keys['other:dbview:keep'], 'keep-unrelated');
    };
    await checkStorage();
    // Exercise the mock lock state, then reload as the real lock event does (mock events are inert).
    await page.evaluate(async () => {
      const { invoke } = await import('/dev-mock/core.ts');
      const { useApp } = await import('/src/store/app.ts');
      await invoke('enable_security', { removeUnprotectedCopies: false });
      await invoke('lock_vault');
      await useApp.getState().refreshStatus();
    });
    await checkStorage();
    await page.reload();
    await page.getByRole('button', { name: 'New notebook', exact: true }).waitFor();
    await checkStorage();
    report.checks.push({ theme, result: 'Pre-seeded dbview keys removed; unrelated layout keys preserved after load, mock lock and reload' });
    await page.evaluate(() => document.fonts.ready);
    // Open via the sidebar, rather than replacing production navigation with a test-only route.
    await page.getByRole('treeitem', { name: 'Tasks', exact: true }).click();
    await page.getByRole('tablist', { name: 'Views' }).waitFor();
    const databaseId = await page.locator('[data-database]').getAttribute('data-database');
    await functionalChecks(page, databaseId);
    const before = await page.addStyleTag({ content: `[data-theme="${theme}"] { --faint: ${beforeFaint[theme]}; }
      svg.lucide-star, svg.lucide-star[fill="currentColor"] { color: inherit; }
      span.opacity-70:has(> svg.lucide) { opacity: .7; }` });
    await snapshot(page, `${theme}-before-style-off`);
    await page.getByRole('button', { name: 'Add to favorites', exact: true }).click();
    await page.getByRole('button', { name: 'Remove from favorites', exact: true }).waitFor();
    await snapshot(page, `${theme}-before-style-on`);
    await before.evaluate((el) => el.remove());
    await verifyStar(page, true);
    await snapshot(page, `${theme}-after-on`);
    await page.getByRole('button', { name: 'Remove from favorites', exact: true }).click();
    await verifyStar(page, false);
    await snapshot(page, `${theme}-after-off`);
    // Compact density plus the minimum window width, with all top-bar controls still visible.
    await page.setViewportSize({ width: 760, height: 800 });
    await page.evaluate(async () => {
      const { applyAppearance } = await import('/src/lib/theme.ts');
      const { useApp } = await import('/src/store/app.ts');
      await applyAppearance({ ...useApp.getState().settings, density: 'compact' }, false);
    });
    const boxes = [];
    for (const name of ['Import', 'Export', 'Minimize', 'Maximize', 'Close']) {
      const box = await page.locator('.title-bar').getByRole('button', { name, exact: true }).boundingBox();
      assert(box && box.x >= 0 && box.x + box.width <= 761, `${theme}: ${name} fits at 760px`);
      boxes.push(box);
    }
    for (let i = 1; i < boxes.length; i++) assert(boxes[i].x >= boxes[i - 1].x + boxes[i - 1].width, 'Top-bar controls do not overlap');
    await snapshot(page, `${theme}-after-compact-off`);
    await page.getByRole('button', { name: 'Add to favorites', exact: true }).click();
    await verifyStar(page, true);
    await snapshot(page, `${theme}-after-compact-on`);
    report.checks.push({ theme, result: 'Favorite outline/gold in both densities, top-bar layout at 760px passed' });
    await context.close();
    console.log(`ok ${theme}: before/after styles, favorite off/on, compact layout and mock clicks`);
  }
  assert.deepEqual(report.pageErrors, [], 'No mock page errors');
  report.status = 'passed';
  console.log(`ALL OK (${report.screenshots.length} screenshots): ${output}`);
})().catch((error) => {
  report.status = 'failed';
  report.error = error.stack || String(error);
  console.error(report.error);
  process.exitCode = 1;
}).finally(async () => {
  if (browser) await browser.close();
  report.finished = new Date().toISOString();
  manifest();
  console.log(`Evidence: ${output}`);
});
