# Cipherleaf: failure intelligence

**Created:** 2026-10-22

**Last substantive update:** 2026-10-28 (FI-05 entered; FI-04 entered 2026-10-23; FI-01 to FI-03 entered on 2026-10-22, FI-01 moved here from STATUS)

**Dated correction (2026-11-07, records catch-up EX-2026-11-07-44):** the line above was not updated when FI-06 and FI-07 were entered on 2026-11-05 (V1.5.1 records); the last substantive update is therefore 2026-11-05. No new entry is added on 2026-11-07: nothing in the records catch-up met the admission boundary.

**Dated correction (2026-11-08, refresh thread, [EX-2026-11-08-61](execution/2026-11-08-ai-pbs-records-refresh.md)):** two entries were added on 2026-11-08, FI-08 and FI-09, on the coordinator's instruction (16:46 UTC); the last substantive update is therefore 2026-11-08. Ahmed's side, cloud and Gaming-PC.

**Maintainer:** Claude (Claude Code), for the owner

**Scope:** consequential failures, near-misses and system gaps in the work on this project.

**Ordering:** newest incident first. IDs and occurrence dates are stable; put newer dated follow-ups first within an incident.

**Related owners:** [sessions.md](sessions.md) for chronology; [`CLAUDE.md`](../CLAUDE.md) for adopted working rules; [STATUS.md](STATUS.md) for the live state.

## Purpose

To record what materially went wrong or nearly went wrong, why, the actual impact, how it was corrected, and the rule that should prevent it. It explains; it does not change policy by itself.

## Admission boundary

An entry belongs here if the problem reached, or could plausibly have reached, one of: the owner's data or vault; the security posture of the app or the PC; the integrity of the project's records; or work the owner would have had to redo. Ordinary friction (a tool limit, a failed first attempt that was fixed in the same step) stays in the session or execution records. For example, the refusal of `SetParent` on KeePassXC and KeePassXC's resistance to being force-stopped are tool limits, noted in [sessions.md](sessions.md) and [`CLAUDE.md`](../CLAUDE.md), not incidents.

## Incidents

### FI-09 — Codex sat idle for about 30 minutes because stopped lanes were never relaunched

**Occurrence / observation date:** 2026-11-08, about 08:07 to 08:36 UTC (heartbeat log, [EX-2026-11-08-55](execution/2026-11-08-v1-6-build-start.md) section 5). Noticed by Ahmed, not by the system. Ahmed's side (`AE0101`); machine: Gaming-PC (Codex, heartbeat), cloud (the Build thread).

**Last resolution update:** 2026-11-08 (permanent task registered 08:59 to 09:09 UTC; a related start-up cause found 12:04 UTC)

**Classification:** Near-miss on the schedule (work the owner would have had to wait for; no data or security effect)

**Status:** resolved; the prevention rule is applied in the heartbeat and written into the Codex briefing

#### Context

Two lanes had been restarted after lane A merged: the fix of C4b (block ids and block edits, whose tests had failed) and C2 (change signals and recycle bin). Codex runs on one shared account with a 5-hour window, and the owner watches its meter on his own usage screen.

#### Mistake / Gap

Both restarted lanes stopped after about 80 seconds with exit 0 and no work: each needed a tiny edit just outside the files it was allowed to touch, and stopped instead of making it. The heartbeat only relaunched lanes that had not started yet, so nothing restarted a lane that had started and stopped. The system did not notice that Codex was doing nothing; the owner did.

#### Impact

**Actual impact:** about 30 minutes of Codex time with no work (08:07 to 08:36 UTC), in a day paced by a weekly limit that holds about six 5-hour windows, so the loss shifts the schedule by that amount; the effect on the version dates was not measured. No code, vault or data effect.

**Potential impact:** a longer silent stall (overnight, with the owner asleep) would have wasted whole windows without anyone knowing.

**How it was noticed:** Ahmed, 2026-11-08 08:30:44 UTC (project chat): "Also make sure that Codex is not idle or anything is just idling at the development, wasting time, because I feel, and from what I look, and the past 40 minutes that I've been awake and looking at the usage and consumption of Codex, it's been like sitting idle. The hourly quota is just 38% left, and the weekly quota is just 74% left without getting decreased, and that means it's not doing anything."

#### Resolution

