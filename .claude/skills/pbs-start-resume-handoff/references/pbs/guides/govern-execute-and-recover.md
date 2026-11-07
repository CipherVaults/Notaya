# 🎯 Govern, Execute & Recover

## 🧭 Purpose and status

This guide explains how to turn an intended outcome into bounded execution, inspectable results, and a recoverable stopping point. It accompanies the PBS v1.0 execution/control template library. It is for a human owner, a coordinating assistant, an executor, or a receiving worker who was not present for the original conversation.

**Publication state:** candidate material, pending empirical integration, bounded usability validation, and owner acceptance. Authored templates are not proof that a whole operating system has been validated or adopted. The blank records deliberately contain no release claims and no prefilled approvals.

The design consolidates established distinctions between authority, strategy, evidence, acceptance, accepted state, and interrupted work. The exact portable field sets, combined-use guidance, and review workflow here are newly authored synthesis. Project-specific permissions, environments, preservation arrangements, tool choices, and review machinery do not transfer with them.

Use these records when they answer a real question. A library is not a required folder tree. A short task may need a few labeled paragraphs in an existing task record; a long or consequential project may need separate, versioned owners. The aim is enough structure to govern and recover the work while preserving useful understanding.

Related guidance: [System Handbook](system-handbook.md), [Maintain Without Drift](maintain-without-drift.md), and the [Source Ownership Map](../../../assets/templates/source-ownership-map.md).

## 🗂️ What each record owns

A canonical owner is the location responsible for a question, not a claim that one application outranks all others. A person or role maintains that location. Keep these two meanings of “owner” explicit.

| Record | Owning question | Create separately when | Keep elsewhere |
|---|---|---|---|
| [Goal](../../../assets/templates/goal.md) | What outcome is authorized, by whom, within which limits, and where must execution stop? | The scope, duration, consequences, or approval history needs a durable contract | Day-to-day step changes belong in the Plan |
| [Plan](../../../assets/templates/plan.md) | What is the current route and why? | Discovery, alternatives, dependencies, or strategy changes need explanation | Permission belongs to actual approval and the Goal |
| [Plan Review & Amendment](../../../assets/templates/plan-review-and-amendment.md) | What was reviewed, what changes were proposed, and what was actually decided? | Review has material findings or a change needs explicit approval | Advisory recommendations cannot supply owner approval |
| [Evidence](../../../assets/templates/evidence-record.md) | What was observed, under what conditions, and what does it still support? | Proof needs durable identity, multiple consumers, or later reassessment | Acceptance decisions belong in Acceptance |
| [Acceptance](../../../assets/templates/acceptance-record.md) | Which approved requirements are satisfied by applicable evidence, and who accepts that scope? | Multiple claims, integrated results, or required human decisions need tracking | Raw reports remain in Evidence; future plans remain in Plan |
| [Current State](../../../assets/templates/current-state.md) | What is the accepted present and which authority is active? | Readers repeatedly need a compact project orientation | Dirty work and detailed resume steps belong in Continuation |
| [Continuation State](../../../assets/templates/continuation-state.md) | What survived, what is unaccepted, and what should be inspected and done next? | Interruption, handoff, pending work, or costly reruns create recovery risk | It does not replace accepted state or live observations |
| [Handoff & Worker Contract](../../../assets/templates/handoff-and-worker-contract.md) | What bounded responsibility is being transferred, against which baseline and limits? | Work crosses a person, agent, thread, or execution context with meaningful risk of lost assumptions | The parent retains unassigned integration and acceptance responsibility |
| [Worker Return](../../../assets/templates/worker-return.md) | What did the assigned worker actually establish and what must the parent integrate? | A material delegated result would otherwise be trapped in transient context | A return does not certify the whole project |

These records complement, rather than replace, the [Task Brief](../../../assets/templates/task-brief.md), [Agent Entry](../../../assets/templates/agent-entry.md), [Execution Summary](../../../assets/templates/execution-summary.md), [Execution Index](../../../assets/templates/execution-index.md), and [Session Summary](../../../assets/templates/session-summary.md). Execution history explains what happened; session memory preserves how reasoning, decisions, friction, and understanding evolved. Neither should be converted into a copy of the control records.

## 🪶 Full, combined, and minimal use

### Choose by the questions, not by a file count

Use the full templates when their sections prevent real ambiguity. Separate a record when it has a different maintainer, update rhythm, audience, approval history, evidence lifecycle, or recovery use. A record can be a Markdown file, a page, a database item, or a named section in an existing document.

For a small bounded task, combine responsibilities in one existing record:
- State the requested outcome, actual approval source, limits, and endpoint.
- Give the route if it is not obvious.
- Record what actually changed and the scoped check.
- Distinguish the result from its acceptance and any owner decision still needed.
- Leave an exact continuation or an honest terminal state.

