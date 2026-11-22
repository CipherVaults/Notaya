# SF3: offline tamper evidence and permanent-delete content purge (EX-2026-11-09-72)

## 1. Scope, authority and source

Side: Ahmed Emara (`AE0101`). Machine: `GAMING-PC` (read from the execution
environment). Codex is the developer hand; Claude owns independent review,
records integration, commits and pushes. Result: produced, with the named
verification results below; not integrated or accepted.

The user's SF3 brief authorizes S2-M1, S2-M2 and S2-M3 and specifies the anchor,
purge contract and tests. It cites Claude's security call of 2026-11-09 08:31 UTC;
the user message's own time is not available. The review used `1c129af`; this run
re-read the current source. Branch: `AE0101/v1-6-sf3`. Entry baseline:
`4934a04b35f9c0ef32d88cf85cf6861d0bd851d1`, including automatic SF3 checkpoints
`bbc01dd` and `4934a04` on base `d910034`. Their work was preserved and reviewed.
The hand-back includes those checkpoints and the subsequent working-tree diff.
The external heartbeat made checkpoint `e7bbe74` at 18:02:38 UTC during this
continuation; Codex issued no Git mutation command. Its later edits still need
Claude's final commit and exact-commit checks.

Read: AGENTS, partner briefing, the entire security-review guide, security
design, WP-02 through WP-05, current activity/history/store/session code, existing
tamper tests, and both Area 2 proof sources under lane S (read only, not copied
into the build). The repository PBS skill was used. This lane's claim is the
user-assigned branch/write area; shared claim/session/ledger files remain for
Claude to reconcile. No fetch/PR lookup/network, Git mutation, install, app,
browser or real-vault access was performed. Throwaway data and TEMP/TMP are under
`D:\Notaya Tests\v1-6-SF3`; build outputs use `D:\Notaya-lanes\_target\SF3`.

## 2. Findings and resulting behavior

| Finding | Before | After |
|---|---|---|
| S2-M1 | Erasing/dropping all activity could look like an untouched legacy log and start a fresh chain. | The independent anchor commits the expected head and row count; absence/mismatch fails unlock or verification and restore staging. |
| S2-M2 | Removing both rows of a snapshot reduced both the index and the verifier's expected count. | A separate authenticated membership ledger defines the expected set. Missing pairs, one page's entire history, a table or either half fail membership checks. Bodies still load lazily at unlock. |
| S2-M3 | Sticky text, row cells and database templates could remain in encrypted activity previous values after permanent deletion. | Hash-bound encrypted undo bodies are purgeable. Permanent deletion also purges old inline values, history and replacement edits, leaving content-free undo tombstones and a valid re-linked chain. Undo refuses the purged change set. |

## 3. Anchor and deterministic construction

- `journal_anchor`, closed `Table::Anchor`, AAD tag `b"anc"`. AAD otherwise
  remains the existing format binding (`notaya` for NOTA v2), little-endian
  version, vault id, table tag and 16-byte row id. All rows use the existing
  padded XChaCha20-Poly1305 seal with a fresh OS-random nonce.
- Fixed anchor id: sixteen zero bytes. Fields: activity `Head` (`last_seq`,
  `last_hash`, `last_id`, `first_seq`, `base_hash`); total `activity_rows`
  (entries plus the head when present); membership `tail`, `root`, `certificates`
  and live `members` count.
- Membership certificates have OS-random ids with first byte 2. A certificate
  contains `previous`, `previous_hash`, monotonically increasing `sequence` and
  `changes`. Changes are sorted by snapshot version id with duplicates refused.
  An addition binds that id to `{ page, body_hash }`; a deletion has null binding.
  The id names metadata row `00 || suffix` and body row `01 || suffix`.
- Root construction is `SHA-256("notaya/history-membership/v1\0" || canonical
  JSON(certificate))`. The previous root is part of that canonical JSON. Objects
  are sorted recursively, array order is retained; the starting root is 64 zero
  hex digits and the starting tail is empty. This reuses the existing SHA-256
  linked-ledger pattern rather than introducing a commutative accumulator.
- Replay the authenticated certificate chain backwards by sequence/hash, then
  apply its sorted deltas forwards. Duplicate additions and missing removals
  fail. Compare the resulting exact id/page/body-hash map to freshly read
  authenticated metadata, compare counts, and compare both halves' ids to all
  history row ids. A count derived from surviving metadata alone is insufficient.

## 4. Write, check and migration boundaries

