---
id: yaks-d738
title: 'yaks lanes: show what a lane changed (vs merge-base) and how far behind it is, not a diff against this checkout'
type: bug
priority: 2
created: '2026-10-04T21:00:46Z'
updated: '2026-10-04T21:04:11Z'
parent: yaks-38dc
labels:
- cli
- delta
---

Follow-up to yaks-e545 (landed ddb5a83). `yaks lanes` compares each lane's `.yaks` with THIS checkout's, so a lane that is merely behind shows spurious moves ("yaks-7149 shorn -> hairy" for a lane that never touched 7149; the direction reads backwards). Seen on the real thread: of four lanes, every one showed 2 phantom moves. What a coordinator wants: what did this lane CHANGE, and how far is it behind.

Change: compute the farm delta against the lane's MERGE-BASE with this checkout, not against this checkout.
- base = `git merge-base HEAD <lane HEAD>` run in OUR repo (a Delta clone shares objects with ours via alternates; the base is an ancestor of our HEAD so the object exists here). Materialize base's `.yaks` read-only into a temp dir (`git archive <base> .yaks | tar -x -C <tmp>`, or ls-tree + show) and run the existing `compare_farms(base, lane_working_tree)`: that is exactly the lane's own change, committed and uncommitted. Clean the temp dir. If the base cannot be found (unknown object, no common history) fall back to today's comparison and say `vs this checkout` in the output so nothing is silently wrong.
- Add BEHIND next to AHEAD (`git rev-list --count <lane HEAD>..HEAD`): table shows `+N -M`; JSON gets `behind`. A lane that is behind with no changes of its own prints no farm lines (or `no changes of its own`).
- Keep `needs:` semantics as lane-vs-base. Keep the `shares this farm` / `no farm here` states.
- Still strictly read-only in lanes (`git --no-optional-locks`, nothing written there); the temp dir is ours.
Evidence: tests with temp repos: a lane that is only behind (no farm lines, `-N`); a lane with its own added yak, moved yak, note, needs; a lane both behind and with own changes (only its own shown); the unknown-base fallback; read-only (mtimes unchanged). Real run: `yaks lanes` in the coordinator checkout on this thread must show NO phantom `shorn -> hairy` lines for the stale lanes (3d4gr26p09ng, kb369frnzkrr) and must still show a worker's real changes. Docs parity (docs/cli.md lanes row, README row, SKILL.md row, --help). Out of scope: TUI, discovery rules, other commands.

---
▸ 2026-10-04T21:00:51Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-04T21:00:51Z [delta-lead]
Claim (coordinator delta-lead). Owner: worker lanes-2. Scope: src/lanes.rs, the lanes row in docs/cli.md, README.md, .agents/skills/yaks/SKILL.md, the Lanes help in src/main.rs; nothing else. Judge: coordinator re-runs 'cargo test -p yaks' and 'yaks lanes' in my checkout on this thread (stale lanes must show no phantom moves). Forbidden: writes in other checkouts, TUI, discovery rules, coordination skills.

---
▸ 2026-10-04T21:01:30Z [lanes-2]
Start 2026-10-04T21:01:03Z (date -u: Sun Oct 4 09:01:03 PM UTC 2026). ls .yaks/*/yaks-d738.md -> .yaks/shaving/yaks-d738.md. Built release binary; HEAD 04d7114.

---
▸ 2026-10-04T21:04:11Z [lanes-2]
Done. src/lanes.rs: base_farm() = merge-base (git merge-base HEAD <lane HEAD> in our repo) -> ls-tree -r -z + one cat-file --batch (stdin from a file) into a TempDir (removed on drop) -> store::load -> compare_farms(base, lane tree). Fallback (no merge-base / no committed farm at base / farm outside lane checkout) = old compare vs this checkout, labelled '(vs this checkout)'; JSON farm.vs + farm.base. Lane gets behind + farm_base; table column '+AHEAD -BEHIND' shows '+N -M'; empty delta reads 'no changes of its own'. New tests (fail with base_farm disabled): a_lane_that_is_only_behind_has_no_farm_changes_of_its_own, a_lane_behind_with_its_own_changes_shows_only_its_own (committed + uncommitted own changes, ours-only moves not shown), no_merge_base_falls_back_to_this_checkout_and_says_so, base_temp_dir_is_removed_when_dropped; read-only test still passes. cargo test -p yaks green (382+8+28). Real run in own checkout: stale lanes 3d4g/kb36 no phantom 7149/8265 moves (kb36 'no changes of its own'; 3d4g shows its real f49c move+note, confirmed by git show 6345ab0); t36m shows its real e545 move. Docs: cli.md, README, SKILL.md, --help updated. Finished 2026-10-04T21:04:11Z.

---
▸ 2026-10-04T21:04:11Z [lanes-2]
moved: shaving -> shorn
