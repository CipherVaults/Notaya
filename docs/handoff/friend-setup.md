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

## 4. Tools for building the app (path A)

Versions the project was built with (stable releases only, no beta or preview):

- **Node 24** (24.20.0 and npm 11.19.0 were used) from <https://nodejs.org>.
- **Rust, stable, MSVC** (1.99.1 was used) from <https://rustup.rs>, and **Visual Studio Build Tools 2022** with the workload "Desktop development with C++".
- **WebView2 runtime** (already on Windows 11 and most updated Windows 10 PCs).

Then in the project folder: `npm ci` (this installs exactly what `package-lock.json` says; never run `npm install` with a changed version, the project pins exact versions). Checks and what they show are listed in `CLAUDE.md` ("Verification commands"). `npm run tauri dev` starts the app with its own development data in `.devdata\` (ignored by Git, **not** your real notes).

Adding a library or tool **to the project** is your call as a partner, but it changes the lock files and the security review has to cover it, so write the reason in the PR. Making the repository public or adding a licence needs both of you to agree.

## 5. Security helpers

Every code PR gets a security review (rule in section 9 of the guide). In Claude Code, install the Anthropic **Security Guidance** plugin (type `/plugin` to browse plugins) and use the built-in `/security-review` before you hand a code PR to Mohamed. If the plugin isn't available to you, say so in the PR.

## 6. Make a safe place for tests

1. Make a folder **outside the repository**, for example `D:\Notaya Tests`. All builds, test copies and throwaway notes go there, not into `%TEMP%`.
2. If your antivirus deletes unsigned test builds (Mohamed's did), exclude that folder in your antivirus yourself. Claude should never change an antivirus setting; do that yourself.
3. Your own real Notaya notes, if you use the app, are **never** used for tests. When testing an installer, untick "Run Notaya" on the last page.

## 7. Your first session

1. Open a terminal in the `Notaya` folder and start Claude Code (`claude`), or open the repository in the browser version.
2. Paste the **Start** prompt from section 12 of [collaboration-guide.md](collaboration-guide.md), with your name and GitHub login filled in (you are a partner).
3. **Your first task is V1.4.0.** Tell Claude: "Read `docs/handoff/2026-10-28-v1-4-0-resume-note.md` on branch `claude/project-thread-fdn5gx` (draft PR #39) and tell me what is done, what is unverified and what comes next." That note (it exists only on that branch) is where Mohamed's Claude stopped, with the unfinished list in order. Claude reads the records and tells you what is in flight and who owns it. The main job is **V1.4.0** (branch `claude/project-thread-fdn5gx`, draft PR #39, a **shared branch**: both of your Claudes may push to it, following section 5b of the guide). Its resume note, on that branch, lists what is unfinished. Mohamed paused it for the night; either of you says "resume V1.4.0" to restart.
4. Tell Claude "Start <task>" (or "Resume V1.4.0, take <item>"). It claims the work (a branch `<your-login>/<topic>` with a first commit holding your claims row and a draft PR, or, on V1.4.0, a "taken by" mark in the resume note), does the work, writes down what it checked and what it didn't run, and keeps the PR up to date.
5. At the end of every session paste the **Hand over** prompt, so the next Claude (yours or Mohamed's) can continue.
6. Mohamed's project skills (change workflow, real-exe test, project coordination, PBS start/resume/handoff, plus three Notion/chart skills) live in `.claude/skills/` in the repository, so Claude Code loads them from the repo on its own. They were uploaded as exported and not edited; they still say CipherLeaf and use Mohamed's PC paths, so use your own paths and where they differ from `CLAUDE.md`, `CLAUDE.md` wins.

## 8. When something goes wrong

- Claude wants to delete files, add a library, change anything about passwords or encryption, or push to a branch that isn't shared: **stop and decide it yourself as a partner**, and make sure the PR says why. Pushes to someone else's non-shared branch need that person's OK.
- Claude finds a vault, a `.nota` or `.clf` file or a key in `git status`: do not commit it. Stop and find out why it is there.
- A push is refused, the repository isn't offered, or `git` asks for a password you don't have: the account isn't accepted yet, or isn't the invited one. Check section 1; then tell Mohamed.
- Two people appear to be working on the same thing, or you and Mohamed give your Claudes opposite instructions: stop, look at the open PRs and the claims board, and talk to Mohamed before going on.
