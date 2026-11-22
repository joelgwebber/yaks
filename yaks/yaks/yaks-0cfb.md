---
id: yaks-0cfb
title: 'Docshots: regenerate every doc SVG once to absorb toque renderer drift; refresh the Sheds tab shots'
status: hairy
type: chore
priority: 3
created: '2026-10-31T17:12:34Z'
updated: '2026-11-22T16:40:59Z'
tags:
- docs
- tui
---

Reported by both `sheds-1` and `branches-1` (2026-10-31): `cargo test -p yaks docshots -- --ignored` rewrites about 24 SVGs under `docs/assets/` with renderer drift unrelated to the change at hand (`opacity` becomes `fill-opacity`, plus `xml:space`). So both workers reverted them, and the Sheds tab doc images were not regenerated for the new actor-based view.

Fix: regenerate all docshots once in their own commit (after checking that the drift comes from a toque update and is not a regression), so that later runs only touch what changed.

#### 2026-11-22T16:40:59Z

*herd-files*

Duplicate of yaks-daa4 (same docshot refresh, filed twice); worked there in the pre-v0.0.41 batch, which also covers the Sheds tab shots this one names.
