---
id: yaks-7d41
title: 'Matcher core: normalization, identifier-aware tokens, query parser (pure, tested)'
status: shaving
type: feature
priority: 2
created: '2026-11-27T02:43:24Z'
updated: '2026-11-27T03:12:19Z'
parent: yaks-23c9
tags:
- cli
---

The pure core of the shared matcher in [the design, section 8](/doks/design/one-document-model.md#8-queries-and-search), with no wiring into commands yet: a new module (`src/search/` or `src/matcher.rs`) and its tests.

- **Normalization:** NFC, Unicode case folding (full, so `ß` matches `SS`), accent folding; a map from normalized offsets back to the original text's byte offsets. Use `unicode-normalization` and `unicode-segmentation` (Joel approved the dependencies, 2026-11-27); case folding from the standard library or a small crate if needed (say which).
- **Tokens:** Unicode word boundaries, identifier-aware: a run like `issues/123`, `yaks-ca59`, `src/foo.rs` yields its parts and the whole. Each token carries its position and original byte range.
- **Query parser:** words (ANDed), the last word a prefix when asked, `"quoted phrase"`, `-word`, `field:value` scopes (`tag:`, `type:`, `status:`, `kind:`, `herd:`, `path:`, and any other `key:`), producing a small query tree. Unknown syntax degrades to words, never an error.
- **Matching over a document's text fields:** given named fields (title, tags, body, ...), does the query match, and where (ranges for highlighting). Phrases match consecutive tokens across line breaks. No ranking beyond an exact-id/path-prefix flag (BM25 belongs to the index).

Out of scope: wiring into `find`, `--search`, the TUI or pickers; the index.

Evidence: unit tests for each rule above (accents, `ß`, `İ`, offset mapping, identifier tokens, phrase across a newline, `-word`, scopes, garbage input). Gate `cargo test -p yaks`, `cargo fmt --check`.

#### 2026-11-27T02:43:26Z

*herd-unify · kind: move · from: hairy · to: shaving · thread: ksQQvf9tyN_TT6-lYi87QcE1GpLOAHx7AZIBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-27T02:43:41Z

*herd-unify*

Claim (yakherd `herd-unify`). Owner: worker `match-1` (claude-sonnet-5-5), base `4f09d2d2`. Scope: the description; a new module only, plus its `mod` line in `src/main.rs`. **Allowed exception to the brief:** add `unicode-normalization` and `unicode-segmentation` (and a case-folding crate if needed) to `Cargo.toml`, with the `Cargo.lock` change; nothing else in either file. Judge: the yakherd at landing.

#### 2026-11-27T03:00:58Z

*match-1*

Start: Fri Nov 27 02:44:09 UTC 2026. Farm file: `/Users/joel/.local/share/yaks/farms/yaks-539d3edb/yaks/yaks/yaks-7d41.md`. Checkout: `/Users/joel/src/yaks/.delta/worktrees/aefczrq61170/yaks`.

Implemented in `src/matcher.rs` + `src/matcher/{normalize,tokens,query,search}.rs`; 58 unit tests pass.

#### 2026-11-27T03:05:56Z

*match-1 · kind: verify · result: fail*

verify: `cargo test -p yaks` -> FAIL (exit 101)

#### 2026-11-27T03:12:19Z

*match-1 · kind: verify · result: pass*

verify: `cargo test -p yaks` -> PASS (exit 0)
