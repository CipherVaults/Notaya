# Execution Summary: EX-2026-10-22-07 — A13: calculator, reworded and trimmed editor

## 1. Task Overview

**Task requested** (the owner, 2026-10-22 22:26:02 UTC): continue the app development with a calculator, keep the name Cipherleaf, and reword (or reword and trim) the features closest to Notion. Claude asked which option and how big the calculator; the owner chose **reword and trim** and **basic and scientific** (22:27:00 UTC).

**Execution mode:** Direct project. **Executor:** Claude (Sonnet 5.5). **Date:** 2026-10-22, about 22:26–22:38 UTC (the next owner message arrived at 22:40:13; clock times from the system clock and the transcript).

**Authority:** the instruction above, recorded as [A13](../plan/A13-calculator-and-wording.md). Development had been paused for the reconciliation ([EX-06](2026-10-23-pbs-reconciliation.md)); this instruction resumed it for these items only. **Stopping condition:** stop for the owner's hands-on review. Nothing is committed (the owner has not asked for a commit).

## 2. Context Used

STATUS and its open questions 2, 3 and 5; the editor code from A11 and A12; the owner's earlier concern (17:56 UTC) about resemblance to Notion.

## 3. Work Performed

1. **Calculator engine** `src/features/sticky/calc.ts`: a hand-written parser (no `eval`) with precedence, implicit multiplication, percent, factorial, constants, functions, degree/radian modes, clear error messages and number formatting.
2. **Calculator panel** `src/features/sticky/Calculator.tsx`, shown under the notes in `StickyBoard.tsx` (the old full-screen empty state became an inline note so the calculator is always reachable).
3. **Reword and trim:** new block names and the `/` menu wording (`SlashMenu.tsx`, `turnInto.tsx`); a new horizontal selection bar (`BubbleToolbar.tsx`); "Change type" in the block menu; removed the creation paths for columns, toggle headings, heading 4, "Page in", comments, the emoji button, covers and icons (`PageHeader.tsx` now only shows and removes existing ones). The schema still reads the older blocks, so existing pages lose nothing.
4. Removed the word "Notion" from the code comments.
5. `dev-mock/calc-check.cjs`: the calculator checks, runnable from the repository.

## 4. Commands / Tools Used

`npx tsc` (for the project and, with `--ignoreConfig`, for the calculator engine alone), `node dev-mock/calc-check.cjs`, `npm run build`, `npm audit`, `cargo test --lib`; the mock UI with a headless browser and scripted events (a throwaway process, closed afterwards; nothing touched the owner's vault or window).

## 5. Verification / Testing

| Check | Subject | Outcome | Applicability and limits |
|---|---|---|---|
| Calculator engine checks (66 cases: arithmetic, precedence, percent, factorial, constants, functions, degrees and radians, `Ans`, 15 error cases, number formatting) | `calc.ts` as of this step | pass, `ALL OK`; two edge cases found by the first run (`1..2` accepted; a 15-digit integer shortened) were fixed and re-run | current; engine only |
| `npx tsc --noEmit -p tsconfig.json` | final tree | pass | current |
| `npm run build` | final tree | pass | current |
| `npm audit` | final tree | 0 vulnerabilities | current |
| `cargo test --lib` | final tree (no Rust changed in this step) | pass, 33/33 | current |
| Mock UI scripts: calculator typing, Enter, history, scientific keys (`sin(90)` = 1), `1/0` message; horizontal toolbar; block-type list; `/` menu; an older page with columns, a toggle heading, a comment and a heading 4 still displays | Mock data, headless browser, synthetic events | pass | current for the mock only |

**Not verified:** the clipboard Copy inside the real app (it falls back to the older copy method if the clipboard API is refused); real mouse and keyboard use; the calculator inside the real window; whether the new wording and toolbar read well to the owner. In the real app a page that already has columns or comments was not tested (the owner's pages are not known to contain any).

## 6. Problems Encountered

- A scripted import edit removed the first three import lines of `SlashMenu.tsx`; found before the type check and restored.
- The first mock run showed a blank page because the dev server was still warming up; a re-run worked.
- A test that clicked calculator keys in one tick showed stale state (the keys read the previous text). Real clicks arrive in separate events, so this is a test artifact; the keys were re-tested with pauses between them.
- A double-click to the right of a paragraph's last line selected into the next block, so one test changed two blocks. That is normal browser selection behavior, not a defect.

## 7. Documentation Impact

Added A13 and this file; updated the amendments record, STATUS, the acceptance ledger, the session history, the execution index, `security-design.md` (calculator note) and the README (feature list). A11 and A12 got a one-line note that A13 partly supersedes them.

## 8. Git / Version Control Impact

Nothing committed. New and changed code paths are uncommitted on branch `feat/a9-keepassxc`, together with A10–A12.

## 9. Decisions Made During Execution

- Keep the schema for removed block types so older pages still display (a data-safety choice, not requested by the owner). Basis: removing the types would silently drop content from any page that already has them.
- Calculator history is memory-only, not stored in the vault. Basis: simplest and most private; it can be added later if wanted.
- The calculator sits under the notes in the Sticky notes view, as the owner asked ("under sticky notes"), and not as a floating window.
- Percent follows handheld-calculator behavior (`200 + 10%` = 220). Basis: what most people expect.

## 10. Items That Need Human / Coordinator Review

- Does the calculator do what the owner wants (keys, percent behavior, history)?
- Do the new menu names and the horizontal toolbar feel acceptable, and are they different enough from the reference they were originally based on?
- Whether to commit A10–A13.

## 11. Current Result

Implemented and checked at engine, build and mock-UI level as listed in section 5. **Not accepted;** only the owner accepts results.

## 12. Recommended Next Step

The owner starts the dev app (it is not running), tries the Sticky notes calculator and the notes editor, and says what to change. Then decide the commit (STATUS question 1) and the portable build (question 4).

**Do not repeat:** the reconciliation, the calculator engine design.