At 08:33 UTC the Build thread confirmed it and fixed it: a stop at the edge of a write area now counts as idle and the lane is relaunched; Codex makes small mechanical edits outside its area itself and lists them in the hand-back note; the heartbeat interval was cut from 30 to 10 minutes and at 08:35 four lanes were started again. Ahmed then asked for the heartbeat to be permanent and automatic (08:57:27 and 09:03:55 UTC, [authority record](plan/approval-and-amendments.md)); he registered the scheduled task "Notaya heartbeat" (every 10 minutes) himself at 08:59 to 09:09 UTC. **A second cause was found at 12:04 UTC:** lanes started by the scheduled task could not find the Codex install, which was visible only to the packaged Claude app, so some lanes died at once; they now start from the real install path through WMI and a lane that dies immediately is relaunched. Not confirmed here: how many of the earlier stalls this second cause explains.

#### Prevention Rule

**When:** a Codex lane process ends or goes quiet.
**Do:** classify it by exit code and by its diff, never by the fact that it stopped; treat "exit 0 and no file changes" as idle and relaunch it; make small mechanical edits just outside a write area and list them; compare the Codex meter with the lanes the heartbeat believes are running.
**Adoption:** applied in the heartbeat script and in [`docs/handoff/codex-partner-briefing.md`](handoff/codex-partner-briefing.md) section 6 and `AGENTS.md` (dated notes of 2026-11-08); no change to `CLAUDE.md` proposed (the owner decides on rule changes).

---

### FI-08 — The heartbeat reported lane C4 "finished" when it had done no work

**Occurrence / observation date:** 2026-11-08, corrected at about 06:56 UTC (heartbeat log, [EX-2026-11-08-55](execution/2026-11-08-v1-6-build-start.md) section 5); the time of the false report itself is not recorded. Ahmed's side (`AE0101`); machine: Gaming-PC (Codex, heartbeat), cloud (the Build thread, which caught it).

**Last resolution update:** 2026-11-08 (branch deletion approved 16:48:18 UTC; not yet done)

**Classification:** Near-miss on the records and the branches (a false "done" that was caught before it was merged)

**Status:** resolved; prevention rule applied in the heartbeat

#### Context

The heartbeat starts Codex lanes, makes checkpoint commits and pushes a finished lane. Lane C4 (block ids and block edits) had been started with a write area that did not include one file it needed.

#### Mistake / Gap

Codex stopped at a file outside its write area and ended with exit 0. The heartbeat read the ended process as a finished lane and pushed the branch `AE0101/v1-6-c4` (`4e3c457`) with no C4 work in it. Lane C2 had stopped the same way, for three files outside its area (that one was not reported as finished).

#### Impact

**Actual impact:** none on `main`. One stray branch with no work of its own exists on `origin` (deleting it needs Ahmed's word; see the dated follow-up below); some time was lost before the lane was restarted (not measured).

**Potential impact:** an empty lane merged or recorded as verified; a later lane built on work that did not exist. (The sources do not say how the false report was first noticed.)

#### Resolution

The Build thread's log records the correction at about 06:56 UTC: the branch is to be ignored and C4 restarts as `AE0101/v1-6-c4b`; the write areas of C4 and C2 were widened (C4 may add `mod blocks;` to `lib.rs`; C2 may touch `lib.rs` registration lines, `src/lib/types.ts` and others). The same finding led to the idle rule of FI-09.

**Dated follow-up (2026-11-08 about 16:55 UTC, refresh thread): the stray branch.** Checked on `origin`: `AE0101/v1-6-c4` is at `4e3c457`, one commit, a merge of lane B (`37886fd`) into lane C1's history (`85c8dcd`); both parents are already in `main` (PRs #74 and #75), so the branch holds no work of its own. The coordinator asked Ahmed (Q55) whether it could be deleted and he answered "yes" (2026-11-08 16:48:18 UTC, cited by the coordinator; [authority record](plan/approval-and-amendments.md)). **A deletion from the cloud container was refused** (`git push origin --delete` returned HTTP 403 from the environment's proxy; the GitHub tools offered here have no delete-branch call), and nothing was worked around. **The branch still exists.** It can be deleted from Ahmed's PC session (a Remote Control session, which has the repository credentials) or by Ahmed on GitHub; when done, add a line here.

#### Prevention Rule

**When:** a lane process ends and the heartbeat decides what that means.
**Do:** call a lane finished only when its branch holds commits inside its write area and the checks have run on that exact commit; call "ended with no changes" idle, not finished; inspect the diff before recording a lane as done.
**Adoption:** applied in the heartbeat; the inspection step is the Build thread's standing practice (A44 item 2, A46); no change to `CLAUDE.md` proposed.

