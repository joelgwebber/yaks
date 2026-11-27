---
id: yaks-9bcd
title: 'Derived per-user index: postings, link edges, fields, modified times (prototype storage first)'
status: shaving
type: task
priority: 2
created: '2026-11-27T02:38:24Z'
updated: '2026-11-27T13:02:01Z'
parent: yaks-7e32
depends_on:
- yaks-8fb0
- yaks-0ef9
tags:
- perf
needs: human
---

Section 9 of [the design](/doks/design/one-document-model.md#9-the-index-and-modified-time): one derived per-user cache (positional postings, link edges, parsed fields, modified times from git with mechanical commits skipped). Start with the prototype: tantivy against SQLite FTS5 over the whole job, measuring a cold-cache build of this farm and a synthetic 10k one; tantivy only if it needs no second store (Joel, 2026-11-27). Takes over yaks-8f50; needs a per-user cache home (yaks-0ef9).

#### 2026-11-27T02:43:25Z

*herd-unify · kind: move · from: hairy · to: shaving · child: yaks-2704 · thread: ksQQvf9tyN_TT6-lYi87QcE1GpLOAHx7AZIBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-27T13:02:01Z

*herd-unify · kind: ask · needs: human*

asked: needs human

Index storage, from the prototype (yaks-2704, [report](/yaks/yaks/yaks-2704/index-storage-report.md)). Both stores do the whole job with **no second store**, and every query is under 3.5 ms median at 10k files. The worker recommends SQLite; your rule from 2026-11-27 ("ok with a heavy Rust crate if it solves the whole problem without an external dep") points at tantivy, which qualifies.

| | SQLite FTS5 | tantivy |
|---|---|---|
| build: this farm / 10k | 0.23 s / 1.8 s | 0.66 s (0.19 s on 1 thread) / 0.77 s |
| update one file | 0.5–5 ms | 85–115 ms |
| index size, 10k | 62 MB | 11 MB |
| binary / crates added | +1.8 MB / 7 | +4.4–5.3 MB / ~50 |
| build | C compiler on all five targets (works with today's workflow, but the "pure Rust" note in `release.yml` stops being true) | pure Rust with `default-features = false` |
| identifier tokenizer | ~170 lines of `unsafe` FFI | ~110 lines of safe Rust |
| edges, mtimes | ordinary tables, joins | a multi-valued field and a fast field; outgoing links and joins are clumsier |

Cold build is dominated by the `git log` pass for modified times either way (0.5 s here, ~9 s at 28k commits), so that pass needs to be incremental in both.

Which store? **Lean: tantivy**, by your rule: pure-Rust release builds, a safe tokenizer and a smaller index matter more over time than ~3 MB of binary, and the clumsy parts (outgoing links, joins) are needs the rewrite engine meets by scanning files anyway. The 100 ms single-file update is fine for a save but means batching writes. Pick SQLite if you'd rather have the smaller dependency tree and plain tables.
