---
id: yaks-8e75
title: 'Schemas as data: schemas/<type>.md, warn-only checks, defaults, yak field extensions'
status: shorn
type: feature
priority: 2
created: '2026-11-01T18:53:10Z'
updated: '2026-11-01T21:36:47Z'
parent: yaks-f301
depends_on:
- yaks-1f34
tags:
- wiki
- schema
verify: cargo test --workspace
---

Implements yaks-8384: `schemas/<type>.md` (Hearth's format: `fields: {name: {type, description, default}}`), warnings from `page set`/`doctor`, defaults on `page new`, and `schemas/yak.md` extending the yak's built-in fields.

---
▸ 2026-11-01T20:09:16Z [delta:farm-wiki]

Joel (yaks-8384): `schemas/yak.md` should also carry known tags with descriptions, covering yaks-640a and yaks-f04d (explicit label definitions in config).

---
▸ 2026-11-01T21:29:02Z [delta:farm-wiki]

kind: move
from: hairy
to: shaving
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-01T21:34:45Z [delta:farm-wiki]

kind: checkpoint
branch: wiki
commit: 21617d6
---

Done on the `wiki` branch.

## What it does

- [`src/schemas.rs`](code:src/schemas.rs) reads `schemas/<type>.md` in Hearth's format. A field is `{type, required, default, description}` or the shorthand `name: <type>`. Kinds: `text`, `number`, `bool`, `date`, `datetime`, `money`, `link`, `list`; anything else reads as text, with a warning.
- Advice only, per Hearth's decision on hearth-1c4e: no write is ever refused.
  - `page new` fills in defaults.
  - `page new` and `page set` print warnings: a usually-set field missing, a value of the wrong kind, a field the schema doesn't list. OKF bookkeeping keys are exempt.
  - `doctor` reports `schema-mismatch`.
- `schemas/yak.md` extends yaks: its fields describe extra yak fields (the built-ins are never reported), and its `tags:` map lists the farm's known tags with descriptions. That covers yaks-640a and yaks-f04d at the data level; autocompletion is still theirs to do.
- `yaks page types` lists the untyped kinds plus every schema with its fields and tags.
- `yaks page query --rows [--type] [--where col=value]` lists table-page rows (Hearth's table pages).
- Docs: typed pages in `docs/cli.md`, plus doctor. Skill: typed pages under Pages, and the Tags section points at `page types`.

## Evidence

- Scratch farm with `joel`'s real `schemas/agent-platform.md` and `wiki/topics/agent-platforms.md`: `page types` shows its nine fields with descriptions, and `page query --rows --type agent-platform --where category=coding-assistant` returns the five coding assistants from the table.
- `page new wiki/home/dishwasher.md --type appliance --set purchased=spring` wrote `warranty: none` from the default and warned about the missing `name` and the non-date `purchased`. The `due: soon` yak field was flagged by doctor against `schemas/yak.md`.
- `tests/schemas.rs` (through the binary) and `schemas::tests`. `cargo test --workspace`: 910 passed.

---
▸ 2026-11-01T21:36:47Z [delta:farm-wiki]

kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-01T21:36:47Z [delta:farm-wiki]

kind: move
from: shaving
to: shorn
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
