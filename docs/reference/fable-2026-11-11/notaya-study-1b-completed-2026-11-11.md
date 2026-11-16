# Notaya Study 1b: validated changes, safe AI read/write/edit, receiving and updating hubs

<!-- CHECKPOINT:BEGIN -->
## CHECKPOINT
- Status: DONE
- Last finished: full reread of sections 1 to 14 and fixes applied
- Next unit: none. Study 2 is NOT started and needs Ahmed's own go
- Time: 2026-11-11T03:41:14Z
- HEAD: 9977e72 at /home/user/Notaya (same as Study 1a's baseline; no fetch made)
- Sections: 1 to 14 written and reread
- Lines read: code about 4,600; records about 280; extra 0 (ceilings: code 9,000; records 3,000; extra 3,000). Reading closed.
- Usage: observed by Blue at 03:23 UTC on 11 November: 5-hour 11%, weekly all models 17%, weekly Fable 22%. No meter access in the run: later values UNKNOWN
- Resume line: "Continue Study 1b from the CHECKPOINT in /mnt/project-files/notaya/studies/2026-11-study-1b-validated-changes-ai-hub-receive.md"
<!-- CHECKPOINT:END -->

Side: Ahmed (AE0101), relayed by Blue · Machine: cloud thread on Mohamed's account · Planning only; nothing built, run or tested.

## 1. Recommendation and limits in plain words

**The recommendation.** Put one checkpoint in front of everything that changes your notes. Today there are several doors (the editor, the database screens, import, the AI, undo, restore, the bin, delete for good) and each checks different things; the editor's saves are not checked at all, while the AI's are checked strictly. With one checkpoint, every change, whoever asks, is looked at the same way: is it allowed, is someone editing this, is the content well-formed, do ids and links stay sound, does it need your yes, and then it is saved completely or not at all, together with its history and its way back.

**What this gives you.**
- **The AI door stays safe as it grows.** Most of it exists and reads well on paper: levels, hidden notebooks, "you are editing, so the AI waits", confirmations, Stop. The gaps found are specific: an AI can overwrite a page it has not re-read; a revision number from before a lock can be mistaken for a current one; a preview says how many items, not what would change.
- **Hubs can be updated, not only added.** Each received item remembers where it came from and what was last received. On the next receive Notaya compares, field by field, what Notion has now, what you have now, and what came last time. Unchanged things are left alone, source changes arrive, your changes stay, and where both changed you keep yours and are shown the other. Something missing from the source is never deleted for you. An interrupted receive can simply be run again.
- **Ways back that are honest.** Page history, undo of an AI's or a receive's changes (finer than today: only the fields it changed), the bin, backups, and a "removed" list that makes removing a property reversible. What cannot be taken back always asks you first and says what will be gone: delete for good and emptying the bin in a Windows dialog, clearing the log with a typed word. One thing is automatic today and asks nothing: old history versions are thinned out over time (a decision left open in Study 1a).

**What it does not do.** It does not promise universal undo or a complete copy of Notion. It does not decide speed at hub size (study 2), which database or page features to build (studies 3 and 4), or screens. It does not review hostile files. Eight points ask to amend Study 1a, for example keeping numbers as they are stored today, and deciding whether the log may keep the title of something you deleted for good.

**How sure this is.** Facts about today's code were read at commit 9977e72 and carry file and line. They say what the code does on paper. Nothing was built, run or tested, and no design here is proven safe: each is a proposal to be reviewed, built and tested. Ten decisions for you are in section 13.

## 2. Baseline, evidence and conflicts

**What this is.** A planning study. Nothing was built, run, tested, benchmarked or security-reviewed. It continues the finished Study 1a (file `2026-11-study-1a-content-identity-format.md`, CHECKPOINT "DONE"), which is not changed.

**Baseline.** Local repository at commit 9977e72 (`git log`, no fetch), the same commit as 1a. Accepted product baseline is still V1.5.1; the 1.6 code is merged but not accepted and development is paused [REPORTED: STATUS, A56, via 1a section 2]. N2 was read from the local branch `origin/AE0101/v1-6-n2` at 7806d00.

**How to read the tags.** [VERIFIED: file:lines] means that code was read at 9977e72 (or the commit named) in this run; it says what the code does on paper, not that it was run. [REPORTED] is a project record. [PROPOSAL] is this study's design. [UNKNOWN] was not established. [NOT RUN], [ESTIMATE] and [ASSUMPTION] mean what they say. "1A CHANGE REQUEST" marks a place where this study asks for a change to a 1a proposal.

**Standing of 1a.** Its requirements R1 to R15, assumptions AS1 to AS6 and five questions are used here as the interface. They are 1a's proposals, not decisions Ahmed has accepted. Where this study depends on one of Ahmed's open 1a decisions it says so as a conditional branch.

**Conflicts found (not resolved silently).**

| # | Conflict | Sides | Handling here |
|---|---|---|---|
| K1 | Multi-action batches | Spec: `batch.run` runs up to 500 actions as one change set [REPORTED: spec WP-06e]. Code: `batch.run` is refused and the limit reported to the AI is 0 [VERIFIED: ai/dispatch.rs:214, 772] | Treated as "not available today". Section 7 says what a receive needs instead; decision 6 |
| K2 | Titles after permanent delete | 1a I8 and R15: nothing about a purged item keeps its title. Record and code: the activity log keeps "title at the time" [REPORTED: security-design SF3; VERIFIED: vault/activity.rs:62-67, 533-560] | 1A CHANGE REQUEST CR2; decision 4 |
| K3 | Purged relation targets | 1a: a reference to a purged item shows "purged". Code: a purged row is cut out of every relation cell first [VERIFIED: commands/database.rs:984-1030] | 1A CHANGE REQUEST CR3 |
| K4 | Exact numbers | 1a 5.1: exact numbers as decimal text. Code: Number cells are binary floating point [VERIFIED: commands/database.rs:167-176] | 1A CHANGE REQUEST CR1 |
| K5 | One write path | 1a AS1 assumed one validated write path. Code: the AI door validates documents, the editor door does not [VERIFIED: blocks/mod.rs:22-31; commands/notes.rs:385-418] | AS1 is false today; making it true is the core of section 5 |
| K6 | "1.5.1 can still open a 1.6 vault" | Carried from 1a (C6). New detail in 11.2 (CR6) | Supports 1a decision 1; still Ahmed's |

## 3. How changes happen today: verified paths and unknowns

### 3.1 The doors

| Door | Entry point | What is checked | All-or-nothing? | Way back today |
|---|---|---|---|---|
| Page editor | `save_page`: the whole document JSON, 500 ms after typing stops, with the revision last seen [VERIFIED: PageEditor.tsx:84-116; notes.rs:371-376] | Length only. The document is stored as given [VERIFIED: notes.rs:385-418] | One item, one transaction | In-editor undo (memory); page history, a version at most every 10 minutes of a person's typing [VERIFIED: vault/history.rs:210-219] |
| Database screen | `save_database`: the whole definition; `set_cells`: a patch of cells [VERIFIED: database.rs:529-537, 738-791] | Definition sanitised, relations validated, kinds cannot change in place; each cell cleaned against its property kind [VERIFIED: saves.rs:266-316; database.rs:158-200] | Row plus the other side of two-way relations in one write; automations then run as further writes | None stored for a person's cell or schema change (in-screen undo was not read [UNKNOWN]) |
| File import | `import_notes` [VERIFIED: import_notes.rs:176-270] | Converters build items; hostile-file handling was not reviewed here | Pictures first, then everything in one write; failure takes the pictures back | Delete the new notebook |
| AI action | `dispatch::call` to `prepare` to `atomic_action` [VERIFIED: dispatch.rs:192-490, 751-759, 1036-1090] | Fixed order: AI on, not stopped, pairing, level, parameter schema, targets, hidden, leases and claims, locked database, revision if sent, confirmation; documents validated by the Rust schema [VERIFIED: blocks/mod.rs:22-67] | Yes, per action, memory and disk, also on a crash inside the action [VERIFIED: vault/mod.rs:92-126] | Before-images for every item an AI request changes; `activity.undo` [VERIFIED: activity.rs:367-405, 852-957] |
| Automation | Fired by row changes and buttons inside the triggering request [VERIFIED: database.rs:748-756] | As the cores it calls (automation code not read [UNKNOWN]) | Inside an AI request: yes [VERIFIED: test name dispatch.rs:1230]. After a person's change: separate writes [derived: no atomic wrapper on the person path] | Shares the AI request's undo when triggered by an AI [VERIFIED: test name activity.rs:1203] |
| History restore | `history_restore` [VERIFIED: history.rs:257-268] | None on the restored body | One write | The pre-restore body is snapshotted first |
| Bin and restore | `trashed_at` set or cleared on the item and its descendants [VERIFIED: notes.rs:613-642; database.rs:943-981] | Parents in the bin are asked about | One write | Restore from the bin |
| Delete for good | `delete_bin_tree` after a single-use native approval [VERIFIED: notes.rs:691-712, 1484-1500] | Item must be in the bin; approval bound to the exact set | AI path: yes (inside `atomic_action`). Person path: several separate transactions [VERIFIED: same lines; state.rs:318-319 comment], so a failure part-way can leave a partly deleted tree [derived, NOT RUN] | None |
| Copy inside the editor | ProseMirror paste [VERIFIED: extensions.ts:38-93] | Duplicate block ids re-issued inside one document only | n/a | Editor undo |
| Row duplicate | `duplicate_row_core` [VERIFIED: database.rs:829-852] | Cells filtered; body copied unchanged | One write | Delete the copy |

### 3.2 What every door shares

- One storage primitive. `put_many` seals whole items and writes them with their history and activity rows in one transaction; `remove_many` does the same for removal [VERIFIED: vault/mod.rs:309-438]. It checks AI coordination, hidden notebooks and locked databases. It does not look inside an item.
- Revisions. A per-item counter in memory, bumped on every write, starting again at unlock [VERIFIED: vault/mod.rs:600-603; REPORTED: spec WP-02]. An AI's `expect_rev` is optional; a person's stale save is never refused: it lands, and if an AI wrote in between, the AI's value is kept as a "replaced edit" [VERIFIED: vault/mod.rs:535-544, 581-598].
- Leases and claims. A window that is editing holds a lease (3 s grace); an AI holds claims (2 minutes automatic, 30 minutes explicit). They stop AIs, never the person, and vanish at lock [VERIFIED: changes.rs:26-121].
- One change signal per command with ids, kinds and revisions, no text [VERIFIED: changes.rs:10-16; state.rs:314-334].

### 3.3 Gaps that matter for this study

| # | Gap | Evidence |
|---|---|---|
| G1 | The two main doors accept different documents. The editor can store JSON that the AI door then refuses to read or edit at all | [VERIFIED: notes.rs:385-418; blocks/mod.rs:22-31, 34-52] |
| G2 | No place checks an item's content or the identity rules for every door | [VERIFIED: vault/mod.rs:309-383] |
| G3 | A person's changes to cells, schemas and structure have no stored way back; only AI requests do | [VERIFIED: activity.rs:367-405] |
| G4 | History holds page and row bodies only (no cells, no origin); restoring validates nothing and re-issues no block ids | [VERIFIED: history.rs:44-91, 257-268] |
| G5 | One block id can end up in two pages: copy and paste across pages, and row duplicate | [VERIFIED: extensions.ts:44-50; database.rs:829-837] |
| G6 | Source tags: set by a second write after the action, exact match on app and id, no uniqueness, no source set, none on the file route | [VERIFIED: dispatch.rs:153-171, 818, 909-930; import_notes.rs at both commits] |
| G7 | A stale revision number from before a lock can equal a current one, because counters restart at unlock | [derived from VERIFIED: vault/mod.rs:447, 600-603; NOT RUN] |
| G8 | An AI that sends no `expect_rev` replaces whatever is there, including a person's saved edits it never read (the earlier body stays in history) | [VERIFIED: vault/mod.rs:535-544; activity.rs:319-327] |
| G9 | A preview tells the AI and the person which items and how many, not what would change | [VERIFIED: dispatch.rs:1050-1054] |
| G10 | No multi-action unit exists for an AI (K1); a hub refresh is thousands of independent actions with nothing tying them together | [VERIFIED: dispatch.rs:214, 772] |
| G11 | A file receive can only add a new notebook; a second import duplicates and warns by name | [VERIFIED: import_notes.rs:244-251] |
| G12 | Everyday AI confirmations are answered in the app window, which a compromised window could answer | [REPORTED: security-design A51] |

### 3.4 Still unknown after this run

What the editor does with a node type it does not know (library not present locally). Whether removing a property deletes its values from rows at once. The in-screen undo of the database views. The full pipe and wire code, restore, backup and automation internals. Whether a Notion file export carries block ids. Cost of everything at hub scale (study 2). All of the above as runtime behaviour: nothing was run.

## 4. Architecture options

All [PROPOSAL]. Four ways to make every change, from any door, obey the same rules.

**Option A. Checks at each door (today).** Each entry point validates what it knows; the storage primitive only seals and writes.

**Option B. One commit gate over whole items (recommended).** Storage stays "one item, one encrypted record". Every door builds a *change set* (complete new values of the items it wants to store, the ids it wants to remove, who is asking, which named action, which revisions it started from) and hands it to one gate. The gate works out what actually changed, checks shape, identity rules, concurrency and confirmation, and commits content, history, undo data, activity, reference lists and receive records in one transaction, or nothing.

**Option C. An operation log.** Every change is stored as a typed operation ("insert block", "set cell"); items are rebuilt from operations; undo is the inverse operation; merging is replaying.

**Option D. Merge-by-design data (CRDT style).** Text and structures are stored in a form that merges concurrent edits automatically at character level.

| Criterion | A. Door checks | B. Commit gate | C. Operation log | D. Merge data |
|---|---|---|---|---|
| Same rules for editor, import, AI, undo, restore, bin, purge | No: G1, G2 exist because of this | Yes, by construction: one function | Yes | Yes |
| Change to the editor | None | None to how it saves; it may now be told "refused" | Must emit operations instead of documents | Replaces the editor's document model |
| Change to storage and format | None | Small: new derived records; item records unchanged | Very large: a new format, every item migrated | Very large, plus a library dependency |
| Exact text, Arabic, numbers | As today | As today; the gate adds a check that nothing was altered on the way | At risk in the conversion of existing content | At risk; character-level merges can interleave right-to-left text badly |
| Undo and recovery | Per door, uneven | One model: before-images per change set, field-level | Elegant in theory; unbounded log, thinning is hard | History is implicit and hard to explain to a person |
| Receive and update | Each route re-implements matching | Routes differ only in how they build the change set | Natural | Natural but conflicts become invisible merges, which hides source-versus-local disagreements the person should see |
| Security review surface | Many places | One place, plus each door's builder | New engine | New engine and a dependency |
| Cost and risk | Zero now, growing | Moderate; mostly moving existing checks to one place | A rewrite | A rewrite |
| Dead-end risk | High | Low: a change set records its action name, so C can be layered on later | Low | Medium |

**Why B.** The code is already most of the way there: one storage primitive, one atomic wrapper, a compiled list of 125 named actions with level, confirmation and undo class [VERIFIED: registry; vault/mod.rs:92-126, 309-383]. What is missing is that the gate does not look inside items and that the editor and import doors do not pass through the same checks. B closes that without touching the format of pages. C and D solve a problem Notaya does not have (several people typing in the same text at once) at the price of a rewrite.

## 5. B1: the shared change contract

All [PROPOSAL] on the verified base of section 3. Words: an *item* is a notebook, section, page, row, database, sticky or settings record; a *door* is a way a change arrives.

### 5.1 The change set (the gate's only input)

| Part | Content | Who supplies it |
|---|---|---|
| Actor | Person, Automation, or an AI pairing | The trusted caller, never the request's parameters (as today [VERIFIED: actions/actor.rs]) |
| Door | editor, screen, import, ai, automation, undo, restore, bin, purge, system | The entry point, compiled in |
| Action | A name from the action registry | The entry point. Person doors get names too, so the log and the rules speak one language |
| Base | For each item to be replaced: the revision token the door last read (5.5) | The door. Required or optional by door (5.4 stage 3) |
| Puts | Complete new values of items | The door |
| Removes | Item ids to delete for good | Purge door only |
| Receive part | Optional: receive id, unit key, ledger entries, capability notes (section 7) | Receive only |
| Request id | Optional key for safe retry (5.6) | AI and receive |

A door never says what changed. The gate compares each put with the stored item and derives the *delta*: fields changed (title, body, look, placement, bin state, origin), cells changed by property id, schema parts changed (properties, options, views, templates, automations by their ids), block ids added, removed and kept, references added and removed. Rules are written against the delta, so a door cannot under-declare.

### 5.2 Classes of change

| Class | Examples | Way back | Confirmation |
|---|---|---|---|
| Ordinary | Typing, a cell, a rename, a move, a new item | History or before-image | None for a person; "ask if big" for an AI |
| Structural with possible loss | Remove a property, a view, a template; change a property's kind; restore an old version; update from a source | Before-image, or the soft removal of 8.3 | Ask (person: the existing dialogs) |
| To the bin | Bin an item or a container | Restore | Ask for an AI; containers get a safety backup first (as today) |
| Irreversible | Delete for good, empty the bin, clear the log, purge receive records; automatic thinning of old history | None in this vault | Person only, never an AI's to confirm: native dialog for delete and empty bin, typed word for the log (both exist). Thinning asks nothing today (1a decision 6) |
| Outside the contract | Format upgrade, security and password, AI access settings, restore of a backup | See 1a 6.6 | Person only; no AI action exists |

### 5.3 What the gate promises

1. All or nothing: content, history, undo data, activity entry, anchor, reference lists, receive records and the in-memory picture change together or not at all.
2. A refusal before commit changes nothing and says why with a fixed code and no note text.
3. The same change set gives the same answer whatever door built it.
4. Nothing is created that the change set did not contain, except the derived records listed in 1.
5. After commit: revisions move, one change signal goes out, claims are refreshed. A failure here never undoes the commit (as today [VERIFIED: state.rs:320-334]).

### 5.4 The stages, in order

The order is part of the contract: cheap and privacy-relevant refusals come first, so a caller without rights learns nothing from later stages (this is the existing dispatcher order, generalised [VERIFIED: dispatch.rs:192-490]).

| # | Stage | Rule | Refusal |
|---|---|---|---|
| 0 | Vault state | Unlocked; not opened read-only by the tolerant reader (1a 5.1); no migration running | `locked`, `read_only`, `migrating` |
| 1 | Actor and right | AI: access on, not stopped, pairing valid, action's level allowed. Person: none. Automation: inherits the request that fired it | `off`, `stopped`, `not_paired`, `level_too_low` |
| 2 | Footprint | Every item written or removed, every container above it (current and new), the other side of relations, rows affected by a schema change. For an AI, anything in a hidden notebook answers as if it did not exist | `not_found` |
| 3 | Concurrency | Leases and claims stop non-person actors. Base tokens: a person's stale save lands (person wins, the replaced value is kept); for AI, automation, receive, undo and restore a stale or missing base on a whole-item replace is refused | `person_editing`, `claimed_by_other_ai`, `item_changed` |
| 4 | Shape | Per kind: document schema with the carry-through rule (5.7); cells against property kinds; definition sanitised; sizes; origin fields bounded (1a R5) | `invalid`, `too_big`, `unsupported_content` |
| 5 | Identity and references | On the delta: ids never change (I2); a block id added here must not live in another document unless this change set removes it there (I3, 5.8); references have the one shape and name ids (I1); nothing is created to satisfy a reference (I5); a source id is unique in its source set (I6, section 7); restore and purge rules (I7, I8, section 8) | `invalid`, `duplicate_source` |
| 6 | Confirmation | Strength from the class and the footprint size, bound to the base tokens; asked in the app window or, for irreversible classes, the native dialog | `not_confirmed`, `cancelled` |
| 7 | Commit | One transaction (5.3 point 1). On any failure, disk and memory return to the state before stage 7 | storage fault code |
| 8 | After | Revisions, signal, claims | never |

Stages 0 to 5 are pure: they read and decide. That is what makes a *preview* honest: a preview runs 0 to 5 and reports the delta summary without committing (section 9).

### 5.5 Revision tokens (closes G7)

Today's counter restarts at unlock, so an old number can match a new one. Proposal: a token is the pair (session mark, counter), where the session mark is a random value made at unlock and never stored. A token from another session never matches. 1A CHANGE REQUEST CR5: 1a's "content stamp" on reference lists and this token stay two things; a stamp says "derived data matches content", a token says "you read the current version".

### 5.6 Retry without doubles

A request id (AI) or unit key (receive) may accompany a change set. The gate remembers, for the session (AI) or in the receive record (receive), which keys committed and what they answered. The same key again returns the first answer and changes nothing. A different change set under a used key is refused (`invalid`). This is what makes "the connection dropped, did it land?" answerable.

### 5.7 Documents: one schema, and the carry-through rule (closes G1; answers 1a AS1)

One schema definition (1a 5.1) is checked at stage 4 for every door, the editor included. Because the editor's saves were never checked, stored pages may exist that the validator refuses; turning the check on must not lock a person out of their own page. Rule:

- A node or attribute the schema does not know may be **carried through**: if it is byte-identical to one in the stored version of the same item, it may stay, move or be deleted. It may not be created or altered. This is also how a 1a "unknown block" and a stored original survive edits.
- A person's save that fails the shape check for another reason is not thrown away: the gate stores it as a history version marked "not applied" and tells the editor, so typing is never lost. [ASSUMPTION: rare; to be measured on real pages by a read-only audit before the check is switched on, Phase P2 in 11.1.]
- AI reads return carried-through nodes as opaque placeholders with their block id; an AI cannot address their inside.

### 5.8 Block ids across pages (closes G5; meets 1a I3)

The gate needs to know which document owns a block id. 1A CHANGE REQUEST CR8: the per-item derived list of 1a 5.2 (references found in the item, with a stamp) also lists the item's block ids; the in-memory map "block id to owner" is built from those lists at unlock (cost: study 2).

| Situation | Rule |
|---|---|
| Move inside one change set (AI `page.move_blocks` across pages) | Ids kept; a collision is refused (as today [VERIFIED: blocks/ops.rs:354-383]) |
| Cut in page A, paste in page B (two editor saves) | The editor flushes page A's save at the cut. When B's save arrives the ids have no live owner, so they are kept: a move |
| Copy in A, paste in B | The ids are owned by A: the gate re-issues them in B and re-points links inside the pasted material to the new ids |
| Row duplicate, template applied, page created from another | Always re-issued (these doors are copies by definition) |
| History restore | Ids in the old body that now live elsewhere are re-issued (I7); the rest return |
| The same id arrives from an AI as new content | Refused: an AI must not invent an existing id |

Residual edge: a cut whose source save has not landed before the paste (for example a crash between them) is treated as a copy; links to those blocks then point at the old page's history, and show as unresolved. Stated, not solved.

### 5.9 Worked example (invented)

Sara has the page "Trip plan" open and is typing. Her AI helper, paired at Edit, tries to append a checklist to it. Stage 3 finds her lease: `person_editing`, nothing changes. The helper waits, reads again (token T7) and sends the append with base T7. Sara meanwhile fixed a typo; her save landed and the page is at T8. Stage 3 refuses: `item_changed`. The helper reads again (T8), appends with base T8. Stages 4 and 5 pass: three new blocks, new ids, no references. Not big, so no question. Commit: the page, a "before AI" version of the body, a before-image for undo, one activity line "page.append by Helper", the page's reference list. Sara's clean editor reloads and shows the checklist. Later she clicks Undo on that activity line: the gate builds a change set from the before-image through the same stages, as door "undo".

## 6. B2: the AI's read, write and edit surface

The AI door is the most finished door today. This section keeps what exists, names what to change, and marks each line. "Exists" means [VERIFIED] in section 3 or the log; "change" and "add" are [PROPOSAL].

### 6.1 Levels

| Level | May | Exists? |
|---|---|---|
| Off | Nothing; no pipe | Exists |
| Look | Read visible, live content. A Look read never writes, not even block ids | Exists [VERIFIED: dispatch.rs:789-794] |
| Edit | Ordinary and structural changes and moves to the bin, each through the gate | Exists (85 actions) |
| Full | Also: look settings; ask for a permanent delete, which the person must confirm twice, the second time in a Windows dialog | Exists (3 actions) |
| Never | Format upgrade; security, password, keyfiles; AI access settings and pairing; clearing the log; restoring a backup; locking and unlocking; resolving a receive conflict against the person's version without the person; purging receive records or stored originals; retiring a source set | First six exist as "no such action": none of the 125 registered names does any of them [VERIFIED: registry list; REPORTED: spec WP-06g]; the last three are added |

A read that assigns ids is a write (1a R7): it already needs Edit and passes the write checks; it must also be refused in a read-only vault (stage 0) and must not create a history version or an activity line of its own (it changes no content). Add.

### 6.2 What an AI is told, and what it is not

- **Hidden notebooks.** Everything inside answers `not_found`; structured references to hidden items are stripped from reads; an AI cannot write a reference to one. Exists [VERIFIED: dispatch.rs:123-141, 399-401, 443-455]. Add for 1a's reference shape: a reference in a visible page whose target is hidden is returned without its cached label, in the same state as any other unavailable target (answers AS6).
- **One "unavailable" state.** For an AI, unresolved, purged and hidden targets are indistinguishable: `unavailable`. "In bin" is told only for visible items. This answers 1a question 5: telling an AI "purged" would confirm that something existed.
- **Notes are data.** Every read separates app facts (ids, tokens, kinds, states, counts) from note content, which is marked as untrusted text. Titles, source ids, origin fields, capability labels and stored originals are all on the untrusted side. Stored originals (1a 7.2) are returned only by an explicit read with a size cap, never inline. Errors never carry note text (intended and tested by name [VERIFIED: a test of that name at dispatch.rs:1652; saves.rs:29-37 maps database errors to a fixed code]; not checked line by line for every action).
- **Confirmation text is built by Notaya.** Titles shown in the in-window "Allow?" dialog get the same cleaning as the native dialog (control and direction characters stripped, 80 characters) [REPORTED: A51 does this for native prompts only]. Add. Residual limit G12 stays: a compromised window can answer everyday confirmations.

### 6.3 Stale writes and "the person wins"

| Case | Today | Proposal |
|---|---|---|
| Person is editing the item | AI refused, `person_editing` | Keep. Add a "retry after" hint in milliseconds |
| Another AI holds it | Refused, `claimed_by_other_ai` | Keep |
| AI sends an old token | Refused, `item_changed` | Keep, with the session-proof token of 5.5 |
| AI sends no token on a whole-item replace (`page.write`, `db.row.write_page`, `sticky.write`, property and view updates, `history.restore`, `activity.undo`) | Accepted: the AI can overwrite edits it never read (G8) | Refuse: a base token is required for whole-item replaces. Block and cell actions stay token-optional because they name what they touch and merge into the current item |
| Person saves over an AI change they had not seen | Person's save lands; the AI's value is kept as "replaced by your edit" | Keep. Show the person that it happened (section 9) |

### 6.4 Confirmations and big changes

Exists: None, Ask, Ask-if-big (more than a settable number of items, default 20), Strong (a typed pair of random words), plus the native dialog for permanent deletes; pending questions die at Stop, lock and after 5 minutes; the action is re-prepared after the answer and refused if any target moved [VERIFIED: rules.rs; confirm.rs:28-66; dispatch.rs:1066-1090].

Change:
1. Size is measured on the delta, not only on the item count: blocks changed, cells changed, rows touched by a schema change. One `page.write` that replaces 400 blocks is "big" although it is one item.
2. The question says what kind of change and whether it can be taken back, from the class table in 5.2: "Helper wants to change 14 cells in 6 rows of 'Plants'. You can undo this from the activity list."
3. Multi-action units (K1). Recommended: allow a bounded unit of up to 500 actions that commits as one change set under the strongest confirmation any step needs, with two limits: no irreversible action inside a unit, and the whole unit obeys the request size limit. A receive uses its own unit (section 7). If Ahmed prefers to keep units off, a receive by AI is still possible action by action but cannot be all-or-nothing per source page. Decision 6.

### 6.5 Stop, cancel, lock

Exists: Stop, lock and switching AI off raise an epoch; open connections are marked cancelled, pending confirmations are cancelled, new calls answer `stopped` or `locked`; an action that is executing either commits whole or not at all; claims and leases vanish at lock [VERIFIED: ai/mod.rs:39-40; pipe.rs:350-351; dispatch.rs:1071-1073; changes.rs:114]. Add: a unit in flight is one action for this purpose; a receive stops at the next unit boundary and its record says "interrupted" (section 7); after Stop the person sees what the AI finished and what it had only asked for.

### 6.6 What changes for the action list

| Change | Why |
|---|---|
| Every action gets a class from 5.2 in the registry, next to level, confirmation and undo | The gate and the dialogs read one table |
| Person doors get action names | One log vocabulary; rules testable per action for every actor |
| `find.by_source` takes a source set and returns at most one item | I6; today it returns a list [VERIFIED: dispatch.rs:818] |
| The source tag is part of the action's own write, not a second write | Uniqueness can be checked at stage 5; one activity line instead of two |
| `db.property.change_kind` becomes one gate operation | Today it is remove then add [VERIFIED: dispatch.rs:867-877]; see 8.3 |
| Receive actions (section 7): begin, unit, report missing, end | A receive is visible as such |
| New refusal codes: `read_only`, `migrating`, `duplicate_source`, `unsupported_content`, `needs_person` | Machine-readable causes (section 9) |

### 6.7 Worked example (invented)

A helper at Edit asks to remove the property "Old notes" from the database "Plants" (310 rows). Stage 2 widens the footprint to the database and its rows. Class: structural with possible loss. The app asks Sara: "Helper wants to remove the property 'Old notes' from 'Plants'. 212 rows have a value in it. It goes to the database's removed properties and can be put back for 30 days." She allows. The gate commits the soft removal (8.3). The helper then asks to empty the bin. It is paired at Edit: `level_too_low`. Raised to Full by Sara, it asks again: Sara types the two words in the app, then answers the Windows dialog that names the count and five titles. Only then does the purge run, as door "purge", through the same gate.

## 7. B3: receiving and updating hubs

All [PROPOSAL]. Today a file receive can only add a new notebook (G11) and an AI refresh has exact-match tags with no uniqueness (G6). The goal is one receive model with two routes, both ending in the same native items through the gate of section 5. It uses 1a's origin record, source sets, capability notes and receive ledger (1a 5.2, 7.2).

### 7.1 The pieces

| Piece | What it is | Lives |
|---|---|---|
| Source set | One hub as the person thinks of it ("my Notion workspace 'Garden Club'"). Has an id, the app, a name the person gave, and the receives that fed it. Two imports of the same hub into two source sets are two independent copies on purpose | An encrypted record, created only with the person's confirmation |
| Receive | One run: id, route (file or AI), start and end, state (open, complete, interrupted, cancelled), the known source scope (1a 7.3), totals | Ledger record of its source set |
| Unit | The atomic step: one source element (a page with its blocks, a row with its cells and body, a database definition) with its ledger entries and capability notes. One unit is one change set | Committed by the gate; its key is remembered in the receive |
| Origin | On each item, property, option, view and (where the route gives ids) block: source set, source kind, source id, times | 1a 5.2 |
| Baseline | Per field of an item with an origin: a stamp (hash) of the value as last received | Added to the origin record: 1A CHANGE REQUEST CR4 |

### 7.2 Matching and uniqueness (I6)

A source element is matched by (source set, source kind, source id) and by nothing else: never by title, path or position. At stage 5 the gate refuses a put that would give a second live or binned item the same triple (`duplicate_source`). The file route reads Notion's 32-hex ids from the export names, as the importer already does in memory [VERIFIED in the 1a run: notion_db.rs:62-68; not re-read here], and now stores them. Source ids are untrusted: bounded length and character set, never used as Notaya ids, never shown as trusted text (1a R5).

Which source set a file belongs to is the person's choice, with a suggestion: "This export shares 262 of its 270 pages with 'Garden Club' (last received 3 May). Update it, or add it as a separate copy?" An AI names the source set it is refreshing; opening a new one needs the person.

### 7.3 Field by field: three-way comparison

Fields are of three kinds.

- **Source-led with local edits allowed:** title, body, each cell of a property that has an origin, definitions of properties, options and views that have an origin, placement under a parent that has an origin.
- **Local only, never touched by a receive:** order among siblings, favourite, local icon and cover choices once changed, properties, views, rows and pages without an origin, column widths and other view preferences, where the hub sits in the tree.
- **Derived:** computed values, reference lists. Recomputed, never received. Captured source results of formulas and rollups are data (1a 5.1) and are source-led.

For each source-led field the gate compares three things: the incoming value, the local value, and the baseline stamp.

| Source changed since baseline? | Local changed since baseline? | Result | Recorded as |
|---|---|---|---|
| No | No | Nothing happens. No revision, no history, no activity | Unit counted "unchanged" |
| Yes | No | Updated; baseline moves | "updated" |
| No | Yes | Local kept | "kept local" |
| Yes | Yes, to the same value | Nothing changes; baseline moves | "unchanged" |
| Yes | Yes, differently | **Conflict.** Local stays. The incoming value is kept beside the item as a pending incoming value (body: as a history version marked "from source"; cell or title: in an encrypted side record). Baseline does not move | "conflict", listed for the person |

This answers 1a question 1: the local value wins by default and nothing incoming is lost. Only the person resolves a conflict ("keep mine", "take the source", for bodies "open both"); an AI may read the list and propose, never decide (6.1). Whether "local wins" is the right default is Ahmed's (decision 2); the alternative "source wins, local kept in history" uses the same data.

**Bodies.** On the file route, whose exports carry no block ids as far as is known [UNKNOWN], the body is one field: any local edit to a page whose source body also changed is a conflict on the whole body. On the AI route, where the AI declares source block ids, the comparison runs per block: blocks changed only at the source are replaced; blocks changed locally are kept; new source blocks are inserted after their source predecessor; local-only blocks stay where they are relative to their neighbours; a source block that disappeared is removed only if it is locally unchanged, otherwise kept with a note. Order differences are not merged: local order wins and a note says so. Block-level comparison is the costlier half of this section and can ship after body-level (11.1).

**Splits and merges of received blocks** (1a question 2). Split: the first part keeps the block id and the origin; the new part has neither. Merge: the surviving block keeps its origin; the absorbed block's source id goes on a small per-item list "absorbed into block X" so the next receive does not insert it again.

### 7.4 Missing is not deleted

An element in the baseline that is absent from a receive is never deleted or binned by the receive.

| What the route says | State set on the item | What the person can do |
|---|---|---|
| The scope was a declared complete inventory of that container, and the element is not in it | "No longer in the source" | See the list; bin them in one step (reversible); or keep |
| The element could not be read this time (the AI reports no access, a file is damaged) | "Not readable this time" | Nothing needed |
| No inventory | Nothing: absence proves nothing | |

The reverse case: an item the person binned or deleted for good. A binned item with that source id is not updated and not restored ("in the bin, not updated"). A purged one must not quietly come back. That needs the source set to remember that the person declined it, which touches 1a I8; see CR4 in 11.2 and decision 3.

### 7.5 Same thing twice, interruptions, restart

- **No-op.** A unit whose every field lands in the "nothing happens" rows writes nothing but its count. Receiving an unchanged hub again changes no item, revision or history.
- **Same file again.** If the handed-over files have the same hashes as an earlier receive of that source set, the person is told before anything is read further.
- **Replay.** A unit key is (receive id, source kind, source id). A key already committed returns its first result (5.6).
- **Atomic unit.** Content, origins, baseline stamps, capability notes, ledger entries and the unit key commit together or not at all (1a R13). Pictures are stored first and held, as today; an interrupted unit leaves at most unused pictures, which the existing clean-up removes [VERIFIED: import_notes.rs:189-209, 259-268].
- **Interruption** (cancel, Stop, lock, crash, a refused unit): committed units stay; the receive is marked "interrupted after N units" on the next open. Running the same receive again is safe because of the two points above: finished units are no-ops.
- **All or nothing for a first receive.** The file route's present behaviour (one transaction, "nothing was added" on failure) can be kept for a first receive into a new source set, where there is nothing to conflict with, as long as study 2 confirms the size is workable. Updates are unit by unit.
- **A person is editing.** A unit that meets a lease is skipped and retried at the end of the receive; if still held, the item is listed "being edited, not updated".
- **Taking a receive back.** Every unit stores before-images like an AI change; the whole receive is one family of change sets and can be undone from the activity list while its undo data is kept, item by item, skipping what was edited since (8.2).

### 7.6 Who the actor is; provenance, security, privacy

- A file receive is the person's action; an AI refresh is the AI's, at Edit, with a confirmation at the start that names the source set and, when declared, the count. A page an AI simply writes belongs to no source set and has no origin; that it was written by an AI is in the activity log and history (answers 1a question 4).
- The receive door trusts nothing it is given: files are hostile input (parsing them safely is the separate hostile-file review, not this study); origin fields and declared scopes are claims, recorded as claims.
- The ledger holds file names and source ids, and Notion file names contain page titles. It is encrypted with the vault, is listed for the person, and can be deleted per receive ("forget the details of this receive") without touching content. When an item is purged, its ledger lines are reduced to a count.
- Stored originals and pending incoming values are purged with their item (8.5).

### 7.7 Worked example (invented)

Sara received "Garden Club" from a Notion export in May: 40 pages, a database "Plants" with 30 rows. Since then she renamed the page "Notes" to "Meeting notes", added a property "Watered" (local, no origin) and changed the "Bed" cell of "Fig tree". In June she exports again and chooses "Update Garden Club". The export holds 40 pages (39 she already has and one new) and the 30 rows: 70 units. Sixty-six are unchanged. "Fig tree": the source changed "Height", she changed "Bed": both apply, no conflict. "Notes": the source title is still "Notes", hers differs: kept local. "Rose": the source changed "Bed" to "North", she had changed it to "East": conflict; "East" stays, "North" waits for her answer. One new page arrives and is created under its source parent. "Old tools" is not in the export; the export is a full export of the hub, so it is marked "no longer in the source" and stays. "Watered" is untouched. The report: 66 unchanged, 1 updated, 1 kept local, 1 conflict, 1 new, 1 no longer in the source; the new page holds 2 embeds, kept as stored originals. She cancels nothing; had the app crashed after unit 20, running the update again would have redone nothing and finished the rest.

## 8. B4: undo, history and recovery

All [PROPOSAL] unless tagged. There is no universal undo. The aim is that every change has a named way back with a stated limit, or asks first and says what will be gone (1a 6.6).

### 8.1 The ways back, and what each covers

| Way back | Covers | Limit | Today |
|---|---|---|---|
| Undo in the editor or screen | The person's own last steps | Until the page or app is closed | Exists for the editor; database screen not read [UNKNOWN] |
| Version history of an item | Title, body, look of pages and rows; **add** cells of rows and the definition of a database | Thinning rules (8.4) | Bodies only [VERIFIED: history.rs:44-91] |
| Undo of a change set (activity list) | Everything a request changed, any kind of item | While its before-images are kept; refused for parts that were purged | AI requests only [VERIFIED: activity.rs:367-405] |
| Bin | Whole items and containers | Until emptied | Exists for all kinds |
| Backups | The whole vault | Whatever was written after the backup is lost on restore | Exists |

### 8.2 Change-set undo, extended and made finer

- **Who gets before-images.** Today: AI requests (and automations inside them). Add: receives (7.5), a person's structural-with-loss changes (5.2), and a person's cell edits, coalesced per row in the same 10-minute bursts the log already uses. A person's typing in a page stays with version history. Storage cost of the person's cell before-images is the reason this is decision 5.
- **Field-level instead of item-level.** Today undo skips a whole item if a person touched it later or its hash changed [VERIFIED: activity.rs:896-913]. Proposal: the before-image records which fields the change set wrote and a stamp of each written value. Undo puts back exactly those fields that still hold the value the change set wrote, and reports the rest as skipped. Example: an AI set three cells of a row; the person later edited a fourth; undo reverts the three.
- **Relations.** A relation change is two rows in one change set; undo restores both sides in one change set. A target that is in the bin is linked again (it shows "in bin"). A target that was purged is not linked and counts as skipped (I5).
- **Created items** are moved to the bin by undo, never purged (as today).
- **Rules that stay** [VERIFIED: activity.rs:870-884, 945-959]: an AI may undo only its own change sets; the person may undo any AI's; settings undo never touches security or AI fields; a change set that includes purged content is refused with a plain message.
- Undo and restore are doors like any other: they pass stages 0 to 8, so hidden notebooks, leases, locked databases and shape rules apply to what they put back (1a R1).

### 8.3 Schema changes

Removing a property today removes it from the definition; whether its values leave the rows at once was not read [UNKNOWN], and a kind change through the AI is "remove, then add" [VERIFIED: dispatch.rs:867-877], which discards the old values. Proposal: **soft removal**. A removed property (and a removed view or template) moves to a "removed" list inside the database with its values left in the rows, hidden from views, formulas and AIs, for a retention time (suggested 30 days), then is purged by the maintenance pass with the same rules as history thinning. Putting it back is one small write. A kind change becomes: new property of the new kind filled by conversion, old property soft-removed. This makes the two most destructive database actions reversible without storing a before-image of every row, which at 200,000 rows would be very large [ESTIMATE; study 2]. Details of conversion belong to study 3.

### 8.4 History: content, restore, retention

- **Version body, second form.** 1a asked for a version marker and a frozen first form. The second form adds a row's cells (property id to stored value) and, for databases, the definition. A version stores values exactly as stored: no re-encoding of text, direction marks or numbers.
- **Restore** goes through the gate: shape check with carry-through (a version that contains unknown blocks restores them untouched); block ids that now live in another document are re-issued (I7); cells are restored only for properties that still exist with the same kind, and relation cells only to targets that still exist; the rest is listed as skipped. Calculated values are never restored, they are recalculated; captured source results are data and are restored.
- **Retention** [VERIFIED: history.rs:271-333]: the newest 20 versions of an item and "before AI" versions younger than 90 days are protected; everything is kept 7 days, then hourly to 30 days, daily to a year; soft cap 500 MB; no question asked. Add: "before receive" versions are protected like "before AI"; the "not applied" versions of 5.7 are kept at least 30 days. Whether thinning should tell the person is 1a decision 6.
- **Read-only vault** (tolerant reader): no history, no undo, no thinning, no clean-up, no id assignment on read (1a R8).

### 8.5 Permanent delete

What a purge removes today, in one transaction on the AI path: the items, their history versions, the undo bodies that held them, replaced-edit copies, and files nothing else uses; the activity chain is re-sealed and the anchor updated [VERIFIED: vault/mod.rs:384-438; activity.rs:533-572]. Add, so nothing private outlives its item (1a R15, I8): the item's origin and baseline, its stored reference and block-id list, capability notes, pending incoming values and stored originals, and its ledger lines (reduced to a count).

Three points need decisions or fixes:

1. **Titles in the log** (K2). The log keeps the title of a purged item by recorded design. Since the purge already rewrites the affected entries, blanking the title in them costs no extra pass. 1A CHANGE REQUEST CR2 and decision 4.
2. **Relations** (K3). A purged row is cut out of other rows' relation cells. That is defensible (a relation cell is data, and a dangling id helps nobody), but it is a silent change to other rows. Proposal: keep cutting, and say so in the confirmation ("also removes 12 links from other rows") and count it in the activity line. Links inside page bodies are not rewritten; they show as deleted.
3. **The person's purge is not atomic** (section 3.1). Fix: run it inside the same atomic wrapper the AI path uses. Prerequisite P2.

What cannot be recovered after a purge: anything in this vault. What still holds the content: older backups and exports, and any copy the person made elsewhere [REPORTED: security-design SF3]. The confirmation says so.

### 8.6 Exactness under undo and restore

Undo, restore, bin and receive move stored bytes; no door may tidy them (1a R6). Numbers: a Number cell is a binary floating-point value today (K4). Notion's own numbers are of the same kind [ASSUMPTION: general knowledge of that product, not checked in this run], so for received hubs "exact" can only mean "the same double, written back in its shortest exact form". 1A CHANGE REQUEST CR1: keep the Number kind as it is with that round-trip rule, never round on receive, and leave decimal-exact storage for a future separate kind if study 3 wants one (money, measurements). "Empty" stays distinct from zero: an empty cell is an absent key today [VERIFIED: database.rs:770-777].

## 9. B5: what the person and the AI see

All [PROPOSAL]; data and wording, not screen design (screens belong to studies 4 and 5).

### 9.1 States a person can meet

| State | Where | Says |
|---|---|---|
| Kept your version | A page or row the person saved over an unseen AI or receive change | "Helper also changed this. Your version was kept. See what Helper wrote." |
| Conflict | Receive report; the item | "Changed here and in Notion. Yours is shown. Compare / keep mine / take Notion's." |
| Refused | Any door | The reason in plain words; nothing changed |
| Not applied | Editor (5.7) | "This change could not be saved as it is. Your text is kept in the page's history." |
| Partial | Receive | "Stopped after 20 of 41. Nothing is half-done. Run it again to finish." |
| Unsupported, kept | Page | 1a's placeholder: "from Notion, not available yet, original kept" |
| Link states | Any link, mention, relation | In bin; deleted; not found |
| No longer in the source / not readable this time | Receive report; the item | With "move these to the bin" for the first |
| Read-only | Whole vault | 1a 6.5 wording |

### 9.2 Preview, confirmation, progress

- A preview runs stages 0 to 5 and reports, per item, the kind of change and sizes (blocks added, changed, removed; cells changed; rows affected; links cut), the class, and the way back. It never includes note text. Both the person's dialogs and the AI get the same summary (closes G9).
- A confirmation states who, what, how much, and whether it can be undone and for how long. Delete for good and emptying the bin are native and name counts and up to five cleaned titles (exists [REPORTED: A51]).
- A receive shows counts by outcome as it runs, can be cancelled at a unit boundary, and leaves a report the person can open later (the ledger).
- After Stop: a short list of what each AI finished and what was pending.

### 9.3 Machine-readable results for an AI

Success: change set id; for each item its id, new token and the fields changed; skipped parts with a code; notes (for example "ids re-issued", "carried through 2 unknown blocks"); whether undo is available. Refusal: a code from the fixed list, whether a retry can help and after how long, and which of the caller's own named targets caused it when that reveals nothing hidden. No refusal ever contains note text, titles or the existence of hidden items.

Codes: today's fifteen [VERIFIED: dispatch.rs:28-56] plus `read_only`, `migrating`, `duplicate_source`, `unsupported_content`, `needs_person` (a conflict or a class only the person may decide).

### 9.4 Honest limits to state in the product

No universal undo. A receive reports only against what it was handed. A copy of a hub is as complete as the route allows, and the report says what was not captured. An AI that can also control the mouse on the PC can click "Allow" in the app window; the typed words and the Windows dialog slow that down for the gravest actions only [REPORTED: spec WP-06f; A51].

## 10. Disposition of 1a's requirements, assumptions and questions

"Today" is the verified state at 9977e72. "Here" is where this study answers. "Open" names what still needs Ahmed or another study. Every "Here" is a proposal, not an accepted design.

| 1a item | Today | Here | Open |
|---|---|---|---|
| R1 every change path keeps I1 to I8 | No common place (G2). I3 broken by paste and row duplicate (G5); I7 not done (G4); I6 not checked (G6) | Gate stage 5 (5.4), block-id rules (5.8), restore and purge rules (8.4, 8.5) | Cost of the block-owner map: study 2. Depends on 1a decision 2 |
| R2 kind checklist enforced | No such list in code | The class column in the registry (6.6) plus a compiled per-kind table the gate reads; a kind without bin, history, purge, clean-up, search and export entries does not compile | Content of the table per new kind: studies 3 and 4 |
| R3 conversion keeps one id, no second body | No page-to-row conversion exists; import folds row pages into rows in memory [VERIFIED: import_notes.rs:215-241] | A conversion is one change set that changes the kind and keeps the id (stage 5 allows the kind change only for the conversion action). A53 item 2 (N3) builds on this, not on a second scheme | N3 not built [REPORTED] |
| R4 source id unique per source set | Not checked; lookup returns a list | Stage 5, `duplicate_source` (7.2); lookup returns at most one (6.6) | 1a decision 8 (store origins on the file route) |
| R5 source ids and origin fields untrusted | Length limits only, AI route only [VERIFIED: dispatch.rs:153-171] | Stage 4 bounds; untrusted side of every read (6.2); cleaned in dialogs | Exact limits: with the area 3 review |
| R6 no path alters stored text; cuts reported | AI door: exact by rule and test [VERIFIED: blocks/mod.rs test at 104]. Import: silent cuts (1a) | Gate promise (8.6) and capability notes on receive (7.5); a tested rule per door, since "nothing was altered" cannot be fully checked at run time | Lifting or reporting the importer's cuts: 1a Phase 0 item 10 |
| R7 a read that assigns ids is a write | Done for the AI door [VERIFIED: dispatch.rs:789-794] | Kept; plus refused in a read-only vault, and no history or log line of its own (6.1) | None |
| R8 read-only vault accepts no change | No read-only mode exists | Stage 0 for every door, including clean-up, thinning and id assignment (5.4, 8.4) | Depends on 1a decisions 1, 2 and 9 |
| R9 migration cannot be interrupted, started or confirmed by an AI | No migration action exists for an AI [REPORTED: spec WP-06g] | "Outside the contract" class (5.2); `migrating` refusal; Never list (6.1) | 1a's migration protocol itself is 1a's |
| R10 a cell change has a way back | AI: yes, whole-row before-image. Person: none stored | Cells in row versions (8.4); before-images for a person's cell edits and field-level undo (8.2) | Decision 5 (storage cost) |
| R11 destructive steps keep their confirmation boundary; none confirmable by an AI | Holds for permanent delete, AI on, pairing (native). Everyday confirmations are in-window (G12). Thinning has none | Classes (5.2); Never list (6.1); confirmations sized on the delta (6.4) | 1a decision 6 (thinning); G12 is an accepted limit unless Ahmed wants native prompts for more classes (decision 8) |
| R12 nothing is created to satisfy a reference; unresolved is reported, also to an AI | Select options must exist [VERIFIED: database.rs:179-183] and relation definitions are validated [VERIFIED: saves.rs:310-311]; whether a relation cell's target must exist was not read [UNKNOWN]; page links are text marks | Stage 5; link states (9.1); one `unavailable` state for AIs (6.2) | Mention and link nodes: study 4 |
| R13 a receive writes scope, ledger, notes and content together | No receive records exist | The unit is one change set (7.1, 7.5) | Decision 6 (units for AIs) |
| R14 stored originals are inert | None exist yet | Carry-through (5.7); explicit capped read on the untrusted side (6.2); never rendered or run | Rendering rules: study 4; hostile-content review separate |
| R15 purge removes origin, reference list, notes; tombstone and ledger keep no content | Purge removes content, history, undo bodies; log keeps titles (K2) | Additions in 8.5 | CR2 and decision 4; CR4 and decision 3 |
| AS1 one write path for editor, import and AI | False today (K5, G1) | Made true by the gate with the carry-through rule (5.7) | Read-only audit of real pages before switching the check on (P2) |
| AS2 source sets, ledgers, notes are created by the receive protocol | Nothing exists | 7.1 | None |
| AS3 the contract decides when a capability note is retired | Nothing exists | A note is retired when its target is purged, or when a later receive or a feature upgrade supersedes it; a person's edit never retires it (it describes the receive) | None |
| AS4 migrations run only with the person present | No AI action exists | Kept (5.2) | None |
| AS5 the reference index is updated with the content | No index exists | Part of the commit (5.3); the stamp makes it rebuildable | Study 2 for cost |
| AS6 hidden notebooks keep their rules; references into them | Structured references are stripped; label leak possible once 1a's cached labels exist | Label withheld, state `unavailable` (6.2) | None |
| Q1 who wins per field on re-receive | No update path | Local wins, incoming kept as pending (7.3) | Decision 2 |
| Q2 origin after split or merge | n/a | 7.3 | None |
| Q3 may a moved block stay in the old page's history | History is per item; no rule | Yes: I3 speaks of live documents; restore re-issues ids that now live elsewhere (5.8, 8.4) | None |
| Q4 source set of an AI-written page | Tag optional | None, and no origin (7.6) | None |
| Q5 what an AI is told about purged ids | No tombstones exist | `unavailable`, the same as unresolved and hidden (6.2) | None; follows from decision 4 of 1a only if tombstones are kept |

## 11. Prerequisites in order, 1A change requests, and boundaries

### 11.1 Prerequisite sequence (B6)

All [PROPOSAL]; nothing here is authorised. It continues 1a section 8 (items 1 to 10 there are not repeated) and says what in this study must come before what.

| # | Step | Blocks | Depends on |
|---|---|---|---|
| P1 | 1a's format decision and tolerant reader (1a Phase 0 items 1 to 3) | Everything below that adds stored fields: baselines, ledger, soft-removed lists, second history form, person before-images | Ahmed (1a decisions 1 and 2) |
| P2 | The commit gate for the doors that exist, with no new stored data: shape check on the editor door with carry-through, preceded by a read-only audit that counts how many stored pages the validator would refuse (run on throwaway and then, with Ahmed's word, on real data, reporting counts only); the person's purge made atomic; session-proof tokens; base token required for whole-item AI replaces | Any further AI or import work: until then the doors disagree (G1, G7, G8) | Nothing in 1a. Could be done first; it is the cheapest safety gain |
| P3 | Block-id rules at the copy doors (paste across pages, row duplicate, templates, restore) and the block-owner list | Block links (1.8), AI edits by block id being trustworthy across pages | P2; study 2 for the map's cost |
| P4 | Origins with source sets and uniqueness; the source tag written inside the action; `find.by_source` narrowed | Any AI refresh or import whose result is meant to be kept | P1; 1a decision 8 |
| P5 | Ways back: cells and definitions in history, soft removal, field-level undo, purge additions | Structural AI work on databases at scale; receive updates | P1 |
| P6 | Receive: units, ledger, baseline stamps, body-level three-way, missing handling, report | Updating a hub by either route | P4, P5; decisions 2, 3, 6 |
| P7 | Block-level three-way on the AI route; file-route update | Fine-grained refresh | P6; study 4 for block set |

The numbers show dependencies, not a calendar: P2 needs nothing from P1 and is recommended first (decision 10).

What the paused 1.6 work needs before it continues: P2, and 1a's items 1 to 4. Area 3 (AI edits) should not be released with G7 and G8 open.

### 11.2 1A CHANGE REQUESTS

Each is a request to amend a 1a proposal, with evidence and consequence. None is applied to the 1a file.

| # | 1a text | Request | Evidence | Consequence if accepted |
|---|---|---|---|---|
| CR1 | 5.1 "numbers that must be exact are stored as decimal text" | Keep Number as binary floating point with exact shortest round-trip; decimal-exact only as a possible new kind | [VERIFIED: database.rs:167-176]; the source's numbers are doubles [ASSUMPTION] | No migration of existing numbers; study 3 decides on a money or decimal kind |
| CR2 | I8, R15 "holds no title" | Decide whether the activity log may keep the title of a purged item; if not, blank it in the purge pass | K2 | Either 1a's rule is narrowed ("except the log's title at the time") or the purge changes; decision 4 |
| CR3 | 5.2 resolution state "purged" for every reference | Relation cells are cut at purge (and say so); "purged" applies to links in page bodies | K3 | No dangling ids in cells; one more line in the purge confirmation |
| CR4 | 5.2 origin record; I8 | Add per-field baseline stamps to the origin; add to the source set a list of declined source ids kept as keyed hashes, so a purged item is not received again | 7.3, 7.4 | Three-way comparison becomes possible; a trace that "something from this source was declined" stays. Without the list, purged items return at the next receive. Decision 3 |
| CR5 | 5.2 "stamp of the content" | Keep stamps for derived data; add the session-proof revision token as a separate thing | G7 | Clear vocabulary; no stored change |
| CR6 | Section 3, conflict C6 | Add: a 1.5.1 settings save drops the latch field; a 1.5.1 password change leaves anchor, history and log rows sealed under the old key, and the anchor is checked at every unlock, so the newer app would then most likely refuse to open the vault, not merely lose data | [VERIFIED: anchor.rs:151-166, 193-199; 1a's verified 1.5.1 re-key scope]; outcome [derived, NOT RUN] | Strengthens 1a decision 1; the PC check in 1a Phase 0 item 1 should watch for a failed unlock |
| CR7 | AS1 | Record as false today; the carry-through rule is the condition for making it true without locking people out | K5 | P2 |
| CR8 | 5.2 reference list | The per-item list also carries the item's block ids | 5.8 | I3 becomes checkable; cost to study 2 |

### 11.3 Boundaries: what this study hands to others

| Topic | Belongs to | What it receives from here |
|---|---|---|
| Parsing hostile files, archives, pictures | The separate hostile-file review (WP-17) | "The receive door trusts nothing"; units; that origin fields are claims |
| Cost at hub scale | Study 2 | Block-owner map and reference lists at unlock; delta computation on 240,000-character pages; unit size; first-receive-as-one-transaction limit; undo data volume; soft removal at 200,000 rows |
| Property kinds, formulas, rollups, conversion, decimal kind | Study 3 | Soft removal; cells in history; CR1; relation behaviour at purge |
| Block set, unknown-block screen, split and merge in the editor, mentions | Study 4 | Carry-through; block-id rules; link states |
| Where hubs, source sets and reports live in the workspace | Study 5 | Source set, receive report, "no longer in the source" list |
| Security review of the gate, the read-only mode, the receive actions | A review Ahmed orders | Stages and Never list as the things to attack |

## 12. Acceptance design (paper only)

Nothing here was built or run [NOT RUN]. Ten invented scenarios on the "Garden Club" fixture of 1a section 10, each with the result this design expects. A pass would support the design on paper-sized data only; it says nothing about speed or about hostile files.

| # | Scenario | Expected |
|---|---|---|
| 1 | Person edit against an AI. Sara types in "Trip plan" while a helper appends; then the helper retries with an old token; then with a fresh one | `person_editing`; then `item_changed`; then the append lands as one change set with a before-AI version. Sara's text is never altered. A token from before a lock never matches after it |
| 2 | Hidden target. The helper asks for a page in a hidden notebook, searches for its title, and reads a visible page that links to it | `not_found`; no search hit; the link comes back `unavailable` with no label. Nothing in any error or count reveals the page |
| 3 | Denied action. A Look pairing asks to write; an Edit pairing asks to empty the bin; a Full pairing asks to start a format upgrade | `level_too_low`; `level_too_low`; no such action. Nothing changed, nothing logged as a change |
| 4 | Duplicate and replayed receive. The same export is received twice into the same source set; one unit is sent twice by an AI; an item is created with a source id already used in that set | Second receive: every unit "unchanged", no revision moves. Replayed unit: first answer returned. Third: `duplicate_source` |
| 5 | Omitted source. A page is missing from a full export; a row is missing from a partial AI refresh; a page the person purged is in the next export | "No longer in the source", not binned; nothing recorded for the row; the purged page is not created again if the declined list is adopted (decision 3), otherwise it returns and the report says "new" |
| 6 | Block copy and move. Copy three blocks from page A to B; cut and paste one block from A to B; duplicate a row with a body; a link points at one of the moved blocks | Copies get new ids and their internal links point to the copies; the moved block keeps its id and the link still reaches it in B; the duplicate row shares no block id with its original |
| 7 | Relation and cell undo. A helper sets "Bed" on three rows (two-way relation) and a number cell; Sara then edits another cell on one of them; undo the helper's change set | Both sides of the relation return on all three rows; the number returns exactly; Sara's later cell stays; nothing reported skipped. If a target bed was purged meanwhile, that one link is skipped and counted |
| 8 | Unknown content. A page holds a stored original and a block type this version does not know; Sara edits a paragraph, moves the unknown block, then restores yesterday's version; a helper tries to change the unknown block | Edits save; the unknown block and the original are byte-identical after every step; the helper sees a placeholder with an id and its change is refused (`unsupported_content`) |
| 9 | Interruption and restart. A 41-unit update is cut by a lock after unit 20; the app is reopened; the update is run again | Twenty items updated, none half-written; the receive shows "interrupted after 20"; the rerun changes only the remaining 21; pictures stored for the unit in flight are cleaned up later; ledger and content agree |
| 10 | Irreversible purge. Sara empties the bin holding a row with history, undo data, an origin, a pending incoming value and links from other rows | Native dialog with counts, titles and "also removes 2 links from other rows"; after it, no record, history version, undo body, origin, note or pending value of the row remains; an undo of the change set that had edited it is refused with the plain message; the activity title follows decision 4; a failure part-way leaves everything as before |

Separate reports, as in 1a: correctness of each refusal; identity (ids and links after each step); exactness (byte comparison of text, numbers and pictures before and after); atomicity (state after a forced failure at each stage).

## 13. Decisions for Ahmed, and what later studies receive

None of these is taken by this study. Each has a recommendation and a consequence.

1. **Adopt the single commit gate (option B) as the line for all change paths?** Recommended: yes. Consequence: the editor's saves are checked for the first time; an audit of existing pages comes first so nobody is locked out of a page.
2. **On a re-receive, when both you and the source changed the same field, which is shown?** Recommended: yours, with the source's value kept beside it until you choose. The alternative (source wins, yours in history) is workable with the same data.
3. **May Notaya remember, as an unreadable mark, that you deleted a received item for good, so the next receive does not bring it back?** Recommended: yes. Consequence: a trace that something from that source was declined stays in the vault. If no: purged items return at the next receive and are reported as new.
4. **After "delete for good", may the activity log keep the item's title?** Today it does, by a recorded decision. Recommended: blank it at purge, so that nothing readable about a purged item remains in the vault. Consequence: old log lines for it read "deleted item".
5. **Store a way back for your own cell edits and database structure changes** (not only the AI's)? Recommended: yes for structure (through soft removal) and for cells in row history; the finer "undo this edit" for your own cells is optional. Consequence: more stored history; size to be measured in study 2.
6. **Multi-action units for an AI** (the spec planned up to 500 actions as one; the code refuses them). Recommended: allow bounded units without irreversible actions, and a dedicated unit for receives. Consequence if not: an AI refresh works item by item and cannot be all-or-nothing per source page.
7. **Require an AI to prove it read the current version before replacing a whole page, row body or sticky?** Recommended: yes. Consequence: an AI must read before it overwrites; block and cell actions are unaffected.
8. **Which confirmations use the Windows dialog?** Today: permanent delete, switching AI on, pairing. Recommended: add the start of a receive into a new source set and the resolution "take the source's version" for more than a set number of items. Consequence: one more click in rare cases; everyday confirmations stay in the app window with their known limit.
9. **Soft removal period** for removed properties, views and templates. Recommended: 30 days, then purged by maintenance, shown in the database's "removed" list.
10. **Order.** Recommended: P2 (the gate over existing doors) before any further AI or import work, then 1a's format decision, then P3 to P7. This is compatible with either order of area 3 and N2 that 1a left to you.

**What later studies receive from 1b.** Study 2: the list in 11.3 of everything whose cost is unknown. Study 3: soft removal, cells in history, CR1, purge behaviour of relations, the conversion action. Study 4: carry-through, block-id rules, link states, split and merge. Study 5: source sets, the receive report, the "no longer in the source" list. A security review: the nine stages and the Never list.

## 14. Reading log, usage log and unchecked limitations

### Reading map (saved at the start; ceilings, not quotas)

| Decision | Code to read (at 9977e72 unless stated) | Records |
|---|---|---|
| B1 change contract | actions/mod.rs, actions/registry.rs; changes.rs; vault/mod.rs (put, batch, remove, revisions, with_mut); state.rs (with_mut, signals); commands/notes.rs save path; src/ editor save | 1.6 spec parts for registry and change signals |
| B2 AI surface | ai/dispatch.rs, ai/rules.rs, ai/confirm.rs, ai/mod.rs, ai/wire.rs; ai/pipe.rs only for session, Stop and lock; native_confirm.rs outline | security-design AI sections; RV-N2 review |
| B3 receive and update | importers/plan.rs outline, commands/import_notes.rs, commands/notion_db.rs; dispatch.rs source and refresh paths; origin/AE0101/v1-6-n2 diff outline | A53; N1 and N2 records |
| B4 undo, history, recovery | vault/history.rs, vault/activity.rs (undo data), bin and purge in vault/mod.rs and commands; database/saves.rs (rows, cells), database/value.rs (numbers), relations.rs; vault/anchor.rs | spec WP-04, SF3 |
| B5 experience | wire.rs errors; confirm.rs; frontend conflict handling by grep | 1.6 UX records by grep |
| B6 order | none new | STATUS continuation; 1a sections 8 and 12 |
| 1a gaps | editor save and unknown nodes (src/); anchor.rs; duplicate, paste and block ids (src/ and blocks/); numeric cells (value.rs); N2 code | |

### What was not checked

- Nothing was built, run, tested, benchmarked or reviewed for security. [VERIFIED] means "read in the code at the named commit". No design here is proven; words like "closes" a gap mean "is designed to close it".
- Not read: the editor library's handling of unknown nodes (not present locally); the database screens and their undo; automation internals; the full pipe and wire code; restore, backup and settings commands; stickies; the RV-N2 findings text; whether property removal clears cells; whether a relation cell's target must exist.
- Several facts are taken from test names or comments, and are marked as such.
- Cited line ranges were spot-checked with a search after writing; a correction entry in the working log lists the ones that were wrong. Remaining ranges are function-level and may be off by a line or two.
- The statement that the source's numbers are binary floating point is general knowledge, not checked in this run (no web, no connector).
- Local repository state only (9977e72, no fetch). The GitHub API, the session counter, the Notion connector and the web were not used.
- Usage: no meter reading was available inside the run.
- Size: sections 1 to 13 are about 72 KB against a target of 35 to 55 KB. Nothing was cut to fit; the excess is mostly the matrix of section 10 and the worked examples.

### Usage log
- 2026-11-11 03:23 UTC [OBSERVED BY BLUE]: 5-hour 11%, weekly all models 17%, weekly Fable 22%, reset label 8:40 AM. Ahmed permits the whole weekly quota. No reading inside the run.

### Working log (append-only)
- Q0 2026-11-11T03:24:19Z. Ahmed's go relayed by Blue at 2026-11-11T03:23:30Z. HEAD 9977e72 = origin/main as known locally (`git log`; no fetch). 1a file read: CHECKPOINT DONE; R1 to R15, AS1 to AS6, five questions taken as provisional proposals. Read actions/mod.rs 1-90, actions/actor.rs, changes.rs 1-170 (270 lines). [VERIFIED: actions/mod.rs:11-56] every action has a Level (Look, Edit, Full, Never), a Confirm (None, Ask, AskIfBig, Strong), an Undo class (History, Bin, Previous, View, None) and a JSON schema; the comment says metadata is not permission. [VERIFIED: actions/actor.rs:3-8] Actor is Person, Automation or Ai{pairing_id,label}, supplied by the caller. [VERIFIED: changes.rs:26-121] coordination is session-only: per-window editing leases (3 s grace after editing stops), AI claims of 120 s automatic or 1,800 s explicit; an AI is refused with person_editing or claimed_by_other_ai; a person is never refused. [VERIFIED: changes.rs:10-16] the change signal carries ids, kinds, tree flag, "by" and revisions, no text.
- Q1 u1 2026-11-11T03:24:57Z. Read vault/mod.rs 31-146, 309-438, 447-630; state.rs 303-378; commands/notes.rs 354-440, 613-760, 1447-1506 (about 800 lines) @9977e72. [VERIFIED: vault/mod.rs:309-383] `put_many` is the one item-write primitive: whole items are sealed and stored in one SQLite transaction together with the journal (history and activity) rows; it checks AI coordination, hidden notebooks and locked databases through `ai_write_targets` (464-504), but does NOT validate an item's content. [VERIFIED: 384-438] `remove_many` deletes records, their history versions, purges undo data and cleans files in one transaction and refreshes the anchor. [VERIFIED: 92-126] `atomic_action` wraps one AI request: snapshot of all in-memory state, store begin, commit or rollback, also on panic. [VERIFIED: 535-544] revision check `expect_rev` applies to AI actors only. [VERIFIED: 581-598] `person_save`: a person's stale save is never refused; if an AI wrote in between, the AI's value is kept as a "replaced edit" in settings and the person's save wins. [VERIFIED: 600-619] every write bumps a per-item in-memory revision (reset at unlock per the test name at changes.rs:336) and, for an AI, takes an automatic claim. [VERIFIED: state.rs:310-334] `with_mut` runs as Person, drains the pending change signal after the closure even when a later step failed ("old cores perform several transactions"). [VERIFIED: notes.rs:385-418] `save_page_core` checks only lengths (title 300, page MAX_PAGE_CHARS) and for an AI that the page is not in the bin; the editor's document JSON is stored as given, with no schema validation: this answers 1a's UNKNOWN on the editor save path. [VERIFIED: notes.rs:1484-1500] permanent delete walks the bin tree and calls delete_database, delete_rows, remove_many and collect_garbage as separate steps, guarded by a single-use native approval "permit". Next: Q1 u2 registry and dispatcher.
- Q1 u2 2026-11-11T03:25:57Z. Read actions/registry.rs 1-40 plus a generated list of all 125 actions; ai/dispatch.rs 22-61, 80-340, 395-500, 741-960, 1036-1112; ai/rules.rs; ai/confirm.rs 1-120 (about 950 lines) @9977e72. [VERIFIED: registry.rs] 125 actions: 37 Look, 85 Edit, 3 Full (settings.set, bin.delete_forever, bin.empty); none Never. [VERIFIED: dispatch.rs:192-490] `prepare` runs in fixed order: AI enabled, not paused ("stopped"), pairing and token hash, action exists, level, schema (additionalProperties false), source tag shape, target discovery from named id keys, existence and bin rules, descendants for trash/move/bin, related databases for db actions, hidden check (answers not_found), leases and claims, locked database, expect_rev against every target, hidden references inside cells and definitions, confirmation strength from count. [VERIFIED: dispatch.rs:215] `batch.run` is refused; ai.limits.get reports batchActions 0 (767): there is NO multi-action transaction for an AI today. [VERIFIED: 1036-1090] `call`: prepare under read lock; a preview returns only ids, count and confirmation; confirmation waits up to 300 s in the app window (confirm.rs:28-45), Strong needs a random two-word phrase typed (54-66) and no wire message can answer (line 1); permanent deletes add a second native dialog; then under the write lock it re-prepares and refuses with item_changed if any target revision moved, and with stopped if cancelled or the epoch changed. [VERIFIED: 751-759] each non-Look action runs in `atomic_action`; a safety backup is made first for notebook.trash, section.trash and the first db.create. [VERIFIED: 786-791] a read by an Edit or Full pairing may persist legacy block ids (a write, authorised as one); a Look read never does. [VERIFIED: 818] find.by_source returns every live visible item with an equal tag; [VERIFIED: 153-171, 911-932] the tag is set AFTER the action, by a second put inside the same atomic action, with length limits 64 and 256 and no uniqueness check. [VERIFIED: 879-888] db.property.change_kind is remove_property followed by add_property. [VERIFIED: 28-56] wire error codes: locked, not_found, rate_limited, off, stopped, not_paired, pairing_refused, level_too_low, person_editing, claimed_by_other_ai, item_changed, not_confirmed, cancelled, too_big, invalid. Next: Q1 u3 blocks, history, activity undo.
- Q1 u3 2026-11-11T03:26:46Z. Read blocks/mod.rs 1-150; blocks/ops.rs 227-275, 354-384; blocks/ids.rs 40-70; ai/dispatch.rs 960-1035; vault/history.rs 40-92, 159-335; vault/activity.rs 60-125, 203-236, 265-440, 533-572, 758-800, 852-975 (about 1,050 lines) @9977e72. [VERIFIED: blocks/mod.rs:19-28, 51-63] the AI path parses and validates the whole document on every read and write; a page whose stored JSON the Rust validator refuses cannot be read or edited by an AI at all. Together with the unvalidated editor save (Q1 u1) this means the two doors accept different documents. [VERIFIED: ops.rs:261-272] AI insert re-issues ids of inserted nodes that clash inside the destination page only. [VERIFIED: ops.rs:354-383; dispatch.rs:980-989] an AI cross-page move keeps block ids and refuses a collision; it is two page writes in one atomic action. [VERIFIED: history.rs:44-91] history bodies exist for pages and rows and hold title, content, text, icon, cover only: no cells, no origin. [VERIFIED: history.rs:257-268] history_restore snapshots the current body, then applies the old body as stored: no validation, no block-id re-issue (1a I7 is not implemented today). [VERIFIED: history.rs:271-333] thinning: latest 20 per page and "before AI" versions under 90 days are protected; all kept 7 days; hourly to 30 days; daily to 365 days; soft cap 500 MB; no confirmation. [VERIFIED: activity.rs:313-327, 367-405] undo data is written ONLY for changes that originate from an AI request: a whole-item before-image (page and row bodies via history, everything else, cells included, as a purgeable encrypted row); a person's changes get an activity line but no stored undo. [VERIFIED: activity.rs:852-957] activity.undo restores per item, skips any item that a person touched afterwards, whose current hash differs from the recorded after-hash, or that is leased; created items are binned; only the AI's own change sets (or the person undoing an AI's) qualify; settings undo is limited to look fields. [VERIFIED: activity.rs:62-67, 203-218, 533-560] activity entries store each item's "title at the time"; permanent delete wipes undo bodies and re-chains hashes but leaves those titles in the log. This matters for 1a I8/R15. [VERIFIED: 758-797] clearing the log is person-only with a typed word. Next: Q1 u4 database rows, cells, numbers, relations.
- Q1 u4 2026-11-11T03:27:21Z. Read commands/database.rs 158-200, 305-318, 529-560, 725-796, 829-856, 943-1032; database/saves.rs 22-54, 167-185, 266-340; relations.rs 61-93 (about 430 lines) @9977e72. [VERIFIED: database.rs:167-176] a Number cell is stored as a JSON binary floating-point number; text input is trimmed and parsed to f64. This answers 1a's UNKNOWN and collides with 1a's "exact numbers as decimal text" rule (handled as a 1A CHANGE REQUEST in section 11). [VERIFIED: 305-316] a row's cells are one JSON text of property id to value, at most 20,000,000 characters. [VERIFIED: 763-791] `change_row` cleans each given cell against the property kind, syncs the other side of two-way relations and stores the row and the touched rows in ONE put_many; automations then fire as further writes in the same request. [VERIFIED: 829-852] row duplicate copies the body JSON unchanged, so both rows hold the same block ids: 1a I3 does not hold today. [VERIFIED: 984-1030] deleting rows for good first cuts their ids out of relation cells in every other row, then removes them: a relation never shows "purged", the link is silently gone. [VERIFIED: saves.rs:266-316] a database definition is saved whole; fixed fields are forced from the stored copy; a property's kind cannot change in place; definition is sanitised and relations validated before the write. [VERIFIED: saves.rs:41-53] AI patches to properties and views are shallow, with protected keys. Next: Q1 u5 import paths and N2.
- Q1 u5 2026-11-11T03:27:57Z. Read commands/import_notes.rs 1-30, 170-272, 304-345; importers/plan.rs 1-40 @9977e72; diff of import_notes.rs and notion_db.rs between 9977e72 and origin/AE0101/v1-6-n2 (7806d00, about 110 changed lines); head of that branch's docs/execution/2026-11-09-n2-notion-import-fixes.md (60 lines, record). [VERIFIED: import_notes.rs:1-18, 176-270] a file import always makes NEW notebooks. Pictures are stored first as separate writes and "held" in memory; then every notebook, section, page, database and row is saved in one `put_many` (all or nothing); on failure the stored pictures are taken back; cancel, lock and a 15-minute limit end it the same way. It runs as the Person actor through `with_mut`. [VERIFIED: 244-251] importing the same export twice makes a second notebook and adds a warning by notebook NAME: there is no update path, and no source identity is consulted. [VERIFIED at 7806d00: diff] N2 changes discovery, grouping and reporting (CSV-only databases, generated placeholders, ambiguous names kept separate with a warning) and still creates new content only. [REPORTED: N2 execution record on the branch] "No persistent model/storage field was added"; N2 result 752 passed, 3 failed (two old expectations that conflict with A53, one sandbox registry test); canonical rows are N3. So at both commits a file-received hub has no stored source identity and no way to be updated in place. Next: Q1 u6 the editor side (save, stale saves, unknown nodes, paste and duplicate) in src/.
- Q1 u6 2026-11-11T03:28:32Z. Read src/lib/changes.ts (60); src/features/notes/editor/extensions.ts 24-100, 240-264; src/features/notes/PageEditor.tsx 60-135; ipc.ts by grep (about 240 lines) @9977e72. [VERIFIED: PageEditor.tsx:84-112] the editor saves the WHOLE document JSON 500 ms after typing stops, one save after another, sending the revision it last saw. [VERIFIED: changes.ts:33-59] an editor reloads on a change signal only when it is clean; a dirty editor keeps its text and its later save wins (Rust side: person_save, Q1 u1). [VERIFIED: extensions.ts:38-93] the editor assigns block ids and re-issues duplicates inside ONE document, keeping the id at the original position; ids are written to and read from HTML as data-bid, so blocks copied in one page and pasted into another arrive with their ids: copy across pages can put one block id in two pages (1a I3 does not hold today on this path either). No page-duplicate command exists in commands/notes.rs (outline read). [VERIFIED: extensions.ts:257-264] bad JSON becomes an empty document. [UNKNOWN] what the editor (TipTap 3.32.0 per package.json; library not present locally) does with a node type it does not know; the usual behaviour of that library family is to fall back to an empty document, which the next save would then store [ASSUMPTION, NOT RUN]. Q1 done. Next: Q2 u1 anchor, native confirmation, Stop and lock.
- Q2 u1 2026-11-11T03:29:01Z. Read vault/anchor.rs 1-275; native_confirm.rs, ai/mod.rs, ai/pipe.rs, commands/ai.rs by outline and grep (about 330 lines) @9977e72. [VERIFIED: anchor.rs:151-190] at unlock: if an anchor record exists it is always checked, whatever the latch says; if no anchor record exists and the latch is true (or any anchor-table row exists) unlock fails with "journal anchor is missing"; only when the latch is false AND the anchor table is empty is the vault treated as "genuine legacy", fully verified and newly anchored. This closes 1a's UNKNOWN: with an anchor present and the latch false, verification still runs. [VERIFIED: vault/mod.rs:316; anchor.rs:150 comment] the latch is an ordinary settings field that "1.5.1 can read and ignore"; a 1.5.1 settings save would write settings without it (derived from 1a's verified fact that unknown fields are dropped; NOT RUN). [VERIFIED: anchor.rs:193-270] the anchor check compares the activity head and row count and replays membership certificates against history metadata; it does not cover item records or undo bodies' content. [VERIFIED: native_confirm.rs:1] native approvals are Rust-only and single-use; neither prompt nor answer crosses IPC. [VERIFIED: ai/mod.rs:39-40; pipe.rs:350-351] Stop, lock and disable bump an epoch that cancels pending confirmations and marks open connections cancelled; pipe.rs enforces request and response size limits and a rate limit (lines 345-346, 572, 586). [VERIFIED: commands/ai.rs outline] AI settings (enable, level, hidden notebooks, big-change switch, pause, resume, pairing answer) are person-only commands. Next: Q2 u2 records.
- Q2 u2 2026-11-11T03:32:12Z. Records read at 9977e72: docs/security-design.md 7-27 and 108-172 (A51, SF3); docs/plan/2026-11-08-v1-6-spec.md 120-141, 184-256, 373-400 (WP-02, WP-06, section 6.4); the A53 row by grep (about 220 long lines). Code: vault/store.rs 232-256 (80 lines). [REPORTED: security-design SF3] "The log retains its existing titles, actors, times, item ids, actions and counts" after a permanent delete: a recorded decision, and it matches the code read in Q1 u3. [REPORTED: same] "writing or re-keying that copy with an older version is not supported by this promise." [REPORTED: A51] a compromised main window can still answer everyday AI confirmations; only permanent delete, turning AI on and pairing have the native dialog. [REPORTED: spec WP-06e] `batch.run` was specified as up to 500 actions in one change set with one confirmation; [VERIFIED Q1 u2] the code refuses it. Recorded as conflict K1. [REPORTED: spec WP-06h] source tags were added so an AI-driven refresh "updates the same page instead of creating a new one"; file-route tags and re-import without duplicates were left for 1.9. [REPORTED: spec WP-02] revisions are reset at unlock by design ("an AI must read again after an unlock anyway"). [VERIFIED: store.rs:236-255] store transactions opened inside an action stay inside the outer action and roll back with it. Reading closed at code about 4,600 lines, records about 280 lines, extra 0: well under the ceilings; no gap needed more. Not read: RV-N2 findings text (titles known from 1a only), ai/pipe.rs and wire.rs in full, commands/vault.rs, restore.rs, backup.rs, automation.rs, stickies.rs, the database screens in src/. Next: Q3 writing.
- Q3 u1 2026-11-11T03:33:01Z. Sections 2 (conflicts K1 to K6) and 3 (doors table, gaps G1 to G12) written. Next: sections 4 and 5.
- Q3 done 2026-11-11T03:34:04Z. Section 4 (four options; B commit gate recommended) and section 5 (change set, classes, promises, nine ordered stages, revision tokens, retry keys, carry-through rule, block-id rules, example) written. Interim change requests named: CR5 tokens, CR8 block-id list. Next: section 6.
- Q4 u1 2026-11-11T03:34:43Z. Section 6 written (levels with a longer Never list, one 'unavailable' state for AIs, required base token for whole-item replaces, delta-sized confirmations, bounded units as decision 6, action-list changes). Next: section 7.
- Q4 u2 2026-11-11T03:35:33Z. Section 7 written (source set, receive, unit, baseline stamps; three-way table with local-wins default; body-level versus block-level; missing is not deleted; no-op, replay, interruption; 1a questions 1, 2 and 4 answered; CR4 named). Next: sections 8 and 9.
- Q4 done 2026-11-11T03:36:26Z. Sections 8 (five ways back, field-level undo, soft removal, history second form, purge additions, CR1 numbers, CR2 titles) and 9 (states, preview, results, limits) written. Next: sections 10 and 11.
- CORRECTION 2026-11-11T03:37:09Z. A check of cited line numbers with grep found that some ranges in the log entries above were a few lines off, because the reading commands printed several ranges at once (and one listing without blank lines). Corrected values, used in the numbered sections: ai/dispatch.rs batch refusal 214 and limits 772 (not 215, 767); id-assigning read 789-794 (not 786-791); change_kind 867-877 (not 879-888); source tag after the action 909-930 (not 911-932); cross-page move 981-992; blocks/mod.rs parse 22-31, read 34-52, write 55-67 (not 19-28, 31-48, 51-63); vault/activity.rs undo skip rule 896-913, undo ownership 870-884, settings limit 945-959; commands/database.rs empty cell 770-777; PageEditor.tsx 84-116; state.rs comment 318-319. The facts themselves are unchanged. Other ranges are function-level and may be off by a line or two at their ends.
- Q5 u1 2026-11-11T03:38:04Z. Sections 10 (matrix for R1 to R15, AS1 to AS6, Q1 to Q5) and 11 (P1 to P7, CR1 to CR8, boundaries) written. Next: sections 12, 13, 1.
- Q5 u2 2026-11-11T03:38:55Z. Sections 12 (ten paper scenarios), 13 (ten decisions, hand-over to later studies) and 1 (plain words) written. Next: full reread.
- 2026-11-11T03:41:14Z DONE: full reread finished (privacy, coherence, R and AS coverage, evidence tags, limitations); fixes applied; CHECKPOINT set to DONE. Nothing was built, tested, committed or pushed. Study 2 not started.
