// Browser wiring only: actual ExportDialog + IPC wrapper + Sonner, synthetic mock replies.
// Caller owns the reviewed loopback mock server and browser tooling. No app, vault or export files.
// The normal entry is tools/checks/baseline-browser-check.cjs with this exact suite name.
// Optional direct args: loopback URL, reserved output folder, comma-separated case names.
// An explicit empty/unknown/repeated selection fails before acquiring a browser.
// No timing/performance claim: bounded waits below only wait for the requested UI outcome.
'use strict';
const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const http = require('node:http');
const { createChecks } = require('../tools/checks/fail-loud.cjs');

const checks = createChecks('export-warning-ui');
const NAMES = Object.freeze(['clean-success', 'cleanup-warning', 'primary-error-leftovers']);
const [baseArgument, _reservedOutputFolder, selectedArgument, ...extraArguments] = process.argv.slice(2);
const selectedRaw = selectedArgument === undefined ? process.env.NOTAYA_EXPORT_WARNING_CASES : selectedArgument;
const selected = checks.selection(selectedRaw === undefined ? [...NAMES] : selectedRaw === '' ? [] : selectedRaw.split(','), NAMES);
const EXPECTED_CHECKS = selected.length + 1; // one complete effect assertion per case, then an exact-count gate
const DESTINATION = 'X:\\notaya-synthetic-export\\notes.md';
const WARNING_A = 'Some unencrypted export files could not be safely removed from X:\\notaya-synthetic-export\\notes.assets\\g-old. They were kept; remove them yourself only after checking them.';
const WARNING_B = 'Some unencrypted export files could not be safely removed from X:\\notaya-synthetic-export\\notes.assets\\g-other. They were kept; remove them yourself only after checking them.';
const PRIMARY_ERROR = 'File error: INJECTED synthetic document replacement failure.';
const ERROR_WITH_LEFTOVERS = PRIMARY_ERROR + ' ' + WARNING_A;
const SUMMARY = Object.freeze({
  path: DESTINATION, pages: 1, images: 0, missingImages: 0, imagesLeftOut: 0,
  charsReplaced: 0, charsNoFont: 0, assetsFolder: null, files: 1, archive: false,
});
const EXPECTED_REQUESTS = Object.freeze([{
  cmd: 'export_notes',
  args: { request: { format: 'markdown', includeSubpages: true, output: 'single', scope: 'pages', ids: ['p1'] } },
}]);
const FIXTURES = Object.freeze({
  'clean-success': { summary: { ...SUMMARY, cleanupWarnings: [] }, error: null },
  'cleanup-warning': { summary: { ...SUMMARY, cleanupWarnings: [WARNING_A, WARNING_B] }, error: null },
  'primary-error-leftovers': { summary: null, error: ERROR_WITH_LEFTOVERS },
});
const canonical = (value) => Array.isArray(value) ? value.map(canonical)
  : value && typeof value === 'object' ? Object.fromEntries(Object.keys(value).sort().map((key) => [key, canonical(value[key])]))
    : value;
let counted = 0;
function equal(label, actual, expected) {
  counted++;
  const ok = checks.equal(label, canonical(actual), canonical(expected));
  if (!ok) console.log('EXPORT_WARNING_DETAIL ' + JSON.stringify({ label, actual, expected }));
}
function fail(label, detail) { counted++; checks.check(label, false, detail); }
const sha256 = (text) => crypto.createHash('sha256').update(text).digest('hex');
const message = (error) => error?.stack || String(error);

