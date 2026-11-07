# Start and shape a project

Use this guide to choose a working path, establish the right source owners, and turn an idea into useful authorized work without creating unnecessary process. The template library is a set of options. It is not a requirement to instantiate every file.

The pack's release and validation status belongs in its main entry and release records. Copying a template creates a working document; it does not certify a project, approve its Plan, or establish owner acceptance.

## 1. Choose the smallest sufficient entry

### A small, clear, bounded task

Use the [Direct Executor bootstrap](../../../assets/templates/direct-executor-bootstrap.md) if the executor needs role guidance, and the [Task Brief](../../../assets/templates/task-brief.md) when a durable assignment is useful. A clear request can already supply the task's authorization; record it accurately rather than adding a ceremonial approval step.

A bounded task has a specific input or target, a useful output, a manageable consequence, and a clear check. Examples include inspecting a file, correcting a stated defect, producing a one-off analysis, or transforming a supplied sample. A full project architecture, roadmap, and session archive are not prerequisites.

Stop and rescope if the task becomes a persistent service, an ambiguous program, a larger commitment, or a meaningful change in risk. Do not silently turn “make a useful sample” into “build a platform.”

### An unclear, consequential, or persistent project

Use the [Coordinator bootstrap](../../../assets/templates/coordinator-bootstrap.md). Begin with discovery, existing reality, and the smallest useful version. Introduce a separate executor only when execution capability or a distinct workspace helps.

The coordinator owns understanding and integration, not the user's decisions. The same assistant may coordinate and execute when authorized, but should make the active role clear.

### Existing work that needs explanation or a better structure

Use a named Hybrid pattern:

- **Retrospective understanding:** Inspect what already exists, then explain, review, document, or integrate it. Do not invent a historical plan or approval that was never recorded.
- **Iterative reasoning and execution:** Run bounded authorized experiments, use their results to improve the design, and return consequential direction changes for review.

The original sources used “Hybrid” in both senses. Naming the actual pattern prevents ambiguity. Neither means “execute an unapproved idea and justify it afterward.”

## 2. Inspect before designing a new system
Read the actual request, relevant existing material, and applicable workspace instructions. Identify what works, what is unfinished, and what the owner wants preserved. Check actual access and available tools rather than assuming a particular environment exists. For execution-dependent work, confirm the exact task identity, actual environment, tools, connection and permissions before claiming capability. Use the “🖥️ Verify the execution environment” decision procedure in [System Handbook](system-handbook.md); preserve the difference between a cloud worker and a local connected executor. Inventory runtime evidence read-only, classify cloud/owner-local/unknown without guessing, then follow its routing table and stop conditions. Missing UI tools alone do not establish cloud execution.

Answer five practical questions:

1. What useful outcome does the owner want, and for whom?
2. What exists already, and what must remain intact?
3. Which assumptions or unknowns could materially change the route?
4. What is the smallest useful result, and how could it be checked?
5. What may happen now, and what still requires a decision?

Aspirational goals describe intent. An approved execution Goal defines an actual bounded authority. Do not blur them merely because both use the word “goal.”

## 3. Assign source ownership by question

Use the [Source Ownership Map](../../../assets/templates/source-ownership-map.md) when the work has multiple relevant records or surfaces. Identify one canonical owner for each distinct question. Choose the actual useful source for the project; do not impose a universal app hierarchy.

For example, a knowledge-work project's operational database may own its live queue, while a Markdown document owns durable architecture. A software repository may own implementation while a review record owns acceptance. Both arrangements can be coherent. Neither app owns all truth by default.

If two sources disagree, identify the question and its assigned owner. Do not settle the conflict by copying both answers into more places. Source migrations need an explicit old-to-new ownership map, appropriate recoverable preservation, and readback at the intended destination.

Use documents only when their questions need distinct durable answers:

