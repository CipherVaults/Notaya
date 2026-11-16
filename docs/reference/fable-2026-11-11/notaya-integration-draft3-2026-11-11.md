# Notaya 1.6 integration plan, draft 3 (decision-ready): Studies 1a and 1b built into the unfinished 1.6

**Draft 3. Planning only; not an approved implementation.** Nothing here is authorised, built, run, tested, copied or accepted. It consolidates draft 2 with Ahmed's answers, the new source boundary, Blue's six clarifications and Ahmed's direction on history. Drafts 1 and 2 and both studies stay unchanged beside this file.

<!-- CHECKPOINT:BEGIN -->
## CHECKPOINT
- Status: DRAFT 3 DECISION-READY (planning only; not approved, not accepted)
- Last finished: sections 0 to 17 consolidated; changed and new parts reread, unchanged parts carried from draft 2 as reread there
- Next unit: none until Ahmed's decisions or Blue's next review. Study 2 NOT started
- Time: 2026-11-11T10:19:23Z
- Baseline: local repository 9977e72 (no fetch). Branch tips as confirmed by Blue on 11 November
- Inputs: Study 1a, Study 1b, drafts 1 and 2, Blue's reviews of 10:01 and 10:13 UTC with Ahmed's relayed words
- Usage: last display given by Blue (10:01 UTC): weekly Fable 46%, all models 29%. No meter inside the run: later values UNKNOWN
- Resume line: "Continue draft 3 of the 1.6 integration plan from the CHECKPOINT in /mnt/project-files/notaya/plans/2026-11-11-v1-6-integration-plan-draft-3.md"
<!-- CHECKPOINT:END -->

Side: Ahmed (AE0101), relayed by Blue. Machine: cloud, read-only. Authority for this draft: Blue's message of 2026-11-11 10:13:22 UTC (thread time stamp). Ahmed's own words inside it carry no recorded time of their own.

**Tags.** [1a n], [1b n]: section of a finished study, where file-and-line evidence is. [VERIFIED]: read in this run at 9977e72; says what the text shows, not that anything ran. [REPORTED]: a project record. [BLUE]: supplied by Blue today, not checked by me. [PROPOSAL]: this plan. [derived]: reasoned from read code. [UNKNOWN], [NOT RUN]: as they say.

**Ids.** MI (minimum invariant), C (1.6 minimum safe core item), U (optional uplift), IP (integration package), H (hazard), RG (release gate), D (decision), OD (open design point), HR (history retention). Draft 1's F numbers are mapped in section 17.

---

## 0. Summary, and what changed from draft 2

### 0.1 The plan in one page

1. **Source stays untouched.** `D:\Notaya` and every lane folder are read-only from now on. Later work happens only in a verified copy in a separate working root, made on Ahmed's word (section 14).
2. **Two tracks.** The *door track* fixes how changes are checked and needs no storage decision. The *format track* makes storage able to grow and sits behind a build switch until release (section 6).
3. **Nine minimum invariants** (section 2) define what must hold for the doors 1.6 already has. Where one cannot be met globally, the limit is written down.
4. **The AI door** keeps its whole-item undo data and gets three targeted restrictions, subject to Ahmed (section 4).
5. **History keeps everything by default**, as Ahmed directed, with limits a person can set. This has real costs beyond disk, set out in section 13, and one scope question only he can answer.
6. **Nothing is adopted silently.** Section 9 separates planning from approval; section 16 is the compact decision bundle.

### 0.2 Changes from draft 2

| # | Topic | Draft 2 | Draft 3 |
|---|---|---|---|
| 1 | Open sequence | Feature list and anchor checked before the vault's age was known | Branches first on the authenticated format and known legacy layout. A legitimate old vault is never refused for lacking newer structures; nothing is created before consent (7.2) |
| 2 | Receive state | Two commit kinds | Still two kinds, but one transaction whenever both change; a state-only commit is conditional on the item being unchanged (5.5) |
| 3 | Baselines | "Established when equal" | Only at a stable same revision, with compatible reader and form versions, tagged with the receive. Source stamps prove sameness only; a difference is "possibly changed" (5.3, 5.5) |
| 4 | Unknown fields on copy | Not copied | Three declared behaviours: reset, carry, block. Never silent loss. Unknown values kept losslessly (7.3) |
| 5 | Cut, crash, paste | "Nothing lost" | Honest loss boundary stated; an optional extra version before a block removal (2.3, U9) |
| 6 | Refused edit | Kept in the window | Stated as not durable: lost at close, lock or crash; the stored page is the last accepted version (3.2) |
| 7 | AI tag uniqueness | Enforced in an "unassigned" namespace | Not enforced on existing tags. Inventory, ambiguous result, explicit mapping by the person; uniqueness only inside an explicit source set (5.4) |
| 8 | Later versions | Assigned to 1.7, 1.8, 1.9 | Table of existing roadmap commitment against proposed placement (section 15) |
| 9 | Real-notes question | Treated conservatively | Answered by Ahmed: tests used some real notes, nothing he fears losing. Still not permission to discard; path M-b stays a real path |
| 10 | Vault size | Unknown | A Notion export a little over 1 GB. That is export size, not vault size or working memory; sizes are measured on synthetic vaults (13.3, RG-G) |
| 11 | Capturing dirty lanes | A commit or copy in place | No write to the source at all. Captured by the copy handoff's manifest (section 14) |
| 12 | History | Thinning unchanged | Default without age or size limit, configurable; the old 90-day, 365-day and 500 MB rules are not carried over silently (section 13) |

The disagreements recorded in draft 2 section 0.2 stand as accepted for planning by Blue; the owner still decides the restricted actions, the batch question and retention.

---

## 1. Facts this draft stands on

### 1.1 States

Accepted: V1.5.1 on `95ca8b2`. Integrated and not accepted: `main` at `9977e72`. Development paused (A56). One 1.6 release is the approved intent (A44); this plan proposes no intermediate public release.

### 1.2 Lanes to preserve, with today's worktree facts [BLUE]

| Lane | Tip | Worktree | Preservation rule |
|---|---|---|---|
| N2 | `7806d00` | clean | Exact-tip checks and a fresh review before any merge |
| RV-N2 | `d8e0fcb` | clean | Note merges after N2 |
| P6c | `ddb52d9` | clean | Final browser run still owed |
| RV-P6b | `3b1756e` | clean | Q1 focus question unproven |
| K1, K1a, K1b | `ebd0141`, `3fc8d32`, `9141770` | clean | Compare, do not pick one silently |
| **K1c** | `4ed8257`, local only at `D:/Notaya-lanes/K1C`, unpushed | **Five untracked files**: `docs/handoff/dev-kit.md`, `tools/dev-kit/{CHANGELOG,INSTALL,README,USAGE}.md` | The commit is not the whole lane. Untracked files are captured before anything else touches that folder |
| **S** | `3a2581f` | **Untracked `tools/security-pass/area3/`** | Evidence from the blocked attempt. Captured as it is; not quoted into records before someone reviews what it contains |
| **W** | `1ae2c04` | **Modified `docs/plan/2026-11-09-whole-app-test-results.md`** (nine observation reports) | The modified sheet is the current result, not the committed one |
| R1 | `397b870` | clean | WIP records |
| ledger-readings | `23fe829` | clean | Merge rows without duplicates |

**Rule for all of them:** no reset, clean, forced checkout, rebase or re-creation of a worktree. A committed tip is not treated as a complete snapshot where the table says otherwise. They are not captured in place: committing or copying inside the source would be a write to it. They are carried by the copy handoff's manifest (section 14, IP-0b), on Ahmed's word.

### 1.3 Existing contracts that bind the plan

| Contract | Text | Consequence here |
|---|---|---|
| G3 (A44) | 1.5.1 is expected to still open a 1.6 vault; one-time safety copy at first 1.6 unlock; confirmed 00:46:05 UTC on 8 November [VERIFIED: spec section 3; A44 row] | A format raise changes an approved gate: amendment needed. The backup wording question (1a C5) is narrowed: A44 records G3 as confirmed; whether the exact wording was confirmed is still [UNKNOWN] |
| WP-06h | Optional `source: {app, id}` on AI creates and updates in 1.6; file-route tags and re-import without duplicates in 1.9 [VERIFIED] | AI source tags are in scope; file-route origins are not |
| A53 | Six arrangement changes; "no crypto, storage format or permission change" [VERIFIED] | N3 may redirect links and fold rows; it may not add stored origins on that authority |
| WP-06e | `batch.run` up to 500 actions as one change set; the bridge lists a `notaya_batch` tool [VERIFIED: spec WP-06d, WP-06e] | Code refuses it [1b K1]. Unresolved release contract (D8) |
| WP-04 | History: before and after versions for AI writes; thinning 7 days, hourly to 30, daily to a year, AI before-versions 90 days, newest 20 kept, 500 MB cap; "delete for good really deletes, history included" [VERIFIED] | Any change to thinning or any new kept record is a retention decision |
| A51 | Native dialog for delete for ever, empty bin, AI on, new pairing; everyday confirmations stay in the window [VERIFIED] | Adding a native dialog elsewhere is a visible security-behaviour change needing Ahmed's word |
| Version plan section 8 | Vault-format gate placed at 1.8 [VERIFIED] | Moving it to 1.6 is a roadmap change: amendment |

### 1.4 New code facts read for this draft [VERIFIED at 9977e72]

- Removing a property changes the database definition only (`database/saves.rs:167-184`); no non-test code in that file writes row cells. So the values stay in the rows under the removed id. Whether a later row save drops them was not read [UNKNOWN].
- `db.property.change_kind` clones the old property with the new kind, removes the old and adds the clone, so the **same property id** returns with another kind (`ai/dispatch.rs`, the `change_kind` arm). The old values are still in the rows and would be read as the new kind [derived, NOT RUN]. What a person then sees is [UNKNOWN]. Study 1b's line "which discards the old values" was too simple; this is a correction to 1b's reading, recorded here and not in the 1b file.
- The activity log, with its undo data, is pruned after 365 days or beyond 100,000 entries (`vault/activity.rs:799-807`); history thinning and this pruning run from one maintenance step at most once a day (`vault/activity.rs:240-252`). Where in the open sequence that step can first run was not read [UNKNOWN]; it matters for 7.2.

### 1.5 Ahmed's answers and the new boundary [BLUE, relaying Ahmed]

- **Real notes.** 1.6 was used only in testing; some real notes were used, none he fears losing. Read as: not proof that no real data ever met a 1.6 build, and not permission to discard anything.
- **Size.** His largest Notion export is a little over 1 GB and will grow. This is the size of an export, not of a Notaya vault and not of memory in use. No time, disk or memory figure is inferred from it.
- **Boundary.** `D:\Notaya` is the existing source. Future development uses a copy and must not modify the source. A local Codex thread is being prepared; no copying or development is authorised now. Schedulers are not restarted and no automation is carried over switched on.
- **History.** Default without limit, with age and size limits a person can set, so a change from a year ago can be recovered where there is room. A planning direction, not implementation authority.

---

## 2. The minimum that must hold in 1.6

### 2.1 Nine invariants for the doors 1.6 already has

Doors: editor, database screen, file import, AI action, automation, history restore, undo, bin, delete for good. "Today" is from the studies unless tagged.

| # | Invariant | Today | 1.6 minimum | Limit that remains, stated in the release |
|---|---|---|---|---|
| MI1 | **All or nothing per logical change**: content, history, undo data, activity entry and anchor together | True for AI actions and import. False for the person's delete for good (several transactions) | Person's purge runs in the same atomic wrapper as the AI path | Automations fired by a person's change remain separate writes; a crash between leaves the row changed and the automation not run |
| MI2 | **No write in a state that forbids it**: locked, upgrading, vault refused as newer | Locked: true. The other two do not exist | Part of the open sequence in 7.2, including maintenance and clean-up | None intended; proven only by RG tests |
| MI3 | **Nothing overwrites what it has not seen, except the person**: session-proof token; an AI's whole-item replace needs the current token; the person's stale save lands and the replaced value is kept | Counter restarts at unlock; token optional | Both fixed. Applies to `page.write`, `db.row.write_page`, `sticky.write`, whole-definition updates, `history.restore`, `activity.undo` | Block and cell actions stay token-optional: they name what they touch |
| MI4 | **Every stored document is schema-valid, or its only unknown parts were already stored for that item** | AI door only | All doors that store a document: editor, AI, import, restore, row duplicate, templates (section 3) | Unknown parts cannot be created or altered in 1.6, only kept, moved or deleted |
| MI5 | **A block id is never knowingly given to two documents** | Unique inside one document; duplicated by cross-page paste and row duplicate | Targeted rules in 2.3, no index needed | Not a proof of global uniqueness. Duplicates made by earlier development builds are counted, not repaired |
| MI6 | **A source tag never silently names two items** | No check; lookup returns a list; tag written by a second write | Tag written inside the action's own write. Existing tags keep today's meaning; a lookup that finds more than one says so plainly. Uniqueness is enforced only inside an explicit source set (5.4) | Twins that already exist stay until the person maps them |
| MI7 | **Delete for good removes everything stored about the item** that 1.6 itself writes | Item, history, undo bodies, replaced-edit copies, unused files; log keeps the title by recorded decision (SF3) | Same list, atomic; plus any 1.6 addition (receive state record, recovery versions) | The title in the log, unless Ahmed takes CR2 |
| MI8 | **Undo and restore obey the same rules as a fresh change**: hidden notebooks, leases, locked databases, shape, relation targets that still exist | Partly; not read for every rule [UNKNOWN] | Verified by test per rule; fixed where a test fails. A relation cell put back by undo is filtered against existing targets | Whole-item skip stays (section 4) |
| MI9 | **No door changes stored text on the way through**; cuts are reported | AI door by rule and test; importer cuts silently [1a] | Importer cuts reported | "Exact" means equal values, not equal bytes (7.1) |

### 2.2 What this costs, in kind and not in lines

| Invariant | Risk it closes | New risk it brings | Review burden |
|---|---|---|---|
| MI1 | Half-deleted trees | Low: reuses an existing wrapper | Low |
| MI2 | A refused or upgrading vault being modified | Medium: the open sequence is security code | High: needs its own review |
| MI3 | Silent loss of a person's saved edits to an AI | AI clients must read first; the bridge and help text change; old proofs and harness scripts must be re-run | Medium |
| MI4 | Two doors with two ideas of a valid page; an AI unable to read a page a person can | **Highest: the typing path.** A wrong rule blocks saving | High, with the three gates of 3.3 |
| MI5 | An AI addressing the wrong block | Low in 1.6, because nothing links to blocks yet [VERIFIED: spec 6.4 "groundwork only"] | Medium |
| MI6 | Twins on refresh | Low | Low |
| MI7 | Something private outliving its item | Low | Medium |
| MI8 | Undo or restore as a way round the rules | Low | Medium: many small cases |
| MI9 | Quietly inexact receives | Low | Low |

Optional future stages, not needed for the invariants above: one footprint rule for all doors, person doors with action names, rules written on the computed difference, global reference rules, the receive stages.

### 2.3 Block ids without an owner index

1.6 has no stored list of which document owns which block id, and building one is a study 2 cost question. So global promises are not made. Targeted rules that need no index:

| Door | Rule in 1.6 | Why it is safe to say |
|---|---|---|
| Paste from another document (copy or cut) | Pasted blocks always get new ids | The editor cannot tell a cut from a copy reliably across two saves. Nothing links to a block in 1.6, so nothing breaks |
| Paste inside the same document | As today: duplicates inside one document are re-issued [1b 3.1] | Exists |
| Row duplicate, template applied | Always new ids | Copies by definition |
| AI `page.move_blocks` across pages | Ids kept; a collision is refused (exists) | One atomic change set sees both documents |
| AI sends content with ids | An id may be kept only if it is in the current stored version of that same document; otherwise re-issued | Checked against one document |
| History restore | An id in the restored body is kept only if the live version of that same item has it now; every other block gets a new id | The live document provably owns those ids; nothing else can be shown |

**Not promised:** that a block a person moves to another page keeps its id; that links follow a moved block. Both wait for an owner index or a move protocol, which must exist before block links ship (1.8 in the version plan). **Acceptance gate:** RG-B in section 10.

**What a crash between a cut and a paste can lose.** New ids have nothing to do with it. After the cut, page A is saved without the blocks; they exist only on the Windows clipboard until pasted. If the app or the PC stops before the paste, the blocks can be brought back from page A's history only as they were in its last version. A person's versions are taken at most every 10 minutes and when leaving a page, and the version taken on leaving is taken after the save, so it already lacks the cut blocks [VERIFIED: history.rs:210-222 and the comment there]. So up to about 10 minutes of edits to those blocks can be lost, as in any crash; 1.5.1 had no history at all. Uplift U9 narrows this: a version is taken just before a person's save removes whole blocks, at most once a minute. It adds versions and so storage (section 13). The test expectation is this boundary, not "nothing lost". **Legacy:** only vaults written by 1.6 development builds can already hold cross-document duplicates (1.5.1 has no block ids). The upgrade pass can count them while it reads every item; it does not rewrite pages.

---

## 3. The editor door: an enforced contract that keeps old and unknown content

### 3.1 The rule

When the editor saves a document, the core compares it with the schema and with what is already stored for that item.

| Case | What the document contains | Result |
|---|---|---|
| A | Only schema-valid content | Stored |
| B | Valid content plus unknown or invalid parts, each **equal in value** to a part of the item's current stored version, or of an authenticated history version of that item | Stored. Those parts are carried: kept, moved or deleted, never created or altered |
| C | A new or altered part the schema refuses | **Not applied.** The live item is unchanged (see 3.2) |

"Equal in value" means equal as data with nothing lost: text compared as stored, numbers compared as written, not after rounding through floating point. It does not mean equal bytes. Legacy pages therefore stay editable everywhere except inside their odd parts. An AI reading such a page gets a placeholder with a block id for each carried part, instead of today's refusal to read the page at all [1b G1].

Provenance for case B is narrow on purpose: the item's own current version and its own authenticated history. A recovery copy from case C is never provenance, because it holds unchecked input.

### 3.2 Two different refusals

| | Invalid edit (case C) | No-change refusal |
|---|---|---|
| When | The vault is writable, the document is not acceptable | The vault may not be written: locked, upgrading, refused as newer |
| What is written | **Level 1** (no stored change, independent of the format decision): nothing; the editor keeps the text in its window and says which part could not be saved. **Level 2** (needs the format work and a retention decision, D7): the attempted document is kept as a history entry marked "not applied" | Nothing at all. No recovery write, ever |
| What the person sees | "This page could not be saved because of one part. Your text is still here." The page stays open; closing asks first | The state's own message |

Level 1 is the 1.6 minimum. Level 2 is uplift U1.

**Level 1 is not durable recovery.** The refused text lives only in the open window. It is lost if the person closes the page after the warning, if the app crashes, and at lock: an automatic lock does not wait, because locking on time is a security promise. The stored page stays the last accepted version throughout. After an unlock the person is told that an edit to that page was discarded; that notice holds the page id only. If Ahmed wants refused text to survive a lock or crash, that is level 2 and a retention decision.

### 3.3 Three gates before enforcement is switched on

1. **Equivalence.** A generated set of every node and mark the editor can produce, in every attribute combination the extensions allow, passes the core schema. This is a standing test, re-run when either side changes.
2. **Producers.** Everything else that writes documents produces valid ones on invented fixtures: the importers at the N2 tip and later N3, templates, row duplicate, history restore. This is the semantic link between this work and N2 that file lists do not show.
3. **Stored pages.** A read-only count of pages whose stored version has parts the schema refuses: on fixture vaults; on a real vault only with Ahmed's word, numbers only. Any non-zero class is either explained and covered by case B, or it blocks.

**Stop conditions.** If gate 1 or 2 fails, enforcement is not switched on and the work goes back to the schema or the producer. If the gates are not met when everything else is ready, that is decision D9: hold 1.6, or release with G1 open and stated in the release notes. It is not carried to 1.8 without that decision.

### 3.4 The editor must not lose what the core keeps

