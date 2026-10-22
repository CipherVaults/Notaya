# A14: A small calculator inside each sticky note

**Status:** applied on the owner's instruction (2026-10-22 22:40:13 UTC; see [approval-and-amendments.md](approval-and-amendments.md)). The owner has not yet reviewed the result. **Reading confirmed by the owner at 22:55:26 UTC** ("Yes, a calculator inside each sticky note is what I wanted"); a separate floating window is not wanted. It adds to [A13](A13-calculator-and-wording.md); the calculator under the Sticky notes list stays.

## What the owner asked for

> Add small calculator in the sticky note itself when it opens or floating just small one i can click on it

## How it was read

A sticky note already is a small floating window on the desktop. So the small calculator lives **inside the note's window**, one click away, instead of being a separate window. (The owner confirmed this reading at 22:55:26 UTC; no separate calculator-only window is wanted.)

## What changed

- Every sticky note has a **calculator button** in its top bar. Click it and a small calculator takes the place of the note's text, in the note's own colors. Click the button again to go back to the note, or press **Esc** twice (the first clears the display, the second closes the calculator). The note's text is untouched while the calculator is open.
- The small calculator is basic arithmetic: digits, `+ − × ÷`, brackets, `%` (so `200 + 10%` is 220), `±`, delete, clear, `Ans` (the last result) and `=`. Typing works as well as tapping, and Enter calculates. A live preview of the answer shows under the display. Errors are shown in plain words.
- It uses the same calculation engine and the same rules as the bigger calculator (A13). That bigger one keeps the scientific mode and the history.
- **Privacy:** nothing is stored. Closing the calculator or locking the vault clears it. Sticky notes already close when the vault locks.
- To make room in the note's top bar, its **strikethrough** button was removed (the other formatting buttons stay).

## Not changed

Sticky-note storage and windows, the vault format, security behavior, the bigger calculator's features.

## Known limits

- Checked only in the mock UI at the sticky note's size; not tried in a real sticky window or with real clicks.
- In a very small note the keypad scrolls instead of shrinking.
- The small calculator has no history and no Copy button; use the bigger one for those.
