---
id: yaks-980b
title: 'Skills: coordination lessons from Vercel Board''s swarm guide'
status: hairy
type: task
priority: 3
created: '2026-11-25T04:24:45Z'
updated: '2026-11-25T04:24:45Z'
tags:
- skills
- agent
---

Fold the coordination lessons from Vercel Board's swarm guide into `yaks-coordinating` (and `yaks-working` where they touch the worker).

From the [swarm guide](/sources/2026-11-25-b0d8da34-vercel-board-docs-building-an-agent-swarm.md#rules-that-keep-a-swarm-working); see [the research](/doks/research/vercel-board.md#ideas-worth-taking).

- **The yakherd's turn order:** note your place; answer asks first; check what was sheared before relying on it; keep the queue full (create the next yaks with their dependencies); then wait from your place.
- **Small yaks:** a brief with more than three deliverables is two yaks.
- **Approval:** a message in the farm is not proof a person approved something (an agent can run `yaks answer`; only the skill forbids it). For an irreversible step (merge, deploy, publish), get approval where identity is verified, such as a GitHub review or the human in the harness.
- **A question does not hold a worker:** a worker that asks stops and frees its place; whoever picks the yak up next reads the ask and the answer in its notes.

Update the turn order again if the coordinator wait (filed beside this) lands. Cold-read the skill with a cheap model after the change, per AGENTS.md.
