---
id: yaks-9af0
title: 'Pages: schema-7 layout, page discovery, yaks page list/show/new/set/query, doctor'
status: shorn
type: feature
priority: 1
created: '2026-11-01T18:53:10Z'
updated: '2026-11-04T20:03:41Z'
parent: yaks-f301
depends_on:
- yaks-1f34
- yaks-37da
tags:
- cli
- doks
verify: cargo test --workspace
---

Implements yaks-aae6 on yaks-1f34. Reserved names `wiki`, `sources`, `schemas`; `wiki/**.md` and `<herd>/wiki/**.md` are pages; `yaks page list|show|new|set|query` with `--json`; doctor checks page frontmatter. Docs, help and skills in the same change.

---
▸ 2026-11-01T20:26:23Z [delta:farm-wiki]

kind: move
from: hairy
to: shaving
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-01T20:34:04Z [delta:farm-wiki]

kind: checkpoint
branch: wiki
commit: 62f4475
---

Done on the `wiki` branch.

## What it does

- [`src/pages.rs`](../src/pages.rs): pages live under `wiki/` and `<herd>/wiki/` at any depth and are addressed by farm-relative path (a leading `/` is accepted). Frontmatter is `Fields`, so it round-trips exactly; the body is untouched. Kind comes from `type:` (default `note`); title from `title:`, else the first `# ` heading, else the file name.
- `yaks page list [prefix]`, `show`, `new`, `set` and `query` (`--type`, `--where key=value`, ANDed; list fields match by membership). Reads take `--json`, and writes hold the farm lock.
- `yaks doctor` gains `page-problem`: no frontmatter, no `type:`/`title:`, a non-kebab name, or YAML yaks can't read. The page path goes in `ids`.
- Docs: a `page` section in `docs/cli.md`, plus doctor. The `yaks` skill has a short "Pages" section on when knowledge goes to a page, a yak, or the code repo's `docs/`.

## Evidence

- On a scratch farm holding a copy of the `joel` repo's real `wiki/` and `schemas/`: `page list` shows all 31 pages with kind and title, including the typed `agent-platform` table page. `page query --type topic --where tags=signals` returns the 13 signal pages. `doctor` is clean.
- Round-trip: `page set --set x_probe=1` then `--unset x_probe` on every one of the 32 real pages leaves each one byte-identical.
- `tests/pages.rs` runs the real binary: new/set/show/list/query, refusals (bad name, outside a wiki root, missing page), and doctor's JSON for a page without frontmatter.
- `cargo test --workspace`: 889 passed.

Not here: schema checks for typed kinds (yaks-8e75), links and refs over pages (yaks-101d), the TUI (yaks-ebb2).

---
▸ 2026-11-01T20:35:11Z [delta:farm-wiki]

kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-01T20:35:12Z [delta:farm-wiki]

kind: move
from: shaving
to: shorn
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
