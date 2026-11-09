// Lane E: run `npm run mock`, then `node dev-mock/ai-ui-check.cjs`.
// Only fake in-memory notes are used. Optional env: NOTAYA_MOCK_URL,
// NOTAYA_PLAYWRIGHT_MODULE (existing module), NOTAYA_CHROMIUM_PATH, NOTAYA_AI_SHOTS.
const assert = require('node:assert/strict');
const path = require('node:path');
const fs = require('node:fs');
const { chromium } = require(process.env.NOTAYA_PLAYWRIGHT_MODULE || 'playwright');
const origin = process.env.NOTAYA_MOCK_URL || 'http://127.0.0.1:1421';
let checks = 0;
const check = (name, actual, expected) => { assert.deepEqual(actual, expected, name); console.log(`ok ${++checks} ${name}`); };

(async () => {
  const browser = await chromium.launch({ executablePath: process.env.NOTAYA_CHROMIUM_PATH || undefined });
  try {
    const page = await browser.newPage({ viewport: { width: 1280, height: 1000 } });
    const errors = [];
    page.on('pageerror', (e) => errors.push(e.message));
    const mock = (method, ...args) => page.evaluate(async ({ method, args }) => {
      const { aiMock } = await import('/dev-mock/ai.ts');
      return aiMock[method](...args);
    }, { method, args });
    const wait = async (predicate) => {
      for (let attempt = 0; attempt < 60; attempt++) {
        if (predicate(await mock('state'))) return;
        await page.waitForTimeout(100);
      }
      throw new Error(`Timed out after 6 seconds waiting for mock state: ${predicate.toString()}`);
    };
    // Reach the real SettingsView from the existing app before using the standalone marker host.
    await page.goto(origin);
    await page.getByRole('button', { name: 'Settings', exact: true }).click();
    await page.getByRole('heading', { name: 'AI access', exact: true }).waitFor();
    check('AI access is reachable in the real Settings view', await page.getByRole('switch', { name: 'AI access', exact: true }).getAttribute('aria-checked'), 'false');
    await page.goto(`${origin}/dev-mock/ai-harness.html`);
    const access = page.getByRole('switch', { name: 'AI access', exact: true });
    await access.waitFor();
    await access.click();
    const dialog = page.getByRole('dialog');
    const enable = dialog.getByRole('button', { name: 'Turn on AI access', exact: true });
    check('enable requires typed ALLOW', await enable.isDisabled(), true);
    await page.getByLabel('Type ALLOW', { exact: true }).fill('allow');
    check('wrong case cannot enable', await enable.isDisabled(), true);
    await page.keyboard.press('Escape');
    check('dismissing enable leaves access Off', (await mock('state')).enabled, false);
    await access.click();
    check('typed confirm resets on reopen', await page.getByLabel('Type ALLOW', { exact: true }).inputValue(), '');
    await page.getByLabel('Type ALLOW', { exact: true }).fill('ALLOW');
    await enable.click();
    await dialog.waitFor({ state: 'hidden' });
    await wait((s) => s.enabled);
    await page.waitForFunction(() => document.querySelector('[role="switch"][aria-label="AI access"]')?.getAttribute('aria-checked') === 'true');
    check('ALLOW enables access', await access.getAttribute('aria-checked'), 'true');
    await mock('pair', 'Writing assistant');
    await dialog.waitFor();
    await dialog.getByRole('button', { name: 'Deny', exact: true }).click();
    await dialog.waitFor({ state: 'hidden' });
    check('denied pairing is not saved', (await mock('state')).pairings.length, 0);
    await mock('pair', '<img src=x onerror=alert(1)>');
    await dialog.waitFor();
    check('client name is rendered as text', await dialog.locator('img').count(), 0);
    await page.keyboard.press('Escape');
    await dialog.waitFor({ state: 'hidden' });
    await mock('pair', 'Writing assistant');
    await dialog.getByRole('button', { name: 'Allow', exact: true }).click();
    await dialog.waitFor({ state: 'hidden' });
    const first = (await mock('state')).pairings[0].id;
    await mock('pair', 'Research assistant');
    await dialog.getByRole('button', { name: 'Allow', exact: true }).click();
    await dialog.waitFor({ state: 'hidden' });
    await wait((s) => s.pairings.length === 2);
    const second = (await mock('state')).pairings[1].id;
    check('new pairings start at Look', (await mock('state')).pairings.map((p) => p.level), ['look', 'look']);
    await mock('pair', 'Writing assistant');
    await dialog.getByRole('alert').filter({ hasText: 'already paired' }).waitFor();
    check('duplicate pairing is refused without a label or content', [await dialog.getByRole('button', { name: 'Allow', exact: true }).isDisabled(), (await dialog.innerText()).includes('Writing assistant')], [true, false]);
    await dialog.getByRole('button', { name: 'Deny', exact: true }).click();
    await dialog.waitFor({ state: 'hidden' });
    const firstCard = page.getByLabel('Access for Writing assistant', { exact: true });
    await firstCard.getByRole('radio', { name: /^Edit/ }).click();
    await wait((s) => s.pairings[0].level === 'edit');
    check('level is per AI', (await mock('state')).pairings.map((p) => p.level), ['edit', 'look']);
    await firstCard.getByRole('radio', { name: /^Full/ }).click();
    await wait((s) => s.pairings[0].level === 'full');
    await mock('fail', 'ai_set_level');
    await firstCard.getByRole('radio', { name: /^Look/ }).click();
    await page.getByRole('alert').filter({ hasText: 'could not be saved' }).waitFor();
    check('failed save keeps the stored level', (await mock('state')).pairings[0].level, 'full');
    check('failed level does not appear selected', await firstCard.getByRole('radio', { name: /^Full/ }).isChecked(), true);
    await mock('fail', 'ai_set_level', 'rate_limited');
    await firstCard.getByRole('radio', { name: /^Look/ }).click();
    await page.getByRole('alert').filter({ hasText: 'Too many AI requests are waiting.' }).waitFor();
    check('window overload error uses fixed plain copy', await page.getByRole('alert').innerText(), 'Too many AI requests are waiting. Wait a moment, then try again.');
    await page.getByRole('checkbox', { name: 'Personal', exact: true }).click();
    await wait((s) => s.hiddenNotebooks.length === 1);
    check('hidden notebook ids are sent', (await mock('state')).hiddenNotebooks, ['n1']);
    const big = page.getByRole('switch', { name: 'Ask before big changes', exact: true });
    check('big-change confirmation defaults on at 20', [(await mock('state')).bigChangeConfirm, (await mock('state')).bigChangeItems], [true, 20]);
    await big.click();
    await wait((s) => !s.bigChangeConfirm);
    await page.waitForFunction(() => document.querySelector('[role="switch"][aria-label="Ask before big changes"]')?.getAttribute('aria-checked') === 'false');
    check('big-change switch can turn off', await big.getAttribute('aria-checked'), 'false');
    await page.getByText('How to connect an AI', { exact: true }).click();
    check('connection help uses the bridge path', await page.locator('pre').filter({ hasText: 'notaya-bridge.exe' }).count(), 2);
    // Dedicated settings cannot be overwritten by the ordinary settings saver.
    await page.evaluate(async () => {
      const { api } = await import('/src/lib/ipc.ts');
      const settings = await api.getSettings();
      await api.saveSettings({ ...settings, ai: { enabled: false } });
    });
    check('ordinary settings save preserves AI access', (await mock('state')).enabled, true);
    await page.getByRole('button', { name: 'Notes', exact: true }).click();
    await mock('confirm', 'ask');
    await dialog.waitFor();
    check('Ask lists every affected item outside Settings', await dialog.getByRole('listitem').allTextContents(), ['Plan', 'Budget', 'Notes']);
    await dialog.getByRole('button', { name: 'Allow', exact: true }).click();
    await dialog.waitFor({ state: 'hidden' });
    check('Ask answer is a window command', (await mock('calls')).filter((c) => c.command === 'ai_confirm_answer').at(-1).args.allow, true);
    await mock('confirm', 'strong');
    await dialog.waitFor();
    const allow = dialog.getByRole('button', { name: 'Allow', exact: true });
    check('Strong starts disabled', await allow.isDisabled(), true);
    await page.getByLabel('Confirmation word', { exact: true }).fill('maple');
    check('Strong requires the exact random word', await allow.isDisabled(), true);
    await page.getByLabel('Confirmation word', { exact: true }).fill('MAPLE-CEDAR');
    await allow.click();
    await dialog.waitFor({ state: 'hidden' });
    check('Strong submits the typed word to the window command', (await mock('calls')).filter((c) => c.command === 'ai_confirm_answer').at(-1).args.word, 'MAPLE-CEDAR');
    await mock('confirm', 'strong', { word: 'CEDAR' });
    await dialog.waitFor();
    check('a new Strong request starts empty', await page.getByLabel('Confirmation word', { exact: true }).inputValue(), '');
    await page.keyboard.press('Escape');
    await dialog.waitFor({ state: 'hidden' });
    check('Escape denies a confirmation', (await mock('calls')).filter((c) => c.command === 'ai_confirm_answer').at(-1).args.allow, false);
    // C5 events have no expiresAt. Age the window's receipt clock instead of
    // adding a fake-only event field that would hide a broken real adapter.
    await page.evaluate(async () => {
      const { aiMock } = await import('/dev-mock/ai.ts');
      const now = Date.now;
      try { Date.now = () => now() - 301_000; aiMock.confirm('ask'); }
      finally { Date.now = now; }
    });
    await dialog.waitFor();
    check('expired requests cannot be allowed', await allow.isDisabled(), true);
    await dialog.getByRole('button', { name: 'Deny', exact: true }).click();
    await dialog.waitFor({ state: 'hidden' });
    await mock('activity', first, ['Plan', 'Budget']);
    await mock('activity', second, ['Notes']);
    const activity = page.getByLabel('AI activity', { exact: true });
    await activity.waitFor();
    await page.getByText('with Writing assistant, Research assistant', { exact: true }).waitFor();
    check('marker shows both active AIs and claims', await activity.locator('[title]').getAttribute('title'), 'Writing assistant: Plan, Budget\nResearch assistant: Notes');
    await activity.getByRole('button', { name: 'Stop', exact: true }).click();
    await activity.getByRole('button', { name: 'Resume AI', exact: true }).waitFor();
    check('Stop pauses and clears connections and pending requests', [(await mock('state')).paused, (await mock('state')).activity.length, (await mock('state')).confirmRequests.length], [true, 0, 0]);
    await activity.getByRole('button', { name: 'Resume AI', exact: true }).click();
    await activity.waitFor({ state: 'hidden' });
    check('Resume reopens AI access without fabricating activity', (await mock('state')).paused, false);
    await mock('activity', first, [], Date.now() - 31_000);
    await page.waitForTimeout(150);
    check('activity older than 30 seconds is hidden', await activity.count(), 0);
    await mock('confirm', 'ask');
    await dialog.waitFor();
    await mock('lock', true);
    await dialog.waitFor({ state: 'hidden' });
    check('lock cancels pending confirmations', (await mock('state')).confirmRequests.length, 0);
    await mock('lock', false);
    await page.getByRole('button', { name: 'Settings', exact: true }).click();
    await firstCard.waitFor();
    await page.getByRole('button', { name: 'Remove Research assistant', exact: true }).click();
    await wait((s) => s.pairings.length === 1);
    check('Remove revokes only the selected pairing', (await mock('state')).pairings[0].id, first);
    await page.getByRole('switch', { name: 'AI access', exact: true }).click();
    await wait((s) => !s.enabled);
    check('Off clears live AI activity', (await mock('state')).activity.length, 0);
    // All fixed themes, plus narrow layout; screenshots stay in the caller's throwaway folder.
    if (process.env.NOTAYA_AI_SHOTS) fs.mkdirSync(process.env.NOTAYA_AI_SHOTS, { recursive: true });
    for (const theme of ['sunrise', 'sunrise-dark', 'black', 'graphite', 'white', 'paper']) {
      await page.goto(`${origin}/dev-mock/ai-harness.html?theme=${theme}`);
      await page.getByRole('heading', { name: 'AI access', exact: true }).waitFor();
      await page.getByRole('switch', { name: 'AI access', exact: true }).waitFor();
      await page.getByRole('heading', { name: 'AI access', exact: true }).scrollIntoViewIfNeeded();
      check(`${theme}: AI panel renders`, await page.getByRole('switch', { name: 'AI access', exact: true }).isVisible(), true);
      if (process.env.NOTAYA_AI_SHOTS) await page.screenshot({ path: path.join(process.env.NOTAYA_AI_SHOTS, `ai-${theme}.png`) });
    }
    await page.setViewportSize({ width: 390, height: 844 });
    check('narrow layout has no horizontal overflow', await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true);
    check('no browser runtime errors', errors, []);
    console.log(`ALL OK (${checks} checks; mock UI only)`);
  } finally { await browser.close(); }
})().catch((e) => { console.error(e); process.exitCode = 1; });
