---
type: session-review
title: 'Wrapping up yaks-1c89: tools, refs, reflection'
source: session-note
actor: herd-upkeep-2
person: Joel Webber
host: joel
harness: delta
thread: ksQQKUp3Ux8dTNqgb6cSSWpUMpLOAEX_fZIBxBCA0aLJyrhJPreUvVwLftHG
day: 2026-11-06
reviewed: '2026-11-07T00:04:54Z'
good: 2
---

## Summary

Joel asked to finish yaks-1c89 (except yaks-3f35, another thread's) and its sibling yaks-ff4c. I removed `import-hearth` myself, wrote the reflection design ([yaks-7168](yaks-7168#2026-11-06T23:46:31Z)), and fanned out five workers for the rest: `doks-1`, `tools-1`, `tui-1`, `query-2` and `refs-1`. Everything landed on `main` in two pushes ([first](yaks-1c89#2026-11-07T00:00:18Z), [second](yaks-1c89#2026-11-07T00:04:36Z)) and is installed. yaks-1c89 stays open only for yaks-3f35 and yaks-323b.

## Turns

- good: work through the rest of yaks-1c89 and yaks-ff4c, with sub-agents
- good: answer the reflection ask, then plow ahead and land it

## Discussion

Joel accepted all five reflection leans: one farm-wide `doks/lessons.md`, agents read it at session start, instruction-file changes only as asks, a two-session threshold, and a third weekly recurring yak. He liked that every decision goes through the one ask mechanism, Hearth's harness included.

## Saved

- yaks-06cd: Load-sensitive tests: init_modes worktree test and sync's private-farm test fail under parallel builds (created)
- yaks-1c89: Upkeep: session notes, session review and housekeeping (Hearth's knowledge procedures in yaks)
  - [Fan-out, 2026-11-06 (yakherd `herd-upkeep-2`), all from `main` at 671320f9 (w...](yaks-1c89#2026-11-06T23:15:35Z)
  - [Landed on `main` at 30653149 and installed (`cargo install --path .` from the...](yaks-1c89#2026-11-07T00:00:18Z)
  - [Second batch landed on `main` at 38fc4358 and installed:](yaks-1c89#2026-11-07T00:04:36Z)
- yaks-252c: Hash-checked dok edits (no silent overwrite from a stale read) (hairy -> shaving)
  - [Claim (yakherd herd-upkeep-2, 2026-11-06): worker `doks-1`.](yaks-252c#2026-11-06T23:13:37Z)
- yaks-2e00: refs --incoming on a yak: include doks and session notes (hairy -> shaving)
  - [Claim (yakherd herd-upkeep-2, 2026-11-06): worker `refs-1`, then yaks-525b in...](yaks-2e00#2026-11-06T23:13:39Z)
  - [landed: 487a2508 yaks-2e00: refs --incoming lists doks and session notes that...](yaks-2e00#2026-11-06T23:54:20Z)
- yaks-323b: session list --since <date>: a reflection round's notes, reviewed or not (created)
- yaks-39f9: dok query: contains, compare, not-equal and missing (hairy -> shaving)
  - [Claim (yakherd herd-upkeep-2, 2026-11-06): worker `query-1`, then yaks-6cdd i...](yaks-39f9#2026-11-06T23:13:37Z)
- yaks-525b: Attribute an agent's hand edits committed with yaks commit (hairy -> shaving)
  - [Claim (yakherd herd-upkeep-2, 2026-11-06): worker `refs-1`, after yaks-2e00,...](yaks-525b#2026-11-06T23:13:40Z)
  - [landed: 30653149 yaks-525b: yaks commit --as <actor> stamps hand edits with a...](yaks-525b#2026-11-06T23:54:23Z)
- yaks-6cdd: Write and delete table dok rows by key (hairy -> shaving)
  - [Claim (yakherd herd-upkeep-2, 2026-11-06): worker `query-1`, after yaks-39f9,...](yaks-6cdd#2026-11-06T23:13:38Z)
- yaks-7168: Design: weekly reflection, what moves to yaks (hairy -> shorn)
  - [Proposal: /doks/design/session-reflection.md. One ask follows.](yaks-7168#2026-11-06T23:16:21Z)
  - [ask: Five points on the reflection proposal, each with my lean:](yaks-7168#2026-11-06T23:16:22Z)
  - [pickup: herd-upkeep-2: writing the reflection procedure into yaks-upkeep, the session...](yaks-7168#2026-11-06T23:46:30Z)
  - [Decision (Joel, 2026-11-06, answering the ask): all five leans.](yaks-7168#2026-11-06T23:46:31Z)
  - [Done in aad2377b: the "Reflecting on sessions" section and third recurring ya...](yaks-7168#2026-11-06T23:48:11Z)
- yaks-80bd: Small tool gaps: mark sources, write schemas, backdate imports (hairy -> shaving)
  - [Scope (herd-upkeep-2, 2026-11-06): the third bullet, backdating, is now yaks-...](yaks-80bd#2026-11-06T23:12:41Z)
  - [Claim (yakherd herd-upkeep-2, 2026-11-06): worker `tools-1`.](yaks-80bd#2026-11-06T23:13:38Z)
  - [landed: 22076525 yaks-80bd: add yaks source mark and yaks dok define](yaks-80bd#2026-11-06T23:47:20Z)
- yaks-b6a7: TUI: following a note link lands on its note (hairy -> shaving)
  - [Claim (yakherd herd-upkeep-2, 2026-11-06): worker `tui-1`.](yaks-b6a7#2026-11-06T23:13:40Z)
  - [landed: 6006e665 yaks-b6a7: following a note link in the TUI lands on its note](yaks-b6a7#2026-11-06T23:28:26Z)
- yaks-fcdc: Tools an agent layer needs on top of a farm (hairy -> shorn)
  - [All four children landed on `main` (22076525, 7e8f0eb5, 8988dcc9, 38fc4358)....](yaks-fcdc#2026-11-07T00:04:34Z)
- yaks-ff4c: Remove import-hearth (hairy -> shorn)
  - [Done in 671320f9: `src/hearth_import.rs`, its test, the clap command and ever...](yaks-ff4c#2026-11-06T23:12:41Z)
- created [/doks/design/session-reflection.md](/doks/design/session-reflection.md): Reflecting on sessions

## Lessons

- Parallel workers that each update `docs/cli.md`, the `yaks` skill and `src/main.rs`'s dispatch conflict on landing even with symbol-scoped briefs; the conflicts were all adjacent additions. Landing the workers that share a doc section in one serial arc would avoid it.
- `yaks land` stops a conflicting cherry-pick and leaves recording the landing to you: after `git cherry-pick --continue` it records nothing, so I added the `kind: landed` notes by hand.
- Load-sensitive tests made `yaks verify` flaky for every worker (yaks-06cd; `cargo test --no-fail-fast` gets past it).
- A brief generated before a rename kept the old worker name in the claim notes (`query-1` vs `query-2`); write claim notes after choosing names.
- `cargo fmt -- <files>` still reformats `src/actor.rs` and `tests/commits.rs`; every worker reverted them by hand.
