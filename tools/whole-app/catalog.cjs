const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '../..');
const plan = path.join(root, 'docs/plan/2026-11-09-whole-app-test-plan.md');
function planChecks() {
  const rows = fs.readFileSync(plan, 'utf8').split(/\r?\n/)
    .filter(line => /^\| W\d{3} \| [ABC] \|/.test(line))
    .map(line => { const [, id, kind, feature] = line.split('|').map(s => s.trim()); return { id, kind, feature }; });
  if (rows.length !== 250 || rows.some((r, i) => r.id !== `W${String(i + 1).padStart(3, '0')}`)) throw Error('Plan must have unique contiguous W001-W250');
  return rows;
}
const ids = (...numbers) => numbers.map(n => `W${String(n).padStart(3, '0')}`);
const range = (a, b) => Array.from({ length: b - a + 1 }, (_, i) => a + i);
// Correlation is supporting evidence, never an assertion that a legacy suite covers every
// variant of a plan row. Full-coverage row observations must be explicitly declared separately.
const mocks = [
  ['automations-ui-check.cjs', 'mock', ids(85, 102, 103, 104, 105)],
  ['block-ids-check.cjs', 'mock', ids(34, 35, 231), 'url'],
  ['block-ids-waits-check.cjs', 'node', ids(231)],
  ['c2-check.cjs', 'node', ids(226, 228, 229)],
  ['c2-ui-check.cjs', 'mock', ids(226, 229), 'url'],
  ['c3-check.cjs', 'node', ids(233, 234, 236, 237, 238, 239)],
  ['c3-ui-check.cjs', 'mock', ids(233, 234, 236), 'url'],
  ['calc-check.cjs', 'node', ids(124, 125, 126)],
  ['calc-ui-check.cjs', 'mock', ids(124, 125, 126)],
  ['database-fixes-ui-check.cjs', 'mock', ids(53, 59, 60, 61, 62, 100, 101, 102, 103)],
  ['database-ui-check.cjs', 'mock', ids(51, 52, 53, ...range(63, 92), 100, 40, 41)],
  ['database-views-check.cjs', 'views', ids(...range(54, 62))],
  ['database-mock-check.mts', 'node', ids(...range(52, 62), 100)],
  ['equation-identity-check.cjs', 'node', ids(34, 35, 231)],
  ['icons-covers-ui-check.cjs', 'mock', ids(2, 9, 10, 37, 48, 49)],
  ['lane-a-check.cjs', 'node', ids(13, 14, 15, 16, 213)],
  ['lane-a-contrast-check.cjs', 'node', ids(...range(199, 204))],
  ['lane-a-shots.cjs', 'mock', ids(9, 14, 16, 49, 148, ...range(199, 204)), 'url'],
  ['mermaid-ui-check.cjs', 'mock', ids(32, 33), 'url-shots'],
  ['page-look-layers-check.cjs', 'mock', ids(49)],
  ['ribbon-ui-check.cjs', 'mock', ids(...range(22, 31), 35, 42, 43, 44)],
  ['rtl-ui-check.cjs', 'mock', ids(46, 130)],
  ['template-keys-check.cjs', 'node', ids(94)],
  ['ai-mock-check.mjs', 'node', ids(...range(241, 249))],
  ['ai-ui-check.cjs', 'mock', ids(242, 243, 244, 245, 247, 250)],
].map(([file, mode, checkIds, args = 'shots']) => ({ file, mode, checkIds, args }));
const helpers = ['block-ids-waits.cjs', 'lane-a-wait.cjs'];
const completeA = { W126: ['calc-ui-check.cjs'] }; // All W126 unit-toggle/display variants are asserted here.
const rustAreas = {
  'crypto-model': ['crypto::', 'model::', 'generator::', 'filenames::'],
  vault: ['vault::', 'openvault::'], recovery: ['backup::', 'restore::', 'import::'],
  'windows-state': ['portable::', 'legacy_guard::', 'shell::', 'state::'],
  foundation: ['actions::', 'changes::', 'blocks::'],
  'notes-commands': ['commands::notes::', 'commands::stickies::', 'commands::tools::', 'commands::vault::', 'commands::search_history::', 'commands::calculator::', 'commands::backup::'],
  'database-commands': ['commands::database::', 'commands::database_io::', 'commands::automation::', 'commands::notion_db::'],
  'database-engine': ['database::'],
  import: ['commands::import_notes::', 'importers::'], export: ['commands::export::', 'export::'],
  roundtrips: ['roundtrip_tests::'], logging: ['logging::'],
  // commands::ai contains window adapters, no #[test] functions. Their core tests
  // live under ai::; do not invent a zero-match required test suite.
  'integrated-ai': ['ai::'],
};
// Rows requiring a person, actual native pickers/OS actions, or unavailable reviewed fixture routes.
const human = new Set(ids(36, 39, 48, 71, ...range(107, 112), ...range(138, 147), ...range(149, 167),
  178, 179, 180, 184, 185, 188, 190, 193, 195,
  197, 198, 211, 218, 219, 221, 225, 230, 232, 235, 237, 238, 239, 240, ...range(241, 249)));
