# EX-2026-10-27-28: live audit (PR #10) compared with V1.1.0

Short form (4 sections). Docs only. Times are UTC: the owner's message timestamps and the cloud container's clock.

## 1. Task Overview

The owner asked "check the live audit" (2026-10-27 17:07:25 UTC) about the idle session "Live PC audit checks" linked to PR #10, then "compare the current verison with this thread to see if everthign got fixed" (17:11:44), "stop now at resume point" (17:13:39) and "Resume" (18:56:19). Executor: Claude (Claude Code, "Kevin") in a cloud thread; read-only until the records PR. Work 17:08 to 17:14 and 18:56 to about 19:05 UTC (system clock).

## 2. What was done

- Read PR #10 on GitHub: merged by the owner 2026-10-24 09:23:51 UTC (`e696e11`), one file, `docs/audit/2026-10-23-live-checks.md`. The PC session that wrote it is finished (report merged, its temp folder deleted by the owner on 2026-10-24).
- Read the whole "Live PC audit checks" thread (2026-10-23 22:03 UTC to 2026-10-25) and compared every finding with `main` `6981442` (code identical to the V1.1.0 tag commit `70d38a1`), in the code and in the records.
- Result: [docs/audit/2026-10-27-live-checks-vs-v1-1-0.md](../audit/2026-10-27-live-checks-vs-v1-1-0.md): 5 fixed (F-02, F-13, F-11, ctutils, F-09), 3 partly fixed, 1 no longer relevant (SE-04, KeePassXC removed), 15 still open, 2 passed. Not everything is fixed. The largest open items: WebView2 traffic (plan postponed), F-04, F-10, SE-03, CR-02, contrast and focus.

## 3. Checks

- Code read in a cloud checkout of `main` `6981442`; `git diff 70d38a1 HEAD -- src src-tauri` is empty. `npm audit --package-lock-only`: 0 vulnerabilities. `Cargo.lock` has `ctutils` 0.4.5.
- **Not run:** `cargo audit`, `cargo test`, `cargo clippy`, the exe, the UI, axe. Nothing was run on Windows. Rows marked "not re-measured" are from code reading only.

## 4. Open items

- The owner merges this PR; then his PC copy is fast-forwarded to the same `main` (record-every-step rule, 2026-10-27 16:57 UTC).
- Whether to plan a fix release for the open items. Nothing starts without the owner's word. Nothing is accepted.
