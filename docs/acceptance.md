# Acceptance ledger: Cipherleaf

This record answers one question: **which approved requirements are satisfied by applicable evidence, and who decided that.** It doesn't hold raw results (those live in the Execution Summaries in [`execution/`](execution/)), the current state ([STATUS.md](STATUS.md)) or the authority history ([approval-and-amendments.md](plan/approval-and-amendments.md)). Created 2026-10-22 (reconciliation, [EX-06](execution/2026-10-23-pbs-reconciliation.md)).

## How to read it

Three independent columns, never merged:

| Column | Values |
|---|---|
| **Outcome** | pass, fail, inconclusive, not-run (what a check found) |
| **Applicability** | current (applies to the source named below), historical (an earlier source; not asserted as current), partly invalidated, unavailable |
| **Owner decision** | pending, accepted, rejected. Only the owner decides. A result in "Outcome" is not acceptance. |

**Target assessed:** the working tree on branch `feat/a9-keepassxc` after commit `2105a57` (amendments A10–A14; only documentation changed afterwards). Last checks on this code: `cargo test --lib` 33 passed (22:50 UTC); `npx tsc --noEmit` 0 errors; `npm run build` passed; `npm audit` 0 vulnerabilities; the calculator engine 66 checks (22:50 UTC); the release build at 23:04 UTC. Any later source change makes these historical. **Update 2026-10-23:** the code changed afterwards (A15, A16 and the lock and data-loss fixes A17, commit `dc95fd3`). On `dc95fd3`: `cargo test --lib` 39 passed, `npx tsc --noEmit` exit 0, `npm run build` passed, the calculator engine checks ALL OK, `npm audit` 0 vulnerabilities (earlier on 2026-10-23); see row X14 and [EX-13](execution/2026-10-23-lock-and-data-loss-fixes.md). The counts in older rows (33 tests) describe the older source.

**Owner decisions recorded so far:** none formal. The owner has not accepted any requirement. One informal signal: at 2026-10-22 18:06:46 UTC the owner sent a screenshot of the running app (Paper theme, the sidebar and a page titled "Hello Cipherleaf") with the single word "happy". It shows the app runs in their vault and that they like what they saw. It is **not** recorded as acceptance, because it didn't name a requirement. Ask the owner to confirm what it covers.

## Effective scope: the plan plus its amendments

The approved plan ([snapshot](plan/approved-plan-2026-10-22.md)) is the contract. Amendments change it as follows (details in [the amendments record](plan/approval-and-amendments.md)):

- **A9** replaced Cipherleaf's own password vault, generator, TOTP, CSV import, breach check and clipboard code with the bundled KeePassXC 2.8.0. So plan criteria about those features now apply to KeePassXC, not to Cipherleaf's code.
- **A10–A12** changed the look (themes, layout), the notes editor (blocks, images, sub-pages, equations), and show KeePassXC inside the window. They add features; they don't remove a plan criterion.
- A1–A8 are unreviewed differences between the plan and the code (see the amendments record).

## Ledger: the plan's Verification section

