---
id: yaks-93c5
title: 'Skills: one Families section (the invariant, graduating a child, where a follow-up goes); retire the scattered lines'
status: shorn
type: task
priority: 2
created: '2026-11-01T16:43:15Z'
updated: '2026-11-01T17:31:48Z'
parent: yaks-9b41
tags:
- skills
---

Part of yaks-9b41. Land it last, once the moves it describes exist.

Family rules are spread across the `yaks` skill (Workflow 2 "and its parent if still hairy", Workflow 6's sentences on shearing parents plus the converse added in yaks-62f2), `yaks-working` (steps 2 and 7) and `yaks-coordinating` (Reconcile). Nothing says where a follow-up found while finishing a yak goes, which is how shorn parents ended up over new hairy children.

## Plan

A **Families** section in the `yaks` skill, after Workflow, short because the tool now enforces most of it:

- The invariant: a shorn or dead yak has no hairy or shaving children; `shorn`, `slaughter` and the moves that open a yak refuse to break it, and `doctor` flags it. Shear the parent when its last child is shorn.
- **Graduating:** a parent that is done except for some children: move each out (`yaks reparent <child> --parent <nearest open ancestor it fits>` or `--unparent`) with a note saying where it came from, then shear.
- **Follow-ups found while working:** a child of your yak only if your yak is not done without it (then you finish it first). Otherwise file it under the nearest open ancestor it belongs to, or top-level, and name it in your summary note. If the parent's own description promises that work, it stays open.

Then replace the Workflow lines, `yaks-working` 2 and 7 and coordinating Reconcile with short pointers, and check the result reads as one rule. Docs parity: README and docs if they restate it. Cold-read with a cheap scout afterwards (AGENTS.md).

---
▸ 2026-11-01T17:30:52Z [yakherd]

kind: move
from: hairy
to: shaving
thread: ksQQtfBVdv4DR2eA_txRWzEtg5LOADbbFpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-01T17:31:47Z [yakherd]

kind: landed
sha: ea50761
---

Done in ea50761.

- `yaks` skill: a **Families** section after Workflow: the invariant and which commands keep it, closing a family (one `yaks shorn` in any order; graduate unfinished children with a note), where a follow-up goes (your yak's child only if your yak isn't done without it; else the nearest open ancestor or top level), abandoning (`slaughter --family`, shorn stays shorn). Workflow 2 and 6 shrink to pointers; the commands table says "open descendants".
- `yaks-working` 2 and 7, `yaks-coordinating` Reconcile, and the repo-only `land` skill drop their own family rules and point at the section.
- Net: the step-6 converse added in yaks-62f2 and every "and its parent if hairy" are gone; the rule is stated once, and the tool enforces it.

Evidence: `cargo test --bin yaks skills` (37 pass, BUNDLED guard); grep for "its parent if", "unfinished children", "graduat" across `.agents/skills`, README, docs/skills.md, src/brief.rs finds only the new section and pointers. Not done: the AGENTS.md cold read by a cheap scout (subagents are off unless asked); I re-read the three skills top to bottom myself.

---
▸ 2026-11-01T17:31:48Z [yakherd]

kind: move
from: shaving
to: shorn
thread: ksQQtfBVdv4DR2eA_txRWzEtg5LOADbbFpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
