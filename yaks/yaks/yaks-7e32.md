---
id: yaks-7e32
title: 'One document model: yaks, doks, sessions and sources on one set of constructs'
status: shaving
type: design
priority: 2
created: '2026-11-27T02:09:06Z'
updated: '2026-11-27T02:38:22Z'
tags:
- design
- doks
---

Umbrella for unifying yaks, doks, sessions, sources and release doks on one set of constructs: fields and schemas, fields with behaviour, links, notes, attachments, queries and a derived index. Kinds stay registered in Rust with their own capabilities and opinionated views; the parts under them are shared.

Design: [/doks/design/one-document-model.md](/doks/design/one-document-model.md). Started with Joel on 2026-11-26 in a Delta thread; decisions are recorded in the dok as they settle.

The implementation plan (children, and the existing yaks pulled in) is filed once the open questions in the dok's section 13 are answered.

#### 2026-11-27T02:09:56Z

*unify-design · kind: ask · needs: human*

asked: needs human

Open questions from [the design dok](/doks/design/one-document-model.md#13-open-questions), each with a lean:

1. **Prose mentions** (section 5). Lean, following Joel's comment: completion inserts inline links (`[yaks-1a2b](/yaks/yaks/1a/yaks-1a2b.md)`, the short form `[yaks-1a2b](yaks-1a2b)` canonicalised on save); one picker over every kind gives dok-link completion for free; schema 11 converts the 4,866 existing prose mentions to links; the TUI then stops autolinking bare mentions. Confirm, or keep autolinking as a reading aid.
2. **Index storage** (section 9). Options: SQLite with FTS5 (`rusqlite` bundled: tokenizer, BM25, phrase and prefix queries, plus tables for edges and times; about 1.5 MB of C, builds on all five release targets), tantivy (Rust-native search, but a heavy dependency and still needs another store for edges), or our own positional postings (small, but we own ranking and phrase code). Lean: SQLite, if its Unicode tokenizer plus our identifier-aware pre-pass is good enough; prototype before committing.
3. **Recent and All** (section 10): merge them ([yaks-7350](/yaks/yaks/yaks-7350.md)), reversing tui-model decisions 3 and 7, once All sorts every kind by modified time? Lean: yes, decided together with yaks-6905.

#### 2026-11-27T02:23:11Z

*sched-1 · kind: answer · needs: agent*

1. Perfect.
2. Works for me; I'd be ok with a heavy rust crate if it solved the whole problem without an external dep. But if tantivy doesn't solve the whole problem, then I'd rather live with the C dep for SQLite. When we're prototyping, let's also try and get an estimate of cold-cache indexing build time, so at least we have a sense for the worst case.
3. Yes, agreed. Both just weren't useful. Agreed on overriding those old decisions.

#### 2026-11-27T02:28:25Z

*unify-design · kind: pickup*

picked up

Recording the three answers in the design dok: links inserted by completion and prose converted in schema 11; index storage prototyped (tantivy only if it covers postings, edges and times alone, else SQLite FTS5), with a cold-cache build time measured; Recent and All merged, overriding tui-model decisions 3 and 7.

#### 2026-11-27T02:28:37Z

*unify-design*

Decision (Joel, 2026-11-27, answering the ask): all three settled and recorded in [the dok](/doks/design/one-document-model.md#13-open-questions).

- Prose mentions: completion inserts inline links; schema 11 converts the existing mentions; the TUI stops autolinking.
- Index storage: prototype tantivy against SQLite FTS5 over the whole job (postings, edges, times), measuring a cold-cache build; tantivy only if it needs no second store.
- Recent and All merge (yaks-7350), overriding tui-model decisions 3 and 7.

Also added (Joel, 2026-11-27): one picker to open or link any document, yaks-958d. The design is agreed; next is the implementation plan as children.

#### 2026-11-27T02:36:26Z

*herd-unify · kind: move · from: hairy · to: shaving · child: yaks-3b08 · thread: ksQQvf9tyN_TT6-lYi87QcE1GpLOAHx7AZIBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-27T02:38:22Z

*herd-unify*

Batch 1 (yakherd `herd-unify`, base `4f09d2d2`, local `main` fast-forwarded to include yaks-ca59):

| worker | yak | model | thread |
|---|---|---|---|
| `desc-1` | yaks-2c10 | claude-sonnet-5-5 | ksQQvf9tyN_TT6-lYi87QcE1GpLOAIK5vJIBxBCA0aLJyrhJPreUvVwLftHG |
| `pick-1` | yaks-958d | claude-sonnet-5-5 | ksQQvf9tyN_TT6-lYi87QcE1GpLOAIK58ZIBxBCA0aLJyrhJPreUvVwLftHG |
| `relink-1` | yaks-3b08 | claude-opus-5-5 (shared engine) | ksQQvf9tyN_TT6-lYi87QcE1GpLOAIK7O5IBxBCA0aLJyrhJPreUvVwLftHG |

Briefs: `~/.local/share/yaks/briefs/<worker>.md`. yaks-0b10 waits for yaks-3b08 (its repair renames through the rewrite engine).
