---
id: yaks-045c
title: 'yaks release changes: compute a release''s yaks from its commit range, --check for missing change lines; land warns'
status: shorn
type: feature
priority: 2
created: '2026-11-21T21:20:57Z'
updated: '2026-11-21T22:26:25Z'
parent: yaks-a4d8
depends_on:
- yaks-bcb8
tags:
- cli
- release
- git
---

Decisions 2 and 3 of [the changelog design](/doks/design/changelog.md#2-release-membership-is-computed-never-stored). Membership: a commit in the range (previous release tag..tag, on the checked-out branch) names the yak by id or a note's `commit:` (same rule as src/yakcommits.rs). Tags `v*` unless `git config yaks.releaseTags`. Output groups: shorn with a change line, shorn without (gaps; an ancestor's line in the same release covers a child), open/dead yaks named, commits naming no yak. `--json`; `--check` exits non-zero on gaps. `yaks land` warns when it lands a commit naming a shorn yak without a change line. Never stored.

#### 2026-11-21T22:02:42Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-21T22:02:42Z

*herd-files*

Claim (herd-files): worker rel-1 (Sonnet), base c20ad225. Scope: new src/release.rs (membership + report), `Command::Release` with subcommand `changes` in src/main.rs (leave room for `dok` and `changelog` later), a warning in src/land.rs after a landing, docs (docs/cli.md, README command table, yaks-coordinating skill's landing section + RELEASING.md note). Reuse src/changeline.rs (`latest`) and the commit-naming rule of src/yakcommits.rs. Out of scope: release doks, CHANGELOG.md. Evidence: integration tests on a temp repo with tags; a real run of `yaks release changes v0.0.39..v0.0.40` and `yaks release changes` (last tag..HEAD) on this repo, output attached. Judge: herd-files at landing.

#### 2026-11-21T22:11:17Z

*rel-1*

Start 2026-11-21T22:03:58Z. Top-level: /Users/joel/src/yaks/.delta/worktrees/8c177wnt6s3h/yaks (base c20ad225). Farm file: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/yaks/yaks/yaks-045c.md.

Plan: src/release.rs (range resolution, one `git log` over the range, membership by message scan with refs::scan_text + `commit:` notes via yakcommits::collect, report + text/json rendering); Command::Release{action: ReleaseCmd::Changes} in main.rs; warning in land.rs::record; docs.

#### 2026-11-21T22:26:00Z

*rel-1 · kind: verify · result: pass*

verify: `cargo test -p yaks` -> PASS (exit 0)

[v39-v40](yaks-045c/v39-v40.txt "real run on this repo: v39-v40")

[v40](yaks-045c/v40.txt "real run on this repo: v40")

[default](yaks-045c/default.txt "real run on this repo: default")

[default-check](yaks-045c/default-check.txt "real run on this repo: default-check")

[default](yaks-045c/default.json "yaks release changes --json (v0.0.40..HEAD)")

[gate](yaks-045c/gate.txt "cargo test -p yaks output (all green)")

#### 2026-11-21T22:26:23Z

*rel-1*

Decision: membership by message uses `refs::scan_text` (the whole-token id scan every other yaks reader uses, with the farm's ids as the known set) over one `git log --no-merges <range>`, not `git log --grep=<id>` per yak and not a substring test. Rejected: substring matching (`yaks-a1` would match `yaks-a1b2`), one git process per yak (the brief forbids it).

Decision: a `commit:` note field counts only when it is 7 or more hex digits, matched as a prefix of the commit's full sha. Rejected: any prefix (a 4-digit accident would name an unrelated commit).

Decision: `covered_by` / nesting uses the nearest ancestor (by `parent:`, walking past ancestors that are not in the range or have no line) that is itself a changelog entry of this range (shorn, line, not `section: none`). A `--no-change` ancestor covers nothing. A child with its own line is a normal entry but is printed nested under that ancestor. Rejected: only the direct parent (a grandchild under a parent with no line would be a false gap).

Decision: `--json` is one document with a flat `yaks` array carrying a `group` field (`change`, `no_change`, `covered`, `gap`, `unfinished`) plus `gaps` and `commits_without_yak`, instead of one array per group. Rejected: per-group arrays (a consumer filters one list; a yak never needs to move between lists).

Decision: with `--check --json` the failing one-line summary goes to stderr so stdout stays one parseable document; in text mode it is the last line of stdout. Exit code 1 in both. A passing `--check` in text mode prints `check: ok ...`.

Decision: `release changes` outside a code checkout (cwd not in git, or in the farm's own worktree) is an error, not an empty report; it reuses `Farm::code_repo` (made `pub(crate)`) rather than a second copy of the discovery.

Decision: version-bump exclusion is the named const `VERSION_BUMP = "Bump version"` and `is_version_bump` (subject prefix). Only commits that name no yak are filtered; a bump naming a yak would still count that yak.

Decision: the land warning is emitted per yak right after that yak's `recorded the landing` line, only for `LandedOutcome::Recorded` (so also in a dry run, which reports `Recorded`), and only for shorn yaks with no `kind: change` note at all (a `--no-change` note counts as a line). It never changes the exit status. Rejected: warning for AlreadyRecorded (the earlier landing already said it).

Observation (not a defect): the brief expected the default `yaks release changes` on this repo to show just yaks-c955 and yaks-bcb8. v0.0.40..HEAD at base c20ad225 has 13 commits naming 11 yaks: only yaks-bcb8 has a change line, the other 10 (including yaks-c955) are shorn with none, so they are listed as gaps and `--check` exits 1 there. That is the check working: those yaks predate the change-line convention.

#### 2026-11-21T22:26:24Z

*rel-1*

Summary: new src/release.rs (range resolution, membership, report, text/--json, --check), Command::Release/ReleaseCmd::Changes in main.rs, land warning in land.rs (missing_change_line), Farm::code_repo pub(crate), dead_code allow removed from changeline.rs. Tests: 9 unit tests in release.rs, tests/release_changes.rs (8), tests/land_change_warning.rs (1). Docs: docs/cli.md (release section, land warning), README row, yaks-coordinating skill (6 Landing step 1 + 5), RELEASING.md. Gate: cargo fmt --check clean; cargo test -p yaks green (1315 unit + all integration).

#### 2026-11-21T22:26:25Z

*rel-1 · kind: move · from: shaving · to: shorn · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAHdNjJIBxBCA0aLJyrhJPreUvVwLftHG*

moved: shaving -> shorn

#### 2026-11-21T22:26:25Z

*rel-1 · kind: change*

`yaks release changes [<range>] [--json] [--check]` lists the yaks a release's commits name, by change line and section, with the gaps, open yaks and commits naming no yak; `--check` exits 1 on a gap. `yaks land` warns when it records a landing on a shorn yak with no change line.
