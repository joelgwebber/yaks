# yass update: the binary updates itself: plan
<!-- Intent: how, and how we'll know it's done. With code, only mark boxes here ([/] in progress, [x] done);
     anything else is a plan revision in its own commit. -->

## Approach
<!-- The shape of the solution. Hard-to-undo calls go in design.md. -->
**M1 adds `yass update`,** in a new `internal/yass/update.go`, registered in [`main.go`](../../../internal/yass/main.go). It works in three stages, each usable on its own:
1. **Decide.** Read the binary's version and whether it's a release build ([design §1](design.md): `main.channel`, set in [`.goreleaser.yaml`](../../../.goreleaser.yaml) beside `main.version`, and passed to `yass.Main` from [`cmd/yass/main.go`](../../../cmd/yass/main.go)); the repo's version, from the stamps `versionNote` in [`upgrade.go`](../../../internal/yass/upgrade.go) already reads (factored out so both use it); and the latest release, from the `releases/latest` redirect ([§2](design.md)). Pick the target: the repo's version when it's ahead of the binary, else the latest, or `--latest`/`--version`. `--check` stops here and reports.
2. **Fetch and verify** the platform's archive and `checksums.txt` into a temporary folder; the checksum, then `gh attestation verify` when `gh` is there ([§3](design.md)).
3. **Swap** the binary in place ([§4](design.md)), then say what's next (`yass upgrade` where the repo is now behind).

Source builds stop after stage 1 with the command for their install method. The release source is `YASS_RELEASES_URL` for tests only ([§5](design.md)); e2e serves fake releases (archives built from the test binary, a matching `checksums.txt`) from a local `python3 -m http.server`, with a fake `gh` on PATH.

**M2 points people and agents at it.** `versionNote` and `yass upgrade`'s refusal name `yass update`. A new [`kit/.agents/skills/yass-update/SKILL.md`](../../../kit/.agents/skills/) ships like the other playbooks ([§6](design.md)); [`templates/agents.md`](../../../internal/yass/templates/agents.md) and `yass-status` point to it. [`docs/install.md`](../../../docs/install.md), [`docs/skills.md`](../../../docs/skills.md) and [`README.md`](../../../README.md) describe the command and the sixth playbook.

## Acceptance
<!-- Per milestone, one observable behavior each:
### M1
- [ ] AC1 (R1) Given …, when …, then … — verify: <test, flow, or manual steps> -->
### M1
- [x] AC1 (R1) Given a release binary at 0.4.0 in a repo stamped 0.5.0, with 0.6.0 the latest release, when you run `yass update --check`, then it prints the three versions and that `yass update` would install 0.5.0, changes nothing, and exits 0 — verify: e2e (fake releases)
- [x] AC2 (R1) Given the binary already at the latest release, then `--check` says it's up to date and exits 0 — verify: e2e
- [x] AC3 (R2) Given that repo, when you run `yass update`, then the binary is replaced by 0.5.0 (`yass version` says so) — verify: e2e
- [x] AC4 (R3) Given that update, then the output says 0.6.0 exists, gives `yass update --latest`, and says `yass upgrade` in each repo and a `chore: upgrade YASS` commit come after; with no newer release, it says neither — verify: e2e
- [x] AC5 (R4) Given a release binary outside any repo, or in a repo at or behind it, when you run `yass update`, then it installs the latest release — verify: e2e
- [x] AC6 (R4) Given `--latest` in a repo stamped 0.5.0, then it installs 0.6.0; given `--version v0.5.0`, then 0.5.0 — verify: e2e
- [x] AC7 (R4) Given `--version` older than the binary, then it refuses and changes nothing — verify: e2e
- [x] AC8 (R5) Given a release whose archive doesn't match `checksums.txt`, or isn't listed in it, then `yass update` refuses, says so, and the binary is unchanged — verify: e2e
- [x] AC9 (R6) Given `gh` on PATH whose `attestation verify` succeeds, then the output says provenance was verified; given one that fails, it refuses and the binary is unchanged — verify: e2e (fake `gh`)
- [x] AC10 (R6) Given no `gh` on PATH, then it installs after the checksum and says provenance wasn't checked and why; with `--require-provenance`, it refuses — verify: e2e
- [x] AC11 (R7) Given a downloaded binary that doesn't run, or reports a different version, then nothing is replaced — verify: e2e (a release whose binary is a shell script printing the wrong version)
- [x] AC12 (R7) Given the swap, then it's a rename in the binary's folder: no moment leaves the binary missing or partly written; on Windows the old one moves to `yass.exe.old` and is removed on the next run — verify: go test (swap function on a temp folder, both strategies); manual: one real update on macOS. *Windows not run by hand yet: only the go test covers its strategy; a real update there is the follow-up change `2026-10-09-validate-yass-update-on-windows-and-linux`.
- [x] AC13 (R8) Given a binary without the release marker, then `yass update` replaces nothing and prints `go install github.com/donjaime/yass/cmd/yass@<version>` for a tagged build, or how to rebuild a clone build — verify: e2e
- [x] AC14 (R9) Given the binary's folder isn't writable, then it says so before downloading anything, with what to do — verify: e2e (`chmod -w`)
- [x] AC15 (R9) Given the release source unreachable, or `--version` naming a release that doesn't exist, then it says which and changes nothing — verify: e2e (server stopped; a missing version)
- [x] AC16 (R10) Given each platform releases are built for, then `yass update` picks that platform's archive (`.zip` on Windows) — verify: go test (asset name per GOOS/GOARCH)
- [x] AC17 (R1–R10) Given the real 0.4.0-or-later release on GitHub, when a release binary runs `yass update --check` and `yass update`, then both behave as above against github.com — verify: manual: after the next release, on macOS. *Linux not run by hand yet: CI's e2e covers it against fake releases, not github.com; that's the follow-up change `2026-10-09-validate-yass-update-on-windows-and-linux`.

