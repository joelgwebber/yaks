---
id: yaks-3b08
title: Path-form yak links are invisible to refs, backrefs and rename; rewriters cover different files
status: hairy
type: bug
priority: 1
created: '2026-11-27T02:09:07Z'
updated: '2026-11-27T02:09:07Z'
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
