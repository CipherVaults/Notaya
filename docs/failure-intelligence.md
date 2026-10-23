# Cipherleaf: failure intelligence

**Created:** 2026-10-22

**Last substantive update:** 2026-10-23 (FI-04 entered; FI-01 to FI-03 entered on 2026-10-22, FI-01 moved here from STATUS)

**Maintainer:** Claude (Claude Code), for the owner

**Scope:** consequential failures, near-misses and system gaps in the work on this project.

**Ordering:** newest incident first. IDs and occurrence dates are stable; put newer dated follow-ups first within an incident.

**Related owners:** [sessions.md](sessions.md) for chronology; [`CLAUDE.md`](../CLAUDE.md) for adopted working rules; [STATUS.md](STATUS.md) for the live state.

## Purpose

To record what materially went wrong or nearly went wrong, why, the actual impact, how it was corrected, and the rule that should prevent it. It explains; it does not change policy by itself.

## Admission boundary

An entry belongs here if the problem reached, or could plausibly have reached, one of: the owner's data or vault; the security posture of the app or the PC; the integrity of the project's records; or work the owner would have had to redo. Ordinary friction (a tool limit, a failed first attempt that was fixed in the same step) stays in the session or execution records. For example, the refusal of `SetParent` on KeePassXC and KeePassXC's resistance to being force-stopped are tool limits, noted in [sessions.md](sessions.md) and [`CLAUDE.md`](../CLAUDE.md), not incidents.

## Incidents

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
