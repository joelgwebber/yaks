---
id: yaks-f301
title: 'Core: the farm data layer (layer 2)'
status: shorn
type: task
priority: 1
created: '2026-11-01T18:53:09Z'
updated: '2026-11-01T22:26:31Z'
parent: yaks-44b3
labels:
- wiki
---

Layer 2 of the umbrella: the CLI and store for pages, sources, schemas, links and commits. All code on the `wiki` branch.

---
▸ 2026-11-01T18:54:20Z [delta:farm-wiki]
kind: move
from: hairy
to: shaving
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-01T22:26:26Z [delta:farm-wiki]
kind: move
from: shaving
to: shorn
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn

---
▸ 2026-11-01T22:26:31Z [delta:farm-wiki]

Core layer complete on `wiki`. Every child is shorn:

- fields (yaks-1f34)
- schema 7 (yaks-37da)
- pages (yaks-9af0)
- links and peek (yaks-101d)
- per-change commits (yaks-a838)
- sources (yaks-2c42)
- schemas (yaks-8e75)
- merge driver and `yaks conflicts` (yaks-4c8c)
- attach encoding (yaks-fd7a)
- `yaks weight`/`yaks lfs` (yaks-2b68)

Each was verified with `cargo test --workspace`; the last run had 926 passing.