- **README:** What does someone need to know quickly to enter or use this?
- **Architecture:** What is this project, how is it structured, and where are its boundaries?
- **Methodology:** How do we repeatedly work and make decisions?
- **Roadmap:** What are the phase purposes, risks, dependencies, and exit gates?
- **Plan:** What route is proposed or being adapted for this piece of work?
- **Goal / Task Brief:** What outcome and actions are actually authorized, and within what limits?
- **Evidence:** What was observed, under what conditions, and what does it support?
- **Acceptance:** What did the relevant reviewer/owner actually accept?
- **Current / Continuation:** What present-state question does this project need answered, and where can work safely resume?
- **Execution Summary:** What did the executor actually do and verify?
- **Session Summary:** What happened, why, what changed in understanding, and what should future readers learn?
- **CHANGELOG:** What meaningful version or milestone changes affect users or maintainers?
- **Agent Entry:** What persistent workspace-specific instructions must an execution agent follow?

One physical record can hold clearly separated sections for a small project. The requirement is one canonical owner per question, not one file for every label. Split records when different update rates, readers, or consequences make a single record confusing.

## 4. Review the Plan before establishing the execution Goal

For substantial new work, use this practical sequence:

1. **Intent and discovery:** Establish the useful outcome, existing state, boundaries, and unknowns.
2. **Proposed Plan:** Describe the route, alternatives, dependencies, risks, evidence needed, and stopping conditions. Label it a proposal.
3. **Review and amendments:** The owner and appropriate reviewers test the approach, make choices, and record changes. Preserve material rejected alternatives and reasons.
4. **Approved Goal:** Record the bounded outcome, allowed actions, limits, acceptance conditions, stopping conditions, and actual approval source. Reference the reviewed Plan and amendments.
5. **Adaptive implementation:** Let the implementation route change within the approved boundary. Keep the Plan current; do not ask permission for every ordinary in-scope implementation choice.
6. **Material amendment:** If the outcome, permission, cost limit, external commitment, consequential risk, or acceptance boundary changes, pause the dependent action and obtain the required decision.
7. **Evidence, reconciliation, and acceptance:** Check the actual result, update only affected owners, distinguish technical success from owner acceptance, and stop at the agreed boundary.

Use the full [Plan](../../../assets/templates/plan.md) and [Goal](../../../assets/templates/goal.md) templates where warranted. A small Task Brief can combine the approach, actual authority, and checks. An unreviewed Plan does not authorize execution; an approved Goal does not require an inflexible implementation script.

## 5. Copy and adapt the raw files

Each file in the package-relative `assets/templates/` shelf is the reusable artifact itself. Copy the entire file, then:

1. Replace every placeholder, whether written as `{{placeholder}}`, `<Placeholder>`, or `[Placeholder]`, with a real value or an explicit “Unknown,” “Not applicable,” or “Not yet agreed.” An unresolved placeholder is not a fact or approval.
2. Remove non-applicable repeat blocks and empty optional categories, while preserving the document's identity and method.
3. Replace references with the project's actual canonical owners. Links in this guide navigate the template pack; references inside copied artifacts must point to the instantiated project records.
4. Use the actual owner and assistant identities. Do not copy another person's name, invent a model, or pretend the assistant has a pre-existing relationship.
5. Record only known dates, versions, checks, and approvals. The package manifest owns the template-pack version. A copied project record owns its own identity and actual status.
6. Read the result as a newcomer: can the reader find the next useful action, distinguish proposed from approved, and identify what remains unknown?

Do not copy the guide's teaching notes into every live artifact. The templates already contain their intrinsic purpose and change rules. Keep this guide available for deeper explanation.

## 6. Grow documents when the work earns them

Start from the questions the task must answer, then add records as a real trigger appears:

- A recurring handoff or future maintenance need justifies a README and possibly Agent Entry.
- Components and interfaces that become hard to explain justify Architecture.
- Repeated procedures or decisions justify Methodology.
- Meaningful phase choices and exit conditions justify a Roadmap.
- Material work worth reconstructing justifies a durable Execution Summary.
- A substantial session with evolving reasoning, corrections, or friction justifies the full Session Summary.
- A release or milestone that changes useful behavior justifies CHANGELOG.

Do not create empty documents to satisfy a fixed file count. Do not erase useful depth because routine input should be small. Smallest sufficient context means a short reading path with deeper material available, not an artificially shallow history.

