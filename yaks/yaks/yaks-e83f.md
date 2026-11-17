---
id: yaks-e83f
title: 'Design: one file holds a source''s text and its provenance (retire the duplicate extracted.md)'
status: hairy
type: idea
priority: 2
created: '2026-11-17T21:16:47Z'
updated: '2026-11-17T21:16:47Z'
parent: yaks-4cfe
tags:
- doks
- design
---

Part of yaks-4cfe. A source keeps provenance in `source.md` and its text in `extracted.md`, and for a markdown or text original it also keeps the original, byte-identical to `extracted.md` (all 14 sources saved here on 2026-11-16). Doks cite `/sources/<id>/extracted.md#heading`, which has no frontmatter and no link back, so a reader who follows a citation can't see where the text came from.

## Why it is this way

- yaks-dedc's diagnosis kept the duplicate on purpose: search and the TUI read `extracted.md`, and the two-file layout is Hearth's format, so a Hearth `raw/sources/` migrates as is.
- yaks-0df0 added `--text-only` (one file, `file: extracted.md`) but left it opt-in, and explicitly left "collapse existing pairs" as a separate format decision. Agents mostly don't pass it.

## Options

- **A. Keep the layout and patch it.** Make `--text-only` the default for a markdown or text original. Prepend a one-line back-link to every `extracted.md`: `> From [<title>](source.md) · <url>`. Cost: `extracted.md` is no longer exactly the text. A text-only source's `sha256` would have to skip the header. Every reader of the text gets a line it didn't write.
- **B. `source.md` is the source.** Its frontmatter is the provenance and its body is the text. `extracted.md` is retired. The original sits beside it only when it is not the text: HTML, PDF, JSON, an image. Citations become `/sources/<id>#heading`, and `/sources/<id>` already resolves to `source.md`. One page shows the provenance and the text, in the TUI and on GitHub. Cost: a format change. It needs a migration that merges each pair and rewrites `.../extracted.md` links (the `yaks mv` link rewrite exists), plus a reader for the old layout and Hearth's. `sha256` stays the hash of the original, or of the text for a text-only source.
- **C. A TUI-only fix.** Opening an `extracted.md` shows its `source.md` header above it. Nothing changes on disk, and GitHub still has no way back.

**Lean: B.** It leaves one file to cite, read, search and diff, and the back-link problem goes away instead of being patched. A keeps two files that can drift and a header that isn't part of the text. C fixes only one of the three readers.

## Done when

- A new source writes one text-bearing file. Existing sources are migrated, and their links follow.
- `find`, the TUI viewer, `peek`, `check`'s citation count and the `yaks-research` skill all read the new layout.
- Docs, `--help` and skills say it in the same change.
