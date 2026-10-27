# EX-2026-10-27-25: combined test of PRs #17 to #20, then a check of `main` after the merges

Short form (4 sections). **Reconstructed** on 2026-10-27 (about 16:40 UTC, system clock) in the records catch-up ([EX-2026-10-27-26](2026-10-27-records-catch-up.md)) from the messages of the thread "Test the four new changes together" and of the project chat, because no record was written at the time. The numbers below are that thread's own report from the owner's PC (model output); they were not re-run for this record. Times are the system clock of the PC session and the GitHub merge times, both UTC.

## 1. Task Overview

The owner wrote "Pause the plan for now let is finish the app / Yes test" at 2026-10-27 07:21:42 UTC (message timestamp). "Yes test" answered the offer to test the four draft PRs from his message of 06:45:11 UTC together in the real app: [#17](https://github.com/AbyssWatcher2/CipherLeaf/pull/17) sticky multi-delete (A26), [#18](https://github.com/AbyssWatcher2/CipherLeaf/pull/18) ribbon (A27), [#19](https://github.com/AbyssWatcher2/CipherLeaf/pull/19) pop-out calculators (A25) and [#20](https://github.com/AbyssWatcher2/CipherLeaf/pull/20) KeePassXC removal (A24). After the owner merged them ("Done check them", 08:05:22; "Check out", 08:05:45 UTC), the same thread checked `main`. Executor: a Claude Code session on the owner's PC in a throwaway copy under `%TEMP%`, throwaway vaults only.

## 2. What was done

- **Combined the four PRs** (07:22 to 07:39 UTC) in the order 18, 17, 20, 19. Code conflicts: #17 and #19 both changed the sticky notes toolbar (the result has Select, Calculator, Show all and New note); #19 still had KeePassXC code that #20 removes (taken out of #19); both #19 and #20 used A24, so the calculator amendment became A25. Every pair also conflicted in the execution index.
- **Pushed merge commits** to the PR branches so each merges cleanly after the one before it, with no force push and nothing merged into `main`: `e589baf` (ribbon into #17), `ef548eb` (#17 into #20) and `996ae34` (#20 into #19, with the renumbering to A25).
- **Checked `main` after the merges** (08:06 to 08:20 UTC) at `4600561`, the merge of PR #19. The owner's merges, GitHub times: #18 08:04:20 (`b523df8`), #17 08:04:39 (`d69a3f7`), #20 08:04:55 (`7f64b86`), #19 08:05:20 (`4600561`). The calculator history limit of 200 (`58c299c`) and its record (`b995e4a`) came in with #19.

## 3. Checks

| Check | Combined build (07:22 to 07:39 UTC) | `main` `4600561` (08:06 to 08:20 UTC) |
|---|---|---|
| `cargo test --lib` | 128 of 128 passed (124 at the stage right after #20) | 128 of 128 passed |
| Type check and build | clean, the 2 old warnings only | clean, release build clean, the 2 old warnings only |
| Exe SHA-256 (as reported, shortened) | `A1BAADD3…F87433` | `7B18B3E9…DFC0358` |
| Existing real-exe scripts | optional security 47 of 47 (down from 52 because the KeePassXC checks went), restore 13 of 13, lock flow as designed | not run (only the 200-line commits changed since) |
| New feature checks in the real app | 23 of 23: KeePassXC commands and the Passwords entry gone; the ribbon shows, Bold works, the "/" menu opens; 4 sticky notes selected, "Delete 4 sticky notes?", all 4 deleted; two calculator windows share one history, close when the vault locks, the history is there after unlocking. A first run had 4 failures caused by the test script (it clicked a note's own Delete button); the fixed run passed. | 27 of 27: the same, plus no Passwords switch in Settings; 210 more entries leave exactly 200, newest first; a repeated line is saved once; after a normal close (no forced exit) and a restart all 200 lines are still there |
| Clean-up | `%TEMP%\clf-integ-0727` and `clf-integ-target` left on the PC (deletion offered) | debug port 9335 closed, the test variable not set, the throwaway folder deleted |

**Not run:** a hand click-through, the release ZIP (built later in [EX-2026-10-27-24](2026-10-27-v1-1-0-portable-build.md)), screen readers.

## 4. Open items

- The owner's acceptance of A24 to A27; nothing is accepted.
- The two `%TEMP%` folders above are among the `clf-*` folders the owner was given a delete command for at 08:42:34 UTC; whether he ran it is not recorded.
