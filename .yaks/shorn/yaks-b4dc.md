---
id: yaks-b4dc
title: 'discover: walk-up crosses the git top-level and finds another checkout''s farm (Delta private mode)'
type: bug
priority: 2
created: '2026-10-03T20:39:31Z'
updated: '2026-10-04T05:14:50Z'
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

---
▸ 2026-10-04T05:11:28Z [delta-lead]
TRIAL 6 (yaks-fe33): worker 't6-bound', SHORT brief, loads yaks-working itself. Claim committed. DECIDED (Joel, see his note above): discovery stops at the git top-level unless a .yaks pointer file or $YAKS_DIR names the farm. Spec: store::discover walks up as today and a .yaks dir, pointer file or symlink at any level still wins; NEW: (1) if $YAKS_DIR is set it is used first (the .yaks dir itself, a dir containing one, or a pointer file; relative paths from the cwd); (2) when the walk reaches a directory that contains .git (dir or file) and no .yaks, STOP with an error naming that git top-level and the three fixes (yaks init, a .yaks pointer file, YAKS_DIR); (3) outside any git repository the old walk to the filesystem root is unchanged. Consequence to document: an in-tree git worktree or a Delta checkout of a PRIVATE farm no longer finds the live farm by accident; it needs a pointer or YAKS_DIR (yaks init --mode private writing the pointer is the separate yak yaks-814a). Scope: src/store.rs (discover and its tests), docs that describe discovery, AGENTS.md, README, and the discovery/walk-up wording in the yaks, yaks-coordinating-private, -worktrees and -delta skills. Evidence: gate cargo test -p yaks; a test per rule; a transcript on a scratch layout that mimics Delta (outer repo with a private .yaks, a nested checkout with its own .git and no .yaks): error without YAKS_DIR, works with it. Ask about anything that is a real fork (for example whether a pointer file may be given to YAKS_DIR). Judge: coordinator re-runs gate and transcript.

![discover-bound-transcript](artifacts/yaks-b4dc/discover-bound-transcript.txt)

---
▸ 2026-10-04T05:14:50Z [t6-bound]
Implemented Joel's design in store::discover (+ discover_with taking YAKS_DIR as a param): YAKS_DIR first (the .yaks dir, a dir containing one, or a pointer file; relative from cwd; empty=unset; bad value=error, no fall-through); walk stops at the first dir with .git (dir or file) and no .yaks, with an error naming that top-level + yaks init / pointer file / YAKS_DIR; outside git the walk is unchanged. A pointer file is accepted by YAKS_DIR (spec said so; herd: honoured). Tests: 8 in store::discover_tests, one per rule. Gate: cargo test -p yaks = 359+28 passed incl. skills guard. Transcript on a Delta-like scratch layout attached (error without YAKS_DIR, success with it, relative, pointer, bad value). Docs/skills updated: README, docs/cli.md (new 'Which farm a command uses'), docs/README.md, AGENTS.md, skills yaks, -coordinating-private/-worktrees/-delta. Note: in-tree git worktrees of a PRIVATE farm now need YAKS_DIR/pointer too.
