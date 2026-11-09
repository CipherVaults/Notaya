// Refresh the derived 250-row sheet; no browser/app launched here.
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const { root, planChecks } = require('./catalog.cjs');
const destination = path.join(root, 'docs/plan/2026-11-09-whole-app-test-results.md');
const args = process.argv.slice(2);
const init = args.includes('--init');
const files = args.filter(a => a !== '--init');
const rows = planChecks();
const states = new Map(rows.map(row => [row.id, { status: 'not run', observations: [], defects: [] }]));
let sheetSource = 'No execution JSON imported. All 250 checks are not run.';
let imported = [];
if (!init && fs.existsSync(destination)) {
  const prior = fs.readFileSync(destination, 'utf8').match(/<!-- WHOLE_APP_STATE\n([\s\S]*?)\nWHOLE_APP_STATE -->/);
  if (!prior) throw Error('Existing sheet has no machine state; refusing to overwrite manual work');
  const saved = JSON.parse(prior[1]);
  for (const [id, value] of Object.entries(saved.states)) { if (states.has(id)) states.set(id, value); }
  imported = saved.imported;
}
if (init && fs.existsSync(destination)) throw Error('--init never overwrites an existing result sheet');
for (const file of files) {
  const absolute = path.resolve(file);
  const bytes = fs.readFileSync(absolute);
  const hash = crypto.createHash('sha256').update(bytes).digest('hex');
  if (imported.some(i => i.sha256 === hash)) continue;
  const report = JSON.parse(bytes.toString('utf8').replace(/^\uFEFF/, ''));
  if (report.schemaVersion !== 1 || !Array.isArray(report.checks) || !report.runId) throw Error(`Invalid report ${file}`);
  const seen = new Set();
  for (const observation of report.checks) {
    if (!states.has(observation.id) || seen.has(observation.id)) throw Error(`Unknown/duplicate check ${observation.id}`);
    seen.add(observation.id);
    if (!['passed', 'failed', 'not run'].includes(observation.status)) throw Error(`Invalid status ${observation.id}`);
    const expectedKind = rows.find(r => r.id === observation.id).kind;
    if (expectedKind !== report.group || observation.kind !== expectedKind) throw Error(`Wrong executor KIND for ${observation.id}`);
    const state = states.get(observation.id);
    const previous = state.observations.at(-1);
    const record = { ...observation, json: absolute, runId: report.runId, utc: report.finishedUtc || report.startedUtc,
      source: report.source, sourceAfter: report.sourceAfter, sourceChangedDuringRun: report.sourceChangedDuringRun,
      binary: report.binary || null, cleanup: report.cleanup, executor: report.executor, home: report.home };
    state.observations.push(record);
    const identity = r => JSON.stringify([r.source?.head, r.source?.mergeHead, r.source?.dirtyAndRunnerHashes, r.binary?.sha256]);
    if (previous && state.status === 'passed' && identity(previous) !== identity(record)) state.status = 'not run';
    // A passing supporting suite is useful evidence, but does not complete a feature row.
    const cleanupFailed = Object.values(report.cleanup || {}).some(v => v === false);
    const complete = observation.coverage === 'complete' && !report.sourceChangedDuringRun && !report.fatalError && !cleanupFailed;
    if (observation.status === 'failed') {
      state.status = 'failed';
      state.defects.push(observation.defectGroup || `HARNESS-${report.group}`);
    } else if (observation.status === 'passed' && complete && state.status !== 'failed') state.status = 'passed';
    // Failure remains visible across reruns until a reviewed, full-coverage retest explicitly
    // resolves the earlier defect. This importer never silently erases failed evidence.
  }
  imported.push({ path: absolute, sha256: hash, group: report.group, runId: report.runId, utc: report.finishedUtc || report.startedUtc });
}
if (imported.length) sheetSource = `${imported.length} immutable JSON observation(s) imported. See provenance and machine state below.`;
const safe = s => String(s ?? '').replaceAll('|', '&#124;').replace(/\r?\n/g, ' ');
const tally = kind => ['passed', 'failed', 'not run'].map(status => rows.filter(r => r.kind === kind && states.get(r.id).status === status).length);
let text = `# Whole-app test results - lane W, phase 2\n\n` +
  `Produced scaffold for Ahmed Emara (AE0101), Gaming-PC; Claude executes and owns commits/records. Baseline: W HEAD 04fcb96 with staged integration of main 5077183. Runtime evidence applies only to the source and binary identities inside each imported report. No owner acceptance is implied.\n\n` +
  `${sheetSource}\n\n` +
  `| KIND | Passed | Failed | Not run | Total |\n|---|---:|---:|---:|---:|\n`;
