# Entities, Properties and Native Views

## Identify What a Record Represents

Choose canonical ownership per genuine entity type before designing views. By default, databases have separate child-page surfaces and Home links to them. The explicitly selected [software-project recipe](software-development-project-hub.md) permits the milestone database/view inside Home; reuse its canonical source rather than copying records. Other entity/domain boundaries remain unchanged. Reuse the existing source when it already owns those items; relocation needs the access checks in [verification and operations](verification-and-operations.md). Relations connect entities without merging their meanings. Linked views select attention over the same source, never duplicate datasets. Keep operational records separate from setup/development tasks.

For an explicit software inside-Home request, shared source identity and matching rows are necessary preservation checks, not proof that native board columns/cards are visible on Home or that its tabs/card open-return work there. Apply the [mandatory rendered acceptance](software-development-project-hub.md) and keep an owner-reported presentation failure separate from source/data checks. Never duplicate an existing source or cards to make an uninspected layout repair.

| Domain | Entity / useful lens | Adaptation, not a universal requirement |
| --- | --- | --- |
| Software | Feature or release work; Version, Lifecycle, Area | Use versions only for real iterations. Accepted, requested planning, research prerequisite and future candidate remain distinct. A version group is not progress. |
| R&D / Local AI | Experiment; Phase, Status, evidence basis | Keep candidate, planned, running-unconfirmed, user-reported completion and measured acceptance separate. Model/hardware/settings belong only to relevant evidence. |
| Opportunities | Opportunity/application relationship; Stage, Fit, Priority, Freshness | One opportunity exists once as its relationship changes. Fit is not current visibility. Preserve historical records; submissions/outcomes remain real relationships. Do not import Connects/schedule fields into unrelated work. |
| Portfolio | Project/idea reminder; Type, Domain, Role, Status, Priority, Load | Registry classifies/reminds and links to the standalone hub. It is not the full project container. Load and human attention are subjective, not inferred technical scores. |
| Reading | Knowledge object; Status, Reading Mode, Resume From, Next Reading Step | Preserve Queue/Active/Paused/Stalled/Reference/Archived/Dropped distinctions. Optional progress must not fabricate precision. Reuse the existing reading system; properties may be enough. |
| Personal finance | Setup tasks, transactions, recurring obligations and possibly accounts | Treat each as distinct when actually needed. Start with the authorized workflow, not a prebuilt full finance system; do not create records or balances from speculation. |

## Minimal Work Schema

Adapt names and types to the existing schema. Add only useful fields; do not remove meaningful existing ones for uniformity.

| Concept | Typical Notion property | Job |
| --- | --- | --- |
| Identity | Title | One actual item, not a claim of progress |
| Lifecycle | Status or select | What is true now; preserve the existing property's type/semantics where practical |
| Grouping | Phase, Version or category select | Where it belongs; independent of lifecycle |
| Purpose/outcome | Rich text | Short intended or observed result, honestly labeled |
| Next action | Rich text | Concrete continuation or decision/gate |
| Source | URL, relation or native rich-text mention | Canonical evidence; preserve existing mentions and relationships |
| Evidence basis | Select when ambiguity warrants it | Unknown, proposed, user report, source report, observed/measured; acceptance needs its actual decision/evidence |
| Evidence date | Date | When evidence applies, not a fresh automatic edit timestamp |
| Area/type | Select/multi-select if useful | What kind of item/work |
| Priority | Select if it improves choice | Attention importance, not completion |
| Record quality | Existing select/status or a proposed field only if needed | Completeness/usability of the record, not item lifecycle or strength of evidence |
| Order | Number, usually hidden | Presentation order only |
| Example marker | Checkbox/explicit label for demos | Demonstration, never genuine progress |

The source starter uses **select** for Status and Phase; that is an observed implementation, not an instruction to convert every native Status property. Its four phases and six statuses are demonstration vocabulary, not required options.

## Schema-Aligned Dictionary and Operating Guide

Read the actual property types, exact option labels and saved view filters before writing the guide. A field primer makes the important questions visible; lower native toggles hold the richer dictionary and human/AI rules. Explain only the fields the system uses, with depth proportional to ambiguity rather than a mandatory taxonomy.

For each relevant property and option, keep its exact readable label (including established emoji), question/meaning, inclusion and exclusion rules, a small domain-appropriate example and the treatment of unknown or edge cases. Explain easily confused alternatives together. Use synthetic examples when distilling reusable guidance; never copy private people, amounts, transactions or raw notes into this skill.

