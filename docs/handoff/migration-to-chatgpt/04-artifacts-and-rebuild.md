# The artifacts, and how to keep them alive without Claude

**Dated pause correction (2026-11-10, R3):** This file's earlier open-item, heartbeat and next-lane statements are historical. Read the [pause resume note](../2026-11-10-pause-resume-note.md) first: development, fixes, scripts and merges are paused on Ahmed's 06:13:35 UTC words, clarified 06:25:54 UTC. A55 still requires his handover acceptance. Scheduled-task removal is not verified.

**Who and where:** Ahmed's side (`AE0101`). Written by Claude, cloud, 2026-11-10 about 04:20 UTC (system clock). Source: the artifacts themselves, read from their claude.ai links on 2026-11-10 (versions named below), and `tools/usage-ledger/`. Result state: *produced*; the dashboard build command **was run once** in the cloud container on 2026-11-10 (`python3 -I tools/usage-ledger/build_dashboard.py /tmp/dash-test.html` read 37 sessions and wrote a 192 KB page, after this migration's ledger row was added); the forecast-CSV command and everything on the PC were **not run**.

**The core fact:** Claude artifacts are web pages hosted on claude.ai. Only a Claude session can *edit* them. They do not disappear when Claude stops, and the links stay readable by whoever has access, but they will go stale. So each one now has a **local home in the repository**, and from now on the **repository file is the source of truth**.

| Artifact | Link (read-only after the move) | What it is | Source of truth now | Snapshot taken |
|---|---|---|---|---|
| **Usage ledger dashboard** | https://claude.ai/artifact/SS1aMUgppEXH1gTYKMPjSY | Charts of what uses what (Claude and Codex), quota readings, the 1.6 to 2.5 forecast | The CSVs in `docs/ledger/` plus `tools/usage-ledger/build_dashboard.py` (template `dashboard-template.html`, model `forecast-model.cjs`). The page is **generated**, never hand-edited. | `docs/artifacts/snapshots-2026-11-10/usage-ledger-dashboard.html` (version `1794225756-0d66`) |
| **Roadmap** | https://claude.ai/artifact/LgHxQi61Fg9p619DU6yT6M | Version roadmap 1.6 to 2.5 with cards (points = rough size, about one developer-day each, **not quota**) | Hand-built single HTML file | `docs/artifacts/snapshots-2026-11-10/roadmap.html` (version `1794225924-f3d4`) |
| **Handover guide** | https://claude.ai/artifact/UhKn7cS38rp7Sq89YfGLyp | The guide for Mohamed, as a page | Two copies must match (A39): `docs/handoff/2026-11-07-handover-guide-for-mohamed.md` (Markdown, **wins**) and the HTML | `docs/artifacts/snapshots-2026-11-10/handover-guide.html` (version `1794226029-cb2b`, **content of 2026-11-08; the repo Markdown is newer**) |
| **FFmpeg lessons** | https://claude.ai/artifact/7gG8pDGVi3eEoXhbDJscg1 | Lessons from Ahmed's FFmpeg Encoder project (reference repo `AE0101/FFmpeg-Encoder`); a Claude *Docs*-type page, not Notaya-specific | Not stored in this repo (it belongs to the FFmpeg project). A private copy is placed in `D:\Claude's Projects\Notaya-private-notes\` | not in Git |

An unused FFmpeg forecast page also exists (`https://claude.ai/artifact/Fauju9JfwGE5LLYMs8niby`); it is not Notaya's and needs nothing.

## 1. Usage ledger dashboard (the one that must keep working)

**Why it matters:** Ahmed's rule (A37, 2026-11-07 20:12:10 UTC) is that the ledger is critical, to analyze where tokens go and to predict the pace. It is Notaya's own ledger, separate from the FFmpeg one (2026-11-07 20:19:06).

