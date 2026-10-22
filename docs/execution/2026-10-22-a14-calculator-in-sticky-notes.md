# Execution Summary: EX-2026-10-22-08 — A14: a small calculator inside each sticky note

## 1. Task Overview

**Task requested** (the owner, 2026-10-22 22:40:13 UTC): a small calculator in the sticky note itself, or a small floating one they can click on.

**Execution mode:** Direct project. **Executor:** Claude (Sonnet 5.5). **Date:** 2026-10-22, about 22:40–22:55 UTC.

**Authority:** that instruction, recorded as [A14](../plan/A14-calculator-in-sticky-notes.md). Development was already resumed for A13 (22:26:02); this is a follow-on request. **Stopping condition:** stop for the owner's review. Nothing is committed.

## 2. Context Used

A13's calculator engine and panel; `StickyWindow.tsx`; the mock harness's support for opening a sticky window (`?label=sticky-…`).

## 3. Work Performed

1. `useCalc.ts`: the calculator's state and actions moved out of the big calculator into a hook, so two calculators share them. `Calculator.tsx` now uses it (its behavior is unchanged; the typed-text update is now functional, which also removes a stale-state quirk when keys are pressed in the same tick).
2. `MiniCalculator.tsx`: the small calculator, in the note's colors.
3. `StickyWindow.tsx`: a calculator button in the top bar and a panel that replaces the note body while open; the editor stays mounted (hidden) so nothing is lost. Removed the strikethrough button to keep the bar from overflowing at the minimum note width.

## 4. Commands / Tools Used

`npx tsc`, `npm run build`, `npm audit`, `cargo test --lib`, the calculator engine checks (`dev-mock/calc-check.cjs`); the mock UI with a headless browser opened as a sticky window at 330×350 and 320×320 (a throwaway process; nothing touched the owner's vault or window).

## 5. Verification / Testing

| Check | Subject | Outcome | Applicability and limits |
|---|---|---|---|
| Calculator engine checks (66 cases) | `calc.ts` (unchanged in this step) | pass | current |
| `npx tsc --noEmit -p tsconfig.json` | final tree | pass | current |
| `npm run build` | final tree | pass | current |
| `npm audit` | final tree | 0 vulnerabilities | current |
| `cargo test --lib` | final tree (no Rust changed) | pass, 33/33 | current |
| Mock UI as a sticky window: the calculator button opens the panel; typing `12.5*(3+4)` shows a live `= 87.5`; Enter gives 87.5; the first Esc clears the display and the second closes the calculator back to the note; the panel fits a 330×350 window | Mock data, headless browser, synthetic events | pass | current for the mock only |

**Not verified:** a real sticky window (frameless, always-on-top, saved size and position), real mouse use, the panel at the smallest note size, the big calculator after the hook change in the real app (it was re-checked only by the type check, build and the earlier mock behavior).

## 6. Problems Encountered

- A first test of Esc used a selector that also matched the calculator button, so it reported the wrong result; the check was corrected and re-run.
- A test that pressed Esc twice in one tick showed the stale-state effect (both presses saw the old text). Real key presses arrive in separate events; the test was re-run with a pause.
- A JSON test file lost its backslashes in the tool; the regular expression was replaced with a simple slice.

## 7. Documentation Impact

Added A14 and this file; updated the amendments record, STATUS, the acceptance ledger (X10), the execution index, the session history, the security design (P15 now covers the small calculator), the README.

## 8. Git / Version Control Impact

Nothing committed. Changes are uncommitted on branch `feat/a9-keepassxc` together with A10–A13.

## 9. Decisions Made During Execution

- Read "floating small one" as the sticky note's own window (already floating) with the calculator inside, instead of building a new calculator-only window. Basis: no new window code or Rust change, and the owner offered "in the sticky note itself" first. The other reading is a clear alternative.
- Small calculator is basic only, no history, no Copy. Basis: it must stay small.
- Removed the strikethrough button from the note's top bar. Basis: space at the minimum width. It's reversible.

## 10. Items That Need Human / Coordinator Review

- Is a calculator inside the note what the owner meant, or do they want a separate small floating calculator window?
- Is the loss of the strikethrough button acceptable?
- Does the small calculator need Copy or scientific keys?

## 11. Current Result

Implemented and checked at build, engine and mock-UI level as in section 5. **Not accepted;** only the owner accepts results.

## 12. Recommended Next Step

The owner opens a sticky note in the dev app (it is running, but needs the rebuilt UI, which hot-reloads), clicks the calculator button, and says what to change.

**Do not repeat:** the calculator engine design.

## Follow-up, 2026-10-23 (owner's report with a screenshot, about 01:42 on the owner's phone clock)

The owner pointed at a white border around the calculator's display (in the small calculator inside a sticky note). Cause: the app's global focus rule (`*:focus-visible`, a 2 px outline) overrides the utility class `outline-none`, so the display input showed a focus outline. Fix: both calculators' expression inputs now set `outline: none` and `box-shadow: none` inline (`MiniCalculator.tsx`, `Calculator.tsx`). Checked in the mock UI: computed outline style `none` and no shadow on the focused input in both calculators; the screenshot shows no border. `tsc` and `npm run build` pass. The release exe and ZIP made before 23:30 UTC did not include this fix; the rebuild at 23:30 UTC does (see [EX-11](2026-10-22-icon-border-fix-and-rebuild.md)).
