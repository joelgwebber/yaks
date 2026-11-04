---
id: yaks-101d
title: 'Links: one resolver for yaks, farm paths, code: and @rev pins; refs and doctor over pages; yaks peek'
status: shorn
type: feature
priority: 1
created: '2026-11-01T18:53:11Z'
updated: '2026-11-04T20:03:30Z'
parent: yaks-f301
depends_on:
- yaks-9af0
tags:
- links
- doks
verify: cargo test --workspace
---

Implements yaks-ccb1. A shared resolver used by refs, doctor, peek and the TUI; old `../src` and `../blob/<sha>` forms stay valid; `yaks refs` accepts a page path; `yaks peek <link>` prints the span; `yaks mv` moves a page and rewrites links to it.

---
▸ 2026-11-01T20:35:16Z [delta:farm-wiki]

kind: move
from: hairy
to: shaving
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-01T20:50:13Z [delta:farm-wiki]

kind: checkpoint
branch: wiki
commit: 99416ed
---

Done on the `wiki` branch (`99416ed`).

## What it does

- [`src/links.rs`](code:src/links.rs) is the one link grammar:
  - `/path` from the farm root; relative links go downward only (Joel's answer on yaks-ccb1).
  - `<id>/<file>` attachments resolve from anywhere.
  - `code:path` for the code repo, and `@<sha>` pins any path.
  - Anchors are `#L12-L40` lines or a `#heading` slug (GitHub's rule, with `-1` for repeats).
  - The legacy `../src` and `../blob/<sha>` forms stay valid in yaks; any other `../` is `Upward`.
- The TUI's source-link parser and code-checkout lookup now call into it: one rule instead of two copies.
- New commands:
  - `yaks peek <link> [--context N]`: prints lines, a section, or a page's frontmatter and first paragraph; pinned reads go through `git show`.
  - `yaks refs <page> [--incoming] [--json]`.
  - `yaks mv <from> <to> [--dry-run]`: rewrites every link to the page in the farm-root form, keeping `@rev` and `#anchor`, and also rewrites the moved page's own relative links.
  - `doctor` gains `broken-link`. Placeholder links in prose (`[x](path)`, `{id}/{name}`) are skipped.
- Docs: a Links section in `docs/cli.md`, the source-link paragraph in `docs/tui.md`, and the skill's link rules (now teaching `code:` and `peek`).

## Evidence

- Scratch farm with a code repo, two pages and a yak:
  - `peek '/wiki/topics/links.md#grammar'` prints just that section.
  - `peek 'code:links.rs#L84-L86' --context 1` prints numbered lines 83-87; the pinned `@<sha>` form reads the commit.
  - `refs --incoming` lists the yak and the other page, with lines.
  - `mv` rewrote three documents, and doctor is clean afterwards.
- `doctor` on migrated copies of the real farms finds only genuine problems: this farm, `yaks-8222`'s missing pasted PNG; Hearth, `hearth-c858`'s missing test log; `joel`, `fullstory-14dd`'s `../../sightmap/...` link out of the repo.
- `tests/links.rs` covers peek/refs/mv and doctor's broken and upward links through the binary; `links::tests` cover grammar, slugs, sections, extraction and peek. `cargo test --workspace`: 896 passed.

Not here: the TUI following page links and a peek preview (yaks-7395), link checks over sources (yaks-2c42/yaks-c0ac).

---
▸ 2026-11-01T20:51:28Z [delta:farm-wiki]

kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-01T20:51:28Z [delta:farm-wiki]

kind: move
from: shaving
to: shorn
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