This is combined use of responsibilities, not a competing short-form Session Summary. Preserve the established Session Summary identity and depth when a substantial session warrants one.

### How to use a full template without creating bureaucracy

1. Copy the record whose question needs an owner. Give it a durable identity and an actual responsible role.
2. Replace descriptive placeholders with relevant facts, decisions, or explicit unknowns. Do not leave a placeholder in a record presented as ready for execution.
3. Remove sections whose trigger does not exist. For example, omit pending workers when there are none and omit recovery automation when nothing is configured. There is no requirement to create a worker, process, scheduler, backup system, or reviewer to fill a field.
4. Keep distinctions that matter even in a combined record: permission versus observation versus acceptance; implemented versus verified versus integrated; actual owner decision versus pending decision.
5. Link to the canonical owner rather than copying long bodies. A brief labeled snapshot can orient the reader, but include its source and as-of state.
6. Preserve a recoverable prior version before a material rewrite when the consequence warrants it. Do not impose the same preservation method on every task.

A full template is a complete writing aid, not a schema that must be mechanically populated. If an irrelevant section is removed, its absence should not obscure a real permission boundary or an outstanding requirement. When a required condition does not apply, record the basis once in its owner rather than scattering unexplained “N/A” entries.

## 🔄 The practical lifecycle

### 1. Discover the real problem and the live starting point

Recover the human's intended outcome, intended audience, useful existing work, constraints, and important unknowns. Inspect relevant sources before proposing a rebuild or treating old notes as current. Establish what discovery is authorized. Reading and planning do not automatically authorize implementation, purchases, publication, or other consequential changes.

The [Source Ownership Map](../../../assets/templates/source-ownership-map.md) should identify where answers come from. A project can use a live Notion database for operational records, Markdown for durable method and history, and another tool for delivery. A software project can use repository state and build outputs for different observed questions. There is no universal local-file-over-Notion rule.

### 2. Prepare a proposed Plan

Describe a credible route, not just a wish list. Include observed starting state, alternatives actually considered, dependencies, proportionate checks, preservation needs, and decision points. Keep unverified assumptions visible.

The proposed Plan allows the owner and reviewers to understand what execution would involve. It does not authorize itself. If the user's existing request already grants a clear bounded action, refer to that authority; do not manufacture an extra ceremonial approval cycle.

### 3. Review, question, and amend

Human review and AI review have different functions. AI can examine feasibility, contradictions, scope, evidence coverage, and alternatives. The human owns purpose, material priorities, consequential tradeoffs, and the approval decisions reserved to them.

Record findings and exact proposed amendments. Distinguish a recommendation, a requested change, an actual decision, and incorporation of that decision into saved records. “The reviewer approved the approach” is not owner approval unless that reviewer really has authority over the decision in question.

### 4. Establish the approved bounded Goal

The Goal turns the reviewed outcome into an execution contract: scope, limits, requirements, assurance, recovery needs, and endpoint. Record the exact approved revision and the actual approval source. A Goal can incorporate an earlier Plan with specific amendments; if so, identify both precisely.

Do not rewrite the approval source or treat a prefilled status as approval. No wording in a template overrides the executing system's safety or confirmation rules. Later explicit instructions may pause, replace, or narrow the authority; check them before resuming.

### 5. Execute using an adaptive Plan

Within the approved envelope, the executor chooses and revises the route. Ordinary implementation changes can remain Plan changes when they preserve the approved outcome, constraints, permissions, and acceptance requirements.

A material scope expansion, weakened criterion, newly consequential action, changed resource commitment, or product-direction decision needs the appropriate amendment or new authority. Record the blocked dependent step and continue independent authorized work when useful. A recoverable defect is not automatically a reason to stop the whole Goal.

Treat an adopted milestone order as an implementation constraint, not a backlog to spread work across. Close the active milestone against its actual criteria, evidence and required decision before advancing; do not invent another approval when standing authority already permits the transition. Authorized read-only preparation and ordinary within-milestone adaptations remain allowed. If work is blocked or oversized, preserve working and partial results, distinguish the unmet condition from a recoverable defect, and propose the smallest useful uncertainty investigation or scope amendment. A bounded spike requires applicable authority and a stopping boundary; it is not feature completion. On an explicitly authorized deferral or reorder, check dependencies and revised acceptance, keep unfinished functionality out of the stable result, and preserve its evidence, unknowns and return trigger. Deferred is not complete. Use existing records only within writing authority; a read-only task returns findings without creating or updating them.

### 6. Produce evidence and integrate the result

