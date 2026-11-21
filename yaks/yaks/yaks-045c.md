---
id: yaks-045c
title: 'yaks release changes: compute a release''s yaks from its commit range, --check for missing change lines; land warns'
status: shaving
type: feature
priority: 2
created: '2026-11-21T21:20:57Z'
updated: '2026-11-21T22:26:00Z'
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