The journal, snapshots, certificate and anchor are written in the same
`Store::transaction` as the corresponding item write. Explicit snapshot writes,
history thinning, activity markers, prune, clear, template removal and
`remove_many` all update the anchor in their own data transaction. After a purge
the final head/anchor refresh is in that same transaction too. In-memory heads,
indexes and anchor are accepted only after success; the outer AI action rollback
snapshot includes the anchor. Re-key/format upgrade and copy verification include
the new table.

Unlock checks the authenticated activity head/count, certificate chain and exact
history metadata/pair membership. It never opens a history body for an already
anchored vault. `activity_verify` (the existing Check the log command) additionally
walks the entire activity chain, authenticates/hashes all history bodies and checks
undo-body hashes and exact references. Restore reaches that same verifier through
`verify_attachments` on its staged copy.

The settings field `journalsAnchored` is an internal authenticated migration
latch. A genuinely old file without an anchor/latch is fully verified (including
legacy bodies/undo), then receives anchor+latch atomically and silently. A set
latch with an absent anchor/table fails with a plain error; mismatch fails as
damaged/tampered. New vaults set the latch with their first settings/journal write.
Every later settings write preserves it. No schema/format version bump is used.
Migration cannot recover knowledge of evidence erased before the first anchor.
Whole-file rollback and coherent replay of an older authenticated journal set
with matching anchor/latch have no offline freshness guarantee.

## 5. Purge boundary and deletion-path coverage

Previous values are stored in `anc` undo-body rows with random ids prefixed by 3;
their SHA-256 hash and reference are covered by the activity entry hash. Page/row
page text remains in version history; non-page values include sticky bodies, row
cells and database templates. Full verification checks every body/reference and
rejects missing, swapped, altered or unreferenced undo-body rows.

`remove_many` deletes records, both history halves and relevant replacement
edits, deletes matching indirect bodies and clears matching legacy inline undo.
Affected `Previous` values retain the item id and a `purged` flag, with content,
references and before/after hashes removed. Activity entries retain their existing
metadata/ids and are re-sealed only when their hash changes; the affected suffix
is re-linked and the anchor refreshed. A selected change set with any purged
member is refused before undo writes anything.

Covered paths (no native-confirmation code was edited):

| Path | How the purge is reached |
|---|---|
| Page/sticky delete for good | `delete_page_core` -> `delete_bin_tree` -> `remove_many`. |
| Empty bin | `empty_trash_core` visits binned roots through `delete_bin_tree`. |
| Notebook/section/page descendants | Tree deletion includes contained databases, rows and nested pages. |
| Database and row delete from bin | `delete_database`/`delete_rows` -> `remove_many`, including row subpages. |
| Single/bulk sticky bin | Binning preserves undo; a later permanent delete or empty-bin purges it. |
| Template removal with database retained | `put_many` compares old/new template ids, removes matching replacement copies, purges database undo values containing those templates, refreshes the chain/anchor and collects unused files before commit. This covers the existing database-save core too. |
| Bin-related file/picture GC | `remove_many` collects unused files in the purge transaction using the prospective items/settings and excluding deleted history; existing callers' later `collect_garbage` remains safe. GC follows the just-written undo head and preserves pictures in new snapshots not yet in the RAM index. Ordinary binning and surviving references preserve files. |
| Clear/prune | Removes indirect bodies belonging to removed log entries and writes the authenticated clear/prune marker and anchor in one transaction. |

There is no separate timed bin-expiry/GC deletion path in the current source;
`collect_garbage` itself removes attachments, not items. All actual item removal
call sites converge on `remove_many`. Independently copied live content, retained
search/calculator values, older backups/exports, host memory and physical disk
remanence are outside this item-owned current-record purge promise.

## 6. Regression tests

`vault/security_tests.rs` covers offline activity erase/drop, tail truncation,
middle deletion, replacement, swapping and old-head replay; offline history pair,
page/table and single-half deletion and pair swaps; deleted/dropped/mismatched
anchors; pre-C3 and pre-SF3 migration; thinning and typed clear; failed anchor
write and failed purge/final refresh rollback; missing undo bodies; all-record
random-marker scans after delete/empty-bin for page/sticky/row-cells/templates;
template removal via the real save core, including undo-only files; a mixed save
that removes a template while keeping files needed by new history/undo; file-GC
failure rolling back the complete purge; notebook/section cascades with row
subpages and undo-only attachments; bulk sticky bin/undo/empty; and G3-style old
reader tolerance. Attacks occur after dropping the session, use SQL without a key,
and check restore rejection. Purge scans decrypt every current record in all five
data tables and never print the random marker or credentials.