function ownedMockModule(sourceUrl, target) {
  const url = new URL(sourceUrl);
  assert.equal(url.protocol, 'http:', 'Mock module fetch uses local HTTP only');
  assert.equal(url.origin, target.origin, 'Mock module fetch stays on the verified caller-owned origin');
  assert.equal(url.pathname, '/dev-mock/core.ts', 'Only the literal mock Core module may be fetched');
  assert(!url.username && !url.password && !url.hash, 'No authentication or fragment in the mock module URL');
  const maximumBytes = 4 * 1024 * 1024;
  return new Promise((resolve, reject) => {
    let request;
    let response;
    let settled = false;
    const chunks = [];
    let bytes = 0;
    const fail = (error) => {
      if (settled) return;
      settled = true;
      clearTimeout(deadline);
      response?.destroy();
      request?.destroy();
      reject(error);
    };
    const deadline = setTimeout(() => fail(new Error('Owned mock module fetch exceeded 10000 ms')), 10000);
    try {
      // Built-in HTTP does not consult proxy environment variables. No persistent agent,
      // cookies, authentication or redirects; the reviewed caller already verifies server identity.
      request = http.get(url, { agent: false, maxHeaderSize: 16384, headers: { Accept: 'text/javascript' } }, (incoming) => {
        response = incoming;
        if (incoming.statusCode !== 200) return fail(new Error('Owned mock module fetch returned ' + incoming.statusCode));
        if (incoming.headers['content-encoding'] && incoming.headers['content-encoding'] !== 'identity') {
          return fail(new Error('Unexpected encoded mock module response'));
        }
        incoming.on('error', fail);
        incoming.on('aborted', () => fail(new Error('Owned mock module response aborted')));
        incoming.on('data', (chunk) => {
          bytes += chunk.length;
          if (!Number.isSafeInteger(bytes) || bytes > maximumBytes) return fail(new Error('Owned mock module exceeds 4 MiB'));
          chunks.push(chunk);
        });
        incoming.on('end', () => {
          if (settled) return;
          try {
            assert(bytes > 0, 'Owned mock module must not be empty');
            const source = new TextDecoder('utf-8', { fatal: true }).decode(Buffer.concat(chunks));
            settled = true;
            clearTimeout(deadline);
            resolve(source);
          } catch (error) { fail(error); }
        });
      });
      request.on('error', fail);
    } catch (error) { fail(error); }
  });
}

