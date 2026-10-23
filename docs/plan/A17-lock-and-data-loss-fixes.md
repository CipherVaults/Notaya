# A17: Lock and data-loss fixes

**Status:** applied and committed (`dc95fd3`) on the owner's instruction of 2026-10-23 (time not recorded; see [approval-and-amendments.md](approval-and-amendments.md)). The owner has not yet reviewed the result. This is bug fixing, not a new feature. It does change one security-relevant behavior: the order of the steps when the vault locks.

## What the owner asked for

> Yes, fix the lock and data-loss problems first (unsaved edits on lock, the failed commit, lock timing, KeePassXC lock and re-show, sticky delete confirm, calculator +/-), then rebuild and commit.

This answered Claude's code review of the same morning (the owner had asked to "re review the codes and apps"). The problems came from that review, not from a report by the owner.

## What changed

1. **Text typed just before Lock or close is no longer lost.** The editor saves a moment after you stop typing (about half a second for a page, 0.4 s for a sticky note, 0.35 s for settings). Before, locking wiped the session first, so a save still waiting was lost. Now, before locking, the app tells every open window "save now". Each window writes what it has and answers. The app waits for each window, at most 2 seconds, and then locks anyway. This covers the Lock button, Ctrl+L, the palette, idle lock, Win+L, sleep, minimize and closing the window. If the USB drive was pulled out, nothing waits.
2. **A safer order when locking.** Wait for the saves, then hold the session, hide KeePassXC, close sticky notes, tell the window to go back to the lock screen, and only then make the backup, wipe the session from memory and ask KeePassXC to lock. (Before, the slow backup ran first, so notes stayed visible for a while.)
3. **A failed save to the database is undone cleanly.** Before, one failed "commit" could leave the database half-open, and later saves looked fine but were not saved.
4. **KeePassXC is asked to lock even if Cipherleaf did not start it** (for example after a Cipherleaf crash and restart). While the vault is locked, Cipherleaf refuses to place or embed KeePassXC, and the Passwords panel no longer shows KeePassXC if you leave the view while it is starting.
5. **A page that failed to save is kept as unsaved** and tried again on the next edit, when you leave the page, or at the next lock (before, it was dropped). Saves run one after another. Closing a sticky note stays open and shows "Couldn't save this note. Try again." if its save fails.
6. **Deleting a sticky note from its own window needs a second click.** The button turns into a red "Delete?" for 3 seconds.
7. **Calculator +/- key** now negates the whole expression: `-2+3` becomes `-(-2+3)` = -1. (It used to give 5.)

## Things to know

- If a save truly hangs (for example a drive that stopped answering), the lock goes ahead after 2 seconds and that last edit is lost. There is no automatic timed retry.
- After sleep or a screen lock, locking can wait up to 2 seconds for the windows to answer (a window that is reloading does not answer).
- Checked in the real exe on a throwaway vault: typing then clicking Lock at once keeps the text (a page, and a sticky note typed into while Lock was clicked in the main window), and closing the window right after typing keeps it. The previous build lost the text in both cases. Not tested in a real window: idle lock, Win+L, sleep, drive removal, the KeePassXC lock and re-show paths, the sticky delete confirm and the calculator +/-. Details: [EX-13](../execution/2026-10-23-lock-and-data-loss-fixes.md).
- Older portable ZIPs (including any copy already given to others or uploaded to a Drive) still have the problem.

## Not changed

The vault format, the encryption, the unlock delay, and every other feature. The review found more problems that were not fixed; they are listed under "Findings not fixed (backlog)" in [EX-13](../execution/2026-10-23-lock-and-data-loss-fixes.md).
