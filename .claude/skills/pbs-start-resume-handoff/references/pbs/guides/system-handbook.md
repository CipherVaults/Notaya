# 🧭 How PBS works

## 🎯 The problem it solves

An AI can produce work faster than its owner can follow every implementation detail. That can be useful, but it creates several different risks: the work may drift from the original intent; a later assistant may not know what is actually finished; tests may prove less than the report implies; and a working result may become difficult for its owner to understand.

PBS preserves a practical answer to each problem:

- **Intent and authority:** what outcome is wanted, and what may be done?
- **Observed reality:** what exists and what actually happened?
- **Acceptance:** which claims have sufficient applicable evidence and the required decision?
- **Recovery:** what survives an interruption, and what may happen next?
- **Understanding:** how does the result work, and why did the choices evolve?

These answers can fit in one small record or occupy several documents. The point is reliable ownership of the answers, not a fixed number of files.

## 👥 People, collaborators and tools

The **owner** decides the purpose, priorities, consequential tradeoffs, scope and final product acceptance.

A **coordinator** or Mother Thread helps turn incomplete ideas into coherent outcomes. It preserves the reasoning context, compares options, reviews plans, explains results and maintains continuity. It can challenge assumptions. It does not become the owner, and it is not a compulsory relay for every routine action.

An **executor** investigates practical details, chooses an approach within the approved scope, creates the result, verifies it, records evidence and keeps progress recoverable. It may use specialist workers when they are genuinely useful.

A **worker or reviewer** receives a bounded question, source baseline, ownership boundaries and expected return. Its completion is an input to integration, not proof that the entire project is accepted.

The user need not name PBS roles or design an agent team. The working thread normally owns integration; add coordination/workers only when useful. A single AI may fill more than one role during a simple task. The responsibilities still remain distinct. A new assistant uses its actual identity and capabilities. A predecessor's role or success claim does not create authority. Verify and carry forward the owner's actual still-valid bounded authorization, subject to later instructions, actual access and applicable action-specific confirmation requirements. Changing assistants alone does not require reapproval. Reconcile unverified memories against sources.

## 🖥️ Verify the execution environment
Before relying on a capability, identify the exact task/thread and where it actually runs. A cloud coordinator or native worker is not interchangeable with a local connected executor.
- **Identify whose computer and browser:** dot or a cloud-native worker may have its own cloud filesystem/browser; that access does not establish access to the owner's PC, local files or authenticated local Chrome session. Verify the actual tools and attachment/connection state for the selected task rather than inferring them from a cloud/local label.
- **Separate sign-in, connection and authority:** a Notion sign-in screen in the cloud browser says nothing about whether the owner's local browser is authenticated or the PC is online. Check those states through the supported local route. A connected local executor still needs the applicable authorization for the specific local operation; connection alone is not permission.
- **Check the route:** confirm task identity, actual environment, available tools, live connection and applicable permissions. A familiar task title or an attached computer does not establish all five.
- **Prove the bounded claim:** when capability is uncertain, use the smallest authorized test with a clear stopping point. Keep the actual operation receipt and, where relevant, the owner's observation. A finished thread alone does not prove that the requested operation succeeded.
- **Keep substitutions visible:** evidence from another executor or environment supports only that route. Never present it as success for the requested route; report unavailable, not-run, blocked and completed distinctly.
- **Respect execution-time denials:** carry the owner's still-valid approval accurately, but do not bypass a denial or silently switch routes to evade it. Preserve staged work and backups, report the exact blocked action and target, and obtain the approval or handoff the executing system requires before dependent work resumes.
### 🔀 Route a verified execution block
Use the actual caller–tool–target capability and permission, not a cloud/local label alone. Cloud Git, files or a browser can be sufficient for a cloud target without providing access to the owner's repository, Chrome session or desktop. Conversely, do not force a local handoff for a cloud-sufficient task. Reuse an applicable verified restriction with its source/time/target scope instead of repeating unchanged doomed probes; refresh only the material prerequisite when tool/target/connection evidence changes. This never relaxes a denial or its recovery requirements.

