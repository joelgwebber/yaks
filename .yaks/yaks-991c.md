---
id: yaks-991c
title: Ensure all agent-yak interactions route through the CLI
status: hairy
type: task
priority: 2
created: '2026-10-16T20:08:27Z'
updated: '2026-10-16T20:09:48Z'
labels:
- cli
- agent
---

As I'm working with many concurrent sheds, and dropping answers and feedback into the main checkout's yaks, it's putting a lot of burden on the coordinator and workers to figure out where to find the most recent changes to any given yak. This is compounded when I (constantly) forget to commit them.

If instead we ensure the skills are crystal clear about going through the CLI for everything, then we can also ensure that all calls take the current state of the whole farm into account. Eg, if a worker in a shed needs to know the answer to a question, and the user responded to it in the main shed, we can ensure that it sees the totality of edits across all sheds.

Done well, this should also make it much easier for agents to coordinate implicitly across sheds, without having to be made explicitly aware of one another -- eg, if a worker in another shed discovers something interesting and notes it on yak-aaaa (even if it's not been formally assigned that yak), then its notes would become instantly visible to the worker that actually _is_ shaving that yak.

---
▸ 2026-10-16T20:09:48Z [Joel Webber]
This also relates to yaks-5cea. Hopefully we can unify the CLI/TUI/lib implementations in such a way that these things Just Work.
