---
id: yaks-ef3b
title: 'Run-on filter: yaks whose markdown has lines that join into one paragraph'
status: shaving
type: feature
priority: 2
created: '2026-10-26T23:07:10Z'
updated: '2026-10-26T23:07:33Z'
parent: yaks-db07
labels:
- cli
- markdown
---

---
▸ 2026-10-26T23:07:33Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-26T23:07:33Z [delta-lead]

Claim (delta-lead, yakherd). Owner: `runon-1`. Plan: the latest note on yaks-db07.

**Goal:** a shared filter flag `--run-on` that keeps yaks whose description or any note contains a paragraph with a soft line break, i.e. adjacent lines that CommonMark joins into one paragraph.

- **Detection:** split the body with the existing comment splitter (`content::parse` or whatever the TUI renderer uses, so note headers aren't counted). Parse each block with `pulldown-cmark` using the same options as the TUI renderer, and flag a `SoftBreak` event inside a paragraph (not inside a code block or table, which have no soft breaks anyway). Put it in a small pure function (e.g. `markdown_lint::has_run_on(body) -> bool`, or in `store`/`content`, wherever the renderer's parser setup can be reused without the TUI) with unit tests.
- **Filter:** add `run_on: bool` to `FilterSpec` (`src/filter.rs`; fix every construction site, most use `..Default::default()`) and match it in `FilterSpec::matches`. Add `--run-on` to `FilterFlags` and `build_spec` in `src/main.rs`, so `list`, `next`, `path`, `log`, `bulk`, `inbox` and the rest all get it.
- **Docs:** `docs/cli.md` "Filter flags (shared)"; the `--help` doc comment.

**Out of scope:** the TUI filter drawer (don't add a chip), `request`/`bulk` code (`kinds-1` owns the `Request` and `Bulk` variants and arms in `src/main.rs`, and `describe_bulk_mutation`), the skills (delta-lead).

**Evidence:**
- Unit tests that fail first:
  - flagged: a two-line paragraph; a run-on paragraph inside a note but not the description; `a\nb` in a list item's paragraph.
  - not flagged: blank-line paragraphs, lists, fenced code, a hard break (two trailing spaces or a trailing backslash), a table.
- An integration test running `list --run-on`.
- Paste in a note `yaks list --run-on | wc -l` against the live farm, using your OWN build read-only (`./target/debug/yaks list --run-on`), plus 3 example ids. That shows it finds real candidates; the 2026-10-17 survey estimated ~440 run-on lines.

**Gate:** `cargo test -p yaks`. **Judge:** delta-lead at landing.
