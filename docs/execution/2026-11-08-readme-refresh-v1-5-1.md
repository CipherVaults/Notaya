# EX-2026-11-08-63: README refresh for Notaya 1.5.1 and the GitHub About text

**Who and where (A38):** Ahmed's side (`AE0101`); Claude cloud thread "Update Notaya's GitHub README and About" (machine: cloud container), model `claude-sonnet-5-5`. Short form (4 sections). Times are UTC: message stamps and the container clock.

## 1. Task Overview

- **Baseline:** `origin/main` `5efc5d4` (merge of PR #97).
- **Asked by:** Ahmed, project chat 2026-11-08 16:59:07 UTC ("can you also update the about / readme of the GitHub on Notaya to be up to date with it's current state?"). Correction 16:59:43: "at least with v1.5.1, not the future middle of development state". 17:00:18 and 17:00:48: use the Notaya Manual artifact (https://claude.ai/artifact/K6HUfVdCJbuSFNCkiiCu3K; its source is `docs/manual/notaya-manual-first-edition.html`) for better descriptions and possibly images. The coordinator's first note asked to mention 1.6 as in development; Ahmed's correction overrides it, so **1.6 is not mentioned** in the README or the About text.
- **Scope and write area:** `README.md`; the proposed About text and topics (below, section 2; GitHub repository settings are Mohamed's click only, so they are handed over as text); the records of this task. The licence is untouched (both partners must decide).
- **Result state:** *produced*. Not integrated, not accepted.

## 2. Work Performed

1. Read `CLAUDE.md`, STATUS, the claims board, the release notes of 1.3.0 to 1.5.1, the manual, the 1.6 spec (to know what to leave out) and the live repository (`private: true`, description and topics read through the GitHub API; latest release v1.5.1 with the setup and the portable ZIP).
2. **Found stale in the README:** it still said "Version 1.0.1", and had no databases, import/export, Notion import, Arabic and right-to-left text, page icons and covers, Mermaid diagrams, themes (Sunrise), tray, error log, back and forward, the 2 GB import limits, or the owner's acceptance; it listed the 1.0 upgrade steps as if current.
3. **Rewrote the README** for released 1.5.1 only: current-release box, features grouped as Notes, Databases, Import and export, Sticky notes and calculators, Looks and Windows, Security, Backups, Install or portable; How to use it; Updating from an older version; Troubleshooting; Is it safe?; a new "Known limits in 1.5.1" list (taken from the manual and release notes); Credits naming both partners; Requirements. The security text keeps its honest caveats (WebView2 traffic, unsigned exe, exports not encrypted, nothing defended against malware). The icon (`assets/notaya-icon.svg`) heads the page.
4. **Images:** the repository holds no screenshots of the app. The manual's one picture is a drawn window layout, not a screenshot. Screenshots for a 1.5.1 README need the mock screen (`npm run mock`) run on the `v1.5.1` tag (not on `main`, which now carries unreleased 1.6 work), and that needs `npm install`, which needs Ahmed's own word. **Not done; offered as a follow-up.** The lane A screenshots on the PC are from 1.6 builds and were not used.
5. **About text (for the repository settings; the current text is outdated by the passwords-era wording and by the databases):**
   - **Description (one line):** `Notaya: a local-only notes app for Windows. Notebooks, full databases, sticky notes, calculators, Arabic and right-to-left text, import and export, optional password lock, automatic backups. Installer or portable ZIP. Built with Tauri 2 and Rust.`
   - **Topics:** `notes-app`, `windows`, `desktop-app`, `offline-first`, `local-first`, `privacy`, `encryption`, `tauri`, `rust`, `react`, `typescript`, `databases`, `markdown`, `sticky-notes`, `rtl`.
   - **Website:** leave empty (the repository is private). **Not set by Claude:** GitHub repository settings are Mohamed's click only (his account owns the repository); Ahmed or Mohamed pastes the text.

## 3. Verification and What Was Not Run

- **Checked:** each feature claim against the 1.5.1 release notes, the manual (itself read from the code at `9474511`) and the 1.3.0 and 1.4.0 notes; the Noto font licence line in `THIRD-PARTY-NOTICES.md` for the PDF fonts; links to files that exist (`docs/manual/README.md`, the manual HTML, `docs/release-notes/v1.5.1.md`, `docs/acceptance.md`, `docs/security-design.md`, `THIRD-PARTY-NOTICES.md`). The README was read back after editing.
- **Not run:** the app (this task does not change code); no feature was re-tested in the real app. **Not checked:** how GitHub renders the page. The statement "accepted by Mohamed on 2026-11-05" comes from STATUS (X27).

## 4. Open Points and Next Step

- **Ahmed or Mohamed:** paste the About description and topics into the repository settings (Mohamed's click).
- **Optional follow-up:** real screenshots of 1.5.1 for the README (needs Ahmed's word to install the dependencies, and a build of the `v1.5.1` tag with the mock screen).
- **The README must be revised at each release** (1.6 and later) from that release's notes, not from work in progress.

## Addendum (2026-11-08, about 17:25 UTC): real screenshots added

Ahmed said yes to screenshots (17:04:11 UTC, answer to Q56), asked that his Notaya be closed during the capture (he closed it, 17:07:12) and that only Notaya's window be shown (17:11:33). A Remote Control session on Ahmed's Gaming-PC downloaded the released `Notaya-V1.5.1-portable.zip` (SHA-256 `4576D503...A28E7`, equal to the published hash; `Notaya.exe` reports 1.5.1), ran it as a throwaway copy on a fresh vault with security off inside `D:\Notaya Tests`, drove it through a WebView2 debug port, and took ten 1400 by 900 captures of made-up demo content between about 17:08 and 17:17 UTC. It did not touch `D:\Notaya`, the Codex lanes or Ahmed's own Notaya; afterwards the port was closed, no Notaya process was left and `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` was unset. Not captured: lock and password screens, dark themes in use, Gallery, Calendar, Timeline and List views, the installer, floating sticky windows. Picture 06 is a composite of two captures (the calculator is a separate window). This thread looked at all ten pictures before publishing: only Notaya's window, no taskbar, no real name or path. They are in `docs/manual/screenshots/` with `SOURCES.md`, and the README has a Screenshots section. Test folders `v151-dl`, `v151-portable`, `v151-run`, `v151-src` and `shots-clone` under `D:\Notaya Tests` are left for Ahmed to delete. The "No screenshots" point in section 2 and the follow-up in section 4 are closed by this addendum.

