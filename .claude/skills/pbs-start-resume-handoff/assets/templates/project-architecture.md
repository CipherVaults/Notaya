# 🏗️ Architecture: {{project name}}

This document owns durable project identity, structure, components, roles, interfaces, and boundaries. Update it only when that architecture changes. Current tasks, progress, staffing, and session chronology belong elsewhere.

**Architecture revision:** {{identifier}}
**Maintainer:** {{role or person}}
**Last structural review:** {{date}}
**Document status:** {{Proposed / Adopted / Superseded}}
**Adoption / decision reference:** {{actual record, or not yet adopted}}

## 🎯 Identity and purpose

{{Explain what this project is, who it serves, the enduring problem it addresses, and what makes it a coherent system rather than a loose collection of tasks.}}

**Primary outcome:** {{durable purpose, not the current task}}

**Secondary supporting outcomes:** {{supporting purposes, if any}}

**Non-goals:** {{explicit enduring exclusions}}

## 🧠 Enduring principles

{{Explain the few principles that actually shape structure and tradeoffs. State their practical consequences. Do not list generic values that influence nothing.}}

- **{{principle}}:** {{what it means for the design and what tradeoff it creates}}
- **{{principle}}:** {{meaning and consequence}}

## 🧩 System structure

{{Describe how the main parts fit together and why this decomposition exists. Distinguish what currently exists from an explicitly proposed change.}}

### {{component or layer name}}

**Responsibility:** {{one coherent job this component owns}}

**Inputs:** {{what enters, from whom or which component}}

**Outputs:** {{what it produces and who consumes it}}

**Boundary:** {{what it does not own and what it delegates}}

**Dependencies / interfaces:** {{durable relationships and relevant contracts}}

**Detailed owner:** {{reference to implementation or specialist explanation when needed}}

Repeat the component section only for meaningful components. Avoid mirroring every file or temporary work item.

## 🔄 Main flows and interfaces

{{Walk through one representative end-to-end flow in plain language. Explain handoffs, information changes, external interfaces, and where validation or human judgment belongs.}}

1. {{entry / input boundary}}
2. {{transformation or decision boundary}}
3. {{output / consumption boundary}}

**Failure or uncertainty boundaries:** {{where incomplete, stale, unavailable, or conflicting information must be surfaced}}

## 👥 Durable roles and decision boundaries

**Owner / accountable decision role:** {{responsibility for intent, consequential decisions, and acceptance}}

**Coordination role:** {{responsibility for direction, integration, and continuity, if applicable}}

**Execution / contributor role:** {{responsibility for bounded implementation and evidence}}

**Review / acceptance role:** {{who evaluates which kind of claim; distinguish technical checks from owner acceptance}}

Describe stable roles. Names, models, current staffing, and temporary onboarding belong in operational records unless a specific identity is architecturally necessary.

## 📚 Information architecture

{{Explain the durable categories of information, how they relate, and why distinct owners exist.}}

**Canonical source map:** {{single authoritative reference for question → source ownership}}

**Entry and navigation:** {{how a reader reaches the right owner}}

**Living state versus history:** {{structural separation; link records rather than copy their current contents}}

**External copies / exports:** {{their architectural role and how readers identify the canonical source; detailed sync procedure belongs in Methodology}}

## 🧱 Boundaries and invariants

**Inside the project:** {{enduring system boundary}}

**Outside the project:** {{external systems, adjacent projects, and excluded responsibilities}}

**Compatibility / preservation invariants:** {{behavior, data meaning, interfaces, or history that must remain intact}}

**Security / privacy / trust boundaries:** {{structural responsibilities and interfaces, without secrets or live access grants}}

**Architecturally significant constraints:** {{constraints that explain the design; label assumptions and link decisions}}

## ⚖️ Structural tradeoffs and open decisions

{{Explain important accepted tradeoffs and link their decision records. Keep unresolved proposals visibly separate from adopted architecture.}}

- **{{tradeoff or question}}:** {{consequence, status, and decision reference}}

Do not turn this section into the active task list. Move implementation sequencing to Plan or Roadmap and live progress to its designated owner.

## 🔄 Change rule

Update this document only when project identity, structure, components, durable roles, interfaces, information architecture, or boundaries materially change.

Before changing it, inspect the affected sources, identify the structural change, and obtain any required decision. Preserve useful existing structure and rationale. Record the actual revision and link the decision; do not spread a new current-state fact across this document merely because it is important elsewhere.

**Related owners:**
- Current position: {{reference}}
- Phase progression: {{Roadmap reference}}
- Recurring working practice: {{Methodology reference}}
- Active authorized work: {{Goal / Task Brief reference}}
- Structural decisions: {{decision-log reference}}
