# 📋 Write Execution Reports

## Purpose, audience, and update trigger

An Execution Summary answers: “What did the execution layer actually do?” It makes meaningful execution inspectable through concrete inputs, changes, commands/tools, checks, problems, decisions, and a next step. Use the [full blank report](../../../assets/templates/execution-summary.md) and, when several reports need navigation, the [Execution Index](../../../assets/templates/execution-index.md).

The executor writes the report after meaningful work, at a handoff, or at a material stop. The coordinator or responsible reviewer checks and integrates it. Humans, future executors, and documentation maintainers should be able to understand the result without reopening the whole execution session.

Create a report when work materially changes artifacts or behavior, creates a deliverable, runs consequential analysis, restructures documentation, changes version-control state, or leaves a dependency others must understand. Do not require a large report for every trivial read or command. Add a dated correction when later evidence changes an earlier claim; do not silently convert an unverified historical result into a verified one.

The index changes when a report is added or its review, integration, or correction status changes. It owns discoverability, not current project authority. It does not replace the Goal, Current State, Evidence, or Acceptance owner.

## Preserve the execution/session distinction

An Execution Summary is a tool-level account. A [Session Summary](../../../assets/templates/session-summary.md) is deeper project memory: it preserves strategic evolution, friction, learning, decisions, and the whole session's re-entry context. Link them when both are warranted. Do not make a bounded executor invent a reflective discussion it never participated in. Do not use a short execution report to replace a substantial Session Summary.

A coordinator should integrate a worker's result against the active task, source owners, and applicable checks. “Worker returned success” is not equivalent to “integration passed,” and neither automatically means “owner accepted the product.”

## Select the right reporting depth

For substantial work, use the full twelve-section template. Keep its section names stable so readers can scan familiar locations. Use “None,” “Not applicable,” “Not checked,” or “Unknown” honestly rather than leaving ambiguous blanks.

For a genuinely small task, this exact compact form preserves the original proportional-reporting principle:

```markdown
# 📋 Execution Summary: [Date] — [Small bounded task]

## 1. Task Overview
[Request, scope, actual executor, and authority/task reference]

## 2. Work Performed
[Actual actions, affected artifacts, relevant tools, and save/readback state]

## 3. Current Result
[Observed result; checks and evidence; unresolved limits and review needs]

## 4. Recommended Next Step
[Concrete action or explicit stop; any prerequisite or approval; what not to repeat]
```

This compact report is suitable only when it accurately captures all material facts. A hidden uncertainty, external change, risky decision, recovery problem, or multi-worker integration may require expansion. No “small task” label excuses omitting a consequential failure or overstating verification. The full Session Summary structure is unaffected by this execution-only proportionality rule.

## How to complete the twelve sections

### 1. Task Overview

Describe the task as requested, its workspace, executor, mode, actual known date, and stopping condition. Link the authorizing request or Goal. Include important exclusions. A report records authority; it cannot create authority after the fact.

Use practical mode names. In a hybrid workflow, state whether execution informed later understanding, reasoning and execution iterated together, or another explicit blend was used. Do not assume all historical uses of “Hybrid” meant exactly the same thing.

### 2. Context Used

Distinguish documents supplied from documents actually read. Bind volatile inputs to their dates or revisions. Explain whether the executor checked current state before resuming, what conflicting context it found, and which assumptions remain unverified. Do not claim alignment merely because a familiar filename exists.

### 3. Work Performed

Record real actions in order. Identify created, modified, and deleted artifacts plus external state changes. Describe the relevant change without dumping an entire diff into the report. Separate preparation, attempted write, confirmed save, publication, and destination readback. A failed upload followed by a local candidate is not a published update.

If a deletion was authorized and relevant, record recovery status. Do not repeat a non-idempotent operation because its first result is uncertain; inspect live state first and follow the actual platform's confirmation rules.

### 4. Commands / Tools Used

Include commands needed to reproduce or explain material behavior, with relevant configuration and known versions. Remove secrets, tokens, credentials, unnecessary personal data, and machine-specific details that do not help reproduction. Name logical workspaces and project-relative paths where possible. A raw terminal transcript is usually an attachment or evidence artifact, not the report's main narrative.

### 5. Verification / Testing

For each material claim, identify what was checked, expected behavior, actual observation, result, evidence, and limitations. Bind results to the applicable artifact, revision, configuration, and input where those affect interpretation. Keep outcome (pass/fail/inconclusive/not-run), coverage or blocker, current applicability, and acceptance decision separate. A blocked check normally remains not-run or inconclusive with the actual reason recorded. Partial coverage is not a substitute outcome that conceals which checks passed or remain unperformed.

