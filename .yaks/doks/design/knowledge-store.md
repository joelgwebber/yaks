---
type: topic
title: The farm as a knowledge store
description: How yaks, pages, sources and schemas fit together, and why
tags:
- wiki
- design
---

# The farm as a knowledge store

A farm holds working knowledge next to the work: **yaks** (what to do), **pages** (what we know), **sources** (what we read) and **schemas** (what shape typed facts take). It is all markdown with frontmatter on the farm branch, and every write is its own commit. Built under [yaks-44b3](yaks-44b3) in November 2026; this page records the model and why it is shaped this way.

## The pieces

| Piece | Where | Written by |
|---|---|---|
| Yaks | `<herd>/yaks/<id>.md` | `yaks create`, `update`, the state moves |
| Pages | `wiki/**.md`, `<herd>/wiki/**.md` | `yaks page new`, `page set`, the TUI's Wiki tab, or any editor |
| Sources | `sources/<date>-<sha8>/` | `yaks source add`, `yaks promote` |
| Schemas | `schemas/<type>.md`, `schemas/yak.md` | by hand |
| Sessions | `sessions/**.md` | imported from Hearth (`yaks import-hearth`) |

Product documentation that ships with the code stays in `docs/` on `main` ([code:docs/README.md](code:docs/README.md)). The farm is the working knowledge behind it, so the reserved name is `wiki/`, never `docs/`.

## Rules that hold it together

- **Files are the truth.** Frontmatter round-trips byte for byte ([yaks-1f34](yaks-1f34)), so a hand edit, Hearth, and yaks can share a file.
- **One link grammar.** A yak id, `/wiki/x.md#heading` from the farm root, `<id>/<file>` for an attachment, and `code:path@sha#L10-L20` for code. `@sha` pins any of them. `yaks peek` reads just what a link names, and doctor reports links that lead nowhere ([yaks-101d](yaks-101d)).
- **Every write commits** with a `Yaks-Actor` trailer; hand edits are committed separately; `yaks undo --by <actor>` reverts a session ([yaks-a838](yaks-a838)).
- **Merges never break fields.** The merge driver merges frontmatter field by field and prose line by line. Where both sides changed the same lines, it keeps both between markers for a person or agent to settle; `yaks conflicts` lists them ([yaks-4c8c](yaks-4c8c)).
- **Schemas advise, never refuse** ([yaks-8e75](yaks-8e75)), as in Hearth.
- **No model inside yaks.** yaks fetches, converts and schedules nothing: an agent harness (Hearth, later) owns channels, the model loop, the clock and trust policy, and drives yaks through `--json`.

## Weight

Every clone of a code repository fetches the farm branch too. `yaks weight` measures that, and `yaks lfs` stores binary originals in Git LFS (on for this farm since the schema-7 migration). A farm heavy with sources belongs in its own repository (`yaks init --farm`). Measured on this farm before LFS: 5.5 MiB per clone, 6.6 MiB of it attachments. Per-change commits cost about 650 bytes each ([yaks-2b68](yaks-2b68)).

## Open questions

- [x] Range comparisons in `dok query --where` (dates, numbers), as Hearth's `page_query` has: `~`, `<`, `>=`, `!` and missing, in yaks-39f9.
- [ ] Fewer git calls per write: 49 against 22 without per-change commits ([yaks-7058](yaks-7058)).
- [ ] When Hearth moves onto a yak herd, which of its jobs (nightly check, session review) become yaks commands and which stay in the harness. Being worked out under yaks-1c89.
