---
id: yaks-991c
title: Ensure all agent-yak interactions route through the CLI
status: dead
type: task
priority: 2
created: '2026-10-16T20:08:27Z'
updated: '2026-10-19T04:26:22Z'
labels:
- cli
- agent
- sheds
---

As I'm working with many concurrent sheds, and dropping answers and feedback into the main checkout's yaks, it's putting a lot of burden on the coordinator and workers to figure out where to find the most recent changes to any given yak. This is compounded when I (constantly) forget to commit them.

If instead we ensure the skills are crystal clear about going through the CLI for everything, then we can also ensure that all calls take the current state of the whole farm into account. Eg, if a worker in a shed needs to know the answer to a question, and the user responded to it in the main shed, we can ensure that it sees the totality of edits across all sheds.

Done well, this should also make it much easier for agents to coordinate implicitly across sheds, without having to be made explicitly aware of one another -- eg, if a worker in another shed discovers something interesting and notes it on yak-aaaa (even if it's not been formally assigned that yak), then its notes would become instantly visible to the worker that actually _is_ shaving that yak.

---
▸ 2026-10-16T20:09:48Z [Joel Webber]

This also relates to yaks-5cea. Hopefully we can unify the CLI/TUI/lib implementations in such a way that these things Just Work.

---
▸ 2026-10-19T04:26:21Z [delta:herd-pass]

Obviated by one farm per machine (yaks-6a34, yaks-73f4): every shed reads and writes the same farm worktree, so there are no per-shed copies to look up, merge, steer or lose; LFS-tracked attachments no longer travel through code merges (the orphan farm branch has no .gitattributes). Slaughtered in the 2026-10-19 herd pass; revive if it recurs under the new model.

---
▸ 2026-10-19T04:26:22Z [delta:herd-pass]

moved: hairy -> dead