const legacyB = [
  { name: 'optional-security', file: 'optionalsecurity.mjs', checkIds: ids(168, 169, 170, 171, 181, 187, 194), phases: ['p1', 'p2', 'p3', 'p4', 'p5'] },
  { name: 'restore-flow', file: 'restoreflow.mjs', checkIds: ids(187, 191, 192, 194), phases: ['all'] },
  { name: 'lock-page', file: 'lockflow.mjs', scenario: 'page', checkIds: ids(21, 172), phases: ['setup', 'type', 'check', 'reopen'] },
  { name: 'lock-sticky', file: 'lockflow.mjs', scenario: 'sticky', checkIds: ids(122, 172), phases: ['setup', 'type', 'check', 'reopen'] },
  { name: 'close-page', file: 'lockflow.mjs', scenario: 'close', checkIds: ids(21), phases: ['setup', 'type', 'check', 'reopen'] },
];
function bRoute(row) {
  if (row.id === 'W217') return { route: 'partial unattended', suites: ['installed-paths'], reason: 'Installed/portable first-start/reopen assertions run with isolated LocalAppData; native import picker is not automated.' };
  const suites = legacyB.filter(s => s.checkIds.includes(row.id)).map(s => s.name);
  if (suites.length) return { route: 'partial unattended', suites, reason: 'Legacy assertions run unattended; remaining plan variants need a later slice/person/fixture.' };
  if (human.has(row.id)) return { route: 'not unattended', suites: [], reason: 'Actual native dialog, OS/person action, AI window consent, or missing reviewed fault/time/legacy fixture route; see exact plan steps.' };
  return { route: 'later unattended harness', suites: [], reason: 'Can be scripted on disposable data; no complete assertion driver for this row in slice 1.' };
}
module.exports = { root, plan, planChecks, mocks, helpers, completeA, rustAreas, legacyB, bRoute };
if (require.main === module) {
  const [command, file] = process.argv.slice(2);
  if (command === 'update-plan-routing') {
    const start = '<!-- WHOLE_APP_ROUTING_START -->', end = '<!-- WHOLE_APP_ROUTING_END -->';
    let body = fs.readFileSync(plan, 'utf8');
    const b = planChecks().filter(r => r.kind === 'B');
    let appendix = `${start}\n## Phase-2 slice 1: execution routing and scaffolding\n\n` +
      `Dated ${new Date().toISOString()} (system clock), Ahmed (AE0101), Gaming-PC. The current phase-2 instruction authorizes only tools/whole-app/* and this plan/results sheet, no app edits or execution here. Inspected W HEAD 04fcb96; Claude's integration of main 5077183 is staged, with no unresolved conflicts at orientation. The phase-1 facts above remain historical. No fetch, network, installs, Git writes, browser or real exe ran in this scaffolding session. Claude owns commits, authority/session/usage records and runtime execution. User-message time and quota receipts were not supplied.\n\n` +
      `The PC runners use fresh GUID homes in D:\\Notaya Tests\\whole-app and unique archives in D:\\Notaya-lanes\\out\\whole-app. They write fixed latest JSON files there. Both home variables and child TEMP/TMP/profile are isolated. Installed/portable path probes deliberately clear both home overrides for the app child and use only their disposable LocalAppData; no real profile or installation is used. All processes are owned by private kill-on-close Windows jobs. Port availability, teardown and unset WebView arguments are recorded (FI-01). Existing Notaya/Cipherleaf makes B/performance refuse launch. Dependencies/toolchain must already exist; Cargo is offline/locked. No downloads are attempted.\n\n` +
      `Group A inventories all executable dev-mock CJS/MJS/MTS suites, including integrated AI scripts, and identifies the two consumer-only helpers. Its adapter records original hashes, infrastructure substitutions and executed CJS sources; selectors/assertions stay unchanged. It supplies the installed Playwright/Chrome, relocates output/temp and loopback ports, and transpiles the calculator into external temp. The views harness uses its own free port. Rust discovery precedes every listed module-prefix area; zero discovery/passing tests is not run. Bridge normal/native ignored tests and integrated AI modules are separate supporting suites. Legacy suite-to-ID mapping is supporting evidence; only W126 currently has an explicit complete A mapping. Most existing suites do not cover every variant of their plan row. Script success cannot silently pass the other A rows.\n\n` +
      `Group B slice 1 runs optional-security p1-p5, restoreflow, independent page/sticky/close lockflow fixtures, and isolated installed/portable first/reopen probes. It adds exit/assertion guards to copies of legacy JS, bounds every phase, preserves fixtures and logs, and performs actual quit/reopen/readback. It does not run the historical password-manager tests (sources absent, feature removed). Existing tray script mixes tray checks with actual startup-registry changes and is deferred to a clean-user/session slice. No AI consent is answered through injected IPC.\n\n` +
      `Performance defaults to blank startup plus 1,000 fake notes, a 1,000-block mixed text/list/code/RTL page and a 1,000-row database through the real app commands; five warm reopen/probe samples, median/range, first-window time, seven backend view-query times and sampled process-tree memory. Increase sizes explicitly after inspecting smaller fixtures. No performance threshold or zeroisation claim; protected unlock, richer long-page blocks, full view rendering, backup/restore and ten multiwindow cycles remain later measurements.\n\n` +
      `Routing below covers all ${b.length} B checks. **partial unattended** means the named suite runs but does not complete every plan variant. **later unattended harness** means scriptable work for a later slice, currently not run. **not unattended** means this runner cannot complete it without actual native-dialog/person/OS/AI-consent work or a reviewed fixture/fault/transport harness that it does not supply; this is a current execution limitation, not a claim automation can never exist. All unimplemented checks stay visible as not run in JSON/results. C's 27 checks still need a person.\n\n` +
      `| ID | Feature | Slice-1 route | Suite or remaining work |\n|---|---|---|---|\n`;
    for (const row of b) { const route = bRoute(row); appendix += `| ${row.id} | ${row.feature.replaceAll('|', '&#124;')} | ${route.route} | ${route.suites.join(', ') || route.reason} |\n`; }
    appendix += `${end}\n`;
    if (body.includes(start)) { const before = body.indexOf(start), after = body.indexOf(end, before); if (after < 0) throw Error('Missing routing end marker'); body = body.slice(0, before) + appendix + body.slice(after + end.length).replace(/^\r?\n/, ''); }
    else body += `\n${appendix}`;
    fs.writeFileSync(plan, body);
    console.log(`CHECKPOINT: all ${b.length} B routes recorded in plan done`);
    return;
  }
  if (command !== 'write-manifest' || !file) throw Error('Usage: node catalog.cjs write-manifest <json>');
  fs.writeFileSync(file, JSON.stringify({ mocks, helpers, completeA, rustAreas, legacyB, checks: planChecks().map(r => ({ ...r, ...(r.kind === 'B' ? bRoute(r) : {}) })) }, null, 2));
}
