---
id: yaks-bcd7
title: 'Unknown tags: create and update say when a tag is not among the farm''s known tags'
status: shaving
type: feature
priority: 3
created: '2026-11-10T21:11:52Z'
updated: '2026-11-10T21:11:52Z'
parent: yaks-f04d
tags:
- cli
---

From yaks-f04d ("agents make up labels"). Known tags with descriptions already exist: the `tags:` map in `schemas/yak.md` (parsed in `src/schemas.rs`, shown by `yaks dok types`). Nothing uses them when a tag is written. Advisory, never a refusal: `yaks create --tags` / `yaks update --add-tag` print a warning naming a tag the farm does not list, with the nearest known ones, when the farm has a `schemas/yak.md` with `tags:`; silent when it has none.

---
▸ 2026-11-10T21:11:52Z [herd-unlock]
kind: move
from: hairy
to: shaving
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFQ8T5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving
