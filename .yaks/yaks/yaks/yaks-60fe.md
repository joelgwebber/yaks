---
id: yaks-60fe
title: create --set key=value, as update has
status: shaving
type: feature
priority: 2
created: '2026-11-08T00:37:14Z'
updated: '2026-11-08T00:37:22Z'
parent: yaks-44b3
tags:
- cli
---

`yaks create` takes `--due`, `--every`, `--wake` and the other built-in fields, but no `--set key=value` for any other frontmatter field; only `update` has it. A harness that tags new yaks with its own fields has to create, then update: two commits, and a window where the yak exists without the field.

Found on Hearth's first live turn on a farm (2026-11-07, hearth-1ed0 in the Hearth repo's farm): the model followed a skill that said `yaks create "..." --due ... --set person=joel`, got `error: unexpected argument '--set'`, and recovered with `update`. Hearth's reminders and recurring yaks all carry `person:`.

Proposal: `create --set key=value` (repeatable) and `--unset` refused, with exactly `update --set`'s rules (built-in fields refused, lists and maps parse the same, round-trip as written), written in the same commit as the create. `--key` find-or-create leaves an existing yak's fields alone. Update `docs/cli.md`, the `--help` text and the `yaks` skill's command table.

---
▸ 2026-11-08T00:37:21Z [herd-reshape]
kind: move
from: hairy
to: shaving
thread: ksQQBgNNWOHrR9mtaBzuvtn_eJLOADWrTJIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-08T00:37:22Z [herd-reshape]

Claim (herd-reshape): worker set-1, from yaks main e8bc2c4f.