The [full Session Summary](../../../assets/templates/session-summary.md) is a separate, substantial artifact with its established section identity and newest-first chronology. A task return, Current note, or compact checkpoint is not a substitute. This guide deliberately does not introduce a shortened Session template.

## 7. Keep durable structure separate from live work

Architecture changes when architecture changes. Methodology changes when recurring practice changes. Roadmap changes when phase purpose, progression, risks, or gates change. Their update triggers are deliberately slower than a live queue.

A new current fact does not belong everywhere. Before updating a document, identify its owning question and confirm that the fact affects it. Work one document at a time: read, restate its purpose, preserve useful content, make the scoped change, inspect the diff, and read back the destination. See [Maintain without drift](maintain-without-drift.md).

Session history has a different job. It may preserve temporary context, abandoned alternatives, cross-project influences, friction, and earlier states because they explain the dated reasoning. Do not impose Architecture's narrowness on the Session Summary or rewrite old entries to pretend later knowledge was known earlier.

## 8. Verify a usable handoff before stopping

At the end of the agreed work, ensure the next person can distinguish:

- requested outcome from actual result;
- proposed approach from approved authority;
- prepared artifact from saved and destination-verified artifact;
- historical evidence from evidence applicable to the current artifact;
- technical completion from actual owner acceptance;
- useful suggested successor from authorized next work.

After interruption, inspect actual state and running operations before retrying. Preserve completed-but-undocumented work; perform the missing verification or reconciliation rather than rebuilding it. A missing report does not prove nothing happened.

Stop when the agreed result and stopping condition are reached, or pause the dependent action for a necessary owner decision or unavailable authority. Do not turn “keep the project recoverable” into an endless documentation program.

## 9. Short synthetic walkthroughs

### A bounded data cleanup

The owner requests a separate normalized copy of a sample list while preserving the original. The task brief records the input, allowed normalization, output, and checks. The executor inspects it, makes the separate copy, verifies row count and representative values, and returns the artifact plus limitations. No Architecture, Roadmap, or full Session Summary is needed unless something materially more complex happens. Deduplication, source replacement, and external upload are separate scope questions.

### A persistent reference project

The owner wants a reference system for an audience whose needs are not yet clear. The coordinator clarifies the readers and smallest useful topic, then proposes a Plan. Review narrows the first edition and chooses its evidence standard. The approved Goal authorizes that bounded result. Architecture explains the durable intake/assessment/synthesis structure; Methodology explains repeated source checking and review; Roadmap defines expansion gates. The operational queue owns today's articles. The result is accepted only through the actual reviewer/owner decision.

### Recovering work after a pause

The continuation note says a draft was being written, but inspection shows the artifact exists and the destination is readable. The returning executor checks the artifact and its version, finds that quality review is missing, and completes that review. It does not regenerate the draft merely because the old note still says “in progress.” The execution return and current-state owner are reconciled; deeper session history preserves how the interruption was resolved.

## 10. Template-specific teaching notes and lineage

The following notes explain how to write each artifact well and how it relates to the inherited foundation. Lineage identifies preserved methods and explicit adaptations; it does not claim universal empirical validation. Public-safe examples are synthetic. The raw templates contain no historical personal identities, private source links, private project facts, or machine-specific paths.

## Template notes: Coordinator / Mother Thread bootstrap

### Document contract

- **Question owned:** How should a coordination conversation establish understanding, choose a working mode, and turn an idea into bounded, reviewable work?
- **Use:** Start or re-establish coordination for ambiguous, consequential, architecture-heavy, or persistent work. A small, clear task can use the [Direct Executor bootstrap](../../../assets/templates/direct-executor-bootstrap.md) and [Task Brief](../../../assets/templates/task-brief.md).
- **Readers and writers:** The owner supplies intent and constraints; the coordinator investigates, proposes, and preserves understanding. The owner approves consequential choices.
- **Change trigger:** Change this instantiated bootstrap when the coordination role, collaboration arrangement, or project-level decision boundary changes. It is not a live work queue or a substitute for project records.
- **Related owners:** Architecture owns enduring project identity; Methodology owns recurring practice; Plan owns the proposed/adaptive route; Goal or bounded Task Brief owns execution authority; Session Summary owns the reasoning history.

