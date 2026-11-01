---
id: yaks-bdcb
title: 'yaks find: full-text search over pages, sources'' extracted text and yaks'
status: shaving
type: feature
priority: 2
created: '2026-11-01T22:27:16Z'
updated: '2026-11-01T23:20:51Z'
parent: yaks-d26a
labels:
- wiki
- cli
---

Hearth has wiki_search and source_search; yaks search only matches yak id/title/description. Add one search over page bodies and frontmatter, sources' extracted.md and source.md, and yak bodies (notes included), returning path plus matching line, with --scope (wiki/, sources/, a herd) and --json. Plain substring/regex scan of the files first (farms are small, see yaks weight); an index only if measured slow, and then a per-user rebuildable cache per AGENTS.md. Found while writing the research skill (yaks-577f).

---
▸ 2026-11-01T23:20:50Z [delta:farm-wiki]
kind: move
from: hairy
to: shaving
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-01T23:20:51Z [delta:farm-wiki]

Claim (yakherd delta:farm-wiki): worker `find-1`, on branch `wiki` at 844bf53.

**Task.** `yaks find <pattern> [--regex] [--scope <prefix>]... [--kind yaks|pages|sources|sessions] [--json] [-i]` is one search over the farm's text.

It searches:

- yak files (title, description, notes);
- pages (frontmatter and body);
- sources (`source.md` and `extracted.md`, never binary originals);
- `sessions/**.md`.

The match is case-insensitive by default; it matches a literal substring, or a regex with `--regex`, using a hand-rolled matcher or the `regex` crate only if it is already in `Cargo.lock` (check; no new dependencies). Output is one line per match: `path:line: text`, trimmed to about 160 characters, grouped by file, with the yak's title or the page's title on the group header. `--json` gives `[{path, kind, title, line, text}]`.

Scan files directly, with no index. Measure the time on a scratch import of `joel` (161 files) and on this farm's size (about 600 yaks), and record both in a note. Exit 1 when there is no match, like `grep`.

Also reserve `sessions` as a herd name: add it to `store::RESERVED_HERD_NAMES`. Update the error messages, docs and test that list the reserved names (grep `RESERVED_HERD_NAMES` and `"schemas"`).

**Scope.**

- New `src/find.rs`.
- A `Find` variant plus its dispatch arm in `src/main.rs`; put it next to `Search`.
- `store::RESERVED_HERD_NAMES`.
- Docs: `docs/cli.md`, the `yaks` skill command table, `README.md`, and the `yaks-research` skill's "Find before you write" section, replacing its `grep -ril` line with `yaks find`.
- Tests: `tests/find.rs`.

**Out of scope.** Changing `yaks search`.

**Evidence.**

- `tests/find.rs` covers each kind, `--scope`, `--json`, exit codes, and that a binary file is skipped.
- Paste a real run on a scratch farm into a note.
- The full gate passes.

**Judge.** The coordinator, at landing.
