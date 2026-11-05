---
platforms: [all]
source: 
follows: 
blocked:
---
# Separate plans mode

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
YASS knows when plans live outside the code repo, says so, and notes when the plans repo has uncommitted changes or is behind.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC30–AC33 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `separate.go`: separate when the yass folder's git common dir isn't the code repo's (another repo, or none); a linked worktree counts as the same repo
- [x] `yass status` ends with `plans: separate (<folder>)` per separate folder, and notes uncommitted files there and how far it's behind its upstream as of its last fetch (no fetch); inline plans add nothing
- [x] `yass root -v` prints `inline` or `separate` after each folder; one-letter switches (`-v`) are accepted where a command has one
- [x] e2e §42 (622 ok): inline (and from a linked worktree), another repo, uncommitted files, behind upstream without fetching, no repo; README

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- "Same repo" compares git's common dir, so plans in the main checkout stay inline from a linked worktree, as design §1 says (claude, 2026-10-09)
- The `plans: separate` line comes after the notes, so it's the last line, as AC31 asks; the uncommitted and behind messages are notes, so `--strict` passes (claude, 2026-10-09)
- `-v` needed the parser to take one-letter switches; it does so only for commands that declare that letter, so nothing else changes (claude, 2026-10-09)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-09 (claude)
- Did: inline or separate plans in `yass status` and `yass root -v`, and the notes for a separate plans repo. `go test`, `tests/e2e.sh` (622 ok), `tests/examples.sh`, `tests/site.sh` pass. The piece is done.
- Next: none here; pieces 8 (`repo-qualified-citations`) and 9 (`yass-trailers`) can start.
