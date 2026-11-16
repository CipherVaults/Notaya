# Notaya history and recovery: technical integration design

Planning document only. Nothing in it is approved, built, tested or measured.

## CHECKPOINT

- Status: DONE (sections 0 to 15 written; headings, placeholders, table formatting and two line citations checked; not reread line by line)
- Last finished: section 15
- Next unit: none. Waits for Ahmed's decisions (section 13). No further work is started automatically.
- Time: 2026-11-11T14:58:39Z (system clock, `date -u`)
- Interim checkpoints 2 and 3 are kept in place below as trace.

## 0. What this is, and on whose word

- **Request:** Ahmed (AE0101), relayed by Blue, 2026-11-11 11:18:27 UTC (message stamp): a deep technical integration design for history and recovery, a useful first version and its evolution, grounded in the actual code and the observed Notion screens. Planning-document output only.
- **Not authorised by that message:** development, tests, builds, benchmarks, copying, repository, Notion or roadmap edits. None was done. `D:\Notaya` and `D:\Notaya-lanes` were not touched; the code was read from the cloud copy of the repository at baseline `9977e72`, read-only.
- **Model ceiling:** Fable 5.2, Medium.
- **Earlier files stay unchanged:** Studies 1a and 1b, and plan drafts 1 to 3 in `/mnt/project-files/notaya/`. This file refines draft 3 section 13 (history retention) and the 11:15 UTC recommendation; where they differ, this file is the newer proposal and says so.
- **Words used:** *verified* means "I read this code at `9977e72`". It never means run or tested. Code-derived does not mean runtime-tested. *UNKNOWN* means not inspected or not decidable from what was read. *Proposed* means mine, not approved.

### 0.1 What Ahmed has accepted as a goal, and what he has not

| Item | State |
|---|---|
| History and recovery should grow to cover database cells and structure, with the Git idea as a user-facing analogy only | Accepted direction (11:06 UTC) |
| Database recovery is central, possibly more important than page history | Accepted direction (11:15 UTC) |
| Default retention is indefinite, with limits the user sets; no silent coarsening | Accepted direction (10:13 UTC) |
| 1.6 may raise the vault format, with a verified backup first and older versions refusing without change | Accepted planning direction (11:04 UTC), no amendment written |
| Not all of this must ship in 1.6 | Ahmed's clarification (11:18 UTC) |
| Which parts land in which version | **Not decided.** Section 10 proposes; it is not placement |
| Every new product choice in this file (section 13, HD list) | **Not decided** |
| WP-04 thinning text, G3 in A44, A53 "no storage format change" | Still binding contracts; this design needs amendments to them before any build (section 10.5) |

## 1. Summary of the recommendation

1. **Record at the storage door, not at each feature.** Every stored change in Notaya already passes through two functions, `put_many` and `remove_many`, where the old and the new item are both in hand inside one database transaction. A change record written there covers the person, the AI, automations, import, bulk actions and any door added later, by construction. Doors only add labels (what the action was called, who did it).
2. **Record both sides of each change.** For each changed field the record holds the value before and the value after. That allows walking backwards from today (inversion) and forwards from an earlier point (replay).
3. **Baseline by first touch, not by copying the vault.** The first time a pre-existing row or database is changed after history is switched on, the record also holds a full picture of it as it was. Rows never touched since then are their own baseline. Section 5 shows why this is enough and where it is not.
4. **Every stored row and database carries a small lineage stamp** naming its last change record and a fingerprint of its recoverable fields. A mismatch proves an unrecorded write and is recorded as a gap, never hidden.
5. **Restore is always: preview, explicit scope, one atomic commit or a refusal.** A restore is itself a recorded change, so it can be taken back.
6. **First useful version:** recording for all database writes, a per-database timeline, row history with before and after values, restore of a cell, of a row's properties, and of a removed property, and "take back this change" for any actor. Whole-database restore to a date, the read-only historical view and bookmarks come later on the same records.
7. **The recording foundation should ship with the format upgrade even if the screens come later**, because history cannot be filled in afterwards: any period without recording is lost for good.

## 2. Existing implementation map (baseline `9977e72`, verified by reading)

Paths are under `src-tauri/src/`. Line numbers are those of `9977e72`.

### 2.1 What exists

| # | Area | What the code does | Where |
|---|---|---|---|
| E1 | Store | Five tables, each `(id BLOB PRIMARY KEY, data BLOB)`: Records, Attachments, History, Activity, Anchor. Ids are stored in the clear; data is sealed. | `vault/store.rs` 202-229 |
| E2 | Store safety | `journal_mode = DELETE`, `synchronous = FULL`, `secure_delete = ON`, `temp_store = MEMORY`. | `vault/store.rs` 41-51 |
| E3 | Transactions | `transaction` uses `BEGIN IMMEDIATE`, or a nested `SAVEPOINT`, and rolls back on error or panic. `begin_action / commit_action / rollback_action` wrap a longer unit. | `vault/store.rs` 162-198 |
| E4 | Journal tables | `ensure_journal_tables` creates `history`, `activity`, `journal_anchor` at unlock of an older vault, after authentication, before any notice. | `vault/store.rs` 65-72; `vault/mod.rs` 239-257 |
| E5 | Sealing | `seal_item` pads then seals with associated data (format, vault id, table, id). Padding buckets: 256 bytes up to 4 KB, 4 KB up to 64 KB, 64 KB above. Item limit 32 MiB. | `vault/mod.rs` 26, 291; `crypto.rs` 129-142 |
| E6 | Unlock | All Records are decrypted into memory; then history index, activity head and anchor are loaded. In-memory revision numbers start again at each unlock. | `vault/mod.rs` 239-257, 250, 447 |
| E7 | The write door | `put_many`: checks, seals each item, `maintain_journals`, `prepare_journal(actor, items, [])`, then **one** transaction that writes the records and the journal, then `accept_journal`, then in-memory revision bumps. | `vault/mod.rs` 313-383 |
| E8 | The remove door | `remove_many`: verifies the activity chain, then one transaction deletes records, their history versions, purges undo bodies, re-links the chain, collects unreferenced attachments. | `vault/mod.rs` 385-438 |
| E9 | Other Records writes | Besides E7 and E8, Records rows are written at `vault/anchor.rs` 185 (not read beyond that line: UNKNOWN purpose, believed to be the settings flag), in `rekey` (`vault/mod.rs` 848) and in the loop at `vault/mod.rs` 921 (not read: UNKNOWN, believed format upgrade or backup). | grep of `Table::Records` |
| E10 | Actor and grouping | `run_as` sets the actor and opens one `change_set` id for the outermost call; nested cores and automations share it. `run_action` adds a compiled action name. Person commands enter through `AppState::with_actor`, which calls `run_as`. | `vault/mod.rs` 551-570; `state.rs` 310-331 |
| E11 | Person operations are several transactions | The comment in `with_actor` says so: "Some old cores perform several transactions. Publish durable partial writes even if a later step failed". Examples read: `set_cells` (row write, then automation writes), `delete_rows` (`put_many`, `remove_many`, `collect_garbage`), `delete_database` (same shape). | `state.rs` 317-319; `commands/database.rs` 738-756, 984-1030, 1054-1093 |
| E12 | AI operations are one unit | `atomic_action` clones the whole in-memory state (all items, revisions, history index, heads), runs the request, and on failure rolls the store back and restores the clone. | `vault/mod.rs` 92-126; `ai/dispatch.rs` 758 |
| E13 | Page history | `VersionBody {title, content, text, icon, cover}` for pages and rows. **Cells are not in it.** Person versions at most every 10 minutes, plus on leaving a page and before a restore. The index (all version metadata) loads whole at unlock; bodies are read lazily and checked against a stored hash. | `vault/history.rs` 19-91, 129-268 |
| E14 | Page restore | `history_restore`: write check, refuse if in the bin, snapshot the current state first, apply the body, `put`. The restore is a new save; old versions stay. | `vault/history.rs` 257-268 |
| E15 | Thinning | `thin_history` runs from `put_many` at most daily: keep everything 7 days, hourly to 30, daily to 365, newest 20 and AI before-versions (90 days) protected, 500 MiB soft cap. | `vault/history.rs` 15, 271-333; `vault/activity.rs` 238-254 |
| E16 | Activity | One hash-chained entry per write, with `seq`, time, actor label, `change_set`, action name, items with their title at the time. Capped at 100,000 entries and 365 days. Entries stay on disk until asked for; only the head is checked at unlock. | `vault/activity.rs` 21, 265-453, 799-807 |
| E17 | AI undo data | **Only for AI writes**, the entry keeps a whole-item earlier picture (body cleared, bodies live in history), sealed as a separate row in the Anchor table, with hashes of the earlier and later state. | `vault/activity.rs` 290-438 |
| E18 | AI undo | `activity_undo_plan`: only AI-owned change sets; refuses if anything in it was permanently deleted; skips an item if a person changed it later, if its current hash differs from the recorded later hash, or if writing is not allowed; a created item is moved to the bin, never deleted. Then one `put_many`. | `vault/activity.rs` 852-960 |
| E19 | Permanent delete and journals | Deleting for good replaces undo data with content-free tombstones and re-links the chain. | `vault/activity.rs` header, 533-572 |
| E20 | Attachment reachability | An attachment is kept while any current item, any history version's picture list, any undo body or any replaced-edit copy names it. Row cells are scanned as text. | `vault/mod.rs` 733-775 |
| E21 | Rows | `Row {id, db, title, content, text, cells, icon, cover, order, favorite, number, created_by, edited_by, source, trashed_at, …}`; `cells` is a JSON object text, property id to value, at most 20,000,000 characters. | `database/types.rs` 480-510; `commands/database.rs` 305-316 |
| E22 | Row writes | Create: `add_row_inner` (row, database for the ID counter, and reciprocal rows, one `put_many`). Change: `change_row` (row and reciprocal rows, one `put_many`; returns changed property ids). Duplicate, reorder (rewrites every listed row), bin, restore from bin: one `put_many` each. | `commands/database.rs` 593-887, 943-982 |
| E23 | Relations | `relation_sync_diff` makes the other side list or drop this row; a "one row only" other side keeps just the newest link, silently dropping the older one. A reciprocal pair may span two databases. | `database/relations.rs` 60-143 |
| E24 | Row and database delete for good | Cuts links out of other rows, removes rows and sub-pages. Deleting a database also removes relation and rollup properties in **other** databases and strips their cells. | `commands/database.rs` 984-1093 |
| E25 | Schema | `Database` holds `props`, `views`, `templates`, `automations` and bookkeeping (`next_id`, `updated`); limit 30,000,000. `remove_property` removes the definition from `props` only. Kind cannot change in place; the AI's `change_kind` removes and re-adds the same property id. | `database/types.rs` 374-422; `database/saves.rs` 167-184, 266+; `ai/dispatch.rs` (change_kind arm) |
| E26 | Automations | Fired by commands after a row write, as separate writes inside the same change set; depth 3, 200 actions, 50 new rows per run. Not fired by `put_many` itself. | `commands/automation.rs` 26-28, 113-219; `commands/database.rs` 749-753 |
| E27 | Import | One `put_many` for the whole import, inside one `with_mut`. | `commands/import_notes.rs` 236-256 |
| E28 | Rekey | Re-seals Records, Attachments, History, Activity, Anchor in one transaction; the table list is written out by hand. | `vault/mod.rs` 842-850 |
| E29 | Screens | `src/features/notes/HistoryDialog.tsx` and `src/features/settings/ActivityPanel.tsx` exist. Their contents were **not read**: UNKNOWN. | file listing |

