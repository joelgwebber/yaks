---
id: yaks-f4ea
title: 'YAKS_THREAD: a harness-neutral thread id for moves and session notes'
status: shorn
type: feature
priority: 3
created: '2026-11-07T16:57:32Z'
updated: '2026-11-07T17:33:01Z'
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

---
▸ 2026-11-07T17:06:29Z [stars-thread]

Start: Sat Nov  7 17:06:26 UTC 2026

- yaks path yaks-f4ea: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-f4ea.md
- checkout: /Users/joel/src/yaks/.delta/worktrees/sf3w4x0cnprb/yaks

---
▸ 2026-11-07T17:09:10Z [stars-thread]

Red before green (src/actor.rs reverted via stash, new tests run): all three failed.

- `move_thread_is_yaks_thread_over_the_delta_thread_id` (tests/attribution.rs): FAILED
- `whoami_reports_yaks_thread_over_the_delta_thread_id` (tests/attribution.rs): FAILED
- `a_session_note_records_yaks_thread_over_the_delta_thread_id` (tests/sessions.rs): FAILED

Own build (`cargo build`, `./target/debug/yaks whoami`):

```
$ YAKS_THREAD=abc YAKS_ACTOR=w yaks whoami
actor: w (from $YAKS_ACTOR)
thread: abc
$ DELTA_CURRENT_THREAD_ID=dd YAKS_THREAD=abc YAKS_ACTOR=w yaks whoami
actor: w (from $YAKS_ACTOR)
thread: abc
$ DELTA_CURRENT_THREAD_ID=dd YAKS_ACTOR=w yaks whoami
actor: w (from $YAKS_ACTOR)
thread: dd
```

Decision: one reader. All three places (move metadata in store.rs, `session note`, `whoami`) already call `actor::harness_thread()`, so the change is that function, over a `THREAD_VARS` list and an injectable `thread_from` for a unit test. Rejected: per-caller lookups.

Decision: an empty or blank `YAKS_THREAD` counts as unset and falls through to the Delta id (same as the existing Delta rule), rather than blanking the thread.

Decision: the per-place tests are integration tests that spawn the binary with `.env(..)`, not unit tests with `set_var`. Added `env_remove("YAKS_THREAD")` to the env scrubbing in tests/attribution.rs, tests/sessions.rs (new `cmd_env` helper; `cmd` delegates) and tests/sheds.rs so a developer's shell can't leak it into existing tests.

Decision: left `harness_actor` (`delta:<DELTA_CURRENT_THREAD_ID>` actor fallback) and `harness_from_env` (`--harness` default) alone: actor names and `--harness` are out of scope, so YAKS_THREAD does not derive an actor or a harness.

---
▸ 2026-11-07T17:09:59Z [stars-thread]
kind: verify
result: fail
---

verify: `cargo test -p yaks` -> FAIL (exit 101)

---
▸ 2026-11-07T17:14:53Z [stars-thread]
kind: verify
result: pass
---

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-11-07T17:23:43Z [stars-thread]

Summary: `actor::harness_thread()` (the one reader behind move `thread:`, `session note`'s `thread:` and `whoami`) now reads `YAKS_THREAD`, then `DELTA_CURRENT_THREAD_ID`; blank counts as unset. Docs: `docs/cli.md` (session note frontmatter, whoami, Attribution), the `whoami` clap help, and the yaks-coordinating skill line. Tests: 3 integration tests (move, whoami, session note; each failed before the change) plus a unit test for the precedence. `yaks verify` PASS. One test assertion of mine was wrong at first (the `delta:t-123` actor stamp contains the id); fixed to assert on `thread: t-123`.

---
▸ 2026-11-07T17:23:44Z [stars-thread]
kind: move
from: shaving
to: shorn
thread: ksQQcvZn6b-aQ7OxtwNwU3-8iJLOAEpr_pIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn

---
▸ 2026-11-07T17:33:01Z [herd-stars]
kind: landed
commit: b7cebb1b4da01ef2c13afb4d5d1e504b552bd911
---

Landed by herd-stars as `b7cebb1b` (worker `797b367a`), by cherry-pick: a conflict in `tests/sessions.rs` with yaks-db66's new test, resolved by keeping both tests. Gate `cargo test --workspace` green on the merged tree.
