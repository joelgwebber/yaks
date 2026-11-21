---
id: yaks-c708
title: 'Release doks: yaks release dok <tag> writes doks/releases/<tag>.md with a generated changes block'
status: shaving
type: feature
priority: 2
created: '2026-11-21T21:20:58Z'
updated: '2026-11-21T23:09:58Z'
parent: yaks-a4d8
depends_on:
- yaks-045c
tags:
- cli
- doks
- release
---

Decision 4 of [the changelog design](/doks/design/changelog.md#4-a-release-is-a-dok-with-a-generated-block). Kind `release` (schema: version, date, tag, range, yanked). Creates the dok or rewrites only the block between `<!-- yaks:changes -->` and `<!-- /yaks:changes -->`, grouped by section, families nested, each entry linking its yak; a yak with no change line listed by title, marked `(no change line)`. Prose outside the block is never touched. `yaks check` flags a release dok whose block is out of date. Skill: the release workflow (yaks-coordinating / RELEASING.md) writes the headlines and pins images with @<sha>.

#### 2026-11-21T22:32:10Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-21T22:32:11Z

*herd-files*

Claim (herd-files): worker rdok-1 (Sonnet), base 5d8a524c. Scope: src/release.rs (a renderer for the changes block reusing the report), `ReleaseCmd::Dok` in src/main.rs, a `release` dok schema written on first use (or via `dok define` machinery), `yaks check` flagging a stale block (src/check.rs), docs + the yaks-coordinating skill's release step + RELEASING.md. Out of scope: CHANGELOG.md (yaks-7278). Evidence: tests; a real `yaks release dok v0.0.40` and an unreleased one on this farm, rendered in the TUI headless. Judge: herd-files.

[rdok-dryrun-v0.0.40](yaks-c708/rdok-dryrun-v0.0.40.txt "yaks release dok v0.0.40 --dry-run on this farm")

[rdok-dryrun-v0.0.41](yaks-c708/rdok-dryrun-v0.0.41.txt "yaks release dok v0.0.41 --dry-run (unreleased: v0.0.40..HEAD)")

[rdok-tui-release-dok](yaks-c708/rdok-tui-release-dok.txt "Headless TUI frame (120x40): a release dok in the Doks tab, scratch farm")
