---
id: yaks-8aeb
title: Tighten skill guidance for markdown formatting
status: shorn
type: task
priority: 1
created: '2026-10-26T20:52:10Z'
updated: '2026-10-26T21:00:47Z'
labels:
- markdown
- skills
---

We're getting lots of shitty formatting like:

```
G1 and G2 landed (d84aa65, pushed). G1: a join whose row is a negation chain's row, or a family glob whose tokens all are, is listed under that N-question ('the objective proposes 21 of these rows as joins too (1335 pairs)'). To make that possible, class families that cover any token judged name are split into their tokens (two names sharing a prefix are two decisions); families stay for ids/unjudged tokens. G2: V-questions, splits whose children are one tag told apart by unjudged class tokens, grouped by token pair, offering 'surveyor set SITE class TOKEN=variant'. fullstory: joins 161 + 21 under N1 (the top is now id=name, id=description, *__hint, id=adminInactivityTimeout: real 'generic absorbs specific' questions); splits 14 in 8 V-groups (V1 *__default vs *__emphasized covers 5) + 32 S. turbotax 2 V + 12 S; homedepot 1 V + 9 S.
NO FIXTURE for G1 yet: a hand-built template-slot fixture didn't reproduce the chain. In it the template images become ONE kind via the repetition pass (siblings under one frame), and the general kind's base selector does not forgive the specific ones the way fullstory's does; what makes fullstory's chain is forgiveness WITHDRAWN on the real corpus (DateRangeMinimap-style conflicts), which a small fixture doesn't trigger. Left for survey-84d3, whose fixture (wrapper + layer + template in one kind) is the same shape.
Next pattern spotted: fullstory's top joins are now a specific field named by a hand-written id (id=name, id=adminInactivityTimeout) vs the generic SearchInput (64 unrelated inputs): accepting erases the developer's name. That's G3's question (which ids are names and which are instances), so it belongs with survey-7af5.
```

I think the skills/agent are not entirely clear about what markdown variant we're using, and what choices we've made (eg, blank lines between blocks). Let's take a pass over all the markdown choices we've made, and ensure they're captured in the skills, with no more verbosity than necessary to get the point across.

Let's also update AGENTS.md to ensure that any _future_ changes and additions to our markdown rendering are captured in the skills, so they also don't drift.

---
▸ 2026-10-26T20:55:58Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-26T20:56:25Z [delta-lead]

Claim (delta-lead, yakherd): skills prose, done in the coordinating thread together (all three edit .agents/skills/yaks and yaks-working; AGENTS.md says keep skill rewrites in the coordinator). Worker answer-editor-1 (yaks-ce5e, thread ksQQXwyj…, model: profile default, base 2b1f3ba) runs in parallel and touches no skills.

---
▸ 2026-10-26T21:00:47Z [delta-lead]

Done in fb432be (delta-lead), with yaks-8aeb, yaks-df02 and yaks-248d together, since they share the same skill sections.

- `yaks` skill, "Notes and descriptions": the blank-line-between-blocks rule, short blocks, lists for parallel facts, what renders and what doesn't, a wrong/right example, every link form (including linking a yak's absolute `yaks path` from a chat harness), and stdin for any note with a newline, backtick or `$`.
- "Evidence before you shear": evidence by kind of change (web UI with `sightmap browser`, TUI frame, code link, focused log lines), and a short summary-note spec (no bare "done", long write-ups go in an attachment).
- `yaks-working` and the `src/brief.rs` notes and finish lines say the same thing in brief.
- AGENTS.md: a markdown rendering change must update the skill's Notes section and `brief.rs`.

Evidence: a cold-read by Sonnet scout cold-read-1 (scenarios 1-10 answered correctly from the text); its gaps were fixed, except two that already existed before this change, now filed as a new yak. `cargo test`: 708 passed, 0 failed.

---
▸ 2026-10-26T21:00:47Z [delta-lead]

moved: shaving -> shorn
