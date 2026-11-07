# Notaya usage ledger: how it works

**What this answers:** how much Claude and Codex work each Notaya session and task used, where it went (which thread, which model, which effort), and how fast the usage limits are being spent, so the coordinator can plan the pace of the next work and Ahmed and Mohamed can see what each milestone cost.

**Why it exists:** Ahmed's rule of 2026-11-07 20:12:10 UTC ("the usage ledger is also critical because we need to analyze how we are even using the data, and tokens going back and forth from where, so we can analyze our work to predict our pacing and understand it"), recorded as [A37](../plan/approval-and-amendments.md). It is mandatory for every session on Ahmed's side of the project (see the "Usage ledger" lines in [`CLAUDE.md`](../../CLAUDE.md)).

**Its own design.** This ledger belongs to Notaya only (Ahmed, 2026-11-07 20:19:06 UTC: "the ledger will be a ledger dedicated for Notaya, not our own artifact for the FFmpeg encoder. So make sure that Notaya's artifacts are their own artifacts"). The FFmpeg Encoder project's record 41 and its dashboard were read as a reference for what such a ledger can measure; nothing of theirs is copied, linked as a data source or shared. Notaya's figures are never mixed into FFmpeg's files, and FFmpeg's figures are never used as Notaya facts.

**Owner of the facts:** the files in this folder. A session summary or an Execution Summary may quote a headline figure from here, but the numbers live here.

## Files

| File | One row per | Filled from |
|---|---|---|
| [`usage-ledger.md`](usage-ledger.md) | dated block per session or day, newest on top | a plain-language reading of the three CSV files, the totals, what they mean for pacing, and what was not measured |
| [`claude-sessions.csv`](claude-sessions.csv) | Claude session (a thread, the coordinator, a Remote Control session), or one task inside a session when it was split | the session record (`get_session` usage block) or the transcript scanner below |
| [`codex-runs.csv`](codex-runs.csv) | Codex run (`codex exec` or one Codex app task) | Codex's own token report and its quota reading before and after |
| [`quota-readings.csv`](quota-readings.csv) | reading of a usage limit at a known time | Claude's `/usage` screen or claude.ai usage page, Codex `/status`, the session record's rate-limit block, the credit balance |

A dashboard page (an Artifact) is built from the CSV files with `python3 -I tools/usage-ledger/build_dashboard.py OUT.html` and published to the same link each time; the page never owns a fact. CSV is used so the files open in Excel, Notion or any chart tool, and so a later dashboard can read them directly. Every row says where its numbers came from (`source`) and what they do not show (`notes`).

## What is measured, and the units

**Claude, tokens per session (measured).** Four raw counts as the Anthropic API reports them: fresh input, cache write, cache read, output. Each Claude reply is counted once (a streamed reply can be written to a transcript several times; the scanner keeps it once by its message id).

**Claude, weighted tokens (derived).** One comparison number per row: `input × 1 + cache write × 1.25 + cache read × 0.1 + output × 5`. The weights follow the relative list prices of the API (a cache read costs a tenth of fresh input, output five times fresh input, a 5-minute cache write 1.25 times). They make a session that re-reads a large cached context comparable with one that writes a lot. They are a yardstick, not a bill: 1-hour cache writes really cost more than 1.25, and the subscription limits are not published per token.

**Claude, reported cost in US dollars (measured by the platform).** The session record of a cloud thread carries a `cost_usd` figure at API list prices. It is the closest measure of what a session draws from a dollar credit such as the cloud credit Ahmed mentioned on 2026-11-07. Whether a particular session is billed to that credit, to the plan's limits, or both, is **not** shown by the record and is not assumed.

**Codex (measured when a receipt exists).** Model, reasoning effort, input, cached input and output tokens from Codex's own report, and the 5-hour and weekly quota percentages read before and after the run.

