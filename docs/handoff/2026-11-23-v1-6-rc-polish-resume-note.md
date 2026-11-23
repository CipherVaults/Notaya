# Notaya 1.6 RC desktop polish

Ahmed (AE0101) · Gaming-PC · 2026-11-23 UTC

Ahmed's Step 1 request authorized desktop polish, version alignment, isolated verification and a local commit on `AE0101/v1-6-rc` from `94f48841eacab0c2e61fd4d52b651eb1a9e9f63a`. The owner message's UTC timestamp was unavailable. The stop point is the local candidate and verification receipt. Publication follows the independent review in Step 3.

## What changed

- Main, sticky and calculator windows share WebView2 settings. The application keeps text-editing context menus available and cancels menu events on its chrome.
- Ctrl+F opens Find on the focused note or database, or Search in the unlocked shell elsewhere. Note Find searches body text. Desktop and production builds cancel browser shortcuts and mouse gestures that would surface browser controls.
- Window chrome stays unselectable while editors, table cells and code retain text selection. Explicit application drag handles retain their behavior.
- Frontend, Tauri and Rust package metadata agree on 1.6.0. The bridge already had that version. Both lockfiles contain only local version changes and the three frozen JSONL outputs match `cbf0193f` byte for byte.

## Evidence and review

The Gaming-PC receipt `v1-6-rc-polish-summary.txt` records the exact candidate, commands, counts, retries and cleanup. The new Playwright check exercises unmodified Vite dev, simulated Tauri dev and a production mock bundle. The Windows probe used the actual window builders with hidden windows and disposable profiles. It checked the native settings in both debug-assertion modes. It did not exercise the packaged release or manually select a native context menu item.

The author self-check covered the locked COM interface IDs and layout, the release-only DevTools assignment, editable menu policy, input handling, and lockfile integrity. Find keeps queries and matches in editor memory. It adds no IPC, storage or log paths. The independent security and product review remains the Step 3 review.

The designated checkout stores its Git metadata in the protected sibling. The receipt distinguishes the candidate commit from the live branch readback and records how to resume. [Session 149](../sessions.md#session-149).