| ID | Requirement (plan wording, shortened) | Effective under A9–A12 | Evidence | Outcome | Applicability | Owner decision |
|---|---|---|---|---|---|---|
| V1a | Rust tests: encrypt and decrypt round trip | Applies | `crypto::tests::seal_open_round_trip`, `vault::tests::create_reopen_and_read` | pass | current (the whole suite, 39/39, passed again on `dc95fd3`, 2026-10-23) | pending |
| V1b | A wrong password or keyfile fails | Applies | `vault::header::tests::create_and_unlock`, `keyfile_is_required_when_set` | pass | current | pending |
| V1c | A tampered header byte or row fails | Applies | `every_header_byte_is_protected`, `tampered_row_is_rejected`, `crypto::tests::tampering_is_detected` | pass | current | pending |
| V1d | A swapped row is rejected (authenticated data) | Applies | `vault::tests::swapped_rows_are_rejected` | pass | current | pending |
| V1e | Changing the password keeps the data | Applies | `vault::tests::change_password_keeps_data`, `header::tests::rewrap_keeps_data_key_and_vault_id` | pass | current | pending |
| V1f | The generator meets character-class rules and a loose uniformity test | **Superseded by A9** for passwords (KeePassXC's generator). Cipherleaf keeps only the passphrase suggestion. | `generator::tests::passphrase_*`, `crypto::tests::random_below_stays_in_range` | pass for what remains | current | pending (the owner should confirm the criterion is retired) |
| V2 | Plaintext scan: no known note phrase or password in `vault.clf` or backups | Applies to notes (passwords are now in `passwords.kdbx`, KeePassXC's format) | `vault::tests::no_plaintext_on_disk` on a library-made vault. The scan the plan describes (`Select-String` on a real vault and its backups) was **not run**. | partial: test pass, scan not-run | current for the test | pending |
| V3a | Portability: copy the exe to a new folder and run it | Applies, plus the KeePassXC folder | A release exe was built (commit `2105a57`, rebuilt once) and started from a temporary copy, both builds: it created `data\` next to itself and showed its window ([EX-09](execution/2026-10-22-release-build-and-commit.md)). A vault was not created and the Passwords view not opened. | pass (limited) | current for that exe | pending |
| V3b | Nothing new in `%APPDATA%` or `%LOCALAPPDATA%` | Applies | The same smoke test: no folder named like the app, its WebView2 profile or KeePassXC before or after; also seen on the first dev launch. Names only; one run. | pass (limited) | current for the release exe | pending |
| V3c | Pull the USB stick while unlocked: it locks | Applies | Implemented; never tested | not-run | unavailable | pending |
| V4 | Network: no connections with the breach check off | Applies; the breach check was removed (A9), so Cipherleaf has no network code | Release-exe smoke test: 0 non-listening TCP connections over 12 seconds, no debug port. Dev build and KeePassXC alone (8 s) earlier. Not tested through Cipherleaf with KeePassXC open. | pass (limited) | current for the release exe; historical for the rest | pending |
| V5a | Auto-lock on idle, Win+L, sleep; sticky notes vanish on lock | Applies | Locking through the app was observed in a throwaway vault: the UI returned to the lock screen and KeePassXC was hidden. Idle, Win+L, sleep never exercised. (2026-10-23: the Lock button and closing the window were exercised in the real exe after A17; see X14. Idle, Win+L, sleep still not exercised.) | partial | historical (the lock code changed in A17) | pending |
| V5b | Clipboard clears and never appears in Win+V history | **Superseded by A9** (KeePassXC does the copying and clearing) | none | not-run | unavailable | pending |
| V5c | Windows show black in a screenshot | Applies to Cipherleaf windows; KeePassXC blocks capture on its own | The owner turned the setting off and captured the app (so "off" works); at 22:58:30 UTC they said they left it off for now and "it's working". The "on = black" state was not observed in this record. | inconclusive | historical | pending |
| V6a | `cargo audit` | Applies | Not run | not-run | unavailable | pending |
| V6b | `npm audit` | Applies | 0 vulnerabilities at 21:49 UTC, 199 packages | pass | current | pending |
| V6c | `/security-review` on the code | Applies | Not run | not-run | unavailable | pending |

## Ledger: things the amendments added (checks that exist, no plan wording)

| ID | Claim | Evidence | Outcome | Applicability | Owner decision |
|---|---|---|---|---|---|
| X1 | The bundled KeePassXC is the verified 2.8.0 release and starts only if its hash matches (A9) | SHA-256 and PGP checks (EX-02); `keepassxc::tests::refuses_missing_or_unverified_keepassxc` | pass | current | pending |
| X2 | KeePassXC is shown inside the Cipherleaf window and follows it (A10) | Script checks in a throwaway vault: rectangle match, move, resize, minimize/restore, lock hides, detach, and again after the A11 layout change (EX-03, EX-04). Typing and mouse use inside it were not tested. | pass (limited) | historical (before A12 edits; no related change since) | pending |
| X3 | New look and layout (A11) | Screenshots with mock data; the owner's own screenshot of the Paper theme (informal, see above) | pass (visual) | current for what the owner showed | **pending: informal "happy" only** |
| X4 | Block editor, pictures, covers, icons (A11) | Mock-data scripts; real-backend check in a throwaway vault: pasted picture and cover stored, shown, and kept after lock and unlock; `vault::tests::a_page_cover_keeps_its_image_alive` | pass (limited) | historical for the real-backend part | pending |
| X5 | Block-type list, selection toolbar, sub-pages, comments, equations (A12), as reworded and trimmed by A13 | Mock scripts and screenshots; `commands::notes::subpage_tests::pages_nest_and_follow_their_parent`. After A13 the list and toolbar were checked again in the mock; columns, comments, toggle headings and heading 4 are no longer creatable but still display. | pass (limited) | current for the unit test and the A13 mock checks; historical for the earlier UI scripts | pending |
| X6 | Older vault records still open after the format additions (icon, cover, parent) | `vault::tests::pages_saved_before_icons_and_covers_still_load` | pass | current | pending |
| X7 | Equations run with KaTeX 0.19.0 locally with no network (A12) | Dependency pinned; `npm audit` clean; renders in the mock. Not checked in the real app. | partial | current for the audit | pending |
| X8 | The calculator under Sticky notes (basic and scientific) gives correct results and clear errors (A13) | `dev-mock/calc-check.cjs`: 66 cases, all pass. Mock-UI scripts: typing, Enter, history, scientific keys, divide-by-zero message. Not checked: Copy and history-clears-on-lock in the real app. | pass (engine); partial (UI) | current for the engine; mock only for the UI | pending |
| X9 | Reword and trim: the editor no longer offers the trimmed features and uses new wording, without dropping content from older pages (A13) | Mock-UI scripts and screenshots; an older mock page with columns, a toggle heading, a comment and a heading 4 still displays. Not checked in the real app or with a real older vault. | pass (limited) | current for the mock | pending: the owner decides whether the new wording is different enough |
| X10 | A small calculator inside each sticky note opens from a button, calculates, and returns to the note (A14) | Mock UI as a sticky window (330×350 and 320×320): open, type, live preview, Enter, Esc to clear and to close. Not checked in a real sticky window or with real clicks. | pass (limited) | current for the mock | pending |
| X11 | A release build exists and runs (first build) | Commit `2105a57`; `npm run tauri build` 4 m 32 s; exe SHA-256 `D237C5B5D9346323EA2C7C5B2B0E1585FF7BCB226F551C90233AF91CD39B4183` (rebuilt at about 08:56 UTC on 2026-10-23 from commit `dc95fd3`, with A15 to A17; ZIP SHA-256 `A6BD40E62621760F40FFE7BA0D968D3D2D9BF556A16BACFD44B6426254E2C382`; the portable folder holds the verified KeePassXC (hash equal, also checked in the ZIP) with its licenses and source). **Superseded exe hashes:** `F4F1E679725021B41F222E852B1CC25BE021FF57C4826640F4A3E4D3FF654800` (23:49 UTC on 2026-10-22, commit `3a22c56`; this is the one in the ZIP the owner uploaded to their Drive), `6129922C…`, `5D766105…` and `A10AAB35…`. The earlier smoke test (as for V3a/V3b) was made on the superseded `F4F1E679…` exe, not repeated on the new one. The real-exe lock test (X14) was made on a build of the committed source; that it was the very file with hash `D237C5B5…` is not recorded. Not tested: Passwords view, USB. | pass (limited) | current for the exe `D237C5B5…`; a source change makes it historical | pending |
| X13 | Note pages are wider (Normal 1040 px, Wide 1320 px, new Full 1680 px) and Paper is the default look, slightly refined (A16) | In the mock UI at 1920×1080 the default look is Paper and a note's text column is about 1050 px wide (it was 780 px at Normal); `npx tsc --noEmit`, `npm run build` and `cargo test --lib` (33 tests) pass. Not checked: the real window, the Full width, the lock screen in Paper, a vault that already has a saved look. | pass (mock, checks); not-run (real window) | current for commit `3a22c56` and the later code; included in the portable builds `F4F1E679…` and `D237C5B5…` (not re-checked in the mock after A17) | pending: the owner's eyes on the real window |
| X12 | The app uses the Cipherleaf icon instead of the Tauri icon (A15) | The embedded icon of the rebuilt exe was extracted and shown to be the Cipherleaf leaf; the first rebuild still carried the Tauri icon until the cached build output was cleaned. Not checked: the icon in the real taskbar and the window title bar on a screen. | pass (exe icon); not-run (taskbar) | historical: checked on the exe with hash `F4F1E679…` (after the A16 rebuild); not re-checked on `D237C5B5…`; Windows may cache the old icon | pending: the owner's eyes on the taskbar |
| X14 | Text typed just before Lock or close is saved first; the lock order is safer; a failed commit is rolled back; KeePassXC is locked and not re-shown wrongly; a failed page save is kept; sticky delete needs a second click; the calculator +/- negates the whole expression (A17) | On commit `dc95fd3`: `cargo test --lib` 39 passed (6 new); `cargo clippy --all-targets` only 3 old warnings; `npx tsc --noEmit` exit 0; `npm run build` passed; `dev-mock/calc-check.cjs` ALL OK; `npm audit` 0 vulnerabilities (earlier 2026-10-23). **Real exe on a throwaway vault** (debug port closed afterwards): type then Lock at once kept the text for a page (pass) and for a sticky note typed into while Lock was clicked in the main window (pass); closing the main window right after typing kept it (pass). The previous exe (commit `3a22c56`) lost the text in both cases (fail), so the test does detect the problem. **Not run:** idle lock, Win+L, sleep, drive removal, the KeePassXC lock and re-show paths, the `quit_app` command path, the sticky delete confirm and the calculator +/- in a real window. Known limit: a save that hangs is dropped after 2 seconds. ([EX-13](execution/2026-10-23-lock-and-data-loss-fixes.md)) | pass (unit tests, checks and the three real-exe cases); not-run (the untested items) | current for source `dc95fd3`; the test ran the exe `D237C5B5…`, the same file as in the portable folder | pending: the owner tries the new exe |

## Open decisions that block acceptance

- **A2** (replay of a single older record isn't detected) and **A8** (Cargo.toml uses ranges, Cargo.lock pins) need the owner's decision; A1 and A3–A7 need evidence. See the amendments record.
- Retire or keep criteria V1f and V5b (superseded by A9).
- Which requirements the owner's hands-on review covers.

## Updating this ledger

Add or change a row only when new evidence arrives, a source it applies to changes, or the owner decides. When the source changes, mark affected rows historical and re-run only the checks that apply. Never write "accepted" without the owner's actual decision and its date.
