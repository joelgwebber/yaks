---
id: yaks-64c4
title: 'Yak archaeology: fast retrieval of past reasoning & provenance stitching'
status: hairy
type: idea
priority: 4
created: '2026-09-09T00:38:19Z'
updated: '2026-10-27T22:29:32Z'
labels:
- cli
- search
---

Umbrella/holding-pen (no concrete design yet) for making it fast for humans AND agents to do yak archaeology: find past reasoning, reconstruct the history/provenance of a change, idea, decision, or architecture, and stitch a narrative across yaks + git. Motivating context: the 'parking' discussion. Dead/ yaks (decided-against or abandoned) carry real research output; today they're RECOVERABLE ('list --all' includes dead, 'show <id>' reads any state, 'revive' resurrects) but not easily DISCOVERABLE (which dead yak holds the reasoning I want?) or STITCHABLE. So dead-invisibility is 'both': a FEATURE that dead is hidden from default 'list'/'next' (out of sight is the point for terminal work), and a latent GAP that there's no search/stitch to make burial non-lossy. Candidate sub-ideas, NONE committed: (1) keyword search -> candidate set across ALL states incl. dead (maybe existing affordances + a --include-dead/--all-aware search); (2) provenance stitching — build on 'yaks commits' (file-follow) plus a 'git blame'-equivalent that maps a code line/decision back to the yak(s) that produced it (long-standing idea; the circular-dependency of stamping commit hashes onto yaks is why we anchor on the file-follow, not a stored hash); (3) one-view lineage of a yak (parent/children/refs/commits/notes). PARKING PRINCIPLE this reframes (candidate skill nudge, not built): dead = terminal archive, not a black hole; record the decision + rationale as a note BEFORE slaughter (a dead yak is only worth keeping if it carries its conclusion — a trigger can't fire from the grave); deferred-but-live work stays hairy (it needs to resurface). Don't rabbit-hole now; this yak just holds the concept so the parked/dead visibility question has a home.

---
▸ 2026-10-12T21:49:52Z [Joel Webber]

moved: shaving -> hairy

---
▸ 2026-10-27T22:29:32Z [delta:Check for `.yaks` file repo redirect sup]

Case study (2026-10-27, for yaks-a99f): Joel asked "do we still support an in-tree `.yaks/`, and did we lose the pointer file in the move to branch farms?" Getting the answer took about 8 tool calls. Notes for whoever builds this:

**What worked**

- Grepping the yak files directly (`grep -liE "farm.branch|retire|deprecat" $(yaks path)/yaks/yaks/*.md`), then printing `status:` and `title:` per hit. That gave a usable candidate list in one pass. `yaks search` would have needed a separate query per term, and it hides dead yaks.
- Shorn umbrella yaks carry the narrative. yaks-6a34's verdict note and yaks-b048's "Done" note listed their children and landing SHAs, so one `cat` of each rebuilt the story.
- **The landing SHA in a note was the key that linked the yak to git.** `git show f615cfb --stat` showed that "drop the in-tree farm" changed skills and docs only, not code. That gap was the real answer, and no yak said it outright.

**What was missing**

- **What a design deferred.** yaks-73f4's design note deferred "named farms per team via a pointer `branch:` key", and no yak was ever filed for it. Only reading the full note surfaced it. A query that finds "Deferred to…" lines with no matching yak would catch this.
- **Scope of a retirement.** Nothing records which surfaces a "retire X" change touched and which it left alone (here: skills yes; `init` defaults, discovery, README no). The SHA plus `--stat` reconstructs it, but only when someone thinks to look.
- **Retirement vs code.** A "retired" concept that is still live in code (`Mode::Team` in `src/init.rs`, `brief.rs`, `preflight.rs`) can only be found by grepping the code against the yak's claim. A check like "this yak says retired; these symbols still exist" would be a useful archaeology primitive.
- `cut -c1-N` on yak files was needed to keep descriptions readable. The one-paragraph body style of older yaks (yaks-64c4 itself) is hard to skim.
