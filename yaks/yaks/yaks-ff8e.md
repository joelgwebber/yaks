---
id: yaks-ff8e
title: 'Run notes: make recurring and skill-driven jobs reviewable from the farm'
status: hairy
type: idea
priority: 4
created: '2026-11-25T04:31:49Z'
updated: '2026-11-25T04:31:49Z'
tags:
- design
- agent
---

A rough idea, not a design. Notion's guide says a shared agent ("factory") beats a shared skill because someone can watch its runs and improve it; a skill runs in each person's harness, out of sight ([the read](/doks/research/notion-new-shape-for-work.md#ideas-worth-taking)). yaks will not host a model, but the farm could still be the place where runs are seen.

## What it might look like

A recurring yak (`every:`, `--wake agent`) already holds the instruction in its description and the history in its notes ([scheduling](/doks/design/scheduling.md#principles), principle 3). The idea is to make each round's note regular enough to review across rounds and yaks:

- one note per run, `kind: run`, with a few fields: which skill (and its version or commit), the outcome (`ok`, `partial`, `failed`, `skipped`), and a link to the evidence;
- a query over them: the last N runs of one recurring yak, or every failed run this week (it overlaps yaks-4a0d, schemas for note kinds, and the `--meta` query it wants);
- the job's owner reads that list in the weekly upkeep and fixes the skill or the yak's description.

## Open

- Is a convention in the skills enough, with no code at all?
- What counts as a run when an agent follows a skill outside any recurring yak? Possibly nothing: only recurring work gets run notes.
- Haks derives a review's stage from notes in the same way ([haks](/doks/research/haks-orchestration-layer.md#patterns-worth-keeping)).
