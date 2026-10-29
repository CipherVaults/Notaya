# Setup for the friend: from invitation to first session

For the person Mohamed invited to work on Notaya. You need: a GitHub account, a Claude account that can use Claude Code, and (to build and test the Windows app) a Windows PC. About an hour, most of it installing tools. Read [collaboration-guide.md](collaboration-guide.md) afterwards; your Claude will read it too.

You are a **full partner** on Notaya (Mohamed's words, 2026-10-29 08:39:33 and 08:40:02 UTC: full work access, from the beginning to after V1.4.0). Your words direct your Claude like his do, and you can merge, release and accept like he can. You work on branches and draft pull requests, and you can't change GitHub repository settings (only Mohamed can, because he owns the account). Nothing either of you does should touch the other's real notes or PC.

## 1. Accept the invitation (GitHub)

1. Sign in to GitHub with the account Mohamed invited. Turning on two-factor authentication is a good idea (GitHub profile picture, Settings, Password and authentication).
2. Open the invitation from the email GitHub sent, or go to <https://github.com/AbyssWatcher2/Notaya/invitations> and press **Accept invitation**.
3. Check: <https://github.com/AbyssWatcher2/Notaya> opens (a private repository shows "404" until you accept, or if you are signed in with a different account).

## 2. Choose how your Claude works

| | A. Claude Code on your PC (recommended) | B. Claude Code in the browser (cloud) |
|---|---|---|
| Builds and runs the Windows app | Yes | No (no Windows; it can still run `tsc`, the build and the Rust tests under Wine, as `CLAUDE.md` describes) |
| Tests the real exe on throwaway data | Yes | Only through a session on your PC |
| Setup | Install tools (section 4) | Connect GitHub (below) |

**A.** Install Claude Code by following Anthropic's current instructions: <https://code.claude.com/docs/en/quickstart>. Sign in with your Claude account.

**B.** Go to <https://claude.ai/code>, sign in, and connect your GitHub account at <https://claude.ai/connect-github>. The Claude GitHub app must be allowed on the repository: if `AbyssWatcher2/Notaya` isn't offered when you pick a repository, tell Mohamed (the owner of the repository may have to install or approve the app at <https://github.com/apps/claude/installations/select_target>). Cloud sessions can only reach repositories you added to the session.

Either way, **everything the two Claudes need to share goes into the repository.** Your Claude account and Mohamed's are separate; neither sees the other's chats or memory.

## 3. Get the code

Install **Git for Windows** (<https://git-scm.com/download/win>) if you don't have it. Pick a folder that is **not** inside OneDrive, Dropbox or another sync folder (they fight with build folders), then:

```powershell
git clone https://github.com/AbyssWatcher2/Notaya.git
cd Notaya
git log --oneline -5
```

Git asks you to sign in through the browser the first time; use your own account. Never put a token or password in a file, a prompt or a commit.

## 4. What to install: checklist (path A, to build and test on Windows)

Versions are the ones the project was built and tested with (from `docs/STATUS.md`, `package.json`, `src-tauri/Cargo.toml`, `tools/windows-build/build-windows.ps1`). Stable releases only, no beta or preview. A newer patch version is normally fine; if something odd happens, match these. Open a **new** PowerShell after installing, then run the check in the last column; each should print a version, not an error.

| # | Program | Version used | Where | Check |
|---|---|---|---|---|
| 1 | Windows PowerShell | 5.1 (built into Windows 10 and 11) | already there | `$PSVersionTable.PSVersion` |
| 2 | Git for Windows | 2.55.0 | <https://git-scm.com/download/win> | `git --version` |
| 3 | GitHub CLI (`gh`) | 2.103.0 | <https://cli.github.com>, then `gh auth login` and sign in as **your own** account | `gh --version` and `gh auth status` |
| 4 | Node.js (with npm) | Node 24.20.0, npm 11.19.0 | <https://nodejs.org> (the 24 line, LTS installer) | `node --version`, `npm --version` |
| 5 | Visual Studio Build Tools 2022 | 2022, workload **Desktop development with C++** (MSVC and a Windows SDK) | <https://visualstudio.microsoft.com/downloads/> (Tools for Visual Studio, Build Tools) | `where cl` in "Developer PowerShell for VS 2022", or `rustc --version --verbose` showing host `x86_64-pc-windows-msvc` |
| 6 | Rust (rustup, stable, MSVC host) | 1.99.1, edition 2024 (needs a recent stable) | <https://rustup.rs>; no extra targets are needed for Windows builds | `rustc --version`, `cargo --version` |
| 7 | WebView2 runtime | current (preinstalled on Windows 11 and updated Windows 10) | <https://developer.microsoft.com/microsoft-edge/webview2/> | `Get-ItemProperty 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}' \| Select pv` |
| 8 | Claude Code | current | <https://code.claude.com/docs/en/quickstart>, sign in with your Claude account | `claude --version` |
| 9 | NSIS (makes the installer) | **nothing to install**: Tauri downloads it on the first `npm run tauri build` | automatic | the build prints the download; the setup appears in `src-tauri\target\release\bundle\nsis\` |

Not needed: Wine (that is the cloud container's way to run Rust tests), `cargo audit` (optional), and any Python.

**Project dependencies** come with the repository, not with a separate install: in the `Notaya` folder run `npm ci` (installs exactly `package-lock.json`, including the Tauri CLI 2.12.4; never run a plain `npm install` that changes versions: `.npmrc` pins exact versions). Rust crates are locked in `src-tauri/Cargo.lock` and downloaded by `cargo`.

**Smoke test, in this order** (each shows what it does not prove, see `CLAUDE.md` "Verification commands"):

```powershell
npm ci
npx tsc --noEmit -p tsconfig.json     # types
npm run build                          # frontend bundles
cd src-tauri; cargo test --lib; cd ..  # Rust unit tests (the first run compiles for a few minutes)
npm run tauri dev                      # the app opens with its own dev data in .devdata\ (ignored by Git, not your notes)
```

If these pass on `main`, your PC matches the build setup. Say so in your first message to Claude and in your first resume note, with the commit you ran them on.

Adding a library or tool **to the project** is your call as a partner, but it changes the lock files and the security review has to cover it, so write the reason in the PR. Making the repository public or adding a licence needs both of you to agree.

### Claude skills and plugins used on this project

| Item | What it is | How you get it |
|---|---|---|
| **Security Guidance** plugin (Anthropic) | Required for every code PR (guide section 9); its hooks check edits by pattern | In Claude Code type `/plugin`, find "Security Guidance", install and enable it. Check: `/plugin` lists it as enabled. |
| **`/security-review`** | Built into Claude Code; run it on the diff before a code PR goes to the other partner | Nothing to install. Check: type `/security-review` in a repo with a change. |
| **`CLAUDE.md` and `docs/handoff/`** | The project's rules, the PBS work cycle, claims and resume notes | Already in the repository; Claude Code reads `CLAUDE.md` by itself. This is what carries the rules for both of you. |
| Mohamed's skills in `.claude/skills/` (`cipherleaf-change-workflow`, `cipherleaf-real-exe-test`, `cipherleaf-project-coordination`, `pbs-start-resume-handoff`, plus three Notion/chart skills) | Exported from Mohamed's Claude account and committed to the repository (PR #42), unedited | Nothing to install: Claude Code loads them from the repo once they are on your branch (`git merge origin/main`). Tell Claude at session start to load and use them (see the skills step at the end of section 7). `CLAUDE.md` wins where they differ. |

## 5. Security helpers

Every code PR gets a security review (rule in section 9 of the guide). In Claude Code, install the Anthropic **Security Guidance** plugin (type `/plugin` to browse plugins) and use the built-in `/security-review` before you hand a code PR to Mohamed. If the plugin isn't available to you, say so in the PR.

## 6. Make a safe place for tests

1. Make a folder **outside the repository**, for example `D:\Notaya Tests`. All builds, test copies and throwaway notes go there, not into `%TEMP%`.
2. If your antivirus deletes unsigned test builds (Mohamed's did), exclude that folder in your antivirus yourself. Claude should never change an antivirus setting; do that yourself.
3. Your own real Notaya notes, if you use the app, are **never** used for tests. When testing an installer, untick "Run Notaya" on the last page.

## 7. Your first session

1. Open a terminal in the `Notaya` folder and start Claude Code (`claude`), or open the repository in the browser version.
2. **Set up the programs (paste this first, once).** Only Git for Windows and Claude Code itself are installed by hand (sections 2 and 3). Everything else Claude Code can install, one program at a time, with your OK each time:

   > You are Claude Code on my Windows PC. Set up the build tools for the Notaya project. Read section 4 of docs/handoff/friend-setup.md (clone the repository first if it isn't here yet). For each program in the checklist that is missing or has the wrong version (rows 3 to 7; Git and Claude Code are already installed, row 9 needs nothing): check it with the command in the Check column; if it is missing, find it with `winget search` and `winget show --versions`, tell me the package id, the exact version from the checklist and what it will change, and **ask me before each install**. Install that exact version with `winget install --id <id> --version <version> --exact` (for Visual Studio Build Tools 2022 add the workload "Desktop development with C++"; for Rust use rustup with the stable MSVC toolchain). If the exact version isn't available, ask me before using a newer stable one. Never install a beta or preview, never change antivirus or Windows security settings, and never use a token or password in a command. After each install open a new PowerShell and run its Check command; show me the output. Then run `npm ci` in the Notaya folder and the smoke test at the end of section 4, and report what passed and what didn't run. Do not start any other work.

3. Then paste the **Start** prompt from section 12 of [collaboration-guide.md](collaboration-guide.md), with your name and GitHub login filled in (you are a partner).
4. **Your first task is V1.4.0.** Tell Claude: "Read `docs/handoff/2026-10-28-v1-4-0-resume-note.md` on branch `claude/project-thread-fdn5gx` (draft PR #39) and tell me what is done, what is unverified and what comes next." That note (it exists only on that branch) is where Mohamed's Claude stopped, with the unfinished list in order. Claude reads the records and tells you what is in flight and who owns it. The main job is **V1.4.0** (branch `claude/project-thread-fdn5gx`, draft PR #39, a **shared branch**: both of your Claudes may push to it, following section 5b of the guide). Its resume note, on that branch, lists what is unfinished. Mohamed paused it for the night; either of you says "resume V1.4.0" to restart.
5. Tell Claude "Start <task>" (or "Resume V1.4.0, take <item>"). It claims the work (a branch `<your-login>/<topic>` with a first commit holding your claims row and a draft PR, or, on V1.4.0, a "taken by" mark in the resume note), does the work, writes down what it checked and what it didn't run, and keeps the PR up to date.
6. **Keep the resume note current.** Your Claude must update the resume note (V1.4.0: `docs/handoff/2026-10-28-v1-4-0-resume-note.md` on its branch; other work: its own `docs/handoff/YYYY-MM-DD-<topic>-resume-note.md`) and **push it every time it finishes a step, and whenever it stops for any reason** (you say stop, a limit, a closed laptop). Mohamed's Claude picks up from that note. At the end of every session also paste the **Hand over** prompt.
7. Mohamed's project skills live in `.claude/skills/` in the repository (uploaded as exported and not edited; they still say CipherLeaf and use Mohamed's PC paths, so use your own paths, and where they differ from `CLAUDE.md`, `CLAUDE.md` wins). They exist on `main` once PR #42 is merged; on a branch made before that, run `git merge origin/main` first. Claude Code loads them from the repo. At the start of every session, tell Claude: "Load the skills in `.claude/skills/` and use the ones that apply." Which to use:

- `cipherleaf-change-workflow`: for any code or docs change (plan, build, review, test on throwaway data, push, draft PR, report). Use it for every change.
- `pbs-start-resume-handoff`: at the start of a session, when resuming paused work, and when handing over (PBS records, resume notes).
- `cipherleaf-real-exe-test`: whenever the built Windows app must be tested, on throwaway data only, never real notes.
- `cipherleaf-project-coordination`: only if your Claude acts as the coordinator that routes work between threads.
- `notion-native-artwork`, `personalized-notion-hubs`, `personalized-charts`: not Notaya work; they are Mohamed's own Notion and chart skills (written for "Ahmed"). Ignore them unless you are asked for that kind of task.

   The V1.4.0 resume note (on branch `claude/project-thread-fdn5gx`) doesn't mention the skills yet; the first prompt above and this list cover it.

## 8. When something goes wrong

You are a full partner: fix it yourself, with your Claude. Mohamed is there if you want a second opinion, but you don't need to wait for him.

- **Claude wants to delete files, add a library, change anything about passwords or encryption:** it's your call as a partner. Decide it, and make sure the PR says why. Pushing to someone else's branch that isn't marked `shared branch` needs that person's OK.
- **Claude finds a vault, a `.nota` or `.clf` file or a key in `git status`:** do not commit it. Find out why it is there, move or ignore it, and carry on.
- **A push is refused, the repository isn't offered, or `git` asks for a password you don't have:** check you accepted the invitation (section 1) and are signed in as the invited account, then let Claude retry. If it still fails, tell Mohamed: only he can change GitHub repository settings.
- **A merge conflict or a red check:** have Claude merge `main` into your branch and fix it. Don't rebase or force-push.
- **Two people working on the same thing, or opposite instructions to the two Claudes:** look at the open PRs and the claims board, then settle it between the two of you before going on.