Case B only helps if the editor hands carried parts back. What the editor does today when it loads a block type it does not know is [UNKNOWN]; if it drops it, an ordinary save would delete it and the rule would allow that deletion. So one of these is part of the minimum, chosen after a short trial (a run, so it needs Ahmed's go):

- the editor holds each carried part in a plain placeholder block that it returns unchanged ("This part cannot be edited in this version. It is kept as it is."), or
- a page whose stored version has carried parts opens for viewing only in 1.6, with that reason shown.

The second is the conservative fallback and needs no new editor node.

**Cost not known:** checking a document of up to 240,000 characters on every 500 ms save [NOT RUN]. If it is too slow, the check runs on the changed blocks only; that is a design change that needs its own review.

---

## 4. What an AI can change in 1.6, and the way back for each

### 4.1 The map

Grouped from the spec's action tables [VERIFIED: spec 6.1 to 6.3] and the studies' reading of the code. It is a map by group; a check per action is gate RG-A. "Undo" means `activity.undo` from the whole-item before-image. Two limits apply to every undo and are not repeated in each row: **(i)** undo skips a whole item if the person changed that item afterwards [1b 8.2]; **(ii)** undo data is gone after 365 days, beyond 100,000 log entries, when the log is cleared, or when the item is deleted for good [VERIFIED: activity.rs:799-807; 1b 8.5].

| Group | Asked? | Way back | Limit specific to the group | Proposal for 1.6 |
|---|---|---|---|---|
| Page body: `page.write`, append, insert, replace, delete and move blocks, `block.*` | None; "ask if big" counts items, so one page never asks | "Before AI" history version (always kept under section 13; today protected 90 days) and undo | After (i), history is still there. Good coverage | Core: MI3. Uplift U2: `page.write` over an existing page asks when it removes more than a set number of blocks |
| Page facts: rename, look, favourite, move, reorder | None | Undo | (i) | None |
| Row cells and row body: `db.row.set`, `write_page`, create, duplicate, reorder | None or ask if big | Undo (cells included in the before-image); body also in history | **After (i) the cells have no way back**: history holds no cells | Core: state the limit. Uplift U3: when undo skips a row, the person can open "what it was before" from the activity entry (the data is already stored) |
| `db.property.trash` | Ask | Undo of the definition. Values stay in the rows [VERIFIED: 1.4] | The definition is one item that many small actions touch, so (i) will often apply; then the property cannot be put back by undo although its values still exist | **Core: either** a tested promise that undo restores property and values while (i) does not apply, with the question saying so, **or** U4 (removed list). Decision D5 |
| `db.property.change_kind` | Ask | Undo of the definition | Same id returns with another kind; old values read as the new kind [derived, NOT RUN]; result [UNKNOWN] | **Core: withheld from AIs in 1.6** until its behaviour is tested and described. A restriction of an approved action: needs Ahmed's word (D5) |
| Views, templates: add, update, trash | None, ask, or ask if big | Undo of the definition | (i) as above. They hold layout, not the person's data | State the limit |
| Automations: create, update, pause, trash | Ask if big for create and update, so **a single new automation is not asked about** | Undo of the definition. Changes the automation later makes after a person's edit are not AI changes and have **no undo** | Standing power created without a question | **Core: create and update always ask**, and the question states trigger and actions in plain words. A change to an approved confirmation rule: Ahmed's word (D5) |
| `db.button.press` | As its actions | As its actions | Not read [UNKNOWN] | Gate test |
| `db.file.remove` | Ask | Undo; whether the stored file is still there to put back is suggested by a test name [VERIFIED: name only, activity.rs:1168], body not read | [UNKNOWN] | Gate test |
| To the bin: notebook, section, page, database, row, sticky | Ask | Restore from the bin | Until the bin is emptied | None |
| `bin.restore` | None | Bin again | None | None |
| `history.restore` | Ask | A version is taken first | Restored body is not checked today | Core: MI4, MI5, MI3 |
| `activity.undo` (own change sets) | Ask if big | Whether an undo can itself be undone was not read [UNKNOWN] | | Gate test; state the answer |
| Stickies: write, style, place | None | Undo only (no history for stickies) | (i) | Core: MI3 |
| `calc.clear` | Ask | Not read [UNKNOWN] | | Gate test |
| `settings.set` (Full) | None | Undo; never security or AI fields | | None |
| `bin.delete_forever`, `bin.empty` (Full) | Strong, then the Windows dialog with count and titles | **None** | Relation links from other rows are cut without being named | Uplift U6: the dialog adds "also removes N links from other rows" |
| `batch.run` | n/a | n/a | Refused in code | D8 |

### 4.2 What a person must be told to give an informed yes

| Needed in 1.6 (without it the yes is uninformed) | Already there | Enhancement for later |
|---|---|---|
| Which items, by title and count | Yes, in "Ask" dialogs | Per-field and per-cell counts |
| That a step cannot be taken back | Yes, for the two Full actions, natively | The class of every change in the dialog |
| What a new or changed automation will do | **No** | |
| What happens to the values when a property is removed, and how long it can be taken back | **No** | Soft-removal countdown |
| Which version a restore brings back (date, who) | Not read [UNKNOWN]; gate | Side-by-side difference (1.8, `history.compare`) |
| Size of a whole-page replace | No (U2) | Difference preview |

### 4.3 Undo after a delete for good

The studies left a tension: refuse the whole undo, or undo what is left. Proposal:

- **1.6:** keep today's rule, which exists and was read: a change set that includes purged content is refused as a whole, with a plain message. Reason: partial undo needs to say exactly which parts were skipped and why, which is the field-level work that is not in 1.6.
- **One fix inside MI8:** when undo puts back a row whose relation cell names a row that no longer exists, that link is dropped and counted. Otherwise undo could store an id that points nowhere [derived; NOT RUN].
- **Later, with field-level undo:** undo the rest, list purged parts as "deleted for good, not restored", refuse only when nothing is left. This is a bounded open decision for the version that builds field-level undo, not for 1.6.

---

## 5. Origins, baselines and what "update later" can honestly rest on

### 5.1 What is in scope by existing authority, and what is not

- **AI source tags** exist in 1.6 by WP-06h. Making them unique and writing them inside the action is a correction inside approved scope (MI6).
- **Origins on the file route** are placed in 1.9 by WP-06h and are outside A53. Pulling them into 1.6 is a roadmap and stored-data change that needs an amendment (D4). Everything in 5.2 to 5.5 is the design **if** that amendment is given. 5.6 is the path if it is not.

### 5.2 Where things are stored

| Thing | Where | Why |
|---|---|---|
| Origin: namespace, source kind, source id, first received | Inside the item | Identity travels with the item to the bin and back; uniqueness is checked on it |
| Receive state: baselines, last seen, later pending values | A separate small record keyed by the item's id | A change to it never rewrites the item, so no revision moves, no editor is disturbed, no history version appears (5.5) |

A copy never carries either: duplicate, template and paste produce items and blocks with no origin and no receive state. Unknown fields follow their declared copy behaviour (7.3). Delete for good removes both (MI7).

**OD1 (open design point, needed before IP-7):** one receive-state record per item, or one per container. Per item is the conservative choice for correctness; its cost on a large hub belongs to study 2.

### 5.3 Baseline v1

| Part | Definition |
|---|---|
| Purpose | Tell, at a later receive, whether the source changed and whether the local value changed, per group. Nothing more |
| Groups | Title; body; all cells of a row as one group; a database's definition. Per cell is uplift U5 |
| Two stamps per group | **Source stamp:** over the source's own bytes for that group, where the route has them (the HTML of a page body, the CSV line of a row), with the name of the route and the reader version. **Stored stamp:** over the value as Notaya first stored it |
| Why two | Comparing a new converted value with an old converted value would call every item "changed at the source" as soon as the converter improves. The source stamp answers "did the source change"; the stored stamp answers "did the person change it" |
| Algorithm | SHA-256, recorded by name with a form number, so a later version can add another without guessing |
| Stored form for the stamp | A fixed serialisation defined for this purpose (ordered keys, text as stored without normalisation, numbers in their shortest exact form, absent distinct from empty). It is defined once and versioned; the vault's ordinary JSON is not used for stamping |
| What it is not | Not per field. Not a merge base: the old value itself is not kept, so 1.6's baseline can say "both changed" but cannot show what the old value was |
| What a source stamp can and cannot say | Equal stamps: the source bytes are the same, so the source did not change. Different stamps: **possibly changed**, nothing more. An export can differ in wrapper, order, encoding or options without any change in meaning. Then the converted value is compared, and only if the reader and form versions match those recorded; otherwise the answer is "unknown" |
| Consequence | No promise to detect changes of meaning. False "possibly changed" results are expected and are resolved without loss: if the converted value equals the local one, nothing changes; if not, the local value stays and the incoming one is offered. Transformations the reader does not support are listed as such, never guessed |

### 5.4 Identity of smaller things, and namespaces

| Thing | What the routes give | 1.6 treatment |
|---|---|---|
| Pages, rows, databases | Notion's 32-hex id in export names; the AI's declared id [1b 7.2] | Origin with that id |
| Properties | CSV gives **column names only**. Whether the HTML export carries property ids was not established [UNKNOWN] | Origin keyed by name and marked "name-keyed". A rename at the source will look like one property gone and one new; by the missing rule nothing is deleted |
| Select options | Names | As properties |
| Views | Ids exist in the HTML export (A53 item 6 speaks of same name, different ids) [REPORTED] | Origin with that id |
| Blocks | Whether a file export carries block ids is [UNKNOWN] | None on the file route |
| Namespace | Today a free `app` text; the tag is optional and nothing stops two items carrying the same one [1b G6] | Existing `{app, id}` tags keep exactly today's meaning. A source set is a new, explicit thing a person creates; uniqueness is enforced only for tags inside one. A namespace value 1.6 does not know is kept as it is |
| Existing duplicates | Possible today | Never collapsed or guessed. An inventory (counts only) is shown; `find.by_source` returns all matches and marks the result ambiguous; the person maps tagged items to a source set, choosing which twin belongs. Enforcing uniqueness on existing tags would change an approved optional contract (WP-06h) and needs an amendment |

### 5.5 Policies

**Baseline unknown.** An item can have an origin and no baseline for a group: items tagged by an AI, items from 1.6 development builds, cells under v1, anything linked by hand. Rule: unknown is never read as "unchanged". At a later receive, a baseline may be established from equality only when all of these hold: the incoming converted value equals the local value; the item's revision is the same when the comparison is made and when the baseline is written, inside one transaction; the reader version and the stamp form are recorded and are ones this version understands; and the baseline is tagged with that receive. It is recorded as "established at receive N", never as "as first received". If they differ, the local value stays and the incoming one is offered to the person. Nothing is overwritten on the strength of an unknown baseline.

**No origin at all** (hubs imported with 1.5.1, or with 1.6 if D4 is "no"). There is no reliable automatic matching. A later version can offer explicit linking: the person picks a notebook and a source; Notaya proposes matches and the person confirms them per container; ambiguous ones stay unlinked; everything linked starts with baseline unknown. So "can never be updated" in draft 1 was too strong; "cannot be matched safely without the person" is right.

**Content commit and receive-state commit.** A later receive unit may change nothing a person can see while still recording that it saw the item or that both sides now agree. Proposal: two *kinds* of change, not two transactions. When a unit changes content and receive state, they commit together with the origin, the baseline, the history version, the activity entry and the unit's report line: one logical unit, one transaction. Only when a unit changes nothing visible may it write receive state alone; that write moves no revision, sends no signal and adds no history, and it is made only if the item is still at the revision the unit compared against (checked inside the transaction; if not, the unit is retried). It is counted in the receive's own entry. 1.6 itself writes receive state only at import, in the item's transaction.

**How an older 1.6 is told about later receive data.** Draft 1 followed 1a in calling the ledger "optional or read-only". Resolved here as a proposal:

| Later data | Class for 1.6 | Why |
|---|---|---|
| Origin and baseline v1 | Known to 1.6 | It writes them |
| More groups or stamps inside the receive-state record | Optional | Wrong only in the safe direction: 1.6 edits, the stamp then says "local changed" |
| Pending incoming values, the receive ledger, a declined list | **Writer-critical: 1.6 must not write such a vault** | 1.6's delete for good would leave them behind (a private value outliving its item; the item returning at the next receive). With refuse-only, 1.6 refuses the vault once a later version first stores them |

Owner impact: after the first hub update with a later version, 1.6 no longer opens that vault. That is the normal direction of an upgrade, but it must be said.

### 5.6 If the amendment is not given

N3 proceeds exactly as A53 allows: one canonical row identity, redirected links, no stored origin. Hubs imported with 1.6 are then in the "no origin at all" class above. AI source tags still get MI6. The cost is paid later, by the person, in confirmations at linking time.

---

## 6. Keeping the door work independent, and development vaults safe

### 6.1 Two tracks

| Track | Content | Depends on the format decision? |
|---|---|---|
| **Door track** | MI1, MI3, MI4 at level 1, MI5, MI6 for AI tags, MI8, MI9, the three restrictions of section 4 | **No.** None of it adds a stored shape. It can be briefed as soon as development restarts |
| **Format track** | Format 3, tolerant reader, feature list, open sequence, upgrade, the three shape changes of 1a (origin record, history hash over stored bytes, history marker), receive state, level 2 recovery copies | Yes |

The PC check of old-version behaviour (IP-1) informs only the format decision. Draft 1's sentence "before any of it" was wrong.

### 6.2 Why "raise the number last" was not enough, and what replaces it

The risk: between now and a release, development and test builds write vaults. If format-track shapes reach `main` one by one under the old number, every intermediate shape becomes something a later build may meet.

Proposal:

1. **A build switch.** Format-track code is merged with its writers switched off. With the switch off, `main` keeps writing exactly the shapes of `9977e72`. Unit tests and fixture tools turn the switch on.
2. **A development epoch.** A vault written with the switch on carries, in its feature list, a marker "development epoch N". The epoch number is raised whenever a format-track shape changes. A build opens a development vault only if the epoch is its own.
3. **Release.** The release build writes format 3 with no epoch marker and refuses any vault that has one: "made by a test build". This applies only to vaults that future test builds write with the switch on. It is not a licence to discard anything that exists today: every vault written by `main` so far is format 2 and is covered by path M-b.

### 6.3 The migration paths that are owed

| Path | From | To | Note |
|---|---|---|---|
| M-a | Format 2 as 1.5.1 and older write it | Format 3 | The main path |
| M-b | Format 2 with 1.6 content as `main` writes it today: history, activity, anchor, strict `{app, id}` tags, bin state on containers, block ids | Format 3 | Tags become origins with baseline unknown; today's history hash form is kept as "form 1" and still checked the old way; cross-document duplicate block ids are counted |
| M-c | The older `.clf` data (A32 copy migration) | Format 2, then M-a in the same first launch | Interaction not read [UNKNOWN]; a fixture and a gate |
| M-d | A format 2 backup restored into 1.6 | Through M-a or M-b | Restore code not read [UNKNOWN] |
| none | Development-epoch vaults | Refused | By design |

**M-b is a real path.** Ahmed's answer (1.5) says 1.6 tests used some real notes. He does not fear losing them, but that is not permission to strand them, so M-b is owed and tested like M-a.

---

## 7. Format: fail closed, tolerance with limits, upgrade and rollback

### 7.1 What is and is not promised

- 1.6 keeps unknown fields and unknown block types **without loss of value**, not as equal bytes. Unknown values are held as their original JSON text and written back as that text, so a number 1.6 does not understand is never passed through floating point. Key order and whitespace around an unknown value are not promised; a later writer that needs those exact puts the value in as text.
- Authenticated is not the same as safe. Unknown or carried content, whether from the live item or from history, is never rendered, run or followed as a link by 1.6. It is shown as a placeholder and given to an AI as a placeholder.
- 1.6 does **not** promise that the format number never rises again. The feature list makes most later additions possible without a raise; a change that cannot be expressed that way still raises the number.
- Refuse-only is a bounded choice: 1.6 either can write a vault safely or does not open it. Read-only mode is uplift U7.

### 7.2 The open sequence: branch first, change nothing before the decision

What happens today [VERIFIED: vault/store.rs:41-51; vault/mod.rs:224-257; vault/anchor.rs:151-166]: every open uses a read-and-write connection and sets journal options; unlock decrypts every item into memory, then **creates the history and activity tables if they are missing**, then loads the history index, the activity head and the anchor. So a 1.5.1 vault opened by today's `main` is changed (tables added) before any notice or copy. The anchor rule already tolerates old vaults: a missing anchor is accepted unless the settings say one must exist or the anchor table is not empty.

Proposed sequence. Steps 1 to 4 use a read-only connection.

| Step | Does | Outcome |
|---|---|---|
| 1 | Reads the plain header: format number | Higher than this build knows: refuse, no key used |
| 2 | Unlocks: derives the key and authenticates the header and records | Wrong secret: refuse |
| 3 | **Branches on the authenticated format** | Format 3: path N. Format 2: path L. Older `.clf` data: the existing A32 path, not read here [UNKNOWN] |
| 4 N | Format 3. The feature list must exist and authenticate; each name is classified; the anchor is verified by its own rule, not updated | Missing or bad list, an unknown name not declared optional, a foreign development epoch, a failed anchor: refuse |
| 4 L | Format 2. Recognises the layout by what is present: **L1**, as 1.5.1 writes it (records and files only; no history, activity, anchor or feature list), or **L2**, as `main` writes it (those tables present; anchor present or absent). Verifies only what that layout has, using today's anchor rule for L2 | A legitimate old vault is never refused for lacking a feature list, journal tables or an anchor. Refuse only on what that layout itself forbids (for example L2 with a required anchor missing) |
| 5 | Decides: open (format 3), offer the upgrade (format 2), or refuse | For format 2 nothing is created, initialised, maintained or copied yet |
| 6 | Only after the person agrees to the upgrade, or for an accepted format 3 vault: read-and-write connection; the verified copy; the upgrade; then maintenance, anchor updates, clean-up | Declined: the file is exactly as it was |

**Test obligations for the read-only steps** (part of RG-F, not design facts):
- The vault uses SQLite's rollback-journal mode [VERIFIED: store.rs:47], so a read-only open should create no side file; this is to be shown, on local disks and on removable media.
- A vault left with a rollback journal by a crash cannot be read correctly without first rolling back, which is a write. Proposal: this one recovery is allowed before the decision, because it restores the last committed state and adds nothing; it is recorded. The alternative is to refuse and say why. OD6.
- Setting journal options at open, as today, must not happen on the read-only connection.
- After every refusal and after a declined upgrade: file bytes, file times as far as the system allows, and the folder listing are unchanged.

### 7.3 Unknown fields: what 1.6 may do with them

1.6 cannot know what an unknown field means. So the later writer declares it, in the feature list, per field or block type:

| Declared | Meaning | On save | On copy (duplicate, template, paste) | On delete for good |
|---|---|---|---|---|
| Optional, **carry** | The person's content in a form 1.6 cannot show; true whatever happens to known fields | Kept without loss | Copied with the item | Removed with it |
| Optional, **reset** | Provenance or state that must not be shared by two items (ids of origin, baselines) | Kept | Not copied; the copy starts clean. This is not loss: it is the same rule 1.6 applies to its own origin and receive state | Removed |
| Optional, **block** | Content whose ownership or relations 1.6 cannot keep right when copying | Kept | **The copy is refused** with a plain message naming the reason; nothing is dropped | Removed |
| Anything else | Depends on known fields, or needs clean-up 1.6 does not do | 1.6 does not open the vault (7.2 step 4 N) | | |

An unknown field with no declaration at all can only be old stray content in a format 2 vault; it is treated as **carry**, since dropping a person's content silently is the worse error.

So tolerance in 1.6 is: keep what is declared, copy only what is declared copyable, refuse the rest out loud. It is not "old versions cope with anything".

### 7.4 Unknown content in history

A history version may hold block types 1.6 does not know (written by a later version, or carried in an old page). Restoring it must not be blocked and must not be a way to inject content. Rule: on restore, unknown parts are accepted when they come from an authenticated history version of the same item (3.1 case B provenance). The restored page then carries them like any other carried part. A history body in a form 1.6 does not know is listed as "needs a newer Notaya" and cannot be restored by 1.6; a vault containing such bodies will carry a feature that makes 1.6 refuse it in step 3 anyway.

### 7.5 Upgrade, rollback and old versions

Unchanged from 1a 6.3 in substance: person present, AIs disconnected, free disk checked, a verified named copy that is never deleted automatically, one transaction, verification after (row counts, item counts by kind, a digest over decrypted items, anchor), then first use of new shapes. Additions from this draft: the upgrade is entered only from step 6; it handles M-a and M-b; it counts legacy duplicates.

| Situation | Outcome |
|---|---|
| Declined | The vault is left untouched and is not opened by 1.6. Conservative choice for the minimum core; read-only use instead is U7 |
| Interrupted before the transaction | Nothing changed; an extra copy may exist |
| Interrupted during it | Rolled back at next open; offered again |
| After it | Way back only by the pre-upgrade copy and an older Notaya, losing later work |
| 1.5.1 meets format 3 | Refuses with its fixed text [1a 6.5] |
| A format 3 backup offered to 1.5.1 | Refused the same way [derived] |

Hazards H1 to H10 of draft 1 stand; none was run. What keeps paths conditional: the real size of a vault and its history (only an export size is known), and the exact confirmed backup wording. The plan measures on large synthetic vaults before release, treats the lighter upgrade variant as needing its own security review, keeps M-b as an owed path, and asks for the wording once more. History kept without limit makes every one of these operations larger over time (13.3).

**If the format decision is "stay on 2":** the door track is unaffected. The format track shrinks to nothing that changes stored shapes, origins on the file route are not possible in 1.6, the old-version hazards stay open and must be in the release notes, and the single raise moves to the first later version that needs it, which then also owes M-b.

---

## 8. Tiers and sequenced packages

### 8.1 Three tiers

**1.6 minimum safe core** (what the plan says 1.6 should not be released without, each still subject to Ahmed's decisions):

| # | Item | Track |
|---|---|---|
| C1 | MI1 atomic delete for good | Door |
| C2 | MI3 tokens and required base token | Door |
| C3 | MI4 enforced editor contract at level 1, with 3.4 and the three gates | Door |
| C4 | MI5 block-id rules of 2.3 | Door |
| C5 | MI6 for AI source tags | Door |
| C6 | MI8 and the relation fix of 4.3 | Door |
| C7 | MI9 importer cuts reported | Door (importer) |
| C8 | The three AI restrictions and questions of section 4 (D5) | Door |
| C9 | Format 3 with the open sequence, feature list, tolerance as in 7.3, upgrade paths M-a to M-d, development epoch (D1) | Format |
| C10 | The three stored shapes of 1a changed before release (origin record replaces the strict tag, history hash over stored bytes, history marker) | Format |
| C11 | History retention as a stored policy with no age or size limit by default, replacing the fixed thinning rules; the minimum controls of 13.5 (D14) | Format |

**Optional uplift** (worth doing in 1.6 if Ahmed chooses; none is assumed):

| # | Item | Needs |
|---|---|---|
| U0 | Origins and baseline v1 on the file route, inside N3 | Amendment (D4); C9, C10 |
| U1 | Level 2 recovery copies of refused edits | Retention decision (D7); C9 |
| U2 | `page.write` asks when it removes many blocks | D5 |
| U3 | "What it was before" for items an undo skipped | None beyond D3 |
| U4 | Removed list for properties (definition kept, hidden, can be put back) | Retention decision (D7); C9 |
| U5 | Per-cell baselines | U0; study 2 cost |
| U6 | "Also removes N links" in the Windows delete dialog | Ahmed's word (A51 text) |
| U7 | Read-only mode, for declined upgrades and for newer vaults | Its own security review |
| U8 | Placeholder block in the editor, if 3.4's trial shows it is small | The trial |
| U9 | A version taken just before a person's save removes whole blocks (2.3) | D14: it adds stored history |

**Later consumers** are set out against the existing roadmap in section 15. Studies 2 to 5 keep the hand-over lists of 1b 11.3, plus OD1, the save-time check cost and the history measurements of 13.3.

### 8.2 Packages

Nothing is executed or authorised. **Every package below runs only in the approved copy of section 14; the source and the preserved lanes are never written.** Standing rules when work restarts: at most three lanes, a bounded write area and fresh brief per lane, exact-commit checks, review by a session that did not write the diff, merges as merge commits so that a lane can be taken back by reverting its merge [REPORTED: pause note, CLAUDE.md].

| # | Package | Preserves and touches | Interface owner; who must re-run | Evidence gate | Stop condition | Way back |
|---|---|---|---|---|---|---|
| IP-0a | Evidence: remaining answers of section 11.3 | Nothing | Blue | Answers recorded | n/a | n/a |
| IP-0b | Verified copy handoff (section 14), which also captures K1c's five untracked files, S's `area3/` folder and W's modified sheet | Source and all lanes, read-only | Ahmed's word; whoever he names | Manifest and checksums match; destination status equals the source inventory | Any write to the source; any scheduler or automation left switched on in the copy | The copy is discarded; the source is as it was |
| IP-1 | Old-version PC check on throwaway data (1.5.1 exe against a vault written by `main`) | Test folder only | Feeds D1 only | Observations of H1 to H4, including "does it still unlock" | Any path that is not the test folder | n/a |
| IP-2 | Record and script lanes: RV-P6b note, P6c, W with its modified sheet, S, R1, ledger rows | Those branches as they are | W owns the harness. The 36 flagged assertions are resolved as harness faults or real defects before the sheet is used as a gate | P6c's final browser run; harness signals classified | A merge that would drop W's modified sheet or S's untracked evidence | Revert the merge |
| IP-3 | N2 | `7806d00` | Importer owner. Consumers: N3, gate 2 of section 3.3 | Exact-tip `cargo test --lib` and a fresh independent review; M1, M2, L1, L2 reconciled against RV-N2 | A failing test that was passing at `e11b115`; any importer output the core schema refuses | Revert the merge |
| IP-4 | Door track A: C1, C2, C6 | Vault write path, state, dispatcher, purge | Owner: this lane. Must re-run: bridge and AI help text (token shape), S area 1 and 2 proofs, W's AI rows, native-confirm tests | RG-C, RG-E | A change to any stored shape; a person's save being refused | Revert; no stored data involved |
| IP-5 | Door track B: C3, C4 | Editor save path, block validator, paste, duplicate, templates, restore | Owner: this lane. Must re-run: importer fixtures (N2, later N3), editor checks in `dev-mock/`, P6 inline-table checks | The three gates of 3.3; RG-B; save-time cost measured | Gate 1 or 2 fails; a valid page that cannot be saved; typing delay a person notices | Revert. Enforcement has its own switch during development so it can be turned off without removing the lane |
| IP-6 | Door track C: C5, C8, chosen U2, U3 | Action registry, confirmation rules, source tag write | Owner: this lane. Must re-run: spec section 6 tables and the bridge's tool list (if an action is withheld, it is not listed) | RG-A per action | An action changing level or confirmation without D5 | Revert |
| IP-7 | Format track, writers switched off: C9, C10, C11, and U1, U4, U7 if chosen | Vault open, model, history, anchor, upgrade dialog | Owner: this lane, alone on the vault files while it runs. Must re-run: backup and restore, installer and portable first launch, every harness script that opens a vault | RG-F, RG-G, RG-I, RG-Q on fixtures | `main` with the switch off writing anything `9977e72` did not; a refusal that changes a file | Switch stays off; revert |
| IP-8 | N3 | N2's result; A53's six points | Importer owner. With D4: writes origins and baseline v1 (needs IP-7). Without: 5.6 | RG-J if D4; importer output passes the core schema | Stored origins without the amendment | Revert |
| IP-9 | Speed P7, P8 | WP-16 as specified | Database views owner; coordinate with IP-5 on row duplicate | The plan's accuracy gate | A visible change not listed for Ahmed | Revert |
| IP-10 | Security: area 3 on the route Ahmed chooses, area 4, and a review of the open sequence, upgrade, tokens, editor contract, origins as untrusted input | Review notes; S's evidence | Reviewer independent of the authors | Findings closed or listed for Ahmed | A filter refusal is a stop, not something to work round | n/a |
| IP-11 | Polish sweep, About text, X-1 on Mohamed's PC | WP-18 | Partners | Partner checks | | Revert |
| IP-12 | Release candidate: format switch on, epoch removed, all gates on the exact commit | Everything | Coordinator | Section 10 | Any gate not run is reported as not run, never as passed | No release |
| IP-13 | Amendments and records for whatever Ahmed decides | Repository records, by whoever he authorises | | | | |

**Order.** IP-0a, IP-0b first. Then IP-2 and IP-3 beside IP-4. IP-5 after IP-3 (it needs the importer's output) and after IP-4 (same files). IP-6 after IP-4. IP-7 after IP-5, alone on the vault files. IP-8 after IP-3, and after IP-7 if D4. IP-9 any time within the lane limit. IP-10 once IP-4 to IP-8 are reviewable. IP-1 any time after Ahmed's go; it gates only D1.

---

## 9. What is planning, what is approved, what waits

Nothing in this plan changes a recorded decision. "May be planned now" means design and briefs on paper only.

| Topic | Existing authority | May be planned now | Before implementation | Before release |
|---|---|---|---|---|
| Format raise in 1.6 | G3 in A44 says 1.5.1 still opens a 1.6 vault; format gate sits at 1.8 | Design of sections 6 and 7 | Amendment of G3 and the version plan gate; Ahmed's word on a vault-format change | Old-version statement in release notes; Mohamed told |
| Tokens and required base token | G1 covers the AI door's rules; this tightens them | Yes | Ahmed's word (security behaviour) | `security-design.md` |
| Enforced editor check | None; a behaviour change on the typing path | Yes | Ahmed's word; gates of 3.3 | D9 if gates unmet |
| Withholding `change_kind`; automations always ask; property removal rule | Spec 6.2 approves the present levels and questions | Yes | Amendment of the action table | Release notes |
| `batch.run` | WP-06e promises it; code refuses | Both designs | D8: build as specified, or amend the spec | Tool list matches what works |
| File-route origins and baselines | WP-06h: 1.9. A53: no storage change | Section 5 | Amendment | |
| Any new kept data: recovery copies, removed list, declined list, a changed thinning rule | WP-04's thinning text; SF3 on titles | Yes | Ahmed's word on each (privacy and retention) | `security-design.md` |
| Titles in the log after delete for good (CR2) | SF3: kept | Both options | Ahmed's word to change | |
| Native dialog text or new native dialogs | A51 names four actions | Yes | Ahmed's word | |
| Read-only mode | None | Yes | Security review; Ahmed's word | |
| K1 development kit | Unmerged; K1b and K1c not done [REPORTED] | Options only: finish and verify in its own lane, or keep it outside the release, or drop a part | Ahmed's choice (D10); K1c captured first | Not shipped unverified |
| N2, P6c, S, W, R1, ledger | Lanes produced under 1.6's go; paused by A56 | Order and gates | Restart authority (D0) | Their own gates |
| CR1, CR3, CR5, CR6, CR7 | Descriptions of today's code or vocabulary | Used as working assumptions | Nothing to implement | |
| CR4, CR8 | Study proposals | CR4 as in section 5; CR8 to study 2 | As file-route origins | |
| History kept without limit by default | WP-04 fixes thinning rules and a 500 MB cap; Ahmed's direction of 11 November is for planning | Section 13 | Ahmed's word to implement; his answers to D14; amendment of WP-04's thinning text | `security-design.md` (removed text stays until the person removes it); release notes |
| Enforcing uniqueness on existing AI tags | WP-06h makes the tag optional with no such rule | 5.4 | Amendment, if ever wanted | |
| Copy of the source to a working root | Ahmed's boundary of 11 November; no copying authorised | Section 14 | Ahmed's word, destination named by him | |

---

## 10. Evidence gates

**All NOT RUN.** A result will apply only to the exact commit it ran on. Fixtures are invented (the "Garden Club" set of the studies); real data only where a row says so, on its owner's PC, numbers only.

| Gate | Shows | Notes |
|---|---|---|
| RG-A | Per AI action: the question asked, the way back, and that the way back works; the unknowns of 4.1 answered (`button.press`, `file.remove`, undo of undo, `calc.clear`, restore dialog text); `change_kind` behaviour observed before any decision to offer it again | Rust tests plus a PC round on a throwaway vault |
| RG-B | Block ids: copy and cut across pages give new ids; duplicate and template give new ids; AI move keeps ids and refuses a collision; restore keeps only ids the live item has; after a crash between cut and paste, page A's last version still holds the blocks as they were then (the boundary of 2.3), and with U9 the version from just before the cut | Includes the count of legacy duplicates in a development-build fixture |
| RG-C | Tokens: a token from before a lock never matches; whole-item replace without a token is refused for each action in MI3; a person's stale save still lands and the AI's value is kept | |
| RG-D | Editor contract: the three gates of 3.3; a legacy page with odd parts stays editable around them; a new invalid part is refused and the text stays in the window; save-time cost on a large page | Gate 3 on a real vault only with Ahmed's word |
| RG-E | Delete for good: forced failure at each step leaves everything as before; afterwards nothing 1.6 stores about the item remains; undo and restore obey hidden notebooks, leases, locked databases and relation targets | |
| RG-F | Open sequence: a 1.5.1-layout vault and a `main`-layout vault, with and without an anchor, are each recognised and offered the upgrade, never refused for lacking newer structures; for each refusal and for a declined upgrade the file, its times and the folder listing are unchanged; no table, anchor or copy is created before consent; no side file from the read-only steps on local and removable disks; a vault left with a rollback journal behaves as OD6 decides | Windows, real exe |
| RG-G | Upgrade: M-a to M-d on fixture vaults, counts and digest equal, anchor verifies, copy present and verified; kill at each step, disk full, lock and quit during, an AI connecting during; time and disk on a large fixture | Windows |
| RG-H | Old versions: 1.5.1 exe on an upgraded copy refuses and changes nothing; IP-1's sequence | Windows |
| RG-I | Tolerance: a hand-made later vault with only "optional" unknowns opens, is edited, and its unknown values are written back without loss (including a number too precise for floating point); a "carry" field is on the duplicate, a "reset" field is not, a "block" field makes the duplicate refuse with nothing dropped; carried content is never rendered; with anything else it is refused unchanged; a development-epoch vault is refused by a release build | |
| RG-J | If D4: same export twice into one source set is refused as duplicate; into two is two copies; baseline v1 says "unchanged" for an untouched item and "changed" after a local edit; a copy carries no origin; delete for good removes the receive state | |
| RG-K | Per lane: `cargo test --lib`, `tsc`, build, audit on the exact commit; independent review | Ahmed's PC |
| RG-L | Security areas 3 and 4 and the review of new pieces | |
| RG-M | Whole-app sheet (A47): harness faults fixed first, then all 250 rows on the release candidate after the upgrade step exists | Earlier partial results do not carry over |
| RG-N | WP-19 step 4 with Ahmed's private exports on his PC | Nothing from them in Git or records |
| RG-O | X-1 on Mohamed's PC; About text; installer and portable ZIP | |
| RG-P | Records: amendments, `security-design.md`, release notes | |
| RG-Q | History retention: the tests of 13.7 | Synthetic vaults; Windows for disk-pressure cases |
| RG-R | Copy handoff: the checks of 14.3 | On Ahmed's PC, when authorised |

No design here is "safe" or "proven" until its gate has run and been reviewed; then it is "verified on that commit" and nothing more. Acceptance is each partner's own word.

---

## 11. Decisions and open questions

### 11.1 Decisions for Ahmed

None is taken by this plan. "Latest point" is the last moment the decision can wait without holding a package; a conservative path is named where planning can go on without it.

| # | Decision | Recommendation | Impact on Ahmed | Latest point | Meanwhile |
|---|---|---|---|---|---|
| D0 | Restart development, and under which coordinator (A55, A56) | Not mine | Everything | Before IP-2 | Planning only |
| D1 | Raise the format in 1.6 (amends G3 and the 1.8 gate), or stay on 2 | Raise, after IP-1 | After upgrading, 1.5.1 no longer opens his notes; the pre-upgrade copy is the only way back | Before IP-7 is briefed | Door track proceeds either way |
| D2 | Run the old-version PC check | Yes | A short test on throwaway data | Before D1 | |
| D3 | Core only, or core plus which uplifts | Core plus U0, U3, U6 | More in 1.6, later acceptance | Before IP-6 and IP-7 briefs | Plan core only |
| D4 | File-route origins and baselines in 1.6 (amendment) | Yes | Hubs imported with 1.6 can later be updated without hand-linking | Before IP-8 is briefed | N3 as A53 allows (5.6) |
| D5 | AI door changes: required base token; `change_kind` withheld; automations always ask; property-removal rule | Yes to all | An AI must read before overwriting; one more question when an AI makes an automation; one action unavailable to AIs for now | Before IP-4 (token) and IP-6 (rest) | |
| D6 | Declining the upgrade: not opened by 1.6, or read-only mode (U7) | Not opened | He must upgrade or keep 1.5.1 for that vault | Before IP-7 | Conservative: not opened |
| D7 | New kept data, one by one: recovery copies of refused edits (U1), removed list for properties (U4), titles after delete for good (CR2). History retention itself is D14 | U1 and U4: his call, each is a retention choice. Titles: keep the recorded behaviour | What stays in the vault after he removes or loses something | Before IP-7 | None of them built |
| D8 | `batch.run`: build as WP-06e says, or amend the spec for 1.6 | Amend | An AI works action by action in 1.6 | Before IP-6 | Refused, tool not listed |
| D9 | If the editor gates are not met at release time: hold, or release with the two doors still differing and say so | Hold | Release date against a known gap | At IP-12 | |
| D10 | K1 kit: finish and verify, keep outside the release, or drop a part | Finish and verify in its own lane, outside the release | Tooling for his own PC | Before IP-12 | K1c captured (IP-0b) |
| D11 | Route for security area 3 | His | Release gate | Before IP-10 | |
| D12 | Exact wording of the pre-upgrade copy notice | Confirm once more | What he reads at first launch | Before IP-7 ends | Draft of 1a 6.5 |
| D13 | The copy handoff (section 14): when, to where, by whom | Yes, before any other work; destination his to name | Protects the source and the unfinished lanes | Before IP-1 and every later package | Source stays read-only; nothing is copied |
| D14 | History: the two scope questions of 13.2, and which controls of 13.5 are in 1.6 | Reach back without limit; keep-everything density only after measurement; cells in scope decided by him | What stays in the vault for ever, how large it grows, what a password change and a backup cost | Before IP-7 is briefed | Planned as: no age or size limit, existing version rhythm, pages and row bodies only |

Mohamed: told about the upgrade's one-way nature before 1.6 is offered to him; X-1 and the About text are his; his acceptance is his own word.

### 11.2 Open design points

| # | Point | Needed before |
|---|---|---|
| OD1 | Receive state per item or per container | IP-8 with D4 |
| OD2 | Placeholder block or view-only for pages with carried parts (3.4) | IP-5 |
| OD3 | Whole-document or changed-blocks check at save time | IP-5, after measuring |
| OD4 | Full re-seal or the lighter upgrade variant | IP-7, after measuring on a large fixture; the lighter one needs a security review |
| OD5 | Where maintenance and clean-up are first triggered after unlock today (table creation at unlock is now read: 7.2) | IP-7 |
| OD6 | A vault left with a rollback journal: recover before the decision, or refuse | IP-7 |
| OD7 | History stored as full copies (today) or in a deduplicated form (13.4) | Before "keep everything" is promised at scale |

### 11.3 Still unknown; the plan takes the conservative branch

| Unknown | Conservative branch used |
|---|---|
| How many vaults written by 1.6 builds hold real notes, and where | Answered in kind (some, none he fears losing); M-b is a real path regardless |
| Size of the largest real vault, and its working memory | Only an export size is known (a little over 1 GB). Measured on synthetic vaults of comparable and larger shape; no promise about upgrade time, disk or memory |
| Exact confirmed wording of the G3 notice | Ask once more (D12) |
| Whether Ahmed has said "handover accepted" | D0 stays open |
| Whether the two scheduled tasks are removed | Nothing assumes automation is on or off |
| Whether anyone outside the partners has 1.5.1 data to upgrade | Release notes written as if yes |
| What `tools/security-pass/area3/` contains | Captured unread; reviewed before any use |

---

## 12. (Number not used)

Draft 2's section 12 is section 17 here, so that section numbers 1 to 11 match between drafts 2 and 3.

---

## 13. History kept without limit by default: what it really costs, and a design

Ahmed's direction [BLUE]: history is kept indefinitely by default; a person can set age or size limits; someone with room can recover a change from a year ago. Planning direction only.

### 13.1 What history is today [VERIFIED: vault/history.rs:15-31, 40-91, 205-222, 268-335; vault/mod.rs:239-257, 733-745, 842-850, 908-910]

- A version is a **full copy** of a page or row body: title, the whole document in the editor's structured form, a second plain-text copy, icon and cover. It is not Markdown and not a difference; ten versions of a page cost about ten pages.
- Only pages and row bodies have versions. Cells, database definitions, stickies, notebooks and sections do not.
- Each version also has a small index entry (title, word count, picture ids, size, hash). **All index entries are loaded into memory at unlock.**
- When versions are taken: for a person at most once per 10 minutes, on leaving a page and before a restore; for an AI, before and after its write.
- Any picture or file named by a kept version is kept by the clean-up, even when no live page uses it.
- A password change re-seals every history row in one transaction. A backup copies the whole vault file, history included.
- Today's limits are constants in code: everything for 7 days, hourly to 30, daily to a year, AI before-versions 90 days, newest 20 always, a 500 MB soft cap.

### 13.2 Which stream is meant, and two questions that change the design

| Stream | Holds | Limit today | Under the new direction (proposal) |
|---|---|---|---|
| Content history | Versions of pages and row bodies | The constants above | **No age or size limit by default** |
| AI undo data | Whole-item before-images for AI changes, any kind of item | Goes with the activity log: 365 days or 100,000 entries | Unchanged unless HQ2 says otherwise; shown plainly ("an AI change can be undone for a year") |
| Activity log | Who changed what and when, with titles | Same | Unchanged; it is an audit record, not content recovery |
| Replaced-edit copies | An AI value a person's save replaced | Not read [UNKNOWN] | To be read before deciding |
| Anchor and integrity data | Protects the log and history | None | Unchanged |
| Backups | Whole-vault copies, separate files | Their own rules, not read [UNKNOWN] | Unchanged; they contain history |

**HQ1. "Every version for ever", or "reach back for ever, in coarser steps for old versions"?** With the first, growth is proportional to how much someone edits; with the second it flattens. The first is only affordable at scale with a different way of storing versions (13.4, HR-C).

**HQ2. Does "recover a change from a year ago" include database cells, database structure and AI changes to them?** Today those can only be recovered from AI undo data, which ends at a year and covers AI changes only; a person's cell edit has no stored way back at all. If yes, cells in history (not on the roadmap, section 15) or longer-lived undo data becomes part of this design.

These are the only two questions asked; the rest below holds either way.

### 13.3 The costs, beyond disk

| Cost | Why | Measured? |
|---|---|---|
| Stored size | Full copies. Illustration only: a page whose stored form is 50 KB, edited through a working day, at one version per 10 minutes, adds roughly 2 to 3 MB that day [ESTIMATE]. An AI that makes many separate writes to one page adds two full copies per write | NOT RUN |
| Unlock time and memory | The whole index is read and held at unlock; it grows with every version ever kept | NOT RUN |
| Files that cannot be freed | A removed picture or attached file (up to hundreds of megabytes each) stays as long as one kept version names it. With no limit, that is for ever | Follows from the code |
| Backups | Every backup is a full copy including history; several kept backups multiply it | NOT RUN |
| Password change and format upgrade | One transaction over every row; time and temporary disk of about the vault's size again [1a 6.2 ESTIMATE] | NOT RUN |
| Clean-up work | Deleted rows are overwritten on disk (secure delete is on), so a large explicit clean-up is slow | NOT RUN |
| Privacy | Text a person removed from a page stays readable in history until they remove it there; it travels in backups. Under the old rules it faded within a year | By design |

None of this argues against the direction. It says the default needs three things with it: a way to see the size, a way to remove history on purpose, and an index that is not loaded whole at unlock.

### 13.4 Storage options

| Option | What | For | Against |
|---|---|---|---|
| HR-A | Today's full copies; no limits; person-set limits | Smallest change | Highest growth; needs the lazy index before it is safe at scale |
| HR-B | No age limit, but old versions are coarsened (for example hourly after a week, daily after a month, weekly after a year), never dropping a protected version; a switch for "keep every version" | A change from years ago stays recoverable at day or week precision; bounded growth | Not "every version"; only fits Ahmed's direction if HQ1 says so |
| HR-C | A second history form: blocks stored once per item, a version is a list of block references | Makes "every version" affordable; unchanged blocks cost nothing | Larger work; new clean-up rules; must stay per item so delete for good remains simple. Uses the history marker of C10. OD7 |

**Proposal.** In 1.6: a stored retention policy (13.5) with no age or size limit by default, on today's full copies, with the lazy index and the controls below; the measurements of 13.7 decide whether HR-A is acceptable as the shipped default density or whether HR-B's coarsening is offered as the default with "keep every version" as a choice. HR-C is a later candidate that C10's marker keeps open.

### 13.5 The retention policy and the person's controls

Stored in the vault, changed only by the person through its own commands (never by an AI and never through the general settings action):

| Field | Values | Default |
|---|---|---|
| Age limit | None, or a number of days | None |
| Size limit | None, or a size | None |
| Density | Every version; or coarsen by a named rule | By HQ1 |
| Always kept | The newest versions of each item; the version before each AI change, restore, receive and upgrade | On |

Rules:
- **No old rule applies silently.** A vault without a policy gets the new default. The fixed 7-day, 30-day, 90-day, 365-day and 500 MB constants are not used for content history any more. Vaults that development builds already thinned keep what they have; nothing returns.
- **Limits are opt-in and their effect is shown first.** Setting or lowering a limit shows how many versions and how much space would go, says it cannot be undone and that older backups still hold them, and asks. Then it is kept up by maintenance.
- **Explicit clean-up by the person:** delete the history of this page; delete versions older than a date. Same confirmation.
- **Delete for good is unchanged:** it removes all history of the item, whatever the policy.
- **The person can see** the total size of history and, as uplift, the largest items.

Minimum in 1.6 (C11): the policy with its defaults, the size display, the two explicit clean-ups, the lazy index, and the disk-pressure behaviour of 13.6.

### 13.6 When the disk fills

1. **Notice** at a first threshold of free space: history size and free space shown; nothing changes.
2. **Low:** the optional 10-minute versions of a person's typing are skipped, with a visible "history paused: disk nearly full" marker and a log entry. Never silently.
3. **Required versions are never skipped:** before an AI change, a restore, a receive, an upgrade. If one cannot be written, that action is refused with the reason; a person's own typing still saves.
4. **A save of content must not fail because its optional version could not be written.** Today both are one transaction; the proposal is to retry the save without the optional version.
5. **Nothing is deleted automatically** to make room. The person is offered the explicit clean-ups.
6. **Before a password change, an upgrade or a backup,** the needed space is checked and the person is told before it starts.

Consequence to state: with no limit and a nearly full disk, a password change or an upgrade can become impossible until space is freed or history is reduced.

### 13.7 Acceptance tests (design only, NOT RUN)

On synthetic vaults shaped like real use and larger (many pages, a few very large pages, large databases, many pictures and big files; a hub a little over 1 GB and several times that):

1. With the default policy, after simulated years of editing, no version has been removed; a change from over a year ago is restorable.
2. A vault with no policy gets the default; none of the old constants takes effect.
3. Setting an age or size limit shows the right counts before anything is removed; cancel removes nothing; confirm removes exactly those.
4. "Always kept" versions survive a limit, or the person is told the limit cannot be met and asked.
5. Delete for good removes every version and the item's files; nothing of it is found afterwards.
6. A picture used only by old versions is kept; after the history of that page is deleted, the picture is removed by clean-up.
7. Unlock time and memory against number of versions, with and without the lazy index.
8. Stored size per version for small, large and picture-heavy pages; growth for an AI that edits one page in many steps.
9. Backup size and time; password change time and temporary disk; upgrade time and disk.
10. Disk nearly full: optional versions pause visibly; an AI change that cannot store its "before" version is refused; typing still saves; nothing is deleted.
11. A crash during explicit clean-up leaves history either as before or fully cleaned.
12. An AI cannot read or change the retention policy.

---

## 14. Keeping the source untouched: the copy handoff, and pausing and restarting

### 14.1 Rules

- `D:\Notaya`, `D:\Notaya-lanes\*` and every worktree are **read-only** for planning and afterwards. No command that writes there: no commit, stash, checkout, clean, reset, rebase, fetch or pull.
- Later development, tests and builds run only in an approved copy in a **separate working root**, named by Ahmed. This plan names none.
- The copy is made only on Ahmed's word. Nothing in this plan authorises it.

### 14.2 What a copy must carry, and why a clone is not enough

A fresh clone carries pushed branches only. It would silently leave behind exactly what must be preserved:

| Thing | Where it is | In a clone? |
|---|---|---|
| K1c `4ed8257` | Local branch at `D:/Notaya-lanes/K1C`, never pushed | No |
| K1c's five untracked files | That worktree | No |
| S's `tools/security-pass/area3/` | Untracked in the S worktree | No |
| W's modified results sheet | Modified, uncommitted in the W worktree | No |
| Any other local-only branch, stash, or commit reachable only locally | The source repository | No; to be listed by the inventory [UNKNOWN] |
| Ignored files (development vaults, test data, keys) | Possibly in worktrees | No, and **they must not be copied blindly**: vault data and secrets are listed by name and size only, and copied only if Ahmed says so |
| The live heartbeat scripts and scheduled-task definitions | `D:\Notaya-lanes\heartbeat`; Windows Task Scheduler | No |

A plain folder copy has its own trap: a Git worktree records the absolute path of its repository, so a copied worktree can still point at the **source** repository, and a command run in the copy could then change the source. The copy must not be used until every such link points inside the destination.

### 14.3 The handoff, as a later authorised procedure

1. **Inventory, read-only.** For the main repository and each worktree: path, branch, commit, upstream and whether pushed, modified tracked files, untracked files, ignored files by name and size, stashes, local-only branches. Scheduled tasks and their state. Written to a manifest outside both roots.
2. **Checksums.** A SHA-256 for every modified and untracked file, and the commit id of every branch tip, in the manifest.
3. **Build the destination.** The repository's history is brought over complete, including local-only branches and stashes; each lane is set up as its own worktree in the destination at the recorded commit; then the modified and untracked files are laid over it from the source by the manifest. Ignored vault data and secrets are left out unless Ahmed names them.
4. **Verify.** In the destination: every branch tip equals the manifest; the repository checks as intact; each worktree's status lists exactly the manifest's modified and untracked files; every checksum matches; no path inside the destination's Git data points at the source.
5. **Isolate.** No scheduled task is registered from the destination; the heartbeat and guard scripts are present as files only. Pushing from the destination stays off until Ahmed allows it. The destination gets its own test folder; no setting points at the source's test folder or at anyone's real vault.
6. **Source afterwards.** Unchanged, verified by repeating the inventory and comparing it with the manifest. It stays as the reference.

Stop conditions: any step that would write to the source; any mismatch in step 4 (the copy is discarded and redone, the source is not "fixed"); a scheduled task found running against either root (reported, not touched without Ahmed's word).

### 14.4 Pausing and restarting safely

- Every package ends at a stop point in the destination: work committed and pushed to its own branch when pushing is allowed, a resume note, exact-commit check results, what is unverified.
- No background automation is needed by this plan. If a heartbeat is wanted again, it is registered deliberately, for the destination only, after its kit is verified (D10).
- A restart begins by repeating the destination's inventory against its last resume note, not by trusting memory or an older record.
- A limit reached mid-package is a stop, with the checkpoint written first.

---

## 15. Later versions: existing roadmap against proposed placement

"Roadmap" is the version plan of 7 November as read [VERIFIED: version plan section 4, wave 1]. Nothing in the right-hand column is a commitment; moving anything into a version is Ahmed's roadmap decision.

| Work | On the roadmap today | Proposed placement | Note |
|---|---|---|---|
| Block links that survive a move | 1.8-11, with "every block gets a stable id"; vault-format gate | Stays 1.8 | Needs the owner index or move protocol first; 1.6 only stops duplicates |
| Notion id kept on import | 1.8-14 | 1.6 as U0 if amended, else stays 1.8 | Roadmap change either way it moves |
| History: see what changed | 1.8-8 | Stays 1.8 | |
| Activity panel | 1.8-9 | Stays 1.8 | |
| Mentions, columns, callouts, folding headings, file cards | 1.8-1 to 1.8-10 | Stay 1.8 | With 1.6's feature list, each is a listed feature without a number raise, if designed so |
| Import into an existing place | 1.9-7 | Stays 1.9 | |
| Re-import without duplicates | 1.9-8: "edits you made in Notaya are kept as a second version, not overwritten" | Stays 1.9 | The roadmap wording reads as "source arrives, local kept as a version"; Study 1b recommended "local stays, source offered". An existing difference for Ahmed to settle before 1.9, not now |
| "What did not carry over" report | 1.9-9 | Stays 1.9; 1.6 reports importer cuts (C7) | |
| Sync ledger | 1.9-17 | Stays 1.9 | Classed writer-critical in 5.5 |
| Exchange folder for AI files | 1.7-12 | Stays 1.7 | |
| Descriptive, difference-based confirmations | **Not on the roadmap** | Unplaced. Candidate: with 1.7's menus and shortcuts, or 1.8's history views | Section 4.2 lists what 1.6 itself needs |
| Named actions for person doors | Not on the roadmap as such (1.6-1 names actions) | Unplaced | |
| Read-only mode | **Not on the roadmap** | Unplaced; U7 if wanted in 1.6 | Needs a security review |
| Cells and definitions in history | **Not on the roadmap** | Unplaced; depends on HQ2 | |
| Soft removal of properties, views, templates | **Not on the roadmap** | Unplaced; U4 is a small piece | Retention decision |
| Field-level undo; a way back for a person's cell edits | **Not on the roadmap** | Unplaced | |
| Declined list, pending incoming values | **Not on the roadmap** | Unplaced; with 1.9-8 if wanted | Privacy decision |
| Configurable history retention | 1.6-4 has fixed thinning | 1.6 as C11 by Ahmed's new direction | Amends WP-04's thinning text |
| Deduplicated history (HR-C) | **Not on the roadmap** | Unplaced | |
| Dashboard and whiteboard kinds | 2.1, 2.3, vault-format gates | Stay | Opaque unknown kinds in 1.6 make them listed features |

---

## 16. The owner's decision bundle

Compact form of section 11.1 for Ahmed. Nothing here is decided. Each line is a question, my recommendation, and what it costs him.

| Bundle | Question | Recommendation | What it means for him |
|---|---|---|---|
| A. Restart and source | When and where is the verified copy made, and who continues there? (D0, D13) | Copy first, then everything else | Source and unfinished lanes stay exactly as they are |
| B. Storage format | Raise the format in 1.6 after one small PC check on throwaway data, changing the approved "1.5.1 still opens a 1.6 vault"? (D1, D2, D6, D12) | Yes | After upgrading, older Notaya refuses those notes; the copy made just before is the way back |
| C. AI door | Require an AI to have read a page before replacing it; withhold "change a property's kind" from AIs for now; always ask when an AI makes or changes an automation; say what removing a property does? (D5) | Yes to all four | A little more asking; one AI action unavailable until tested |
| D. Editor check | Enforce one rule for what a page may contain, keeping old odd content, after three evidence gates; and if the gates are not met, hold the release? (D9) | Yes; hold | Protects pages from two doors disagreeing; a refused edit is not kept across a lock or crash unless he also chooses stored recovery copies |
| E. Batches | Build `batch.run` as the spec promised, or amend the spec for 1.6? (D8) | Amend | AIs work action by action in 1.6 |
| F. Imports that can be updated later | Store where imported items came from in 1.6, moving that from the 1.9 plan? (D4) | Yes | Hubs imported with 1.6 can later be updated without linking them by hand |
| G. History | Every version for ever, or reach back for ever in coarser steps (HQ1)? Are database cells and structure part of "recover a change from a year ago" (HQ2)? Which controls in 1.6? (D14) | Decide HQ1 after the measurements; HQ2 is his | How large the vault grows, how long unlock, backup and a password change take, and that removed text stays until he removes it |
| H. Other kept data | Stored recovery copies of refused edits; a "removed" list for properties; titles in the log after delete for good (today: kept)? (D7) | His call on each; no change by default | What remains in the vault after he removes something |
| I. Scope | Minimum core only, or with which uplifts? (D3) | Core plus U0, U3, U6 | Size of 1.6 against how complete it feels |
| J. Tools and reviews | The K1 kit; the route for security area 3 (D10, D11) | Kit verified in its own lane, outside the release; area 3 his | Release gate |

**Measured facts still missing** (none can be had without a run): what 1.5.1 really does to a 1.6 vault; what the editor does with an unknown block; whether property values survive later row saves and what `change_kind` shows; cost of the save-time check; upgrade, password-change, backup and unlock time and space on large vaults; history growth per version; read-only open behaviour on Windows disks. Each has a gate in section 10.


---

## 17. Trace, limits and log

### 17.1 From draft 1 to draft 2 (kept for trace; draft 2 to 3 is in 0.2)

| Draft 1 | Draft 2 |
|---|---|
| F1 format raise | C9 |
| F2a, F2b, F2c tolerance | C9 with 7.1 to 7.4; read-only mode U7; editor side 3.4, U8 |
| F3 three shapes | C10 |
| F4a, F4b tokens | C2 |
| F4c atomic purge | C1 |
| F4d tag inside the action | C5 |
| F4e report-only editor check | Withdrawn; C3 |
| F4f commit seam | Withdrawn as a deliverable; replaced by MI1 to MI9 |
| F5 block ids | C4 with 2.3 |
| F6, F7 origins and uniqueness | U0 (file route), C5 (AI tags) |
| F8 importer cuts | C7 |
| F9 cells in history, soft removal, field-level undo | Later; U3 and U4 as small pieces; section 4 restrictions |
| F10, F11 receive, declined list | Later; classification in 5.5 |
| F12 `batch.run` off | D8 |
| F13 descriptive confirmations | 4.2 split: needed now against later |
| F14 markers, owner index | Later |
| F15 thinning rule | D7; no change proposed by default |
| Scopes S, M, L | Tiers of 8.1 |
| "Reversible defaults" D-R1 to D-R10 | Section 9 |

### 17.2 Limits of draft 2's reading (still true)

- Nothing was built, run, tested, fetched, committed or pushed. No repository, branch, PC, scheduler or Notion change.
- Read in full for this draft: spec section 3 (gates), WP-04, WP-05, WP-06 (a to i), sections 6.1 to 6.5, WP-19; amendment rows A44, A51, A53. Read in part: `database/saves.rs` (remove_property, the start of the save core), the `change_kind` arm of the dispatcher, two short ranges of `vault/activity.rs`. About 330 record lines and 170 code lines.
- Not read: the N2 execution record, the two review notes, the W sheet, the security findings note, restore and backup code, the open sequence, the editor's handling of unknown nodes, automation code, `acceptance.md`, `sessions.md`. Blue's statements about them are tagged [BLUE].
- The action map of 4.1 is by group. I did not check all 125 registered actions one by one.
- Stamp sizes and the save-time check cost are estimates or unknown. No time or usage estimate is given; usage inside the run is UNKNOWN.

### 17.2b Reading added for draft 3

Read at 9977e72: `vault/history.rs` (version body, when versions are due, thinning), `vault/store.rs` (open and journal options), `vault/mod.rs` (unlock and load, the start of re-key and backup), `vault/anchor.rs` (anchor load), about 230 code lines; the version plan's wave 1 tables (1.6 to 1.9), about 95 record lines. Not read: backup rotation rules, replaced-edit copies, where maintenance is first triggered, the A32 `.clf` path, restore code. The copy handoff of section 14 is a procedure on paper; no command in it was run, and how Git's worktree links behave on Ahmed's PC is stated from general knowledge, to be checked in step 4 of 14.3.

### 17.3 Working log (append-only; draft 2's entries kept)

- 2026-11-11T10:09:55Z Draft 2: read the full contract sections and three short code ranges; sections 0 to 12 written. Draft 1 untouched. Next: reread.
- 2026-11-11T10:10:22Z Reread done; cross-references checked (MI, C, U, IP, RG, D, OD); three corrected. Draft 2 handed over. Stopped. Study 2 not started.
- 2026-11-11T10:19:23Z Draft 3: built from draft 2 with Ahmed's answers, the source boundary, six clarifications and the history direction; new sections 13 to 16; read history, store, unlock and anchor code and the roadmap tables. Drafts 1 and 2 untouched. Changed and new parts reread; cross-references checked. Stopped; no further study started.
