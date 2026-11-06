---
id: yaks-3fa5
title: 'Skill: yaks-upkeep (session notes, session review, housekeeping)'
status: shaving
type: task
priority: 2
created: '2026-11-06T04:28:30Z'
updated: '2026-11-06T04:58:07Z'
parent: yaks-1c89
depends_on:
- yaks-17f5
- yaks-e58f
tags:
- skills
verify: cargo test --bin yaks skills && cargo test --test init_modes
---

A skill for keeping a farm's knowledge in shape, in place of Hearth's `hearth-session-notes` and `hearth-housekeeping` skills and its `session-review` job procedure. Working name `yaks-upkeep`; add it to `BUNDLED` in `src/skills.rs`.

Covers:

- **Session notes:** when to write one (end of a session that did real work; not for a quick question), how (the command), what it holds (per the design yak), and the delegated-session rule: a sub-session writes no note and reports its friction to the agent that started it, which folds it into Lessons.
- **Session review:** take the unreviewed notes, move durable facts into doks (citing the note), stamp them reviewed.
- **Housekeeping:** a pass over `yaks check` findings with Hearth's rules: one kind for look-alike kinds; change a value only when its meaning is clear and only its format is wrong, never choose a date, number or amount; fix a link whose target was clearly renamed; don't delete, don't rewrite prose, don't invent; read before changing. Ends with a Changed / Left alone account as the shear note.
- **Registering them:** each is a recurring yak with `--wake agent` (`yaks-scheduling`), created once per farm, whose description points at this skill. `yaks-research`'s "don't fix other people's doks unasked" needs a line saying a housekeeping pass is the asked-for case.

Cold-read it with a cheap model after writing (AGENTS.md).

---
▸ 2026-11-06T04:57:02Z [herd-upkeep]
kind: move
from: hairy
to: shaving
thread: ksQQKUp3Ux8dTNqgb6cSSWpUMpLOAEX_fZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-06T04:57:21Z [herd-upkeep]
kind: verify
result: pass
---

verify: `cargo test --bin yaks skills && cargo test --test init_modes` -> PASS (exit 0)

---
▸ 2026-11-06T04:58:07Z [herd-upkeep]

Done on branch `upkeep` (505d2058): the [yaks-upkeep skill](code:.agents/skills/yaks-upkeep/SKILL.md@505d2058), bundled.

- **Session notes:** when, the delegated-session rule (workers report friction instead), the sections, Saved from `touched`, note links.
- **Reviewing session notes:** list, keep durable facts in doks citing the note, file work, promote every note, account per note.
- **Housekeeping:** Hearth's rules over `yaks check` (one kind for look-alike kinds, fix only unambiguous formats, never choose a date or number, fix clearly renamed links, verify only what you checked, mark sources only with agreement), and the Changed / Left alone account.
- **Setting it up:** two recurring yaks with `--wake agent`.
- Pointers from `yaks`, `yaks-research` (the housekeeping exception), `yaks-working` (step 8), `yaks-coordinating` (close the session), `yaks-scheduling`, the worker brief's final message, and `docs/skills.md`.

Not done: a cold read by a cheap model (AGENTS.md suggests one) and a run of the review and housekeeping procedures on real farm data; Joel has examples to try.

Evidence: `yaks verify` PASS (the skills guard tests, init installs it).