### Before copying

Replace every `{{placeholder}}` with an actual value, or explicitly write `Unknown`, `Not applicable`, or `Not yet agreed`. Do not invent a previous relationship, assistant identity, model, permission, or access capability. A friendly role name is not a claim to be a specific person or model. Supply a short initial idea; discovery can resolve the rest.

**Role and permission modernization:** The original Mother Thread emphasized understanding before implementation. This derivative preserves that role while making the modern authority boundary explicit: planning does not grant execution authority, an AI coordinator cannot grant itself or its workers new authority, and an approved Goal can permit adaptive implementation within its stated limits. The actual platform's safety, permission, and confirmation requirements always apply. No PBS record overrides them. If the owner's request already authorizes a clear bounded task, record that authority rather than asking ritualistically for the same permission again.

**Lineage:** Adapted from Mother Thread Bootstrap v2.0, Documentation System Overview v2.0, Universal Codex Operating Rules v2.0, and Codex Layer Architecture v1.1. Preserved: all core responsibilities, the five project classes, discovery questions, mode selection, documentation/version-history philosophy, scope checks, and first-action sequence. Added: explicit Plan review → approved Goal, project-specific source ownership, evidence/acceptance separation, and precise Hybrid variants. Personal identity and historical project examples have been replaced with portable inputs.

### Small sanitized example

For a shared reference guide with unclear audience, use coordinator-led discovery to settle the readers and first useful edition. Propose the outline and review process before authorizing production. Do not create a repository, database, automated publisher, or full project archive merely because those tools exist. See the startup workflow above for the decision flow.

**Raw reusable file:** [coordinator-bootstrap.md](../../../assets/templates/coordinator-bootstrap.md)


## Template notes: Direct Executor bootstrap

### Document contract

- **Question owned:** How should an execution-capable assistant start and run bounded work when no separate coordinator is leading every step?
- **Use:** A direct task or persistent direct project; also useful for a clearly assigned execution slice inside coordinator-led work.
- **Readers and writers:** The owner supplies intent and authority. The executor inspects, proposes where needed, implements within scope, verifies, and returns an honest result.
- **Change trigger:** A change to the executor's role, work mode, environment boundary, or authorization arrangement. Do not update the bootstrap after every task.
- **Excludes:** Live progress, long-term strategy ownership, fabricated permission, and the full reasoning archive.

### Before copying

Supply the actual environment and authorized boundary. References must be accessible in that environment. Unresolved fields do not become permission. Use a [Task Brief](../../../assets/templates/task-brief.md) for a small task; a substantial project should use the pack's Plan review and Goal records. Read the applicable project agent-entry instructions and task-specific sources, not the entire archive by default.

**Role and permission modernization:** The original Direct Codex Bootstrap v2.0 explicitly required owner approval for repository creation, commits, history changes, and pushes. This derivative does not silently preserve or discard that policy. Record the actual project envelope and follow the platform's confirmation rules. Bounded authority may cover ordinary permitted implementation actions, but never invent a blanket waiver or infer publication permission from permission to edit. The same distinction applies to dependencies, installations, deletion, sensitive data, and external communications.

**Lineage:** Adapted from Direct Codex Bootstrap v2.0, Universal Codex Operating Rules v2.0, and Codex Layer Architecture v1.1. Preserved: direct task/project distinction, inspection-first behavior, scope discipline, proportional documentation/reporting, version-history rationale, escalation, stop-and-ask triggers, and clear communication. Added: explicit approval provenance, adaptive Plan limits, source ownership, recovery, and acceptance boundaries. Historical personal identities and examples are replaced.

### Small sanitized example

A request to normalize a supplied sample dataset is a direct task if its input, output, preservation boundary, and validation are clear. A new reusable import service with ongoing maintenance is a direct project. The first does not silently authorize the second. See the startup workflow above.

**Raw reusable file:** [direct-executor-bootstrap.md](../../../assets/templates/direct-executor-bootstrap.md)


## Template notes: Bounded Task Brief

### Document contract

