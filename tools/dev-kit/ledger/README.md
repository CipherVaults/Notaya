# Local usage ledger

This is a sanitized copy of the existing CSV dashboard builder and forecast model,
with its matching HTML template. Python 3 builds the standalone HTML using only
its standard library. Node is needed only for `--forecast-csv`; the interactive
forecast model is also inlined into the HTML. No dependency install or network
request is required. The bundled `example/` is entirely invented, dated in 2000,
and carries explicit EXAMPLE identifiers. It is not a reading of any real account.

From this directory:

```text
python build_dashboard.py out.html
python build_dashboard.py --input-dir example out.html
python build_dashboard.py --forecast-csv --input-dir example
```

The first two commands read only their input CSVs and write the requested output.
The third writes `pace-forecast.csv` in the selected input directory; it is optional.
An optional `--input-dir` selects a separately maintained ledger. Never point this
at private source records while preparing a distributable example. To keep a
private installation's paths in one place, use a path derived from `Cfg.TestsDir`
or `Cfg.KitDir` when invoking the builder from that installation's PowerShell.
No private path is embedded in the Python, JavaScript, template or fixtures.

## Input files and columns

The CSV headers are retained from the source. Numeric blanks mean unrecorded.
Quota rows must be newest first; the builder takes the first matching reading.
For repeated session/start pairs, the newest snapshot is kept. All example
numbers, labels and dates are fake. The examples are the executable format reference.

- `claude-sessions.csv`: date_utc, partner, session_id, thread_or_task, role,
  where, model_configured, model_last_served, effort, start_utc, last_update_utc,
  input, cache_write, cache_read, output, weighted, cost_usd_reported,
  snapshot_utc, source, notes.
- `codex-runs.csv`: date_utc, partner, machine, run_label, task, ordered_by,
  model, reasoning, start_utc, end_utc, input, cached_input, output,
  quota_5h_before, quota_5h_after, quota_week_before, quota_week_after,
  result, source, notes.
- `quota-readings.csv`: time_utc, tool, account, limit, reading, resets_utc,
  read_by, notes. Example format: tool `Claude`, limit `weekly`, reading
  `11% used`. That percentage is an invented format demonstration.
- `project-usage-screen.csv` (optional): reading_utc, thread_title, model_shown,
  last_activity_shown, tokens_shown, cache_hit_shown, share_shown, source, notes.
- `forecast-inputs.csv`: key, value, low, high, kind, source, notes.
- `forecast-versions.csv`: version, theme, plan_points, points_low, points_high,
  done_points, basis.
- `pace-forecast.csv` (optional generated file): version, plan_points,
  codex_windows, codex_pct_of_week, claude_pct_of_week, finish_best,
  finish_likely, finish_worst, paid_by.

The input keys consumed by the forecast are all present in `example/forecast-inputs.csv`.
Two formerly personal key names are now `secondary_build_share` and
`secondary_claude_pct_per_point`; the corresponding internal model fields are
`secondary` and `primary`. Migrate those keys when adapting an older ledger.
No source rows were copied, only column headers.

## What changed and what remains project-specific

The source builder, template and model assumed a particular project's two-person
workflow, fixed release roadmap, account ownership, historical credit balance,
reset dates and screen totals. This copy removes that private history and derives
version labels, dates, screen totals and policy caps from inputs. It removes
remote font links for offline output, personal category rules, credit/window
fallbacks and personal names in model identifiers. JSON embedded in HTML escapes
angle brackets and ampersands so CSV strings cannot terminate the script element.

The forecast arithmetic is retained: work in plan points, window supply, a
coordinator allowance, an optional second builder, overhead, banked resets and a
pause between versions. These are workflow assumptions, not universal billing or
service rules. The model **simulates** banked resets; it never applies one.
Model-family labels remain the source's Opus/Sonnet 5.5 groupings, token weighting
remains input + cache_write x 1.25 + cache_read x 0.1 + output x 5, and presets
remain rough estimates. They are not current prices or quota guarantees. The
new-token chart recognizes the legacy notes text `tokens used' printed: N`.
Other notes formats leave that chart value unrecorded. Blank token values can be
shown as zero by some inherited aggregate charts; consult source rows before
interpreting those totals. An empty version list is not supported by this UI.

This is a source copy for local reuse, not an automatic meter collector. No
transcript scanner is required to build the example; no account data is read.
No licence is added: both partners must decide it, and redistribution must wait.
