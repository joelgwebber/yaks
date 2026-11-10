---
id: yaks-bcd7
title: 'Unknown tags: create and update say when a tag is not among the farm''s known tags'
status: shaving
type: feature
priority: 3
created: '2026-11-10T21:11:52Z'
updated: '2026-11-10T21:30:26Z'
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

---
▸ 2026-11-10T21:12:26Z [herd-unlock]

Claim (yakherd herd-unlock-2): worker `tagwarn-1`, Sonnet 5.5.

## Scope

A tag not in the farm's known tags (`schemas/yak.md` `tags:`, via `src/schemas.rs`) gets a warning on stderr from `yaks create --tags`, `yaks update --add-tag` and `--tags`, and from `yaks bulk` setting tags: `warning: tag "foo" is not among this farm's known tags (schemas/yak.md); known: a, b, c` (nearest by edit distance first, at most five). Never refuses, never warns when the farm has no known tags. The TUI form: no warning (out of scope). Docs: `docs/cli.md` (tags), the `yaks` skill's Tags section (one line), clap help for `--tags`.

## Evidence

Integration tests: warning on an unknown tag, none on a known one, none when no schema. Gate `cargo test --workspace`.

---
▸ 2026-11-10T21:14:15Z [tagwarn-1]

Start 2026-11-10T21:14:15Z; checkout /Users/joel/src/yaks/.delta/worktrees/jaghh9sj6kkr/yaks

---
▸ 2026-11-10T21:19:37Z [tagwarn-1]

Decision: one helper, `schemas::unknown_tag_warnings(root, tags)`, called from the create, update and bulk arms in `src/main.rs` via `warn_unknown_tags`. Rejected: putting the warning in `Farm::create`/`update` (the TUI form would then warn too, which the claim puts out of scope, and the library would print).

Decision: `update --tags` does not exist (only `--add-tag`/`--remove-tag`), so only added tags are checked; `--remove-tag` never warns. Rejected: warning on removal (removing an unknown tag is cleanup).

Decision: nearest by Levenshtein distance, stable sort so ties keep the schema's order, at most five; hand-rolled 15-line `edit_distance` rather than a new dependency (Cargo.toml is off limits).

Decision: bulk warns before the dry-run preview as well as on `--commit`, so the preview shows the problem before anyone commits.

Scratch farm run (own binary):

- `create x --tags dox,ui` -> `warning: tag "dox" is not among this farm's known tags (schemas/yak.md); known: docs, ui, cli`, then `Created`
- `create y --tags ui docs` -> silent
- `update --add-tag cly` -> warning, known: cli, ui, docs
- `bulk --tag ui --add-tag wibble` -> warning, then the dry-run preview
- no `schemas/yak.md` -> silent, exit 0

Tests: `tests/tag_warn.rs` (5) + `schemas::edit_distance_counts_edits`. Docs: clap help (`--tags`, `update --add-tag`, `bulk --add-tag`), `docs/cli.md` create section, `yaks` skill Tags line. `cargo test -p yaks skills` passes; `cargo fmt --check` clean; clippy warning count unchanged (104 before and after).

---
▸ 2026-11-10T21:30:26Z [tagwarn-1]
kind: verify
result: pass
---

verify: `cargo test -p yaks` -> PASS (exit 0)
