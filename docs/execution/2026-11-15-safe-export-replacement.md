# EX-2026-11-15-83 — Safe export replacement, step c

**Who and where:** Ahmed (AE0101), Gaming-PC; the continuing Root chat/49E objective under direct owner continuation and A57. Root coordinates and runs checks; database_undo_classification authors the bounded product change, and a separate non-author reviewer reviews it. Requested/inherited Sol Ultra; actual serving model/effort and attributable usage remain unknown.

## 1. Goal and authority

Implement the export-replacement candidate already approved in [A57](../plan/approval-and-amendments.md#a57-safety-evaluation-owner-decisions-20261114), in its lettered order. The document replacement is the commit point. A precommit failure must preserve the previous document and its pictures. Report any new plaintext files that could not be removed; postcommit cleanup trouble must not misreport a successfully replaced document as a failed export.

The selected repaired-grader baseline closed at 2026-11-15T09:13:17Z /11:13:17 Cairo, with the evidence and limits in [EX82](2026-11-14-safety-evaluation-baseline.md#browser-baseline-closure-20261115). This is engineering qualification, not full-app or owner acceptance. During this continuation the owner asked what steps b and c mean; Root explained both without treating those questions as a pause or new implementation approval. Owner-message times are unknown.

Starting Git HEAD is b9a841760216cb7b8f1a4f5f38c4ece6300fd640; local test/document changes are preserved. Accepted 1.5.1/95ca8b2 remains distinct from unaccepted 1.6. Root's read-only tag inspection confirmed destructive picture preparation also exists in the released exporter source. The already approved export fix addresses that known line; it is not an authorization for a broader repair campaign or a release.

## 2. Bounded implementation

Write area: src-tauri/src/export/mod.rs and, if useful, one private export helper; relevant existing/new export test bodies; src/lib/types.ts and src/features/notes/ExportDialog.tsx for the cleanup-warning result. Standard execution/session/status/usage records remain separate owners. No new dependency, vault format, note-encryption/storage/locking/migration change, AI action or unrelated UI change is authorized by this brief.

Prepare a fresh generation of pictures, render links to that generation, and finish/sync staging before replacing the document. Keep old referenced assets until commit. Clean up only positively identified owned assets; retain/report unknown or changed files. Preserve existing destination/vault/link refusals. Use existing random IDs, hashing and JSON support if ownership metadata is needed. Serialize cooperating export operations in-process; do not claim atomic protection against unrelated external filesystem writers.

Fix temporary-file ownership in the shared plaintext atomic writer: a failed create-new operation must never remove another writer's file. ZIP rendering/layout and CSV/attachment features are outside expansion scope, although existing shared-writer callers must retain their regression coverage.

## 3. Verification and evidence

At creation of this entry, candidate implementation and test tooling are being prepared. **No step-c test, build, app run, final-source review, integration or acceptance is claimed.** Preserve source preimages and every failed attempt. One deliberately defective review control belongs to this patch review round; it must remain separate from the actual candidate.

Use deterministic private failure injection and throwaway filesystem data. Verify failures at picture write/sync, document write/sync/replacement, temporary-name collision, rollback cleanup, and old-picture cleanup. Compare the previous document and all old picture bytes before/after precommit failures. Verify warning delivery and complete new image links after commit, a no-picture replacement, cooperating concurrent exports, and preservation/refusal of foreign, linked or changed assets. Retain existing applicable format/ZIP checks. Establish the expected named export test set before execution; empty or omitted required selections cannot pass.

Run applicable TypeScript/frontend checks on the final source and bind Rust test artifacts to their exact inputs using isolated homes, outputs, target and qualified offline tools/cache. Preserve previous targets/binaries. These checks do not qualify private-data parity, independent PDF rendering, live clients or the final release candidate by implication.

## 4. Position and continuation

Step c is **IN PROGRESS / unverified**. Root continues within the approved export scope through candidate production, non-author review, meaningful final-source checks and documentation. Any unresolved requirement or ownership limitation stays explicit. There are no real-vault/private comparison inputs, Git mutations/fetches, original/historical-lane changes, scheduler actions or further startup-setting changes in this work.

After step c, prepare the whole no-silent-input-loss design [step d / P16] and wait for Ahmed's required design approval. Key-derivation integration, other explicit A57 owner gates, whole-app release qualification and acceptance remain separate. Do not implement the input-loss design merely because this export candidate is approved.


---

<a id="export-initial-qualification-20261115"></a>

### Initial qualification milestone — fixed cut 2026-11-15T10:00:35.3492877Z /12:00:35.3492877 Cairo

**Who/where/provenance:** Ahmed (AE0101), Gaming-PC; Root coordinates/runs, database_undo_classification authors, browser_diagram_views prepares tooling, ai_core_controller_review independently reviews, and pbs_export_checkpoint records this later assembly. Requested/inherited Sol Ultra; actual model/effort and attributable usage unknown. This is an ongoing step-c checkpoint, not closure, integration or acceptance. [Full 123](../sessions.md#session-123)/[usage 62](../ledger/usage-ledger.md#codex-usage-20261115-block62) preserve chronology and usage.

#### Produced and reviewed source

The initial four-file candidate is pinned in `G:\Notaya Dev folder 2\runs\safety-baseline-20261114T211838Z-4780571e\outputs\export-c-review-20261115T092152Z-7e048759\mandatory-tests-and-source-pins.json` (EC6852D307FCF3D518911474CCAF4065298232B0132616C8C24AD48F49343F1C): mod08439117…, transaction81D52EA4…, typesE93D7262…, dialogB3B56278…. Its preimages/diff remain preserved. Tests use independent byte inventories/link resolution, test-only hooks, commit-order/cleanup/collision/changed-file cases and one real process mutex. CSV and row Save As use other routes; no universal-export claim follows.

The one blinded source control 1EDB7F0EACE1847BBD6F94B7B8B04C02DC0B42CE552EE847B130F587D2641B0D was caught: it validated only the document temp and could publish changed staged pictures. Actual 81D checked all owned files. The control was not installed or run.

Actual review held a P2 in the new helper: unbounded serialized ownership output could exceed its reader's 4 MiB cap. Existing1GiB picture-byte limits permit the source-derived 40,000-picture example (2,720,000 image bytes; 4,549,121 manifest bytes versus4,194,304 cap). First export could succeed and later replacement refuse; no old-file deletion is demonstrated by this mismatch. Root required the same cap checked before staging, with preservation/boundary evidence, not a looser cap/new image-count limit.

By09:59:34Z Root reported corrected transaction52DAAA0809D3431E87569A426AF856D5B975813F45B3E51775C2976085DA8CC3 produced with the cap guard/boundary test and a bounded actual-Windows-junction fixture. The additive `mandatory-tests-and-source-pins-cap-junction-v2.json` hashes E87C5987F8B82A4D8A9411814DC7494023FF32DE5D0068A8186CC19DFF1FED63 and requires 18 Windows cases (prior 16 plus cap/junction). Final delta review and execution are pending. The earlier Unix-only link case is not Windows runtime proof. Root prehashed the future fixture's absolute cmd.exe at 09:56:35Z:15CD209C995B7D3009C91CA40892D5B8F1FB162F3E5F10716642EA278E8E0D7B; no fixture execution yet.

#### Actual frontend result and prepared Rust inputs

Fresh `G:\Notaya Dev folder 2\runs\safety-baseline-20261114T211838Z-4780571e\export-frontend-20261115T094200Z-4fd704d3` ran09:41:18.7725387–09:42:22.7997515Z, exit 0 / no timeout. TypeScript 6.0.3:115 root/680 source files, zero diagnostics. Vite 4420 modules;184 output files/9,009,456bytes. Alias deprecation, ineffective dynamic-import and plugin-timing warnings remain, not an empty-stderr claim. Raw stdout SHA7BF47341D7F2F6F8C732AC040C1B6DD6F71312E901789DA13006833683FDB576; output manifest73bdb31610dd52387d82dfb6f1c14c3cc1f0048544983e335e2f1a056fe0369b; coordinatorA00486BB9E842F168C9C2B55BD81471BEE9ECF1190340F45151DF562267AB8AA.

Root verified all 184 output hashes and a fresh public copy at `D:\Codex Projects\Notaya\resume-20261111T190954Z-ae1f50c8\test-runs\export-frontend-3e598b40`,09:44:56Z. The 274-file source manifest18F2BFE008B653E72C97DDF4947CD57C421743CAFE2FD676119D5D87A5A88730 and 16-case manifestAF144B0143AD109A070F7682E8835B1BAB296568E9DBB798A02339E5172D5A2D were prepared only and are superseded by the held Rust correction. They are not a Rust attempt or pass. Starting/current Root-reported HEAD remains b9a841760216cb7b8f1a4f5f38c4ece6300fd640; working-byte manifests, not HEAD alone, bind qualification.

The isolated controller `c-tooling-20261115T093206Z-4e819c61\run-export-qualification.ps1` is0EE25277BBBB44C663C53CD3E10ED725FC22521BABD479916C7FC78BF887F923, with original/B61 preimages retained. Five separately settled stages cover fresh libtest compile, all/ignored export listings, exact frozen named tests and production-library cargo check. Independent mandatory names must also run. The ignored manual PDF sample is excluded explicitly; unset sample environment cannot produce a counted no-op pass. No step-c Rust compile/test/check has run at this cut.

#### Actual browser setup failure retained

Initial warning grader77BCF4CC555D567D40F8DF5B94916480F601A414D8C462ACC0B97099CF4EFD2E and wrapperC1EBA56870EF3E89C0DD6DC0FA0F615989611D84AED9D5F6584B4B000F46B2A5 passed source review, then actual run `export-browser-faca0b587f9e\export-warning-a61d16cba819\outputs` failed09:53:11.7982396–09:53:47.5355736Z:3failed/4checks, exit 1 / no timeout, zero export calls/module responses. Node API route.fetch attempted CONNECT 127.0.0.1:1421 through the deny-only proxy and received 403; the fixture's own WebSocket closure also errored. This is a new test-setup failure, not an export product finding.

Raw coordinator SHAACBDDDABAA1996B31C5BB5A8F7915887B6B6EB20A92FFAAC77B859B4563CACF8; stdout78E0A815D6207D73FE4A0E09098703C43A40086D92E5056142353AB6F79EC07B; stderrC3C57FE967F802278BA671A84F913CE7403BD2AD486D3330D19404053FE1D9EC. All are retained. The corrected FE63C86EA66C1958BECC07AD373C825CB2963E99521DD5438D416F6D16C2E983 fixture uses bounded direct node:http for only the verified owned Core module, without authentication, redirects, environment proxy or proxy relaxation, and removes forced WebSocket closure. Expectations/tolerances are unchanged. Delta review/fresh retry are pending; no unchanged rerun is claimed.

The wrapper delta is one semantic allowlist entry **plus** the preceding Mermaid LF→CRLF terminator. Its41D134… inverse proof restores both; the earlier “one line only” narrative was incomplete and was corrected without source normalization. Root observed zero standalone test-browser processes09:56:35Z. Mock server 37580/port 1421, created09:51:17.6069373Z and identity-checked09:52:31Z, remains intentionally live for retry, not closed.

#### Position and exact continuation

c remains IN PROGRESS / unverified. Root owns fresh source pins, final non-author delta review, meaningful exact-revision runs and owned-server closure/readback. Then close c and prepare d/P16's whole no-silent-input-loss design; wait for Ahmed's required approval. d–h have not run. Accepted 1.5.1, unaccepted 1.6, N2/P6c/Q1 and all release/private/owner gates are unchanged. No original/historical-lane/private-vault, Git, scheduler, dependency or release action is added.

Root's earlier09:33:38Z three-record/inverse/A57 readback and saved Notion comparison closed the initial brief update (historical hub tail byte-identical; literal bracket escaping normalized only for new-field comparison). New123/62/current-record/hash/Notion readback remains pending after this assembly. This writer read current bounded owners and full 122/61/EX83, not every accumulated historical tail.


---

<a id="export-source-ui-pass-rust-start-20261115"></a>

### Later-received evidence — update 2026-11-15T10:11:11.2919817Z /12:11:11.2919817 Cairo

Ahmed (AE0101), Gaming-PC. This update records outcomes returned after Session 123's evidence was frozen; individual execution times below can precede its cut. Full 123/usage 62 and the earlier failed/held states are unchanged. Root fully read 123/62, the EX83 milestone/current pointers and matched all seven hashes at 10:09:19Z; new pointer/Notion readback remains pending.

**Source gate cleared, not runtime acceptance.** Independent final delta review passed transaction 52DAAA0809D3431E87569A426AF856D5B975813F45B3E51775C2976085DA8CC3 with unchanged mod08439117…, typesE93D7262… and dialogB3B56278…. The writer now checks its existing 4 MiB ownership limit before staging; the actual cap blocker is closed in source. Additive E87C manifest requires 18 Windows cases including boundary/junction coverage. The same single 1EDB control was caught; no second control or owner decision was required.

**Warning-display check passed.** Corrected FE63/C1EB ran 10:00:17.7691084–10:00:24.6577971Z, exit 0/no timeout: four checks (three fresh-page cases plus count). Exact grader original/executed hashes match; all three case/module markers and successful summary are present. Positive DOM vectors were not dumped: source-reviewed fixed assertions, not an independent DOM transcript, support the pass. Independent actual readback was reported PASS; its time is not recorded. This is synthetic browser wiring, not native Tauri transport/disk recovery/rendering/acceptance. Initial 77BC failure remains.

Closed logs: `export-browser-faca0b587f9e\export-warning-73fe597d2ef4\outputs`; coordinator59ECCEF56866D993315A6C7A8A12A02EA0AF78E794A9B8ACADA5592E0AA1FBAE, stdout7EE53A1901D42E156712AACD0ACD1E1D2ECD97F7AC65DFA4EFC5F620A34EFA9F, stderr 0 bytes. Root requested server stop 10:02:15Z; supervisor returned exit 0 and CLOSED 37580/exit 0. Root 10:02:54Z found no 1421 listener or standalone browser. Completed server stdout contains CLOSED; its stderr retains one Vite deprecation warning. Root read completed logs 10:05:18Z; this writer also read them, without inspecting active Rust logs.

**Exact-source Rust qualification started, no result yet.** Root started 10:04:40.3012869Z with source 274 manifest2BC1D293DFA687CF5F298CC5AE62B117C6C95AE4D3C81F62FBDD2EF59A1FAE74 and 18-name manifest5020C96994FBC38E070491B3E8A75341FD24A307A833194CF54387BAC0155D14 in `export-inputs-20261115T100300Z-77a67fe3`. Reviewed five-stage controller0EE and parent PowerShell2A6769A3330DC1084D0154BFDE7D164AB750CA7F700F313F9D417C1BE78B2233 use fresh target `export-qualification-20261115T100500Z-174c725e`; supervisor is `export-supervisor-20261115T100500Z-0289cdb7`. Root's latest supplied state is running; no binary binding, test count/pass, production-check result or closure is claimed. Active logs/source-generated state were not read by this writer.

Step c remains IN PROGRESS/unverified. Root awaits settled final-source results/readback, retaining all failures. Product author may prepare P16 context read-only, not implement it. After c closes, write the whole no-silent-input-loss design [d/P16] and wait for Ahmed. Wider gates, accepted 1.5.1/unaccepted 1.6, protected originals/history/private inputs and all explicit owner waits remain unchanged.


---

<a id="export-parser-stop-20261115"></a>

### Qualification stopped at the tooling parser — later observation 2026-11-15T10:15:18.5716391Z /12:15:18.5716391 Cairo

Ahmed (AE0101), Gaming-PC. Root's fresh actual read at 10:11:25Z established that the Rust qualification had already finished 10:08:58.4466528Z, exit 1. The earlier “running” update preserves information then supplied; this later observation is current. Outer B1 process 38080 settled with exit/log/drain closure true, no timeout/termination/retained resources; closureErrors are empty and unsettled=false.

Fresh compilation succeeded: owned Cargo 47200 started 10:05:59.3516567Z, exited 0/closed, with successful build-finished. Its libtest executable is 51,319,296bytes, SHA256 F8C5FB2ECABB1BC8E72D46CF62B12BD80AB684657B30E72942E3ACB7AF472562, under `export-qualification-20261115T100500Z-174c725e\target\x86_64-pc-windows-msvc\debug\deps\notaya_lib-b1b1676f59df303b.exe`. Compile identity is not a test pass or production-library check.

The list-all stage 35088 started 10:08:33.2166124Z and exited 0/closed. The new controller then rejected valid `commands::export::tests::default_export_excludes_inherited_bin_pages_pictures_and_inline_databases`: libtest's `export::` filter is a substring, while the parser incorrectly admitted only root-prefixed names. Overall qualification is **FAILED**, a new tooling defect, not a product finding. No tests, ignored listing or production check ran.

Closed supervisor logs `export-supervisor-20261115T100500Z-0289cdb7\logs`: stdout52ACCFE5BC05C75FB5453707271247DF03B19D4CB6A1B16BDB4FA9239AC40B7D; stderrEEDF6CB506552096001BCA35E133FC743BAB1CB7F14F6DECEA4F1D65C4CDA84B. Root saved process-result.json; this writer hashed only the now-closed raw logs. The failed attempt/binary/source remain immutable evidence. Root is gating reuse of the exact compiled binary with a corrected parser, fresh homes/logs and a new production-check target; no unchanged retry or recompile is claimed.

c remains IN PROGRESS/unverified; source review/frontend/browser results retain their separate scope. Root owns the tooling correction/review and admitted continuation; no owner decision is needed for this new-tool defect. Full 123/usage 62 and prior dated states remain unchanged. After c, prepare d/P16's whole input-loss design and wait for Ahmed.



---

<a id="export-c-closure-20261115"></a>

### Step c closure — 2026-11-15T10:41:20Z /12:41:20 Cairo

**Who and state:** Ahmed (AE0101), Gaming-PC. Root marks safe export replacement [c] **VERIFIED-SCOPED / COMPLETE** after independent actual readback PASS, with no holding issue. This is engineering qualification of the bounded change, not integration, owner acceptance or release. The current next action is the whole no-silent-input-loss design [d/P16], design only, then Ahmed's required approval. Saving notes, locking and vault storage have not changed.

**Final qualification and identity.** The reviewed recovery controller 0AC… reuses the exact immutable compiled libtest binary through a fresh copy, new homes/logs and a new production-check target in `export-qualification-20261115T103000Z-1a7d9639`. Producer and copied binary both remained 51,319,296bytes/F8C5FB2ECABB1BC8E72D46CF62B12BD80AB684657B30E72942E3ACB7AF472562, with matching before/after metadata. The original 0EE whole attempt remains FAILED and unchanged; this was not a recompile or a relabelled old run.

All four owned stages exited 0 with strict Boolean exit/log/drain closure, no retained/unresolved resource, stop or failure:

- All-test listing:149 names, start 10:30:29.9174547Z; stdoutFAA179CE5FAF2B13D0B16F4DB01D23CD8D476B8F426C45681D21ECCCF4E7C788.
- Ignored listing: only the manual PDF sample, not executed; start 10:30:30.7429999Z, stdoutDAC0CEFAB9226A7FAE220D9ECA2650318046B259F0DF39394F16669090A52615.
- Export execution: start 10:30:31.239619Z; **148 unique named tests passed once, zero failed/ignored**,637 filtered/unrun. All 18 independently required cases passed, including cap, Windows junction, precommit preservation, temporary-file collision, rollback/cleanup warnings and concurrent writers. Runtime 18.74s is not a performance benchmark. Frozen 148-name file hashes837618C7DB5C049B8D17DB71AEC25ACBCE2B1B3F52947F149297FED22DE6CD29; stdoutD3C25F8D6F34FABA9D3D28523C8AF426C9568BF17580BB9B4F29BCBACFB42B61.
- Production library check: start 10:30:50.3900341Z; build-finished success and fresh-target Main library metadata with profile.test=false/fresh=false. stdout28BDA6330BE5A4C7E4A940A9C57DF48E9F2551CF2628BF1CF25B73A81FA8231F; stderrB179477039C2BAE1C436B51AA687461624E2CB334A0364B9CA43085BE8C95C1A. All 23 unused/dead-code warnings are retained, none in transaction.rs; Root read them 10:38:20Z. This is not an empty-Rust-stderr or native executable qualification.

The bounded parent began 10:29:49.1571259Z and closed exit 0. Root's observed tool END was 10:33:32.3550138Z; that END is not a persisted field. Supervisor `export-supervisor-20261115T103000Z-4f25db43` owns saved process-resultF3C3756F9C531971807EDE6A0888A8F9DD49CD2CE8CC90A9E0C1E4CD932ED51D and outer stdout57D286C50F580993CE3F549C7B43EF2CDA7F435FE5BEFB966F7C8BC084F019D3/stderr 0. This writer read the closed parent result and checked its hashes/148-name manifest; Root and the independent reviewer own the detailed actual qualification.

Source 274/namespace2BC1…, all 9 selected tool vectors, public Cargo-cache metadata, frontend73BD…/D asset copy and all 4 schema-exception frames matched before/after. Product084/52DA/E93/B3B,18-name5020… and source-E87C pins matched. Root checked CMD15CD… unchanged 10:35:26Z and no 1421/1423 listener/standalone browser 10:39:10Z. The older frontend zero-diagnostic/build pass and mock-warning three-case-plus-count pass retain their own warnings/limits. Initial 77BC failure, 0EE parser failure and 1EDB control remain preserved.

**What is now supported.** Markdown replacement keeps old referenced pictures until document replacement commits; only verified owned old generations are cleaned afterward. Failed cleanup reports plaintext leftovers. Unclassified or already-changed ownership can refuse the replacement before staging, preserving existing files; recognized legacy files and postcommit cleanup leftovers are retained and reported. Ownership output above the existing 4 MiB cap is refused before staging; no cap was widened. Shared atomic temporary-file collisions preserve unowned files. Warning delivery has scoped browser proof. CSV/row Save As use other direct-write routes; no universal-export-atomicity promise is made.

**Limits and continuation.** No whole-library/native-app, private parity, power-loss, external-writer compare-and-swap, OS sandbox, independent rendering, packaging or release qualification follows. Accepted 1.5.1/unaccepted 1.6 and wider gates remain separate. d is DESIGN ONLY in preparation; e–h NOT RUN. Full 123/usage 62 stay at their earlier fixed cut; Root will request one final full 124/usage 63 for this work block after the reviewed design/owner gate is ready. Root verified the earlier hub update at 10:21:22Z (three fields/literal-bracket normalization, earlier tail byte-identical); A57 remains 2C052…. New closure/pointer/hash/Notion readback is pending.






<a id="export-workblock-handoff-20261115"></a>

### Work-block handoff — 2026-11-15T11:08:07.1667975Z /13:08:07.1667975 Cairo

Root read the scoped closure/current pointers and matched their hashes 10:51:25Z, then read the corrected ownership wording 10:54:30Z. c remains VERIFIED-SCOPED/COMPLETE with all earlier failures/limits intact. [Full 124](../sessions.md#session-124)/[usage 63](../ledger/usage-ledger.md#codex-usage-20261115-block63) preserve the later work block. The next result is [EX84's reviewed input-loss proposal/owner wait](2026-11-15-input-loss-design-review.md), not implemented behavior. Root verified the Notion safety-card progress body 11:04:50Z after inline-code/canonical-mention normalization, preserving its prior properties; hub final update awaits these saved records. No new c test, source or acceptance is claimed. Final record/hash/Notion readback remains pending.