**Quota readings (measured, at a time).** A percentage or status of a limit, its window (5-hour, weekly, credit) and its reset time. Readings are account-wide: other work on the same account in the same window moves them too, so a change between two readings is an upper bound for the work between them, not an exact cost.

**Not measured, and never guessed:** a session whose record or transcript was not reachable is written with empty number columns and the reason in `notes`. A model or effort that was not recorded stays "not recorded". Wall-clock time between a thread's start and its last update is not active working time.

## Columns

`claude-sessions.csv`: `date_utc, partner, session_id, thread_or_task, role, where, model_configured, model_last_served, effort, start_utc, last_update_utc, input, cache_write, cache_read, output, weighted, cost_usd_reported, snapshot_utc, source, notes`

- `partner`: whose side the session worked for, with the GitHub login: `Ahmed (AE0101)` or `Mohamed (AbyssWatcher2)` ([A38](../plan/approval-and-amendments.md), 2026-11-07).
- `role`: coordinator (the mother thread), thread, reader, Remote Control session, reviewer.
- `where`: the machine: cloud, or the PC's name (Ahmed's is `Gaming-PC`).
- `model_configured` and `model_last_served` are both kept because they can differ (a model switch or a fallback). The session record is the evidence; a selection label alone is not proof of what served the work.
- `snapshot_utc`: when the numbers were read. A running session's numbers grow after the snapshot; a later row with a later snapshot replaces nothing, it adds a reading.

`codex-runs.csv`: `date_utc, partner, machine, run_label, task, ordered_by, model, reasoning, start_utc, end_utc, input, cached_input, output, quota_5h_before, quota_5h_after, quota_week_before, quota_week_after, result, source, notes`

`quota-readings.csv`: `time_utc, tool, account, limit, reading, resets_utc, read_by, source, notes`

## How to refresh it (at every session stop)

1. **Cloud threads and the coordinator:** read each session's record (in a Claude project thread: the `get_session` tool with the session id) and copy the `usage` block (input, cache write, cache read, output, cost) with the snapshot time into `claude-sessions.csv`.
2. **Sessions on a PC** (Remote Control or a local Claude Code): run the scanner on that session's transcript, on the PC, and copy its CSV line:

   ```powershell
   python -I tools\usage-ledger\scan_claude_transcript.py "$env:USERPROFILE\.claude\projects\<folder>\<session>.jsonl" --csv
   ```

   Add `--since` and `--until` (UTC, ISO 8601) to split one long session into tasks, for example at each commit. The scanner only reads the file and prints token counts, model names and the first and last reply time; it never prints message text.
3. **Codex:** after each run, write one row with the token counts Codex reports and the quota percentages from Codex `/status` read just before and just after.
4. **Quota:** add a reading whenever someone looks at the limits (Claude `/usage` or the claude.ai usage page; Codex `/status`), and at least once per session stop. Say who read it.
5. **Summarize:** add a dated block at the top of `usage-ledger.md` with the totals, what moved, what it means for the pace, and what was not measured. Quote the headline line in the session summary (`docs/sessions.md`, Models and Provenance) and in the Execution Summary of a milestone.

## Pacing limits

**Not set for Notaya yet.** Ahmed has said the coordinator watches the hourly usage, keeps the work going and chooses models per task, and that Claude threads come first while the 2026-11-07 cloud credit lasts. He has not set a percentage at which work slows or stops. The FFmpeg Encoder project used 80% of a 5-hour window and 90% of a week; that is FFmpeg's decision and is not adopted here without Ahmed's word.

## Limits of this ledger

- Usage limits are shared by everything on the same account, including other projects; the ledger cannot separate Notaya's share inside one reading.
- Mohamed's sessions run on his own account. His usage is not visible from Ahmed's side and is not recorded here unless his Claude adds it.
- Session records of Remote Control sessions on a PC are not readable from the cloud; those rows stay empty until the scanner is run on the PC.
- Figures before 2026-11-07 were never recorded and are not reconstructed.
