---
id: yaks-b2e9
title: Accept --as on every write (dok new, source add, ...) or narrow the skill's 'global flag' claim
status: hairy
type: bug
priority: 2
created: '2026-11-25T04:22:33Z'
updated: '2026-11-25T04:22:33Z'
tags:
- cli
- skills
- agent
source: 'feedback-to-yaks:fullstorydev/surveyor#2026-11-24: `--as` is documented as a global flag but refused'
---

From fullstorydev/surveyor's feedback dok (2026-11-24 entry). `yaks --as <actor> dok new ...` and `yaks dok new ... --as <actor>` both fail with "unexpected argument '--as'"; only `YAKS_ACTOR` worked.

Reproduced on 0.0.41: `yaks --as bob dok new`, `yaks dok new ... --as bob` and `yaks create x --as bob` all refuse. In `src/main.rs` only `-C` is `global = true`; `--as` is a per-command `as_actor` on some writes (update, ask, answer, moves, commit, ...). The `yaks` skill, "Commands", says "Global flags: `-C <path>` ..., `--as <actor>` on any write".

## Lean

Make `--as` a global flag like `-C` (one resolution through `actor::resolve`), and drop the per-command copies: every write then takes it, which is what the skill already promises and what yaks-5250's agents need when `whoami --set` hasn't reached their checkout. Otherwise, narrow the skill text to the commands that take it.
