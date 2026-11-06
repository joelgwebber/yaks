---
id: yaks-3fa5
title: 'Skill: yaks-upkeep (session notes, session review, housekeeping)'
status: hairy
type: task
priority: 2
created: '2026-11-06T04:28:30Z'
updated: '2026-11-06T04:28:30Z'
parent: yaks-1c89
tags:
- skills
---

A skill for keeping a farm's knowledge in shape, in place of Hearth's `hearth-session-notes` and `hearth-housekeeping` skills and its `session-review` job procedure. Working name `yaks-upkeep`; add it to `BUNDLED` in `src/skills.rs`.

Covers:

- **Session notes:** when to write one (end of a session that did real work; not for a quick question), how (the command), what it holds (per the design yak), and the delegated-session rule: a sub-session writes no note and reports its friction to the agent that started it, which folds it into Lessons.
- **Session review:** take the unreviewed notes, move durable facts into doks (citing the note), stamp them reviewed.
- **Housekeeping:** a pass over `yaks check` findings with Hearth's rules: one kind for look-alike kinds; change a value only when its meaning is clear and only its format is wrong, never choose a date, number or amount; fix a link whose target was clearly renamed; don't delete, don't rewrite prose, don't invent; read before changing. Ends with a Changed / Left alone account as the shear note.
- **Registering them:** each is a recurring yak with `--wake agent` (`yaks-scheduling`), created once per farm, whose description points at this skill. `yaks-research`'s "don't fix other people's doks unasked" needs a line saying a housekeeping pass is the asked-for case.

Cold-read it with a cheap model after writing (AGENTS.md).
