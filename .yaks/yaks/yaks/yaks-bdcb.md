---
id: yaks-bdcb
title: 'yaks find: full-text search over pages, sources'' extracted text and yaks'
status: hairy
type: feature
priority: 2
created: '2026-11-01T22:27:16Z'
updated: '2026-11-01T22:27:16Z'
parent: yaks-d26a
labels:
- wiki
- cli
---

Hearth has wiki_search and source_search; yaks search only matches yak id/title/description. Add one search over page bodies and frontmatter, sources' extracted.md and source.md, and yak bodies (notes included), returning path plus matching line, with --scope (wiki/, sources/, a herd) and --json. Plain substring/regex scan of the files first (farms are small, see yaks weight); an index only if measured slow, and then a per-user rebuildable cache per AGENTS.md. Found while writing the research skill (yaks-577f).