Capture inspectable observations at useful boundaries. Preserve failures and important partial results. If workers are used, integrate their baselines, outputs, assumptions, and evidence. If a single executor does all the work, still check the resulting whole where the outcome requires it.

Only affected records need updating. A synchronization check can be a short readback for a small document change or a deeper integration review for a complex system. Its purpose is agreement among material facts, not compulsory edits to every file.

### 7. Decide acceptance, reconcile state, and stop

Assess the approved requirements against applicable evidence. Separate technical assessment, integrated-result checks, delivery where required, and human/product acceptance where required. Record honest limitations and obtain authority before weakening a requirement.

Update Current State only with the accepted scope. Put unaccepted work and exact recovery steps in Continuation. Preserve enough execution and session history to explain the meaningful work. At the approved endpoint, stop. Neither a completed Goal nor spare capacity authorizes a successor.

## 🔐 Three truths and four result states

### Three truths

- **Permission truth:** the owner's actual instructions and active approved contract, subject to applicable system rules. This answers what may happen.
- **Observed truth:** actual records, artifacts, files, service state, outputs, and retained observations. This answers what exists or happened.
- **Acceptance truth:** a requirement-to-evidence judgment by the relevant decision owner, including required human decisions. This answers what counts as satisfying the contract.

A live artifact can disprove stale prose about whether work exists. It cannot make unauthorized work authorized. A Goal can permit a change. It cannot prove the change succeeded. An acceptance decision can recognize a scoped result. It cannot turn an obsolete test into proof of a newer artifact.

When sources conflict, ask which question each owns. Preserve the contradiction, inspect the relevant reality, establish the affected claim, and correct that owner. Do not pick the newest timestamp or declare one application universally authoritative.

### Four result states

- **Implemented or produced:** the work exists in an identifiable form.
- **Verified within scope:** appropriate checks were performed and support specific claims.
- **Integrated:** the result has been reconciled with its intended context and dependencies.
- **Human/product accepted:** the required human decision was actually made for the stated scope.

These are distinct assertions, not four compulsory ceremonies. A straightforward task may establish several in one inspection. A source test may establish component behavior without proving a delivered package works. A worker can be ready for integration while the Goal remains incomplete. When human review is not required for the bounded outcome, record its approved basis rather than inventing a human acceptance event.

## ✍️ Shared section-writing method

### Write claims that can be used

Use concrete nouns, observable conditions, and bounded verbs. “The draft exists in the assigned destination; references were checked against the supplied sources” is more useful than “All done.” State what a check did not cover when that limit changes how the result should be used.

Use actual dates and time zones where timing matters. Do not invent a precise time, model, reviewer, source revision, or tool identity. “Not recoverable from the available record” is a legitimate fact. An inference should name its basis and remaining uncertainty.

### Distinguish a placeholder, an unknown, and a pending decision

- A **placeholder** is a writing prompt in a blank template. Replace or remove it before use.
- An **unknown** is a genuine unresolved fact. State its effect and how to resolve it if necessary.
- A **pending decision** is an action owned by a person or authorized role. Name the decision and its owner.

A blank approval field means no approval is recorded. It is not permission. “Pending” should not hide whether the work is waiting for evidence, integration, human judgment, or an external condition.

### Keep one factual owner

Repeat only a concise, dated orientation when it materially helps. The Goal owns requirements; Acceptance points to the exact version. Evidence owns observations; Acceptance reports their relevant outcome and applicability with references. Current State links acceptance; Continuation describes work beyond that accepted baseline.

Long rationale belongs where it answers the question. A Plan should preserve why a route was chosen. A review record preserves decision rationale. A full Session Summary can preserve deeper reasoning evolution without becoming the permission record.

### Preserve history without freezing living state

Correct a mistaken current statement transparently. Retain the earlier evidence or version when it matters. A failed report remains a failed observation even after a successful correction. Use an addendum or subsequent evidence for the correction. Do not retroactively rewrite a historical account to imply later knowledge was already available.

## 🎯 Writing the Goal

**Audience and maintenance:** the human owner, executor, reviewer, and future replacement executor. Create a distinct Goal for material bounded execution. Update it when authority, scope, criteria, constraints, or endpoint actually changes; routine route changes belong in the Plan.

