---
id: yaks-f313
title: Some way for agents to get notified of available answers
type: feature
priority: 3
created: '2026-10-04T20:50:20Z'
updated: '2026-10-05T22:44:10Z'
source: cli, agent
---

Maybe it's just a skill thing, but it would be nice if there were a way for an agent to run a CLI command in the background, and have it return when something's ready. They do this sort of thing a lot for other CLI tools.

---
▸ 2026-10-05T22:44:10Z [delta-lead]
Relationship to yaks-c061, and when to do it (coordinator delta-lead, 2026-10-05; Joel asked whether they are redundant and whether to do them sooner).

## Not redundant, but ordered: c061 first
- yaks-c061 is about STATE: after a human answers, there is no findable "answered, pending agent pickup" state (a plain-note reply leaves `needs: human` set; an `answer` on a shorn yak drops out of both `inbox` and `next`). It is what lets anyone, human or agent, find an answer.
- yaks-f313 is about WAITING: a command an agent can run in the background that returns when something is ready. It needs the state c061 defines to wait FOR. Without c061 it would wait on an inbox that does not contain what we need.

## Why f313 is weak in Delta today
- A worker does not block: it runs `yaks ask`, returns, and the coordinator wakes it with a message. The coordinator is the one that needs to notice an answer, and it notices by reading the yaks (this very round: Joel's answers sat uncommitted in his checkout; I found them by looking).
- Background commands do not survive a terminal call. I tested it on this thread: a job started with `&` was gone when the call returned (empty log). So "run it in the background and have it return" is not available in Delta terminals; harnesses with real background tasks could use it.
- What Delta already gives is the other direction: subagent completion messages and `send_agent_message`.

## Recommendation
- Do c061 soon, in two slices. (1) No format change: `yaks inbox` marks a blocked yak whose latest note after the ask is from a different actor ("replied") and lists answered-but-untouched yaks since a time (`--answered --since`). Derived from notes, cheap. (2) The `needs: agent` flip is a design fork for Joel.
- It pairs with the lanes work (yaks-7204): asks and answers that sit in another checkout need the same cross-checkout visibility.
- Leave f313 open and lower priority until c061 exists; when it does, `yaks wait [--for answers]` is a small command for harnesses that can run background tasks. In Delta the coordinator loop (check lanes and inbox, wake the worker) is the supported path.
