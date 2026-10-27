---
id: yaks-ee0a
title: 'CLI affordances for sheds and threads: what a human and an agent need to ask and do across checkouts'
status: shorn
type: task
priority: 2
created: '2026-10-06T21:05:37Z'
updated: '2026-10-27T01:18:54Z'
parent: yaks-b5a0
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

---
▸ 2026-10-10T18:43:27Z [delta-lead]

Decisions (Joel, 2026-10-10, shed semantics discussion; recorded by the coordinator). They bind yaks-ad6d, yaks-047a, yaks-ed5e, yaks-9277 and the baseline yak below.
1. `cwd` does not matter. Every shed operation gives the same answer from any checkout; the only thing that depends on where you are is the DEFAULT shed when none is named. "Principle of least surprise: no behavior that changes with seemingly irrelevant environmental parameters."
2. A shed's changes are measured from its FORK POINT, always (the oldest entry of its own HEAD reflog), against its working tree. This is "what it thinks it is doing". The viewer-dependent sync rule (`nearer_baseline`, yaks-22f1) goes. Stable baselines are preferred even when that repeats some yak changes across sheds (for example what a worker merged in from its coordinator).
3. A shed's own side is its WORKING TREE (committed, staged and unstaged together): ONE diff, never three. A yak with uncommitted changes may carry a mark ("more active"); nothing more.
4. The primary's baseline is `origin/main`; a clone's view of "the primary's committed state" is `local/main`, then `origin/main` (local only exists in clones, and points at the primary).
5. `landed` and `stale` mean "already in the REFERENCE farm", not in the viewer's: the primary's working tree when the primary is visible, else `origin/main`. Delta-managed repos have no visible primary at all (`local` is a bare repo with no branches), so there the reference is `origin/main`. Detect both reliably or say nothing.
6. No parent detection and no wait for Delta's thread hierarchy: showing the same yak in two sheds is fine, since the workflow norm is that sheds are yak-disjoint. The TUI shows each yak's state per shed that changed it.
7. The current shed is listed as a marked row (yaks-ad6d), with the same fork-point baseline as any other.

---
▸ 2026-10-10T18:56:54Z [delta-lead]

Spawned (base ece5bf8, claude-sonnet-5-5 both, from the spawn confirmations): edit-1 f73004df4f204368 -> yaks-047a; base-1 bed0da16b2714c3b -> yaks-e9bb. Both edit src/sheds.rs in disjoint regions (yak_changes and counts vs baseline code); expect one merge conflict at most in the test module or docs rows.

---
▸ 2026-10-10T18:57:42Z [delta-lead]

edit-1 (f73004df4f204368) ended with no final message and no work (clean clone e4gr4rszmhgz, no commits, no notes, no target/). First death on yaks-047a; respawned as edit-2 on the same brief and claim note (its notes will carry actor edit-2; the claim names edit-1).

---
▸ 2026-10-10T19:05:33Z [delta-lead]

Landed yaks-e9bb (base-1 bed0da16b2714c3b; f03a81c by land.sh, merge of 82dfdab). Review on this machine: full gate 16 suites, 618 passed, 0 failed; the same shed seen from two viewers now gives the same answer (the old binary gave different baselines for my clone from the primary vs from a sibling: [hairy -> shaving] vs [shaving]). Known regression until yaks-ad6d: the primary's row seen from a clone is measured from its own first reflog entry (154 new, 23 moved, 172 notes), as base-1 reported. Spawned here-1 556e48480bff4e61 (claude-sonnet-5-5, base c212def) -> yaks-ad6d. edit-1 died (no work); edit-2 7341c64aec304ab0 is on yaks-047a.

---
▸ 2026-10-10T19:09:48Z [delta-lead]

Landed yaks-047a (edit-2 7341c64aec304ab0, a2c0ed8 by cherry-pick: docs/main.rs conflicts with yaks-e9bb resolved by hand keeping both the fork-point wording and the edited rule). Gate on this machine: 16 suites, 623 passed, 0 failed. Real machine: from the primary, `changes` on this thread lists the rewritten yaks as edited. Waiting on here-1 (yaks-ad6d).

---
▸ 2026-10-10T20:05:22Z [delta-lead]

Landed yaks-ad6d (here-1 556e48480bff4e61, 5c7c95a by cherry-pick; docs/main.rs conflicts with yaks-047a resolved by hand keeping both). The claim commit had staged only the deletion of hairy/yaks-ad6d.md, so the worker's commit left shaving/yaks-ad6d.md behind (doctor: two status dirs); removed it, its two entries are in the shorn file. Gate on this machine: 16 suites, 630 passed, 0 failed. here-1's hung note (a $(cat <<EOF) with backticks ran top and yaks discover) was freed by killing top; the corrupt entry is not in the landed file.

---
▸ 2026-10-11T20:15:15Z [delta-lead]