- **Identity:** make the Goal recognizable independently of the current conversation. Separate the outcome owner from the executor and optional strategic reviewer. Keep the status proposed until actual approval exists.
- **Intended outcome:** describe the change in the user's situation and how it will be used. Avoid substituting a tool installation or document count for the desired result unless that is genuinely the outcome.
- **Scope and preservation:** name both included work and tempting adjacent work that is excluded. Identify useful existing content or behavior before modifying it. Surface assumptions that could change feasibility or risk.
- **Authorization and limits:** record the exact approval source, revision, targets, and conditions. Populate action categories only from actual approval. Include resource limits where material. Do not invent a broad waiver or copy another project's permissions.
- **Governing sources:** link the actual contract and source map. Separate permission conflicts from disagreements about observed facts. A methodology reference is guidance, not a hidden amendment to a project's approved contract.
- **Success requirements:** use stable requirement IDs when there are several. Write one observable condition per meaningful requirement, identify the evidence needed, and name who owns qualitative judgments. Avoid criteria that can pass while the user's real outcome fails.
- **Human/product boundary:** specify the actual review expected and the material the owner will see. If no separate owner decision is required for the bounded check, state the approving basis. Delivery and publication permissions are separate from drafting.
- **Strategy and amendments:** define which choices can adapt and which alter the contract. Record approved amendment IDs and effects; do not silently edit away difficult criteria.
- **Assurance and recoverability:** connect checks and preservation to actual consequences. State when enough evidence is enough. A practical record should allow useful work to survive interruption without committing the project to unnecessary machinery.
- **Endpoint and history:** distinguish complete, waiting, blocked, paused, and superseded states. Record actual authority changes and preserve the superseded revision. Completion never grants a new Goal.

## 🗺️ Writing the Plan

**Audience and maintenance:** executors, reviewers, and the human owner evaluating the route. Write it during discovery when strategy is material. Update after findings change the route or a meaningful step changes state; do not rewrite the whole Plan after every action.

- **Identity and authority link:** identify the proposed or active Goal and actual planning basis. A Plan can be reviewed without being authorized for execution. Keep those states separate.
- **Starting point:** report an inspected baseline, existing work worth reusing, and uncertainty. Do not present inherited inventory or an old document as a new observation.
- **Outcome alignment:** trace the route to the Goal's requirements and constraints. Link rather than duplicate the full contract.
- **Options and approach:** preserve alternatives genuinely considered and the reasons for selection. Avoid invented deliberation or a long catalogue with no decision. Explain what new evidence would change the choice.
- **Work sequence:** use coherent outcomes, not every keystroke. Identify dependencies, responsible owner, expected output, readiness check, and authority boundary. A step marked proposed cannot be mistaken for completed work.
- **Interfaces and integration:** identify assumptions that multiple parts rely on. Delegation is optional. If it is used, assign disjoint writes or explicit coordination and keep one integrating owner.
- **Verification:** map important requirements to appropriate checks. Identify what can be reused and why. Make the distinction between source checks, combined behavior, delivered artifacts, and owner use where it matters.
- **Risk and contingency:** identify realistic failures and permitted fallbacks. A fallback that changes scope or consequence is a decision request, not a hidden authorization.
- **Recovery:** pick meaningful checkpoint boundaries and expensive or stateful work that must not be repeated blindly. Link the one continuation owner.
- **Change history:** describe the new finding, revised strategy, and whether it stays inside the contract. Preserve important rejected routes and evidence effects without turning the Plan into the execution diary.

## ⚖️ Writing the Plan Review & Amendment record

**Audience and maintenance:** reviewers, the human decision owner, and the person incorporating the decision. Create it when a material proposal or amendment needs a traceable decision. Keep a small review in the Plan or task record when no separate owner is needed.

- **Identity:** bind the review to exact Plan and Goal revisions. State whether the reviewer is advising or deciding a question they actually own.
- **Decision requested:** ask a concrete question and identify work that can proceed while it is pending. Do not hold unrelated authorized work unnecessarily.
- **Review basis:** disclose inspected and unavailable sources. Evaluate intent alignment, feasibility, authority, preservation, evidence coverage, and proportionality to the depth this decision warrants.
- **Findings:** each finding needs a basis, consequence, responsible owner, and actual disposition. Distinguish a defect from an optional preference. Resolution means the response was established, not merely promised.
- **Amendments:** state exact before/after text or precise conditions. Name effects on permission, risk, outcome, and evidence. Proposed amendments are not effective until the required decision exists.
- **Recommendation and decision:** keep an AI recommendation separate from an actual human decision. Record which items were accepted, rejected, or deferred and any conditions. Approval of planning does not by itself approve execution.
- **Incorporation:** derive the effective requirements from the approved original and applicable amendments; update affected tests, artifact scope, acceptance and continuation, including legitimately retired criteria. Preserve prior criteria and results with their original applicability. Verify that the approved change reached the intended records. Prepared text, attempted write, saved change, and destination readback are different states. If readback is unavailable, say so.
- **History:** preserve withdrawals, corrections, and superseding decisions. A changed contract should be reconstructable without guessing which draft was approved.

## 🧾 Writing Evidence

