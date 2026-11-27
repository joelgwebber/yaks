---
id: yaks-7d41
title: 'Matcher core: normalization, identifier-aware tokens, query parser (pure, tested)'
status: shorn
type: feature
priority: 2
created: '2026-11-27T02:43:24Z'
updated: '2026-11-27T03:20:29Z'
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

#### 2026-11-27T03:12:31Z

*match-1*

Decision: case folding is the standard library's (lowercase, uppercase, lowercase), no caseless crate. Rejected `caseless`/`unicode-case-mapping`: the only difference is dotless `ı`, which folds to `i` here; the friendlier answer for search.

Decision: accent folding = drop combining marks in the Unicode blocks 0300-036F, 1AB0-1AFF, 1DC0-1DFF, 20D0-20FF and FE20-FE2F after NFD, then NFC. Rejected dropping all `Mn` (would erase Indic vowel signs). Kana voiced marks (3099/309A) and Hangul jamo are kept. `ø`, `đ`, `ł` are not folded (no decomposition).

Decision: offset map is per grapheme cluster, so a folded range widens to whole original clusters (`ß`->`ss` both map to `ß`). Rejected a per-char map (breaks at composition). Pure ASCII takes an identity fast path.

Decision: tokens split UAX #29 words further at `.`, `_`, `:`; parts joined by `/ \ - _ . : @ #` with no spaces form an identifier chain, emitted as a Whole plus its Parts. Camel case is not split.

Decision: the query tree is `All | Text | Field | Not | And`, and parse never fails (broken scopes, unterminated quotes, `http://x` degrade to words). Prefix is the last term only, never when quoted or followed by whitespace. A bare identifier term is a phrase of its parts. `id_prefix` is set only for `Field::id` fields whose folded text starts with the raw typed term.

Decision: `Field` names are caller-chosen and compared folded; the matcher does not alias `tag`/`tags`. Phrases never cross fields.

Decision: `src/matcher.rs` has `#![allow(dead_code, unused_imports)]` until the callers land (as `fields.rs` does).

#### 2026-11-27T03:12:32Z

*match-1*

Summary: new pure module src/matcher/ (normalize with offset map, tokens, query parser, field matching; 58 unit tests). Gate cargo test -p yaks PASS (verify note), cargo fmt --check clean. Deps unicode-normalization 0.1 and unicode-segmentation 1 added.

#### 2026-11-27T03:12:32Z

*match-1 · kind: move · from: shaving · to: shorn · thread: ksQQvf9tyN_TT6-lYi87QcE1GpLOAILelJIBxBCA0aLJyrhJPreUvVwLftHG*

moved: shaving -> shorn

#### 2026-11-27T03:12:32Z

*match-1 · kind: change · section: none*

No user-facing change.

#### 2026-11-27T03:13:06Z

*herd-unify · kind: landed · commit: 5d5b71bd9e5188dfbe01f38b461cb582482349a8 · source: a44d4b8f1addf48d07a73fb6fdb77908dc5e59d0 · branch: main*

landed: [5d5b71bd](https://github.com/joelgwebber/yaks/commit/5d5b71bd9e5188dfbe01f38b461cb582482349a8) yaks-7d41: matcher core: normalization, identifier-aware tokens, query parser, field matching

#### 2026-11-27T03:20:29Z

*herd-unify*

Landed as `5d5b71bd` (herd-unify). Coordinator check: full gate green including 58 matcher tests; fmt clean. Not wired into any command yet (yaks-23c9). The flaky `release_changes::yaks_release_tags_overrides_the_glob` match-1 saw did not recur in my run.
