---
id: yaks-36b6
title: Auto-update installed project-local skills when yaks is upgraded?
type: feature
priority: 3
created: '2026-10-05T21:08:13Z'
updated: '2026-10-05T21:08:13Z'
parent: yaks-b5a0
labels:
- skills
- cli
needs: human
---

From Joel (2026-10-05): it would be nice if installed skills auto-updated when there is a new yaks version. TODAY: user-level skills in ~/.agents/skills already do (docs/skills.md: ordinary yaks commands install what is absent and upgrade what is cleanly stale; never modified/unmanaged/held/source; YAKS_SKILLS_AUTOSYNC=0 disables). yaks-3859 (landed c568e09) deliberately left PROJECT-LOCAL installs (./.agents/skills, now the default) out, because they are tracked files in a working tree. This yak decides whether and how to extend it.

---
▸ 2026-10-05T21:08:13Z [delta-lead]
Design fork (coordinator delta-lead). Options for project-local skills (./.agents/skills in a repo): (A) status quo: never auto-written; 'yaks skills status' shows them stale and 'yaks doctor' prints the advisory; you re-run 'yaks skills install'. (B) AUTO-UPDATE: any yaks command run inside that repo upgrades a cleanly-stale, unedited skill (same rules as the user-level sync, same opt-out) and prints one line saying what it did. Hazards, all seen in our own workflow: it dirties a tracked tree on an ordinary command (git status, an agent's 'git add -A' sweeps it into an unrelated commit); land.sh refuses a landing while the tree holds changes the worker's commit does not explain; 'yaks lanes' dirty counts and lane labels get noise; every Delta/worktree checkout of the same repo would upgrade ITS OWN copy, so N lanes carry N identical diffs that conflict or duplicate at landing. It only fires once per yaks release per checkout, so the noise is rare but it is exactly at the moment (a new release) when lanes are in flight. (C) NOTIFY, not write: the first yaks command per release in a repo with a stale project-local install prints one line ('project skills are from yaks X, this is Y: run yaks skills install'), plus doctor and status. My lean: (C), because an automatic write into a committed tree breaks the property that a worker's lane contains only its own changes; and (B) only behind an explicit opt-in, e.g. 'yaks skills install --auto' recording a marker so only those installs auto-update. Answer A, B, C or an alternative.
