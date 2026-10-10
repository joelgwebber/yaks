---
id: yaks-ee0a
title: 'CLI affordances for sheds and threads: what a human and an agent need to ask and do across checkouts'
type: task
priority: 2
created: '2026-10-06T21:05:37Z'
updated: '2026-10-10T17:18:02Z'
parent: yaks-7204
labels:
- cli
- design
---

Joel (2026-10-06): start by working out the CLI affordances for sheds and threads, because an agent needs the same ones a human does, and use what that teaches to decide the TUI. His use cases, in his words: "what yaks has [thread] created or modified?", "are any threads waiting on answers from me?", "answer the question on [yak] from [thread]: ...". He also wants "dirty yaks" surfaced (he forgets to `git status` and so forgets to commit yaks), and notes that a yak seen in several sheds has N states, which matters for what you are looking at and what you are updating.

## Status (2026-10-10): the proposal is built and released in v0.0.14
Each question below is answered by a shipped command; the decisions Joel made on the open questions are in the notes (a shed's copy is "not yet in the primary"; only `answer` and maybe `ask` write across sheds, enforced in the library; an answer stays uncommitted in the asking shed and its worker commits it; the vocabulary stops at "shed").
- What sheds exist and what are they doing? `yaks sheds`.
- What has shed S created or modified? `yaks changes <shed>`.
- Is any thread waiting on me? `yaks inbox --sheds`.
- Answer the question on yak X from shed S: `yaks answer <id>@<shed> --note ...`.
- What state is yak X in, everywhere? `yaks show <id> --sheds`.
- What have I forgotten to commit? `yaks status` (then `yaks commit`).
- Act on another shed without cd'ing: global `-C <path>` and `--shed <name>` (full access, tighten later only if it proves problematic).
- A shed has a name a human recognizes: the slug of its Delta thread title (yaks-886a).

## Still open
- The shed state vocabulary Joel decided on 2026-10-07 (active, idle, landed, stale, orphan; "its claim disagrees with the primary" a side note): only `landed` is filed (yaks-ed5e); the rest is filed as its own yak below this one.
- The TUI over all of this: a unified view (dirty marks, shed badges, an inbox that spans sheds) that calls these commands. Designed under yaks-7204, which is the next conversation with Joel.
- Showing which thread spawned which: parked (see yaks-7eea) until Delta has CLI affordances for thread hierarchy.

The vocabulary and the original proposal are in the history of this file (git) and the notes below, which keep the word "lane": read it as "shed".

---
▸ 2026-10-06T21:32:29Z [delta-lead]
Terminology decision (Joel, 2026-10-06, via the naming subthread): a "lane" is now a SHED, everywhere, no aliases; `yaks lanes` becomes `yaks sheds`. Text above this note, and all history, keeps the word "lane": read it as "shed". Not decided: "clip" (what a shed has produced) and "barn" (the primary checkout). The rename itself is yaks-dfca; it lands before the baseline fix (yaks-37fa) because both edit the same file.

---
▸ 2026-10-06T21:41:31Z [delta-lead]
Decision (Joel, 2026-10-06): the vocabulary stops at "shed". "Clip" and "barn" are NOT adopted. Where a distinction is needed, add a qualifier instead of a new noun: the main shed (the primary checkout), the yakherd's shed, a worker's shed. Commands and docs use "shed" plus a qualifier; no new word for what a shed has produced either (`yaks changes <shed>` in the proposal above stands).

Also decided in the same discussion: (1) a shed's copy of a yak is simply that shed's version, "not yet in the primary" (no special rules for how a thread works); (2) an answer written across sheds stays UNCOMMITTED in the asking shed's tree and the worker commits it with its own yak file; `--commit` is dropped; (3) only `answer` (and maybe `ask`) may write into another checkout, enforced in the shared library so the CLI and the TUI both get it, while a worker keeps full write access to its own yaks. Build order unchanged: baseline fix (yaks-37fa), then `yaks status`, then the read-only shed commands, then the cross-shed `answer`.

---
▸ 2026-10-07T01:00:44Z [delta-lead]
Decisions (Joel, 2026-10-06, from the abandoned-shed discussion):
1. A shed's state is a small vocabulary (active, idle, landed, stale, orphan); "its claim disagrees with the primary" (its own copy of a yak is `shaving` while the viewer's is `shorn`) is a SIDE NOTE on the row, not a state of its own.
2. The yaks tools never remove or clean up a shed: "there is enough complexity and inference that this could be unpredictable and dangerous". They name what is safe and may print the command; Delta owns its clones, and `git worktree remove` stays the user's call.
3. Liveness starts as the simplest signal: no farm activity for N hours while its own yak is still `shaving`. Revisit only if it misleads.
Context: the two dead workers' sheds (brf-1, brf-2) hold uncommitted drafts and 0 commits of their own while their yak (yaks-2c22) is shorn in the main checkout; a finished worker's shed (sta-1) is clean with its one commit landed.

---
▸ 2026-10-09T20:12:18Z [delta:Delta :Yaks (cont'd)]
Discovery is settled (yaks-c635): from inside any checkout, `yaks sheds` sees every git worktree and every pinned Delta clone, including workers that have not committed yet. The per-shed commands proposed above (changes, inbox --sheds, show --sheds, answer <id>@<shed>) can rely on it. Next is the CLI and skill design pass, then the UI.

---
▸ 2026-10-10T17:18:02Z [delta-lead]
moved: hairy -> shaving
