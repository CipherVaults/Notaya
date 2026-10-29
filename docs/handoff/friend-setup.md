# Setup for the friend: from invitation to first session

For the person Mohamed invited to work on Notaya. You need: a GitHub account, a Claude account that can use Claude Code, and (to build and test the Windows app) a Windows PC. About an hour, most of it installing tools. Read [collaboration-guide.md](collaboration-guide.md) afterwards; your Claude will read it too.

You will work on **branches and draft pull requests**. Mohamed reviews and merges everything. You cannot change repository settings, and nothing you do should touch his notes.

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

Adding any new library or tool **to the project** needs Mohamed's OK first. Installing the tools above on your own PC doesn't.

## 5. Security helpers

Every code PR gets a security review (rule in section 9 of the guide). In Claude Code, install the Anthropic **Security Guidance** plugin (type `/plugin` to browse plugins) and use the built-in `/security-review` before you hand a code PR to Mohamed. If the plugin isn't available to you, say so in the PR.

## 6. Make a safe place for tests

1. Make a folder **outside the repository**, for example `D:\Notaya Tests`. All builds, test copies and throwaway notes go there, not into `%TEMP%`.
2. If your antivirus deletes unsigned test builds (Mohamed's did), exclude that folder in your antivirus yourself. Ask Mohamed before telling Claude to change any antivirus setting; Claude should never change one.
3. Your own real Notaya notes, if you use the app, are **never** used for tests. When testing an installer, untick "Run Notaya" on the last page.

## 7. Your first session

1. Open a terminal in the `Notaya` folder and start Claude Code (`claude`), or open the repository in the browser version.
2. Paste the **Start** prompt from section 12 of [collaboration-guide.md](collaboration-guide.md), with your name and GitHub login filled in and the role `collaborator`.
3. Claude reads the records and tells you what is in flight and who owns it. **Don't ask it to build anything yet.** Message Mohamed with what you would like to work on (or ask him what is free). V1.4.0 is being built on another branch and shouldn't be touched.
4. When Mohamed agrees on a task, tell Claude "Start <task>". It claims the work (a branch `<your-login>/<topic>`, a first commit with your claims row, a draft PR), does the work, writes down what it checked and what it didn't run, and opens the draft PR for Mohamed.
5. At the end of every session paste the **Hand over** prompt, so the next Claude (yours or Mohamed's) can continue.

## 8. When something goes wrong

- Claude wants to delete files, add a library, change anything about passwords or encryption, or push to someone else's branch: **say no and ask Mohamed.**
- Claude finds a vault, a `.nota` or `.clf` file or a key in `git status`: do not commit it. Stop and ask Mohamed.
- A push is refused, the repository isn't offered, or `git` asks for a password you don't have: the account isn't accepted yet, or isn't the invited one. Check section 1; then tell Mohamed.
- Two people appear to be working on the same thing: stop, look at the open PRs, and tell Mohamed.
