# Execution Summary: EX-2026-10-24-N1 — Proposal: run Cipherleaf without WebView2 or Microsoft components

(4-section form for a small task. The EX number is to be assigned when merged, to avoid clashing with open PRs #9 and #10.)

## 1. Task Overview

The owner (project thread message of 2026-10-24 08:59:52 UTC) asked for a plan so the app does not use WebView2 or Microsoft, because some people removed Edge. Scope: a plan document only, no app code, read-only research.

## 2. What was done

- Read `CLAUDE.md`, `docs/STATUS.md`, the deep audit, the security design, the Tauri config and capabilities, `ui.rs`, `model.rs`, the editor extensions and the draft live-checks report (PR #10).
- Researched options on the web (Tauri CEF runtime, WebView2 Fixed Version, Slint licence, Servo, Edge-versus-WebView2 uninstall behaviour).
- Wrote [proposal-2026-10-24-no-webview2.md](../plan/proposal-2026-10-24-no-webview2.md): options O0 to O5 with effort, licence, security, size, portability and vault-compatibility notes, a recommendation with phases, and decisions for the owner. It is a proposal, not an amendment.

## 3. Not verified

Nothing was built or run. Effort figures, the Fixed Version runtime size and terms, Tauri-on-Windows CEF behaviour, Servo's state and most licences are unverified (listed in section 9 of the proposal). The network finding comes from unmerged draft PR #10.

## 4. Open items

The owner's answers to the decisions at the end of the proposal. No approval was given; `docs/plan/approval-and-amendments.md` and `docs/STATUS.md` are unchanged on purpose to avoid conflicts with open PRs; add the amendment and the status row when the owner chooses a direction.
