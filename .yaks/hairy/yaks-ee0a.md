---
id: yaks-ee0a
title: 'CLI affordances for lanes and threads: what a human and an agent need to ask and do across checkouts'
type: task
priority: 2
created: '2026-10-06T21:05:37Z'
updated: '2026-10-09T20:12:18Z'
parent: yaks-7204
labels:
- cli
- design
---

Joel (2026-10-06): start by working out the CLI affordances for lanes and threads, because an agent needs the same ones a human does, and use what that teaches to decide the TUI. His use cases, in his words: "what yaks has [thread] created or modified?", "are any threads waiting on answers from me?", "answer the question on [yak] from [thread]: ...". He also wants "dirty yaks" surfaced (he forgets to `git status` and so forgets to commit yaks), and notes that a yak seen in several lanes has N states, which matters for what you are looking at and what you are updating.

This is a PROPOSAL for Joel to react to, not a decision. Nothing here is built.

## Vocabulary
- A **lane** is a checkout: the primary, a `git worktree`, a Delta clone. Its name is a label derived from what already exists: the actors on its own entries (`who`), the Delta thread title, the branch. Selectors accept the label, the directory id or the path.
- A yak has one **state per lane** that carries a copy. `<id>@<lane>` addresses one of them; a bare `<id>` means this checkout, as today.

## What each question needs
| Question | Proposed command | Today |
|---|---|---|
| What lanes exist and what are they doing? | `yaks lanes` | exists; defect yaks-37fa (own changes measured from the viewer's merge-base) |
| What has lane L created or modified? | `yaks changes <lane>` (per yak: created, moved from/to, notes, with the actor; since the lane forked) | the data is in `yaks lanes`, no per-yak CLI |
| Is any thread waiting on me? | `yaks inbox --lanes` (the awaiting-a-human section across lanes, each row with its lane label and the question text read from that lane's working tree) | `yaks inbox` reads only this checkout |
| Answer the question on yak X from lane L | `yaks answer <id>@<lane> --note ...` (writes one appended note and `needs: agent` into THAT lane's copy) | not possible |
| What state is yak X in, everywhere? | `yaks show <id> --lanes` (this checkout's state, then one line per lane copy that differs) | not possible |
| What have I forgotten to commit? | `yaks status` (this checkout's uncommitted yak changes, one line per yak, then `yaks commit`); `yaks status --lanes` for all lanes | `yaks commit` exists, nothing says what it would commit except `--dry-run` |

## Open questions for Joel
1. **Which copy of a yak is authoritative?** Proposal: none; the primary is what a human reads and writes, and a lane's copy is a proposal until it lands. A cross-lane view always says which lane it is showing.
2. **Where does a cross-lane answer live and who commits it?** Proposal: in the asking lane's copy, left uncommitted. The worker commits it with its yak file (the finish rule already stages the yak file), and it reaches the primary when that lane lands. `yaks answer ... --commit` could commit only that file in the lane (`git -C <lane> commit --only -- <path>`), but the lane's agent may be mid-edit or mid-merge, so it should refuse then and say why.
3. **What may write across lanes?** Proposal: only `answer` (and possibly `ask`), each one appended note, nothing else. Everything else stays read-only in other checkouts, as today.
4. **Names.** Use the best name available (thread title, worker name, branch), and show the directory id only as a fallback or with `--ids`.

## Order I would build in
1. Fix the lane baseline (yaks-37fa), so the existing output can be trusted.
2. `yaks status` (small, useful immediately, and the same shape as the worktree problem at one checkout).
3. `yaks changes <lane>` and `yaks inbox --lanes` (read-only).
4. `yaks show <id> --lanes`.
5. `yaks answer <id>@<lane>` last, once the read-only pieces show where the questions are.
Then the TUI: a unified view (dirty marks, lane badges, an inbox that spans lanes) that calls these.

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
