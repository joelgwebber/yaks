# The binary carries the playbooks: plan
<!-- Intent: how, and how we'll know it's done. With code, only mark boxes here ([/] in progress, [x] done);
     anything else is a plan revision in its own commit. -->

## Approach
<!-- The shape of the solution. Hard-to-undo calls go in design.md. -->
**The binary carries the kit.** `kit/` (the playbooks and the hook script) becomes a Go package with an `embed.go` (`//go:embed all:.agents tools`), so every `yass` binary, from a release, a clone or `go install`, carries the files for its own version. The files in `kit/` stay unstamped; the version is added as they're written.

**`yass init` writes everything.** [`cmdInit`](../../../internal/yass/commands.go) already writes the yass folder, `yass.yaml` and the `AGENTS.md` section. It takes over the rest of what [`install.sh`](../../../install.sh) does: the playbooks (`.agents/skills/`, or the user folder with `--global`), Claude Code's copies and the `CLAUDE.md` import (`--claude`), the hook script, and `core.hooksPath` (`--hooks`), with the same paths, so nothing moves in existing repos. Run for a folder (`yass init <folder>`), it sets up only that folder's planning, unless `--agents` asks for the `AGENTS.md` section and the playbooks (and Claude's copies with `--claude`) in that folder too; the hook and `core.hooksPath` always belong to the repo root. On a repo or folder that's already set up, it writes what's missing, keeps the rest, and says `yass upgrade` would update files older than the binary. Every file it writes is stamped as [design.md](design.md) §2 describes: `metadata.yass-version` in each playbook's frontmatter, `version=` in the `AGENTS.md` begin marker, and `# yass-version:` in the hook header.

**`yass upgrade` upgrades.** A new command, run anywhere in a repo. It finds every YASS file from the repo root down, as [design.md](design.md) §6 describes (`git ls-files` for playbooks in any `.agents/skills/` or `.claude/skills/`, every `AGENTS.md` with the YASS marker, and the hook), plus the user folder's playbooks. It reads each one's stamp and compares it with its own version (design §1). Older or unstamped files get upgraded, files at the same version are left alone (unless the binary is `+dirty`), and if any file is newer, nothing is written and it says to upgrade the binary. It prints whether it upgraded (from which version) or found everything up to date, and lists every file it wrote, labeling any outside version control. It takes no options: it upgrades what's there, where it is.

**`install.sh` becomes a binary installer, and only that.** It downloads and verifies a release (when piped), or uses the binary next to it (a release folder or a clone's `bin/`), copies it to `--bin-dir`, says where it went and whether that's on PATH, and says `yass init` sets up a repo. The documented one-liner chains the two explicitly: `curl -fsSL https://raw.githubusercontent.com/donjaime/yass/main/install.sh | bash -s -- --bin-dir ~/.local/bin && ~/.local/bin/yass init`. It calls the binary by its full path, so it works before PATH is fixed. Given a repo path or a setup option (`--claude`, `--global`, `--hooks`, `--path`, `--upgrade`) as earlier versions took them, `install.sh` installs nothing, exits non-zero and prints the commands to run instead: the binary install, then `yass init …` (or `yass upgrade`, for `--upgrade`). Release archives drop the kit files, since the binary carries them; `install.sh` stays in them as the way to put the binary on PATH.

**`yass status`** (M2) reads the same stamps and adds one line when the binary and the repo are out of step.

**Docs.** `yass init`'s options and `yass upgrade` go in the README's CLI reference with the code that adds them, along with a short "Upgrading" section in the README and `docs/install.md` once `yass upgrade` exists: reinstall the binary, then run `yass upgrade` in each repo and commit what it changed. The install model in the README, `docs/install.md` and the landing page is rewritten in the piece that changes `install.sh`, so the docs never describe a flow that doesn't exist. The landing page doesn't cover upgrading.

## Acceptance
<!-- Per milestone, one observable behavior each:
### M1
- [ ] AC1 (R1) Given …, when …, then … — verify: <test, flow, or manual steps> -->
### M1
- [x] AC1 (R1) Given a fresh git repo and a `yass` binary with no release files beside it, when you run `yass init`, then it writes the five playbooks under `.agents/skills/`, an executable `tools/yass/githooks/pre-commit`, the `AGENTS.md` section and the yass folder — verify: e2e (binary copied alone into an empty folder on PATH)
- [x] AC2 (R1) Given that repo, then each file matches its source in `kit/` (or `templates/agents.md`) except for its version stamp — verify: e2e (diff with the stamp lines removed)
- [x] AC3 (R1) Given a binary built by `go install` or a release, then it carries all five playbooks and the hook — verify: go test (the embedded files list)
- [x] AC4 (R2) Given `yass init --claude`, then it also writes the playbooks under `.claude/skills/` and puts `@AGENTS.md` at the top of `CLAUDE.md` once, keeping what was there — verify: e2e (with and without an existing `CLAUDE.md`, run twice)
- [x] AC5 (R2) Given `yass init --global`, then the playbooks go to `~/.agents/skills/` (or `$YASS_SKILLS_DIR`), and to `~/.claude/skills/` with `--claude`, and none to the repo — verify: e2e (with a temporary `HOME`)
- [x] AC6 (R2) Given `yass init --hooks`, then `core.hooksPath` is `tools/yass/githooks` for this clone — verify: e2e
- [x] AC7 (R2) Given `--path`, `--private` and `--no-agents`, then they behave as they do today — verify: e2e (the existing sections, run through `yass init`)
- [x] AC8 (R7) Given a fresh `yass init` by a binary at version V, then every playbook's frontmatter has `metadata.yass-version: "V"` and still has its `name` and `description`, the `AGENTS.md` begin marker has `version=V`, and the hook has `# yass-version: V` — verify: go test (the frontmatter parses as YAML); e2e (all three stamps)
- [x] AC9 (R3) Given a repo whose YASS files are stamped with an older version, when you run `yass upgrade`, then every one is rewritten at the binary's version, and it prints that it upgraded, from which version to which — verify: e2e
- [x] AC10 (R3) Given a repo set up with `--claude`, or with playbooks only in the user folder, when you run `yass upgrade`, then those copies are upgraded too, where they are, and no new ones are added — verify: e2e
- [x] AC11 (R3) Given changes, a `yass.yaml`, text outside the `AGENTS.md` markers and other content in `CLAUDE.md`, when `yass upgrade` runs, then none of them changes — verify: e2e (checksums before and after)
- [x] AC12 (R10) Given a repo already at the binary's version, when you run `yass upgrade`, then no file changes, `git status` is clean, and it says the repo is already up to date at that version — verify: e2e
- [x] AC13 (R11) Given a repo with any YASS file stamped newer than the binary, when you run `yass upgrade`, then no file changes, it names the newer version and says to upgrade the binary, and it exits non-zero — verify: e2e
- [x] AC14 (R12) Given versions to compare, then a clone build after a release (`v0.3.1-0.…`) is newer than that release, a later release is newer than the clone build, `+dirty` is ignored in comparing, and an equal version written by a `+dirty` binary is rewritten — verify: go test (a table of version pairs and the expected action)
- [x] AC15 (R12) Given a binary with no version (`dev`), when you run `yass init` or `yass upgrade`, then it writes nothing, says how to get a versioned build, and exits non-zero — verify: go test (the decision for an empty version); e2e (a binary built with `-buildvcs=false`)
- [x] AC16 (R13) Given `yass init` or `yass upgrade` writes files, then it prints each one, and marks those outside version control (user-folder playbooks) as such — verify: e2e
- [x] AC17 (R13) Given a hand-edited playbook in a repo at an older version, when `yass upgrade` runs, then the edit is replaced, and `git diff` shows it — verify: e2e
- [x] AC18 (R14) Given a repo set up by v0.2.0's `install.sh` (no stamps), when you run `yass upgrade`, then it upgrades every YASS file — verify: e2e (a fixture installed from v0.2.0's kit)
- [x] AC19 (R4) Given that upgraded repo and a fresh `yass init` of an empty repo with the same options and binary, then their YASS files are identical — verify: e2e (`diff -r` of the YASS files, with `--claude` and `--hooks` too)
- [x] AC20 (R5) Given `install.sh --bin-dir DIR`, run inside a repo or outside one, then it copies the binary to DIR, says where it put it and whether DIR is on PATH, says `yass init` sets up a repo, and writes nothing in the current folder — verify: e2e (both places; `git status` clean)
- [x] AC21 (R5) Given `install.sh` with a repo path or any of `--claude`, `--global`, `--hooks`, `--path`, `--upgrade`, then it installs nothing, exits non-zero, and prints the commands to run instead: `install.sh --bin-dir …`, then `yass init …` with the options carried over (or `yass upgrade`, for `--upgrade`) — verify: e2e (each old form)
- [x] AC22 (R5) Given the documented one-liner's two commands, run in a fresh repo with DIR not on PATH, then the binary lands in DIR and the repo is set up — verify: e2e (`install.sh` from the clone, then `DIR/yass init`, with PATH lacking DIR)
- [x] AC23 (R3) Given `yass upgrade` run outside a git repo, then it changes nothing and says it works inside a git repo — verify: e2e
- [x] AC24 (R5) Given the piped one-liner, then `install.sh` still downloads the latest release and checks it against `checksums.txt` before installing the binary, and the chained `yass init` sets up the repo — verify: manual: run the documented one-liner against a release candidate in a fresh repo
- [x] AC25 (R1, R5) Given a release archive, then it has the binary, `install.sh` and the docs, and no `kit/` files, and its `install.sh` installs the binary — verify: CI (GoReleaser snapshot build); e2e (an unpacked-release folder without `kit/`)
- [x] AC26 (R6) Given a repo that already uses YASS, when a teammate runs `install.sh` (or `go install`) without `yass init`, then the binary is installed and no file in the repo changes — verify: e2e (`git status` clean); manual: docs review
- [x] AC27 (R9) Given the README, `docs/install.md` and the landing page, then they describe the model: get the binary, run `yass init` in each repo, and joining needs only the binary; the README's CLI reference lists `yass init`'s options and `yass upgrade` — verify: manual: review; `tests/site.sh` (the page's commands match the docs)

- [x] AC31 (R15) Given a monorepo set up at the root and `yass init apps/web --agents`, then `apps/web/` gets its yass folder, an `AGENTS.md` with the YASS section and the playbooks under `apps/web/.agents/skills/`, and no second hook; with `--claude`, Claude's copies and the `CLAUDE.md` import go in `apps/web/` too — verify: e2e
- [x] AC32 (R15) Given `yass init apps/web` without `--agents`, then it writes only `apps/web/yass/` (or `apps/web/yass.yaml` with `--path`), and no `AGENTS.md`, here or at the root — verify: e2e
- [x] AC33 (R16) Given a repo already set up, with files stamped older than the binary, when you run `yass init`, then it writes only what's missing, keeps the rest, and says `yass upgrade` would update them — verify: e2e
- [x] AC34 (R3) Given a monorepo with agent files at the root and in `apps/web/` and `services/api/`, all stamped older, when you run `yass upgrade` from `services/api/`, then every copy in the repo is upgraded where it is — verify: e2e
- [x] AC35 (R9) Given the README and `docs/install.md`, then each says how to upgrade: reinstall the binary, then run `yass upgrade` in each repo and commit what changed; the landing page doesn't mention upgrading — verify: manual: review

### M2
- [x] AC28 (R8) Given a repo whose YASS files are stamped newer than the binary, when you run `yass status`, then it warns which version wrote them and says to upgrade the binary — verify: e2e
- [x] AC29 (R8) Given a repo whose YASS files are stamped older than the binary, when you run `yass status`, then it notes that `yass upgrade` would upgrade the repo — verify: e2e
- [x] AC30 (R8) Given a repo at the binary's version, or with no YASS files installed, then `yass status` says nothing about versions — verify: e2e

## Pieces
<!-- PR-sized pieces, each its own folder in here: `yass new "<title>" --in <this change>`.
     `yass status` lists them; say here what order they go in and why. -->
1. `init-writes-the-whole-repo` (M1, done): the kit embedded in the binary, the stamps, and `yass init` writing every file with the new options, keeping files that already exist.
2. `agent-files-per-folder` (M1): `yass init <folder> --agents`, the agent files' location separate from the repo root's hook, `yass init <folder>` without `--agents` leaving `AGENTS.md` alone, and `yass init` on a set-up repo pointing at `yass upgrade`. It goes before `yass upgrade` because that has to find the nested copies this creates.
3. `yass-upgrade` (M1): the new command, with discovery from the repo root, reading stamps and comparing versions, upgrading or saying up to date or refusing, unstamped files, the file listing, the Go 1.24 floor (design §5), and the upgrade docs in the README and `docs/install.md`. Blocked on piece 2.
4. `install-sh-installs-the-binary` (M1): `install.sh` as a binary installer only, its refusal of old-style arguments (pointing `--upgrade` at `yass upgrade`), release archives without the kit, the one-liner that chains `yass init`, and the install docs and landing page rewritten to match. Blocked on piece 3.
5. `status-knows-the-versions` (M2): the `yass status` line. Blocked on piece 3 for the stamp reading; it can go before or after piece 4.

Once piece 4 lands, the monorepo change's `yass-trailers` piece (which waits on this change) ships its hooks through `yass init` and `yass upgrade`.

## Validation
<!-- How the whole thing is verified before it's called done: suites, platforms, manual passes. -->
- CI on each piece: `go test`, `tests/e2e.sh`, `tests/examples.sh` and `tests/site.sh`, on Ubuntu and macOS, plus the GoReleaser snapshot build.
- Dogfood: after piece 3, `yass upgrade` in this repo upgrades its own playbooks, Claude copies and hook, and the examples' monorepo checks nested copies. The diff should be stamps only, and `install.sh` is no longer needed here.
- A release candidate (`v0.3.0-rc.1`) before release: the piped one-liner in a fresh repo (AC24), an unpacked archive on macOS and Linux, and upgrading a repo set up by v0.2.0.
- The release notes say how to upgrade: get the new binary, then run `yass upgrade` in each repo.
