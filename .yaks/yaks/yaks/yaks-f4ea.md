---
id: yaks-f4ea
title: 'YAKS_THREAD: a harness-neutral thread id for moves and session notes'
status: shaving
type: feature
priority: 3
created: '2026-11-07T16:57:32Z'
updated: '2026-11-07T17:05:35Z'
parent: yaks-44b3
tags:
- cli
- doks
---

yaks records `thread:` metadata on moves (and a `thread` field on session notes) only from Delta's `DELTA_CURRENT_THREAD_ID` (`src/actor.rs`). A harness that is not Delta, such as Hearth (hearth-1ed0 in the Hearth farm), cannot point its writes back at its own conversation.

Proposal: `YAKS_THREAD=<id>` wins over the Delta variable wherever yaks reads a thread id: move metadata, `session note`'s `thread:`, and the `whoami` report. Nothing else changes. Related: yaks-8003 (stable thread ids in actor stamps) and yaks-3fc6 (session note records the Delta thread under another `--harness`).

Small; one test per place it is read.

---
▸ 2026-11-07T17:04:43Z [herd-stars]
kind: move
from: hairy
to: shaving
thread: ksQQcvZn6b-aQ7OxtwNwU3-8iJLOAEpUNJIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-07T17:05:35Z [herd-stars]

Claim (yakherd herd-stars): worker `stars-thread`.

**Scope.** As the description says: `YAKS_THREAD=<id>` wins over `DELTA_CURRENT_THREAD_ID` everywhere yaks reads a thread id (move metadata, `session note`'s `thread:`, `whoami`). `src/actor.rs` and its callers; `docs/cli.md`, the clap help that names the Delta variable, and any skill line that says where `thread:` comes from.

**Out of scope.** Actor names (yaks-8003), `--harness` (yaks-3fc6).

**Evidence.** A test per place it is read (fails first). `yaks whoami` output from your own build with `YAKS_THREAD` set, pasted. Gate `cargo test -p yaks`.

**Judge.** herd-stars at landing.
