# Evidence, interpretation and delivery

## Source ledger and coverage

Preserve raw measurement, selected display value, unit, exact model/configuration, source location/URL, source role, measurement/publication/retrieval dates as applicable, benchmark/version/task set, harness/settings and limitations. Track observed, reported, derived, reconciled and unavailable statuses separately. Retain source precision and use sensible consistent display rounding.

Every requested item/metric needs either a plotted value or an explicit omission reason. Missing data remains missing. Reported 0.0 is a real zero. A percentage/relative advantage without absolute measurements does not establish either bar height.

Do not pool independent, vendor-reported and personal results into one leaderboard. Separate incompatible benchmark versions, task subsets, scoring rules, evaluation snapshots/harnesses and price snapshots. Flag unknown repeat counts, budget parity and remaining condition differences.

For screenshot extraction, inspect original pixels if available; cross-check totals and ratios, keep conflicting OCR readings and document the chosen value. A ratio-based reconciliation is derived evidence, not visual verification. If key identity/units are missing, record what is visible and hold the dependent comparison pending clarification. Missing logos can be replaced with text without blocking a chart.

## Arithmetic and claims

| Quantity | Calculation / limit |
| --- | --- |
| Relative rate change | (new rate / old rate - 1) × 100 |
| Equal-work time change | (old rate / new rate - 1) × 100 |
| Percentage-point change | new percentage - old percentage |
| Used share | used / total × 100 |
| Headroom | total - used |

Check denominators and mark undefined calculations rather than forcing division by zero. Identify adjacent and initial baselines. Keep percentage-point changes distinct from relative percentage changes.

The archived arithmetic example 64.6 → 41.2 tokens/s is a 36.2% rate drop and about 56.8% more time for the same output count. It is not simply 36.2% longer. Actual waiting also depends on generated length and measurement scope; inspect both token count and generation time when present.

Keep prompt-processing rate separate from output-generation rate, and generation time separate from click-to-visible-text/first-token latency. Cache hits may be part of prompt totals and must not be counted twice. RAM and VRAM are separate pools; whole-system readings are not model-only allocations.

A final context/memory snapshot is not a history of each reply. Slowdown plus high occupancy does not prove paging, offload, memory pressure or context growth caused it. Label hypotheses and the measurements needed to test them. Do not invent repeated trials, error bars, significance or broad capability conclusions from a small record.

## Export review

Before delivery, open every final rendered image and check:
- Source-to-plot values; exact models/settings/dates; derived/reconciled/missing labels; complete coverage.
- Correct units, direction, baselines, sorting/cohorts and source role.
- Supported takeaway, paired-metric reasoning and important limitations visible in the standalone PNG.
- Clipping/overlap, contrast, logo proportions, text hierarchy and intended-screen/phone readability.
- For Arabic, proper shaping/RTL and mixed-direction labels; identical data across language versions.

Do not claim image inspection, a write, upload or reproduction check unless actually completed.

## Deliverables and revisions

Default to PNGs with concise findings and accessible data/text when available. Use clear filenames with a date/version. If a reproducibility package is requested, include exact CSV, source notes, calculations, runnable script, required assets/licenses and README. Test the package before sharing; document actual environment requirements without silently installing packages.

When revising labels, numbering or captions, synchronize affected PNGs, source ledger/CSV, scripts, page references and download packages. Distinguish a relabeling edition from a new measurement run.

## Requested Notion delivery only

Publication requires a user request and an available authorized connector. Read an existing target before editing and preserve unrelated content. Use native visible image blocks in scrollable groups, a brief overview, short takeaways beside each chart and collapsed contents/source-methodology details. Do not hide charts in toggles or offer only download links.

Use the authorized destination. If the user requests a new Notion page without naming a parent, a private standalone draft is appropriate when supported. Do not move or share pages without direction. Verify the saved page and downloads before reporting success.

If access/publication is unavailable, return images and ready-to-paste content and state the actual limitation. This installed skill does not require Notion to make charts.

## Source provenance

Derived from Ahmed's [Chart-Making Playbook](https://app.notion.com/p/3f4bd7ade63b817ea5b9e280616c81ab), version 1.0 (9 October 2026). Its full copy-ready brief was read via the authorized Notion connector during installation. The extracted UTF-8 brief was 11,436 bytes, matching the stated portable brief size. Library ID: libfile_4614034a20d081919205b06b3faa9402, Chart-Making-AI-Brief.txt. Library materialization was attempted twice; the second attempt returned HTTP 403 and no local Library file. The Library bytes were therefore not compared; the Notion brief is the implementation source.

