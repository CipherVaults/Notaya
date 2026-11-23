# Notaya 1.6 RC audit #4 Escape fix

Ahmed (AE0101) · Gaming-PC · 2026-11-23 UTC

Ahmed's message at 2026-11-23T14:56:17.328Z authorized the blocking Escape-order fix on `AE0101/v1-6-rc` from `bc4ac5ed43097afb915e5a7ddc4a6656180f6396`. The endpoint is one local commit, a verified incremental bundle and a fast-forward of the named checkout. Publication stays held.

## Changes

- Note Find checks for open popup content in its document even when a trigger or the editor still has focus.
- Block options handle Escape before RowPeek. The nested Change type popover keeps Radix dismissal, and the editor takes focus when Escape removes the focused block controls.
- The custom slash menu uses the same Escape priority and keeps its existing dismissal state.
- The Chromium regression opens Find alongside each surface and exercises Escape in full-page, side-peek and centered-peek editors.

## Evidence and next step

The local `v1-6-rc-polish-fix4-summary.txt` contains the tested source identity, check counts, file hashes and incremental bundle verification. The bundle requires the audited baseline. The corrected candidate still needs independent review and packaged WebView2 QA. [Session 153](../sessions.md#session-153).
