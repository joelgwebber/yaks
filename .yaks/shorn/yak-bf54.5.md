---
id: yak-bf54.5
title: 'Tighten sync field policies: namespaced labels + upstream-wins priority +
  per-field policy table'
type: feature
priority: 2
created: '2026-04-25T17:06:53Z'
updated: '2026-04-25T18:06:52Z'
---

Surfaced during the first dry-run of /yaks:sync against SUBTEXT-301. The skill now has the policy text (commit-this-pass), but the implementation work to make these policies actually enforceable is its own thing.

Three policies to bake in (currently only documented):

1. 'When in doubt, do not sync' principle. Already in SKILL.md hard rules. The agent is expected to follow it; no code enforcement.

2. Namespaced labels. Synced labels round-trip with a tracker prefix: upstream 'bug-fix' <-> local 'jira-bug-fix'. Bare labels (no '<tracker>-' prefix) are local-only; never ferried upstream. Tracker prefix derived from the source URL (jira: *.atlassian.net, linear: *.linear.app, github: *.github.com). This sidesteps two real problems: (a) some trackers require pre-defined label sets so arbitrary local labels would 400 on push; (b) users want to add personal taxonomy ('urgent', 'review-me') without worrying about it leaking upstream. Implementation: pure skill convention initially; could later become a yaklib helper that classifies labels by prefix.

3. Priority is upstream-wins. Never propose pushing yak.priority upstream. On sync, accept upstream's value. Rationale: PMs and Engs re-tune priority frequently upstream, and yak.priority is more of a 'how I plan to prioritize my queue' signal than a shared truth. If users want a stable local priority that doesn't get overwritten, that's a future 'local-only field' mechanism (deferred).

Done when: sync skill applies these rules consistently in dry-run tests; bootstrap scripts adopt the namespacing convention; a test scenario verifies that a bare local label (e.g. 'urgent') is never proposed for upstream ferry.

---
▸ 2026-04-25T18:06:52Z
Done. SKILL.md step 3 rewritten to explicitly state per-field policies (title/desc/status prompt-and-ask; priority silently upstream-wins; labels namespaced with bare = local-only). Rubric table already had the long-form versions. Verification: bare 'urgent' label test passes by reading — synced bucket is empty, no upstream prompt fires; priority drift silently aligned to upstream, no prompt. The skill is now self-consistent on these three policies.
