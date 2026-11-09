# Notaya 1.6 independent security pass (WP-17)

Findings first; no fixes are authorized by this report. Claude reviews the evidence and makes the final security call. Later area reviews may append their own sections below.

## Area 1: Tauri commands and CSP

- **Commit reviewed:** `13185af2c1be96a539ec1ec5446e99e0e14def89`, branch `AE0101/v1-6-s`; clean at entry.
- **Reviewer:** fresh Codex session which did not write the reviewed code; Ahmed Emara (`AE0101`) side. Machine name to be recorded from the runtime.
- **Authority:** Ahmed's Area 1 instruction in this conversation (message timestamp unavailable); A49 / WP-17. Review began 2026-11-09; checkpoint clock: 07:46:15 UTC (system tool).
- **Scope:** every registered Tauri command, action catalogue, CSP/capabilities, frontend active-content sinks and the command paths to vault, files, settings, windows and processes.
- **Write area:** this file and proof-only files under `tools/security-pass/area1/`. No product edits, installs, network, Notaya executable, or Git mutations.
- **Return / stop:** command inventory, proven findings and explicitly unproven questions, evidence limits and self-check; stop before fixes for Claude's review. Shared claims, session and usage records remain Claude's responsibility under this bounded brief.
- **Result state:** review in progress; no claim of acceptance or whole-app security.

Review details follow in this section. Later areas should be appended as new level-two sections.