async function runCase(browser, target, name) {
  let context;
  let page;
  let actual;
  let scenarioFailure;
  const errors = [];
  const blocked = [];
  const moduleErrors = [];
  const modulePins = [];
  try {
    context = await browser.newContext({ viewport: { width: 1366, height: 900 }, serviceWorkers: 'block', acceptDownloads: false });
    await context.route('**/*', async (route) => {
      const url = new URL(route.request().url());
      if (url.origin === target.origin) return route.continue();
      blocked.push('non-mock request');
      return route.abort();
    });
    // Retain the reviewed caller wrapper's owned-origin HMR / other-origin refusal policy.
    page = await context.newPage();
    page.setDefaultTimeout(10000);
    page.on('pageerror', (error) => errors.push(error.message));
    page.on('download', () => errors.push('Unexpected download'));
    page.on('dialog', async (dialog) => { errors.push('Unexpected browser dialog'); await dialog.dismiss(); });
    context.on('page', (opened) => { if (opened !== page) errors.push('Unexpected additional page'); });
    await page.addInitScript((fixture) => {
      window.__exportWarningFixture = fixture;
      window.__exportWarningCalls = [];
    }, FIXTURES[name]);

    // Change only the test browser's response. Product/mock files remain unchanged.
    // This seam is the existing export_notes branch, never the whole invoke function.
    await page.route('**/dev-mock/core.ts*', async (route) => {
      try {
        const url = new URL(route.request().url());
        assert.equal(url.origin, target.origin, 'Only the caller-owned mock origin');
        assert.equal(url.pathname, '/dev-mock/core.ts', 'Only the actual mock Core module');
        const original = await ownedMockModule(url.href, target);
        const marker = 'case "export_notes": {';
        assert.equal(original.split(marker).length - 1, 1, 'The export-only injection marker must occur exactly once');
        const injection = marker + '\n' +
          'window.__exportWarningCalls.push(structuredClone({cmd,args}));\n' +
          'const exportWarningFixture = window.__exportWarningFixture;\n' +
          'if (!exportWarningFixture) throw "Missing export warning fixture";\n' +
          'if (exportWarningFixture.error !== null) throw exportWarningFixture.error;\n' +
          'return structuredClone(exportWarningFixture.summary);\n';
        const executed = original.replace(marker, injection);
        modulePins.push({ originalSHA256: sha256(original), executedSHA256: sha256(executed) });
        await route.fulfill({ status: 200, contentType: 'text/javascript; charset=utf-8', body: executed });
      } catch (error) {
        moduleErrors.push(message(error));
        await route.abort();
      }
    });

    const url = new URL('/', target);
    url.search = '?s=app&cover=none';
    await page.goto(url.href);
    const row = page.locator('[role="treeitem"][data-rowkey="p:p1"]');
    await row.waitFor({ state: 'visible' });
    assert.equal(await row.count(), 1, 'The fixed p1 sidebar fixture must be unique');
    await row.click();
    await page.waitForFunction(() => document.querySelector('main input[placeholder="Untitled"]')?.value === 'Morning pages');
    await page.getByRole('button', { name: 'Export', exact: true }).click();
    const dialog = page.getByRole('dialog', { name: 'Export', exact: true });
    await dialog.waitFor({ state: 'visible' });
    await dialog.getByRole('button', { name: 'Markdown', exact: true }).click();
    const before = await page.evaluate(() => ({
      calls: window.__exportWarningCalls,
      toastCount: document.querySelectorAll('[data-sonner-toast]').length,
    }));
    await dialog.getByRole('button', { name: 'Export…', exact: true }).click();

    const expectedType = name === 'primary-error-leftovers' ? 'error' : 'success';
    await page.locator('[data-sonner-toast][data-type="' + expectedType + '"]').waitFor({ state: 'visible' });
    if (name === 'cleanup-warning') {
      await page.locator('[data-sonner-toast][data-type="warning"]').waitFor({ state: 'visible' });
    }
    if (name === 'primary-error-leftovers') {
      await page.waitForFunction(() => {
        const button = [...document.querySelectorAll('[role="dialog"] button')].find((element) => element.textContent === 'Export…');
        return button && !button.disabled;
      });
    } else {
      await dialog.waitFor({ state: 'hidden' });
    }
    const effect = await page.evaluate(() => {
      const visible = (element) => {
        const box = element.getBoundingClientRect(), style = getComputedStyle(element);
        return box.width > 0 && box.height > 0 && style.display !== 'none' && style.visibility !== 'hidden';
      };
      return {
        requests: window.__exportWarningCalls,
        toasts: [...document.querySelectorAll('[data-sonner-toast]')].map((element) => ({
          type: element.getAttribute('data-type'),
          title: element.querySelector('[data-title]')?.textContent ?? null,
          description: element.querySelector('[data-description]')?.textContent ?? null,
          visible: visible(element),
        })).sort((a, b) => a.type.localeCompare(b.type)),
      };
    });
    const dialogVisible = await dialog.isVisible();
    actual = {
      before, ...effect, dialogVisible,
      exportButtonEnabled: dialogVisible ? await dialog.getByRole('button', { name: 'Export…', exact: true }).isEnabled() : null,
    };
  } catch (error) {
    scenarioFailure = message(error);
    if (page && !page.isClosed()) {
      try {
        actual = await page.evaluate(() => ({
          requests: window.__exportWarningCalls,
          toasts: [...document.querySelectorAll('[data-sonner-toast]')].map((element) => ({
            type: element.getAttribute('data-type'), text: element.textContent,
          })),
        }));
      } catch (readError) { errors.push('Failure readback: ' + message(readError)); }
    }
  } finally {
    try { await page?.close(); } catch (error) { errors.push('Page cleanup: ' + message(error)); }
    try { await context?.close(); } catch (error) { errors.push('Context cleanup: ' + message(error)); }
  }
  console.log('EXPORT_WARNING_MOCK_SOURCE ' + JSON.stringify({ case: name, modules: modulePins }));
  if (scenarioFailure) {
    fail(name, JSON.stringify({ scenarioFailure, actual, errors, blocked, moduleErrors }));
    return;
  }
  const expectedToasts = name === 'primary-error-leftovers'
    ? [{ type: 'error', title: ERROR_WITH_LEFTOVERS, description: null, visible: true }]
    : [{ type: 'success', title: 'Exported 1 page.', description: DESTINATION, visible: true }];
  if (name === 'cleanup-warning') expectedToasts.push({ type: 'warning', title: WARNING_A + ' ' + WARNING_B, description: null, visible: true });
  equal(name, { ...actual, moduleResponses: modulePins.length, moduleErrors, errors, blocked }, {
    before: { calls: [], toastCount: 0 }, requests: EXPECTED_REQUESTS, toasts: expectedToasts,
    dialogVisible: name === 'primary-error-leftovers',
    exportButtonEnabled: name === 'primary-error-leftovers' ? true : null,
    moduleResponses: 1, moduleErrors: [], errors: [], blocked: [],
  });
}

let browser;
(async () => {
  try {
    if (!selected.length) return;
    assert.equal(extraArguments.length, 0, 'Unexpected export-warning arguments');
    const target = new URL(baseArgument || process.env.MOCK_URL || 'http://127.0.0.1:1421/');
    assert(target.protocol === 'http:' && ['127.0.0.1', 'localhost', '[::1]'].includes(target.hostname)
      && !target.username && !target.password, 'Use only the caller-owned loopback HTTP mock');
    console.log('EXPORT_WARNING_SELECTION ' + JSON.stringify({ selected, expectedChecks: EXPECTED_CHECKS }));
    const { chromium } = require('playwright');
    browser = await chromium.launch({ headless: true });
    for (const name of selected) await runCase(browser, target, name);
  } catch (error) {
    fail('export-warning runner', message(error));
  } finally {
    try { await browser?.close(); } catch (error) { fail('browser cleanup', message(error)); }
    if (selected.length) equal('exact selected-case check count', counted + 1, EXPECTED_CHECKS);
    checks.finish();
  }
})();
