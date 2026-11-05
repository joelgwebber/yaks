# The binary carries the playbooks: PRD
<!-- Intent: what should change or exist, and why. Not a description of the current system;
     the code and its docs are that. Edit this file only in commits without code. -->

## Why
<!-- The problem, with evidence. -->
Using YASS takes two things: the `yass` binary on your PATH, once per machine, and a few files in each repo, set up once and committed. Today one script, [`install.sh`](../../../install.sh), does both, and the playbooks and the hook only reach a repo through a downloaded release folder. That causes four problems:
- **The two jobs blur.** People can't tell which part of the command does what. The binary half fails quietly when its folder isn't on PATH, which happened on a stock Mac and needed [a docs fix](../../archive/2026-10-04-install-say-where-to-run-it-and-how-yass/change.md) to explain.
- **Joining a repo has no real path.** A teammate who clones a repo that already uses YASS needs only the binary. The docs fall back to copying it out of a release archive by hand ([install.md](../../../docs/install.md#joining-a-repo-that-already-uses-yass)).
- **Upgrading every repo needs the release every time.** `install.sh --upgrade` has to run from a release folder (or the piped download) for each repo, even though the binary you just upgraded could do it.
- **Nothing knows which version wrote a repo's files.** A teammate with an older binary and a repo with newer playbooks get no warning that they're out of step.

The binary already does part of the repo setup: `yass init` writes the yass folder, the `AGENTS.md` section and `yass.yaml`, it embeds the change templates, and it knows its version.

## Users and outcomes
<!-- Who it's for, and measurable targets ("p50 time to log < 5s"). -->
- **Someone adopting YASS** in a repo. Outcome: two steps, each saying what it did: get the binary, then `yass init`. The one-line install runs both, visibly.
- **A teammate joining a repo that already uses YASS.** Outcome: they get the binary and they're done; nothing in the repo needs to change.
- **Someone who uses YASS in several repos.** Outcome: setting up another repo needs no download (`yass init`), and neither does upgrading one: reinstall the binary once, then `yass upgrade` in each repo.
- **A monorepo team** whose parts keep their own `AGENTS.md` and skills folders. Outcome: YASS's agent files go in the folder they choose, the hook stays at the repo root, and one `yass upgrade` finds and upgrades every copy.
- **Anyone whose binary and repo are out of step.** Outcome: `yass status` says so, and says which to upgrade.

Targets:
- Setting up a second repo, or upgrading any repo after the binary: 0 downloads, 1 command.
- Joining a repo: 1 step (get the binary), no `install.sh`.
- Every repo set up by today's `install.sh` upgrades with `yass upgrade`, with no manual cleanup.

## Requirements
<!-- One line each: "- **R1** [M1] A user can …". IDs are never reused. -->
- **R1** [M1] A user with `yass` on their PATH sets up a repo with `yass init`, run for the repo itself: the playbooks, the hook script, the `AGENTS.md` section and the yass folder (or `yass.yaml`), the same files `install.sh` writes today, from the binary alone.
- **R2** [M1] `yass init` takes the setup options `install.sh` has today: `--claude` (Claude Code's copies and the `CLAUDE.md` import), `--global` (playbooks in the user folder), `--hooks` (turn on the hook for this clone) and `--path`, alongside the options it already has.
- **R3** [M1] `yass upgrade`, run anywhere in a repo, finds every YASS file the repo has, from the repo root down (playbooks in any folder's `.agents/skills/` or `.claude/skills/`, the YASS section of any `AGENTS.md`, the hook script), plus playbooks in the user folder, and upgrades each to the binary's version where it is. It says it upgraded and from which version. It never touches changes, `yass.yaml` or anything outside the `AGENTS.md` markers.
- **R4** [M1] A repo set up by an earlier `install.sh` upgrades with `yass upgrade` and ends up exactly as a fresh `yass init` at the same version would leave it.
- **R5** [M1] `install.sh` only installs the binary: it says where it put it, whether that folder is on PATH, and that `yass init` sets up a repo. The documented one-line install runs it and then `yass init` explicitly, so it still sets up a repo in one go. Given a repo path or a setup option the way earlier versions took them, it installs nothing and prints the commands to use instead (`yass init …`, or `yass upgrade` for `--upgrade`).
- **R6** [M1] A teammate joining a repo that already uses YASS installs only the binary, with `install.sh` or `go install`, and doesn't run `yass init`.
- **R7** [M1] Every file YASS installs records the version that wrote it, committed with it: the playbooks in their frontmatter, the `AGENTS.md` section in its marker, the hook script in a header. `yass upgrade` decides what to do by comparing those versions with its own. `yass --version` reports the binary's.
- **R8** [M2] `yass status` warns when the binary is older than the version that wrote a repo's YASS files (upgrade your binary), and notes when it's newer (`yass upgrade` would upgrade the repo).
- **R9** [M1] The README, `docs/install.md` and the landing page describe the new model: get the binary, run `yass init` in each repo, and joining needs only the binary. The README and `docs/install.md` say how to upgrade: reinstall the binary, then run `yass upgrade` in each repo and commit what it changed. The landing page doesn't cover upgrading.
- **R10** [M1] When a repo's YASS files are already at the binary's version, `yass upgrade` changes nothing and says the repo is already up to date.
- **R11** [M1] `yass upgrade` never downgrades. When any of a repo's YASS files is newer than the binary, it changes nothing, says which version wrote it, and says to upgrade the binary.
- **R12** [M1] A build from a clone stamps its own version (the release it follows plus a commit, and `+dirty` with uncommitted changes) and upgrades a repo stamped with an older release; a newer release then upgrades over it; a `+dirty` build rewrites files stamped with its own version. A binary with no version refuses to write.
- **R13** [M1] Upgrading replaces YASS's files, hand edits included. `yass init` and `yass upgrade` list every file they write, so the change shows up for review in `git diff`, and label any written outside version control (playbooks in the user folder, with `--global`).
- **R14** [M1] A file without a version stamp, as written by v0.1 and v0.2, counts as older than any stamped version, so `yass upgrade` upgrades it.
- **R15** [M1] `yass init <folder> --agents` puts the `AGENTS.md` section and the playbooks in that folder (and Claude Code's copies there with `--claude`), for a part of a monorepo that keeps its own agent files. The hook script and `core.hooksPath` stay at the repo root. Without `--agents`, `yass init <folder>` sets up only the folder's planning (its yass folder, or `yass.yaml` with `--path`) and touches no `AGENTS.md`.
- **R16** [M1] `yass init` on a repo or folder that's already set up writes only what's missing and keeps what's there. When the files there are older than the binary, it says `yass upgrade` would update them.

## Non-goals
<!-- What this change will not do. Agents treat these as walls. -->
- No Homebrew tap, other package manager or `yass self-update`. Getting the binary stays a release download, the one-liner or `go install`.
- No editing of shell startup files. Installers say what to add to PATH; the user adds it.
- No change to the playbooks' content, the hook's checks or the change file formats; only how they get into a repo.
- No detecting or preserving hand edits to YASS's files. They're committed with the repo, so an upgrade that replaces one shows up in review; `install.sh --upgrade` behaves the same way today.
- No change to where files land in a repo. Same paths as today, so existing repos upgrade in place.
- No new installer for Windows. `yass init` and `yass upgrade` work there natively; getting the binary stays a release download or `go install`.
- No changes to the monorepo change's trailer hooks themselves. Its `yass-trailers` piece waits for this change and ships its hook scripts through `yass init` and `yass upgrade`.

## Milestones
| Milestone | When it ships, a user can … |
|---|---|
| M1 | set up a repo, or a monorepo folder with its own agent files, with `yass init`, and upgrade every copy with one `yass upgrade` (or see it's already up to date), with no download after the binary; join a repo with just the binary; and read docs that describe it that way, including how to upgrade. A one-line install still sets up a repo, by running `yass init` itself. |
| M2 | see from `yass status` when their binary and a repo's YASS files are out of step, and which to upgrade |

## Open questions
- None. The version format and comparison are in [design.md](design.md).