### M2
- [x] AC18 (R11) Given a binary older than the repo, then `yass status`'s note and `yass upgrade`'s refusal name `yass update` — verify: e2e
- [x] AC19 (R12) Given an update that leaves the binary ahead of the current repo, then the output ends with `yass upgrade` for that repo — verify: e2e
- [x] AC20 (R13) Given `yass init`, then `yass-update` is installed with the other playbooks (and in `.claude/skills/` with `--claude`); given an existing repo, `yass upgrade` adds it — verify: e2e
- [x] AC21 (R13) Given the `yass-update` playbook, then it checks both versions with `yass update --check`, asks before downloading, runs `yass update` then `yass upgrade`, shows the diff, and commits only with latitude — verify: manual: review; ask an agent to "update YASS" in a scratch repo with fake releases
- [x] AC22 (R14) Given the `AGENTS.md` section and `yass-status`, then both point to `yass-update` when the binary and the repo are out of step — verify: e2e (section text); manual: review
- [x] AC23 (R15) Given `docs/install.md`, `docs/skills.md` and `README.md`, then they describe `yass update` (what it verifies, how it picks a version, when to use a download, `install.sh` or `go install` instead) and the sixth playbook — verify: manual: review

## Pieces
<!-- PR-sized pieces, each its own folder in here: `yass new "<title>" --in <this change>`.
     `yass status` lists them; say here what order they go in and why. -->
1. **`2026-10-08-update-check`** (AC1, AC2, AC13, AC16, and the release marker). Deciding what to install, with no download: the stage everything else builds on, and useful alone.
2. **`2026-10-08-update-installs`** (AC3–AC12, AC14, AC15). Fetching, verifying and swapping. Needs piece 1.
3. **`2026-10-08-update-for-agents`** (AC18–AC23). Messages, the playbook and docs. Needs piece 2, so nothing points at a command that can't install yet.

AC17 runs after the first release that carries the marker.

## Validation
<!-- How the whole thing is verified before it's called done: suites, platforms, manual passes. -->
- `go test ./...`, `tests/e2e.sh` and `tests/examples.sh` on macOS and Linux in CI.
- A local release dry run, as for archive-that-scales: build two "releases" with the marker, serve them, update from one to the other, then `yass upgrade` a repo.
- AC17 against github.com after the release, on macOS. Windows and Linux by hand are the follow-up change `2026-10-09-validate-yass-update-on-windows-and-linux`.
