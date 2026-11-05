# yass update: the binary updates itself: PRD
<!-- Intent: what should change or exist, and why. Not a description of the current system;
     the code and its docs are that. Edit this file only in commits without code. -->

## Why
<!-- The problem, with evidence. -->
Upgrading YASS takes two steps on two levels: get the newer binary (per machine), then run `yass upgrade` (per repo) and commit the diff ([docs/install.md](../../../docs/install.md#upgrading)). The second step is one command; the first is a manual download, checksum check and copy, or a `go install`, or a rebuild from a clone. [`the-binary-carries-the-playbooks`](../../archive/2026/10/2026-10-04-the-binary-carries-the-playbooks/) left self-update out on purpose (a non-goal: "getting the binary stays a release download, the one-liner or `go install`").

That first step is where upgrades stall:
- `yass status` already notices when the binary and a repo are out of step and says which to upgrade, but "upgrade your binary" leaves the user to find the release page and repeat the install by hand.
- A teammate whose repo was upgraded gets that note on every `yass status` until they do. Archive-that-scales makes that sharper: an older binary misreads month folders (`archive/2026` shows as a change).
- Agents are the main users of YASS, but they can't help with the binary today: no playbook covers upgrading, and nothing tells them which version the repo expects or how to get it safely.

## Users and outcomes
<!-- Who it's for, and measurable targets ("p50 time to log < 5s"). -->
- **People using YASS** on their own machines, who installed it from a release.
- **Their agents,** asked to "update YASS" or acting on `yass status`'s version note.
- **Teams,** whose members' binaries need to keep up with what the repo was upgraded to.

Targets:
- From "the repo is newer than your binary" to a working binary at the right version is one command, with no browser and no manual checksum.
- An agent asked to "update YASS" checks both levels, updates what's behind, and hands back a reviewed `chore: upgrade YASS` diff, asking before it downloads anything.
- A binary that can't be verified is never installed.

## Requirements
<!-- One line each: "- **R1** [M1] A user can …". IDs are never reused. -->
- **R1** [M1] `yass update --check` says the binary's version, the version the repo's YASS files are stamped with (inside a repo), the latest release, and what `yass update` would install, and changes nothing. It exits 0 whenever the check itself succeeds, whether or not an update is available.
- **R2** [M1] Inside a repo whose YASS files are newer than the binary, `yass update` installs exactly the repo's version, so the binary matches the repo rather than jumping ahead of it.
- **R3** [M1] When a release newer than the one it installs exists, `yass update` (and `--check`) says so, with the command to get it (`yass update --latest`) and what comes after: `yass upgrade` in each repo, then committing the diff (`chore: upgrade YASS`).
- **R4** [M1] Outside a repo, or when the binary is already at or past the repo's version, `yass update` installs the latest release. `--latest` and `--version <vX.Y.Z>` choose otherwise. It never installs an older version than the binary's.
- **R5** [M1] `yass update` installs nothing that doesn't match its entry in the release's `checksums.txt`.
- **R6** [M1] When the GitHub CLI is installed, `yass update` also verifies the release's signed build provenance (`gh attestation verify`). It says which checks ran, and `--require-provenance` refuses to install without that one.
- **R7** [M1] `yass update` replaces the binary it's running as, in place; if it's interrupted or fails, the old binary still works. This holds on Windows.
- **R8** [M1] A binary built from source (`go install`, or a build from a clone) isn't replaced: `yass update` prints the command that updates it the way it was installed.
- **R9** [M1] When it can't write the binary's folder, can't reach the releases, or the requested version doesn't exist, `yass update` says what went wrong and what to do, and changes nothing.
- **R10** [M1] `yass update` works on every platform releases are built for (macOS and Linux on amd64 and arm64, Windows).
- **R11** [M2] `yass status`'s note for a binary older than the repo, and `yass upgrade`'s refusal for the same, name `yass update`.
- **R12** [M2] After installing, `yass update` says what's next: `yass upgrade` in each repo whose YASS files are now older than the binary.
- **R13** [M2] A `yass-update` playbook, installed and upgraded like the others: for "update YASS" or the version note, it checks both versions (`yass update --check`), asks before downloading anything, runs `yass update`, then `yass upgrade` in the repo, shows the diff, and commits only with the latitude to.
- **R14** [M2] The `AGENTS.md` section and `yass-status` point to `yass-update` when the binary and the repo are out of step.
- **R15** [M2] `docs/install.md` and `README.md` describe `yass update`: what it verifies, how it picks a version, and when to use the release download, `install.sh` or `go install` instead.

## Non-goals
<!-- What this change will not do. Agents treat these as walls. -->
- Updating automatically or in the background. YASS updates only when someone runs the command.
- Checking for new releases on every `yass status` by default (network calls in a command people and hooks run constantly).
- Downgrading.
- Mirrors or another release source (`YASS_RELEASES_URL`) for air-gapped networks; releases come from github.com/donjaime/yass.
- A Homebrew tap or other package managers, and updating binaries they installed.
- Verifying provenance inside the binary (Sigstore libraries); provenance is checked through the GitHub CLI when it's there.

## Milestones
| Milestone | When it ships, a user can … |
|---|---|
| M1 | run `yass update` to get a verified binary matching their repo (or the latest), and learn when a newer one exists and what to do after |
| M2 | ask their agent to "update YASS" and get both levels done, with YASS's messages and docs pointing the way |

## Open questions
- None.
