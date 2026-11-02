---
id: yaks-37da
title: 'Schema 7: labels become tags; wiki, sources and schemas reserved'
status: shorn
type: feature
priority: 1
created: '2026-11-01T20:09:15Z'
updated: '2026-11-01T20:26:23Z'
parent: yaks-f301
tags:
- wiki
- schema
verify: cargo test --workspace
---

Implements the schema-7 format change decided on yaks-aae6 and yaks-8384, in one migration so farms migrate once:

- `labels:` becomes `tags:` in every yak; `Task.labels` becomes `tags`; CLI `--tags`/`--tag`/`--add-tag`/`--remove-tag` with the `--label*` forms as hidden aliases; JSON key `tags`; TUI and docs say tags.
- `wiki`, `sources` and `schemas` become reserved herd names.
- `SCHEMA = 7`; `yaks migrate` rewrites the key (a fixup), refusing a farm with a herd named like a reserved dir.

---
▸ 2026-11-01T20:09:21Z [delta:farm-wiki]

kind: move
from: hairy
to: shaving
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-01T20:25:12Z [delta:farm-wiki]

kind: checkpoint
branch: wiki
commit: ba4245d
---

Done on the `wiki` branch (`ba4245d`, rebased onto `main` v0.0.28 first).

## What changed

- `labels:` is now `tags:` everywhere: `Task.tags`, JSON `tags`, `show` prints `tags:`, the TUI says Tags (detail row, `L` editor, create form, filter drawer, help).
- CLI: `--tags`, `--tag`, `--add-tag`, `--remove-tag`. The `--label*` spellings stay as hidden aliases; one CLI test deliberately still uses them.
- The reader accepts `labels:` from an unmigrated file, and saved TUI views read either key, so nothing breaks before a farm migrates.
- `SCHEMA = 7`. `yaks migrate` gains `labels-become-tags`, a frontmatter fixup: it renames only the top-level key (value untouched, nested `labels:` left alone, idempotent). It refuses a farm whose herd is named `wiki`, `sources` or `schemas`, which are now reserved alongside `docs`.
- Docs (`docs/cli.md`, `docs/tui.md`, `docs/README.md` layout tree, `README.md`) and skills (`yaks`, `yaks-tracker`, `land`) say tags.

## Evidence

- `cargo test --workspace`: 883 passed, 0 failed. 12 TUI insta snapshots changed, and only in wording (`Labels:` became `Tags:`, `labels` became `tags`).
- Real migration on copies of all three farms with the release build:
  - this farm: 557 keys in 557 yaks, plus 117 old-shape comment headers fixed; doctor clean.
  - the `joel` farm: 486 keys; doctor's findings are the same as `main`'s, plus one new "finished over unfinished children" finding that comes from `main`'s yaks-35a4 check, not this change.
  - the Hearth farm: 91 keys; doctor clean.
  - After migrating, no `labels:` key is left in any of them.

## Not migrated, on purpose

The shared `yaks` farm and the installed binary stay on schema 6 until the `wiki` branch squash-merges: a schema-7 farm would be refused by the binary released from `main`. Coordination keeps using the installed `yaks`; the `wiki` binary runs only on copies and scratch farms.

---
▸ 2026-11-01T20:26:18Z [delta:farm-wiki]

kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-01T20:26:23Z [delta:farm-wiki]

kind: move
from: shaving
to: shorn
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
