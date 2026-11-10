# Notaya Whiteboard: study for the feature discussion

**Author:** Ahmed (AE0101), written by Claude (Sonnet 5.5, high effort) · **Machine:** cloud thread, no PC involved
**Date:** 2026-11-07, about 23:30 UTC (container clock) · **Phase:** 1, discussion only. Nothing is planned, staged or built. No version is assigned.
**Baseline checked:** `main` at `9e3e4e3` (PR #60 merge), accepted baseline V1.5.1.
**Ahmed's words (23:16:16 UTC):** a free-form canvas you move around freely and draw on; templates like Microsoft Whiteboard had; if templates cannot be found, a whiteboard with different backgrounds, colours, shapes, pens, fonts and sizes, fully customizable; a timer; all the critical core offline features of Whiteboard, "because it's now missing"; draw inspiration from OneNote for the ruler and the style of the features, such as the shapes (his screenshot of OneNote's Draw tab: pens, Add Pen, Panning Hand, Ruler with a 90° readout, Insert Space, Shapes, Automatic Shapes, Full Page View).

---

## 0. Ahmed's answers so far (added 2026-11-07, 23:23:44 UTC, message by Ahmed; recorded by Claude, cloud thread)

Quoted from Ahmed's message: the board is "a separate section of its own", "almost like the sticky notes are basically a section of its own"; "you draw with a mouse, you can draw with a pen, you can if you have a touch screen, but mainly if you have even like a drawing tablet like these people use for drawing anime characters ... I have one of these, and that's one of the useful features that I would have used, which have pressure sensitive pens"; "the 5 templates that you've just suggested is good, actually. They are good enough. We can expand later"; sticky notes on a board: "you can have both ... One type of sticky note that you can link ... the normal sticky note, or it can be separate ... I don't want to have to choose between either."

| Question | Answer recorded | What changes in this study |
|---|---|---|
| 1. Where boards live | **Their own separate section**, like Sticky notes. | Confirmed; no change. |
| 4. Pen, touch or mouse? | **All of them, and a pressure-sensitive drawing tablet matters a lot** (Ahmed owns one). | Pen pressure moves from "nice to have" to a **core, first-release requirement** (item 7 below): pressure changes line width (and optionally softness), tilt later, tablet eraser end, palm rejection with touch, tested on his tablet. Mouse and touch work too. |
| 5. Templates | Ahmed accepts "the 5 templates that you've just suggested" and says more can come later. **Which five is not stated in his message.** The coordinator relayed the list as: weekly planner, Kanban, mind map, SWOT, lined page. This study's own guess had been brainstorm, SWOT, task board, retrospective, meeting agenda. The two lists differ (SWOT and a task board / Kanban are in both). | **Use the coordinator's relayed list as the first five** (weekly planner, Kanban task board, mind map starter, SWOT, lined page) unless Ahmed says otherwise. A "lined page" is also one of the plain backgrounds, so it costs nothing extra. Brainstorm, retrospective and meeting agenda stay in the later set. |
| 8. Board sticky notes | **Both kinds, user picks per note:** a board note can be **linked** to a normal sticky note (same note; an edit shows in both places) or be a **separate copy** that lives only on the board. Ahmed does not want to have to choose one system. | Design rule: every board note has a "linked / separate" setting, changeable later ("link to a sticky", "make a separate copy"). Open design detail for planning: what happens to a linked note on the board when the real sticky is deleted (suggest: the board note keeps its text as a separate note and says so). |

**Still open:** questions 2, 3, 6, 7, 9, 10, 11, 12, 13 (section 9). Question 5's list should be confirmed by Ahmed in his own words.

---

## 1. What Microsoft Whiteboard offered (researched on the web)

I describe things in my own words and copy no Microsoft art, template files or wording of layouts. Sources are at the end.

**Status.** Wikipedia says Whiteboard was retired for personal Microsoft accounts on 16 October 2026 and still works for work and school accounts. I could only read that one page and I did not confirm it with Microsoft, so treat the date as "reported". It matches what Ahmed said.

**Template gallery.** Microsoft's own help page names nine categories, and its announcements name some templates. What I could confirm:

| Category (Microsoft's own list) | What it was for | Template names I could confirm |
|---|---|---|
| Brainstorming | Collect ideas as a group | Brainstorming |
| Problem solving | Break a problem down | Problem solving |
| Design and research | Understand customers, track feedback | not named on the pages I could read |
| Strategy | Compare options, set goals | SWOT analysis |
| Project planning | Task boards, daily stand-ups | Kanban sprint planning, Project planning, Project milestones |
| Games | Icebreakers such as "Two truths and a lie" | not named beyond that example |
| Retrospectives | Look back on what worked | Retrospective |
| Workshops | Guide a group session | Wishing method, Open Space |
| Learning | Capture lessons, compare ideas | KWL (Know, Wonder, Learn) |
| (meetings) | | Effective meeting |

Microsoft said it planned "30+ customizable templates", but the pages I could reach name about a dozen. **I could not find a downloadable copy of the templates or a complete list.** They were pre-made layouts (frames, columns, note grids, lists) dragged onto the board from a gallery. They were part of the online service, so nobody can lift them out, and copying Microsoft's designs would be wrong anyway. **So the plan is: our own original templates, using the same kinds of layouts** (section 4). Ahmed's fallback ("a whiteboard with different backgrounds") is included either way.

**Other features I found, with how sure I am:**

| Feature | Notes |
|---|---|
| Pens | Several pens with many colours and thicknesses (10 and later 15 extra colours were announced), a highlighter, an eraser, a selector. Effect pens (rainbow, "galaxy") were announced. |
| Ruler | Straight lines from a ruler or by holding Shift while drawing. |
| Shapes | Draw a rough shape and it straightens itself ("ink to shape"). Arrows, including double-ended. |
| Sticky notes | 10 colours; **note grids** (12 colours) for workshops. |
| Text boxes, images, stickers | Images from the local drive, stickers, copy and paste. |
| Backgrounds | A black board (people asked for it), 9 board colours, 8 kinds of grid lines. Background colour and pattern were announced as formatting. |
| Alignment and snapping | Announced. |
| Locking | Lock images and canvas content (announced for the desktop app). |
| Infinite canvas | You pan and zoom freely. |
| Reactions, live cursors, laser pointer, sharing | Online team features. **Not relevant to Notaya** (single person, offline). |
| **Timer** | **I could not confirm that Whiteboard itself had a timer.** I found no mention in any page I could read. Ahmed asked for one, so I treat it as his addition (a good fit for workshops). |

---

## 2. What Notaya already has (checked, so nothing is redesigned)

From the full inventory (`2026-11-07-notaya-full-feature-inventory.md`, items 20 and 21) and `package.json` / `docs/STATUS.md` on `main`:

- **No canvas, ink, pen or handwriting code at all.** The Whiteboard is a new item kind, like Sticky notes. The inventory expects a new section in the sidebar/Shell, a new vault item kind, and "likely a new dependency (ask first)".
- **Reusable today:** free-position pictures on pages (anywhere, front or back, picture checks by bytes, 25 MB, PNG/JPEG/GIF/WebP only; SVG refused); Mermaid diagrams (`mermaid` 12.2.0); the 8 sticky colours and sticky windows; the pop-out calculator; the text-style palette (18 colours); the accepted fonts (Inter, Segoe, Serif, Mono); the export engine (PDF, Word, Markdown, HTML); backups; the Trash.
- **Already agreed elsewhere in the discussion, which the whiteboard should follow:** the corkboard for sticky notes (grid and free modes, pins, strings), the app bar's quick buttons, page tabs, version history with an activity panel, right-click menus on everything, editable shortcuts, a global-shortcut idea for pop-ups, and the rule that every feature has named AI actions.
- **Link to OneNote mode (V1.8, undefined):** OneNote's free-canvas pages would build on the whiteboard canvas. Worth designing the canvas as a reusable engine so that later it can also become a "free page".
- **Already exist and should not be copied into the whiteboard:** Mermaid already draws flowcharts from text. The whiteboard is for hand-made, visual thinking, not for replacing it. Pasting a Mermaid diagram onto a board could be a later extra.

**Gap that matters:** the app keeps its own copy of the data and does not refresh from outside (inventory section 6.1). A whiteboard saves large drawings, so "save the whole board" would be the riskiest kind of save for AI changes. A whiteboard should save by small changes (add, move, delete one object) from day one.

---

## 3. Feature list: core and later

### CORE (the "critical offline features of Whiteboard" plus Ahmed's list)

**The canvas**
1. Infinite canvas: pan (hand tool, space-bar drag, middle mouse, two-finger), zoom (wheel, buttons, "fit all", 100%), a small overview map.
2. **Panning hand** and **Full page view** (hide the toolbar), as in OneNote.
3. **Backgrounds:** about a dozen colours including white, cream, soft grey, black and dark blue; patterns: plain, grid (small/large), dots, ruled lines, isometric, music staff, graph paper; adjustable pattern strength so it works on dark themes. Each board remembers its own.
4. Undo and redo for every action; autosave in the vault.

**Drawing**
5. **Pens:** at least pen, pencil, marker and **highlighter** (see-through), eraser (whole stroke, or like a rubber); size and colour per pen; **Add pen** (your own, saved as a favourite), a row of favourite pens like OneNote.
6. Colours: a palette plus a custom colour picker; "recent colours".
7. **Pen pressure (core, Ahmed 23:23 UTC):** pressure-sensitive drawing tablets (Ahmed owns one), Surface-type pens and touch screens; pressure changes line width, tablet eraser end, palm rejection while the pen is near, a "pressure curve" setting in Settings; a smooth line for mouse users. Needs to be tested on his tablet on the PC (Windows pen input), which a cloud thread cannot do.
8. **Ink to shape / Automatic shapes:** draw a rough circle, square, triangle, line or arrow and it tidies itself (turn on or off, like OneNote).
9. **Ruler** (OneNote style): lay it on the board, rotate it (angle shown in degrees, with the big 90° dial), draw straight lines along it, snaps at 0/45/90. Plus Shift-for-straight-line.
10. Select, move, resize, rotate, copy, paste, duplicate, group, bring front/back, lock, delete; box-select and **lasso** select; **Insert space** (push everything below or beside a line to make room, like OneNote).
11. Snapping and alignment guides, and align / distribute buttons.

**Shapes and lines**
12. Shapes: rectangle, rounded box, circle/ellipse, triangle, diamond, star, hexagon, speech bubble, cloud, cross, pentagon, plus lines, arrows (single, double, curved, elbow).
13. **Connectors:** arrows that stay attached to boxes and follow them when moved (what makes a flowchart usable).
14. Fill colour, outline colour, thickness, dashed/dotted, opacity, corner roundness, shadow on or off.
15. Text inside any shape.

**Text and objects**
16. **Text box** anywhere: font (the existing four plus a few more only if bundled offline), size, bold, italic, underline, colour, alignment, bullets; free resize.
17. **Sticky notes on the board (Ahmed 23:23 UTC: both kinds, per note):** the same 8 colours as the existing sticky notes; each board note is either **linked** to a normal sticky note (same note, edits show in both places) or **separate** (lives only on the board), and can be switched later ("link to a sticky", "make a separate copy").
18. **Note grid:** a table of notes for workshops (rows and columns that sort notes into place).
19. **Pictures:** paste, drop, add from file; resize, crop, rotate, lock (same checks and 25 MB limit as pages).
20. **Frames** (named areas, like slides): move the contents with the frame; "go to frame"; present frame by frame.
21. **Timer:** a visible countdown on the board (set minutes, start, pause, reset, a soft sound at zero, can be dragged anywhere; several allowed). Plus a **stopwatch**. (Ahmed's request.)
22. Links: a board object can link to a Notaya page; a page can embed or link to a board ("Link to page" already exists).

**Templates and sharing**
23. **Template gallery** (section 4) with categories, previews, "start from" and "drag onto an existing board"; **save your own board or selection as a template**.
24. **Export:** a picture (PNG), a PDF (whole board, each frame as a page), and a copy to the clipboard; later SVG. Print.
25. Boards live in the sidebar like pages (rename, icon, duplicate, move, trash), search by title and by text on the board, right-click menus, and appear in the command palette.

### LATER (valuable, not needed at the start)

- Effect pens (rainbow, glow), handwriting-to-text (offline recognition would need a big component; ask first), math ink to equation.
- Voting dots and reaction stamps for a workshop you run alone or on a shared screen; a **laser pointer** and presenter mode.
- Stickers and an icon library (must be our own or properly licensed).
- Import a PDF or slide as a background to draw on top of (annotate).
- Mermaid diagram placed on a board; Notaya database view or chart placed on a board (live).
- Layers and an outline panel; comments; version history of a board (ties into the agreed page history).
- Mind-map mode (tab key to add a child idea, automatic layout).
- OneNote mode: a free-canvas page that reuses this engine.
- Two people at once: out of scope (offline, single user).

---

## 4. Our own original templates

These are **our own layouts**, described from common thinking methods that nobody owns (a method like SWOT is a public idea, a layout is ours). They would be plain data inside Notaya: frames, text, notes, lines. No images copied from anywhere. Names are ours.

| Category (ours, close to Microsoft's so it feels familiar) | Templates |
|---|---|
| **Think and brainstorm** | Idea burst (a central question with a ring of notes), Mind map starter, Brain dump with sorting columns, Six hats (six coloured areas), "How might we" |
| **Decide** | Pros and cons, Decision matrix (options against criteria with scores), Priority square (urgent against important), Effort against impact, Two-by-two blank |
| **Strategy** | SWOT, Goals ladder, Business canvas on one page, Competitor map, Risks and answers |
| **Plan a project** | Task board (To do, Doing, Done), Weekly planner, Roadmap in three horizons (now, next, later), Milestones timeline, Daily stand-up |
| **Look back** | Went well / didn't / try next, Start / stop / continue, Four L's, Mood weather |
| **Workshop and meeting** | Meeting agenda with a timer, Parking lot, Wish-and-work-out, Silent brainstorm with note grid, Check-in wheel |
| **Learn and study** | Know / want to know / learned, Compare and contrast (two circles), Cause and effect (fishbone), Concept web, Cornell-style notes, Vocabulary cards |
| **Design and customers** | User journey (stages with feelings), Persona sheet, Empathy map, Sitemap, Simple wireframe frames (phone and desktop) |
| **Diagrams** | Flowchart kit (ready shapes and connectors), Org chart, Family tree, Timeline, Venn |
| **Personal** | Weekly review, Habit board, Trip plan, Vision board (pictures and quotes), Reading list |
| **Games and breaks** | Two truths and a lie, Icebreaker questions, Tic-tac-toe, Draw-a-day |
| **Blank backgrounds** | Plain, Grid, Dots, Ruled, Graph paper, Isometric, Music staff, Calendar month, Storyboard panels |

Rules for templates:
- **Customizable:** every part is an ordinary object you can move, recolour, retitle or delete; "save as template" for your own; "reset to original" for ours.
- A first set of about **12 to 15** polished ones is better than 60 thin ones; more can be added in later versions.
- Templates should use the current theme colours, so they look right in light and dark.
- **Sharing with Mohamed or between machines:** a template is a small file; whether we allow importing template files is a security question (section 6).

---

## 5. AI CLI actions for the whiteboard (proposal, names only)

Following the agreed rule that every feature has named AI actions, in the same closed-list style the inventory suggests (read, write, destructive tiers; delete only after confirmation; activity log marks "with Claude"; every AI change undoable).

| Area | Suggested actions | Tier |
|---|---|---|
| Boards | `board.list`, `board.get` (a summary, or the full object list), `board.create` (blank or from a template), `board.rename`, `board.set_background`, `board.duplicate`, `board.move`, `board.export` (png, pdf), `board.trash`, `board.restore` | trash = write (undoable); delete forever = destructive |
| Objects | `board.add_shape`, `board.add_text`, `board.add_note`, `board.add_note_grid`, `board.add_image`, `board.add_line` (incl. arrows and connectors), `board.add_frame`, `board.move`, `board.resize`, `board.style` (colour, font, size), `board.group`, `board.align`, `board.lock`, `board.delete_objects` | delete = destructive |
| Read the board | `board.read_text` (all text and notes in reading order), `board.find`, `board.describe` (layout in words), `board.snapshot` (a picture of it for the AI to look at) | read |
| Templates | `template.list`, `template.preview`, `template.apply` (to a board, at a place), `template.save_from_selection`, `template.delete` | delete = destructive |
| Pens and ruler | `pen.list`, `pen.save` (a favourite) | write |
| Timer | `timer.start`, `timer.pause`, `timer.stop`, `timer.list` | write |
| Link to the rest | `board.link_page`, `board.note_to_sticky`, `sticky_to_board` | write |

What an AI could do for Ahmed in practice: "make me a SWOT board for the Notaya launch and fill it from my notes", "turn this page's bullet list into a mind map", "read my brainstorming board and write a summary page", "tidy up my board (align and space things)". Freehand drawing by an AI is **not** a goal; it works with shapes, text and notes, which are precise and checkable.

**Needed first:** small-change saves (so an AI move never overwrites the person's drawing), and a "board changed" event to refresh the open window (inventory 6.1).

---

## 6. Security notes

Whiteboard follows the existing rules (`docs/security-design.md`) and the agreed AI safety approach. Points specific to it:

1. **Stored inside the vault, encrypted like every other item** (when security is on). Strokes are saved as points and settings, not as files on disk. No temporary plain copies of boards, including in the autosave and the Trash.
2. **Keys and secrets stay in Rust.** The canvas screen only receives the board it displays.
3. **No internet.** No online templates, no online images or fonts, no stickers fetched from a server. Everything shipped inside the app. (The strict content policy already forbids outside scripts, styles, images and connections.)
4. **Hostile content.** Anything imported (pictures, a template file, another app's board, pasted shapes) is treated as hostile: fixed list of object kinds, size and object-count limits (a board with millions of points must not freeze the app), pictures checked by bytes as today, **SVG still refused as input**, no scripts. SVG as an **export** is harmless but should be written by Notaya's own code only.
5. **Templates are data, never code.** Importing template files is a design choice; recommendation is to allow it only through the same hostile-input rules as page import, or not at all in the first release.
6. **Locking.** A locked vault closes board windows and wipes the canvas from memory, the same save-then-lock order as notes (A17). A board in a "locked note" stays locked. The timer's sound or notifications must not show board text while locked.
7. **AI.** Closed action list, no raw access to files, programs or the internet; delete only after confirmation; every AI edit logged and undoable through board history; the AI can **see** a board only when the vault is unlocked; the AI cannot change its own permissions. A snapshot picture given to the AI should be a deliberate action, because it can contain private content.
8. **Exports are not encrypted** (warned in the dialog today); the same warning applies to board PNG/PDF.
9. **Clipboard.** Copying a board picture leaves it in the clipboard; the existing clipboard auto-clear rule should apply.
10. **New dependency.** A drawing library or smooth-stroke library would be a new package: ask first, stable version only, exact version, licence checked and recorded in About, and a security look at it (the existing rule for every new package).

---

## 7. How it could be built (short, for planning later; not a decision)

- **A new item kind** "board" in the vault, next to pages and sticky notes. Objects stored as a list with an id, kind, position, size, style, layer and text. Strokes as simplified point lists. Boards open in their own view and also in a tab.
- **Rendering:** a canvas or SVG drawing surface in the UI. Smooth pen strokes usually use a small known stroke library instead of our own maths ("don't invent our own"); a whole whiteboard engine is another option but is large, and some are free to use while others have licence terms that matter for a project that might go public. This needs a separate small study before choosing (a tool-and-licence study, Sonnet high).
- **Size:** this is a big feature (like databases or dashboards). A sensible split inside the feature, only if Ahmed wants: (a) canvas with pens, eraser, undo, backgrounds, pan and zoom; (b) shapes, text, notes, pictures, ruler, select; (c) templates and the timer; (d) export and AI actions. It is **not** assigned to a version.
- **Performance:** thousands of strokes must stay smooth; lists of points need simplifying and the screen should draw only what is visible.
- **Windows pen and touch:** Surface pens and touch screens should work; this can be tested only on the PC.

---

## 8. Extras I suggest (additions nobody asked for)

- **Drag a board note onto the sticky corkboard** (and back) so the two features feel like one family.
- **Present mode:** frame by frame, with a laser pointer, for walking through a plan.
- **Timer templates:** "Agenda with timer", "Pomodoro board" (25 and 5 minute blocks).
- **Palette for dark themes:** pen colours that look the same on black and white boards.
- **Dyslexia-friendly and large-print option** for the board text.
- **Whiteboard in the app bar's quick-button list** (agreed earlier) and a **global shortcut** to open a quick scratch board.
- **A scratch board** that lives on the Home page (always there, never needs a name).

---

## 9. Open questions for Ahmed (numbered)

1. **Where do boards live?** As their own sidebar group next to Notes and Sticky notes (like I assumed), or also inside notebooks next to pages? (Your earlier answer was "own section like sticky notes".)
2. **Boards in tabs and notebooks:** should a board open as a tab like a page (yes by default) and be linkable from pages?
3. **Pens first:** is a small starter set (pen, pencil, marker, highlighter, eraser, plus "add pen") enough for the first release, with effect pens later?
4. **Do you use a pen or touch screen** (Surface pen, drawing tablet) or mostly the mouse? That changes how much we polish pressure and palm rejection.
5. **Templates:** are the 12 categories and about 12 to 15 starter templates right? Which five matter most to you first (my guess: brainstorm, SWOT, task board, retrospective, meeting agenda)?
6. **Template files:** do you want to be able to save, export and share your own templates (with Mohamed or between machines)? That brings a small security question; I recommend yes, later.
7. **Timer:** a visible board object (my plan), a pop-up window that stays on top like the calculator, or both? Sound on or off by default?
8. **Sticky notes on a board vs real sticky notes:** should they be the same notes (a note on a board is a real sticky note that also shows on the corkboard and desktop) or separate, simple board-only notes? Same notes is more powerful but more complex.
9. **Ruler:** do you want OneNote's exact ruler with the degree dial and the "draw along it" behaviour in the first release? (I put it in core.)
10. **AI and the whiteboard:** is "AI works with shapes, text and notes, never freehand drawing" what you want? And may the AI look at a picture of a board (it can contain private things)?
11. **Handwriting to text and effect pens:** later, or never?
12. **Size and timing:** this is a large feature. After all the discussion is done, do you want the whiteboard staged in steps (canvas and pens first), or one piece? (Phase 2 decision, only asking so you can think about it.)
13. **Connection to OneNote mode (V1.8):** should we design the canvas now so that OneNote-style free pages can reuse it later? (Recommended: yes, no extra work now.)

---

## Sources (web)

- Microsoft Support, "Insert templates in Whiteboard": https://support.microsoft.com/en-us/whiteboard/insert-templates-in-whiteboard (nine template categories, how templates are inserted)
- Microsoft Tech Community, "Microsoft Whiteboard Unleashes Creative Ideation" (template names): https://techcommunity.microsoft.com/t5/Office-365-Blog/Microsoft-Whiteboard-Unleashes-Creative-Ideation-and/ba-p/870604
- Microsoft Tech Community, "Microsoft Whiteboard announces preview of new templates" (Wishing method, Open Space): https://techcommunity.microsoft.com/t5/office-365-blog/microsoft-whiteboard-announces-preview-of-new-templates/bc-p/1595535
- Imperial College London IT blog, "Microsoft Whiteboard Enhancements, September and October 2021" (colours, note grids, ink to shape, announced features): https://blogs.imperial.ac.uk/office365/?p=41
- Windows Central, "Microsoft Whiteboard adds new pen and background colors, grid lines, and more": https://windowscentral.com/microsoft-whiteboard-adds-new-pen-and-background-colors-grid-lines-and-more
- Wikipedia, "Microsoft Whiteboard" (feature summary and the reported retirement date): https://en.wikipedia.org/wiki/Microsoft_Whiteboard

**Not run / not checked:** no PC session was used; OneNote's Draw tab is described from Ahmed's screenshot only; the app was not run; I could not open the Whiteboard app itself, and the full original template list could not be found.
