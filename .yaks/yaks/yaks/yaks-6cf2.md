---
id: yaks-6cf2
title: 'check: flag doks whose cited code has changed since the pin or last verification'
status: hairy
type: feature
priority: 3
created: '2026-11-08T16:24:07Z'
updated: '2026-11-14T17:51:08Z'
parent: yaks-1c89
depends_on:
- yaks-f505
tags:
- cli
- doks
---

From [the agent-memory research](/doks/research/agent-memory-dreaming.md#3-let-evidence-not-only-the-clock-say-what-to-re-check) (yaks-e4f5): `yaks check` only knows clock staleness (`stale_after:`). A dok that describes code goes wrong when the code moves, and the farm already holds the evidence: `code:` links name the files, `@sha` pins what was read.

Add one model-free finding to `src/check.rs`: a dok that cites `code:<path>` where the path has commits in the code repo since the pinned sha (or, unpinned, since the dok's `verified:` or last commit). Report the dok, the path and the commit count. It never edits and decides nothing about whether the dok is wrong: an agent reads the dok against the code and sets `verified:` (the existing meaning of that field).

Design points to settle in the work: what counts for an unpinned link, whether a count threshold is useful, and what to do when no code checkout is reachable (skip, as `peek` refuses). Docs and the `yaks-research` skill's `yaks check` paragraph change with it.

---
▸ 2026-11-14T11:19:34Z [linkplan-lead]
kind: ask
needs: human
---

asked: needs human

Three design points this yak lists; answers I would build with unless you say otherwise:

1. **Unpinned `code:` link:** count commits touching the path since the dok's `verified:` date, else since the dok's last farm commit (a hand-set `verified:` is the deliberate re-check).
2. **Threshold:** none, report any count above zero, with the number, so the reader decides. A threshold is easy to add later.
3. **No code checkout reachable:** skip the dok with one info line (as `peek` refuses), never a finding.

One new interaction: yaks-1151 will store pinned code links as host URLs (`https://github.com/o/r/blob/<sha>/path#L12`) that yaks parses back into code links. `check` must read both forms, so it should go through the shared resolver (yaks-1151's resolver yak) rather than its own parse. Lean: build it after that yak lands, not before.

Answer "go" to accept all three, or say what to change.

---
▸ 2026-11-14T17:42:32Z [sched-1]
kind: answer
needs: agent
---

Ooh, that's a great idea I hadn't even considered for updating doks. Makes perfect sense for unpinned code links, as they can drift (pinned ones _shouldn't_ in all normal cases that don't involve deep git surgery, right?).

I'm good with all those design choices. On the last bit with pinned code links, see my earlier comment. If we *do* need to handle that parsing case for some reason, then yes -- let's defer it and leave a comment on yaks-1151 or another appropriate yak about unifying the resolution.

---
▸ 2026-11-14T17:51:08Z [linkplan-lead]
kind: pickup
---

picked up

Decision (Joel): accept all three (since `verified:` else the dok's last farm commit; no threshold; no code checkout means skip with an info line). Scope: `code:` links only. A pinned link should not drift in normal use, so only unpinned links count as findings (a pinned one still reports when its path has commits since the pin AND the dok has no `verified:` after them, per the original yak; keep it as the yak says). Host-URL code links are not read here: unifying their resolution is recorded on yaks-1151, and this is built through the resolver once that lands.
