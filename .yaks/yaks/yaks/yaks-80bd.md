---
id: yaks-80bd
title: 'Small tool gaps: mark sources, write schemas, backdate imports'
status: shaving
type: task
priority: 3
created: '2026-11-06T04:28:32Z'
updated: '2026-11-06T23:13:38Z'
parent: yaks-fcdc
tags:
- cli
- doks
---

Smaller gaps against Hearth's tools, each a hand edit today:

- **Mark a source uncited on purpose** (`source_mark`: `superseded_by` or `uncited`). `yaks check` honours the fields in `source.md`, but nothing sets them and `yaks-research` doesn't say how.
- **Write a schema** (`page_define_type`): `schemas/<kind>.md` is written by hand; the housekeeping pass merges kinds and adds fields, so a command keeps it well formed.
- **Backdate an import** (`HEARTH_DATE` / `-date`): commits and saved sources dated to the day the imported notes describe.

Split any that grows.

---
▸ 2026-11-06T23:12:41Z [herd-upkeep-2]

Scope (herd-upkeep-2, 2026-11-06): the third bullet, backdating, is now yaks-0fa7 (under yaks-3f35, another thread's), which covers session notes and promotions. This yak keeps marking sources and writing schemas.

---
▸ 2026-11-06T23:12:45Z [herd-upkeep-2]
kind: move
from: hairy
to: shaving
thread: ksQQKUp3Ux8dTNqgb6cSSWpUMpLOAEX_fZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-06T23:13:38Z [herd-upkeep-2]

Claim (yakherd herd-upkeep-2, 2026-11-06): worker `tools-1`.

- **Scope:** `yaks source mark <id or sources/<id>/source.md> (--uncited "<why>" | --superseded-by sources/<id>/source.md | --clear)` sets the field `yaks check` already honours in `source.md`, one commit; refuses a missing source or a `--superseded-by` that doesn't exist. And `yaks dok define <kind> --field name=type[!]...` (`!` marks required; types `text number bool date datetime money link list`) with optional `--title`, `--description`, `--stale-after`: creates `schemas/<kind>.md` in the shape `yaks dok types` reads, or merges into an existing one (adds fields, changes a field's type only with `--replace`), never dropping fields or prose.
- **Files:** `src/sources.rs`, `src/schemas.rs`, `SourceAction`/`DokAction` in `src/main.rs`, `Farm` methods; docs: `docs/cli.md` `### source` and `### dok`, the `yaks-research` skill's typed-doks section (which says to write the schema by hand) and its source rules, and the `yaks-upkeep` skill's housekeeping bullets (marking a source, adding a field to a schema).
- **Out of scope:** backdating (yaks-0fa7, another thread).
- **Judge:** `cargo test -p yaks` with CLI tests: a marked source drops out of `yaks check`; a defined kind shows in `yaks dok types`; merging keeps an existing field and the schema's body.