### 2.2 Findings that shape the design

- **HF1. One capture point exists already.** `prepare_journal` (E7) sees old and new for every item of every write, from every door, in the transaction that stores them. Nothing comparable has to be added door by door.
- **HF2. Cells have no history today.** Page history stores title and body only (E13). A cell change by a person leaves an activity line and nothing to restore from. Only an AI change keeps an earlier picture (E17), and only within the activity limits (365 days and 100,000 entries, whichever comes first).
- **HF3. A removed property probably leaves its values behind in the rows,** because `remove_property` edits only the definition (E25) and `change_row` carries unknown keys forward (`m = old.clone()`, `commands/database.rs` 766-767). Whether any later save strips them was not traced: UNKNOWN. The design must not depend on this accident; it turns it into a stated rule (section 4.6).
- **HF4. Person operations are not atomic today** (E11). History must tell the truth about what was stored, step by step, and may not pretend a multi-step operation was one unit.
- **HF5. The AI's all-or-nothing wrapper copies the whole vault in memory per request** (E12). Its cost on a large vault is UNKNOWN and not measured. Using it for every person command is therefore not a free choice.
- **HF6. Automatic thinning is live in the code** (E15) and contradicts Ahmed's "indefinite by default" direction until WP-04 is amended. It runs from the write door itself.
- **HF7. The history index loads whole at unlock** (E13). A change log that did the same would make unlock grow with every edit. The activity log's pattern (head only, entries on demand, E16) is the one to follow.
- **HF8. Record ids are visible in the file** (E1). Any id built from a database id or a counter would leak structure. Existing journals use random ids; the new records must too.
- **HF9. The "one row only" relation rule silently drops a link** (E23). A restore that re-adds a link can therefore remove another one. That is a collateral edit the restore protocol must surface or refuse.
- **HF10. Every small record costs at least one 256-byte padding bucket** (E5), so a one-cell change costs a few hundred bytes of history, and bulk changes pack far better per change than single ones.

### 2.3 Reuse, extend, new

| Part | Reuse as is | Extend | New |
|---|---|---|---|
| Transaction and rollback (E3, E7) | Yes | | |
| Capture hook `prepare_journal` / `Journal::write` / `accept_journal` | Yes, as the place | Add the change record to the prepared journal | Field-level comparison of rows and databases |
| Actor, `change_set`, action name (E10) | Yes | Action names for person, import and automation doors (today only the AI path supplies one; others record "items.changed") | |
| Canonical JSON and item hash (`vault/activity.rs` 146-194) | Yes | A second hash over recoverable fields only | |
| Hash chain, head, anchor (E16) | Pattern and anchor record | Anchor also commits to the change head | Change chain and per-database stream heads |
| Page versions (E13, E14) | Yes, for bodies | Referenced from change records | |
| AI undo (E17, E18) | Unchanged in the first version | Later: served from the change records | |
| Purge with tombstones (E19) | Pattern | Purge reaches change records | Per-row back-pointers so purge does not scan |
| Attachment reachability (E20) | Yes | One more root: attachments named in change records | Reference counts record |
| Rekey and backup (E28) | Yes | Add the new table to both hand-written lists | |
| Relations (`relation_sync_diff`), cell cleaning (`clean_cell`) | Yes, called by restore | | Conflict detection before the call |
| Thinning (E15) | | Becomes policy-driven; no automatic default | Retention policy record |
| Screens (E29) | UNKNOWN | UNKNOWN | Database timeline, row history, restore preview |

## 3. Architecture alternatives

| | Alternative | Strength | Why not, or when |
|---|---|---|---|
| HA-1 | **Whole-item versions**: extend page versions to hold cells and whole databases | Smallest new idea; restore is "put this copy back" | One cell edit copies the whole row; one schema edit copies up to 30 MB; no field-level before and after for the timeline; a whole-database point in time needs one lookup per row; index-at-unlock problem (HF7) grows. Rejected as the base; kept for page bodies |
| HA-2 | **Earlier-value-only log** | Small | Walks backwards only; one missing record silently corrupts everything older; cannot be checked against the present; cannot replay. Blue's objection is correct. Rejected |
| HA-3 | **Two-sided change records with first-touch baselines and lineage stamps** | Field-level timeline; backward and forward; gaps are detectable; cost follows what changed, not vault size; fits the existing hook and chain | Point-in-time cost grows with the number of changes since that time until checkpoints are added (later phase). **Recommended** |
| HA-4 | **Git-like object store** (every row an object, every database state a tree) | Any point in time is one tree; natural branching | Each cell edit rewrites a row object and the path to the root; a 200,000-row tree; padding makes small objects costly; far from the current one-record-per-item layout. Not now. HA-3's later checkpoints can grow toward it if ever needed |
| HA-5 | **Whole-vault copies** (`VACUUM INTO`, E3 `backup_to`) on a schedule | Simple, already exists, independent of record logic | All-or-nothing; a 1 GB class vault costs 1 GB per point; no "what changed". Keep as the safety net before the upgrade and before large restores, not as history |

**Recommendation: HA-3, with HA-1 kept for bodies and HA-5 kept as the safety net.**

## 4. Data model

All records are sealed like every other record (E5), with the table and id bound into the seal. Every record carries `v` (record version, starting at 1); a reader that meets a higher `v` must refuse to interpret it and must not rewrite it.

### 4.1 Where it lives

