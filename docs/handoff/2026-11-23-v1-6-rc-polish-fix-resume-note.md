# Notaya 1.6 RC keyboard fixes

Ahmed (AE0101) · Gaming-PC · 2026-11-23 UTC

Ahmed authorized fixes for the two findings in the independent desktop-polish audit of `0536631f733d801ad77f5e5ee10cab2dd543a124` on `AE0101/v1-6-rc`. The owner message's UTC time was unavailable. The work ends at the local commit, verified bundle, and source/test sync.

## Changes

- Full-page and Sticky editors receive the underline, strike and highlight shortcuts before the desktop browser filter. Full-page text alignment uses the same routing for Ctrl+Shift+J and Ctrl+Shift+R. Browser defaults are still canceled on desktop chrome.
- Note Find consumes Escape within its own find bar or editor before Radix can dismiss a centered row. The side peek handles its own dismissal before ProseMirror cancels Escape. A foreground dialog with focus receives its own Escape.
- Chromium regressions cover both editors, both peek modes and focus locations, the foreground dialog, composing Escape in the full-page Find input, and browser cancellation on chrome.

## Evidence and review

The local receipt `v1-6-rc-polish-fix-summary.txt` records commands, counts, retries, file hashes, and cleanup. The bundle contains this candidate and its records. Its prerequisite is the original Step 1 commit above. The designated linked worktree receives the source and test file copies. The branch commit is in the independent scratch repository and its bundle.

Packaged WebView2 interaction, an independent review of this fix, and owner acceptance remain open. [Session 150](../sessions.md#session-150).
