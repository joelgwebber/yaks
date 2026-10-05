---
id: yaks-a398
title: 'land.sh and coordinators: say when the repo is shallow (ancestry and merge-base answers are then lower bounds)'
type: task
priority: 3
created: '2026-10-05T20:57:26Z'
updated: '2026-10-05T22:43:08Z'
parent: yaks-df61
labels:
- delta
- skills
---

From yaks-df61 O58: Delta marks imported thread commits as shallow boundaries, so a coordinator clone can be shallow. In a shallow repo 'git merge-base --is-ancestor' and 'git merge-tree --write-tree' can answer wrongly (a missing parent looks like 'not an ancestor', so land.sh picks cherry-pick, or the computed merge differs). It has not bitten yet. Add to .agents/skills/yaks-coordinating-delta/land.sh: when 'git rev-parse --is-shallow-repository' is true, print once at the top 'note: this repository is shallow (Delta marks imported commits so); ancestry and merge results below are lower bounds; git fetch --unshallow local fixes it' and keep going; selftest scenario for it (depth-1 clone). One line in the Delta skill. Cheap; do with the next skills pass.

---
▸ 2026-10-05T22:40:25Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-05T22:40:25Z [delta-lead]
Claim (coordinator delta-lead): part of the skills pass under yaks-5c9f; I do it myself (land.sh plus its selftest).

---
▸ 2026-10-05T22:43:08Z [delta-lead]
Done (96c7b6a, with the skill sentence in 71fb223).

- `land.sh` prints one note when `git rev-parse --is-shallow-repository` is true: ancestry and merge answers are lower bounds, and `git fetch --unshallow origin` (or `local`) fixes it.
- Found while testing: with a depth-1 fetch the worker commit's parent is cut off, and the script died with "the worker commit has no parent". It now says the repository is shallow and names the fix.
- Selftest scenario 6 uses a real depth-2 clone: the clone is shallow, the note is printed, the dry run still works. Scenario 4 now also asserts there is no note in a full repository. `cargo test` runs the selftest (`delta_land_script_selftest_passes`), green.

---
▸ 2026-10-05T22:43:08Z [delta-lead]
moved: shaving -> shorn