---

### FI-07 — A test cleanup command was written to close every "Notaya" process, including the owner's real one

**Occurrence / observation date:** 2026-11-05, between 06:34:52 and 06:37:33 UTC (the PC clock lines of the build log before and after it), V1.5.1 Hubs and direction round ([EX-2026-11-04-43](execution/2026-11-04-v1-5-1-fixes.md)). Reported by the test session in its own report.

**Last resolution update:** 2026-11-05

**Classification:** Near-miss (the owner's running app, stopped before it could happen)

**Status:** resolved; no rule change proposed beyond the one below

#### Context

A second throwaway copy of the app would not start because an earlier throwaway copy (the Hubs test) was still running, and the app allows one instance at a time. The test session wanted to close the old copy.

#### Mistake / Gap

The command it wrote selected processes by name (`Get-Process Notaya`) and asked each to close. The name also matches the owner's installed Notaya and any portable copy. It did not check the path first.

#### Impact

**Actual impact:** none. The tool's safety filter denied the command before it ran. The test session then listed the running processes with their exe paths (one, under `Tests Excluded\v151b\hubs\both`) and closed only that process, after checking the path, and later used a script that closes only processes whose path is under the test folder.

**Potential impact:** the owner's real window could have been closed while they were typing (unsaved text in a sticky note or page is saved within seconds, but not instantly).

#### Resolution

Reported in the round's report. All later closes used `closeapp.ps1`, which matches the path `E:\Project\Notaya\Notaya Tests Excluded\v151b\*`.

#### Prevention Rule

**When:** a test script stops or closes an app process.
**Do:** select it by process id or by exe path under the test folder (never by name alone), print the paths first, and stop only processes the session started.
**Adoption:** proposed here; the existing CLAUDE.md line "stop only processes the test started" already covers it (the owner decides on rule changes).

---

### FI-06 — "Mermaid verified" in the cloud was not true in the real app (freezePrototype)

**Occurrence / observation date:** found in the V1.5.1 PC round of 2026-11-04 (Part A, before the release); owner's decision 2026-11-04 23:20:04 UTC; fixed in `370c7de`; retested on the PC 2026-11-04 23:41:59 to 2026-11-05 00:00:46 UTC ([EX-2026-11-04-43](execution/2026-11-04-v1-5-1-fixes.md) sections 8b and 8c).

**Last resolution update:** 2026-11-05

**Classification:** Near-miss (a cloud "verified" claim that failed in the real app; caught before release)

**Status:** resolved; the security setting was changed by the owner's choice

#### Context

The Mermaid diagram block was built and checked in the cloud and in the browser mock, where the diagrams drew. The real app sets `freezePrototype: true` in `tauri.conf.json`, which freezes `Object.prototype`; the browser mock never applies Tauri settings.

#### Mistake / Gap

The cloud record said the Mermaid item was verified, and a hostile-code check and a "real CSP does not block it" check were listed as to run, but nothing ran in the real window until the PC round. Mermaid assigns to inherited properties, which a frozen `Object.prototype` forbids ("Cannot assign to read only property 'valueOf'"): none of the nine sample diagrams drew.

#### Impact

**Actual impact:** none for the owner: the build was not released, and the defect was found before the merge. It cost one PC round of retests and a security-behavior decision.

**Potential impact:** a release with a visibly broken new feature, and a hidden trade-off: turning the freeze off removes one hardening step of the app.

#### Resolution

The PC session proved the cause (frozen prototype on, same code draws in plain Edge and fails there once the prototype is frozen) and asked the owner. He chose "Turn the freeze off, keep Mermaid" (23:20:04 UTC). `freezePrototype` is off in `370c7de`; the security design says so and what still protects the app (own scripts only, no network, Mermaid in strict mode). The retest showed all nine diagrams drawing and hostile diagram code doing nothing.

#### Prevention Rule

**When:** a feature depends on how the app runs (settings in `tauri.conf.json`, the CSP, window and process behaviour).
**Do:** do not write "verified" from the browser mock; mark such a check "NOT RUN in the real app" until it ran in the real exe on throwaway data, and put it first in the PC round.
**Adoption:** proposed here; not in `CLAUDE.md` (the owner decides on rule changes).

---

### FI-05 — A setup test started the app in the owner's real Windows session

**Occurrence / observation date:** 2026-10-28 15:54:06 UTC, found and reported by the test session at about 15:55 (V1.3.0 retest, [EX-2026-10-28-34](execution/2026-10-28-notaya-v1-3-0-release.md)).

**Last resolution update:** 2026-10-28

**Classification:** Near-miss (the owner's real profile, contained)

**Status:** resolved; the folder was deleted with the owner's word

#### Context

An interactive setup test drove the Finish page of the setup. The "Run Notaya" box is ticked by default and the setup starts the app through the Explorer session, which uses the real `%LOCALAPPDATA%`, not the throwaway one set for the test process.

#### Mistake / Gap

The test script passed both Finish-page options as false in the wrong form, so "Run Notaya" stayed ticked. The earlier plan had said to untick it; the script did not do it.

#### Impact

**Actual impact:** the app ran for a moment and was closed politely. It created `C:\Users\moham\AppData\Local\Notaya Notes\` with `data\backups` (empty) and `data\webview\EBWebView` (172 WebView2 files), and no `vault.clf`. The real `%APPDATA%` and the two old shortcuts were unchanged. No notes were read or changed.

**Potential impact:** had the owner's real notes folder existed, the test copy could have opened it (it was a throwaway copy of the same program) and made a backup or a new vault there.

#### Resolution

Reported at once; nothing deleted until the owner said "Delete it" (16:07:53 UTC). The PC session then re-checked the folder (172 WebView2 cache files, no vault or backup) and deleted only that folder (16:09:31 UTC). The rest of the run kept both Finish boxes unticked.

#### Prevention Rule

**When:** a test drives the Finish page of a setup, or any step that can start the app.
**Do:** untick "Run Notaya" before pressing Finish, check the box state in the script's log before the click, and compare the real `%LOCALAPPDATA%` before and after.
**Adoption:** proposed here; not in `CLAUDE.md` (the owner decides on rule changes).

---

### FI-04 — Edits typed just before Lock or close were lost

**Occurrence / observation date:** from the first build that had the delayed autosave until the fix in commit `dc95fd3` (portable builds up to the exe `F4F1E679…`, including the build and ZIP the owner gave to others). Found on 2026-10-23 by Claude's code review, not by a user report; reproduced the same day on the previous exe in the real-exe test.

**Last resolution update:** 2026-10-23

**Classification:** Failure (data loss risk, contained)

**Status:** contained; the owner action in Resolution is open

#### Context

The editor saves a moment after you stop typing (about 0.5 s for a page, 0.4 s for a sticky note, 0.35 s for settings). Locking the vault (Lock button, Ctrl+L, idle lock, Win+L, sleep, minimize) and closing the app went through the same path. See [EX-13](execution/2026-10-23-lock-and-data-loss-fixes.md) and [A17](plan/A17-lock-and-data-loss-fixes.md).

#### Mistake / Gap

Text typed in the last fraction of a second before a lock, an idle lock or a close was lost. The save was still waiting when the lock wiped the session, and the window was only reloaded.

#### Root Cause

The lock wiped the session before the windows had a chance to write their pending saves, and the UI only reloaded. Supported by the code review and by the real-exe test, where the previous exe lost the text in both a page and a sticky-note case.

**Why it was missed:** the earlier checks tested saving, not saving at the moment of locking, and the mock UI used for most checks has no real lock.

**Evidence / limits:** the code and the real-exe test on a throwaway vault. How many times it happened to anyone is unknown; nobody reported it.

#### Impact

**Actual impact:** none established. Only text typed within about half a second before a lock or close could be lost. No vault corruption.

**Potential impact:** a lost last sentence or number in a page or sticky note. This is the problem a notes app must not have, and it was in a build the owner shared with others.

#### Resolution

Fixed in `dc95fd3` (A17): before locking, every window writes its pending saves and the lock waits for them, at most 2 seconds. **Verification:** the same real-exe test passed on the new build (page, sticky note, closing the window). Not tested: idle lock, Win+L, sleep. **Outstanding action / owner:** replace the older ZIP copies (for example the one on the Drive, SHA-256 `1AD64164…D063`, exe `F4F1E679…`) with the rebuilt `E:\Project\Cipherleaf-portable.zip`. Not yet done as far as the records show.

#### Lesson

A save that waits for a timer can lose data if anything can end the session first. "Saving works" and "saving works at the moment of locking" are two checks.

#### Prevention Rule

**When:** a build changes how the app saves or locks.

**Do:** run a "type, then lock at once" test in the real exe on a throwaway vault (page, sticky note, close the window), and compare with the previous build.

**Check:** the typed text is there after unlocking. The script used (`lockflow.mjs` and `run-lockflow.ps1`) is in the session scratchpad, **not stored in the repository**, so it may need to be rewritten.

**Adoption:** proposed here; not yet in `CLAUDE.md` (the owner decides on rule changes).

**Durable owner:** this entry, until the owner adopts it.

#### Pattern / Confidence

**Pattern:** the checks covered the normal path but not the moment two events meet (typing and locking).

**Confidence:** high that it happened (reproduced); high that it is fixed for the Lock button and closing the window; not established for the other lock paths.

**Limits / revisit trigger:** a save that hangs for more than 2 seconds is still lost on purpose; revisit if the owner wants an automatic retry.

---

### FI-03 — Backslashes in a command string became control characters

**Occurrence / observation date:** 2026-10-22, during the A12 and equation work (about 17:30–17:55 UTC); the earliest sign was in the bash heredocs that failed during the redesign (about 17:00)

**Last resolution update:** 2026-10-22

**Classification:** System Gap

**Status:** contained; the control is adopted as a working rule

#### Context

Test data and edits written through a shell command string. See [EX-05](execution/2026-10-22-a12-notion-menus-subpages.md) and [sessions.md](sessions.md).

#### Mistake / Gap

A TeX example in the mock data (`\frac …`) arrived in the file as a form-feed character plus text, so the equation showed an error. Earlier, some multi-line shell heredocs failed outright. The cause was in how the command text was passed, not in the data.

#### Root Cause

Backslash sequences in the command string were interpreted before the shell or Node saw them (`\f` became a form feed). Supported by checking the bytes of the file (`od -c` showed the control character). Whether other sequences were affected was not tested.

**Evidence / limits:** the file bytes and the rendered error establish the effect; the exact layer that interpreted the text is not established.

#### Impact

**Actual impact:** one wrong line in a development-only mock file, found in a screenshot and fixed. No project data affected.

**Potential impact:** wrong text silently written into a source or record file.

#### Resolution

The line was rewritten from character codes (`String.fromCharCode(92)`). Files that need backslashes are now written with the file-writing tool and read back. **Verification:** the bytes were checked again after the fix. **Outstanding action / owner:** none.

#### Lesson

Text with backslashes isn't safe to pass through a command string.

#### Prevention Rule

**When:** a file's content includes backslashes (TeX, Windows paths, regular expressions).

**Do:** write it with the file-writing tool, or build the characters from codes; avoid heredocs and string-replace scripts for that content.

**Check:** read the file back, or inspect its bytes, for unexpected control characters.

**Adoption:** adopted 2026-10-22 22:03 UTC by the owner's approval of the CLAUDE.md additions (the last sentence of its "Records" section).

**Durable owner:** [`CLAUDE.md`](../CLAUDE.md).

#### Pattern / Confidence

**Pattern:** text altered on the way to a file; see FI-02.

**Confidence:** high that it happened; medium on the exact layer.

**Limits / revisit trigger:** a different shell or tool behaving differently.

---

### FI-02 — A replacement script inserted extra text into two records

**Occurrence / observation date:** 2026-10-22, in the equation step (about 17:50 UTC); found and fixed within the same step

**Last resolution update:** 2026-10-22

**Classification:** Failure (contained)

**Status:** resolved with verification

#### Context

Updating the A12 amendment record and its Execution Summary with the equation work. See [sessions.md](sessions.md).

#### Mistake / Gap

A script used JavaScript `String.replace` with a replacement text that contained `$x^2$`. In a replacement string, `$'` and similar sequences mean "insert the rest of the text", so the files received large pasted fragments. The Execution Summary grew to 294 lines (expected about 75).

#### Root Cause

An unescaped dollar sign in the replacement text, plus no readback before moving on.

**Evidence / limits:** the line counts and the visible duplicated text. A diff against the earlier version was not taken.

#### Impact

**Actual impact:** two documents were wrong for a few minutes; both were rewritten and read back. No code or data affected.

**Potential impact:** wrong history left in the records (the project's main recovery source).

#### Resolution

Both files were rewritten with the file-writing tool and re-read; line counts and content were checked. **Verification:** line counts back to normal; the later link check found no breakage. **Outstanding action / owner:** none.

#### Lesson

A file edited by script needs a readback every time, and replacement text with `$` needs a function replacement or the file tool.

#### Prevention Rule

**When:** a script edits a record or source file with text that contains `$`, backslashes or other pattern characters.

**Do:** use the file tool, or `replace(x, () => text)`; never put `$` text in a plain replacement string.

**Check:** read the result back and compare its length with what was expected.

**Adoption:** adopted 2026-10-22 22:03 UTC by the owner's approval of the CLAUDE.md additions.

**Durable owner:** [`CLAUDE.md`](../CLAUDE.md) ("Records").

#### Pattern / Confidence

**Pattern:** text altered on the way to a file; see FI-03.

**Confidence:** high.

**Limits / revisit trigger:** none.

---

### FI-01 — Debug port left open on the owner's live window

**Occurrence / observation date:** 2026-10-22, 15:58:50 to 16:06:58 UTC (about 8 minutes); disclosed to the owner in the same session

**Last resolution update:** 2026-10-22 (moved here from STATUS at 22:03 UTC)

**Classification:** Failure (contained; impact not established)

**Status:** contained; the owner action in the follow-up below is still open

#### Context

During the first launch of the dev app (see [EX-02](execution/2026-10-22-a9-keepassxc-d0-d1.md) and [sessions.md](sessions.md)), the app was restarted with WebView2's remote debugging port so the first screen could be read by script. The window was the owner's own, and they were using it.

#### Mistake / Gap

The dev app ran with the debug port open on `127.0.0.1:9222`, accepting any origin, while the owner typed in it and created a vault. Any local program could have driven the window or read the vault while it was unlocked. Observed events: Claude connected twice at about 15:59 UTC, read-only (one script read the first screen, one read the length of an input), and never afterwards.

#### Root Cause

The test method (drive the live window by script) was chosen without first making sure the owner wasn't about to use it, and the port was left open after the first reads. Contributing: the app was freshly launched and looked idle. Alternative considered: a throwaway vault in a separate process, which is what was used afterwards.

**Evidence / limits:** the owner's input visible in the form at the time and the file timestamps of the vault (about 16:05) establish the use of the window; whether another program connected is unknown and can't be established.

#### Impact

**Actual impact:** none established. No change was made to the owner's window by Claude.

**Potential impact:** another local program could have read the master password as it was typed, or the vault's contents, during those 8 minutes.

#### Resolution

At 16:06:41 the owner asked for a restart without the port; done by 16:06:58. After the restart nothing listened on 9222 and no WebView2 process had remote debugging. The exposure window was written into the records and corrected once (commit `0bc0309`). **Verification:** port and process checks at the restart; the same checks at the end of each later test run (observed). **Outstanding action / owner:** the owner should change the dev vault's master password (Settings → Master password & keyfile) or start a fresh vault if it will hold real data. Not yet answered.

#### Lesson

A debug port is access to the whole window. A running window the owner may use is not a test target.

#### Prevention Rule

**When:** UI automation or inspection is needed.

**Do:** use a throwaway vault in a separate process (`CIPHERLEAF_HOME` pointing at a temp folder), open the debug port only for that process, never on the owner's live window or vault, and close it afterwards. Don't leave `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` set.

**Check:** after the test, `netstat` shows nothing listening on 9222, and no WebView2 process has `remote-debugging` in its command line.

**Adoption:** adopted 2026-10-22 22:03 UTC by the owner's approval of the CLAUDE.md additions (the "Never test on the owner's live window or vault" rule).

**Durable owner:** [`CLAUDE.md`](../CLAUDE.md) ("Rules").

#### Pattern / Confidence

**Pattern:** acting on something the owner is using without checking first.

**Confidence:** high in the observed events and the correction; the question of whether anyone else connected is unanswerable.

**Limits / revisit trigger:** a different test method that needs the live window.

#### Dated Follow-up

- 2026-10-22 22:03 UTC — This entry replaced the "Debug-port exposure" paragraph in STATUS, which now points here. The original wording and times are preserved above; the window (15:58:50–16:06:58) is the corrected one, as recorded in commit `0bc0309`.
- 2026-10-22 — Later tests (A10–A12) followed the rule above: a throwaway vault, a closed port at the end, and the final port check at 21:49 UTC found nothing listening.
