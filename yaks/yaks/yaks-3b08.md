---
id: yaks-3b08
title: Path-form yak links are invisible to refs, backrefs and rename; rewriters cover different files
status: shaving
type: bug
priority: 1
created: '2026-11-27T02:09:07Z'
updated: '2026-11-27T15:03:50Z'
parent: yaks-7e32
tags:
- links
- cli
---

Found in the design pass for the one document model (the parent). Reproduced in scratch farms by a scout.

## The regression

Schema 9 stores a yak link as `/<herd>/yaks/<id>.md`, and yaks-ca59 (`fd35a6e4`, `4f09d2d2`, on local `main`, **not pushed**) makes every writer canonicalise to it. `refs::scan` tokenizes `[a-z0-9.-]`, so `zz-a8f6.md` is one token and never equals the id. With that build, `create --description "see [x](zz-95ad)"` stores the path, and then `refs <target> --incoming` and `refs <src>` both print nothing. The TUI backrefs (`backrefs_in`) and `rename` read the same way. Don't push `main` before this is fixed.

## Rewriter coverage today

- `rename` / `rename-herd`: miss path links and `#slug` note links; scan yak titles and bodies only (not doks, sessions, sources, markdown attachments, or dok `depends_on`/`parent` fields). `rename-herd` leaves the `/<herd>/` segment (`/zz/yaks/qq-ac76/shot.png`) and the herd's `<herd>/doks/`.
- `rename-attachment`: turns `/zz/yaks/A/shot.png` into a relative `A/pic.png` inside another yak's attachment; skips doks and sessions. Its error prints a stale `.yaks/` path (`src/main.rs` around L2909).
- `promote`, `mv`: skip sessions; `mv` also skips markdown attachments and frontmatter link fields (`sources:`).

## Fix

The design's one rewrite engine (section 5 of the dok): `relocate(old, new)` over body links, link-typed fields and the short form, across every document and markdown attachment; and `refs` reading references through `links.rs`, with `refs.rs` kept only for non-markdown text. Gate: a scratch-farm test per rewriter and link form, and `yaks doctor` clean on a copy of this farm after a rename.

#### 2026-11-27T02:36:26Z

*herd-unify · kind: move · from: hairy · to: shaving · thread: ksQQvf9tyN_TT6-lYi87QcE1GpLOAHx7AZIBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-27T02:36:57Z

*herd-unify*

Claim (yakherd `herd-unify`). Owner: worker `relink-1` (Opus 5.5: it is the shared engine later work builds on), base `4f09d2d2`.

