// Claude runs this before any suites. Never attaches to a personal browser/app.
const assert = require('node:assert/strict');
const { chromium } = require(process.env.NOTAYA_PLAYWRIGHT_MODULE || 'playwright-core');
const [origin, mode] = process.argv.slice(2);
assert.equal(new URL(origin).hostname, '127.0.0.1');
assert.ok(['mock', 'views'].includes(mode));
const fixtures = ['/dev-mock/equation-fixture.ts', '/dev-mock/core.ts', '/src/lib/flush.ts', '/src/store/app.ts', '/src/lib/theme.ts', '/src/lib/navHistory.ts'];
(async () => {
  const browser = await chromium.launch({ executablePath: process.env.NOTAYA_CHROMIUM_PATH, headless: true });
  try {
    const context = await browser.newContext();
    await context.route('**/*', route => new URL(route.request().url()).origin === origin ? route.continue() : route.abort());
    const page = await context.newPage();
    let failures = [];
    page.on('response', response => { if (response.status() >= 400 && new URL(response.url()).pathname !== '/favicon.ico') failures.push(`${response.status()} ${response.url()}`); });
    page.on('pageerror', error => failures.push(error.message));
    const url = mode === 'mock' ? `${origin}/?db=1` : `${origin}/dev-mock/database-views-harness.html?view=board`;
    // Two consecutive clean loads after optimisation, bounded to four attempts.
    let clean = 0;
    for (let attempt = 1; attempt <= 4 && clean < 2; attempt++) {
      failures = [];
      try {
        await page.goto(url, { waitUntil: 'networkidle' });
        if (mode === 'mock') {
          await page.getByRole('button', { name: 'New notebook', exact: true }).waitFor();
          await page.evaluate(async modules => { for (const module of modules) await import(module); }, fixtures);
        } else await page.locator('#view-host > *').first().waitFor();
        await page.evaluate(() => document.fonts.ready);
        await page.waitForLoadState('networkidle');
        // Cover delayed optimiser reloads as well as in-flight fetches.
        const stamp = await page.evaluate(() => performance.timeOrigin);
        await page.waitForTimeout(1500);
        assert.equal(await page.evaluate(() => performance.timeOrigin), stamp, 'navigation during settling');
        assert.deepEqual(failures, [], 'warm-up resource/page errors');
        clean++;
        console.log(`CHECKPOINT: ${mode} warm load ${clean}/2 done`);
      } catch (error) {
        clean = 0;
        console.log(`Warm-up attempt ${attempt} interrupted: ${error.message}`);
      }
    }
    assert.equal(clean, 2, `${mode} did not settle; no dependent suites may run`);
    await context.close();
  } finally { await browser.close(); }
})().catch(error => { console.error(error.stack); process.exitCode = 1; });
