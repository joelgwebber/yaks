---
id: yaks-d426
title: 'Skills: sync and rounds, so recurring agent yaks run without being asked'
status: hairy
type: task
priority: 3
created: '2026-11-09T15:37:41Z'
updated: '2026-11-09T19:23:26Z'
tags:
- skills
---

From Joel's farm (cowpaths/joel), 2026-11-09: review-watch (a `--wake agent` P1D yak) sat overdue for two days. `yaks tick` had put it in the agent inbox, but the only thing the skills tell an agent to do with that inbox is the narrow "If you were asked to run a round" in yaks-scheduling, so a session asked to "run the upkeep" worked the two upkeep yaks and left review-watch. Joel's question was why it didn't run on its own.

Two skill gaps:

1. **Who runs a round.** yaks-scheduling says "a person kicks off a round by hand", and yaks-upkeep's "Setting it up" ends with "`yaks tick` wakes them", but no skill says *when* an agent should tick and work the agent inbox. Proposal: the `yaks` skill's session-start step says to run `yaks tick` and work every *Awaiting an agent* yak before the person's request (for a session that talks with the person; delegated agents skip it), or names this as the opt-in a repo's AGENTS.md makes. Any request for "upkeep", "the morning tasks" or "the recurring yaks" means a full round.
2. **Sync is contradictory.** The `yaks` skill says "Sync at session start, after landing work, and before switching machines", but yaks-scheduling says "It never syncs: `yaks sync` is the human's call" and watcher rule 6 says "Don't sync". Keep "tick and a watcher don't sync by themselves" (true of the commands), drop "the human's call", and point at the `yaks` skill's cadence: the agent running the round syncs after it.

cowpaths/joel worked around both in its AGENTS.md (joel-0333 in that farm): every session that talks with Joel syncs, ticks and works the agent inbox first.

---
▸ 2026-11-09T19:23:26Z [review-watch-agent]

Joel, 2026-11-09 (in the cowpaths/joel review-watch thread): the new design is meant to support aggressive syncing, so agents should be told to sync as often as they like, not to leave it to the human.

Three places still say otherwise:

- `AGENTS.md` ("Task tracking"): "publishing it (`yaks sync`) is the human's call"
- `.agents/skills/land/SKILL.md` step 3: "Leave `yaks sync` (publishing) to the user"
- `yaks-scheduling`, watcher rule 6 and "It never syncs": right for the `tick` command and a watcher script (neither should sync on its own), but it reads as a ban on the agent doing the round syncing afterward. Say so.

The `yaks` skill ("Sync at session start, after landing work...") and `yaks-coordinating` already say to sync.