Scope, following [the design, section 5](/doks/design/one-document-model.md#5-links-and-references):

- **References read through links.** `Farm::refs`, `backrefs`/`backrefs_in` (and so the TUI backrefs) count a markdown link that resolves to a yak (short form, path form, `#note`, attachments) in yak bodies as `links::edges` already does for doks. Bare prose mentions keep counting for now (the TUI autolinks them until schema 11), but a path link must never be missed.
- **One rewrite engine** in the farm layer, e.g. `Farm::relocate`: given a renamed yak id, attachment or dok path, rewrite body links (all forms, keeping `@rev` and `#anchor`, canonical output), link-typed frontmatter (`parent`, `depends_on`, dok `sources:`, schema `link` fields) and, for yak ids, bare prose mentions; across every yak, dok, session, source and markdown attachment. `rename`, `rename-herd` (including the `/<herd>/` segment and `<herd>/doks/`), `mv`, `promote`, `rename-attachment` call it; remove their private loops (`rename_many_locked`'s `refs::rewrite` pass, `dok_move`'s `fix`, `rewrite_links_to`, `replace_path_token`/`replace_link`).
- Fix the stale `.yaks/` path in `rename-attachment`'s error.

Out of scope: changing stored formats, prose-to-link conversion, sharding, the TUI beyond what backrefs need. Ask before touching `src/tui/` otherwise.

Evidence: an integration test matrix (each rewriter × each link form × each file kind) in `tests/`, failing before; `refs`/`refs --incoming` on a path link; then on a copy of the real farm (`cp -r $(yaks path) $TMPDIR/f`, `YAKS_DIR=...`): `yaks rename` one well-linked yak and back, `yaks doctor` reports no new broken links. Gate `cargo test -p yaks`, `cargo fmt --check`. Judge: the yakherd, re-running the farm-copy check.

#### 2026-11-27T02:38:21Z

*relink-1*

Started (relink-1).

- `date -u`: Fri Nov 27 02:38:03 UTC 2026
- `yaks path yaks-3b08`: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/yaks/yaks/yaks-3b08.md
- checkout: /Users/joel/src/yaks/.delta/worktrees/9cs3xapk1bba/yaks (HEAD 4f09d2d2)

#### 2026-11-27T12:47:20Z

*relink-1*

Resumed after a network drop: no edits had been made yet (clean tree). Read the four rewriters, `refs`, `backrefs_in` and `links.rs`; design next note, then code.

#### 2026-11-27T14:47:29Z

*relink-1*

Progress: engine in place, unit tests green (`cargo test -p yaks --bin yaks -- farm::tests`: 70 passed).

- New `src/relocate.rs`: `Move` (Yak, Attachment, Promoted, Path), `Relocation` (rewrites one text: links via `links::rewrite_spans`, link-typed frontmatter, prose ids outside link destinations) and `run` (every yak, markdown attachment, dok, session, source; moves the files; writes each document once).
- `links.rs`: `rewrite_spans` (dest, label, image `!`), `dest_ranges`, `yak_mentions` (links resolving to a yak in any form plus bare mentions outside destinations); `rewrite`/`rewrite_kind` now wrap `rewrite_spans`.
- `Farm::refs` and `backrefs_in` read bodies through `yak_mentions`.
- `rename`/`rename-herd`, `mv`, `promote`, `rename-attachment` call `relocate::run`; removed `rewrite_links_to`, `dok_move`'s `fix`, `replace_link`, `replace_path_token`, and the `refs::rewrite` loop in `rename_many_locked`.

Next: integration matrix in `tests/`, docs, farm-copy check.

#### 2026-11-27T15:03:48Z

*relink-1*

Decision: the engine is a new module `src/relocate.rs` (`Move`, `Relocation`, `run`), not methods on `Farm`. `Farm` validates and takes the lock, then calls it. Rejected: one `Farm::relocate` method (`farm.rs` is already 6k lines, and the migration and collision repair will call the engine without a `Farm`); extending `links.rs` (it is the grammar and stays pure).

#### 2026-11-27T15:03:48Z

*relink-1*

Decision: a bare path in prose (`see yaks-1a2b/shot.png`, no link) is no longer rewritten by `rename-attachment`. The old `replace_path_token` did rewrite it. This follows the design: "a reference is a markdown link that resolves". Bare yak *ids* in prose are still rewritten on `rename`. Rejected: keeping a second, text-token rewriter for attachment paths only. The unit test `rename_attachment_moves_file_and_rewrites_links` changed to match.

#### 2026-11-27T15:03:49Z

*relink-1*

Decision: a rewritten attachment link is written canonically (`links::canonical_dest`) when the old form would not resolve from where it sits. A link in another herd's yak, or in a dok, becomes `/<herd>/yaks/<id>/<file>`. Before, `rename` left `yaks-0001/a.png` in a `yak-` herd yak, and `promote --dok` left `<id>/<file>` in the dok. A link already in farm-path form stays a farm path; a bare sibling stays bare. Rejected: always keeping the form as written, which reproduces the bug where `rename-attachment` turned `/zz/yaks/A/shot.png` into a relative `A/pic.png` inside another yak's attachment. Two unit tests changed (`rename_moves_the_attachment_dir_and_fixes_links`, `an_evidence_document_links_its_sibling_attachments_by_bare_name`).

#### 2026-11-27T15:03:49Z

*relink-1*

Decision: a pinned link (`@<rev>`) is never moved, by any command. It names the path as it was at that commit. `mv` used to rewrite the path and keep the `@rev`, which pointed at a path that never existed at that commit. A pinned link inside a moved dok is still rewritten for position (relative to root), rev kept.

#### 2026-11-27T15:03:49Z

*relink-1*

Decision: `rename-herd` moves `<old>/doks/` to `<new>/doks/` (one `Move::Path` for the directory, links following) and refuses when `<new>/doks/` already exists, rather than merging two dok trees. A single `rename` that happens to empty a herd moves no doks.

#### 2026-11-27T15:03:50Z

*relink-1*

Decision: link-typed frontmatter is edited in the raw lines (`Fields::raw`/`set_raw`), replacing only the value text, so quotes, flow lists and a source's `sources:` provenance map keep their exact shape. The keys are `parent`, `depends_on`, `sources`, `part_of`, `superseded_by` and any schema `link` field. Rejected: `Fields::set`, which reflowed the whole entry (it turned a source's provenance map into flow style).
