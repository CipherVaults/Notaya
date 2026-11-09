# Whole-app test slices (A47)

Prepared on Ahmed's Gaming-PC for lane W phase 2. **Runtime not run by Codex.**
Claude runs these commands outside the sandbox, with existing Node 24, PowerShell 5.1,
Chrome/Playwright, locked npm dependencies, Rust/MSVC cache and WebView2. No installation,
network audit, application edit or Git write is performed by a runner.

From `D:\Notaya-lanes\W`:

```powershell
powershell -NoProfile -File .\tools\whole-app\run-group-a.ps1
powershell -NoProfile -File .\tools\whole-app\run-group-b.ps1 -Exe 'D:\Notaya-lanes\_target_release\release\Notaya.exe' -BuildCommit 5077183
powershell -NoProfile -File .\tools\whole-app\measure-performance.ps1 -Exe 'D:\Notaya-lanes\_target_release\release\Notaya.exe' -BuildCommit 5077183
```

`-BuildCommit` must name the commit Claude actually built. The report records this declaration
and the exe SHA-256 separately; it does not prove the declaration from the hash. A release exe
must be self-contained, as required by the older real-exe runners. B/performance refuse to
launch when any Notaya/Cipherleaf process already exists; close your app yourself first.

The commands write `group-a-results.json`, `group-b-results.json` and `perf-results.json`
under `D:\Notaya-lanes\out\whole-app`. Every run also has its own timestamp/GUID archive
there, plus preserved fake vaults in `D:\Notaya Tests\whole-app`. Never delete or reuse a
failed fixture to make a rerun look clean. Latest JSON is replaced only after archiving.
`in-progress.json` preserves work if a run is interrupted before final reporting.

Each child has isolated home/TEMP/TMP/LocalAppData. Only the installed/portable resolver
probe clears both home overrides, with a disposable LocalAppData. WebView debugging is
child-scoped, checked for loopback binding and owned listener PID before attachment, and
checked closed afterwards. A private Windows job owns each child tree and closes leaked
Chrome/WebView descendants. No broad process-name kill, real-profile access, startup entry
or security-setting change is used. Exit 1 means a failed/not-run required suite, fatal
prerequisite or cleanup failure; read JSON, including partial evidence, before retrying.

Group A runs all 25 executable dev-mock CJS/MJS/MTS suites sequentially, records the two
consumer-only helpers, discovers/runs the plan's Rust module areas, integrated AI and bridge
normal/native ignored suites, and runs local type/build/clippy checks. Each suite has a
timeout (default 240 seconds; Cargo/build default 900). The mock server and separate views
server use free loopback ports. The adapter uses exactly the requested installed Playwright
and Chrome paths, relocates hard-coded artifacts/temp/URLs, and records each original hash,
substitution and executed CJS source. It does not change assertions/selectors.

Group B runs six isolated scenarios: five-phase optional security, restore flow, separate
page/sticky/close lock flows, and installed/portable first/reopen probes. It runs the old JS
through an assertion/exit guard rather than old PowerShell wrappers that delete evidence
and can return success after printed failures. Disposable passwords are child environment
values, never files, argv or logs. Every successful scenario has actual quit/reopen/readback.
All 166 B IDs appear in JSON, including those not run. The plan's appended routing table
names every partial, later-scriptable, and currently non-unattended check. Historical
KeePass password tests are absent/obsolete; tray/startup-registry testing needs its own
clean-user/session slice. AI consent and native fault/legacy fixtures remain explicit gaps.

Performance defaults to blank plus 1,000-note/block/row fixtures, five warm reopen samples,
median/range, backend query/read/search timing, page open-to-editable/two-frame timing and
sampled whole-app process-tree memory. First-window timing is the first nonzero main window
handle, not first paint. Workloads stop on failure/timeout or sampled memory above 6 GiB.
The next run may increase sizes explicitly, after reviewing smaller fixtures:

```powershell
powershell -NoProfile -File .\tools\whole-app\measure-performance.ps1 -Exe 'D:\Notaya-lanes\_target_release\release\Notaya.exe' -BuildCommit 5077183 -NoteCounts '1000,10000' -BlockCounts '1000,10000' -RowCounts '1000,10000,50000'
```

Refresh the derived result sheet after reading the JSON:

```powershell
node tools/whole-app/merge-results.cjs 'D:\Notaya-lanes\out\whole-app\group-a-results.json' 'D:\Notaya-lanes\out\whole-app\group-b-results.json'
```

The sheet has all 250 IDs, KIND, feature, status, evidence/reason and defect group, summary
counts and embedded observation/provenance state. Import is additive and deduplicated by
JSON SHA-256. A complete pass is allowed only for an explicit full-variant observation with
stable source and successful cleanup. Only W126 currently has a complete A mapping; the
other legacy suite mappings are supporting/partial evidence, and none completes a B row.
This is deliberate: green partial suites cannot close A47. A failed supporting assertion
flags HARNESS-A/B for investigation, rather than declaring an application defect. Failure
evidence persists across reruns; coordinator review must resolve it in a later slice.
Performance JSON does not change feature statuses. Manual C reports can be imported later
with group/kind C and actual person observations. A merge/release is not owner acceptance.

Syntax validation here: PowerShell AST parser for all four PS1 files, `node --check` for
all seven JS files. Only document generation ran; no suite, browser, exe, build, Cargo,
audit or performance measurement ran in this scaffolding session. Head was `04fcb96` with
Claude's staged main `5077183` integration; no unresolved merge conflicts were present.
Claude owns commits/pushes, independent security review and records outside this write area.
