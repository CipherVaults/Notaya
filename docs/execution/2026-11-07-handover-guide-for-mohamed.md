# EX-2026-11-07-47: handover guide for Mohamed (Markdown copy in docs/handoff)

**Who and where (A38):** Ahmed's side (`AE0101`); Claude cloud thread "Build a handover guide for Mohamed" in Ahmed's Claude project "Notaya" (machine: cloud). Not recorded: the Claude effort setting.

Short form (4 sections). Normal size: records only, no app code. Times are UTC: message timestamps, `git log` and the container clock.

## 1. Task Overview

- **Baseline:** `origin/main` `987bfc5` (merge of PR #56).
- **Authority:** Ahmed's message of 22:24:27 UTC asked for a published page that carries the frameworks, methods and new findings of his side over to Mohamed's machine, with the writer of each part clear ("which section belongs to which machine, and which account"). The coordinator then asked (22:28:16 UTC) for a Markdown copy in `docs/handoff/`, because the repository is the only memory the two Claudes share. That request is a colleague's, not permission; the permission is Ahmed's 22:24:27 message, and the merge rule is A37 item 3 (his coordinator merges inside work Ahmed approved).
- **Scope and write area:**
  - `docs/handoff/2026-11-07-handover-guide-for-mohamed.md` (new);
  - `docs/handoff/claims.md`, `docs/execution/README.md`, `docs/sessions.md` (entry 42 and its index row), `docs/STATUS.md` (one line), and this report.
- **Out of scope:** app code, dependencies, security behaviour, the plan snapshot, Mohamed's own records. The guide does not change any rule; it only reads the existing ones.
- **Returns:** draft PR #57, merged with a merge commit. **Stop point:** PR #57 merged after the checks. The guide is not sent to Mohamed; Ahmed decides that.
- **Branch:** `claude/project-thread-jh0sak`, from `main` `987bfc5`.

## 2. Work Performed

1. Read, in full where the guide relies on them: `CLAUDE.md`, `AGENTS.md`, `docs/STATUS.md` (current state, open questions, next action, where things live), A37 and A38 with their dated notes and the 2026-11-07 authority rows, the top of `docs/sessions.md` (index and entry 41), `docs/ledger/README.md` and the first ledger blocks, `docs/handoff/collaboration-guide.md` and `claims.md`, and the project memory and shared notes.
2. Published the guide as a private page (Artifact), then wrote the same content as Markdown. Every item is marked **shared rule**, **Ahmed's side** (not yet agreed by Mohamed) or **needs both**.
3. The guide asks Mohamed to confirm A34 (STATUS question 27, never recorded) and to say which of Ahmed's method rules he wants on his side.
4. Added the claims row first (EX-47 and session entry 42 reserved), then the guide, this report, its index row, session entry 42, and one STATUS line.

**Not read in full for the guide:** older sessions, the other execution reports, `docs/security-design.md`, `docs/acceptance.md`, plan records A9 to A35. The guide's "exists today" column quotes Ahmed's inventory of the code at `9474511`; it was not re-checked.

## 3. Verification and Evidence

- **Checks run (this exact source):** the guide's links to repository files were checked to exist on `origin/main`; ID numbers (EX-47, session 42) were checked free across `origin/main` and every remote branch with the commands in the collaboration guide, section 4 (highest in use: EX-46, A38). Results are in the PR description.
- **Not run:** any app check (`tsc`, build, Rust tests), because no code changed; a read of the guide by Mohamed.
- **Result state:** produced and verified as above; integrated on merge; **not accepted** (acceptance is a partner's word).

## 4. Outcome and Open Points

- Mohamed's Claude can now read the guide from the repository (`docs/handoff/2026-11-07-handover-guide-for-mohamed.md`).
- Open: Mohamed has not seen it. Ahmed decides when to tell him.
- Open: the guide is a snapshot of 2026-11-07 22:30 UTC. Ahmed's decisions are moving; whoever changes a rule updates the owning record first, and this guide only if still needed.
- Ledger: this thread's usage is **not recorded** here; Ahmed's coordinator said another thread handles the ledger refresh (22:28:16 UTC).
