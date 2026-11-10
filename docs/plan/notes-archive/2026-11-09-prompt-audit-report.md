# Prompt audit: Claude Code configuration for the Notaya project

Run: 2026-11-09, cloud thread `/checkup`, Ahmed's side (AE0101), cloud container. Nothing was edited. Every change below is proposed.

## Step 0: assumptions

- **Scope.** Only configuration that loads into sessions in this project, as listed in the request. Settings files, `.mcp.json` and `~/.claude.json` were not read.
- **Target model.** The model running this audit, Claude Sonnet 5.5 (`claude-sonnet-5-5`). The coding-agent files are read by that agent, so the audit uses it as the target. `AGENTS.md` is also read by Codex; findings in it are judged on the repository facts, not on Codex's model.
- **Edit authority.** Edits under "stale fact" and "conflict" rows are proposals for you to confirm. Plugin- and account-supplied skills are report-only. Any edit under `~/.claude` would affect all projects and is marked so.
- **Group 4 (request config and architecture):** not applicable. No request-building code or prompt-assembly code is in scope, and no subagent definitions exist in scope. The Notaya AI feature code in `src-tauri/src/ai/` is outside this request.

## Step 1: inventory

Found and read in full:

| File | Notes |
|---|---|
| `CLAUDE.md` (97 lines) | Project root. No `@` imports. |
| `AGENTS.md` (65 lines) | Project root. Points at `CLAUDE.md` by link, not by import. |
| `.claude/skills/cipherleaf-change-workflow/SKILL.md` | Mohamed's exported skill with a dated note on top |
| `.claude/skills/cipherleaf-project-coordination/SKILL.md` | same |
| `.claude/skills/cipherleaf-real-exe-test/SKILL.md` | same |
| `.claude/skills/pbs-start-resume-handoff/SKILL.md` + `agents/openai.yaml` | PBS method skill |
| `.claude/skills/personalized-charts/SKILL.md` + `agents/openai.yaml` | |
| `.claude/skills/personalized-notion-hubs/SKILL.md` + `agents/openai.yaml` | |
| `.claude/skills/notion-native-artwork/SKILL.md` | |
| `~/.claude/skills/session-start-hook/SKILL.md` | Loads in every project |
| `~/.claude/skills/synced/…/` (22 account skills) | Plugin-supplied: `notion-native-artwork`, `pfs-operations`, `personalized-charts`, `personalized-notion-hubs`, `pbs-start-resume-handoff`. The first three of the project's four plugin skills are byte-identical to the project copies; the PBS `SKILL.md` differs as a whole file, probably line endings (not checked further). Anthropic-supplied: `docx`, `pdf`, `pptx`, `xlsx`, `skill-creator`, `docs`, `computer-use`, `chrome-browser`, `built-in-browser`, `deep-research`, `google-workspace`, `import-memory`, `learn`, `morning`, `doc-coauthoring`, `web-artifacts-builder`, `internal-comms`. Report-only. |
| `~/.claude/plugins/synced/…/` (165 skill, agent and command files from installed plugins) | Report-only. Signal-scanned, not read in full. |

Not present: `CLAUDE.local.md`, any ancestor-directory `CLAUDE.md` or `AGENTS.md`, `.claude/CLAUDE.md`, `.claude/AGENTS.md`, `~/.claude/CLAUDE.md`, `rules/`, `commands/`, `agents/` and `output-styles/` folders under `.claude/` or `~/.claude/`, and a managed-policy `CLAUDE.md`.

Read by signal scan only (not line by line): the PBS and notion-hub reference files under `.claude/skills/*/references/` and `assets/`, the Anthropic skills' reference files, and the 165 plugin files.

## Summary

The surface is mostly healthy. Pressure language is nearly absent (the project files have no shouting capitals), there are no thinking or output-ceiling scaffolds, and the three exported Mohamed skills already carry dated notes saying where they differ from `CLAUDE.md`, which is the right pattern. The findings that matter are facts the repository has outgrown, plus one contradiction inside a user-level skill:

