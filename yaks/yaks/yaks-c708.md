---
id: yaks-c708
title: 'Release doks: yaks release dok <tag> writes doks/releases/<tag>.md with a generated changes block'
status: shaving
type: feature
priority: 2
created: '2026-11-21T21:20:58Z'
updated: '2026-11-21T23:10:02Z'
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

#### 2026-11-21T23:09:58Z

*rdok-1*

Decision: derived fields. A rerun updates `range` and `tag` always, and `date` only once the version is tagged (the tag commit's day in the farm's zone; untagged: today at creation, kept afterwards). `title`, `description`, `version`, `yanked`, prose and any hand-added field are never touched. Rejected: rewriting only the block (a dok created untagged would keep a stale `range: v0.0.40..HEAD` and no `tag:` forever, and `check` compares over `range:`); rewriting every field (would clobber hand edits).

#### 2026-11-21T23:09:59Z

*rdok-1*

Decision: dok names may hold a dot inside (`doks/releases/v0.0.41.md`). `doks::path_problem` allowed only `a-z0-9-`, so the version could not be a file name. Allowed `.` inside a part (not at an edge, no `..`) for every dok rather than special-casing releases. Rejected: `v0-0-41.md` (the brief and design say `<tag>.md`; breaks `release dok <tag>` round-tripping).

#### 2026-11-21T23:10:00Z

*rdok-1*

Decision: `yaks check` compares each release dok over the `range:` it records, not over a recomputed range. A release dok with no `range:` or no block markers is skipped. Rejected: recomputing the default range (`HEAD` moves, so a tagged release's block would look stale after the next commit). A range ending `..HEAD` (an unreleased dok) is stale when more yaks land: intended, rerun it. Finding kind `stale-release`; without a code checkout, one note like the code kinds.

#### 2026-11-21T23:10:00Z

*rdok-1*

Decision: entry order within a section is oldest yak first (created, then id). Yak ids are random, and the report's id sort gave a meaningless, unstable order. Rejected: id order (random); commit order (a yak with several commits has no single position).

#### 2026-11-21T23:10:01Z

*rdok-1*

Decision: title is `<herd> <version>` when the farm declares a default herd (`yaks v0.0.41` here), else just the version. `yanked: false` is NOT written: a human adds `yanked: true` when needed; it is in the schema. A rerun on a dok with no block markers is refused, never overwritten. Nothing is written (no commit) when the dok would not change. `schemas/release.md` is written under the same lock, once.

#### 2026-11-21T23:10:02Z

*rdok-1*

Decision: shared the block's membership with `release changes` (one `report_in`, `section_order`, `top_entries`); only `block` renders. Also moved the `Repo` test harness from tests/release_changes.rs into tests/common/mod.rs so tests/release_dok.rs reuses it. Block links use the schema-9 farm-root form `/<herd>/yaks/<id>.md` (as `links::note_link` writes), so they render on GitHub.
