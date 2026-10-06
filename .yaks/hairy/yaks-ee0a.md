---
id: yaks-ee0a
title: 'CLI affordances for lanes and threads: what a human and an agent need to ask and do across checkouts'
type: task
priority: 2
created: '2026-10-06T21:05:37Z'
updated: '2026-10-06T21:32:29Z'
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
