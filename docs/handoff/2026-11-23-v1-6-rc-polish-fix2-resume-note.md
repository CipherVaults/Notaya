# Notaya 1.6 RC audit #2 fixes

Ahmed (AE0101) · Gaming-PC · 2026-11-23 UTC

Ahmed authorized the three blocking fixes from independent audit #2 of `3111706e4368bc46a0d6d9f8b829f6a09b483ced` on `AE0101/v1-6-rc`. The owner-message timestamp was unavailable. The stopping point is the local commit, verified incremental bundle, and fast-forward of the designated checkout.

## Changes

- The Shell accepts plain Ctrl shortcuts. A protected vault stays open when Ctrl+Shift+L left-aligns the active paragraph. Plain Ctrl+L still invokes the lock.
- Escape cancels a property draft inside either row peek. Note Find still takes its own first Escape, and a further Escape from the editor body closes the peek.
- Sticky images retain their drag style. Ordinary chrome images retain browser drag suppression. Full-page pictures keep their existing block drag handle.

## Evidence and next step

The local `v1-6-rc-polish-fix2-summary.txt` receipt records the exact commit, isolated commands, counts, source hashes and bundle replay. The accompanying incremental bundle requires the audited commit. Use the receipt and that source for independent review and separate packaged WebView2 qualification. Publication stays held. [Session 151](../sessions.md#session-151).