- **Question owned:** What specific task may be done, within which limits, and what result is sufficient?
- **Use:** A small stand-alone task or a bounded assignment under an existing Goal. It can replace a full project scaffold when the work is short-lived, clear, and easy to verify.
- **Readers and writers:** The requester/owner approves the boundary; the coordinator or executor prepares the brief; the assigned executor performs and reports the work.
- **Change trigger:** The requested result, scope, material risk, permissions, inputs, or acceptance condition changes. Record amendments without pretending the earlier version authorized them.
- **Excludes:** A whole project's identity, roadmap, exhaustive history, and permission that the requester has not actually given.

### Writing method

Write an observable outcome, not “improve everything.” Specify preserved behavior and excluded targets as carefully as editable ones. A task brief can contain the proposed approach and approval in one place, avoiding unnecessary Plan/Goal ceremony. For a child assignment, the parent Goal remains the authority owner: link its applicable boundary rather than creating a second independent grant. The brief can narrow that boundary but cannot expand it.

An explicit user request may already authorize the task. Record the actual request and any limits; do not require an extra approval ritual when the instruction and platform rules are sufficient. Conversely, “draft” does not authorize sending, and an unchecked template is not approval. If the work becomes persistent, ambiguous, or materially larger, propose a reviewed [Plan](../../../assets/templates/plan.md) and [Goal](../../../assets/templates/goal.md).

Keep this brief stable during execution. Record work performed in the Execution Summary or return. Reopen the brief only for a real amendment. Link to applicable source owners rather than copying live queues or history.

**Lineage:** The brief operationalizes the task-brief purpose in Codex Layer Architecture v1.1 and the bounded-scope, inspect-first, verify/report rules. Its exact schema is a new consolidation, not a claim that an identical historical template was already validated.

### Sanitized example

“Create a separate normalized copy of the supplied sample list, preserve the original, trim surrounding whitespace, retain every row, and return row-count and spot-check evidence” is a bounded task. Deduplicating rows, replacing the source, or uploading the output elsewhere would need the corresponding authority; none follows automatically from the normalization request. The example describes scope, not a real approval.

**Raw reusable file:** [task-brief.md](../../../assets/templates/task-brief.md)


## Template notes: Project agent entry / AGENTS guidance

### Document contract

- **Question owned:** What persistent, project-specific instructions must an execution agent follow in this workspace?
- **Use:** A persistent workspace with repeat agents, non-obvious conventions, important boundaries, or a recurring verification workflow. Place the adapted artifact in the agent-instruction file supported by the actual environment, such as `AGENTS.md` when applicable.
- **Readers and writers:** Execution agents read it; the project maintainer changes it under the project's approval rules.
- **Change trigger:** A durable workspace convention, supported command, source owner, permitted boundary, or recurring verification requirement changes.
- **Excludes:** The current task, copied Goal authority, full Architecture, daily status, secrets, or a task backlog.

### Writing method

Make instructions actionable and specific to this workspace. Replace example paths and commands with inspected, verified values. Do not claim commands were tested if they were not. Keep a short read path, link deeper owners, and state when deeper context is needed. The project source map owns authority by question; this entry points to it.

Do not encode a permanent broad permission merely because one earlier task allowed an action. A recurring workflow may explain how to do something without authorizing it now. Unknown permissions remain unknown. Platform instructions and confirmation requirements continue to apply.

**Lineage:** Derived from the AGENTS role in Documentation System Overview v2.0, Codex Layer Architecture v1.1, and the seven Universal Codex Operating Rules v2.0. Adds explicit recovery, source-ownership, worker-integration, and documentation change controls. The exact reusable entry schema is new.

### Sanitized example

A workspace entry may say, “After changing an import rule, run the fixture comparison and preserve the original fixtures.” It should link the actual fixture procedure and task authority. It should not embed today's import request, declare all future imports approved, or copy the entire project roadmap.

**Raw reusable file:** [agent-entry.md](../../../assets/templates/agent-entry.md)


## Template notes: Project README / entry page

### Document contract