**Audience and maintenance:** the executor, integrator, acceptance reviewer, and anyone reusing the observation. Create a separate evidence record when later use needs more than a nearby source link or short check result. Preserve the original observation and add later applicability assessments rather than rewriting history.

- **Identity:** use a stable ID when other records cite it. Distinguish when something was observed from when the report was written. Record the actual observer or source.
- **Question and scope:** say what the observation can establish, the expected condition when there was a test, and what it excludes. A document search is not a test of the deployed result.
- **Identities and conditions:** record only what changes the interpretation. A document revision and inspected section may suffice for an editorial check. A software or measurement claim may depend on source, configuration, input, runtime, or artifact identity. Do not demand hashes or full environment dumps without a reason.
- **Method and result:** state what actually happened. Keep observation separate from interpretation. Use “not-run” when no check occurred, “inconclusive” when the available check cannot decide, and “fail” when it observed a failure against the criterion.
- **Inspectable evidence:** retain or link the raw basis and material transformations. Use locators accessible to authorized reviewers. Do not copy secrets or unrelated sensitive data into a portable record.
- **Applicability:** compare against the target now being judged. An old result can remain valid for an unchanged claim even after unrelated changes. A new timestamp alone proves neither identity nor applicability.
- **Corrections:** retain failed observations, explain changed interpretation, and link new evidence. The Evidence record informs acceptance but does not make that decision.

### Outcome, applicability, and decision are independent

| Dimension | Values | Meaning |
|---|---|---|
| Outcome | pass, fail, inconclusive, not-run | What the observation or check found |
| Applicability | current, historical, partially-invalidated, unavailable | Whether and how that evidence supports the target now being judged |
| Decision | pending, accepted, rejected | The relevant acceptance owner's judgment about a requirement |

“Historical” means retained as a record of the earlier state and not being asserted as current support. “Unavailable” means the needed source cannot presently be inspected; it does not prove failure. “Partially-invalidated” requires naming the affected claims and the claims still supported. For mixed evidence, use narrower claim records or explain the split instead of inventing a misleading overall green status.

## ✅ Writing Acceptance

**Audience and maintenance:** the integrating owner, technical reviewer, and human outcome owner. Create it when requirements or acceptance layers warrant an explicit ledger. Update after meaningful verification, integration, required human decisions, or a material change that affects an accepted claim.

- **Identity and contract:** identify the precise approved Goal and target being assessed. Acceptance of one revision is not a perpetual certificate for later changes.
- **Requirement decisions:** trace each material requirement to the source clause and specific evidence. Include the observed outcome and current applicability as referenced snapshots. Explain coverage, gaps, and the decision's scope.
- **Decision ownership:** record who made the decision, when, and on what basis. A reviewer may offer a recommendation without owning product acceptance. Never fill in the owner's decision based on silence or enthusiasm elsewhere.
- **Maturity and integration:** distinguish produced, checked, integrated, and delivered states. Identify material worker reconciliation only when workers were used. Check the intended combined context where the requirement calls for it.
- **Record reconciliation:** resolve material contradictions among the actual result, evidence, contract, and affected current records. Do not turn the gate into edits to every document.
- **Human/product acceptance:** record the actual required review, its material, decision, scope, and conditions. If the Goal did not require a separate human decision for this bounded outcome, document that basis instead of fabricating one.
- **Overall disposition:** an honest partial result can be valuable without being complete. A residual limit is not permission to weaken an unmet requirement. Route that change to an actual amendment decision.
- **Reassessment:** scope invalidation to changed claims. Preserve the historical decision and identify what must now be checked. Do not re-run the whole project for an unrelated change.

## 📍 Writing Current State

**Audience and maintenance:** anyone who needs the project's accepted present quickly. Create it when repeated orientation is useful. Update on accepted milestones, authority changes, important applicability changes, and mission-state transitions.

- **Identity and state:** state an actual as-of basis and mission condition. If the accepted baseline is older than the latest inspection, show that distinction.
- **Accepted present:** make only scoped accepted claims and cite their acceptance/evidence. Before the first accepted result, state that no accepted baseline has yet been established. State limitations without blending unaccepted progress into the baseline.
- **Current authority:** point to the actual active Goal and amendments. An attractive next phase remains proposed until authorized.
- **Operational orientation:** identify the project's actual live operational source. Link unaccepted work and next steps to their owners. Keep only the immediate decision or blocker that explains the state.
- **Reconciliation:** record what was checked and material disagreement still unresolved. Link dated history rather than retaining every past status inside this living record.

Current State is not a backlog, full inventory, narrative history, or list of dirty artifacts. A newer Continuation checkpoint does not overwrite an accepted claim merely because it is newer; a new observation can require a scoped reassessment of that claim.

## 🧷 Writing Continuation State

