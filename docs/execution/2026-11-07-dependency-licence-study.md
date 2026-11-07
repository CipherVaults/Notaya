# EX-2026-11-07-52: Dependency and licence study for the planned features

**Who and where (A38):** Ahmed's side (`AE0101`); Claude cloud thread "Study libraries and licences for new features" in Ahmed's Claude project "Notaya" (machine: cloud). Model Sonnet 5.5, effort high. Not measured: the session's own usage (the coordinator reads it later for the ledger).

Short form (4 sections). Normal size: a study, no app code. Times are UTC, from the cloud machine's system clock.

## 1. Task Overview

- **Baseline:** `origin/main` `67b27e4` (merge of PR #62); app version 1.5.1.
- **Authority:** Ahmed's message of 2026-11-07 23:39:31 UTC (project chat: plan and prepare in parallel, "don't wait for my authority" for the planning work), relayed by the coordinator at 23:49:18 UTC with the task from the version plan, section 9.2. Planning authority only: **no go to build and no approval of any dependency.**
- **Scope and write area:** the study `docs/plan/2026-11-07-dependency-licence-study.md` (copy of the shared-files study `notes/2026-11-08-dependency-licence-study.md`), this report, its index row, the claims row. Out of scope: any code, `Cargo.toml`, `package.json`, installing anything, `STATUS.md` (no accepted state changed).
- **Returns:** draft PR. **Stop point:** the study and these records.
- **Branch:** `claude/project-thread-skxkfi`, from `main` `67b27e4`.

## 2. Work Performed

1. Read the plan (sections 3, 5, 9, 11), the whiteboard study, `THIRD-PARTY-NOTICES.md`, `Cargo.toml`/`Cargo.lock`, `capabilities/default.json`, and the export code (`pdf.rs`, `pdf_unicode.rs`, `docx.rs`, `zip.rs`, `deflate.rs`, `tools/make-pdf-fonts.py`).
2. Compared candidates for five needs: whiteboard strokes and pen pressure, global shortcuts, PDF fonts for Arabic/Greek/emoji, an Excel writer, and the MCP bridge over a Windows named pipe. Sources: the npm and crates.io registries, the tldraw licence, Microsoft Learn (named-pipe security, WebView2 pen data), the `global-hotkey` source, the Liberation Fonts licence header.
3. Counted what each Rust candidate would add by running `cargo metadata` on scratch copies of `Cargo.toml` and `Cargo.lock` outside the repository (Windows MSVC target). The repository was not changed by the measurement.
4. Findings that change the plan: Arabic and Hebrew PDF fonts already exist (only Greek, Cyrillic and emoji are missing); tldraw is not usable (licence); a Windows named pipe is by default readable by Everyone, so the AI door needs an explicit access rule; the Excel writer can reuse the existing zip and compression code.

## 3. Verification and Evidence

- **Checks run:** the registry readings and crate counts above, on 2026-11-07 at about 23:55 UTC; the crate counts come from `cargo metadata`, so they show what appears in the dependency graph, not a compiled build.
- **Not run:** any build or test (no code changed); GitHub activity checks (GitHub unreachable from the thread); `cargo audit` or any advisory lookup; the real size of the new font piece (an estimate of 40 to 80 KB, not measured); anything on real Windows, Wine, or a drawing tablet; a read of rmcp's NOTICE files; Twemoji's licence (recalled, marked unverified in the study).
- **Result state:** produced; verified only as above; integrated on merge; **not accepted** (acceptance is Ahmed's word).

## 4. Outcome and Open Points

- Five questions for Ahmed are at the end of the study (tablet model and Windows Ink, emoji in PDF, Greek look, Excel charts, Excel import). Each new dependency still needs Ahmed's own words when its version is specced (study section 8).
- The 1.6 spec should take the named-pipe access rule into its security model (promises P30 to P33).
- Next permitted action (planning only): fold the five recommendations into the version specs; building and every dependency wait for Ahmed's word.
