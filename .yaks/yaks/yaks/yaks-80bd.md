---
id: yaks-80bd
title: 'Small tool gaps: mark sources, write schemas, backdate imports'
status: shaving
type: task
priority: 3
created: '2026-11-06T04:28:32Z'
updated: '2026-11-06T23:46:31Z'
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

---
▸ 2026-11-06T23:14:54Z [tools-1]

Start: Fri Nov 6 23:14:51 UTC 2026

- yak path: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-80bd.md
- checkout: /Users/joel/src/yaks/.delta/worktrees/rxppcdpad50n/yaks

---
▸ 2026-11-06T23:29:57Z [tools-1]

Decision: `source mark` keeps ONE mark: setting `--uncited` removes `superseded_by` and vice versa; `--clear` removes both. Rejected: allowing both fields at once (two reasons for one state, `check` treats them the same).

Decision: `dok define` merges by field name. A field that exists with a different type OR required flag is refused (all conflicts listed, nothing written) unless `--replace`; `--replace` sets type and required and keeps `default`/`description`. Rejected: treating a missing `!` as "leave required alone" (ambiguous restating) and silently overwriting.

Decision: merge preserves the file byte for byte except what changed: new fields are appended as lines under an existing block `fields:`; only a `--replace` change or a flow-style `fields: {..}` re-renders the whole `fields:` entry (comments inside the `fields:` block would then be lost). Title/description/stale_after are written only when given (given-and-different overwrites).

Decision: `dok define` refuses the untyped kinds (note, topic, howto, summary), an invalid kind name, an unknown type, a duplicate `--field`, a bad `--stale-after` (must be a duration `recur::parse_duration` reads, or YYYY-MM-DD), and a call with nothing to define. `default:`/`description:` of a field have no flag (not in the claim); docs say edit by hand.

Decision: `source mark` accepts the bare id, `sources/<id>` or `sources/<id>/source.md` (with optional leading `/`) for both the source and `--superseded-by`; stores `superseded_by: sources/<id>/source.md` (the form check and the skills use). `--superseded-by` naming the source itself is refused.

---
▸ 2026-11-06T23:31:47Z [tools-1]
kind: verify
result: fail
---

verify: `cargo test -p yaks` -> FAIL (exit 101)

---
▸ 2026-11-06T23:33:23Z [tools-1]
kind: verify
result: fail
---

verify: `cargo test -p yaks` -> FAIL (exit 101)

---
▸ 2026-11-06T23:39:50Z [tools-1]
kind: verify
result: pass
---

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-11-06T23:46:19Z [tools-1]
kind: verify
result: pass
---

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-11-06T23:46:31Z [tools-1]

Summary (tools-1): added `yaks source mark` and `yaks dok define`.

- `src/sources.rs` `Mark`/`mark`, `src/schemas.rs` `FieldSpec`/`Define`/`define`, `Farm::source_mark`/`dok_define` (farm lock, so one autocommit each), clap `SourceAction::Mark` and `DokAction::Define` with help.
- Tests: `tests/check.rs` (marked source drops out of `check`, clear brings it back, refusals leave the file alone), `tests/schemas.rs` (`define` shows in `dok types`; merge keeps field, comment and prose byte for byte; type change refused until `--replace`; bad input refused).
- Docs: `docs/cli.md` (table rows, `### dok`, `### source`, check's uncited bullet), `README.md` rows, skills `yaks`, `yaks-research`, `yaks-upkeep`; `yaks check`'s uncited heading and `check --help` now name `yaks source mark`.
- Gate: `yaks verify yaks-80bd` PASS. An earlier two runs failed only in `tui::farmsync::tests` (the 10 s "interval sync did not finish" wait, under load from parallel workers); those 8 tests pass alone and the full `cargo test` passed before and after.