**Historical workflow example · 2026-10-16:** the source owner reported unsuccessful cloud-native attempts to operate his PC and used a capable local executor instead. This is preserved experience, not a recipient's automatic capability default or a universal product limit. Apply a current route only from the relevant verified target evidence. A task being directly messageable does not establish desktop capability or permission to contact arbitrary owner-local conversations. Use only a verified supported connected-executor route; otherwise, when owner action is needed, provide the handoff for the owner to create or open the appropriate local thread.
### 🤖 Executor preflight: observe, classify, route
Apply this decision procedure before dependent execution. It is operating guidance inside the owner's actual task authority, not permission to run a new test.
1. **Inventory read-only facts first.** Record the requested target (cloud resource, owner's file/code, local browser UI or whole desktop UI), exact task identity, runtime-provided environment/target metadata, available tool inventory and reported connection/enablement state. Keep the source and observation time. A task title, successful message delivery or missing UI tools is not enough to classify the runtime.
2. **Classify with evidence: cloud, owner-local or unknown.** Use explicit runtime metadata tied to this task and target. If it is absent, ambiguous or conflicting, keep the classification **unknown**. Missing browser/desktop tools can also mean a local setup failure; do not infer cloud from their absence.
3. **Resolve only facts needed for the target operation.** Unknown execution-runtime classification is not itself a blocker when the requested resource, supported tools and permission are already established. If uncertainty about whose computer/session is involved blocks the requested operation, inspect supported metadata first; ask the smallest useful owner question only when necessary. A product-specific fallback, where the current UI supports it, is: “Does Codex show a cloud icon beside this thread's name?” If needed, ask for a screenshot of that label, excluding unrelated private content. Record an owner-reported icon as supporting, version-sensitive UI evidence; its absence does not prove owner-local execution. If evidence still does not settle the target access needed for the action, keep that gap unknown and pause only dependent operations rather than guess.
4. **Select the route using the table below.** For the requested target, inspect each necessary capability separately: file read and write, supported browser/session control, or whole-desktop screen/mouse/keyboard tools. Check connection, tool/extension enablement, relevant account authentication and the exact permission scope. Local identity does not guarantee any of these capabilities.
5. **Preflight only the unresolved capability needed for the task.** Prefer existing receipts and read-only inspection. If a harmless bounded operation is genuinely necessary and authorized, define its target, expected observation and stop condition before running it. Do not move the pointer or click arbitrarily as a generic connectivity test. Record the actual result; task completion alone proves no operation.
| Observed state | Next action | Stop condition / evidence required |
| --- | --- | --- |
| Verified limitation: the selected runtime cannot perform the required operation on the owner-local target | Reuse that applicable limitation; do not retry an unchanged unsupported operation. Use a verified supported authorized local recipient if available, or prepare the exact missing handoff/prerequisite. | Resume the dependent action only on a verified capable route within authority. New direct-access evidence can change the route; a label alone cannot. |
| Required target access and operation capability verified, whether cloud or owner-local | Check each needed operation's tools, connection, enablement, authentication and permission. Perform only the approved bounded work or necessary authorized preflight. An unresolved runtime label alone does not negate established target access. | Require an applicable operation receipt/artifact check and any relevant owner observation. Stop at a denial, missing prerequisite or the agreed boundary. |
| Confirmed owner-local; required capability unavailable or fails | Diagnose the concrete setup, extension, authentication, permission or technical blocker. Keep the local classification unless new identity evidence contradicts it. | Do not repeat an unchanged failure. Allow at most one bounded retry for that failure after a verified correction, only where current authority and tool rules permit; otherwise checkpoint and request the specific missing action. |
| Unknown or conflicting target access needed for the operation | Resolve the concrete missing fact using supported metadata or a focused owner question. | No dependent action until the needed target capability and authority are established; unrelated authorized work need not stop. |
**Failure discipline:** distinguish a capability block from an approval denial. Preserve staged configuration and backups. A denied operation stops; follow the executing system's approved recovery or owner-confirmation route. The retry allowance above never overrides a denial, an action-specific confirmation requirement or a stricter tool rule, and does not authorize switching routes to evade the block.
### 🧪 Read-only reasoning walkthroughs
These are scenario checks of the decision logic, not runtime tests or new evidence of platform capability:
- **Cloud without the required owner-local access:** metadata identifies the worker's cloud resources but no supported route to the requested owner-local UI. Output: local operation not-run; use a verified authorized local recipient or report the exact unavailable route. Messaging alone does not establish desktop control. If the task instead concerns an accessible cloud repository, use that sufficient route without an unnecessary local handoff.
- **Confirmed local, browser disabled:** metadata identifies the owner's local executor, but its supported browser tool reports an extension/setup problem. Output: owner-local, browser blocked; inspect the reported local prerequisite and request the needed correction. Do not relabel it cloud or infer whole-desktop access.
- **Unknown target access:** the requested UI target cannot be identified from available metadata/tools. Output: dependent UI operation not-run; resolve the target-specific gap through supported metadata or the smallest relevant owner question. A product icon is only supporting evidence, not a universal prerequisite. Unrelated work with known resources, tools and authority may continue.
**Minimal return record:** requested target; exact task/environment and evidence source; classification and uncertainty; capability/connection/authentication/authorization findings; action actually taken or not-run; receipt/check result; preserved work; blocker and next permitted step.
### 📦 Structured local handoff
Use a structured local handoff when local execution is the appropriate authorized route:
1. **Set up or continue the local task with the owner.** Use supported direct delegation to a connected local executor when it is actually available and permitted. If that route is unavailable and owner action is required, ask the owner to create/open the local thread and provide the handoff. Do not assume every handoff must be manual.
2. **Send the bounded assignment at the actual work boundary.** Transfer the outcome, applicable authority, exact artifact/version, working location, relevant source/checkpoint, preserved work, constraints and required verification/return. The receiver checks its actual baseline, capabilities and role. Use planning/review where needed to resolve a material unknown or change; do not demand a new PLAN-mode/approval ceremony for unchanged authorized work. A prepared assignment itself grants no extra permission.
3. **Inspect and amend.** Reconcile the proposed route with local reality and identify material changes requiring the owner's decision.
4. **Establish the approved bounded Goal.** Carry forward still-valid authorization; obtain approval for any changed scope or required action-specific gate.
5. **Switch to execution only after the applicable approval.** Execute locally within the concrete bounded Goal, preserving staged configuration, backups and checkpoints; stop at a genuine block or decision gate. Approval is not blanket permission for future work.
6. **Return evidence to the coordinator.** Identify the exact task/environment, actions, changed artifacts, checks, limits and next step. The coordinator integrates the result; it does not infer success from task completion alone. The owner reviews the result; necessary rework returns to inspection/amendment and the applicable approval boundary.
The steps are responsibility checks, not mandatory separate turns. A receiver carrying an unchanged approved Goal can satisfy the authority checks by readback and continue within that scope; a new material decision or action-specific gate still requires the applicable approval. This workflow is guidance, not a request to create a task or permission to operate a computer.
**Dated pilot evidence:** the source maintainer's 2026-10-16 evolution record reports the historical limits. Private source identifiers are retained outside this shareable candidate. That report does not establish universal platform incapability or amend a recipient project's authorization.

## ⚖️ Three questions that must remain separate

### Permission: may we do this?
The owner's actual instruction and approved Goal/amendments define the scope, subject to the executing system's applicable safety and confirmation requirements. A plan, installed tool, old preference or completed predecessor does not authorize new work.

### Observation: what happened?
Inspect the actual artifact, repository, database, process, output or durable worker return. Documentation can be stale. A recent edit is not proof that its statement is correct.

### Acceptance: does it satisfy the requirement?
Map the requirement to evidence that actually applies. Record the decision-maker, decision, scope and date. A green test, completed worker or saved page is not automatically owner acceptance.

~~~mermaid
flowchart LR
    I["Owner instruction / approved Goal"] --> P["May this action proceed?"]
    O["Live artifacts / outputs / records"] --> R["What exists or happened?"]
    E["Applicable evidence + review"] --> A["Which claim is accepted?"]
    P --> S["Reconciled project state"]
    R --> S
    A --> S
~~~

**Text equivalent:** permission, observation and acceptance are three independent inputs to the reconciled project state. None can replace the other two.

## ▶️ From an idea to approved work

### 1. Discover before committing
Recover the owner's intended outcome, audience, existing material and constraints. Inspect the real sources. Ask about uncertainty that would materially change the result; do not turn discovery into an unnecessary questionnaire.

### 2. Draft a proposed Plan
Explain the baseline, options, chosen strategy, dependencies, risks, expected evidence and proportionate documentation. Make important tradeoffs visible. The plan is reviewable, not automatically authorized.

### 3. Review and amend
The owner and relevant collaborators examine whether the plan solves the right problem. Record actual accepted, rejected, deferred and unresolved choices. Amend the plan rather than hiding disagreement in a polished final version.

### 4. Establish the approved Goal
Bind the outcome, included/excluded work, authority envelope, success criteria, approval gates and stopping point to the actual approval source. The Goal may incorporate the approved plan while preserving its own stable authority function.

### 5. Execute with an adaptive Plan
Ordinary implementation details may change inside the Goal. An important strategy change needs a dated rationale. A material scope or acceptance change needs the appropriate amendment and approval before dependent work.

### 6. Verify, integrate and accept
Produce evidence, check applicability, integrate worker returns and reconcile affected canonical records. Explain the result at the level the owner needs. Record technical and owner/product acceptance separately.

### 7. Stop, checkpoint or begin a separately authorized successor
A useful next idea belongs in the roadmap or a proposed next Goal. It is not authorized merely because the current Goal finished.

~~~mermaid
flowchart TD
    S["Start or recover: inspect sources and actual current authority"] --> Q{"What does current permission allow?"}
    Q -->|"Next step already authorized"| X["Reconcile or adapt Plan, then execute bounded step"]
    Q -->|"Discovery / planning only"| D["Discover intent and sources"]
    Q -->|"Paused or decision-gated"| W["Preserve checkpoint; wait for required decision"]
    D --> P["Proposed Plan"]
    P --> R["Review and amendments"]
    R --> G["Record actual approved Goal / amendment"]
    G --> X
    X --> V["Verify and integrate"]
    V --> A["Acceptance and required owner review"]
    A --> C["Checkpoint: continue authorized work, wait or close"]
    X --> M{"Material scope change discovered?"}
    M -->|"Yes: pause dependent action"| R
    M -->|"No, still in scope"| X
    C -->|"Authorized unfinished work"| S
    C -. "Separate authority" .-> N["Successor Goal"]
~~~

**Text equivalent:** inspect actual current authority first. Carry forward still-valid authorization to the next bounded step without reapproval merely because an assistant changed. Discovery-only work can produce a proposed plan for review and a later approved Goal. A pause or decision gate preserves a checkpoint and blocks dependent action. Verification and integration support scoped acceptance; remaining authorized work continues, required decisions wait, and a satisfied bounded Goal closes. Material changes return for amendment; successor work needs separate authority.

For a clear, low-risk task, the owner's request can already supply the bounded outcome and permission. A short Task Brief can hold the needed plan and checks. PBS does not require ceremonial approval of facts the owner has already clearly approved.

## 🗂️ Documents preserve different answers

- **Architecture:** identity, enduring structure, boundaries and relationships.
- **Methodology:** recurring ways of reasoning and operating.
- **Roadmap:** phases, intended progression, risks and exit conditions.
- **Goal:** authorized outcome and boundaries.
- **Plan:** current strategy and rationale.
- **Evidence:** inspectable observations and artifacts.
- **Acceptance:** requirement-to-evidence decision.
- **Current State:** accepted present at the appropriate project scale.
- **Continuation:** actual unfinished/unaccepted work and exact next permitted action.
- **Session Summary:** meaningful chronology, reasoning evolution, decisions, friction and learning.
- **Execution Summary:** factual work performed during a meaningful execution cycle.
- **Decision:** choice, authority, alternatives, rationale and revisit trigger.
- **Failure Intelligence:** consequential incident, evidence-supported diagnosis, resolution and prevention.

An update to a live opportunity does not necessarily change a project's Architecture. A new test result does not require copying its full log into every document. A valuable temporary discussion can belong in Session history without becoming permanent Methodology.

## 📍 Current and 🧷 Continuation

Current describes what the project accepts as its present state. Continuation describes what a returning worker must inspect and recover, including partial work.

Suppose Current says a feature is accepted at version 1. Continuation says a version 2 change is partially implemented. Those statements can both be correct. Copying the newer partial state into Current as accepted would erase an important boundary.

On re-entry, inspect reality. The feature may have progressed after the checkpoint. Preserve that progress, but do not assume that surviving files were verified or accepted.

~~~mermaid
flowchart TD
    C["Read compact checkpoint and authority"] --> L["Inspect live state and pending results"]
    L --> Q{"What survived?"}
    Q --> A["Accepted and still applicable: retain"]
    Q --> U["Complete but unreconciled: integrate"]
    Q --> V["Implemented, unverified: scoped check"]
    Q --> P["Partial: preserve and continue"]
    Q --> E["Uncertain / evidence unavailable: inspect gap"]
    A --> N["Exact next permitted action"]
    U --> N
    V --> N
    P --> N
    E --> N
~~~

**Text equivalent:** read authority and the checkpoint, inspect actual state, then classify the surviving work. Retain applicable accepted work; integrate completed work; verify unverified work; preserve partial work; investigate specific uncertainty. Continue only the actual missing permitted action.

## 🧾 Evidence is state-bound

Record what a check actually establishes and its limit.

Three independent dimensions are useful:
- **Outcome:** pass, fail, inconclusive or not-run.
- **Applicability:** current, historical, partially invalidated or unavailable.
- **Decision:** pending, accepted or rejected.

A historical pass can remain authentic while no longer applying to changed source. A failed overall report can contain useful scoped observations without becoming a passing report. A screenshot proves a captured view, not everything hidden behind it. A source test may not prove a separately packaged artifact.

Use identities proportional to the claim: document revision and observation date may suffice for a writing task; a benchmark may need source, harness, input, runtime, hardware and warm/cold conditions.

## 🤝 Parallel work needs an integrating owner

Parallel workers can save time, but their conclusions can conflict or remain trapped in temporary conversations. Before delegation, define source baseline, owned area, interfaces, expected artifact and evidence.

Before accepting the integrated result:
1. Inspect worker returns and actual changed artifacts.
2. Resolve contradictory assumptions and overlapping changes.
3. Check that evidence applies to the integrated state.
4. Update only the canonical owners whose facts changed.
5. Record acceptance and the next real boundary.

Do not claim a worker's inaccessible message is a durable recovery route. Do not require workers for tasks that one person can complete more clearly.

## 📚 Compact entry and deep knowledge

A routine software re-entry path might be:
agent entry → Current → active Goal → Continuation → live repository/artifacts → relevant Plan, Acceptance and technical sources.

A knowledge-work project may instead begin with:
project home → current operational database/view → approved task → checkpoint → relevant source documents.

These paths must follow the project's actual source map. Notion can own operational records; a repository can own executable source; a signed document can own a contract. Storage location alone does not settle authority.

The skill package supplies the complete essential method through this root and its bundled resources; no private Notion access is needed. Optional Notion research/examples can enrich the method and feed reviewed versioned updates, but are not automatically synchronized or effective instructions. A newer online draft cannot silently replace an adopted package or project contract. Missing project facts remain gaps, not something a self-contained method may invent.

The deep layer preserves Architecture, Methodology, reports, Session history and specialist explanations. It is opened when the question needs it.

**Optimize routine reading, not the amount of durable understanding the owner is allowed to retain.**

## 🛠️ Preserve execution without turning it into a black box

An execution report explains what was requested, what sources were used, what was actually done, what changed, what was checked and what remains.

A Session Summary explains the larger journey: the starting intention, important turns, rejected alternatives, discoveries, corrections, friction, outcome and where to resume.

These records may link to the same evidence. They have different purposes. Do not flatten them into one generic completion paragraph.

## ⚠️ Verification also needs a stopping rule

Authority controls what may be done. An assurance budget controls how much verification is justified.

Consider irreversibility, privacy, safety, external consequences, cost of a wrong claim, measurement uncertainty and recoverability. A local reversible note does not automatically need the test harness of a high-consequence migration.

Stop when the required claims have sufficient applicable evidence or a named decision boundary is reached. Report remaining limits. Do not recursively test the testing machinery without a concrete defect or consequential uncertainty.

## 🔄 Maintenance and evolution

Changes to PBS should come from real friction and evidence:
observe → extract a reusable lesson → propose a bounded change → try proportionally → review → adopt explicitly.

Keep the original episode and rejected alternatives. Do not silently make a one-project workaround a universal rule.

A new methodology version does not amend existing project permissions. An existing project should adopt changes deliberately through an owner map, controlled edits and verification.

## 🏁 What success looks like

A returning human or AI can answer:
- What is this project and why does it matter?
- What is authorized now?
- What actually exists?
- What is verified and accepted, and what is not?
- How does the resulting system work?
- Why did the important choices change?
- What is the exact next permitted action?
- What must not be repeated or reopened?

The result is useful autonomy with recoverable human understanding.

