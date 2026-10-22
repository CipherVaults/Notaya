# Approval and amendments: Cipherleaf build plan

This record answers two questions: what the owner actually authorized, and which changes to the approved plan are proposed or accepted. The approved plan itself is kept unchanged in [`approved-plan-2026-10-22.md`](approved-plan-2026-10-22.md). Current progress lives in [`../STATUS.md`](../STATUS.md).

## The approved plan (serves as the Goal)

| Field | Value |
|---|---|
| Snapshot | `docs/plan/approved-plan-2026-10-22.md`, a byte-identical copy |
| Original | `C:\Users\moham\.claude\plans\shimmying-sauteeing-koala.md` (outside the project; left untouched) |
| SHA-256 of both | `010F2499AA9F88C75C0888ED6742608C67E9F0290E1CB4327AAC31FE3A6CBA3A` |
| Approved by | The owner, Mohamed Sayed, through Claude Code's plan approval |
| Approved at | 2026-10-22 11:08:21 UTC |
| Approved revision | The plan as last edited at 11:07:28 UTC, after the owner's "Install Them" amendment (11:07:18 UTC) added the toolchain-install step. The file hasn't changed since. |
| Success criteria | The snapshot's **Verification** section. Its **Honest limits** section lists the limitations accepted up front. |

There's no separate Goal file. The snapshot plus this record are the durable statement of what was authorized.

### What the plan is based on

The owner's request, 2026-10-22 11:01 UTC, verbatim:

> Okay so the app i want is as the following
>
> A Portable vault the most secured portable vault it will be a notes app will  be like OneNote and sticky notes and also has customization on it and password vault and generator with the most advances security and the UI/UX choose a one that will make people impress about it
> also don't use any experimental release or any tool still beta

The owner's answers to clarifying questions, the same day, before the plan was written:

- App type: desktop app. Coding experience: beginner.
- Unlock: master password, plus an optional keyfile.
- Sticky notes: float on the desktop and hide when the vault locks.
- Network: offline, with an opt-in breach check.
- Name: Cipherleaf.

## Authority since approval

Newest first. Later instructions narrow or pause the approved plan; they don't erase it.

| When (2026-10-22, UTC) | Owner instruction | Effect |
|---|---|---|
| 13:07:49 | "Approve amendment A9 and procced with develeopnet now conintiuot and download wahtever you want" | A9 approved. Development pause lifted. Downloads needed for the approved work are allowed (the KeePassXC 2.8.0 files and verification key, development tools). Read as also approving the first commit (file list shown earlier) and local checkpoint commits on branch `feat/a9-keepassxc`; no remote, no push. |
| 13:03:51 | "Yes: make A9 option D, bundling the official KeePassXC 2.8.0 portable app with Cipherleaf. Write it up for my approval, including what happens to Cipherleaf's own password section. Development stays paused, and no download yet." | A9 written up as a proposal ([A9-bundle-keepassxc.md](A9-bundle-keepassxc.md)). Not approved yet. Development stays paused; no download. |
| 12:50:52 | "can you involve the KeepassCX in it like the app implement it i want it to be like i can import/export kdbx and using all KeePassCX fetures" | Scope change requested; recorded as A9 |
| 12:32:12 | "Yes: keep the records in the repo's docs/ folder, use the compact set, and set up a local Git repository. Show me the file list before the first commit. Development stays paused." | Authorizes the PBS records and a local Git repository. The first commit waits for the owner's review of the file list. Development stays paused. |
| 12:23 | Shared a review of Claude's PBS understanding (recommending acceptance with five amendments) and said to continue | The amendments were applied to the Notion review page and shape these records |
| 11:53:21 | Asked for a read-only review of PBS before continuing | Review only; no changes |
| 11:51:27 | "can you pause for a secnod" | Development paused |
| 11:19:45 | "Continue" | Implementation resumed under the approved plan |
| Shortly before 11:19 | "can you pause to change the model" | Brief pause; the model changed from Sonnet 5.5 to Opus 5.5 |
| 11:08:21 | Plan approved | Implementation authorized within the plan |

## Proposed amendments (none accepted)

These are differences between the approved plan and the code as written. None is accepted yet. Each needs its impact checked against the approved requirements, with evidence, before it's recorded as accepted. A2 contradicts a stated security promise, so it needs the owner's decision.

| ID | Difference | Requirement affected | Evidence or decision needed | Status |
|---|---|---|---|---|
| A1 | Argon2 settings are fixed (256 MiB, 3 passes, 4 lanes), not tuned per PC | "Calibrated to about 1 s and never below 128 MiB" | Unlock time measured on a typical PC and a slow PC; memory confirmed at or above 128 MiB | Proposed; evidence pending |
| A2 | Records are bound to their vault, table and ID, but an older copy of a single record can be restored without detection | "Rows can't be swapped or replayed" | Owner decision: build rollback detection, or record it as a known limitation | Awaiting owner decision |
| A3 | The breach check uses Windows TLS (native-tls, SChannel) instead of rustls | Breach-check transport security and privacy | HTTPS-only behavior and certificate validation confirmed on Windows; reliance on Windows updates noted | Proposed; evidence pending |
| A4 | Randomness comes from the OS through `getrandom` instead of `rand` | Secure randomness for keys, nonces and generated passwords | Confirmation that it uses the Windows system generator; uniformity tests kept | Proposed; evidence pending |
| A5 | The EFF wordlist comes from the `eff-wordlist` crate instead of a bundled file | Passphrase strength (EFF large wordlist) | The crate's list compared with EFF's published file | Proposed; evidence pending |
| A6 | TypeScript 6.0.3 kept; 7.0.2 is newer | Stable releases only | 6.0.3 confirmed as a stable release | Proposed; evidence pending |
| A7 | Single-instance guard added; module layout simpler than the plan's tree | Approved feature scope | The guard confirmed to add no network access or new file locations | Proposed; evidence pending |
| A8 | `Cargo.toml` uses version ranges; exact versions are pinned in `Cargo.lock` | "Every dependency is pinned to an exact version" | Owner decision: accept lockfile pinning, or pin exact versions in `Cargo.toml` | Awaiting owner decision |
| A9 | Bundle the official KeePassXC 2.8.0 app for passwords, and retire Cipherleaf's own password section (P1). A scope change the owner requested, not a code difference. | The plan's Password vault and Generator features | Owner approval of [A9-bundle-keepassxc.md](A9-bundle-keepassxc.md); then the D0 evidence it lists | **Approved** 2026-10-22 13:07:49 UTC; D0 and D1 in progress |

## Changing this record

- Add new authority events at the top of the table.
- Change an amendment's status only with its evidence or the owner's decision, and say where that decision was recorded.
- Never edit the snapshot.