The activity tests also cover real legacy inline undo (without a latch field or
anchor table), preserved old serialization/hash fields, and purge after silent
migration. Existing individual-row tamper tests remain intact. History fixture
helpers that directly insert snapshots now write matching membership certificates
transactionally; the existing permanent-delete undo assertion now expects the
specified explicit refusal. The 1,000-version test still proves lazy body reads
and small metadata; re-key/security-switch/upgrade/backup regressions cover the
added table.

Final-source focused result: `cargo test --lib --offline vault::`: **93 passed,
0 failed, 0 ignored, 645 filtered out**, 44.19 s, exit 0. Log:
`D:\Notaya-lanes\out\SF3-vault-final-4.log`. This includes 15 new security tests
and the added legacy-inline activity regression. The full-suite result is in
section 7.

## 7. Required checks and source identity

Final Rust command: `cargo test --lib --offline` with the SF3 target directory.
Full-suite result: **732 passed, 1 failed, 5 ignored, 0 measured, 0 filtered out**,
185.34 s, exit 101. The sole failure is the known sandbox-only
`shell::tests::the_registry_value_can_be_written_read_and_removed` at
`src/shell.rs:339`: Windows refused to open the start-up list (code 5).
The long rollup regression and all SF3 tests passed. Log:
`D:\Notaya-lanes\out\SF3-full-tests-final-5.log`.
The five existing ignored tests are three Windows named-pipe PC checks (sandbox
error 5), the one-time old-engine golden capture, and PDF sample generation.
Compiler warnings (six): unused `id` in the existing activity re-key failure
test; unused `claim_core`, `release_core`, `title_prop`, `CALCS` and
`Control::with_deadline`. No warning was suppressed and no ignored test was
enabled or disabled by this lane.
The source is `e7bbe74` plus the final working-tree Rust diff, with seven changed
Rust-file hashes in `D:\Notaya-lanes\out\SF3-final-source-sha256.txt` (SHA-256 of
that manifest: `a961d939aeb2d4bb0c09801e98db0c0eb7c004afffd34ea989353d728d6d209e`).
The manifest was captured before the final full-suite run; all seven hashes were
rechecked after it and were unchanged. Source results must be reattached to
Claude's eventual final commit.
TypeScript/build are not run because no front-end code changed. `npm audit` is
explicitly skipped per the brief. `cargo audit`, clippy, app/browser tests and
installs are not run. `git diff --check` exits 0; Git's LF-to-CRLF advisory is not
a whitespace error. The final diff was checked again after completing this note.

Earlier attempts are retained under `D:\Notaya-lanes\out`: the shared target
hit LNK1104 because another test process held its exe; the existing lane-specific
SF3 target avoids that collision. Two full runs were stopped by exact SF3 test
process id when their source was superseded during GC work; neither is a final
pass. The first GC build exposed a lifetime error, fixed before the successful
focused run. Those attempts are not substituted for the final result.

## 8. Performance

The anchor has a fixed number of scalar/hash/id fields. A normal save appends
one membership certificate containing only that save's additions (normally at
most two versions per edited page) and writes the anchor: O(1) extra work per
item/snapshot, no history or activity rescan for root calculation. An activity-only
write needs no certificate. Batches cost O(their changes), with sorting confined
to that batch. Maintenance can scan existing metadata; permanent deletion can
scan/re-link the retained activity chain, as required to remove legacy inline
content. Those costs are intentionally separate from ordinary saves.

Certificates are content-free but retained: storage and unlock verification grow
with the number of historical membership-change batches, including thinned
versions. Unlock/explicit verification replay certificates and read metadata/ids;
full verification also opens bodies. No latency benchmark or real-vault measurement
is claimed. Old backups remain independent files.

## 9. Security self-check (guide section 4; not independent)

- Secrets/keys: existing vault AEAD/AAD and SHA-256 only; keys stay in Rust;
  nonces and ids use existing OS randomness. New plaintext undo temporaries use
  zeroizing wrappers and existing zeroizing item/entry types. No new credential
  comparison, key export, log or event. Existing lock/zeroization path unchanged.
- Commands/IPC: no new commands, action names exposed to callers, confirmation
  plumbing or permissions. Native confirmations and pairing: untouched, n/a.
  Undo keeps its existing owner/AI checks and refuses purged content before writes.