Spawned (base b721f72, claude-sonnet-5): land-1 8d9d2f1e0c7e4407 -> yaks-ed5e. Resolved its open question first (landed generalizes to cover 'superseded by the reference' via yak_changes(reference, theirs) instead of a second word), and updated it for decision 5 (reference farm = primary's live farm when visible+readable, else origin/main, viewer-independent). yaks-9277 (the state vocabulary) is NOT spawned yet: it has a real design tension (stale vs landed) that needs Joel's call first, per its own 'design first with Joel' instruction.

---
▸ 2026-10-11T20:24:01Z [delta-lead]

Process failure and recovery (2026-10-11, yaks-ed5e): I did all of this turn's early work (resolve note, shave, claim note, spawn-record, yaks-9277 design note) with `cd ~/src/yaks` — the human's PRIMARY checkout — instead of my own clone (mjgxvtvdt1a0). Compounding that, my claim commit (b721f72) repeated the exact bug documented for yaks-ad6d's 5070350: `git add -u` only staged the hairy-dir deletion, never the new shaving file, so the real spec was unreachable by anyone who fetched `local`/`origin` from that commit on.

Consequence: land-1, spawned from MY clone (not the primary, since Delta clones a worker from the spawning thread's own checkout), started from a stale base that had neither the content fix nor the Decision/Claim notes. It correctly stopped and asked (`yaks ask`) instead of guessing.

Fix: rebuilt the full yaks-ed5e.md by hand (local/main's complete Decision+Claim content, with land-1's own shave/ask notes and `needs: human` appended, verified byte-for-byte before committing), merged my clone with local/main (one add/add conflict per touched file, resolved by taking local/main's side plus the hand-merged ed5e file), pushed to local, answered land-1's ask there, and woke it.

Going forward: always `cd` into THIS thread's own clone for coordinator work, never into the primary directly — a coordinator editing the primary bypasses the single-landing-point discipline and is invisible to future spawns until manually resynced, exactly what happened here.

---
▸ 2026-10-11T20:54:29Z [delta-lead]

Landed yaks-ed5e (land-1 8d9d2f1e0c7e4407; fe70e16 by merge after the ed5e-content recovery above). Real-machine: the fixture evidence attached to the yak reproduces the exact 2026-10-10 superseded case (shed: hairy->shaving no notes; primary: hairy->shorn +3 notes) and now reads 'farm: landed (1 yak)'. Worth flagging: the worker found a real bug in my own claim-note spec mid-implementation -- a naive id-membership check on yak_changes(reference, theirs) would ALSO flag a backward-only move (reference behind the shed) as outstanding, since yak_changes's moved field is direction-blind. Fixed with a status_rank comparison (only a FORWARD move counts as outstanding). Gate: 16 suites, 638 passed, 0 failed, rebuilt and re-run on this machine.

---
▸ 2026-10-11T21:58:30Z [delta-lead]

Spawned (base 5557c9e, claude-sonnet-5): live-1 1c43a07092dc4ff4 -> yaks-9277, per Joel's decision A (landed and liveness orthogonal). This time the claim commit and push happened entirely in my own clone (mjgxvtvdt1a0), not the primary -- confirmed reachable via local/main before spawning.

---
▸ 2026-10-11T22:20:18Z [delta-lead]

Landed yaks-9277 (live-1 1c43a07092dc4ff4; a835067 by merge, clean, no unexplained working-tree changes this time). Gate: 16 suites, 644 passed, 0 failed, rebuilt and re-run on this machine. Real-machine: the attached before/after diff surfaced 51 genuinely orphaned Delta pins on this machine (every worker from this whole thread's history that finished or died), none of which showed up anywhere before; active tags on every currently-claimed shed including the primary and my own clone. Checked the liveness fn directly: idle is the default (empty claim or unknown activity), stale only with BOTH a claim and activity older than the 2h STALE_AFTER -- matches the spec exactly. The yaks-38dc family's CLI half (sheds/changes/inbox --sheds/show --sheds/answer @shed, edited, landed, liveness, the current-shed row, origin baseline) is now fully shipped; what remains is the TUI view (yaks-7204/yaks-38dc) and the two smaller loose ends (yaks-6beb: Delta managed-repo discovery, yaks-5138-adjacent follow-ups if any).

---
▸ 2026-10-19T04:27:06Z [delta:herd-pass]

Shorn in the 2026-10-19 herd pass: every question here shipped in v0.0.14 (sheds, changes, inbox --sheds, answer @shed, show --sheds, status). The 'still open' TUI items were built (Sheds tab, yaks-98f1) or are superseded by one farm per machine; what a shed view should show now is yaks-9eaa. Children yaks-8471 and yaks-e475 slaughtered as obviated.

---
▸ 2026-10-19T04:27:08Z [delta:herd-pass]

moved: shaving -> shorn
