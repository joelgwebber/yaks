---
id: yaks-b4dc
title: 'discover: walk-up crosses the git top-level and finds another checkout''s farm (Delta private mode)'
type: bug
priority: 2
created: '2026-10-03T20:39:31Z'
updated: '2026-10-03T22:48:20Z'
parent: yaks-b5a0
labels:
- cli
- delta
---

Found in the Delta run (yaks-df61 O11). store::discover walks up the filesystem with no stop at the enclosing git top-level. Delta checkouts live at <repo>/.delta/worktrees/<id>/<repo>, so a checkout with no .yaks/ of its own (private/out-of-tree mode, or any branch predating the farm) silently reads and WRITES the user's live farm; reproduced in a /tmp tree. The same walk-up is what makes in-tree git worktrees work in private mode (yaks-213b), so the fix must keep that. Options: (a) stop at the nearest git top-level unless a .yaks pointer file or YAKS_DIR names the farm, making private-mode worktrees opt in; (b) keep walk-up but have create/write commands print the resolved farm path when it is outside the current git top-level; (c) leave it and document it. Decide with Joel; (b) is the smallest safe change. Evidence when done: a test that builds the nested layout and asserts the chosen behavior.

---
▸ 2026-10-03T22:28:05Z [delta-lead]
Trial 3 result (log O27-O34): in a real Delta x private run the walk-up worked as the intended mechanism: 4 workers found the live farm with zero setup, claims/notes/asks were visible live to the coordinator with no merge, privacy rules held, and the ask/answer/resume loop ran through the farm alone. So the open question is no longer 'does it work' but 'keep it as a documented feature, bound it, or both'. Options (decide with Joel): (a) keep walk-up; document Delta x private as the live-shared pattern and state the hazard (a checkout missing .yaks silently edits the user's live farm); (b) make it explicit: stop at the git top-level unless a .yaks pointer/YAKS_DIR names the farm, and have 'yaks init --mode private' write that pointer (yaks-814a); (c) (b) plus a doctor/create warning when the resolved farm is outside the current git top-level. Leaning (b) with Delta-aware pointer, since Delta's nested layout makes the walk-up accidental in team mode. Same-yak concurrent writes drop notes (yaks-800d), the one hard limit of the live-shared model.

---
▸ 2026-10-03T22:48:20Z [delta-lead]
Joel (comment, 2026-10-03): agreed on option (b), the explicit pointer, made Delta-aware: discovery stops at the git top-level unless a .yaks pointer or YAKS_DIR names the farm, and 'yaks init --mode private' (yaks-814a) writes that pointer. Recorded here as a note because the decision was made in a thread comment, not through ask/answer (see the next note).

---
▸ 2026-10-03T22:48:20Z [delta-lead]
Process miss (delta-lead): yaks-working says to raise a genuine design fork through ask/answer even with a clear lean, so the decision is an attributed thread on the yak. This was such a fork (where the discovery boundary sits, a semantics change). I wrote the options in a note and asked in chat instead, so the decision lived only in chat and the yak never showed in the human's inbox. Same for yaks-7149 and yaks-2c22, which I have now ask-ed with a stated lean.
