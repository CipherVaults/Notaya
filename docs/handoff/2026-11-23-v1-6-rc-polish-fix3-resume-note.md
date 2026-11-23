# Notaya 1.6 RC audit #3 Escape fix

Ahmed (AE0101) · Gaming-PC · 2026-11-23 UTC

Ahmed's message at 2026-11-23T14:33:35.767Z authorized Audit #3 Finding 1 on the branch `AE0101/v1-6-rc` from `70061b4df52043587ca87069b9f258d74acefa44`. The endpoint is one local commit, a verified incremental bundle and a fast-forward of the named checkout. Publication stays held.

## Changes

- Escape closes Find when focus belongs to its page editor, including the row title. The title keeps focus so the next Escape closes either kind of peek.
- Property inputs inside the note and focused popup controls get their own Escape handling.
- The Chromium regression exercises both title sequences and property cancellation with Find open.

## Evidence and next step

The local `v1-6-rc-polish-fix3-summary.txt` records the exact commit, commands, counts and hashes. The bundle requires the audited baseline. Use these for independent review and separate packaged WebView2 qualification. [Session 152](../sessions.md#session-152).