**Rebuild (from the repository root):**
```
python3 -I tools/usage-ledger/build_dashboard.py docs/ledger/dashboard.html
python3 -I tools/usage-ledger/build_dashboard.py --forecast-csv     # rewrites docs/ledger/pace-forecast.csv
```
Open `docs/ledger/dashboard.html` in a browser; it is one self-contained page (the forecast model is inlined). [Check: Python 3 on the Gaming-PC; if it is missing, Codex can run the script, or port it to Node, since `forecast-model.cjs` is already Node.] Commit the generated page only if Ahmed wants it kept in Git; the CSVs own the facts and the script only arranges them. Nothing is guessed: costs for rows without a reported cost are marked as estimates derived from rows that do report one.

**What feeds it (one row per thing, newest reading kept):** `claude-sessions.csv` (each Claude thread, from the session record's usage block or `tools/usage-ledger/scan_claude_transcript.py`), `codex-runs.csv` (each Codex run with before/after meters), `quota-readings.csv` (the heartbeat appends a row per tick; pushed to branch `AE0101/ledger-readings` about hourly), `project-usage-screen.csv`, `forecast-inputs.csv` (kind measured / rule / rough / unknown, each with a source), `forecast-versions.csv` (points per version). `docs/ledger/README.md` explains each column.

**After the move, add the ChatGPT side:** new rows of `claude-sessions.csv` stop (set `role` to the new ChatGPT coordinator rows in the same file if a separate sheet is not wanted; the columns say model and effort, so a ChatGPT row is fine with `model_configured` = the GPT model). Codex rows continue as before. Record ChatGPT's own meter readings by hand in `quota-readings.csv` when Ahmed reads them off ChatGPT's usage page; mark them `read by Ahmed`. The last Claude readings: weekly 91% and 5-hour 16% at 2026-11-10 03:21 UTC.

**If a hosted page is wanted without Claude:** open the generated HTML locally, or put it in a private location Ahmed controls. Do **not** publish it publicly (the repository is private; making anything public needs both partners).

## 2. Roadmap

A single static HTML page (fonts from Google Fonts; no other external requests; a little `localStorage` for the theme). The default is the light theme. Cards: version, work package id, points, build state. **Edit the HTML directly** (Codex can do this: search for the card title, change the text, keep the structure). Keep it in step with `docs/plan/2026-11-07-version-plan.md` and `docs/STATUS.md` (STATUS wins). Known gaps in the snapshot [Check]: card **1.6-19** (Notion import with real data, A52), the A53 and A54 changes and the heartbeat/Terra rules were to be added by the coordinator (memory TODO of 2026-11-09 10:42). The snapshot's date is 2026-11-09 evening; compare it with `STATUS.md` before trusting a card.

Better home, if Ahmed wants one: `docs/artifacts/roadmap.html` in the repo (the snapshot can simply be moved there), opened locally, with a one-line "Roadmap updated" item in each Session Summary.

## 3. Handover guide

The Markdown file wins. To refresh the HTML from it, ask Codex (Terra high is enough): "Render `docs/handoff/2026-11-07-handover-guide-for-mohamed.md` into the same layout as `docs/artifacts/snapshots-2026-11-10/handover-guide.html`, content identical." Add a line to the guide's change log each time. This migration added a short section 18 to the Markdown (see the file) but did **not** republish the claude.ai copy.

## 4. FFmpeg lessons

Read-only reference; nothing to maintain for Notaya. Its method (usage ledger design, model economics, PBS records comparison) already lives in `docs/plan/notes-archive/2026-11-07-notaya-vs-ffmpeg-pbs-records-report.md` and `2026-11-07-model-choice-guide.md`.

## 5. Notion pages (not artifacts, but kept the same way)

The hub and board update by hand or by ChatGPT's Notion connector [Check]; see `01-migration-guide.md` section 8. Keep a "Notion to update" line in each Session Summary so nothing is forgotten.

## 6. Suggestions (additions, nobody asked)

1. **One `docs/artifacts/` folder** with `roadmap.html`, `handover-guide.html` and (optionally) the generated `dashboard.html`, plus a tiny `README.md` saying "edit the source, then regenerate". (A 10-minute Terra lane.)
2. **A one-command script** `tools/usage-ledger/rebuild.ps1` that runs the dashboard build and the forecast CSV build and opens the page. The heartbeat could run it after each ledger push.
3. **Retire the claude.ai links** from the records only after Ahmed confirms the repo copies open correctly; until then keep both and say which one wins.