**Audience and maintenance:** the next executor or the same executor after interruption. Create it when work can outlive the current context or when repeating actions would be costly or harmful. Update at meaningful checkpoints, impending interruptions when possible, handoffs, material state changes, and closure.

- **Identity:** record actual mission state, authorization, accepted baseline, writer, and next integrating owner. A thread name alone is not a durable result or approval source.
- **Last meaningful work:** locate surviving outputs and explain the delta beyond the accepted baseline. Do not retell the full session.
- **Recovery inventory:** classify only relevant items. Link observable state and evidence, useful work to preserve, and the missing step. Classifications can differ across items.
- **Pending workers or processes:** include this section only when something is actually pending. Identify the external operation and last observed state so a replacement can inspect before retrying.
- **Blockers and contradictions:** state the exact missing fact or decision and its owner. Separate blocked dependent work from work that remains authorized.
- **Do-not-repeat list:** include costly, stateful, externally visible, or completed work at genuine risk of repetition. Record the inspection needed to decide whether another action is necessary.
- **Next action:** provide an executable inspection-and-resume sequence. Include a stop boundary. “Continue work” is insufficient; a frozen command that ignores changed reality can also be unsafe.
- **Recovery mechanism:** omit it unless already configured. If one exists, identify actual state and authorization. A requested pause is not a verified pause, and an old note cannot prove a scheduler is active.
- **Minimal references:** point to the compact entry route and only the deep context needed next. Preserve deep knowledge elsewhere rather than deleting it for easier routine reading.

## 🤝 Writing Handoffs and Worker Returns

### Handoff & Worker Contract

**Audience and maintenance:** the receiver and integrating parent. Use the contract for a bounded worker, a read-only reviewer, or a successor executor when responsibility crosses context. Revise it if the assignment, source baseline, interfaces, or authorized write ownership materially changes.

- **Identity and result:** name one coherent question, expected deliverable, and completion boundary. Make the assignment kind explicit; a reviewer does not silently become an implementer.
- **Authority and ownership:** link actual approval, authorized reads/writes, shared contracts, protected owners, and limits. Assign non-overlapping writes where possible. A parent can delegate only within its own authority.
- **Checkpoint:** provide the relevant baseline, accepted state, partial work, uncertainty, and work not to repeat. The receiver verifies access and relevant live state rather than assuming the handoff is current.
- **Interfaces:** make important assumptions and output format explicit. Give a conflict response that preserves work and resolves ownership before conflicting writes.
- **Required return:** ask for the useful result first, supporting evidence, actual changes, limits, and partial-work continuation. The contract should not require an elaborate report for a trivial lookup.
- **Endpoint and parent gate:** state what remains with the integrator. A successor handoff must identify who now owns integration; it must not leave two incompatible current-state owners.

A read-only review is useful when a distinct question benefits from another perspective. It is not a mandatory role for every task. Parallelism helps only if its results can be recombined.

### Worker Return

**Audience and maintenance:** the receiving integrator and any replacement who must recover the work. Write it before treating a material assignment as finished. Update or add a correction when a finding materially changes; preserve the earlier observation where relevant.

- **Identity and result first:** reference the exact assignment and state whether the result is ready for integration, partial, blocked, or legitimately needs no change. Lead with the useful finding and any qualification that changes its use.
- **Coverage:** distinguish requested from actual baseline and name uninspected material. Do not claim complete inspection because a directory or source list was available.
- **Work and outputs:** identify actual changed targets, important substitutions, surviving partial work, and deviations. Reporting a deviation does not retroactively authorize it.
- **Evidence:** make each important claim inspectable with outcome, applicability, and limits. A worker's confidence is not a substitute for proof of the assigned question.
- **Integration notes:** report interface assumptions, conflicts, affected canonical owners, and the smallest useful parent check. Keep whole-project acceptance with its assigned owner.
- **Recovery:** preserve failures, missing steps, and actions that must not be repeated blindly. If blocked, state the exact action, target, and missing authority or access.
- **Parent receipt:** the parent fills the actual reconciliation and disposition. A worker must not prefill the parent's acceptance. Parent receipt may be a short acknowledgment plus integration evidence when that is sufficient.

## 🧩 Executing and synchronizing without drift

At a meaningful boundary, ask:
1. What changed in reality, permission, evidence, or acceptance?
2. Which existing record owns each changed fact?
3. Are any worker assumptions, outputs, or claims inconsistent with that reality?
4. What proportionate check establishes the combined result?
5. Which records need a targeted update, and what must be preserved?
6. Was the update actually saved and verified at the intended destination?

Read the destination's purpose and relevant content before editing. Update one owner deliberately; inspect the change and read back before moving on when migration or scope-drift risk warrants it. Do not spread one recovered fact into Architecture, Methodology, Roadmap, Current State, and history just because all those documents exist.