- **Question owned:** What does a new reader need to know quickly to understand, use, or enter this project correctly?
- **Use:** A persistent deliverable or project that needs a clear front door. It can be a Markdown README or an adapted project home.
- **Readers and writers:** Users, collaborators, and returning owners read it; the project maintainer updates it.
- **Change trigger:** Purpose-facing explanation, supported use, entry route, prerequisites, access instructions, or important limitations materially change.
- **Excludes:** A live task dashboard, full Architecture, duplicated source map, session archive, release-by-release history, or unverified completion claims.

### Writing method

Lead with what the project does and for whom. Give a real first-use path with prerequisites, steps, expected result, and known limits. For non-software work, “use” may mean opening the correct working surface and following its intake or review procedure. Avoid software-only boilerplate.

Keep one canonical source map. This README points readers to it and the minimal record set; it does not become a second ownership register. Name only documents actually instantiated. If the README is public, omit private references and details; public and internal entry points may be separate records with different audiences.

**Lineage:** Preserves the quick-orientation/public-facing README role in Documentation System Overview v2.0 and the usable-result reporting expectations in the Direct/Executor sources. This exact schema is a new reusable derivative.

### Sanitized example

A reference collection's first-use path could be “Choose a topic, open its maintained guide, and check the guide's evidence date before reusing a claim.” It does not need installation commands or a repository architecture tree. A software utility may instead need a short verified install/run/check sequence.

**Raw reusable file:** [project-readme.md](../../../assets/templates/project-readme.md)


## Template notes: Project Architecture

### Document contract

- **Question owned:** What is this project, how is it durably structured, and where are its boundaries?
- **Use:** When components, roles, interfaces, information flow, or enduring scope need explanation across sessions or collaborators.
- **Readers and writers:** The owner, maintainers, reviewers, and new collaborators read it. A designated maintainer proposes updates; consequential structural changes follow the project's decision process.
- **Change trigger:** Identity, purpose, core components, durable responsibility allocation, interfaces, information structure, or architectural boundaries materially change.
- **Excludes:** Current tasks, current rates or metrics, today's collaborator roster, phase progress, active experiments, proposal details, cross-project implementation trivia, and session chronology.

### Writing method

Explain the structure in prose before enumerating components. “Architecture” applies to software, research, operations, and knowledge work. Components may be modules, workflows, records, services, or human roles. Explain responsibility and boundaries, not merely a folder tree.

Separate current architecture from a proposed future architecture. Label a proposal and link its decision/Plan owner rather than presenting it as built. Link the source map for authority by question. Link Current State for progress, Roadmap for future phases, Methodology for recurring practice, and decisions for why major tradeoffs were chosen.

Use a diagram only if it clarifies a real interface or boundary. Do not use decorative complexity to imply a more mature system. Preserve durable rationale; move dated execution history to its proper owner instead of deleting understanding.

**Lineage:** Preserves the Architecture contract in Documentation System Overview v2.0 and the role-layer distinctions in Codex Layer Architecture v1.1. The narrow update rule and prohibition on status overlays derive from the recorded document-ownership repair. This schema is new; no specific historical project's structure is copied.

### Sanitized example

In a research reference system, “source intake → evidence assessment → maintained synthesis → reader-facing guide” may be a durable architecture. “Three articles awaiting review today” belongs in the operational queue. A new evidence-assessment layer changes Architecture; a new article normally does not.

**Raw reusable file:** [project-architecture.md](../../../assets/templates/project-architecture.md)


## Template notes: Working Methodology

### Document contract

- **Question owned:** How do we repeatedly make decisions, test ideas, execute, learn, and refine this project?
- **Use:** When recurring work benefits from a shared method, especially repeated collaboration, evaluation, production, experimentation, or review.
- **Readers and writers:** The owner and collaborators use it; the method maintainer proposes changes based on evidence and experience.
- **Change trigger:** A recurring procedure or decision rule materially changes, a repeated failure requires a prevention rule, or an adopted improvement changes how future work should be done.
- **Excludes:** Today's tasks, current contributor status, one proposal's details, publication receipts, a chronology of incidents, and another project's machinery copied wholesale.

### Writing method

