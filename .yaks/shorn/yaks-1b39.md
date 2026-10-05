---
id: yaks-1b39
title: 'yaks lanes: ahead/behind are unknown (?) when there is no merge-base'
type: bug
priority: 3
created: '2026-10-05T19:48:22Z'
updated: '2026-10-05T20:01:39Z'
parent: yaks-38dc
labels:
- cli
- delta
---

Seen on Joel's laptop (2026-10-04): the primary checkout was a shallow repository, so git found no merge-base with a lane at 4fc959e and 'yaks lanes' printed +374 -3 (the size of the whole history) and fell back to '(vs this checkout)' for the farm. The fallback is right; the counts are misleading. When no merge-base is found (shallow history, unrelated histories, unknown lane commit) print '?' for AHEAD/BEHIND (JSON null) and keep the existing vs-checkout note. Test: a shallow clone (git clone --depth 1) as the primary with a lane whose HEAD is older than the boundary. Docs parity for the lanes row.

---
▸ 2026-10-05T19:56:24Z [delta-lead]
Finding (coordinator delta-lead, 2026-10-05): the shallow history is not a one-off. Delta's thread sync left BOTH Joel's primary checkout and this thread's laptop clone shallow (boundaries at the thread's own pinned commits: 2c28d23, dab94a1). Fixed by hand with 'git fetch --unshallow' (origin for the primary, local for the clone), one boundary stays (a pinned unreferenced commit). So no merge-base is the normal case on Delta machines, not an edge: print '?' for AHEAD/BEHIND, and when 'git rev-parse --is-shallow-repository' is true in our repo say so once in the output (e.g. 'history is shallow here: run git fetch --unshallow') since that is the actionable fix. Add a test with a depth-1 clone.

---
▸ 2026-10-05T19:56:33Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-05T19:56:33Z [delta-lead]
Claim (coordinator delta-lead). Owner: worker unk-1. Scope: src/lanes.rs (ahead/behind as Option, '?' in the table, null in JSON; the shallow hint), the lanes rows in docs/cli.md, README.md, .agents/skills/yaks/SKILL.md, the Lanes help in src/main.rs. Judge: coordinator re-runs 'cargo test -p yaks' and 'yaks lanes' on this thread. Forbidden: writes in other checkouts, TUI, discovery, coordination skills.

---
▸ 2026-10-05T19:58:58Z [unk-1]
unk-1 start 2026-10-05T19:58:58Z. ls: .yaks/shaving/yaks-1b39.md. Built ./target/release/yaks. Plan: GitState.merge_base (git merge-base HEAD <lane head> in our repo) decides ahead/behind: Option<usize>; Lane.shallow_here (our repo shallow, set in discover) lets render print one trailing note only when some lane has ahead=None; base_farm takes the merge-base instead of recomputing it.

---
▸ 2026-10-05T20:01:39Z [unk-1]
Done. Option<usize> ahead/behind decided by 'git merge-base HEAD <lane head>' in our repo (GitState.merge_base; base_farm now takes that base); '+? -?' / JSON null; one trailing note when our repo is shallow (Lane.shallow_here, set in discover) and a lane has ahead=None; JSON shape unchanged. Tests (src/lanes.rs): a_shallow_primary_cannot_place_a_lane_older_than_its_boundary (depth-1 clone primary, lane at HEAD~2), the_same_repo_unshallowed_shows_numbers_and_no_note, a_shallow_primary_is_silent_when_every_lane_is_comparable, and no_merge_base_falls_back_to_this_checkout_and_says_so extended (orphan => +? -?). With the count change mutated back to the old unwrap_or(0) the shallow and no-merge-base tests FAIL; the other two are negative controls. cargo test -p yaks: 402+8+28 passed. Docs: docs/cli.md, README.md, .agents/skills/yaks/SKILL.md, Lanes --help. Real 'yaks lanes' here shows numbers only (primary was hand-unshallowed), so '?' is evidenced by the tests, not this checkout.

---
▸ 2026-10-05T20:01:39Z [unk-1]
moved: shaving -> shorn
