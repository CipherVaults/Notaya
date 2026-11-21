# Compact-table redo checkpoint — EX-2026-11-21-85

Recorded November 21, 2026, 22:08 Cairo. Scope is the local Notaya v1.6 comparison and its next admissible action. The integration base is `a806ae3e3b4a74940ee4107784a15fee7468cdf8`, tree `0696b773261a74a6963c07bf35b0d2c94fa76e6c`. The comparison ran on the separate database source `d9aadf64d8f7a18829a09d9e9401460b9c9e47cc`. These were test-only observations.

## Authority and recovery

The owner directed local continuation from the verified copy. Code and public-safe project records are to be checkpointed in Git and synced to GitHub frequently after review. Vaults, private or private-looking inputs, raw local evidence and machine-only recovery material stay local. The same-PC restore checked the archive, payload, pinned source and dependency inputs, and standalone browser files. Separate off-machine restoration of the raw evidence is unverified.

## Exact comparison

The same V5 harness mounted the actual React table in standalone Chrome Headless Shell in both runs. All 33 named cases ran in the original order. Both versions used the same isolation, browser settings and limits.

| Source | Results | Evidence |
| --- | --- | --- |
| Original | 19 passed, 14 failed | Lost focus to the document body after the row disappeared in all eight natural redo cases. Redo did not run and the pending redo entry remained. |
| Preserved candidate | 32 passed, 1 failed | Returned focus to the originating table and restored the row and expected stack in all eight natural redo cases. |

The full comparison **failed**. Both versions failed the background-document precondition before the case could exercise the repair. Each run also recorded two fixture callback errors and four unexpected editing calls. No fixture write or unexpected network request was observed. Independent readback confirmed the expected case order, input identity, runtime closure, and closure of both loopback listeners. The raw run data is retained locally.

## Other scoped evidence

- Exact `a806ae3` Windows libtest evidence covers 14 selected synthetic import/export and renderer checks. The pre-N3 defensive baseline disposition is recorded locally. Direct-read limits, native rendering and aggregate PDF and ZIP memory remain open.
- The archived v1.5.1 executable returned all nine expected note titles from a closed, disposable journal-bearing vault created by the current merged Rust code. Both the producer and reader copy were unchanged after close. This is a compatibility precursor. The exact release candidate still needs this test and the approved first-unlock safety copy and notice.

## Continuation

The focus change has not been applied or merged. Obtain a verified setup that observes real background-tab focus and accounts for expected test-only Tauri traffic without weakening any case or safety assertion. Run both sources again under an independently reviewed common setup. Require a qualifying comparison before the unchanged nine-phase 5,000-row integration workflow, then review that exact revision before merge. Query windows and cache [P7] follow the focus gate. Native persistence, release readiness, owner acceptance and publication remain separate.

[Current status](../STATUS.md#q1-local-comparison-20261121) · [Session 149](../sessions.md#session-149) · [Usage block 88](../ledger/usage-ledger.md#codex-usage-20261121-block88) · [Authority update](../plan/approval-and-amendments.md#q1-local-recovery-20261121)