for (const kind of ['A', 'B', 'C']) { const counts = tally(kind); text += `| ${kind} | ${counts.join(' | ')} | ${counts.reduce((a, b) => a + b)} |\n`; }
text += `\nEach numbered row appears once. Supporting suites have independent passed/failed/not run results in JSON; their evidence is correlated here without converting unit/mock/partial coverage into a full feature pass. Failed supporting assertions flag an investigation (HARNESS-A/B), not a confirmed application defect. Manual C checks remain not run until an actual person supplies a KIND C report.\n\n` +
  `## Refresh from JSON\n\nFrom the repo root, after Claude runs a slice:\n\n\x60\x60\x60powershell\nnode tools/whole-app/merge-results.cjs 'D:\\Notaya-lanes\\out\\whole-app\\group-a-results.json'\nnode tools/whole-app/merge-results.cjs 'D:\\Notaya-lanes\\out\\whole-app\\group-b-results.json'\n\x60\x60\x60\n\n` +
  `You may pass multiple report paths, including archived run files. The importer validates IDs/KIND/status, deduplicates JSON by SHA-256 and retains every prior observation. It preserves evidence in the embedded state and never erases a failure with an incomplete rerun. A full passing retest after a failure needs coordinator review/resolution in a later slice. Do not edit generated rows by hand. \x60--init\x60 creates an all-not-run sheet once and refuses an existing file. Performance is a separate measurement report, not a feature pass; inspect perf-results.json directly.\n\n` +
  `Reports contain schemaVersion, group, runId, source HEAD/merge/dirty hashes, executor, UTC, home, suites (script, exitCode, timeout, duration, last30Lines, log), checks (id, kind, status, coverage, evidence, defectGroup, reason), and cleanup. Real-exe reports add SHA-256 and caller-supplied build commit; a claimed commit is not independently proven by a binary hash. Complete means all plan steps and variants, including required reopen/readback. A source change or cleanup failure prevents promotion. Runners preserve unique run directories before updating the latest fixed JSON file.\n\n` +
  `## Checks\n\n| ID | KIND | Feature | Status | Evidence pointer / remaining reason | Defect group |\n|---|---|---|---|---|---|\n`;
for (const row of rows) {
  const s = states.get(row.id), latest = s.observations.at(-1);
  const evidence = latest ? `${latest.json} :: checks[${row.id}] (${latest.coverage || 'unspecified'}); ${latest.reason || ''}` : 'No execution evidence';
  text += `| ${row.id} | ${row.kind} | ${safe(row.feature)} | ${s.status} | ${safe(evidence)} | ${safe([...new Set(s.defects)].join(', ')) || '-'} |\n`;
}
text += `\n## Imported provenance\n\n`;
for (const entry of imported) text += `- ${safe(entry.path)}; SHA-256 ${entry.sha256}; run ${entry.runId}; KIND ${entry.group}; UTC ${entry.utc || 'not recorded'}.\n`;
if (!imported.length) text += 'None. Browser, real exe, cargo suites and performance have not run in the scaffolding session. Syntax parsing is separate evidence.\n';
text += `\n<!-- WHOLE_APP_STATE\n${JSON.stringify({ imported, states: Object.fromEntries(states) }, null, 2)}\nWHOLE_APP_STATE -->\n`;
fs.writeFileSync(destination, text);
console.log(`CHECKPOINT: result sheet refreshed (${rows.length} rows) done`);
