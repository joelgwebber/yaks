---
id: yaks-a6c5
title: 'Skills: an eval suite for the bundled yaks skills, run through an outside harness'
status: hairy
type: idea
priority: 3
created: '2026-11-25T04:31:49Z'
updated: '2026-11-25T04:31:49Z'
tags:
- skills
- eval
---

Evals for the bundled `yaks*` skills themselves: a saved set of scenarios, each with the answer or behaviour a good agent should give, rerun through an outside harness whenever a skill changes.

Today a skills change is checked by a cold-read scout (AGENTS.md, "Delegating to cheaper models"). That finds gaps, but it is a one-off: nothing is kept, so a later edit that fixes one case and breaks another goes unseen. Notion's guide makes the same point about factory prompts ([the read](/doks/research/notion-new-shape-for-work.md#ideas-worth-taking)).

## Rough shape, nothing decided

- Scenarios live next to the skills (or in the farm), one per rule that matters: "you hit a schema refusal", "you finished a child yak", "a source changed".
- Two levels: read-only (give a scout only the skill files and ask the scenario as a question; judge the answer) and behavioural (run an agent against a scratch farm and check the farm afterwards).
- Judged by a cheap model against stated criteria, with a person sampling.
- The `skill-creator` skill has eval tooling worth reading first.

Filed for later at Joel's request.
