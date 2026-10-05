---
id: yaks-75cc
title: 'Skill guidance: treat unexpected yak updates as useful signals from the user'
type: task
priority: 1
created: '2026-09-08T03:26:58Z'
updated: '2026-10-05T22:43:55Z'
labels:
- skills
---

We frequently see crap from agents like:
> I left your live drift (the `d4d2` note, `bd9a`, the `a2ca` move) alone

And they often spend an unreasonable amount of time trying to figure out why the working tree is dirty, or whether they inadvertently did something to a yak.

Instead, it should simply serve as signal that the user is also doing something at the same time. This may or may not be useful signal as to the user's focus, but it shouldn't be a surprise.

---
▸ 2026-10-05T22:43:55Z [delta-lead]
Already in the skills; shearing as done (coordinator delta-lead, 2026-10-05).

Where the guidance lives today:
- `.agents/skills/yaks/SKILL.md` (the default-shipped skill), the live-edits paragraph: "Treat such drift as signal, not noise" (a yak the human just moved, a note they added, tells you what they care about).
- `.agents/skills/yaks-working/SKILL.md`, "While you work": "Changes you did not make are signal, not an error ... Do not investigate, revert or announce it unless it changes your task."
- `.agents/skills/yaks-coordinating/SKILL.md`, section 10 "Drift is signal": a person working beside you, not an error; stage only the yak files you touched.

The mechanics that make the advice cheap to follow also exist now: `yaks commit` commits the farm without touching code, and `land.sh` names exactly which working-tree changes a landing does not explain instead of leaving the agent to investigate.

---
▸ 2026-10-05T22:43:55Z [delta-lead]
moved: hairy -> shorn
