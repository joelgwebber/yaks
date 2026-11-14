---
id: yaks-5f70
title: Session citations in doks read as a bare (session)
status: hairy
type: bug
priority: 2
created: '2026-11-06T21:37:13Z'
updated: '2026-11-14T17:52:05Z'
parent: yaks-3f35
tags:
- skills
- doks
---

From yaks-3f35 (Joel, reading schism's doks in the TUI). The schism review passes wrote 62 citations like `... ([session](/sessions/2025/08/claude-code-2025-08-04-1622-....md)).` into its doks (`doks/howto/dev-workflow.md` has five). In the TUI they render as a stray `(session)` mid-sentence, and following one goes nowhere (yaks-85a6: the TUI can't open sessions).

Cause: the `yaks-upkeep` skill, "Reviewing session notes", step 3 says to "cite the session note in the body (`[session](/sessions/2026/11/...md)`)"; the agent copied the example literally.

## Fix

- Change the example to link words that carry meaning. Lean: link the date the fact was settled, which is what a session citation adds (`renamed to spawns on [2026-03-19](/sessions/...)`), or the phrase the session established; never a bare "session".
- Consider whether citations belong in frontmatter instead (a `sessions:` list, like `sources:`), which keeps prose clean and is queryable.
- yaks-85a6 makes the links openable.
- Offer Joel a pass over schism's 62 citations once the form is settled (a schism-farm chore, not done here).

---
▸ 2026-11-14T11:19:33Z [linkplan-lead]
kind: ask
needs: human
---

asked: needs human

How should a session citation in a dok read? Today the `yaks-upkeep` skill says `[session](/sessions/...)`, which renders as a stray `(session)` mid-sentence.

1. **Link words that carry meaning**: the date the fact was settled (`renamed to spawns on [2026-03-19](/sessions/...)`) or the phrase the session established. Change the skill's example only.
2. **Option 1 plus a `sessions:` frontmatter list**, like `sources:`: keeps prose clean and is queryable by `yaks check` and `dok query`. Larger: schema, `check`, `refs`.
3. **Frontmatter only**, no inline citations.

Lean: 1 now; 2 as its own yak if you want citations queryable. One new fact since this was filed: the sessions and sources paths stay as they are (`/sessions/...`, `/sources/...`), per the decision on yaks-1151, so the example does not change with the layout work. Also: do you want a pass over schism's 62 existing citations once the form is settled (a schism-farm edit)?

---
▸ 2026-11-14T17:48:25Z [sched-1]
kind: answer
needs: agent
---

I'm good with (1) in general. While these repos are still all quite small, though, should we consider updating sessions to include simple slugs in their paths alongside the dates and hashes? They're produced by models in the first place, so that wouldn't be too much of a problem. For the existing ones, we could just leave them as-is, or do one-time surgery to update them in place. I'm pretty sure I'm the only one using them so far, so that shouldn't be much trouble.

---
▸ 2026-11-14T17:51:06Z [linkplan-lead]
kind: pickup
---

picked up

Decision (Joel): option 1. Link words that carry meaning (the date a fact was settled, or the phrase the session established), no `sessions:` frontmatter for now. Existing citations are left as they are. Joel also asked whether session paths should carry a slug: they already do (`sessions/YYYY/MM/<actor>-<day>-<hhmm>-<slug>.md`); the dated-and-hashed directories are sources (`sources/<date>-<sha8>/`), asked separately on the migration yak.

---
▸ 2026-11-14T17:52:05Z [linkplan-lead]

Correction to my pickup note: I said the sources question was "asked separately on the migration yak". It was not; it is now its own yak under yaks-1151 (see the sources-slug idea there). Session paths already carry a slug, so only sources lack one.

Work to do here (own tier, a prose change): in the `yaks-upkeep` skill, "Reviewing session notes" step 3 and the example near it, replace `[session](/sessions/...)` with a citation whose link words carry meaning (the date a fact was settled, or the phrase the session established). Existing citations in other farms stay as they are.
