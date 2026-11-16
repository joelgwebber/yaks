---
platforms: [all]
source: 
follows: 
blocked:
created: 2026-10-08T20:14:53Z
---
# Update check

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
yass update --check decides what to install: the binary's version and whether it's a release build, the repo's version, the latest release; source builds get the right command. Release builds carry the marker.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC1, AC2, AC13, AC16 in [plan.md](../plan.md)
- [x] `.goreleaser.yaml` sets `main.channel=release` beside `main.version`

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `cmd/yass` passes `main.channel` (set to `release` by `.goreleaser.yaml`) through `yass.MainChannel`
- [x] `update.go`: the latest release from the `releases/latest` redirect (no API); the repo's version from its stamped files (`repoVersion`, sharing `stampedFiles` with `versionNote`); the target and why (`planUpdate`); `--check`, `--latest`, `--version`
- [x] Source builds: a tagged version means `go install …@<target>`, anything else (pseudo-version, `+dirty`, `dev`) a clone rebuild
- [x] `--version` after a command that takes it is that command's option, not "print yass's version"
- [x] go test (`TestAssetName`, `TestPlanUpdate`, `TestSourceBuild`); e2e section 35 against a fake release server (522 ok)

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- `yass update` without `--check` says what it would install and that installing isn't built yet, with the release page; it stays out of `yass --help` and the README until `update-installs` lands, so a release in between doesn't advertise a half-built command (claude, 2026-10-08)
- A release server that can't be reached fails the command (exit 2) - without the latest version there's no answer to give; `--check` exits 0 whenever it can answer, update or not (claude, 2026-10-08)
- `yass update --version vX.Y.Z` is the command's own option: the top-level `--version` now applies only without a command, or to one that doesn't take it - the PRD names the option `--version` (claude, 2026-10-08)
- e2e serves fake releases with a few lines of Python (`releases` in tests/e2e.sh), redirecting `/releases/latest` as GitHub does; piece 2 adds the files (claude, 2026-10-08)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-08 (claude)
- Did: built and tested `yass update --check`. `go test`, `tests/e2e.sh` (522 ok), and a run against github.com (latest 0.3.0) pass. The piece is done.
- Next: `update-installs`.
