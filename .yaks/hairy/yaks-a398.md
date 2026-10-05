---
id: yaks-a398
title: 'land.sh and coordinators: say when the repo is shallow (ancestry and merge-base answers are then lower bounds)'
type: task
priority: 3
created: '2026-10-05T20:57:26Z'
updated: '2026-10-05T20:57:26Z'
parent: yaks-df61
labels:
- delta
- skills
---

From yaks-df61 O58: Delta marks imported thread commits as shallow boundaries, so a coordinator clone can be shallow. In a shallow repo 'git merge-base --is-ancestor' and 'git merge-tree --write-tree' can answer wrongly (a missing parent looks like 'not an ancestor', so land.sh picks cherry-pick, or the computed merge differs). It has not bitten yet. Add to .agents/skills/yaks-coordinating-delta/land.sh: when 'git rev-parse --is-shallow-repository' is true, print once at the top 'note: this repository is shallow (Delta marks imported commits so); ancestry and merge results below are lower bounds; git fetch --unshallow local fixes it' and keep going; selftest scenario for it (depth-1 clone). One line in the Delta skill. Cheap; do with the next skills pass.