- **Proposed:** a sixth table, `Changes` (SQLite table `changes`), same `(id, data)` shape. A separate table keeps the hand-written table lists honest (rekey, backup, verify) and binds records to their table through the seal.
- The table is created **only inside the agreed format upgrade step**, never silently at unlock (this corrects the E4 behaviour for the new table; draft 3's open sequence stands).
- Ids are 16 random bytes; the first byte is a type tag, as History and Anchor already do: `0x10` change transaction, `0x11` overflow part, `0x12` image, `0x13` stream head, `0x14` global head, `0x15` attachment references, `0x16` retention policy.
- Alternative considered: reuse the History table with new tag bytes. Fewer code lists to touch, but mixes two retention regimes in one table and makes "how big is page history" harder to answer. Not recommended; either works.

### 4.2 Identity and order

| Thing | Identity | Notes |
|---|---|---|
| Change transaction | `txn` = the record id (random) | One per `put_many` or `remove_many` that touches a row or database |
| Order | `cseq`: one counter for the vault, increased under the session lock at commit | The session lock already serialises all writes (`state.rs` 314-316), so `cseq` order is commit order |
| Logical operation | `change_set` (exists, E10) plus `step` (1, 2, 3 … within it) | Several transactions of one person command share it |
| Time | `at` in milliseconds, from the system clock | Display only. Order comes from `cseq`, never from the clock, which can jump |
| Actor | `person`, `automation`, `ai:<label>` plus the pairing id for AI (exists) | Limits in 11.4 |
| Door | `action` (compiled name) and `origin` (`normal`, `restore`, `take-back`, `import`, `upgrade`, `repair`) | Supplied by the door; absent means `items.changed`, as today |
| Row, database | Their existing ids, never reused | A restore never mints a new id for an existing thing |
| Property, option, view, template, automation | Their existing ids inside the database | Names are labels, never keys |
| Schema version | `sv`: a counter inside each database, increased on every recorded structure change | Lets a row change say which structure it was made under |

### 4.3 The change transaction record

```text
ChangeTxn (tag 0x10)                      -- one per stored write
  v: 1
  cseq: u64
  prev: id            -- previous ChangeTxn in the vault (chain)
  prev_hash, hash     -- chain over the header core (see 8.4)
  at: i64
  actor: string, owner: pairing id?      -- as activity
  change_set: id, step: u32
  action: string, origin: enum
  origin_ref: { txn?, plan? }            -- for restore / take-back: what was restored
  activity_seq: u64?                     -- cross-reference; the activity entry may be pruned later
  parts: [ DbPart ]                      -- one per database touched
  more: [ id ]                           -- overflow parts (tag 0x11) when ops do not fit one record
  ops_hash: hash of all ops              -- not part of the chain (see 8.4)

DbPart
  db: database id
  db_prev: id?        -- previous ChangeTxn that touched this database (stream back-pointer)
  dseq: u64           -- position in this database's stream
  sv_before, sv_after
  ops: [ Op ]

Op (one per entity touched)
  kind: row.create | row.change | row.bin | row.unbin | row.purge
        | db.create | db.schema | db.layout | db.bin | db.unbin | db.purge
        | gap
  id: entity id
  ent_prev: id?       -- previous ChangeTxn that touched this entity (entity back-pointer)
  before_hash?, after_hash?              -- recoverable-field fingerprints (4.5)
  anchor: image id?   -- full earlier picture, on first touch or after a gap (4.4)
  fields: [ { f: field or property id, b: value?, a: value? } ]   -- only what changed
  body: { before_version?, after_version? }   -- page-version ids when a snapshot was taken (E13)
  elems: [ { part: prop|option|view|template|automation, id, b: json?, a: json? } ]  -- db.schema / db.layout
  pair: { db, row, prop }?               -- reciprocal relation edit this op belongs with (4.7)
  cosmetic: bool      -- order, favourite, layout: hidden from the default timeline
  purged: bool        -- values removed by a permanent delete; ids and kind stay
```

Field classes for a row:

| Class | Fields | Recorded | In the fingerprint |
|---|---|---|---|
| Recoverable | `title`, each cell by property id, `trashed_at`, `icon`, `cover`, `number`, `db`, `source` | Before and after | Yes |
| Cosmetic | `order`, `favorite` | Before and after, compact, `cosmetic` | No |
| Body | `content` | By page-version reference only | No (the version has its own hash) |
| Derived or volatile | `text`, `updated`, `edited_by`, the lineage stamp itself | No | No |

For a database: `props` and their options are **schema** (recoverable, fingerprinted, per element); `views`, `templates`, `automations`, `title`, `icon`, `description`, placement are recorded per element; view changes are `db.layout` (cosmetic); `next_id`, `updated`, automation `last_run`, template `repeat_last` are bookkeeping and are not recorded. Rule: a restore never lowers `next_id`.

When the comparison code cannot classify a database change (a field it does not know), it must not guess: the op carries a full earlier image instead (`anchor`) and is marked `db.schema` with no `elems`. Completeness never depends on the comparison being clever.

### 4.4 Images (first-touch baselines)

```text
Image (tag 0x12)
  v: 1
  of: entity id, kind: row | database
  at_cseq: u64               -- state just before this cseq
  data: row without content/text  |  database JSON
  hash: fingerprint of data
```

An image is written:
- the first time an entity that existed before recording began is changed (`ent_prev` is empty and the op is not a create);
- when a gap is detected (4.5);
- for a row created singly (person, AI, automation): the op's `fields` list all its values as `a`, which serves the same purpose going forward;
- **not** for rows created in bulk (import, more than a threshold per transaction, proposed 500): the op records id and fingerprint only (`row.create`, no values). The row is fully in the record of what was created, and its first later change takes an image. This avoids doubling a 1 GB class import. Consequence, stated plainly: for such rows, the state between creation and first change can be rebuilt only by walking back from the present, not forward from the creation record. Technical decision HT-3.

### 4.5 Lineage stamp and gaps

Each stored row and database gains one small field:

```text
lin: { t: id of the last ChangeTxn that touched it, h: fingerprint after that change }
```

- Absent stamp means: untouched since recording began. Such an item is its own baseline.
- On every write, capture computes the fingerprint of the old item. If the old item has a stamp and `old.lin.h` differs from that fingerprint, a write happened that was not recorded. Capture then writes a `gap` op with a full image of the old item, and continues. The gap is kept and shown; reconstruction across it is refused for that entity (section 5).
- On restore and on a "check history" command, the same comparison runs for every entity in scope.
- Each database stream head keeps completeness:

```text
StreamHead (tag 0x13, one per database; its id is stored once in the database item)
  db, last: id, dseq, count
  complete_since: { cseq, at, build }
  gaps: [ { from_cseq, to_cseq, entity?, reason } ]   -- reasons: enablement, mismatch, repair, trimmed
  first: { cseq, at }                                  -- oldest record still kept (moves when trimmed)

GlobalHead (tag 0x14, single)
  last: id, cseq, hash; first: { id, cseq, base_hash }
```

### 4.6 Removed properties: a stated rule instead of an accident

- Removing a property moves its full definition to a new list in the database, `removed_props: [{ prop, removed_at, by_txn }]`, and **leaves the cell values in the rows**, hidden from every view, filter, formula, export and AI read.
- "Remove for good" is a separate, confirmed step: it strips those values from every row and purges them from change records (8.3).
- Restoring a removed property puts the definition back with the same id; the values are already there.
- This replaces HF3's unverified behaviour with a tested rule and avoids copying up to 200,000 values into the log at removal time.
- The same list handles `change_kind` (E25): the old-kind definition is kept in `removed_props` and the old-kind values are kept aside per row under a reserved key, so "change kind back" is a restore, not a guess. This is new behaviour and a product choice (HD-5).

### 4.7 Relations across rows and databases

- A two-way relation edit always produces paired ops in the same transaction (E22, E23): row A gains X in property P, and row X gains A in the back property. Capture marks both with `pair`, derived from the schema, not from the door.
- If the pair spans two databases, the transaction has two `DbPart`s and appears in both timelines, each showing its own side and naming the other.
- A link dropped by the "one row only" rule (HF9) is a third paired op and is recorded as such, so the timeline can say "this also removed the link from Y".

### 4.8 Attachments

- A cell value or image that names an attachment makes that attachment reachable for as long as the record is kept. A small counted record (tag `0x15`: attachment id to number of references from change records and images) is updated in the same transaction; garbage collection (E20) adds it as one more root. Scanning the whole log at each collection is ruled out by HF7.
- How a Files cell names its attachment was not traced (`PropKind::Files`, `database/types.rs` 53): UNKNOWN. The existing collector treats cells as text and looks for ids, which the new code can reuse.
- Cost stated plainly: a 900 MB file removed from a cell stays on disk while history that names it is kept. The retention screen must show this (11.5).

### 4.9 Indexes: what answers which question

| Question | Path | Cost |
|---|---|---|
| Newest changes in a database | Stream head, then `db_prev` pointers | One record per step |
| History of one row or one cell | `row.lin.t`, then `ent_prev` pointers | One record per change of that row |
| Everything in one logical operation | Follow `prev` while `change_set` matches (steps are adjacent or near) | Small |
| State of a database at time T | Walk the stream back to T, inverting (section 5) | Grows with changes since T; checkpoints later (10.3) |
| Is the log whole | Global head against the anchor at unlock; full chain walk on demand | O(1) at unlock |

Nothing here is loaded at unlock except the global head and the anchor comparison. No per-change index is kept in memory.

## 5. Why pre-existing rows stay reconstructable

Blue's challenge: an earlier-value-only log plus a "recording since" date does not, by assertion, let you rebuild rows that existed before recording began. Agreed. The argument below is what replaces the assertion, and it names what it depends on.

### 5.1 The claim

For a database D and a point T at or after its `complete_since`, with no gap interval between T and now touching the entity in question:

- **A row never changed since T** is, at T, exactly what it is now. No record is needed, and the missing stamp (or a stamp older than T) proves it was not changed.
- **A row changed since T** is rebuilt by taking its current recoverable fields and, for each of its records newer than T, newest first, replacing each changed field with its `b` value. Each step is checked: the fingerprint of the working state before the step must equal that record's `after_hash`, and after the step must equal its `before_hash`. A failed check stops the walk for that row and reports it; it never yields a guessed value.
- **A row created since T** did not exist at T. Its `row.create` record says so.
- **A row in the bin** is a field change (`trashed_at`), covered by the second bullet.
- **A row removed for good since T** cannot be rebuilt. Its values were deliberately destroyed (8.3). What remains is a content-free marker: one row existed here, it was removed for good at this time by this actor. The preview counts such rows; it never hides them.
- **Membership at T** = rows of D now, minus those created after T, plus the markers of those removed for good after T (count only).
- **Structure at T** is rebuilt the same way from the database's own records (`db.schema`, `db.layout`).

### 5.2 What makes this more than an assertion

| Risk | What defends against it | What it cannot do |
|---|---|---|
| A write that was never recorded | The lineage stamp: the stored fingerprint will not match the stored row at the next write, restore or check (4.5) | It detects, it does not recover the missing step. The gap is recorded and reconstruction across it is refused for that entity |
| A record lost or altered on disk | Each record is sealed and bound to its id; the vault chain and anchor show truncation or reordering; `before_hash` and `after_hash` must link from one record of an entity to the next | Same: detection and refusal, not repair |
| The present itself is damaged, so walking back from it is unsafe | The first-touch image (4.4) is an independent starting point: image, then apply each `a` value forward, must reach the present fingerprint. Two directions, one answer | Rows created in bulk and never changed have no image; they have only the present (HT-3) |
| An older or other build opening the vault and writing without recording | The format upgrade: older versions refuse the vault (accepted direction, test gate). During development, draft 3's development epoch | A build with a bug that bypasses `put_many`. The three known Records write points (E7, E8, E9) must be the only ones; a test asserts it (HV-03) |
| The clock was wrong | Order is `cseq`. "At time T" is resolved to a `cseq` by the records' times; if times are not increasing, the screen offers the records themselves as restore points, not a date | A wrong clock still shows wrong times |

### 5.3 What "complete" is allowed to mean

- `complete_since` is set at the transaction that performs the upgrade. Before it there is nothing, and the screen says "History starts here".
- A database is **complete over an interval** when its stream has no gap in it. A row is complete over an interval when, in addition, its own chain links without a fingerprint break.
- "Restore the whole database to T" is offered only if the database is complete from T to now. Otherwise the offer is the explicit partial one in 7.6.
- Trimming old history on the user's own limit (8.2) moves `first` forward. Points before `first` are no longer reconstructable, and the stream says so with reason `trimmed`.

## 6. Capture and commit protocol

### 6.1 One capture point

Capture lives in `prepare_journal` (E7) and its twin in `remove_many` (E8). No door calls it; no door can skip it. This is the main structural decision of the design.

```text
put_many(items):                                  -- existing steps kept, new steps marked +
  existing checks (write permission, coordination, size)
+ txn_id = new random id
+ for each Row/Database item in items:
+     old = self.get(id)
+     cmp = compare(old, new)                     -- field classes of 4.3
+     if cmp has no recorded change: continue     -- body-only or bookkeeping-only save
+     if old exists and old.lin present and fp(old) != old.lin.h:  add gap op with image(old)
+     if old exists and old.lin absent:           add image(old)   -- first touch
+     new.lin = { t: txn_id, h: fp(new) }         -- stamp BEFORE sealing
  seal each item                                  -- existing, now after stamping
  journal = prepare_journal(...)                  -- existing
+ journal.change = build ChangeTxn(txn_id, parts, pairs, heads, attachment counts)
  store.transaction:                              -- ONE SQLite transaction (existing)
      write records
      journal.write                               -- history, activity, anchor (existing)
+     write ChangeTxn, overflow parts, images, stream heads, global head, attachment counts
  accept_journal                                  -- in-memory heads move only after commit (existing order)
  bump revisions, signal change                   -- existing
```

Notes for the developer:

- Today items are sealed before `prepare_journal` runs. Stamping needs the transaction id before sealing, so the order of those two steps changes. That is a change inside the most sensitive function of the vault and needs the security review (`docs/security-design.md` applies).
- A body-only person save (the common autosave) produces **no** change record and does not move the stamp, because the fingerprint excludes the body. It stays as cheap as today. Body history remains the existing versions; when a version is taken in the same transaction, the op carries its id.
- `compare` is pure and has no door knowledge. It is the piece to test hardest (HV-10 to HV-19).
- Layout changes by a person (column width, filter tweaks) would flood the log. Proposed: at most one `db.layout` record per view per 10 minutes for the person actor; it holds the before picture of that view at the start of the window. States inside the window are not kept. This is a deliberate, declared coarsening for layout only and therefore a product choice (HD-6), not a silent one.
- `reorder_rows` rewrites every listed row (E22). It is recorded as one compact cosmetic op list (row id, order before, order after) for rows whose order really changed.

### 6.2 What a door must supply

| Door | Enters through | Must add | State at `9977e72` |
|---|---|---|---|
| Person commands | `with_actor` then `run_as` (E10) | A compiled action name per command | Missing: recorded as "items.changed" |
| AI | `atomic_action`, `run_action` | Nothing new | Present |
| Automations | Called by commands, same change set (E26) | Actor `automation` on the ops they cause, with the triggering txn in `origin_ref` | Actor exists (`Actor::Automation`); whether nested writes are labelled automation or with the outer actor was not traced: UNKNOWN |
| Import | One `put_many` (E27) | Action `import`, origin `import`, the import's own id | Missing |
| Bulk actions (multi-row edit, bin many) | Person commands | Action name; one transaction per call where the code already does so | As person |
| Restore and take-back | New | Origin and `origin_ref` | New |
| Upgrade and repair tools | New | Origin `upgrade` or `repair` | New |

A door that supplies nothing is still fully recorded; it is only labelled poorly.

### 6.3 Atomicity, stated exactly

- **One transaction record is atomic with the data it describes.** Records, history, activity, anchor and change records are in one SQLite transaction with a rollback journal and full sync (E2, E3). Either all are stored or none. This is the existing mechanism, verified by reading; the existing test named `journal_failure_rolls_back_records_history_revisions_and_memory` (`vault/activity.rs` 1234) shows the intent. Not run by me.
- **A logical operation of several transactions is not atomic today** for the person (HF4), and this design does not claim to make it so by recording it. Each step is recorded truthfully with `change_set` and `step`. If a later step fails or the app stops, the log shows exactly the steps that were stored. "Take back this change" then takes back those steps.
- **Proposed tightening, in order of value:** (a) make the destructive multi-step person operations single transactions: `delete_rows` and `delete_database` each do `put_many` then `remove_many` then collection, and a stop between them leaves links cut but rows present; (b) row change plus the automations it fires. Mechanism: either merge the steps into one store transaction, or use `begin_action / commit_action` (E3) with a restore of **only the touched in-memory entries**, not the whole-vault clone of E12 (HF5). Whether to do (b) is a behaviour change with a measured cost: HT-1.
- **Large operations.** Three cases, and only the first two may be called atomic:
  1. *One SQLite transaction, many records.* A change record that does not fit one sealed record continues in overflow parts (`more`). All parts are in the same transaction, so splitting the *record* does not split the *commit*. Limit: memory and the size of the rollback journal on disk. To be measured (9.3).
  2. *Staged creation with one publish point.* For an import too large for case 1: new items are written over several transactions carrying `staged: <operation id>` and are invisible to every read; one final small transaction writes "operation published". Reads treat staged items as present only if their operation is published. A stop before the publish point leaves invisible items that the next unlock removes. The single publish record is the commit; that is the concrete mechanism. It works for **new** items only.
  3. *Resumable, visibly partial operation.* For changing very many existing rows when case 1 is too large: an operation record states the plan and progress; each chunk is its own transaction and is fully recorded; the screen shows "in progress, N of M"; a stop leaves a stated partial state that can be continued or taken back chunk by chunk. **This is not atomic and must never be described as such.**
  The first version uses case 1 only. Cases 2 and 3 are designed here so the record format (`change_set`, `step`, operation id) does not need to change later.

### 6.4 Failure behaviour

- **History cannot be written (disk full, I/O error):** the whole transaction fails and the user's change is not stored; the item stays as it was; the screen says the change was not saved and why. Recording is never skipped to let the change through. Reason: a silently unrecorded change is exactly the gap this design exists to prevent, and this is already how the journals behave (same transaction).
- **No "pause history" switch in the first version.** If Ahmed wants one later it must write a gap interval and say so on the timeline (HD-7).
- **Low disk before a large operation:** estimate the cost first (records plus rollback journal) and refuse to start with a plain message, instead of failing midway. Draft 3 section 13's disk-pressure rules stay; nothing here deletes history to make room.
- **Crash during commit:** SQLite's rollback journal restores the file to before the transaction at next open. In-memory state is gone with the process. At unlock, the global head is compared with the anchor, as the activity head is today (E16).
- **Crash between steps of a multi-step operation:** 6.3, second bullet.
- **Failure inside an AI request:** existing `atomic_action` behaviour; the in-memory change heads join the cloned set (one more field in the snapshot at `vault/mod.rs` 93-98).

### 6.5 Retry and idempotency

- A change transaction id is created before the commit. A commit either happened or did not; there is no half state to retry into.
- A door that retries (the AI client after a lost reply) creates a new change set unless the door has its own request identity. Whether the AI dispatcher keeps request ids was not traced: UNKNOWN. Draft 3's door contract (explicit client retry behaviour) stands; history neither fixes nor worsens it, and makes a doubled action visible as two adjacent identical records.
- Restore and take-back carry a `plan` id (7.3). Committing the same plan twice is refused, because the first commit changed the fingerprints the plan depends on, and the refusal can say "already applied" by finding the plan id in the head records.

### 6.6 Concurrency and revisions

- All writes hold the session lock (`state.rs` 314-316), so there is one writer at a time and `cseq` order is real order.
- The in-memory revision numbers restart at each unlock (E6). They stay what they are: a silent check for a client that holds an old copy during one session. History uses the **stored** fingerprints, which survive lock and restart.
- Active typing protection (`check_write`, coordination, the 3-second grace) is unchanged. A restore of a row that is open and being typed in follows the existing page rule: flush, snapshot, then restore (E14), or refuse while the person is typing.

## 7. Preview and restore protocol

### 7.1 Rules that hold for every restore

- **R1.** Two phases: a read-only **plan**, then a **commit**. Nothing is written while planning.
- **R2.** The plan lists every item and field it will write, with current and target values, including every edit outside the thing the user pointed at. Nothing outside the list is written.
- **R3.** The plan records the fingerprint of every item it will write. Commit re-checks them under the session lock. Any difference refuses the whole commit with "this changed since the preview".
- **R4.** Commit is one `put_many`, one transaction: all listed writes or none.
- **R5.** No silent partial restore. If part of the target cannot be restored, the plan says exactly which part and why, and the user either accepts that explicit smaller scope or cancels. The default choice is cancel.
- **R6.** Identity is kept: the same row id, property id, option id. A restore never creates a look-alike.
- **R7.** A restore never deletes for good. Something that should "not exist" at the target goes to the bin.
- **R8.** A restore fires no automations and no reminders. They are side effects, not state. (Formula and rollup values are computed on read and follow by themselves; that computed kinds are never stored was seen only in `duplicate_row`: partly verified.)
- **R9.** The restore is recorded like any change (origin `restore`, `origin_ref` naming the target). That record's `b` values are the before-picture of the restore, so **undoing a restore is an ordinary take-back** under the same rules.
- **R10.** A restore does not reset baselines or completeness. It adds records; it removes none.

### 7.2 Building the target

```text
target_state(scope, k):          -- k = the ChangeTxn the user picked; target = state just AFTER k (or just BEFORE, for "take back")
  for each entity in scope:
      walk ent_prev from entity.lin.t back to k, inverting, checking fingerprints (5.1)
      on a fingerprint break or gap: mark entity UNRESOLVED(reason)
      on a purged op: mark field or entity PURGED
  return { resolved values, unresolved list, purged list }
```

### 7.3 The plan object

```text
RestorePlan
  plan: id, made_at_cseq
  scope: cell | row-properties | row-properties-and-body | property | change-set | database
  target: { txn, side: after | before }
  writes:   [ { item, field, current, target, why: asked | reciprocal | required } ]
  expects:  [ { item, fingerprint } ]          -- R3
  requires: [ schema steps that must be part of it ]   -- e.g. bring back an option
  cannot:   [ { item, field, reason } ]        -- R5; reasons in 7.5
  costs:    { rows, records, estimated bytes, safety backup offered: bool }
```

The plan lives in memory for the session only. It is never stored, so a lock or restart discards it.

### 7.4 Scopes in the first version

| Scope | Target | Closure the plan computes |
|---|---|---|
| One cell | Value of (row, property) at k | Reciprocal cells for a two-way relation; a missing option (7.5) |
| Row properties | All recoverable fields of one row at k | As above for each cell; bin state |
| Row properties and body | Same, plus the nearest page version at or before k | The body comes from existing versions; the preview states the version's own time, which may be earlier than k |
| Removed property | Definition back, same id; values are already in the rows (4.6) | For a two-way relation: the back property in the other database must also come back, or the plan refuses |
| Take back a change set | The `b` side of every op in that set, for any actor | Each op's own closure; ops whose items changed later are listed under `cannot` unless the later change is itself included |
| Row from the bin | Exists today (`restore_rows`) | Unchanged |

The validation of each target value goes through the existing `clean_cell` under the **current** structure, and reciprocal edits through the existing `relation_sync_diff`, so a restore cannot store something a normal edit could not.

### 7.5 Dependencies, conflicts and refusals

| Situation | Minimal consistent scope | Outcome |
|---|---|---|
| Relation cell regains a link to row X; the relation is two-way | Only X's back cell gains this row. X is otherwise untouched; X's database is not rolled back | Listed as `reciprocal`; allowed |
| Relation cell loses a link | Only the other side's back cell loses this row | Listed; allowed |
| X was removed for good | None possible | `cannot`: "link to a row that no longer exists". Other links proceed only if the user accepts the smaller scope |
| X is in the bin | Whether a link to a binned row is valid today was not traced: UNKNOWN | Proposed: `cannot` unless the user also restores X from the bin, offered as an explicit extra |
| The back side is "one row only" and X now links to another row Y (HF9) | Restoring would remove Y's link | `cannot` by default. Offered as an explicit, named extra: "this will also unlink Y" |
| The back property no longer exists in the other database | A one-way link would break the pair rule (`relations.rs` 46-54) | `cannot`; offer "restore that property first" |
| Select or status value whose option was removed | Bring the option back, same id | Listed under `requires`; shown; allowed only with it |
| Property kind changed since (same id, E25) | Old value is not valid under the new kind | `cannot`; offer "change the kind back" (4.6) if that was kept |
| Property removed since | The property must return | Listed under `requires` |
| Value fails today's validation (limits changed, target database gone) | None | `cannot`, with the validator's reason |
| Row is in a locked database, or the actor may not write | None | Whole plan refused before preview: existing `check_write` |
| The item changed between preview and commit | None | Whole commit refused (R3) |
| History has a gap or a break for this item | None | `cannot`: "history is incomplete here", with the interval |
| Target body version was thinned or removed | Properties only | Explicit smaller scope |
| The item is being typed in | None | Refused until typing stops (existing rule) |

External relation targets are therefore **never rolled back as a whole**. Only the one cell that mirrors the restored link changes, and only when that change removes nothing else. Everything stronger is refused or offered as a named extra.

### 7.6 Later: the whole database at a point in time

Designed now so the records support it; not proposed for the first version.

- **Precondition:** the database is complete from T to now (5.3). Otherwise only the explicit partial offer: "N rows can be restored exactly, M cannot, here is why for each group".
- **Plan:** structure at T; every current row's recoverable fields at T; rows created after T go to the bin; rows binned after T come back; rows removed for good after T are counted as not recoverable. Bodies are a separate tick box, default off.
- **Other databases:** only mirror cells of two-way relations that involve this database's rows change. If the partner property was removed or changed kind, that relation property is excluded and named. Other databases' structure is never touched.
- **Size:** up to 200,000 rows (`MAX_ROWS_PER_DATABASE`). One transaction (6.3 case 1) if measurement allows; otherwise 6.3 case 3 with its honest label, preceded by an offered whole-vault safety copy (HA-5).
- **Undo:** the restore's own records (R9). For a 200,000-row restore that is a large record set; the preview states the estimated size.
- **Read-only look first:** the same reconstruction, shown but not written, is the "historical view" of the Notion screens. It should ship before the write version, because it lets the user check the target with no risk.

### 7.7 Provenance after a restore

- Restored values are recorded with actor = who pressed restore, origin `restore`. The timeline shows "restored to the state of <date> (originally by <actor>)"; the original author is information, not the author of the new record.
- The row's `source` tag (WP-06h) is a recoverable field like any other; restoring it is listed in the plan.

## CHECKPOINT 2

- Status: IN PROGRESS (chunk 2 of 4 written)
- Last finished: sections 5 to 7
- Next unit: sections 8 to 10 (retention and purge, performance plan, phasing)
- Time: 2026-11-11T14:55:41Z (system clock)

## 8. Retention, purge, privacy and the limits of erasure

### 8.1 Two logs with two jobs

| | Activity (exists) | Change records (new) |
|---|---|---|
| Job | A feed of who did what, and AI undo | Recovery data for databases |
| Limit | 365 days **and** 100,000 entries, whichever comes first (`vault/activity.rs` 21, 799-807) | None by default; the user's own limits only |
| Person can clear it | Yes (`clear_activity`, typed confirmation) | Not by that command; clearing activity must not touch change records |
| Link between them | `activity_seq` in the change record, `change_set` in both | The link may dangle once activity is pruned; the change record carries its own actor, time and action so it never depends on activity |

The AI undo promise stays bounded by the activity limits, as Blue stated. "Take back this change" on change records is a separate, longer-lived path (7.4). Running both for AI writes stores the AI's earlier picture twice in the first version; merging them is later work (10.3) and unapproved.

### 8.2 Retention policy

- A single sealed policy record (tag `0x16`), as in draft 3 section 13: `{ v, max_age_days?: none, max_bytes?: none, applies_to: changes | page-versions | both, set_by, set_at }`. Absent values mean **keep**.
- **No automatic thinning of change records, ever, and no thinning in the middle.** The only reduction is trimming the **oldest** records when the user's own limit says so. Middle records are never dropped, because that would break the walk of section 5 for everything older.
- Trimming moves the chain start (`first`, `base_hash`, the pattern activity already uses in its head) and each affected stream's `first`, with gap reason `trimmed`. The user sees, before confirming a limit, what date history would then start from and how much space it frees.
- Trimming runs only as an explicit maintenance step with a result line ("history before 3 March was removed on your 2-year limit"), never hidden inside a save. That differs from today's `maintain_journals` inside `put_many` (E15).
- **Page versions:** the WP-04 rules (E15) are live code and a binding contract. Under Ahmed's direction they would stop applying by default, which needs an amendment (10.5). Until then this design only requires that a change record whose body version was thinned says so at restore time (7.5), which is how AI undo already behaves (`vault/activity.rs` 936-944: a missing version is skipped).
- Size reporting: "history uses X" must count change records, images, page versions and attachments kept alive only by history (4.8), separately.

### 8.3 Remove for good

Rule carried from the existing code (E19): deleting something for good removes its content from the journals and leaves content-free markers.

```text
purge(row R):                                  -- inside the existing remove_many transaction
  walk ent_prev from R.lin.t
  for each ChangeTxn reached: replace R's op with { kind, id, at, actor, purged: true }; drop fields, body refs, title
  delete R's images
  decrease attachment counts
  append op row.purge to a new ChangeTxn (no values)
```

- The per-entity back-pointers make this proportional to the changes of that row, not to the log.
- A database purge does the same for its rows and its own records, in chunks **inside one transaction** where size allows; otherwise as a resumable operation (6.3 case 3) that is honest about being in progress. A purge that stops midway must be continued at next unlock; until it finishes the screen says "removal in progress", and nothing of the item is shown.
- "Remove property for good" strips that property's values from every row and from every op (4.6).
- **What a purge does not reach, to be said on the confirmation screen and in the security record:**
  1. Copies of the vault made earlier: backups, the pre-upgrade safety copy, exports, synced or copied files. A vault cannot erase a file it does not hold.
  2. Values that were copied into other items by the user, by an automation that expands text from the row (`commands/automation.rs` 178), or by an import. Those belong to the other item now.
  3. Ids of the removed row inside other rows' relation history. Ids are random and carry no content, but they show that a link existed.
  4. The storage device. `secure_delete = ON` (E2) overwrites freed database pages; it does not control the rollback journal file's remains, file system journals, SSD wear levelling or system snapshots.
- So the honest promise is: **"removed from this vault file's readable content and history"**, not "no value anywhere". And after a purge, exact recovery of that item is intentionally impossible, including through whole-database restore (5.1).

### 8.4 Chain and purge together

A hash chain over full records would force re-hashing every later record at each purge (the activity log re-links its chain, E19; with an unbounded log that cost is unacceptable).

- The chain covers the **header core** only: `cseq, prev_hash, at, actor, change_set, step, action, origin`, and the list of (database id, entity id, op kind). It proves order, and that nothing was removed or inserted.
- The values are protected by the seal of the record (authenticated, bound to table and id) and by `ops_hash` in the header, outside the chain. A purge rewrites the values and `ops_hash` of the affected records and leaves the chain valid.
- Consequence, stated: the chain does not prove that the values of an old record are the original ones; the seal proves they were written by someone holding the vault key. Someone who can swap in an older sealed copy of the same record could bring back purged values, but that person already holds the older file. Accepted limit; to be confirmed in the security review.
- The anchor record (E16) gains the change head, so a rolled-back or truncated change log is detected at unlock the way a truncated activity log is today.

### 8.5 What the file reveals

Per HF8 and E5, someone holding the locked file without the password can see: that a `changes` table exists, how many records it has, their padded sizes (256-byte, 4 KB or 64 KB steps), and by comparing two copies, how many were added between them. They cannot see ids of databases or rows in it, names, values, times or actors: those are inside the sealed data, and record ids are random.

- This is the same class of leak the activity and history tables already have; the change table makes the count more fine-grained (roughly one record per database edit).
- No lookup key is derived from content. An alternative using keyed, deterministic record ids (to find a record by database and position without pointers) was considered and **not** recommended: it adds a new key derivation to the crypto surface for a convenience the back-pointers already give.
- Nothing from change records goes into exports, AI reads or the search index in the first version (HD-8).

## 9. Costs, proposed goals and how to measure

Nothing in this section was measured. The numbers under "proposed goal" are starting proposals to be replaced by the first measurements; none is a promise, and none says "unaffected".

### 9.1 Where the cost lands (code-derived)

| Operation | Added work | Grows with |
|---|---|---|
| Save one cell | Compare two rows (cells JSON parsed twice), two fingerprints, one or two small sealed records, one stream head, one global head, in the **same** transaction (no extra disk sync) | Size of that row's cells; first touch adds one image |
| Save a body (autosave) | One comparison that finds nothing | Row size (cells), not the log |
| Save a database (structure or layout) | Compare two database objects up to 30 MB, fingerprint of the schema part | Database object size. The costly case: large templates. Fingerprint only `props`; compare other parts by element hash |
| Import | One `row.create` op per row, packed (ids and fingerprints, about 100 bytes each before sealing) | Row count; no value copies (4.4) |
| Unlock | Read global head, compare with anchor | Constant. The log is not read |
| Memory | Heads only; a timeline page while it is open | Constant; no in-memory index |
| Rekey | Re-seal every change record and image in the one transaction (E28) | **Total history size.** With indefinite retention this grows without bound; it is the cost most likely to hurt |
| Backup (`VACUUM INTO`) | Copies history too | Total history size |
| Purge of a row | Rewrite each record that touched it | Changes of that row |
| Timeline, row history | One record read per step | Page length |
| State at T | One record read per change since T | Changes since T, until checkpoints exist |
| Disk | About 0.3 to 0.6 KB per single-cell change (one or two 256-byte buckets plus seal overhead); bulk changes pack to roughly the size of the values | Edits. Illustration only: 100 single edits a day is in the order of 15 to 20 MB a year |

### 9.2 Proposed goals (to be reset after first measurement)

| # | Measure | Proposed goal |
|---|---|---|
| PG-1 | Added time to save one cell, median and 95th percentile, against the same build with recording switched off | Median under 2 ms, 95th under 10 ms on the reference PC |
| PG-2 | Added unlock time | Under 50 ms, and flat across log sizes 0, 100 thousand, 1 million records |
| PG-3 | Added resident memory after unlock | Under 1 MB, flat across log sizes |
| PG-4 | Body autosave | No measurable change (within run-to-run noise) |
| PG-5 | First timeline page (50 entries) | Under 200 ms |
| PG-6 | Row history, 100 changes | Under 300 ms |
| PG-7 | Rekey and backup | No goal yet: **measure and report** time and size per GB of history, then decide whether rekey needs a progress screen or a staged design |
| PG-8 | Import of the large fixture | Added time under 10 percent; added disk under 5 percent of the imported size |
| PG-9 | Read-only state at T (later phase) | Report only, per 10 thousand changes walked |

### 9.3 Synthetic workload plan (for the dev thread, when authorised)

- **Data:** invented only, the "Garden Club" fixture family: members, plots, tasks, with two-way relations between tasks and plots, selects, numbers, dates, files of generated bytes. No real notes, no real exports.
- **Sizes:** S: 3 databases, 1,000 rows. M: 10 databases, 20,000 rows. L: one database at 200,000 rows (the code's limit) plus M. XL import: generated export of about 1 GB, to mirror the size Ahmed named for his largest Notion export (an export size, not a vault size).
- **Logs:** each size with 0, 10 thousand, 100 thousand and 1 million change records, produced by a scripted edit mix (70 percent single cell, 10 relation, 10 bulk of 500 rows, 5 structure, 5 bin and restore).
- **Each run records:** build commit, fixture seed, machine, the PG measures, file size before and after, peak memory, rollback journal peak size for the largest transaction.
- **Comparison:** always against the same commit with recording off, same fixture, same machine, three runs, report spread.
- **Place:** a throwaway vault in the test folder outside the repository, never a real vault (repository rule).
- **Status:** NOT RUN. No fixture exists yet.

## 10. Phased integration

### 10.1 What depends on what

```text
Format upgrade decision (accepted direction, needs amendment)
   └─ F  Foundation: recording, stamps, table, chain, purge, retention record        <- cannot be back-filled later
        ├─ S1 First recovery surface: timeline, row history, cell/row/property restore, take back
        │    └─ S2 Read-only state at T, highlights, previous/next, bookmarks
        │         └─ S3 Whole-database restore, checkpoints
        └─ M  Merge AI undo onto change records; page-body diff (1.8-8); activity panel (1.8-9); sync ledger (1.9-17)
```

Dependencies on draft 3: the open sequence (no table created before the user agrees), the door track's action names (C-items for common validation: person doors get compiled names, which this design also needs, 6.2), baseline v1 fingerprints (reuse the same canonical form and SHA-256 rather than a second one), and the format track (one format step should carry `lin`, `removed_props`, the `changes` table and the policy record together, so the format is raised once).

Dependencies on existing lanes: none of the lanes read for draft 3 (N2 import, P6c, K1 family, S, W, R1) writes Records except through `put_many` and `remove_many`, **as far as their merged parts on `9977e72` show**; the unmerged lane tips were not inspected for this design: UNKNOWN. HV-03 is the test that would catch a lane adding a new write path.

### 10.2 Foundation (F): interfaces that should exist before or with the format step

| # | Interface | Contract |
|---|---|---|
| F1 | `compare(old: Option<&Item>, new: &Item) -> Option<Op>` | Pure; field classes of 4.3; unknown database fields produce an image, never a guess |
| F2 | `fingerprint(item) -> Hash` | Canonical form shared with baseline v1; excludes `lin`, body, cosmetic and volatile fields |
| F3 | `lin` on Row and Database; `removed_props`, `sv`, stream id on Database | Unknown to older versions, hence inside the format step |
| F4 | `Table::Changes` and record types of 4.1 | In rekey, backup, verify and upgrade lists; created only in the upgrade step |
| F5 | `ChangeJournal` inside the prepared `Journal` | Written in the same transaction; in-memory heads accepted only after commit; part of the `atomic_action` snapshot |
| F6 | Global head in the anchor | Unlock compares; mismatch behaves like today's journal mismatch |
| F7 | Action names for person, import, automation doors | Shared with draft 3's door work |
| F8 | Purge hooks in `remove_many` and "remove property for good" | 8.3 |
| F9 | Retention policy record; change records exempt from automatic maintenance | 8.2 |
| F10 | Read interfaces: `changes_for_database(db, before, limit)`, `changes_for_row(row, before, limit)`, `change_set(id)`, `check_history(scope)` | Read-only; no screen needed to test them |
| F11 | Development switch | Recording can be built and tested on throwaway vaults before the format decision is an amendment (draft 3's door track and development epoch) |

F alone is useful: from the day it ships, every database change is recoverable **in principle**, and a later version's screens work on history that already exists.

### 10.3 Surfaces

| Phase | Contents | Independent value |
|---|---|---|
| S1 | Database timeline (read-only list, filter by actor, hide cosmetic); row history with before and after per cell; restore cell, row properties, removed property; take back a change set; restore preview with the 7.5 outcomes | Answers "what changed, who, when" and "put this back" for the everyday accidents: wrong cell, wrong bulk edit, removed column, AI or automation mistake |
| S2 | Read-only state of a database at a picked point; change highlights with previous and next; named bookmarks | The Notion-like look-before-you-restore experience |
| S3 | Whole-database restore; checkpoints (a stored full state every N thousand changes so that walking back is bounded); export of history | The large disaster case |
| M | AI undo served from change records; page-body diff; activity panel; alignment with the sync ledger | Removes duplicate storage; roadmap items 1.8-8, 1.8-9, 1.9-17 as they stand |

### 10.4 Three ways to place this in 1.6, with what each gives up

| Option | In 1.6 | Gives up |
|---|---|---|
| P-A | F only | No screens; recovery exists only as data. Smallest risk to 1.6; a later version must add S1 before anyone benefits |
| P-B | F and S1 | The most useful small whole. More test surface (the restore protocol); S2 and S3 wait |
| P-C | F, S1 and the read-only part of S2 | Closest to what Ahmed saw in Notion; the reconstruction walk and its performance enter 1.6 |

My recommendation is **P-B, built in the order of 10.6 so that it can fall back to P-A at a clean line** if the stop criteria trip. This is a recommendation about engineering order. Which version carries what is Ahmed's decision (HD-1) and is not made here.

What I advise against: shipping the format step without F. The format would be raised once for other reasons and then have to be raised again, and every edit made in between would have no history for ever.

### 10.5 Records that would have to change first (none written)

- **WP-04** (thinning rules) against "indefinite by default": amendment.
- **G3 in A44** ("1.5.1 still opens a 1.6 vault") against the accepted format direction: amendment, already noted in draft 3.
- **A53** ("no crypto, storage format or permission change" for the import changes): unaffected in itself, but import gains an action name and `row.create` ops; say so in the amendment text.
- **`docs/security-design.md`**: new table, anchor content, purge limits (8.3), file-visible metadata (8.5), reordered sealing step (6.1). Security behaviour changes need Ahmed's own words.
- **Version plan section 8** (format gate at 1.8) and roadmap items 1.8-8, 1.8-9: placement text, only after decisions settle, on his word.

### 10.6 Practical first-release sequence

Each step ends in something checkable; each is a place to stop.

1. **F1, F2 with tests only.** Pure functions and fixtures; no storage change. Stop here costs nothing.
2. **F7 action names** (shared with the door work).
3. **F3 to F6 behind the development switch,** on throwaway vaults: stamps, table, change journal in the transaction, anchor. Run HV-01 to HV-09 and HV-20 to HV-29.
4. **Measure** (9.3) against PG-1 to PG-4 and PG-8. First stop gate.
5. **F8, F9:** purge and retention record. Run HV-40 to HV-49.
6. **F10 read interfaces,** then the read-only timeline and row history (first half of S1). Second stop gate: P-A plus read-only screens is a coherent release.
7. **Restore protocol** (plan, then commit) for a cell, then row properties, then removed property, then take back. Run HV-50 to HV-69.
8. **Upgrade path:** recording begins in the upgrade transaction; the older-version refusal test on named versions (accepted gate).
9. **Whole-app regression** on the existing sheet, plus rekey and backup with history present.

### 10.7 Stop criteria

Stop and report to Ahmed, do not push on, if any of these holds:

- A Records write path exists that does not pass the capture point (HV-03 fails).
- A fingerprint mismatch appears in any test that did not inject one.
- PG-1 or PG-4 is missed by more than a factor of two after one round of tuning.
- Unlock time or memory grows with log size (PG-2, PG-3).
- Any test shows a stored change without its record, or a record without its change.
- A restore test shows a write outside the plan (R2), or a partial write (R4).
- The older-version refusal test changes the vault file.
- Rekey with a large history cannot complete or cannot roll back cleanly.

On a stop after step 6, the fallback is P-A plus read-only screens; restore waits.

### 10.8 What must keep working

Existing behaviour that this design must not change, each with a test in section 12: page history and restore (E13, E14); AI undo and its limits (E18); the owner-tested AI checks of EX-55 (pairing, read, denied edit with unchanged revision, refusal while the person is typing, confirmed edit, bin and restore); import as one all-or-nothing save (E27); bin and restore of rows; relation behaviour including the "one row only" rule; rekey and backup covering all tables; permanent delete removing history. No bulk path is excluded from recording: import, multi-row edits, reorder and automations are all in, each with its packing rule (4.4, 6.1).

## CHECKPOINT 3

- Status: IN PROGRESS (chunk 3 of 4 written)
- Last finished: sections 8 to 10
- Next unit: sections 11 to 15 (screens, validation matrix, open decisions, source coverage, log)
- Time: 2026-11-11T14:57:11Z (system clock)

## 11. Screens and wording

Grounded in what Blue verified in Notion's own screens. Nothing here assumes how Notion stores anything, and Notion's restoring of database structure was **not tested** by Blue, so nothing here copies it.

### 11.1 What is taken from the verified Notion screens

| Seen in Notion (verified UI) | In Notaya |
|---|---|
| A timeline of versions with a read-only preview | Database timeline; entries open a read-only detail (S1), and later a read-only view of the whole database at that point (S2) |
| Grouping by 1 minute, 15 minutes, 1 hour (a display grouping) | Display grouping only: entries of one change set are one line; adjacent lines by the same actor can fold by time. The stored records are never merged |
| A drill-down showing a relation property before and after, with an exact time | Row history: property name, before value, after value, time, actor, action |
| Source labels such as "with ChatGPT", "with Claude" | Actor label from the record (11.4) |
| Begin restore, a review window listing scope, then "Restore this version" | Restore preview listing scope, extras, and what cannot be restored, then one confirm |
| Old versions kept; a restore makes a newer entry | R9, R10 |
| Change highlights and Previous / Next | S2 |
| Bookmarks | S2: a named point is a label on a change record |
| No selective tick boxes | Notaya differs on purpose: a cell, a row or a property can be restored alone, because a database-wide rollback is too blunt for everyday mistakes |
| A linked view's history looked empty (not reliable evidence) | Not used as evidence. In Notaya an inline or linked database shows the history of the database it points at |
| Older versions can be locked behind a plan | Not applicable; Notaya is local |

### 11.2 The timeline line

`<time> · <actor> · <what, in words> · <how much>`, for example: "Today 14:02 · You · Changed Status on 12 rows · bulk edit". Opening it lists each row with property, before and after. Rules:

- Words come from the action name and the ops, never from free text supplied by a door.
- Cosmetic entries (order, favourite, layout) are hidden behind "Show layout changes".
- A change that also touched another database says so and links there ("also updated 3 rows in Plots").
- A gap is a visible line: "History is incomplete between <a> and <b> for <row>: a change was not recorded." A trimmed start is a line: "History before <date> was removed on your limit."
- A removed-for-good item is a line with no content: "A row was removed for good."
- "History starts here" marks the upgrade.

### 11.3 The restore preview

Sections in this order, always all present (an empty section says "none"):

1. **What will change:** each item, property, current value, value after restore.
2. **What will also change:** reciprocal relation edits; options or properties that come back.
3. **What cannot be restored, and why:** from 7.5, in plain words.
4. **What this does not do:** "Automations will not run. Nothing is deleted for good. Rows added since then go to the bin." (the last only for whole-database restore).
5. **Buttons:** "Cancel" (default) and "Restore". If section 3 is not empty, the button reads "Restore the <n> items listed above only".

After the restore, one line with "Take back" for the session, and a permanent entry in the timeline.

### 11.4 Who did it: what the label can and cannot say

- `person` means "through the app's own screens on this PC". There are no accounts (`created_by` is plain text, `database/types.rs` 506-510), so it cannot say which human.
- `ai:<label>` means "through the AI door with this pairing". The label is what the pairing was named. It does not prove which model or which person prompted it. As Blue noted for Notion's labels, a label is not proof of human authorship.
- `automation` means "an automation rule ran"; the entry names the rule and the change that triggered it, when F7 supplies them.
- `import`, `restore`, `upgrade`, `repair` are origins, shown as such.
- An optional AI `source: {app, id}` (WP-06h) is shown as given, marked "reported by the AI app".

### 11.5 Settings

- "Keep history: for ever (default) / for … / up to … size". Changing it shows the date history would then start from and the space freed, and needs a confirm. Nothing is trimmed until then.
- "History uses …": change records, page versions, and files kept only because history names them, as separate lines, with "these files are kept because history still refers to them".
- "Check history": runs the fingerprint and chain checks and reports gaps.

## 12. Validation matrix

All rows: **NOT RUN.** Type: U = unit test on pure code, I = integration test on a throwaway vault in the Rust tests, X = on the built app with throwaway data on a PC. Phase refers to 10.2 and 10.3.

| ID | Area | Case | Expected | Phase | Type |
|---|---|---|---|---|---|
| HV-01 | Capture | One cell changed by person | One record; `b` and `a` correct; stamp updated; activity entry unchanged in shape | F | I |
| HV-02 | Capture | Same change through AI, automation, import, bulk edit, duplicate, bin, restore from bin | A record each, correct actor and action | F | I |
| HV-03 | Capture | Static check: every write to the Records table is in `put_many`, `remove_many` or a listed exempt function | No other writer; the test fails when one is added | F | U |
| HV-04 | Capture | Body-only autosave | No change record; stamp unchanged; page version behaviour as today | F | I |
| HV-05 | Capture | First touch of a pre-existing row | Image written once; second change writes none | F | I |
| HV-06 | Capture | Two-way relation edit inside one database and across two | Paired ops; two parts for two databases; both timelines show it | F | I |
| HV-07 | Capture | "One row only" back side drops an older link | Third paired op recorded | F | I |
| HV-08 | Capture | Reorder 1,000 rows | One compact cosmetic record; only rows whose order changed | F | I |
| HV-09 | Capture | Import of S fixture | One transaction; `row.create` ops without values; no images | F | I |
| HV-10 | Compare | Every cell kind: set, change, clear | Exact before and after, including number as stored | F | U |
| HV-11 | Compare | Title, icon, cover, bin state, source tag | Recorded as recoverable | F | U |
| HV-12 | Compare | Property add, rename, option add, option remove, option rename | Element ops by id | F | U |
| HV-13 | Compare | Property removed | Definition in `removed_props`; values still in rows; hidden from views, filters, formulas, export, AI read | F | I |
| HV-14 | Compare | Kind changed by the AI path | Same id; old definition and old values kept aside | F | I |
| HV-15 | Compare | View, template, automation edits | Element ops; layout marked cosmetic; 10-minute rule for person layout | F | U |
| HV-16 | Compare | Bookkeeping only (`next_id`, `updated`, `last_run`) | No record | F | U |
| HV-17 | Compare | Unknown field on a database | Image fallback, no guess | F | U |
| HV-18 | Compare | Canonical form stable across key order and number formatting | Same fingerprint | F | U |
| HV-19 | Compare | Fingerprint excludes body, order, favourite, stamp | Unchanged by them | F | U |
| HV-20 | Commit | History write fails (injected) | Change not stored; memory state as before; error shown | F | I |
| HV-21 | Commit | Disk full during a large save (injected) | Nothing stored; file opens clean | F | I |
| HV-22 | Commit | Process killed during commit | After reopen: either all or none; head equals anchor | F | X |
| HV-23 | Commit | Change record larger than one sealed record | Overflow parts; all in one transaction; kill test as HV-22 | F | I |
| HV-24 | Commit | Multi-step person operation with step 2 failing | Step 1 recorded with `step: 1`; no record for step 2; take back restores step 1 | F | I |
| HV-25 | Commit | `delete_rows` stopped between its steps (if 6.3 (a) is adopted: cannot happen) | State and records agree either way | F | I |
| HV-26 | Commit | AI request fails midway | No records; in-memory heads restored | F | I |
| HV-27 | Commit | Panic inside a write | As HV-26; no actor leaks into the next write (existing test's twin) | F | I |
| HV-28 | Commit | Lock and unlock, 0 and 100 thousand records | Heads verified; no log read; time flat | F | I |
| HV-29 | Commit | Rekey, password change, backup with history | All records re-sealed or copied; rollback clean on injected failure | F | I |
| HV-30 | Lineage | Row altered in the store without a record (injected) | Next write records a gap with image; check command reports it | F | I |
| HV-31 | Lineage | Walk back 50 changes of one row | Each step's fingerprints link; result equals the saved fixture state | F | I |
| HV-32 | Lineage | Image plus forward `a` values | Reaches the present fingerprint | F | I |
| HV-33 | Lineage | Record removed from the store (injected) | Chain check fails; restore for affected rows refused with reason | F | I |
| HV-34 | Lineage | Clock set backwards between changes | Order by `cseq`; timeline does not reorder | F | I |
| HV-35 | Lineage | Untouched pre-existing row at any past point | Equals present; no record needed | F | I |
| HV-40 | Purge | Row removed for good | Its values gone from every record and image; markers remain; chain valid; attachment counts down | F | I |
| HV-41 | Purge | Search the raw decrypted records for the purged values | Not found in this vault file's records | F | I |
| HV-42 | Purge | Database removed for good, 20,000 rows | As HV-40; other databases' relation properties handled as today | F | I |
| HV-43 | Purge | Property removed for good | Values stripped from rows and ops | F | I |
| HV-44 | Purge | Purge stopped midway (injected) | Continues at unlock; nothing of the item shown meanwhile | F | I |
| HV-45 | Retention | No policy set, 400 simulated days | Nothing trimmed; no thinning of change records | F | I |
| HV-46 | Retention | Age limit set | Preview shows date and size; only oldest trimmed; `first` moved; gap reason `trimmed` | F | I |
| HV-47 | Retention | Clear activity | Change records untouched | F | I |
| HV-48 | Retention | Activity pruned past 100,000 | Change records still readable without their activity entries | F | I |
| HV-49 | Attachments | File removed from a cell, history kept, then trimmed | Kept while referenced; collected after | F | I |
| HV-50 | Restore | Cell, plain kinds | Preview equals result; one record with origin restore | S1 | I |
| HV-51 | Restore | Row properties | All recoverable fields; body untouched unless asked | S1 | I |
| HV-52 | Restore | Row properties and body | Body from nearest version; its time shown | S1 | I |
| HV-53 | Restore | Removed property, plain and two-way relation | Same id; values visible again; relation pair returns or plan refuses | S1 | I |
| HV-54 | Restore | Relation cell regains a link | Only the mirror cell changes elsewhere | S1 | I |
| HV-55 | Restore | Link target removed for good | Listed under cannot; explicit smaller scope only | S1 | I |
| HV-56 | Restore | Back side "one row only", occupied | Refused by default; named extra works and is recorded | S1 | I |
| HV-57 | Restore | Option removed since | Comes back with the same id as a listed requirement | S1 | I |
| HV-58 | Restore | Kind changed since | Refused with reason | S1 | I |
| HV-59 | Restore | Item changed between preview and commit | Whole commit refused; nothing written | S1 | I |
| HV-60 | Restore | Injected failure on the last write of a plan | Nothing written | S1 | I |
| HV-61 | Restore | Writes compared with the plan | No write outside the plan | S1 | I |
| HV-62 | Restore | Automations with matching triggers present | None fire | S1 | I |
| HV-63 | Restore | Take back a restore | Returns to the pre-restore state under the same rules | S1 | I |
| HV-64 | Restore | Take back a bulk edit of 500 rows where 3 changed later | 497 listed; 3 under cannot; explicit confirm | S1 | I |
| HV-65 | Restore | Take back a person change, an automation change, an AI change | Each works; AI undo (existing) still works separately | S1 | I |
| HV-66 | Restore | Locked database; row being typed in; AI as the actor | Refused before preview; typing protection as today | S1 | I |
| HV-67 | Restore | Same plan committed twice | Second refused as already applied | S1 | I |
| HV-68 | Restore | Row with a gap | Refused with the interval | S1 | I |
| HV-69 | Restore | `next_id` after restoring rows | Never lower than before | S1 | I |
| HV-70 | Upgrade | 1.5.1-format vault upgraded | Verified backup first; table created in the upgrade step only; "History starts here" | F | X |
| HV-71 | Upgrade | Named older versions open the upgraded vault | Refuse; file bytes unchanged (accepted gate) | F | X |
| HV-72 | Upgrade | Decline or failure | Original unchanged | F | X |
| HV-73 | Regression | Page history, page restore, AI undo, EX-55's five AI checks, import all-or-nothing, bin and restore | As before | F | I, X |
| HV-74 | Performance | PG-1 to PG-8 on S, M, L and XL | Reported against the goals | F | X |
| HV-75 | Privacy | Locked file inspected | Only table name, record count and padded sizes visible; no ids of databases or rows in record ids | F | I |
| HV-76 | Screens | Timeline wording for each action, gap, trim, purge marker | As 11.2 | S1 | X |
| HV-80 | Later | Read-only state at T equals a saved fixture state at T | Equal, or stated incomplete | S2 | I |
| HV-81 | Later | Whole-database restore, complete history | Equals fixture at T; created-since rows in the bin; other databases touched only in mirror cells | S3 | I |
| HV-82 | Later | Whole-database restore with one purged row and one gap | Explicit partial offer; exact lists | S3 | I |

## 13. Unresolved decisions

### 13.1 Ahmed's (product choices; none is assumed)

| # | Question | My recommendation |
|---|---|---|
| HD-1 | Which of P-A, P-B, P-C goes into 1.6 (10.4) | P-B, with the fallback line to P-A |
| HD-2 | Restore of single cells, rows and properties, which Notion's screens did not show | Yes; it is the everyday case |
| HD-3 | May "take back" apply to any actor's change, not only the AI's | Yes, person only may press it in the first version |
| HD-4 | May the AI ask for database restores | Not in the first version |
| HD-5 | Keep a removed property's values hidden in the rows until "remove for good"; keep old-kind values when a kind is changed | Yes to both; it is what makes those restores cheap and exact. The cost is that "remove" no longer frees the data until the second step, and the screen must say so |
| HD-6 | Layout changes (column widths, filters) recorded at most once per view per 10 minutes for the person | Yes. This is a declared coarsening for layout only. Alternative: record every one |
| HD-7 | A "pause history" switch | No in the first version |
| HD-8 | History in exports, AI reads, search | No in the first version |
| HD-9 | When history cannot be written, refuse the save (6.4) | Yes. The alternative, saving without history, creates silent gaps |
| HD-10 | Confirmation style for restore and for "remove for good" (A51 puts delete-for-ever in native dialogs) | Restore: in-window preview. Remove for good: native dialog, per A51 |
| HD-11 | HQ1 from draft 3 (every page version or coarser steps) | Still open; this design does not need it answered, but WP-04 does |
| HD-12 | Wording of the purge promise (8.3) | "Removed from this vault and its history. Copies you made earlier are not changed." |

### 13.2 Technical (for the dev thread to settle with evidence, then report)

| # | Question | Leaning |
|---|---|---|
| HT-1 | Make row change plus its automations one transaction; cost of the touched-entries rollback against the whole-vault clone (HF5) | Measure first |
| HT-2 | New table or tagged rows in History (4.1) | New table |
| HT-3 | Bulk-created rows without value images; the threshold (4.4) | 500 per transaction, tune by measurement |
| HT-4 | One record per transaction with overflow parts, against one record per database part | Per transaction |
| HT-5 | Fingerprint cost on large database objects; hash `props` only (9.1) | Yes |
| HT-6 | Rekey with an unbounded history (PG-7) | Measure; possibly staged rekey later, which is a security design question |
| HT-7 | Whether orphan cell values survive today (HF3) and what any migration must do for properties removed before the upgrade | Trace in code first; before the upgrade there is no record, so nothing can be promised for them |
| HT-8 | Whether nested automation writes carry the automation actor (6.2) | Trace in code |
| HT-9 | AI request identity for retries (6.5) | Trace in code |
| HT-10 | Chain over header core only (8.4) | Security review to confirm |
| HT-11 | Purpose of the Records writes at `vault/anchor.rs` 185 and `vault/mod.rs` 921 (E9) | Read before HV-03 is written |

## 14. Source coverage and limits

### 14.1 Read for this design (at `9977e72`, read-only, cloud copy)

| File | Lines read | For |
|---|---|---|
| `vault/store.rs` | 41-72, 81-85, 150-229 | Tables, settings, transactions |
| `vault/mod.rs` | 26, 86-128, 239-257, 291, 300-445, 540-632, 725-775, 842-850 | Doors, atomic action, actor scope, collection, rekey |
| `vault/history.rs` | 15-91, 93-333 | Versions, snapshot, restore, thinning |
| `vault/activity.rs` | 1-130, 236-460, function list, 852-960 | Entries, journal, undo plan |
| `vault/anchor.rs` | 151-166 | Anchor acceptance |
| `ai/dispatch.rs` | 192-219, 399-440, 758, 819-824, 1066-1090, change_kind arm | AI door |
| `database/saves.rs` | 29, 55, 167-184, 266 onward (partly) | Structure saves |
| `database/relations.rs` | all (143 lines) | Relation pairs |
| `database/types.rs` | struct lists, 37, 53, 374-422, 480-510 | Shapes |
| `commands/database.rs` | function list, 24, 305-318, 593-612, 689-887, 943-1094 | Row and database writes |
| `commands/automation.rs` | function list, 26-30 | Limits and call shape |
| `commands/import_notes.rs` | function list, 236-256 | Import save |
| `commands/history.rs`, `commands/activity.rs` | grep lines only | Command names |
| `state.rs` | 300-345 | Session lock and partial publishes |
| `crypto.rs` | 129-142 | Padding |
| `model.rs` | 31-52 | Item kinds, source tag |

Earlier facts reused from Studies 1a and 1b and drafts 1 to 3 were not re-read.

### 14.2 Not read, so UNKNOWN here

- The two screens (`HistoryDialog.tsx`, `ActivityPanel.tsx`) and all other front-end code.
- `clean_cell` in detail, the Files cell format, `save_database_core` beyond its opening, and whether any save strips orphan cell values.
- `vault/anchor.rs` 185 and `vault/mod.rs` 921 in context; the upgrade and backup code around them.
- `purge_undo_in_store` in detail; the person-burst mechanism in detail.
- The automation `do_action` body and which actor nested writes carry.
- The AI dispatcher's request identity; `commands/notes.rs` permanent delete (one line seen).
- The unmerged lane tips (N2, P6c, K1 family, S, W, R1).
- Notion's storage, and Notion's restoring of database structure (not tested by Blue).

### 14.3 Limits of this document

- Code-derived, not runtime-tested. No test, build, benchmark or app run was made for it. Every cost in section 9 is reasoning from code, and every goal is a proposal.
- The data model is a sketch at the level needed to judge the approach. Field names and tag bytes are placeholders for the dev thread.
- The existing tests named in passing were read by name only, not run.
- Nothing here is approved, and no version placement is decided.
- No repository, Notion page, roadmap artifact, PC folder or earlier study or draft was changed. No copying was done.

## 15. Working log (append-only)

- 2026-11-11T11:18:27Z (message stamp): request received via Blue.
- 2026-11-11T11:20:58Z to 11:25:07Z (system clock): targeted code reading; sections 0 to 4 written; checkpoint 1.
- Work stopped at a usage limit after checkpoint 1 (time not recorded).
- 2026-11-11T14:53:55Z (message stamp): continuation requested via Blue; same scope and restrictions.
- 2026-11-11T14:55:41Z (system clock): sections 5 to 7 written; checkpoint 2.
- 2026-11-11T14:57:11Z (system clock): sections 8 to 10 written; checkpoint 3.
- 2026-11-11T14:58:39Z (system clock): sections 11 to 15 written; structure and citation checks; top checkpoint set to DONE.
