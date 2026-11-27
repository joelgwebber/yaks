---
id: yaks-3b08
title: Path-form yak links are invisible to refs, backrefs and rename; rewriters cover different files
status: shaving
type: bug
priority: 1
created: '2026-11-27T02:09:07Z'
updated: '2026-11-27T02:36:57Z'
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

Claim (yakherd `herd-unify`). Owner: worker `links-1` (Opus 5.5: it is the shared engine later work builds on), base `4f09d2d2`.

Scope, following [the design, section 5](/doks/design/one-document-model.md#5-links-and-references):

- **References read through links.** `Farm::refs`, `backrefs`/`backrefs_in` (and so the TUI backrefs) count a markdown link that resolves to a yak (short form, path form, `#note`, attachments) in yak bodies as `links::edges` already does for doks. Bare prose mentions keep counting for now (the TUI autolinks them until schema 11), but a path link must never be missed.
- **One rewrite engine** in the farm layer, e.g. `Farm::relocate`: given a renamed yak id, attachment or dok path, rewrite body links (all forms, keeping `@rev` and `#anchor`, canonical output), link-typed frontmatter (`parent`, `depends_on`, dok `sources:`, schema `link` fields) and, for yak ids, bare prose mentions; across every yak, dok, session, source and markdown attachment. `rename`, `rename-herd` (including the `/<herd>/` segment and `<herd>/doks/`), `mv`, `promote`, `rename-attachment` call it; remove their private loops (`rename_many_locked`'s `refs::rewrite` pass, `dok_move`'s `fix`, `rewrite_links_to`, `replace_path_token`/`replace_link`).
- Fix the stale `.yaks/` path in `rename-attachment`'s error.

Out of scope: changing stored formats, prose-to-link conversion, sharding, the TUI beyond what backrefs need. Ask before touching `src/tui/` otherwise.

Evidence: an integration test matrix (each rewriter × each link form × each file kind) in `tests/`, failing before; `refs`/`refs --incoming` on a path link; then on a copy of the real farm (`cp -r $(yaks path) $TMPDIR/f`, `YAKS_DIR=...`): `yaks rename` one well-linked yak and back, `yaks doctor` reports no new broken links. Gate `cargo test -p yaks`, `cargo fmt --check`. Judge: the yakherd, re-running the farm-copy check.
