---
id: yaks-5bc2
title: 'Per-herd known tags: let a herd override the farm''s tag list'
status: hairy
type: idea
priority: 4
created: '2026-11-14T18:04:22Z'
updated: '2026-11-14T18:04:22Z'
tags:
- config
- cli
---

Split out of yaks-640a when it was sheared as superseded. Known tags with descriptions live in `schemas/yak.md`, one list for the whole farm (`schemas::load`, `unknown_tag_warnings`). A consolidated farm holding several herds (`<herd>/yaks/`, projects with different vocabularies) gets one shared list, so a tag that is right for one project warns in another.

The farm config already cascades one level, herd value else global (`store::Config::default_type_for` and friends, [yaks-98d5](yaks-98d5)). The question is whether tag lists should too, and where a herd's list lives (a `<herd>/schemas/yak.md`, or a `tags:` map under the herd in `config.yaml`).

Only worth doing when a multi-herd farm actually complains. Noted in [yaks-640a](yaks-640a#2026-09-21T18:37:07Z) on 2026-09-21: "label definitions should be per-herd-overridable".
