---
id: yaks-da00
title: 'CLI restructure: verbs on any document, grouped help, shared flag sets, one JSON contract'
status: hairy
type: feature
priority: 2
created: '2026-11-27T02:38:26Z'
updated: '2026-11-27T02:38:52Z'
parent: yaks-7e32
depends_on:
- yaks-23c9
- yaks-362d
tags:
- cli
---

Section 11 of [the design](/doks/design/one-document-model.md#11-cli): verbs take any document address; `dok` keeps `types`, `define`, `row`. `yaks --help` grouped by function (a help template), with flag sections: global, shared sets (filter, output, write) declared once with `#[command(flatten)]`, then the command's own. Collapse the synonyms (`--description`/`--body`/`--text`, the three `--commit`s, `--herd`/`--scope`/`--prefix`), old names as aliases for a release. One `--json` contract per document (yaks-e3b1). Then the skills and docs rewrite.
