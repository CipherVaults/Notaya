# Execution Summary: EX-2026-10-24-N1 — Proposal: a native Windows Cipherleaf with no browser engine

(4-section form for a small task. The EX number is to be assigned when merged, to avoid clashing with open PRs #9 and #10.)

## 1. Task Overview

The owner (project thread, 2026-10-24, 08:59:52 to 09:12:34 UTC) asked for a plan so the app does not need WebView2 or Edge, then clarified: a truly native Windows app, everything inside the build, any Windows 10 or 11 at any update level, looking and behaving exactly like the current app. Scope: a plan document only, no app code, read-only research.

## 2. What was done

- Read `CLAUDE.md`, `docs/STATUS.md`, the deep audit, the security design, the Tauri config, `ui.rs`, `model.rs`, the editor, the UI sources (screens, themes, settings, sidebar, slash menu) and the draft live-checks report (PR #10).
- Researched toolkits and licences on the web (Masonry/Parley/Vello, AccessKit, Slint, iced, RaTeX, Tauri CEF, WebView2 Fixed Version).
- Wrote [proposal-2026-10-24-no-webview2.md](../plan/proposal-2026-10-24-no-webview2.md): requirements, a parity inventory with what each area can and cannot match, toolkit comparison with a recommendation (Masonry/Parley/Vello, iced and Qt as fallbacks), the notes-editor rebuild, vault and backend compatibility, Windows 10/11 and self-contained packaging, phases with a go/no-go spike, licences and decisions. It is a proposal, not an amendment.

## 3. Not verified

Nothing was built or run. Effort, size and speed figures, the toolkits' Windows behaviour (IME, bidi, editing), RaTeX's accuracy, accessibility claims and most licences are unverified (section 13 of the proposal). The network finding comes from unmerged draft PR #10.

## 4. Open items

The owner's answers to the decisions in section 12 of the proposal. No approval was given; `docs/plan/approval-and-amendments.md` and `docs/STATUS.md` are unchanged on purpose to avoid conflicts with open PRs; add the amendment and the status row when the owner chooses a direction.