Write procedures that someone can actually follow: trigger, inputs, steps, check, decision, and stopping condition. Explain why the method exists and when not to use it. Transfer principles without importing unnecessary infrastructure.

Keep incident detail in Session Summary or Failure Intelligence and the resulting recurring prevention rule here. Keep a selected approach for a particular task in its Plan. The source map owns where truth lives; Methodology owns how to use and maintain those owners correctly. Agent Entry may point agents to these procedures and add workspace-specific commands without copying the full method.

**Lineage:** Builds on the Methodology role in Documentation System Overview v2.0; the seven Universal Codex Operating Rules v2.0; and the recorded durable cycle of construct → test → compare → human judgment → controlled refinement → temporary lock. Incorporates the generalized one-document-at-a-time ownership repair and later bounded Goal/Plan/evidence/acceptance distinctions. The exact schema is newly consolidated.

### Sanitized example

A research method may require every reusable claim to carry a source date and a freshness review trigger. A particular stale article belongs in the session or incident record; the reusable “check freshness before reuse” rule belongs here. The method does not need another project's Git scheduler, model roster, or recovery harness.

**Raw reusable file:** [working-methodology.md](../../../assets/templates/working-methodology.md)


## Template notes: Project Roadmap

### Document contract

- **Question owned:** Where can this project go, what does each phase accomplish, and what must be true before advancing?
- **Use:** When progress has meaningful phases, dependencies, learning goals, or strategic choices beyond one task.
- **Readers and writers:** The owner and collaborators read it; the owner or authorized planning role proposes changes to phase design and priorities.
- **Change trigger:** Phase purpose, sequence, dependency, risk, strategic direction, or exit gate changes.
- **Excludes:** Current task lists, daily status, current proposals, collaborator onboarding details, a dated progress overlay, or copied contents of the operational dashboard.

### Writing method

For each phase, explain its purpose, useful outcome, uncertainty it resolves, risks, dependencies, and evidence needed to exit. A phase is more than a list of features. Distinguish an exit criterion from evidence that it has actually been met: the Roadmap owns the criterion; the current-state/acceptance owner records the actual transition.

A roadmap expresses direction, not authorization to execute every phase. Use an approved Goal or Task Brief for the next bounded work. A changed implementation route belongs in the Plan unless it changes phase structure or strategy. Avoid fabricated deadlines; if timing is material, state whether it is a target, external constraint, or actual commitment and link its owner.

**Lineage:** Preserves the phase/milestone/sequencing role in Documentation System Overview v2.0. Adds phase purpose, learning, risks, and exit conditions retained in the recorded ownership repair. The exact reusable schema is new; it does not import any historical project's phases.

### Sanitized example

A reference project might progress through “prove one useful topic → establish repeatable editorial practice → widen coverage.” The first phase can exit when a representative reader successfully uses the guide and its claims pass the agreed source check. Which topic is being edited today belongs in the operational owner, not as an extra Roadmap status section.

**Raw reusable file:** [roadmap.md](../../../assets/templates/roadmap.md)

## Template notes: Changelog

### Document contract

The Changelog owns user- and maintainer-relevant version or milestone evolution. It changes for meaningful released or completed changes, not every tool action. Execution Summary owns implementation detail; Session Summary owns reasoning history; Evidence and Acceptance own checks and decisions.

### Writing method

Write what changed for the reader and why it matters. Include migration or compatibility consequences. Keep unreleased completed work distinct from speculative backlog. Preserve release history newest-first; do not silently renumber inherited originals into a new product's version scheme. A release label is not a substitute for actual acceptance or verified publication.

Use only applicable categories. “Prepared,” “verified,” “accepted,” and “published” are separate claims even if they eventually describe the same artifact. Link their actual owners rather than copying a large evidence dossier into the Changelog.

### Lineage and example

The role follows Documentation System Overview v2.0: version and milestone evolution. The explicit acceptance/publication distinction and historical correction rule are added controls. The exact schema is a new reusable derivative.

For a synthetic reference guide, a useful entry might say that the source-review checklist now flags outdated claims before publication. The individual edit operations and abandoned wording belong in execution/session history.

**Raw reusable file:** [changelog.md](../../../assets/templates/changelog.md)