- **Status/lifecycle:** what is happening to the item now? Not its importance or record completeness.
- **Quality:** is the record sufficiently complete/usable for its intended job? Not proof the event occurred or the task is done.
- **Evidence:** how do we know, from which source and as of when? A complete record can still contain an uncertain claim.
- **Priority:** what deserves attention? Not permission, status or evidence strength.
- **Classification:** what kind of item is it, or where does it belong? A category/phase is not progress.

These distinctions do not require five new properties. Map them to the real schema; keep absent dimensions in explanatory prose if sufficient. An empty value is unknown when evidence is missing, not zero, complete or a silently invented option. Mark every suggested field/option/view as **proposed — not implemented** until created and read back. Keep proposals separate from the actual dictionary and do not document them as current filters.

Explain each saved view by the question it answers and its actual filter (including AND/OR or explicitly none), with grouping/sort when meaningful. Reuse exact field/option labels and stable semantic emoji in the guide and view explanations; do not rename established options for decoration. Refresh the primer, dictionary and affected view explanations whenever authorized schema/filter changes occur, then verify against readback.

The operating guide should state where originals live, how sources remain linked/preserved, how uncertain/conflicting claims are marked, when new evidence updates an existing entity versus justifying a new record, and how suspected duplicates are reviewed without silently losing history. State human/AI responsibilities for the authorized workflow, including who can confirm facts, revise classification, change schema or perform imports. Creating a hub grants no standing authority for future actions. Describe AI processing as manual/on request unless automation is actually configured and verified; a written workflow alone is not an automation.

## View Contract

| View | Required meaning | Verify |
| --- | --- | --- |
| Work Board, primary when it serves daily work | Actual native Board/Kanban grouped by justified phase/status/category; not automatically the entry for operational records | Native type, grouping property/type, group order/empty-group behavior, filters, sorts, visible card fields and canonical source |
| Optional Gallery | Alternate browsing of the same entity records | Native gallery type, source identity and readable decision-useful properties; never substitute for requested Board cards |
| Current / next table | Small operational queue | Exact explicit filters and expected membership; only create when it answers a distinct question |
| All items / full data | Unfiltered source index | No simple/advanced filter restricting rows, same source and meaningful retained completed/stale/archived-category records |

A phase-driven source can group its primary Board by Phase with Status/Area/Outcome/Next action visible. A lifecycle workflow can instead group by Status. For operational records, choose the primary capture/review/resume view by the daily question; a setup board is a separate route, not the assumed entrance. Do not mechanically use the starter's Current OR Next filter for a domain with different stages.

Keep card display lean; full-table properties can expose detail. View names are not proof of type, filters or native behavior. Notion-native archived/deleted objects may require separate retrieval; an unfiltered ordinary view is not proof it includes those objects.

Calendar requires real dates, Timeline meaningful ranges, Chart an actual quantitative question. Do not create all possible views. In the inspected starter, optional Gallery contains simple-filter clauses without explicit values: inspect actual membership instead of assuming no effect. The All view has no filter clauses; the Board is genuinely grouped by Phase. This is a dated source observation, not a claim about every copy.

## Financial Adaptation From Sparse Inputs

For a sparse request such as "help me set up a personal finance hub," first establish whether the daily job is operational capture/review or planning the system. Inspect existing sources; do not infer a setup-board-first home from the word "setup."

- Compact standalone home: purpose, honest current baseline, next decision, a short workflow and field questions. Link the operational records/review surface prominently when that is the job; give setup/development tasks a separate route. Lead with a phase board only when the actual scope is setup project work.
- Every required database has its own child-page surface. Transactions, recurring obligations, accounts and setup tasks are distinct entities when needed; reuse canonical sources and avoid premature empty databases.
- A small Notes / Decisions room can combine work when useful. Use lower native toggles for a schema-aligned dictionary and operating rules rather than mandatory empty guide/archive rooms.
- Suggested setup phases such as Scope, Capture and Review or task statuses such as Proposed, Ready, Waiting and Done remain **proposed — not implemented** until saved and read back. They are not financial-record classifications or approved work.
- Preserve original sources and uncertainty; define duplicate/update handling and human/AI responsibilities before an authorized capture workflow. Choose source ownership before any import; hub creation alone does not authorize importing data or automatic future processing.
- Actual dates, currency, amounts, account names, obligations, periods and budgets stay unknown until supplied or explicitly authorized evidence establishes them. Do not replace missing amounts with zero.

Proposed task examples such as deciding capture format or required fields must be labeled proposed design, not records already saved in Notion. A starter demo is never a financial transaction. Keep financial-source/privacy questions proportional to the next decision, not a demand for all sensitive data up front.
