# Cipherleaf: execution index

**Purpose:** find an Execution Summary and its open points without replaying its contents.

**Maintainer:** Claude (Claude Code), for the owner.

**Report collection:** this folder, `docs/execution/`.

**Ordering:** newest execution first. Report IDs stay stable even if an older report is corrected.

**State owners (this index doesn't replace them):** [STATUS.md](../STATUS.md) (accepted state, where to resume), [approval-and-amendments.md](../plan/approval-and-amendments.md) (authority), [acceptance.md](../acceptance.md) (requirements versus evidence, and the owner's decisions).

Add an entry for each new Execution Summary (see the routine in [EX-06 Appendix A](2026-10-23-pbs-reconciliation.md#appendix-a-documentation-routine-going-forward)). Dates are UTC.

## Reports

### 2026-10-22 (23:24–23:40) — EX-2026-10-22-11 — Calculator border fix, Cipherleaf app icon (A15), rebuilt portable build

- Report: [2026-10-22-icon-border-fix-and-rebuild.md](2026-10-22-icon-border-fix-and-rebuild.md)
- Task / Goal: the owner's messages at 23:24:31 (white border), 23:26:19 (icon) and "rebuild and commit"; amendment [A15](../plan/A15-app-icon.md)
- Executor: Claude (Sonnet 5.5)
- Outcome: border removed in both calculators; the Cipherleaf icon generated from `assets/cipherleaf-icon.svg` and used by the exe and window; the exe, portable folder and ZIP rebuilt; smoke-tested; committed
- Integration / review: not applicable
- Acceptance: pending (the owner's check of the taskbar icon and the calculator)
- Open dependency / next action: the owner tries the build; A2 and A8 decisions

### 2026-10-22 (23:14–23:25) — EX-2026-10-22-10 — Fresh portable build; KeePassXC kept for private use

- Report: [2026-10-22-fresh-portable-build.md](2026-10-22-fresh-portable-build.md)
- Task / Goal: the owner's instruction at 23:14:05 (fresh vault, fresh portable build, keep KeePassXC for private use)
- Executor: Claude (Sonnet 5.5)
- Outcome: the exe was rebuilt from the committed code and the portable folder refreshed with no `data` folder (a fresh vault on first run); smoke-tested again (starts, `data` next to the exe, nothing in AppData, no network, closes cleanly). The dev vault was not copied.
- Integration / review: not applicable
- Acceptance: pending (owner's hands-on use)
- Open dependency / next action: the owner tries the exe; A2 and A8 decisions

### 2026-10-22 (22:58–23:15) — EX-2026-10-22-09 — Commit of A10–A14 and the first release build

- Report: [2026-10-22-release-build-and-commit.md](2026-10-22-release-build-and-commit.md)
- Task / Goal: the owner's instruction at 22:58:30 (commit; build the exe for release; screenshot note)
- Executor: Claude (Sonnet 5.5)
- Outcome: code committed (`2105a57`); release exe built; portable folder assembled outside the repository with the verified KeePassXC, its licenses and source; smoke-tested from a temporary copy (starts, `data` next to the exe, nothing in AppData, no network, closes cleanly). A vault was not created and the Passwords view not opened in the release build.
- Integration / review: not applicable
- Acceptance: pending (owner's hands-on use)
- Open dependency / next action: the owner tries the exe; move the dev vault into the portable folder or start fresh; the GPL approach before sharing (STATUS questions 4 and 9)

### 2026-10-22 (22:40–22:55) — EX-2026-10-22-08 — A14: a small calculator inside each sticky note

- Report: [2026-10-22-a14-calculator-in-sticky-notes.md](2026-10-22-a14-calculator-in-sticky-notes.md)
- Task / Goal: the owner's instruction at 22:40:13; amendment [A14](../plan/A14-calculator-in-sticky-notes.md)
- Executor: Claude (Sonnet 5.5)
- Outcome: a calculator button in each sticky note opens a small basic calculator in the note's colors; shared logic moved into a hook. Type check, build, audit, 33 Rust tests and 66 engine checks pass; works in the mock UI; real sticky window not tried.
- Integration / review: not applicable
- Acceptance: pending (owner's hands-on review)
- Open dependency / next action: the reading was confirmed by the owner at 22:55:26 UTC; commit decision (STATUS question 1)

### 2026-10-22 (22:26–22:38) — EX-2026-10-22-07 — A13: calculator, reworded and trimmed editor

- Report: [2026-10-22-a13-calculator-and-wording.md](2026-10-22-a13-calculator-and-wording.md)
- Task / Goal: the owner's instruction at 22:26:02 and answers at 22:27:00; amendment [A13](../plan/A13-calculator-and-wording.md)
- Executor: Claude (Sonnet 5.5)
- Outcome: calculator built under Sticky notes (basic and scientific); block editor reworded and trimmed with older content still displayed; the name stays Cipherleaf. Engine checks (66), type check, build, audit and 33 Rust tests pass; mock-UI checks pass; the real app was not used.
- Integration / review: not applicable
- Acceptance: pending (owner's hands-on review)
- Open dependency / next action: the owner starts the dev app and reviews A10–A13; commit and portable-build decisions (STATUS questions 1 and 4)

### 2026-10-22 (21:47 onward) — EX-2026-10-23-06 — PBS reconciliation

- Report: [2026-10-23-pbs-reconciliation.md](2026-10-23-pbs-reconciliation.md) (the filename uses the owner's local date)
- Task / Goal: the owner's request at 21:47:32 UTC to reconcile the project under the PBS skill; documentation only
- Executor: Claude (Sonnet 5.5)
- Outcome: records reconciled with the tree checked at 21:49 UTC; acceptance ledger, session history and README added; stale facts corrected. Follow-up at 22:03: the owner approved the CLAUDE.md additions, this index and the failure-intelligence record, now done.
- Integration / review: not applicable (no workers)
- Acceptance: pending (the owner hasn't reviewed the records)
- Open dependency / next action: the owner's product questions in STATUS; the decision-record proposal was not adopted

### 2026-10-22 (17:29–17:55) — EX-2026-10-22-05 — A12: Turn into list, toolbar, sub-pages, fonts, equations

- Report: [2026-10-22-a12-notion-menus-subpages.md](2026-10-22-a12-notion-menus-subpages.md)
- Task / Goal: the owner's request at 17:29:57; KaTeX approved at 17:44:21; amendment [A12](../plan/A12-notion-menus-subpages.md)
- Executor: Claude (Sonnet 5.5)
- Outcome: implemented and checked by unit tests, scripts and screenshots (mock data); partly unverified against the real backend and by hand
- Integration / review: not applicable
- Acceptance: pending (owner's hands-on review)
- Open dependency / next action: the owner's decision about trimming or rewording Notion-like features (STATUS question 2)

### 2026-10-22 (16:55–17:25) — EX-2026-10-22-04 — A11: new look and Notion-style notes with pictures

- Report: [2026-10-22-a11-redesign-and-block-notes.md](2026-10-22-a11-redesign-and-block-notes.md)
- Task / Goal: the owner's request at 16:55:27; amendment [A11](../plan/A11-redesign-and-block-notes.md)
- Executor: Claude (Sonnet 5.5)
- Outcome: implemented; pictures, covers and icons also checked against a throwaway real vault
- Integration / review: not applicable
- Acceptance: pending; the owner's screenshot with "happy" (18:06:46) is informal only
- Open dependency / next action: as for EX-05

### 2026-10-22 (16:12–16:50) — EX-2026-10-22-03 — A10: KeePassXC inside the window, black-and-white theme, font size

- Report: [2026-10-22-a10-embed-and-appearance.md](2026-10-22-a10-embed-and-appearance.md)
- Task / Goal: the owner's request at 16:12:20; amendment [A10](../plan/A10-embed-and-appearance.md)
- Executor: Claude (Sonnet 5.5)
- Outcome: implemented; embedding checked by script on a throwaway vault; typing inside the embedded KeePassXC not checked
- Integration / review: not applicable
- Acceptance: pending
- Open dependency / next action: the owner's hands-on check of the Passwords view

### 2026-10-22 (13:08–16:10) — EX-2026-10-22-02 — A9: bundled KeePassXC (D0 and D1) and first launch

- Report: [2026-10-22-a9-keepassxc-d0-d1.md](2026-10-22-a9-keepassxc-d0-d1.md)
- Task / Goal: [A9](../plan/A9-bundle-keepassxc.md), approved at 13:07:49
- Executor: Claude (Opus 5.5, then Sonnet 5.5 for the resumed work)
- Outcome: KeePassXC 2.8.0 downloaded and verified (hash and signature); Cipherleaf's own password code replaced by a panel; first launch worked; debug-port exposure recorded
- Integration / review: not applicable
- Acceptance: pending
- Open dependency / next action: the GPL approach before distribution (A9 Q5). The debug-port exposure is now [FI-01](../failure-intelligence.md#fi-01--debug-port-left-open-on-the-owners-live-window).

### 2026-10-22 (11:08–11:51) — EX-2026-10-22-01 — Toolchain, scaffold and first full implementation

- Report: [2026-10-22-initial-build.md](2026-10-22-initial-build.md)
- Task / Goal: the approved plan ([snapshot](../plan/approved-plan-2026-10-22.md)), approved at 11:08:21
- Executor: Claude (Sonnet 5.5 for planning and the scaffold; Opus 5.5 for app code)
- Outcome: code written for the planned features; 27 unit tests passed at 11:37; the UI type-checked; never run at that point
- Integration / review: not applicable
- Acceptance: pending
- Open dependency / next action: A2 and A8 decisions; A1 and A3–A7 evidence