1. **`CLAUDE.md` still has a rule about KeePassXC and a "generator logic" test claim.** KeePassXC was removed in V1.1.0 (A24), and `generator.rs` is now only a password-strength estimator.
2. **`AGENTS.md` tells Codex that Claude pauses at 70% of its weekly limit**, but A50 lifted that for 1.6. The briefing carries the dated correction; `AGENTS.md` does not. Flagged for you, not rewritten, because it loosens a limit.
3. **`~/.claude/skills/session-start-hook` contradicts itself:** its principles and wrap-up say "synchronous, no async in the first iteration", while its step-3 template writes the async marker. This affects all projects.

Counts per group: Group 1 (dated prompt text): 3 findings (F5 report-only, L4 and L5 low). Group 2 (configuration files): 10 findings (5 with proposed edits, 2 conflicts flagged, 3 low flags). Group 3 (tool descriptions): not applicable, no tool definitions in scope. Group 4: not applicable.

## Findings, highest confidence first

### F1. KeePassXC rule for a tool the project no longer has
- **Location:** `CLAUDE.md:25`
- **Evidence:** "- **Don't force-stop KeePassXC** (it protects its own process). Close it politely or answer its dialog."
- **Pattern:** Group 2, volatile specifics (a claim the repository contradicts).
- **Why obsolete:** A24 removed KeePassXC entirely (amendments record row A24; the Passwords page, the process start and `--lock` are gone). The only code mention left is a comment in `src-tauri/src/model.rs:364` about an old settings field. Both `cipherleaf-real-exe-test` (line 10) and `cipherleaf-change-workflow` (line 10) already say KeePassXC was removed in V1.1.0, so the rule disagrees with its own sibling files.
- **Confidence:** High.
- **Action:** remove (proposed, confirm).

### F2. "generator logic" names a module that no longer generates anything
- **Location:** `CLAUDE.md:37`
- **Evidence:** "| Rust unit tests | `cd src-tauri; cargo test --lib` | Crypto, vault, notes (pages, sub-pages) and generator logic | …"
- **Pattern:** Group 2, volatile specifics.
- **Why obsolete:** `src-tauri/src/generator.rs` opens with "Master-password strength estimate" and holds one function, `strength()`. The passphrase generator was removed in A22 (STATUS line 108).
- **Confidence:** High.
- **Action:** rewrite (proposed, confirm).