- Files/paths/parsing: no new external path or archive/image parser. SQL values
  are parameterized; table identifiers come only from the closed enum. Row ids
  use the existing exact 16-byte parser; metadata/certificate/body mismatches fail.
  Parsing resource budgets outside these existing encrypted-record paths: untouched,
  n/a. Plain messages do not include note text or credentials.
- AI pipe/bridge, ACLs, pairing, frame bounds and AI event text: untouched, n/a.
- Storage/history/logs: checked head/count/root/membership, lazy bodies, table
  erasure, unchanged old-entry hash encoding, exact undo-body references, all
  data/anchor transaction boundaries, nested rollback, secure-delete, copy and
  re-key coverage. Permitted metadata remains; purge re-linking is an explicit
  authorized operation. No decrypted disk cache is introduced.
- Front end, CSP, HTML rendering, localStorage, capture and auto-lock: untouched,
  n/a.
- Dependencies/build: no dependency/lockfile/package/tool addition or version
  change. npm audit skipped as instructed; cargo audit not run. Diff checked for
  secrets and vault artifacts; fixtures generate throwaway credentials/markers.

Self-check found no additional proven issue in scope. This is the implementer's
check, not the independent Codex/Claude review or Claude's Security Guidance run.
During the self-check, template-only file cleanup was found missing from the
database-save path. It was fixed inside the purge transaction; normal item
deletion now also collects its files before committing. Marker scans and injected
GC failures cover that correction, and mixed-write tests protect retained files.

## 10. Files and changes outside the write area

Implementation: `src-tauri/src/vault/{activity,anchor,history,mod,store}.rs`.
Regression additions: `src-tauri/src/vault/security_tests.rs`, plus tests next to
activity/history. Documentation: this note and `docs/security-design.md`.

Outside write area, mechanical: `src-tauri/src/model.rs` adds only the internal
authenticated `journals_anchored` bool and its false legacy default (three lines).
No `commands/notes.rs` or `commands/ai.rs` edit. No authority/session/ledger file
was changed; Claude owns their final records.

## 11. What remains for review and a person

Claude must review the whole diff and independent security results, run its
required Security Guidance check, integrate records/usage, commit and push. A
post-commit run must attach results to that exact commit; these results apply to
the final working source, not an as-yet nonexistent final commit. Actual 1.5.1 exe
read-only opening of a copy, UI wording/feel, native confirmations, real restore,
backup/lock timing, crash/power-loss hardware behavior and release acceptance are
not exercised here. The G3 test verifies the old read pattern, not the old binary.

Usage: this interface provides no `/status` or trustworthy model/effort/token/quota
receipt; those values are not measured here. Claude/heartbeat must record the
actual receipts; no estimates are substituted. Initial orientation time in this
continuation was read as 2026-11-09 17:54:07 UTC; that is not the earlier SF3 run's
start time.

## 12. Addition for Ahmed and stop point

**'Check the log' now fails on tampering it accepted before; nothing else visible.**

Stop at this hand-back. No commit, push, merge, release, app run or acceptance by
Codex. No new dependency or visible-design decision was made.

## 13. Claude's review (2026-11-09, cloud thread, about 20:10 UTC by the system clock)

Read the whole diff: `vault/anchor.rs` (new), `activity.rs`, `history.rs`, `store.rs`, `mod.rs`, `model.rs`, `security_tests.rs` and the `docs/security-design.md` additions. Checks the PC heartbeat ran on the lane tip `c3f7968` (clean tree equal to the pushed commit): `cargo test --lib` 733 passed, 0 failed, 5 ignored; `npm audit` 0 vulnerabilities; `tsc` and `npm run build` not run (no frontend file in the diff). Result state: **verified** on `c3f7968`.

**The merge with `main` needed a second repair, found by the PC check.** The first merged commit (`557f57d`) did not compile its tests: S-FIX2 (already on `main`) added an approval argument to `delete_page_core` and `empty_trash_core`, and SF3's tests still called the old forms (7 call sites, test code only). The PC session switched them to S-FIX2's test helpers (`test_delete_page_core`, `test_empty_trash_core`); one test, the injected-failure rollback test, builds its approval explicitly and asserts that the error is not the cancelled-approval error, so it still proves the purge failed and rolled back. No production code changed. Result on `f2d32ea` (clean tree, throwaway home, Gaming-PC, 2026-11-09 about 20:50 UTC): `cargo test --lib` 755 passed, 0 failed, 15 ignored (`main` alone is 739); `tsc`, `npm run build` and `npm audit` green on `557f57d` (the repair touches no frontend or package file). Result state: **verified** on `f2d32ea`, then **integrated** by this merge.

