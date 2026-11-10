# Dev kit handoff — incomplete

**State:** investigation only; the kit has not been created and must not be installed or registered. Baseline `64b548c`, lane K1, Gaming-PC; requested 2026-11-10 05:09 UTC. The intended location is `tools/dev-kit/`. The existing `tools/heartbeat/` is a dated reference snapshot; the live heartbeat folder is the requested read-only source. A finished kit would package those tools for a configurable installation; it does not replace the live installation until reviewed and deliberately installed.

## Findings for continuation

- The live heartbeat includes project-specific lane definitions **and** separate hard-coded execution-order and alive-count lists. It performs worktree setup, fetch, merge, checkpoint commits, pushes, probes, and checks. Converting only the lane definition table would leave private values and change or break execution.
- The live `-DryRun` is not a blanket no-write/no-launch guarantee: the per-lane loop reaches worktree and follow-up logic, and the script writes logs and state. Build a genuinely inert dry run before testing an empty configuration.
- The guard independently samples processes and can kill process trees and checkpoint/push work. Its matching, state synchronization, separate lock and stop-line calculations need review when made configurable.
- `build_dashboard.py` depends on the adjacent HTML template and `forecast-model.cjs`, several ledger CSVs, project-specific assumptions and names. Copying the Python file alone is not a usable or sanitized builder.
- The live stop-line data uses the historical property name `predicted12` even though the configured look-ahead is five minutes. Preserve compatibility unless the format is explicitly migrated.

## Remaining work

Build and sanitize all requested files from the live source, create one configuration and a lane table, and document install, usage, history and the absence of a licence. Audit every file and every read/write/process/Git path, including configuration-derived paths. Check behavior against the source, then run the requested parser checks, isolated empty-lane dry run, and private-string scan. Exercise dangerous branches only with inert fixtures and local throwaway repositories; an empty dry run cannot prove behavior parity or filesystem confinement. Do not register tasks, run a browser or the application, use the network, or touch the live source. The coordinator owns independent review and Git actions.

No executable kit, test results, behavior-equivalence claim, licence or acceptance is provided by this note.
