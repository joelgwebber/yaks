---
id: yaks-bcb8
title: 'Change lines: yaks shorn --change/--no-change, a kind: change note per yak'
status: shaving
type: feature
priority: 2
created: '2026-11-21T21:20:56Z'
updated: '2026-11-21T21:35:47Z'
parent: yaks-a4d8
tags:
- cli
- skills
---

Decision 1 of [the changelog design](/doks/design/changelog.md#1-the-change-line-is-a-note-on-the-yak). `yaks shorn <id> --change "<line>"` writes a `kind: change` note with the shear; `--no-change` writes one with `section: none`. Section from the yak's type (feature Added, bug Fixed, else Changed), `--meta section=` overrides; the latest change note wins (corrections are newer notes). Surfaces: clap help, docs/cli.md, the yaks and yaks-working skills and the brief (agents write it at shear), the TUI shear (offer the title as an editable start; never saved unedited without the human's say). A batch shear (several ids) takes `--change` only with one id.

#### 2026-11-21T21:35:47Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving
