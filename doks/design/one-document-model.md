---
type: topic
title: 'One document model: yaks, doks, sessions and sources on one set of constructs'
created: '2026-11-27T02:09:22Z'
description: 'Design for building every farm document (yak, dok, session, source, release) from one set of constructs: fields, schemas, links, notes, attachments, queries and an index; read before changing how any of them is parsed, linked, queried, searched, shown or laid out'
tags:
- design
- doks
- links
---

# One document model: yaks, doks, sessions and sources on one set of constructs

Status: **in design** with Joel, started 2026-11-26. Tracked by [yaks-7e32](/yaks/yaks/yaks-7e32.md). This dok records the decisions as they settle; the open ones are at the end and asked on the umbrella yak.

Read this before changing how any farm document is parsed, written, linked, queried, searched, shown or laid out on disk.

## The finding in one paragraph

Yaks, doks, sessions, sources and release doks have converged: each is markdown with frontmatter, with attachments in a sibling directory, linked by one grammar. But the code still builds them from two sets of parts, and every construct that exists twice behaves differently for each kind and has its own bugs. The fix is **one set of constructs** (fields and schemas, fields with behaviour, links, notes, attachments, queries and an index) that every kind is built from, with kinds registered in Rust and differing only in which capabilities they switch on. It is *not* one generic database: the opinionated views and commands stay, built on shared parts.

## 1. Why: what exists twice today

Code inventory (a read-only scout, 2026-11-26) and a construct × kind audit run in scratch farms. Line numbers are at `origin/main` `7c1452bd`.

| Concept | Yak | Dok / session / source |
|---|---|---|
| Frontmatter parse and write | [`parse_task`](https://github.com/joelgwebber/yaks/blob/7c1452bd5712911a15ba5e244c9079af914a5c76/src/store.rs#L781-L890), a fixed-order writer with its own quoting (`needs_quotes`) | [`Fields`](https://github.com/joelgwebber/yaks/blob/7c1452bd5712911a15ba5e244c9079af914a5c76/src/fields.rs), a different quoting policy (`plain_safe`), key order kept |
| Filtering | [`FilterSpec`](https://github.com/joelgwebber/yaks/blob/7c1452bd5712911a15ba5e244c9079af914a5c76/src/filter.rs#L15-L41): typed fields and relations (ready, parent-of) | `dok query --where` operators ([doks.rs](https://github.com/joelgwebber/yaks/blob/7c1452bd5712911a15ba5e244c9079af914a5c76/src/doks.rs#L415-L481)); neither can express the other |
| Stale-write guard | `TaskEdit.base`, TUI only | `--if-hash` |
| Link rewriting | `rename` via `refs::rewrite` | `mv`, `promote`, `rename-attachment`: four routines, each over a different subset of files |
| Tags | normalized, warned against the known tags | raw values, no checks |
| References | `refs.rs`: bare id tokens | `links.rs`: markdown links |
| TUI field rows | `detail::build_at` | `frontmatter_rows` |
| TUI tags | `[a, b]`, `Tags: a, b` | `#a #b`, `tags: a, b` |
| TUI row identity | `Item`, `RowKind`, `RowKey`, `Star`, `PaneAt`, `DocKind`, each split Yak / Doc | |
| JSON | `show --json`: flat keys, `extra`, no `path` | `dok show --json`: `path, hash, fields, body`; `source list`: `dir`; `find`: `kind: "yaks"` |
| Body flag | `--description` | `--body`, `--text` (a file on `source add`) |

Already shared, and the base to build on: `Fields`, the link grammar (`links::Link`), the resolver (`locate::Place`), `links::edges`, the field-merge rule, the farm lock, per-change commits, `store_attachments`, `entries` (one lister). Sessions and sources are already `Dok`s in code.

### Construct × kind, as it behaves today

| Construct | yak | dok | session | source |
|---|---|---|---|---|
| Write fields | `--set`, built-ins refused | `dok set`, anything accepted, built-ins too | missing | title and text only |
| Schema check | `schemas/yak.md` extras | `schemas/<type>.md` | missing (schema listed, never applied) | missing |
| `due` / `every` / `needs` / `parent` / `depends_on` | work | stored, inert | n/a | n/a |
| `stale_after` | ignored, doctor calls it unknown | works | n/a | n/a |
| Links checked by doctor | yes | yes | yes | **no** |
| Relative link base | farm root | the dok's directory, downward | the session's directory | n/a |
| Rewritten by `rename` | yes | **no** | **no** | **no** |
| Rewritten by `mv` | yes | yes | **no** | n/a |
| Notes | yes | refused | own `##` sections | no |
| Merge driver | fields, notes unioned | fields, body as text | same | same |
| Attach | any file | `.md` refused | `.md` refused | refused |
| `attachment` / `rename-attachment` / `promote` | yes | missing | missing | missing |
| Query | `FilterSpec` | `dok query` | missing | missing |

### Bugs the duplication caused (all reproduced)

- **The canonical yak link is invisible to references.** Schema 9 stores a yak link as `/<herd>/yaks/<id>.md`, and yaks-ca59 made every writer canonicalise to it. `refs::scan` reads `zz-a8f6.md` as one token, so `refs`, `refs --incoming`, the TUI backrefs and `rename` lose every such link. Filed as [yaks-3b08](/yaks/yaks/yaks-3b08.md); `main` must not be pushed before it is fixed.
- **`rename` and `rename-herd`** rewrite only yak titles and bodies, miss path links, and `rename-herd` leaves the `/<herd>/` segment and the herd's doks behind. `rename-attachment` turns a root path into a relative one. `mv` and `promote` skip sessions. All in [yaks-3b08](/yaks/yaks/yaks-3b08.md).
- **`update --description` deletes the notes** (the CLI passes the whole body; the TUI uses `store::with_description`, which keeps them): [yaks-2c10](/yaks/yaks/yaks-2c10.md).
- **Two machines can mint the same id**, and the merge driver then merges two different yaks into one file (empty base, [mergedriver.rs](https://github.com/joelgwebber/yaks/blob/7c1452bd5712911a15ba5e244c9079af914a5c76/src/mergedriver.rs#L263-L285)): [yaks-0b10](/yaks/yaks/yaks-0b10.md).
- Smaller ones: special fields on a dok are silently inert; `--set due=garbage` gets past validation; dok tags are not split (`tags=cli,bar` is one scalar); `similar-kinds` and `dok types` disagree; `/sources/<id>` without `.md` is not counted as a citation.

## 2. Principles

- **Few constructs, used many ways.** As due dates, recurrence and `needs` turned out to be a small machine that expresses many workflows, and release doks needed no new construct, every construct should work the same on every kind that has it.
- **Kinds are registered in Rust**, each with its path rule, identity rule, capabilities and default views: yak, dok, session, source, release. Schemas add *fields* only. A farm cannot give a typed dok a lifecycle through config: that is the step to a generic CRUD app.
- **A capability is either on or refused.** A field with behaviour on a kind without that capability is a warning, never silently inert.
- **One implementation per construct.** A new kind of document switches capabilities on; it does not add code paths to every command.
- **Files stay the truth; any index is a derived per-user cache** (AGENTS.md).

## 3. Identity and hierarchy

Decided 2026-11-26.

- **A yak keeps a stable flat id and a path that never moves.** Nesting yak files under their parent (`yaks-abcd/1/2/3.md`) was considered and rejected. Every reason dotted ids were dropped returns with it: a reparent cascades over the subtree's files, attachments and links, non-atomically (yaklib `reparent.py`, `yak-3fd4.6`); and yaks-8b7a's rule, *no frequent operation moves a file*, which kept modify/delete conflicts, `git add -u` misses and `--follow` gaps out. Reparents also happen at the worst moment, while workers are still noting the children. And a yak's parent is a work breakdown that changes with understanding, while a dok's folder is a classification: one layout would force one meaning on the other.
- **A document's address is its farm path.** A yak id is a short alias that resolves to its path; `Place::Yak` folds into `Place::FarmFile`.
- **One abstract parent relation.** A yak's parent is its `parent:` field; a dok's is its directory. The tree, the family scope and "move under X" are written once; for a yak the move edits a field, for a dok it moves a file and rewrites links through the one rewrite engine (section 5).

## 4. Ids and sharding

Decided 2026-11-26 (growth rule, sharding, collision repair); details settled in the thread.

**Ids grow with the herd.** [`generate_id`](https://github.com/joelgwebber/yaks/blob/7c1452bd5712911a15ba5e244c9079af914a5c76/src/store.rs#L1583-L1594) draws 16 random bits and retries on a local clash. Readers already accept any length (`has_ref_shape`, resolution by existence), so only the generator changes: the smallest length $L \ge 4$ hex digits that keeps the herd at most 1/16 full. That is 4 digits to 4,096 yaks, 5 to 65,536, 6 to about a million. No self-delimiting (UTF-8-style) encoding is needed: an id is always delimited by its file name or the text around it, and distinct strings never collide.

**Collisions are repaired at sync.** Each machine checks only its own ids, so two machines minting $k$ yaks each between syncs collide with probability about $k^2/16^L$ (3.8% at 50 each with 4 digits). `yaks sync` fetches, then, before rebasing, compares the yak files each side *added* since the merge base. The same path with a different `created:` is two yaks: the local one is renamed through the rewrite engine (with every local reference) in its own commit, and the rebase proceeds. As a safety net for a hand `git pull`, the merge driver refuses to merge two added files whose `created:` differ and leaves a conflict rather than one merged yak.

**Sharding by id, cascading with length.** A yak lives at `<herd>/yaks/<s1>/[<s2>/…]<id>.md`, where the shards are successive pairs of the id's hex tail and the depth is $\lfloor (L-2)/2 \rfloor$: one level for 4- and 5-digit ids, two for 6 and 7, three for 8. The file keeps the full id as its name, and its attachments stay beside it.

- The path is a function of the id (`store::task_path`), so nothing is ambiguous, and a yak's path never changes for life.
- Shard directories exist only when they hold files (git stores no empty directory): about 247 of 256 at today's 844 yaks.
- Every listing stays under GitHub's 1,000-entry cap at every size the growth rule allows: a first-level shard holds at most about 256 four- and five-digit yaks with their attachment directories, plus at most 256 subdirectories.
- Cost: a small herd gets one directory per yak or so; a herd mixing lengths has files and subdirectories side by side in a shard.
- It is due now: `yaks/yaks/` holds 1,022 entries ([yaks-0f3a](/yaks/yaks/yaks-0f3a.md)).

Rejected: one hex digit (16 shards pass the cap near 13k yaks and would need a second link migration); no sharding (the web listing truncates today); nesting by parent (section 3).

## 5. Links and references

**One reference model: a reference is a markdown link that resolves.** Proposed 2026-11-27 (Joel's direction, below); to confirm.

Today two models disagree. Yak bodies are scanned for bare id tokens (`refs.rs`); doks and sessions for markdown links (`links.rs`). The farm holds 892 path links (852 in doks and sessions) and **4,866 prose mentions** (`yaks-1a2b` with no link), which the TUI autolinks and GitHub shows as plain text. Tab completion inserts a bare mention.

- **Writers insert links, not mentions.** Completion (one picker over yaks, doks, sources and sessions, which gives dok-link completion for free) inserts `[yaks-1a2b](/yaks/yaks/1a/yaks-1a2b.md)`; agents may write the short form `[yaks-1a2b](yaks-1a2b)`, which writers already canonicalise on save. Inline links, like every other farm link.
- **No magic on prose.** The TUI stops autolinking bare mentions once the migration has converted the existing ones; a bare mention is then plain text, as on GitHub. `yaks check` may report an unlinked mention of a real id as a hint.
- **`refs.rs` shrinks to text that is not markdown:** commit messages (`release changes`, `commits`), `scan-ids`, and recognising the short form inside a link.
- **One rewrite engine**, `relocate(old, new)`: body links, link-typed fields (`parent`, `depends_on`, `sources`, any schema `link` field) and the short form, across every yak, dok, session, source and markdown attachment, keeping `@rev` and `#anchor`. `rename`, `rename-herd`, `mv`, `promote`, `rename-attachment`, collision repair and the migration all call it.
- **`sources:` entries become links** (`/sources/<id>.md`), checked and rewritten like any other; the `link` field type is actually validated.
- **One relative-link rule** for every document: farm root for `/…`; downward from the document's own directory otherwise; a bare name inside an attachment is a sibling.

Rejected: reference-style links (`[yaks-1a2b]` plus a definition at the end) kept raw text short and render on GitHub (checked with `gh api markdown`), but would make yak links the only links rewritten into a different shape. Converting prose mentions on every save was rejected as hidden magic.

## 6. Fields, schemas and tags

- **One document substrate.** A `Document` is a path, a kind, `Fields` and a body. `Task` becomes a typed view over its `Fields` (as yaks-8384 began), so there is one parser, one writer, one quoting policy and one key order. The fast path for listing a large farm stays (yaks-e61e).
- **Schemas for every kind**, including the built-in yak fields, as documentation and advisory checks; Rust stays the source of truth for fields with behaviour. Session and source schemas are applied to their files.
- **One tag vocabulary for the whole farm**: its own schema file (not part of `yak.md`), each tag with a description agents read, applied to every kind with the same normalization and warnings, and printed by `yaks tags`. One display style everywhere. No per-herd or per-kind tags until a real use needs them ([yaks-5bc2](/yaks/yaks/yaks-5bc2.md) waits).
- **Fields stay flat.** No map type ([yaks-dca5](/yaks/yaks/yaks-dca5.md) retired); a dotted path in a query is an error, not a silent empty result.

## 7. Notes

The note log becomes a capability any kind can switch on, with one parser (`#### <timestamp>` and the italic metadata line, [note-format](/doks/design/note-format.md)) and one merge rule (notes unioned). A design dok can then carry its own review thread. Filed as [yaks-e052](/yaks/yaks/yaks-e052.md); it may fall out of the substrate work nearly free. Note-kind schemas ([yaks-4a0d](/yaks/yaks/yaks-4a0d.md)) become a `notes:` section of a kind's schema. `needs` and asks on doks: an idea collecting evidence, [yaks-190e](/yaks/yaks/yaks-190e.md).

## 8. Queries and search

Today there are about a dozen matchers, all `to_lowercase()` plus substring: none tokenizes, normalizes Unicode or ranks, and every multi-word query is one literal. `search` hides dead yaks while `find` and the TUI show them; only `find` sees frontmatter; `--tag TUI` finds nothing; `find cafe` misses `café`; a phrase across a line break is never found.

Decided 2026-11-27:

- **One matcher** behind `--search`, `find`, the TUI `/`, the pickers and completion.
- **Normalization:** NFC, Unicode case folding, accent folding, with a map back to original offsets for snippets and `path:line`. Off-the-shelf crates (`unicode-normalization`, `unicode-segmentation`), not our own.
- **Tokens:** Unicode word boundaries, identifier-aware: `issues/123` indexes as `issues`, `123` and `issues/123`; `yaks-ca59` as `yaks`, `ca59` and the whole id. A field may specialise its tokenizer later.
- **Query language:** words ANDed, the last one a prefix (as-you-type); `"a phrase"` as consecutive tokens across line breaks; `-word`; field scopes `tag:` `type:` `status:` `kind:` `herd:` `path:` that compile to the same predicates as `--where`. Text search, field filters and relations (ready, tangled, parent-of) are one query tree; yak flags are shorthand for it; `bulk` and `list --kind` take any kind. Matching inside a word goes (`earch` no longer finds `search`). `find --regex` stays, line-based.
- **Ranking:** an exact id or path prefix first, then BM25 with field weights (title, tags, body), recency as the tiebreak.

## 9. The index and modified time

- **One derived per-user cache** ([yaks-8f50](/yaks/yaks/yaks-8f50.md)) serving four needs: positional text postings (phrases), every link edge (backlinks, rewrites, [yaks-bda5](/yaks/yaks/yaks-bda5.md)), parsed field values (queries) and modified times (sorting, staleness). Rebuildable, never committed. Archaeology ([yaks-64c4](/yaks/yaks/yaks-64c4.md)) builds on it afterwards.
- **Modified time comes from git for every kind** (decided 2026-11-27): the last commit touching the file, or the file's mtime while it is uncommitted. Commits marked mechanical by a trailer (migrations, rewrites, bulk renames) are skipped, which also fixes `yaks check` staleness resetting at every layout move. **The yak `updated:` field retires** in the schema bump. This reverses yaks-8384's "updated stays on yaks" and agrees with yaks-c316's reason ("updated goes stale under hand edits"). Filesystem mtime alone is ruled out: a clone resets it.
- Brute-force `find` (no index) takes 0.16–0.2 s on this farm's 3 MB; the index is justified more by backlinks, sorting and as-you-type search than by text search itself. No trigram index: positional postings give phrases.
- **Storage is open** (section 13): SQLite with FTS5, tantivy, or our own postings.

## 10. TUI

Keep the opinionated views and the yak detail pane; share what is under them.

- One row type, keyed by the subject's link destination ([files-directories-archives](/doks/design/files-directories-archives.md)), with shared columns and sort keys.
- One pane shell: a header the kind supplies, the body, then notes when the kind has them. One field-row renderer, one tag style.
- **One picker for opening and linking anything** (Joel, 2026-11-27). Today `#` (`NavGoto`) and yak-ref completion share a yak-only picker (`fuzzy_candidates`), and dok-link completion is a second one (`dok_candidates`) whose prefix tier never matches. One picker over every kind, ranked by the shared matcher (section 8): `#` opens the pick in the pane through the one opener, completion inserts its link (section 5), and add-dependency and reparent use it filtered to yaks. Each row shows its kind. Filed as [yaks-958d](/yaks/yaks/yaks-958d.md).
- The All list mixes every kind by modified time ([yaks-7183](/yaks/yaks/yaks-7183.md)). Merging Recent and All ([yaks-7350](/yaks/yaks/yaks-7350.md)) reverses decisions 3 and 7 of [tui-model](/doks/design/tui-model.md) and is decided with [yaks-6905](/yaks/yaks/yaks-6905.md).

## 11. CLI

Verbs that make sense for any document take any address; `dok` keeps only dok-only commands (`types`, `define`, `row`). `yaks --help` is grouped by function with a help template (clap does not group subcommands itself), and flags get sections: global flags, then shared sets (filter, output, write) declared once with `#[command(flatten)]` and a heading, then the command's own. Old names stay as aliases for a release. A first cut of the groups:

- **Work on a yak:** create, shave, shorn, regrow, slaughter, revive, reparent, dep, snooze, verify
- **Any document:** show, set, mv, attach, refs, peek, path, url
- **Find:** list, next, tangled, find, log, inbox
- **Coordinate:** ask, answer, request, pickup, wait, brief, land, sheds, whoami, tick
- **Knowledge:** dok types / define / row, source, session, promote, check
- **Farm:** init, sync, commit, status, undo, conflicts, doctor, weight, lfs, private, migrate, merge, rename-herd, bulk
- **Hidden:** merge-driver, render, scan-ids

Synonyms to collapse (from the audit): `--description` / `--body` / `--text`; `bulk --commit` (apply) against `update --commit` (a sha) against `--dry-run` / `--yes` elsewhere; `--herd` / `--scope` / `--prefix`; `--json` missing on most writers; `session promote` against `promote`. The `--json` shapes become one contract per document ([yaks-e3b1](/yaks/yaks/yaks-e3b1.md)).

## 12. Migration and order of work

Code refactors land first and change no bytes; everything that changes the format is **one schema bump (11)**, rehearsed on a clone of this farm, applied here in place, and released to other farms later.

- Gate for every refactor step: load and save every file of a copy of this farm, **zero diff**; and a startup bench at 10k yaks (yaks-e61e).
- Schema 11 holds: sharding, `sources:` as links, prose mentions converted to links, the converged writer (quoting, key order), `updated:` retired, the mechanical-commit trailer.

Proposed order:

1. Link fixes and the one rewrite engine; `refs` through `links.rs`; `--description` keeps notes; the id-collision safety net.
2. The document substrate (`Task` as a view over `Fields`).
3. One query tree and one matcher.
4. The kind registry, schemas for every kind, the tag vocabulary.
5. The index.
6. TUI: rows, pane shell, tags, views.
7. CLI restructure, then skills and docs.
8. Schema 11.
9. Later: notes on doks, asks on doks, archaeology.

Existing yaks to pull in (triage, 2026-11-27): [yaks-0f3a](/yaks/yaks/yaks-0f3a.md), [yaks-8f50](/yaks/yaks/yaks-8f50.md), [yaks-71c2](/yaks/yaks/yaks-71c2.md), [yaks-6905](/yaks/yaks/yaks-6905.md) with [yaks-59be](/yaks/yaks/yaks-59be.md), [yaks-7350](/yaks/yaks/yaks-7350.md), [yaks-abdc](/yaks/yaks/yaks-abdc.md), [yaks-e3b1](/yaks/yaks/yaks-e3b1.md), [yaks-5bc9](/yaks/yaks/yaks-5bc9.md), [yaks-0fa7](/yaks/yaks/yaks-0fa7.md), [yaks-0ef9](/yaks/yaks/yaks-0ef9.md), [yaks-64c4](/yaks/yaks/yaks-64c4.md), [yaks-bda5](/yaks/yaks/yaks-bda5.md), [yaks-e61e](/yaks/yaks/yaks-e61e.md) as a gate, [yaks-4a0d](/yaks/yaks/yaks-4a0d.md). Closed by the work: [yaks-7183](/yaks/yaks/yaks-7183.md), [yaks-bd59](/yaks/yaks/yaks-bd59.md), [yaks-323b](/yaks/yaks/yaks-323b.md), [yaks-b2e9](/yaks/yaks/yaks-b2e9.md), [yaks-8d53](/yaks/yaks/yaks-8d53.md). Predecessor: [yaks-44b3](/yaks/yaks/yaks-44b3.md), whose open children are rehomed before it is shorn.

## 13. Open questions

- [x] Prose mentions: section 5 as written (completion inserts inline links; schema 11 converts existing mentions; the TUI stops autolinking). Joel, 2026-11-27, on yaks-7e32.
- [x] Index storage: prototype first. tantivy only if it solves the whole problem (postings, link edges, times) with no second store; otherwise SQLite with FTS5 (`rusqlite`, bundled), accepting the C dependency. The prototype measures a cold-cache build of the whole index. Joel, 2026-11-27.
- [x] Recent and All: merge them ([yaks-7350](/yaks/yaks/yaks-7350.md)), overriding tui-model decisions 3 and 7 ("both just weren't useful"). Joel, 2026-11-27.

## Rejected

- Nesting yaks under their parent's path (section 3).
- UTF-8-style self-delimiting ids (section 4).
- One-digit sharding; no sharding (section 4).
- Reference-style yak links; converting prose on every save (section 5).
- Filesystem mtime as the modified time (section 9).
- A trigram index (section 9).
- Map-typed fields (section 6).
- Lifecycles or behaviour defined by farm config (section 2).

## Related design doks

[knowledge-store](/doks/design/knowledge-store.md) (the pieces and rules this keeps), [farm-layout-and-links](/doks/design/farm-layout-and-links.md) (stored path links, decision 6 on sharding, the resolver seam), [note-format](/doks/design/note-format.md), [files-directories-archives](/doks/design/files-directories-archives.md) (a subject's name is its link destination), [tui-model](/doks/design/tui-model.md), [tui-keys](/doks/design/tui-keys.md).