A successful write response may establish save success; a later readback checks the destination's content. Neither proves every behavior is correct. A passing technical test does not establish owner/product acceptance. Link [Evidence](../../../assets/templates/evidence-record.md) and [Acceptance](../../../assets/templates/acceptance-record.md) records when the task needs separate durable owners. If a relevant change invalidates a check, preserve the old result as historical and mark its present applicability correctly.

### 6. Problems Encountered

State errors, handling, and remaining problems. Include recoverable failures when they explain the final state. Avoid defensive prose and unsupported root causes. Put full incident learning in Failure Intelligence only when the consequence threshold is met.

### 7. Documentation Impact

Identify which owner actually changed and what remains to update. Do not refresh every project document because new information exists. A tiny code correction may require no Architecture, Roadmap, or Methodology update. Explain that briefly. For multi-document changes, follow [controlled maintenance](maintain-without-drift.md) and verify one owner at a time.

### 8. Git / Version Control Impact

Record observed local and remote state separately: a suggested commit is not a commit, a local commit is not a push, and a push is not proof of independent backup. A project can legitimately use no Git. Do not initialize, commit, push, or change repositories solely because the report has a field for it; authority and platform rules still apply.

### 9. Decisions Made During Execution

Record meaningful implementation choices, their rationale, and authority. Distinguish an ordinary within-scope choice from a provisional recommendation or consequential decision needing an owner. Link a [Decision Record](../../../assets/templates/decision-record.md) when alternatives or tradeoffs matter to future work.

### 10. Items That Need Human / Coordinator Review

Ask specific questions. Identify who should decide, the consequence, what must pause, and what independent work can continue. For worker returns, state whether the coordinator has actually reviewed and integrated the result. Do not hide this behind “done.”

### 11. Current Result

Summarize observed behavior and the actual deliverable state. Keep delivery, technical verification, integration, and owner acceptance distinct. Mention what does not work or has not been checked. A prepared candidate can be a successful deliverable if that was the task; it is still not a deployed or accepted product.

### 12. Recommended Next Step

Give a concrete first action or an explicit stop. Identify prerequisites and authority. Optional improvements remain proposals. If the stopping condition is reached, do not imply that the next Goal starts automatically. If unfinished work remains in scope, name the next recovery or verification step and the artifact to inspect before repeating an operation.

## Mode-specific additions

- Direct Task: say whether the result is a disposable experiment or something that should be preserved, and why. Recommend documentation or version control when warranted; do not create infrastructure by reflex.
- Direct Project: record the working project context and whether the owner wants to adopt a more durable structure. A growing task may need a new scoped decision.
- Coordinator-led execution: state whether work stayed inside the brief, what instruction was unclear, what returns for decision, and what integration is still pending.
- Hybrid: preserve the practical discoveries that changed understanding, options considered, and whether a bounded Plan revision or owner decision is needed. Link broader reasoning to Session memory instead of making the execution report carry everything.

## Maintain the index without duplicating reports

Create an index when several reports make it useful; one isolated report does not need another mandatory file. Add one compact entry per meaningful report, newest execution first. Keep report IDs and links stable. Summarize outcome, review/integration, acceptance when applicable, and the most important remaining dependency.

The report remains the source for its execution facts. The index can point to an updated correction or later acceptance, but must not silently change what the original execution accomplished. Do not convert the index into a running task database, a second Current State, or a transcript. Keep historical report links when a successor supersedes a result.

## Small worked example

Hypothetical teaching example only: a documentation owner asks an executor to repair a broken internal link in a draft help page. The executor changes that one link, opens the destination successfully in the test environment, and saves/reopens the draft. The page is not published.

A sufficient compact report says what link was requested, which draft changed, that the destination and saved draft were checked, and that publication remains outside scope. It does not need a fabricated architecture decision, emotional reflection, or a repository setup. It must still say “saved draft, not published” in its result and identify who decides publication.

If that same check reveals broken navigation across several rooms, the report should surface the finding and ask for the expanded scope. The original one-link task does not silently authorize a whole-site restructure.

## Quality and safety check

- Could another executor understand and reproduce the material checks?
- Are “provided,” “read,” “attempted,” “saved,” “published,” “verified,” “integrated,” and “accepted” used precisely?
- Do the checks apply to the actual delivered artifact and configuration?
- Are unresolved problems and owner review items visible?
- Are secrets and unnecessary private details absent from shared reports?
- Does the next step respect the original stopping boundary?

## Lineage and deliberate changes

The full report preserves the twelve-section structure and proportional-reporting principle of Codex Execution Summary Template v1.1. The portable derivative replaces named people and tool-exclusive roles with actual executors and coordinators; adds explicit authority/stopping boundaries, save/readback states, evidence applicability, and integration/acceptance distinctions; and keeps version control conditional. The Execution Index is a companion navigation artifact drawn from later report-index practice, not a claim that the original template required an index for every task.
