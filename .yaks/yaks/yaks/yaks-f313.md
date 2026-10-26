---
id: yaks-f313
title: Some way for agents to get notified of available answers
status: shorn
type: feature
priority: 3
created: '2026-10-04T20:50:20Z'
updated: '2026-10-26T22:01:42Z'
labels:
- cli
- agent
---

Maybe it's just a skill thing, but it would be nice if there were a way for an agent to run a CLI command in the background, and have it return when something's ready. They do this sort of thing a lot for other CLI tools.

---
▸ 2026-10-05T22:44:10Z [delta-lead]

Relationship to yaks-c061, and when to do it (coordinator delta-lead, 2026-10-05; Joel asked whether they are redundant and whether to do them sooner).

## Not redundant, but ordered: c061 first
- yaks-c061 is about STATE: after a human answers, there is no findable "answered, pending agent pickup" state (a plain-note reply leaves `needs: human` set; an `answer` on a shorn yak drops out of both `inbox` and `next`). It is what lets anyone, human or agent, find an answer.
- yaks-f313 is about WAITING: a command an agent can run in the background that returns when something is ready. It needs the state c061 defines to wait FOR. Without c061 it would wait on an inbox that does not contain what we need.

## Why f313 is weak in Delta today
- A worker does not block: it runs `yaks ask`, returns, and the coordinator wakes it with a message. The coordinator is the one that needs to notice an answer, and it notices by reading the yaks (this very round: Joel's answers sat uncommitted in his checkout; I found them by looking).
- Background commands do not survive a terminal call. I tested it on this thread: a job started with `&` was gone when the call returned (empty log). So "run it in the background and have it return" is not available in Delta terminals; harnesses with real background tasks could use it.
- What Delta already gives is the other direction: subagent completion messages and `send_agent_message`.

## Recommendation
- Do c061 soon, in two slices. (1) No format change: `yaks inbox` marks a blocked yak whose latest note after the ask is from a different actor ("replied") and lists answered-but-untouched yaks since a time (`--answered --since`). Derived from notes, cheap. (2) The `needs: agent` flip is a design fork for Joel.
- It pairs with the lanes work (yaks-7204): asks and answers that sit in another checkout need the same cross-checkout visibility.
- Leave f313 open and lower priority until c061 exists; when it does, `yaks wait [--for answers]` is a small command for harnesses that can run background tasks. In Delta the coordinator loop (check lanes and inbox, wake the worker) is the supported path.

---
▸ 2026-10-26T21:51:27Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-26T21:51:51Z [delta-lead]

Claim (delta-lead, yakherd). Owner: `wait-1`. Design: the latest design note on yaks-33c8.

**Goal:** `yaks wait` blocks until the inbox has something for the given audience, then prints it and exits 0.

- Flags: `--for human|agent` (reuse `Audience`), the shared `FilterFlags`, `--timeout <dur>` (reuse the duration parsing `log --since` uses; no timeout means wait forever), `--new`, `--json`, and `--interval <dur>` (default 2s; hidden, or documented as a tuning knob).
- On a hit it prints exactly what `yaks inbox` prints for the same audience and filters, by calling `render_inbox(rows, audience, json)` (do not change its signature or body; another worker, `request-1`, owns its body and is changing the agent section title/markers).
- A timeout exits 2 with a one-line stderr message and nothing on stdout.
- `--new` ignores rows (by id and `needs` value) already present at start, so a yak that flips from human to agent counts as new.
- Implementation: poll `Farm::inbox` each interval, reading files and taking no lock. Put the logic in `src/farm.rs` (or a small `src/wait.rs`) so it can be tested without sleeping for real: inject the clock or the interval.

**Scope:** a `Wait` variant right after `Inbox` in `src/main.rs`, its dispatch arm right after `Command::Inbox`, the new logic module. `docs/cli.md` gets a `### wait` after `### inbox` in "Human-in-the-loop". The README command table gets a row if inbox has one. `--help` doc comments.

**Out of scope:** the `request` verb and inbox rendering (`request-1`), the skills (delta-lead), any daemon or file watcher.

**Evidence:**
- Tests that fail first: returns immediately when the inbox is non-empty, times out (exit 2) when it is empty, wakes when another writer answers mid-wait (a thread that writes after a short delay), and `--new` ignores pre-existing rows.
- A real-binary transcript: `yaks wait --for agent --timeout 30s` in one shell while another runs `yaks answer`, pasted in a note.

**Gate:** `cargo test`. **Judge:** delta-lead at landing.

---
▸ 2026-10-26T21:52:30Z [wait-1]

Started (wait-1): `date -u` = Mon Oct 26 21:52:25 UTC 2026. `yaks path yaks-f313` = /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-f313.md

[yaks-f313-wait-transcript](yaks-f313/yaks-f313-wait-transcript.txt)

---
▸ 2026-10-26T21:58:14Z [wait-1]

Real-binary transcript (`./target/debug/yaks`, throwaway `mktemp -d` farm, `init --mode private`), attached as `yaks-f313-wait-transcript.txt`. Shell A ran `yaks wait --for agent --timeout 30s` in the background at 21:57:46; shell B ran `yaks answer` at 21:57:50; A exited 0 at 21:57:52 (within one 2s poll):

```
# shell A output:
Answered, awaiting an agent (`yaks pickup <id>`):
  [H] yak-3d1b  p3 task     Pick a color ✓ answered (needs:agent)
exit=0
21:57:52
$ yaks inbox --for agent   # for comparison
Answered, awaiting an agent (`yaks pickup <id>`):
  [H] yak-3d1b  p3 task     Pick a color ✓ answered (needs:agent)
$ yaks wait --for agent --new --timeout 3s
yaks wait: timed out after 3s: nothing new awaiting an agent
exit=2
$ yaks wait --for human --timeout 0s
yaks wait: timed out after 0s: nothing awaiting a human
exit=2
$ yaks wait --timeout banana
Error: could not parse --timeout 'banana'; use a duration like 30s, 10m or 2h
exit=1
```

Tests fail first: with the hit predicates stubbed to `false`, 4 of the 7 `wait::tests` failed (immediate hit, `--new` flip, `--new` leave-and-return, mid-wait wake); restored, all 7 pass, and the 4 binary tests in `tests/wait.rs` pass.

---
▸ 2026-10-26T21:58:15Z [wait-1]

Decision: the duration grammar is factored out of `store::parse_relative` as `store::parse_duration` (int + s/m/h/d/w), shared by `log --since` and `wait --timeout/--interval`. Rejected: calling `parse_since` and subtracting from now (accepts absolute dates, which make no sense as a timeout); adding a duration crate (forbidden, and a second grammar). Consequence: no sub-second interval (`500ms` is rejected); `--interval 0s` is refused.

---
▸ 2026-10-26T21:58:15Z [wait-1]

Decision: `--interval` is documented (help + docs/cli.md) as a tuning knob rather than hidden. Rejected: hidden, since a harness that wants faster wake-ups or less polling has no other way to discover it.

---
▸ 2026-10-26T21:58:15Z [wait-1]

Decision: `--new` keys rows by (id, needs) and keeps in its baseline only rows continuously present since start, so a yak picked up and then answered again during the wait counts as new. Rejected: a fixed start snapshot, which would miss that re-answer. Rows filtered out by `--for` never enter the baseline, so a human->agent flip is new under `--for agent` either way.

---
▸ 2026-10-26T21:58:15Z [wait-1]

Decision: without `--for`, a hit is any row with a `needs` value (both sections), matching what `inbox` with no `--for` shows. The poll loop lives in a new `src/wait.rs` (generic over a fetch closure and a `Clock` trait) rather than in `src/farm.rs`, so the tests use a fake clock and no sleeps, and farm.rs (which request-1 is also editing) is untouched. Binary tests go in a new `tests/wait.rs`, not `tests/needs_loop.rs`, for the same reason.

---
▸ 2026-10-26T21:58:16Z [wait-1]

Decision: the timeout message is `yaks wait: timed out after <DUR>: nothing [new ]awaiting <who>` on stderr, exit 2; a bad flag value is an ordinary error, exit 1. Rejected: exit 1 for a timeout, which a caller could not tell apart from an error.

---
▸ 2026-10-26T21:59:23Z [wait-1]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-26T21:59:32Z [wait-1]

Summary (wait-1): `yaks wait [--for human|agent] [filters] [--timeout DUR] [--new] [--json] [--interval DUR]` polls `Farm::inbox` (no lock) until a row for the audience appears, then calls `render_inbox(rows, audience, json)` unchanged, so it prints exactly what `inbox` prints, and exits 0. A timeout exits 2 with one stderr line and nothing on stdout.

- Code: `src/wait.rs` (the `wait_for` loop over a fetch closure plus a `Clock` trait, 7 unit tests incl. a real-file mid-wait answer from another thread), `Command::Wait` after `Inbox` in `src/main.rs`, and `store::parse_duration` factored out of `parse_relative`.
- Binary tests: `tests/wait.rs` (hit == inbox output for 3 flag sets, timeout exit 2 with empty stdout, `--new`, a background `answer` waking a spawned waiter).
- Docs: `--help`, `docs/cli.md` (table row + `### wait`), README command row.
- Evidence: transcript note above (attached `yaks-f313-wait-transcript.txt`); `yaks verify` PASS.

---
▸ 2026-10-26T21:59:32Z [wait-1]

moved: shaving -> shorn

---
▸ 2026-10-26T22:01:42Z [delta-lead]

Landed (delta-lead, yakherd): `655dacf` fast-forwarded onto `main` (the base matched). Not pushed yet; it goes out with yaks-b75b.

- Gate re-run: 860 passed, 0 failed.
- Reproduced with my own debug build on a throwaway farm: on an empty agent inbox, `wait --for agent --timeout 3s` exits 2 with the stderr line. With a background `answer` 2s later, the wait printed the inbox and exited 0 about 4s after it started. `--new` then timed out (exit 2), as expected.
- Read the diff: `main.rs` and `store.rs` are clean; `parse_duration` is shared with `log --since`.