Common boundaries include a material worker return, a verification outcome, an authority change, a coherent checkpoint, recovery, acceptance, and closure. Small routine edits can share one reconciliation. Important state should not drift across multiple execution cycles. If a documentation worker is behind, the integrating executor keeps the existing small checkpoint accurate with the last confirmed result, active action, next gate and blocker. Mark deeper reconciliation pending; a brief checkpoint does not replace an adopted substantive Session. Reconcile a late return against newer facts before promoting it.

## 🧷 Recovery: inspect before replaying

The routine entry route is: project entry guidance → Current State → active Goal and amendments → Continuation → relevant live operational state → needed Plan, Acceptance, Evidence, and technical or domain context. This is a reading route, not an application hierarchy.

Check the latest authority before resuming. A later pause or superseding instruction changes current permission while the older approval remains historical. Reconcile the current-facing authority chain, including lower instructions and pending queues, rather than adding only a banner. Use one identified existing current entry to reach the actual Goal/decision, artifact, evidence frontier and next action; retain detailed owners rather than copying every fact into that entry. Reconcile only the known affected alternative entry routes. Mark a misleading old pause or queue historical with its later decision pointer, without erasing its source. A newly edited status label does not create owner intent.

For the same confused thread or a successor, provide the smallest verified correction: applicable authority, actual artifact, unfinished step and limit. A concise receiver readback can expose a material conflict; it is not a new approval form. A genuine newer stop still controls. Then inspect what actually survived, including pending operations. A saved narrative may lag behind a completed output. Conversely, an output may exist without having passed the required check.

| Recovered state | Appropriate response |
|---|---|
| Accepted and still applicable | Preserve it. Do not repeat costly work merely for reassurance |
| Complete but unreconciled | Verify identity and the missing evidence/integration steps; update the affected records |
| Implemented but unverified | Run the missing scoped check; do not rebuild simply because acceptance is absent |
| Partial | Preserve useful work, inspect its state, and continue the true unfinished part |
| Uncertain or externally changed | Preserve first, identify ownership and relevant change, then choose a permitted next step |
| Evidence unavailable | Record the gap and its consequence; consider retrieval, narrower claims, or a justified rerun rather than assuming failure |

These classifications are observations, not permissions. A partial unauthorized action should be preserved as appropriate and reported, not continued because it is already underway. Never blindly reset, overwrite, repeat a transaction, resubmit a message, or rerun a costly process based only on stale prose.

If an authorized recovery mechanism exists, keep its actual state visible. Do not rely on a reminder or scheduler merely because a document mentions it. Avoid removing the only recovery route while unfinished closure work still depends on it. When shutdown is part of authorized closure, preserve a durable checkpoint, perform the action within authority, and verify the external state. If verification is impossible, record that exact limitation. PBS does not require installing or scheduling a recovery mechanism.

## 🧪 Validate the actual use path

A candidate can pass synthetic recovery exercises while its real checkpoint still tells the next actor to repeat completed work. Check the actual current checkpoint, source identities, lower current-facing instructions and pending queues after a meaningful reconciliation. Keep the synthetic trial and real-state validation as distinct evidence.

Derive the useful consumer operation from the Goal: for example, open the delivered copy, reach the intended item and use its required attachment or output. Include the dependency hops that matter to this result, not an exhaustive universal journey. A wrong-but-existing target or a package that works only in the producer folder can pass inventory checks. Preserve those narrower checks and verify the missing consumer claim.

Bind a result to the user's actual entry route and configuration. A test launched through a different shortcut, profile, account, workflow or input can be valid for that route while failing to establish the user's intended experience. Preserve the old result within its valid scope and test only the missing claim.

A verifier should test the intended invariant. For example, a required reference set needs exact unique membership; a matching row count can hide one duplicate and one missing source. A justified negative control should fail the particular guard it is testing. Do not manufacture a large test harness for a simple task or recursively inspect verification machinery without a concrete gap.

Record whether a finding is a fresh observation, a prior recorded result, an owner report/decision, an attributed collaborator result or an inference. Reading a past test report does not rerun the test. Matching hashes establish content identity, not factual truth or acceptance.

An unmet mandatory criterion remains pending or rejected unless its authorized owner explicitly accepts an exception or changes the requirement. “Accepted with limits” cannot silently waive it. Keep aesthetic/usability acceptance scoped to what the owner actually reviewed; it does not accept unrelated engineering checks or authorize publication.

## 🛡️ Proportionate assurance and stopping

Choose assurance by consequence, uncertainty, external exposure, reversibility, and cost of a wrong claim. A reversible correction may need a source comparison and destination readback. A public release or consequential operation can require stronger evidence and separate approvals. Tool availability is not a reason to run every possible check.

