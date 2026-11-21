---
id: yaks-c708
title: 'Release doks: yaks release dok <tag> writes doks/releases/<tag>.md with a generated changes block'
status: shaving
type: feature
priority: 2
created: '2026-11-21T21:20:58Z'
updated: '2026-11-21T22:32:10Z'
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