The first conflict (`vault/mod.rs`, `Session::open`) was resolved by keeping both fields (`native_session_id` from main and `anchor` from SF3).

Flags for Ahmed: (1) this adds to the vault format (a new table and a `journalsAnchored` settings latch; no version bump; older vaults migrate silently once, and the G3 safety copy is planned but not yet built); (2) permanent delete re-links the activity chain and runs a verification on each call, so deleting a very large selection in a large vault may be slow; to be measured in the whole-app pass; (3) the visible effect is only that "Check the log" now fails on tampering it used to accept.


<a id="g3-w198-protected-input-20261122"></a>

## Native protected compatibility precheck, 2026-11-22 14:44 Cairo

**Exact input and build.** The Windows x64 unbundled v1.6 release-profile app matched GitHub main `e3f7fa2da4889a257f1e26d50b6fe68eb15153c3`, tree `40db056a061f8b57eef4f5f504309d0cce672bb5`, exe SHA-256 `42d36778f3e8e7fa0baf60d2c7b6cd7420c9f5594a22f404915c3afe64079c78`. The candidate source was `33d02033051e46199bf4eb642fe11afa1c5ea2b8` at the same tree. A separate published v1.5.1 executable was pinned before the run, with tag `95ca8b203403fae701e6f60521419329b30a7145` and exe SHA-256 `be7426e9d6b5e1e81ca02779dde7cee42ee676dc006fccd64ae3907ed1ec5aca`.

**Boundary and result.** A bounded launch received a closed copy of the previously generated W194 password-only synthetic fixture. The home, portable data directory and WebView profile were isolated. The current application displayed the expected password-only lock screen. The tool could read its accessibility tree and enter a fixture value through the form control. Its normal submit target had unavailable coordinate geometry, and keyboard focus stayed on the document after Raise and Tab. No unlock submission or saved-content assertion ran. The test ended before launching the released reader. The environment result neither supports nor refutes protected compatibility.

**Shutdown and preservation.** The test window remained after its targeted close attempt. Cleanup verified the owned executable path, PID and start time before terminating only that process tree. The runner recorded exit and drained logs with a nonzero exit. The closed copy and input both retained SHA-256 `10c490a65288586d05b3ad4592a7e4cb8bdcfa77d4b61a927a4b6fa8ae0ff3c1`. No SQLite sidecars remained by the copy. Vaults, profiles and raw process receipts stay on the PC.

**Cutoff.** Keep the capture protection. A future native check needs ordinary input at the isolated password screen, exact saved-content checks and inspection of the closed file after the released reader's separate visit. First-upgrade copy and notice (A44), the database redo (Q1) and exact packages retain their own gates.

<a id="g3-w198-native-readback-20261122"></a>

### Full three-object native compatibility readback — 2026-11-22 10:50 Cairo / 08:50 UTC

**Identity and fixture scope.** Verified GitHub main `bfc7b33842be298f346117f45a13bbae93aec2c6`, tree `5474f85501fc69c7fa73218a8e68f50c64cc9f38`. Windows x64 unbundled v1.6 development executable SHA-256 `cd32e84f1f1742833750138b1bf90c4d82a9c513292e6ba79ddb1683fe4065d5`. Actual published v1.5.1 reader at tag commit `95ca8b203403fae701e6f60521419329b30a7145`, executable SHA-256 `be7426e9d6b5e1e81ca02779dde7cee42ee676dc006fccd64ae3907ed1ec5aca`, original distributed ZIP SHA-256 `4576d5039f59708d219f9e365ff9903e5470a7a0471f324839959c811a3a28e7`. Both used fresh disposable portable homes and distinct WebView profiles.

**Fixture creation.** Started from a closed synthetic security-off note with a heading, bullet, marked text and small image. In the current build added a short sticky, closed its window, and made a plain table with one title property, one Number property and a single row. The row name and numeric value `1729` were visible. Closed the current app before copying. The closed fixture passed SQLite quick-check and contained seven ordinary records, one attachment, four history rows, 14 activity rows and three anchor rows. No SQLite sidecar remained.

**Published app readback.** The released binary received a separate exact copy. The page body rendered, the closed sticky's board preview showed its full text, and the table showed the expected row and numeric value together. No edits or sticky-window actions were taken. The reader exited normally. Both the original and the visited copy matched the closed seed SHA-256 `17ca6005e31ba4f4db31a0e3d6b0562d4d614bbbfc4da08cb37ec5c52a4f47df`. The released executable did not see the original path.

