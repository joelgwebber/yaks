---
platforms: [all]
source: 
follows: 
blocked:
---
# Pieces own their files

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
Parallel branches on different pieces of one change never touch the same YASS file: each piece's progress stays in its folder, the hook flags commits that reach outside it, and the parent needs no closing commit.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC10, AC11, AC14 in [plan.md](../plan.md)
- [x] The README's section on large changes explains the rule, and that the parent needs no closing commit

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] Hook: with code, a commit that changes a large change's own files (its `plan.md`, `change.md`, or anything outside its pieces, `assets/` included), or files in two of its pieces, gets a heads-up naming them; new files are fine, and commits without code are left alone
- [x] e2e §40: two branches on different pieces rebase onto each other without conflicts (AC10); each heads-up, and the commits that get none (AC11) (593 ok)
- [x] `yass-plan`, `yass-work` and the README say a piece's branch changes only that piece's folder and the parent needs no closing commit (AC14)
- [x] This piece's own progress follows the rule: only this folder changes; AC10, AC11 and AC14 count as delivered through the `Delivers` box
- [x] Jaime reviews the wording (AC14, README)

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- A parent's subfolder that isn't a piece (no `change.md`, like `assets/`) counts as the parent's own files - the e2e caught the first version letting it through silently (claude, 2026-10-09)
- New files are exempt, as everywhere else in the hook: creating a piece or a change alongside code isn't reaching into another piece's progress (claude, 2026-10-09)
- `--range` decides which folders have pieces from the working tree, so it doesn't flag history in changes since archived; the commit hook, which checks the commit being made, isn't affected. Over v0.3.0..main it flags one commit: #62, which ticked the parent's plan.md with code (claude, 2026-10-09)
- A large change without pieces still ticks its plan criteria alongside code; `yass-work` keeps saying so (claude, 2026-10-09)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-09 (claude)
- Did: the hook's heads-up for commits with code that reach past one piece; e2e for parallel pieces; the wording in `yass-plan`, `yass-work` and the README. `go test`, `tests/e2e.sh` (593 ok), `tests/examples.sh`, `tests/site.sh` pass.
- Next: Jaime reviews the wording (AC14, README); then this piece is done.
### 2026-10-09 (claude)
- Did: Jaime reviewed the wording and merged it (#64). The piece is done.
- Next: none here.