An assurance boundary should answer:
- Which important claims need proof?
- What evidence is sufficient for those claims?
- Which failure would justify deeper investigation or independent review?
- Which uncertainty can be disclosed, and which prevents acceptance?
- When would further checking mostly inspect the checking machinery rather than the user outcome?

Do not recursively verify a verifier without a concrete defect or consequence that warrants it. Do not stop on a recoverable failure while an authorized route remains. Do stop at a real approval boundary, explicit pause, supersession, genuine external block, or the approved endpoint.

Mission states have bounded meanings:
- **active:** an authorized next action exists.
- **quota-wait:** authorized work remains, but compute is temporarily unavailable.
- **approval-wait:** the next dependent action requires an actual decision.
- **paused:** an explicit instruction has intentionally stopped the work.
- **blocked:** no permitted route remains without an external change.
- **superseded:** a newer authorization replaces this mission.
- **complete:** the contract's required endpoint and acceptance conditions are satisfied.

A request to wait, a changed condition, or a pending approval should not be mislabeled complete. A proposal for the next stage can be reported without executing it.

## 🧪 Synthetic examples of proportional use

The following examples are fictional teaching cases. Their approvals, results, and identifiers are illustrative; they do not authorize or verify any real work.

### A. One bounded document correction

An owner asks: “Correct the mislabeled heading in the internal guide and check the saved result.” The request identifies the target and action. The executor need not create nine files or ask for the same approval again.

A combined task record can say:
- **Goal:** correct the specified heading in the named guide; preserve the surrounding content; the owner's request is the approval source.
- **Plan:** inspect the heading in context, make the targeted correction, then read back the destination.
- **Evidence:** the saved heading matches the requested wording; the surrounding section is unchanged within the inspected scope.
- **Acceptance:** the bounded edit and readback satisfy the requested condition; no separate publication or broader rewrite was requested.
- **Current/Continuation:** the correction is complete; no pending action remains under this request.

The record should use actual references and observations in real use. If the write succeeds but readback is unavailable, record “saved; destination verification pending” and preserve the next inspection step. Do not claim complete verified delivery.

### B. A knowledge-work tracker with Notion as operational truth

An owner authorizes updating a project tracker with source-confirmed schedule changes and explicitly excludes sending invitations. The tracker is the live operational owner. A methodology document explains how updates are made; it is not the live schedule.

The proposed Plan identifies the source records to inspect and how ambiguous changes will be handled. Review clarifies that only confirmed dates can be changed. The approved Goal contains that boundary. Evidence identifies the relevant source record and the actual tracker state. Acceptance checks that only supported updates reached the intended items. Current State points to the tracker; Continuation holds any unresolved source conflict.

A newer Markdown note does not override the tracker merely because it is local or newer. An incoming third-party request does not supply the owner's authorization to send invitations. The executor can finish permitted tracker work and leave the separate decision explicit.

### C. A recovered artifact ahead of the checkpoint

A continuation note says a deliverable is unfinished. Live inspection finds a complete artifact and a retained check report, but the report applies to an earlier version. The executor classifies the artifact as complete-unreconciled and the new changes as unverified within the affected scope.

The correct next step is to compare relevant identities, preserve the artifact, and perform only the missing check. The old pass remains historical evidence. Acceptance stays pending for the changed claims until applicable evidence and integration exist. If the Goal requires a human usability decision, that decision remains pending even after technical checks pass.

No worker, scheduler, destructive reset, or complete rebuild is required merely to demonstrate that recovery exists.

## 🧭 Adoption and validation boundary

Before treating an adopted project adaptation as operational, test its important claims at an appropriate scale. A bounded tabletop exercise can be enough to check whether a fresh reader finds the right authority, identifies accepted versus unaccepted work, and chooses the next permitted action. A consequential environment can need more.

Useful candidate checks are:
- Can a reader distinguish a proposed Plan, a reviewed Plan, and an execution-approved Goal?
- Can they identify the exact approved revision and an effective amendment without guessing?
- Can they tell implementation, verification, integration, and required human acceptance apart?
- Can they recover useful work that is ahead of the checkpoint without blindly replaying it?
- Can they identify evidence that passed earlier but no longer covers a changed claim?
- Can they use a live knowledge-work source without inventing a universal application hierarchy?
- Can they finish a small task without creating unnecessary records or reviewers?
- Can they stop at the actual endpoint and leave successor work unapproved?

Record what was actually tried, what was merely inspected, and what remains unverified. Structural completeness, empirical usefulness, integration with an existing project, publication, and owner acceptance are separate milestones. This guide provides a candidate method; adoption should preserve that honesty.
