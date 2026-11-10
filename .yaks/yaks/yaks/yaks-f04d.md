---
id: yaks-f04d
title: Known labels in configuration
status: shaving
type: feature
priority: 3
created: '2026-08-26T13:20:21Z'
updated: '2026-11-10T22:03:46Z'
tags:
- config
- labels
---

We don't want to *limit* labels to a predefined set, but it would be helpful to have some of them predefined in the config.

That way we can give them descriptions, which can in turn provide useful context to an agent for picking labels (it tends to make a lot of shit up now).

---
▸ 2026-09-23T21:35:38Z [coordinator]

Shaved as parent of yaks-7cb3 (lightning-round lane 'labels').

---
▸ 2026-09-23T21:45:01Z [lane-labels]

yaks-7cb3 landed label normalization on lane-labels: model::normalize_labels is the shared parser (commas/whitespace split) and doctor's malformed-label check -- a known-labels config can build on is_canonical_label/normalize_labels.

---
▸ 2026-10-12T21:49:45Z [Joel Webber]

moved: shaving -> hairy

---
▸ 2026-11-10T21:11:51Z [herd-unlock]
kind: move
from: hairy
to: shaving
child: yaks-8d6a
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFQ8T5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-10T21:15:14Z [herd-unlock]

herd-unlock-2, 2026-11-10: the "predefined labels with descriptions" this asks for already exist as the `tags:` map of `schemas/yak.md` (shown by `yaks dok types`, read by the `yaks` skill's Tags section); this farm had none, so I wrote one: [schemas/yak.md](/schemas/yak.md), 22 tags from the ones in use, with `ui`, `rust` and `phase*` named as retired. What was missing is anything using them when a tag is written: yaks-bcd7 (CLI warns on an unknown tag) and yaks-8d6a (Tab completes tags in the TUI form, known ones first) are running.

---
▸ 2026-11-10T22:03:46Z [herd-unlock]

Done (herd-unlock-2): known tags with descriptions live in schemas/yak.md (this farm now has 22); yaks-bcd7 warns on an unknown tag from the CLI, yaks-8d6a completes tags in the TUI form with known ones first. Both landed and checked on a copy of the real farm.
