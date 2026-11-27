---
type: topic
title: The farm as a knowledge store
description: What the farm holds (yaks, doks, sources, schemas, session notes), the rules that keep it coherent, and why
tags:
- doks
- design
stale_after: P6M
verified: 2026-11-10
---

# The farm as a knowledge store

A farm holds working knowledge next to the work: **yaks** (what to do), **doks** (what we know), **sources** (what we read), **schemas** (what shape typed facts take) and **session notes** (how the work went). It is all markdown with frontmatter on the farm branch, and every write is its own commit. Built under [yaks-44b3](/yaks/yaks/yaks-44b3.md) in November 2026 as a home for Hearth's knowledge procedures; this dok records the model and why it is shaped this way.

> [!NOTE]
> Refreshed 2026-11-10 ([yaks-464b](/yaks/yaks/yaks-464b.md)). The first version described *pages* under `wiki/`, written by `yaks page new` and shown in a Wiki tab, and sessions imported by `yaks import-hearth`. Pages became **doks** under `doks/` (`yaks dok`, the TUI's Doks tab); session notes are now written by `yaks session note` at the end of each session, and import-hearth was removed ([yaks-ff4c](/yaks/yaks/yaks-ff4c.md)). Everything below describes the farm as it is now.

## The pieces

| Piece | Where | Written by |
|---|---|---|
| Yaks | `<herd>/yaks/<id>.md`, attachments in `<id>/` | `yaks create`, `update`, the state moves |
| Doks | `doks/**.md`, `<herd>/doks/**.md`, attachments in the sibling directory | `yaks dok new`, `dok set`, `dok row`, the TUI's Doks tab, or any editor |
| Sources | `sources/<date>-<sha8>-<slug>.md` (provenance and text), any original or tree beside it in `sources/<date>-<sha8>-<slug>/` (schema 10, yaks-e83f) | `yaks source add`, `yaks promote` |
| Schemas | `schemas/<type>.md`, `schemas/yak.md` (yak fields and known tags) | `yaks dok define`, or by hand |
| Session notes | `sessions/YYYY/MM/<actor>-<day>-<hhmm>-<slug>.md` | `yaks session note` |

Product documentation that ships with the code stays in `docs/` on `main` ([code:docs/README.md](code:docs/README.md)). The farm is the working knowledge behind it, so the farm's folder is `doks/`, never `docs/`.

## Rules that hold it together

- **Files are the truth.** Frontmatter round-trips byte for byte ([yaks-1f34](/yaks/yaks/yaks-1f34.md)), so a hand edit, a harness and yaks can share a file.
- **One link grammar.** A yak id (and `id#<timestamp>` for one note), `/doks/x.md#heading` from the farm root, `<id>/<file>` for an attachment, and `code:path@sha#L10-L20` for code. `@sha` pins any of them. `yaks peek` reads just what a link names, `yaks refs --incoming` lists what links a yak, and `yaks doctor` reports links that lead nowhere ([yaks-101d](/yaks/yaks/yaks-101d.md)).
- **Every write commits** with a `Yaks-Actor` trailer; hand edits are committed separately (`yaks commit --as`); `yaks undo --by <actor>` reverts a session ([yaks-a838](/yaks/yaks/yaks-a838.md)).
- **Merges never break fields.** The merge driver merges frontmatter field by field and prose line by line. Where both sides changed the same lines, it keeps both between markers for a person or agent to settle; `yaks conflicts` lists them ([yaks-4c8c](/yaks/yaks/yaks-4c8c.md)). Concurrent edits of one dok can also be hash-checked (`dok set --if-hash`).
- **Schemas advise, never refuse** ([yaks-8e75](/yaks/yaks/yaks-8e75.md)).
- **No model inside yaks.** yaks fetches, converts and schedules nothing on its own: an agent harness owns channels, the model loop, the clock and trust policy, and drives yaks through `--json`. Judgement lives in skills (`yaks-research`, `yaks-upkeep`); what can be decided from the files is a command (`yaks check`, `session list --unreviewed`).

## Keeping it current

The upkeep loop is the `yaks-upkeep` skill: a session note at the end of each session; a weekly review that moves what session notes and yak notes learned into doks; a weekly reflection that keeps [/doks/lessons.md](/doks/lessons.md); a weekly tidy over `yaks check` (stale doks, uncited sources, missing descriptions, broken links). Each comes back as a recurring yak. Design: [/doks/design/session-reflection.md](/doks/design/session-reflection.md).

## Weight

Every clone of a code repository fetches the farm branch too. `yaks weight` measures that, and `yaks lfs` stores binary originals in Git LFS (on for this farm since the schema-7 migration). A farm heavy with sources belongs in its own repository (`yaks init --farm`). Measured on this farm before LFS: 5.5 MiB per clone, 6.6 MiB of it attachments. Per-change commits cost about 650 bytes each ([yaks-2b68](/yaks/yaks/yaks-2b68.md)).

## Open questions

- [x] Range comparisons in `dok query --where` (dates, numbers): `~`, `<`, `>=`, `!` and missing, in yaks-39f9.
- [x] Fewer git calls per write ([yaks-7058](/yaks/yaks/yaks-7058.md)).
- [x] Which of Hearth's jobs (nightly check, session review, reflection) become yaks commands: `yaks check`, `yaks session`, and the `yaks-upkeep` skill (yaks-1c89).
- [ ] A `yaks check` finding for doks whose cited code changed since the pin ([yaks-6cf2](/yaks/yaks/yaks-6cf2.md)): the drift this dok itself suffered.

> [!NOTE]
> Being revisited by [the one document model](/doks/design/one-document-model.md) (yaks-7e32, 2026-11-26):  echo 'every kind built from one set of constructs, schemas for every kind, a derived index; the rules here all hold.';; farm-layout-and-links) echo 'decision 6 (sharding) is reversed: shard by id, cascading with id length; a reference becomes a markdown link that resolves.';; note-format) echo 'the note log becomes a capability any kind can switch on, with this parser.';; files-directories-archives) echo 'a subject named by its link destination becomes the identity of every row and pane.';; tui-model) echo 'decisions 3 and 7 (All unpinned, Recent yaks-only) are reopened; one row type and pane shell.';; esac)
