---
name: personalized-charts
description: Create or revise Ahmed's personalized comparison charts, especially AI benchmarks, local-AI test sequences, and quality-versus-cost or effort charts. Use for chart requests referencing his Chart-Making Playbook or established style, preserving exact settings, source roles, and evidence limits.
---

# Personalized charts

Turn the supplied data and question into clear, technically honest comparisons. Use the information-first feel of Hardware Unboxed / TechSpot graphs as inspiration without copying their branding or implying authorship. This skill is self-contained; the original conversation and live Notion access are not required.

## Frame the task

Identify the decision, requested items/metrics, evidence category, relevant settings, dates, language, destination and outputs. Read all relevant input rows before choosing figures. Ask only for a missing detail that materially changes the result; choose reversible layout details yourself. Hold a dependent comparison when critical identity or units are unknown.

Use a reproducible renderer available in the environment: Matplotlib, Pillow, SVG or another accurate charting tool. Generate numeric charts from structured data, never invented image-model values. This skill neither installs dependencies nor authorizes new benchmarks, settings changes, external publication, moving/sharing pages or unrelated edits.

## Establish the evidence

Build a source ledger with raw and displayed values, units, exact item/configuration, source location/URL and role, dates, benchmark/version/task set, relevant harness/settings, and limitations. Distinguish observed, source-reported, calculated, OCR-reconciled and unavailable values.

- Show independent evaluation, vendor-reported evidence and personal tests separately. Separate incompatible benchmark versions, task sets, scoring rules, evaluation/harness and price snapshots into labeled panels.
- Preserve effort, quantization, context, fallback, software version and other consequential conditions. Equal effort labels do not establish equal compute.
- Account for every requested model/metric with a value or explicit omission reason. Missing is not zero; genuine reported zero remains zero. Relative gaps alone cannot supply absolute bar heights.
- For screenshots, inspect original pixels when possible and retain conflicting readings in a reconciliation note. Arithmetic can support reconciliation but cannot verify a pixel reading. Say when pixels could not be checked.
- Never invent observations, error bars, repeated trials, precision or statistical significance.

Read [evidence-and-delivery.md](references/evidence-and-delivery.md) when doing calculations, handling screenshot uncertainty, interpreting performance/memory, or preparing final exports.

## Choose the minimum useful chart set

| Question | Preferred treatment |
| --- | --- |
| Compare compatible configurations | One shared horizontal bar chart, sorted within the valid cohort; exact configuration labels and direct values. |
| Changes across personal replies/runs | Chronological bars or observed points; distinguish adjacent changes from first-relative changes. |
| Quality versus cost/effort | Observed scatter/line points, effort labels and exact score-cost pairs. Color plus markers/line styles; disclose log/focused axes. |
| One capacity pool at a moment | Used/headroom display with used and total amounts, percentage, snapshot date and scope. Keep RAM and VRAM separate. |
| Waiting-time differences | Separate output length, actual generation time and rate/equal-work arithmetic into panels. |
| Only one compatible result | Honest singleton/card with limitation; no invented comparator. |

Bars start at zero; signed changes need a visible zero reference. Separate incompatible units. Connecting score-cost segments only guide the eye between measured settings, with no interpolation or prediction. Do not rank Elo by Elo-per-dollar or select a universal winner from one ratio. Avoid 3-D, perspective distortion and decorative gradients in data marks.

## Apply the visual identity

Use white, dark blue-gray text, readable sans-serif typography, light gray gridlines, generous spacing, strong titles and direct numbers. Blue is the comparison color; orange is a selective highlight/second metric. Meaning must survive without color.

Each exported figure needs a specific title, short question/scope subtitle, units and improvement direction where meaningful, exact item/settings labels, direct values, one supported takeaway, and compact source/date/conditions notes. Keep long methodology outside the plotting area, but keep consequential caveats inside the image.

Read [visual-patterns.md](references/visual-patterns.md) for the appropriate header and configurable palette/type recipe:
- Published benchmarks: compact upper-right source credit with explicit evaluator/vendor role.
- Personal local-AI tests: relevant software/model-maker identification, personal-test date, prominent model + quantization, optional relevant hardware/software/context strip.
- Other subjects: use only relevant elements; no forced hardware strip or corporate branding.

Use authentic available logos only when useful and permitted; otherwise use plain text credits. Never invent logos or imply endorsement.

## Verify and deliver

Open every rendered image and correct value/label mismatches, missing coverage, grouping, baseline/unit errors, clipping, overlap, contrast, logo distortion and viewing-size legibility. Check at intended size, including phone width; high resolution alone does not make small type readable. Provide an accessible data/text companion when needed.

Return PNGs and concise supported findings; add data, scripts or a package when requested. Use [evidence-and-delivery.md](references/evidence-and-delivery.md) for package contents, Arabic/bilingual checks, version synchronization and requested Notion delivery. Verify actual outputs and requested writes before claiming success.

Source: Ahmed's [Chart-Making Playbook, version 1.0, 9 October 2026](https://app.notion.com/p/3f4bd7ade63b817ea5b9e280616c81ab). Archived examples guide the style, not current benchmark facts. Update lasting preferences only when requested; keep one-off settings with their report.