**Return to v1.6.** Reopened the original with the exact current build, read all three objects, and closed normally. Both closed files passed read-only SQLite quick-check. The v1.6 app's later navigation persisted one change to the reserved settings record. The schema, header, attachment and all four history rows matched exactly. No table IDs disappeared. One activity row was updated under its original key, one was added, and one anchor row was updated. The first post-reopen oracle required every activity ciphertext to stay byte-identical even through current app navigation and failed. The retained row-ID comparison and the exact old-visit hash locate all observed changes after the v1.6 reopen. No evidence from that later change is attributed to the old reader.

**Gate boundary.** Creating the first database displayed the existing one-time database-format backup notice for releases through V1.4.0. This happened on database creation, after opening the fixture. It gives no credit to the separate first-v1.6-unlock safety copy (A44), which retains its earlier failed result. G3/W198 here covers the named open-vault shapes and read-only navigation. Password-protected old-app compatibility, editing in v1.5.1 and the final packages remain outside this run. Both executables, fixture vaults, profiles and raw evidence stayed on the PC.

**First-unlock repair boundary, source review only.** Both automatic and password unlock call the same load routine. After it authenticates the records, 'src-tauri/src/vault/mod.rs:249' creates the history, activity and journal-anchor tables before the UI can show an unlock notice. The first-database backup runs later and cannot cover this. Put the existing-vault safety copy after authentication and before these schema writes, sharing the path between automatic and password unlock. Use a verified SQLite snapshot that accepts the earlier schema. The current backup verifier requires the newer tables. Make copy promotion and retry idempotent, and keep restore staging and fresh vault creation outside the new path. The November 8 owner decision (A44) approves the one-time local copy and notice. The later owner rule requires approval for changes to migration behavior. Pending decision: when the copy cannot be written or verified, keep the vault closed, leave its existing schema and notes intact, and show a retry. This is the recommended failure behavior. No migration change was made.

<a id="native-vault-compatibility-20261122"></a>

### Windows vault compatibility and first unlock — 2026-11-22 07:36 Cairo / 05:36 UTC

**Windows identity.** Released v1.5.1 tag `95ca8b203403fae701e6f60521419329b30a7145`, portable ZIP SHA-256 `4576d5039f59708d219f9e365ff9903e5470a7a0471f324839959c811a3a28e7`, exe SHA-256 `be7426e9d6b5e1e81ca02779dde7cee42ee676dc006fccd64ae3907ed1ec5aca`. Windows x64 unbundled release-profile v1.6 development app `6c5e38cad80c76b5325b083c8d26a0123a0a4f15`, tree `f747b2e3f83a38788e75efe25a8e3ae53c1bae39`, exe SHA-256 `05572537bf6a8404ebcefb39b321f4035919fe83d6895dc588e5cf098410fc22`. Distinct portable homes, security-off synthetic data and no debug endpoint.

**Released app readback (G3), passed for this fixture.** The published v1.5.1 app opened a copy of the v1.6 vault containing activity, history and journal-anchor tables. The title, body, two lists, table and picture appeared in the editor. No edits were made. The original and visited copy remained byte-identical. Both passed SQLite quick-check. A redundant schema and row comparison initially failed because the checker assumed each table had a row ID. The corrected comparison found identical rows in all six tables. This does not certify safe editing in the old app. The specification's warnings about old-app saves and visibility remain in force.

**First-unlock safety copy (A44), failed for automatic unlock.** Copied a closed, locally generated released vault with header, records and attachments tables. Its older source stayed unchanged. The v1.6 candidate opened the copy and showed the saved synthetic note on Home. No first-unlock backup notice was visible. After shutdown, one ordinary backup existed. It already contained the new journal tables, as did the upgraded live copy. There was no separate pre-upgrade copy. A second unlock and password-protected path were not run. Both remain part of verification after a fix.

**Design proposal awaiting Ahmed.** On both existing-vault paths, verify a dedicated local copy after successful authentication and before journal initialization. Preserve the vault's current protection, use the existing checked-copy pattern with a legacy-schema verifier, keep the copy out of automatic rotation and show the one-time notice after successful unlock. If copy verification or storage fails, recommend leaving the old vault untouched and offering a retry. Fresh vaults, failed credentials and previously upgraded vaults need explicit coverage. No storage change was made. Both inputs, copies, backups and raw records stay local.