### F3. Session-start-hook skill contradicts itself on async mode (all projects)
- **Location:** `~/.claude/skills/session-start-hook/SKILL.md:79`, `:98`, `:181`
- **Evidence:** line 79 "Don't use async mode in the first iteration. Only switch to it if the user asks for it"; line 98, inside the step-3 template: `echo '{"async": true, "asyncTimeout": 300000}'`; line 181 "Hook execution mode: Syncronous".
- **Pattern:** Group 2, instruction text that contradicts itself on one point.
- **Why obsolete:** Following the template produces an async hook, and then the wrap-up reports it as synchronous. The principle and the wrap-up agree with each other, so the template line is the odd one out.
- **Confidence:** High (the file's own text).
- **Action:** remove the template line (proposed). **Affects every project.** The file is harness-supplied and may be overwritten on update.

### F4. "has not been installed" is no longer true
- **Location:** `.claude/skills/notion-native-artwork/SKILL.md:8` (same text in the account copy)
- **Evidence:** "The packaged skill has not had an independent end-to-end replay and has not been installed;"
- **Pattern:** Group 2, volatile specifics and history narrative.
- **Why obsolete:** The skill is installed: it sits in this repository's `.claude/skills/` and is synced to the account (plugin manifest, updated 2026-10-29). The sentence is a status line from the 2026-10-17 draft.
- **Confidence:** High.
- **Action:** rewrite (proposed). Upstream is a plugin, so mirror the edit there if you want the two copies to stay identical.

### F5. Plugin skill teaches dated chain-of-thought idioms (report only)
- **Location:** `~/.claude/plugins/synced/…/5e2cbecc…/skills/oracle-prompt-engineering/SKILL.md:70-72`
- **Evidence:** "Explicit CoT instruction: 'Think step by step before answering'", "Hidden CoT: Instruct model to reason in a `<thinking>` block".
- **Pattern:** Group 1b, scaffolds replaced by API features.
- **Why obsolete:** On current models adaptive thinking plus `effort` replaces both lines, and asking for reasoning to be reproduced in the answer can trigger a `reasoning_extraction` refusal on Sonnet 5.5.
- **Confidence:** High.
- **Action:** `replace-with-API-feature`, but this is a plugin file, so **no edit is proposed**. Reported only.

### F6. `cargo audit` claim in the verification table
- **Location:** `CLAUDE.md:40`
- **Evidence:** "Rust crates (`cargo audit` not yet run)"
- **Pattern:** Group 2, volatile specifics.
- **Why obsolete:** `docs/STATUS.md:140` records `cargo audit` as done on the V1.0.1 lock file (0 vulnerabilities), while `docs/acceptance.md` row V6a says "Not run" for the current lock. The records disagree with each other and the rules file picks one, so the sentence goes stale on every dependency change.
- **Confidence:** Medium.
- **Action:** rewrite to point at the owning record instead of stating a status (proposed, confirm).

### F7. `AGENTS.md` still states the lifted 70% Claude pause (flag)
- **Location:** `AGENTS.md:53`
- **Evidence:** "Claude pauses at 70% of its weekly limit."
- **Pattern:** Group 2, instruction against a newer record.
- **Why obsolete:** A50 (Ahmed, 2026-11-09 07:10:43 UTC; `docs/STATUS.md:78`) lifts the 70% line for 1.6, and `docs/handoff/codex-partner-briefing.md:84-85` carries the dated correction. `AGENTS.md` has none.
- **Confidence:** Medium. The newer record is clear, but the change loosens a limit.
- **Action:** `flag`. Decision for you: confirm A50 is still in force, then add a dated note beside the line, as the briefing does. No edit is proposed because the correction relaxes a limit and comes from a record, not from a file I can order by history (blame stops at a shallow boundary commit).

### F8. Ask-first list and cloud-thread rule disagree on pushing (flag)
- **Location:** `CLAUDE.md:21` against `CLAUDE.md:50`
- **Evidence:** line 21 "**Ask the owner first** before: … making Git commits, adding a remote or pushing"; line 50 "Push and open PRs freely, but ask before a force-push…".
- **Pattern:** Group 2, conflict. The cloud-threads section is narrower and probably an intended override, but it does not name the rule it overrides, and line 21's Ahmed exception covers only the coordinator.
- **Confidence:** Medium.
- **Action:** `flag`. Decision for you: a one-line cross-reference in line 21 ("on a cloud thread's own branch, see Cloud threads") would remove the doubt, but it loosens an ask-first rule, so it is left for you.

### Low-confidence flags (no edits proposed)

| # | Location | Evidence | Pattern | Note |
|---|---|---|---|---|
| L1 | `AGENTS.md:7` | "These replace the first version of this section … which Ahmed changed in his messages of …" | Group 2, history narrative | `CLAUDE.md` rule 8 ("add dated corrections instead of editing history") asks for exactly this provenance. Leave. |
| L2 | `.claude/skills/pbs-start-resume-handoff/SKILL.md:8,14,61` | "approved 26 October 2026. Promoted from reviewed candidate.4", "Environment-routing reference refresh · October 16, 2026", "retain v1.0 candidate lineage" | Group 2, history and time-sensitive text | Vendored PBS text. The skill itself says to read its origin note before editing; changing it diverges from upstream. |
| L3 | `.claude/skills/personalized-notion-hubs/SKILL.md:51,59` | "**not yet tested**"; "An owner report that the board opens outside Home is a failed presentation result." | Group 2, time-sensitive status and one-incident rule | Could not check against the validation notes in time. |
| L4 | `~/.claude/skills/session-start-hook/SKILL.md:156,162,166,61` | three `IMPORTANT:` lines on validation steps; "Make a todo list for all the tasks … one after another" | Group 1a and 1c | Three scattered markers on real requirements; plain wording would do. All projects. |
| L5 | `skill-creator/references/schemas.md:312` (account copy) | `"executor_model": "claude-sonnet-4-20250514"` | Group 1d, retired model pin | Inside an example schema, Anthropic-supplied. Report only. |

## Proposed diff

One hunk per finding. These are illustrative unified diffs, not applied.

**H1. `CLAUDE.md` (F1)**
```diff
 - **Quote times, don't estimate them.** Take a time from the session transcript … If neither is available, write "not recorded".
-- **Don't force-stop KeePassXC** (it protects its own process). Close it politely or answer its dialog.
```

**H2. `CLAUDE.md` (F2)**
```diff
-| Rust unit tests | `cd src-tauri; cargo test --lib` | Crypto, vault, notes (pages, sub-pages) and generator logic | App runtime, OS integration |
+| Rust unit tests | `cd src-tauri; cargo test --lib` | Crypto, vault, notes (pages, sub-pages) and password-strength logic | App runtime, OS integration |
```

**H3. `CLAUDE.md` (F6)**
```diff
-| Dependency audit | `npm audit` | Known advisories for the npm packages | Rust crates (`cargo audit` not yet run), code defects |
+| Dependency audit | `npm audit` | Known advisories for the npm packages | Rust crates (`cargo audit` covers them; its last result is in `docs/STATUS.md` and `docs/acceptance.md`), code defects |
```

**H4. `.claude/skills/notion-native-artwork/SKILL.md` (F4)**
```diff
-… Structural skill validation passed. The packaged skill has not had an independent end-to-end replay and has not been installed; the reference distinguishes observed results from capability-dependent options and states the verification limits.
+… Structural skill validation passed. The packaged skill has not had an independent end-to-end replay; the reference distinguishes observed results from capability-dependent options and states the verification limits.
```

**H5. `~/.claude/skills/session-start-hook/SKILL.md` (F3). Affects all projects.**
```diff
 #!/bin/bash
 set -euo pipefail

-echo '{"async": true, "asyncTimeout": 300000}'
-
 cd "${CLAUDE_PROJECT_DIR:-.}"
```

No hunk for F5, F7, F8 or L1 to L5 (report-only or flagged).

## Reviewed and left alone

- The long dated notes at the top of the three `cipherleaf-*` skills. The stale lines below them (old repository name, `E:\Project\cipherleaf`, "Never merge", the missing `portable.txt` step) are each named by the note and overridden by `CLAUDE.md`, so they are overrides, not conflicts. The same note repeated in three files is working redundancy.
- The Wine line at `CLAUDE.md:51` and its dated note at `:53`: the note says the general rule stays for Mohamed's side. Correct pattern.
- `CLAUDE.md` prohibitions (stable releases, never edit the approved plan, never commit vault data, never test on the live vault): each encodes a project or safety constraint with its reason beside it. Kept.
- `Call yourself Kevin.` in two skills: a persona choice, not substituting for context.
- `pptx` and `web-artifacts-builder` NEVER and VERY IMPORTANT lines: they name specific design defaults to avoid, which is the form that works. `skill-creator` "ALWAYS use this exact template" pins a fixed output format. Kept.
- Every project path named in `CLAUDE.md`, `AGENTS.md` and the three `cipherleaf-*` skills resolves (checked with file reads), and `npm run mock`, `npm run tauri` and `npm run build` are defined in `package.json`.
- `AGENTS.md:46` ("cargo clippy (MSVC) has not been run on the V1.5.1 final commit"): still true as worded. Clippy has since run on the P1 lane (166 warnings, `docs/execution/2026-11-09-wp16-stage1-numbers.md`), so it will date soon.

## Not covered

- The PBS, notion-hub and Anthropic-skill reference files and the 165 plugin files were checked by signal scan (emphasis capitals, thinking idioms, retired model names, numeric output caps, update suppressors) and not read line by line. One signal hit (F5); the rest were clean.
- Step 7 behavioural probes were not run: nothing was edited.
