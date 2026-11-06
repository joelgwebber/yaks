---
type: session-review
title: 'Upkeep in yaks: assessment, note links, session notes, yaks-upkeep'
source: session-note
actor: herd-upkeep
person: Joel Webber
host: joel
harness: delta
thread: ksQQKUp3Ux8dTNqgb6cSSWpUMpLOAEX_fZIBxBCA0aLJyrhJPreUvVwLftHG
day: 2026-11-06
reviewed: '2026-11-06T05:08:43Z'
good: 3
---

## Summary

Joel asked where yaks stands on taking over Hearth's knowledge management. The assessment found storage, search, sources and checks covered, but nothing for session notes, session review or the housekeeping procedure. That became the family yaks-1c89, with the session-note design settled on [yaks-bca6](yaks-bca6#2026-11-06T04:56:54Z). This session then built the path to a usable skill on branch `upkeep`: note links ([yaks-c5bd](yaks-c5bd#2026-11-06T04:58:06Z)), `yaks session` ([yaks-17f5](yaks-17f5#2026-11-06T04:58:06Z), [yaks-e58f](yaks-e58f#2026-11-06T04:58:07Z)) and the `yaks-upkeep` skill ([yaks-3fa5](yaks-3fa5#2026-11-06T04:58:07Z)). It ended with a build installed for Joel to try, not yet landed on `main`.

## Turns

- good: assess the yaks skills against Hearth's knowledge-management goals
- good: file the work as a family under yaks-44b3, with the design question as an ask
- good: build it so the skills can be tried; the review and housekeeping procedures are untested on real data

## Discussion

Joel's refinement of the design: a session note cites the yak notes that hold the work instead of restating them, and keeps exposition for what no yak holds. That needed a link to one note, so a note is now addressed by its timestamp, with `-1`, `-2` for later notes of the same second (common: a shear note and its move).

## Saved

- yaks-17f5: Command: write a session note into sessions/ (created, hairy -> shorn)
  - [verify: `cargo test --test sessions && cargo test --bin yaks sessions` -> PAS...](yaks-17f5#2026-11-06T04:57:12Z)
  - [Done on branch `upkeep` (505d2058): `yaks session note|touched|list|promote`...](yaks-17f5#2026-11-06T04:58:06Z)
- yaks-1c89: Upkeep: session notes, session review and housekeeping (Hearth's knowledge procedures in yaks) (created, hairy -> shaving)
  - [Filed the family (herd-upkeep, 2026-11-05):](yaks-1c89#2026-11-06T04:28:55Z)
- yaks-252c: Hash-checked dok edits (no silent overwrite from a stale read) (created)
- yaks-2e00: refs --incoming on a yak: include doks and session notes (created)
- yaks-39f9: dok query: contains, compare, not-equal and missing (created)
- yaks-3fa5: Skill: yaks-upkeep (session notes, session review, housekeeping) (created, hairy -> shorn)
  - [verify: `cargo test --bin yaks skills && cargo test --test init_modes` -> PAS...](yaks-3fa5#2026-11-06T04:57:21Z)
  - [Done on branch `upkeep` (505d2058): the yaks-upkeep skill, bundled.](yaks-3fa5#2026-11-06T04:58:07Z)
- yaks-525b: Attribute an agent's hand edits committed with yaks commit (created)
- yaks-6cdd: Write and delete table dok rows by key (created)
- yaks-7168: Design: weekly reflection, what moves to yaks (created)
- yaks-80bd: Small tool gaps: mark sources, write schemas, backdate imports (created)
- yaks-b6a7: TUI: following a note link lands on its note (created)
- yaks-bca6: Design: session notes in a farm (narrative plus references to yak notes) (created, hairy -> shorn)
  - [Decision (Joel, 2026-11-05, in the assessment thread): a session note referen...](yaks-bca6#2026-11-06T04:28:54Z)
  - [ask: Four details to settle before yaks-c5bd and yaks-17f5 are built; my lean on e...](yaks-bca6#2026-11-06T04:28:54Z-1)
  - [pickup: herd-upkeep: recording the answers and building on them (yaks-c5bd, yaks-17f5).](yaks-bca6#2026-11-06T04:38:03Z)
  - [Decision (Joel, 2026-11-06, answering the ask): all four leans, with one refi...](yaks-bca6#2026-11-06T04:56:54Z)
- yaks-c5bd: Links to one note of a yak (created, hairy -> shorn)
  - [verify: `cargo test --test links && cargo test --bin yaks a_note_link` -> PAS...](yaks-c5bd#2026-11-06T04:57:07Z)
  - [Done on branch `upkeep` (505d2058): a link can name one note of a yak by its...](yaks-c5bd#2026-11-06T04:58:06Z)
- yaks-e58f: Session review mechanics: list unreviewed session notes, stamp promoted: (created, hairy -> shorn)
  - [verify: `cargo test --test sessions` -> PASS (exit 0)](yaks-e58f#2026-11-06T04:57:17Z)
  - [Done on branch `upkeep` (505d2058): `yaks session list --unreviewed` (agents'...](yaks-e58f#2026-11-06T04:58:07Z)
- yaks-fcdc: Tools an agent layer needs on top of a farm (created)
- yaks-ff4c: Remove import-hearth
  - [Joel, 2026-11-05: go ahead. Hearth never built a real data repo beyond tests,...](yaks-ff4c#2026-11-06T04:28:55Z)

## Lessons

- `yaks commit` stamps no actor on hand edits, so an agent's hand-edited dok is missing from `session touched` (yaks-525b).
- The first real `touched` run showed noisy gists (`verify: verify: ...`, raw link targets); fixed in the second commit. Running a new listing on the real farm caught what the tests didn't.
- `cargo fmt -- <files>` still reformatted `src/actor.rs` and `tests/commits.rs`, as AGENTS.md warns; check `git status` after each run.
- `tests/init_modes.rs` `farm_works_from_a_git_worktree...` fails intermittently on `git worktree add` into a temp path; looks environmental, not checked against `main`.
