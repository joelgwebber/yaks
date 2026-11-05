---
platforms: [all]
source: 
follows: 
blocked: 2026-10-04-separate-plans-mode
---
# Repo-qualified citations

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
Several code repos can share one plans repo without false warnings, citing code as repo@sha, and yass archive refuses changes whose done boxes cite missing or unmerged code.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC34–AC39 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `codeRefs` reads `repo@sha` as well as a bare sha (`codeRef`); `yass decisions --cites` matches either by sha
- [x] The code repo's name: `repo:` in its `yass.yaml`, else its `origin` URL's last segment, else its main worktree's folder (`repoName`, `repoFromURL`); `repo:` is a known setting
- [x] `checkCode` checks only this repo's citations and notes how many it left to each other repo; one `citation` check shared with `yass archive`
- [x] `yass archive` refuses done boxes citing a missing or unmerged commit here (`--force` overrides), and lists the citations it couldn't check, with where to
- [x] go test (`TestCodeRefs` with qualified forms, `TestRepoFromURL`); e2e §43 with two code repos cloned from bare origins sharing a plans repo (636 ok); the existing §9 single-repo checks unchanged (AC37); README

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- A citation that names another repo is a note per repo ("2 citation(s) name web, so they're checked from there"), not a warning: it's not wrong, just not ours to check (claude, 2026-10-10)
- The name from a linked worktree is the main worktree's folder, read through git's common dir, so every checkout of a clone agrees (claude, 2026-10-10)
- A citation token with an invalid repo name (`w*b@…`) is ignored rather than guessed at; `@sha` with no name is taken as unqualified (claude, 2026-10-10)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-10 (claude)
- Did: repo-qualified citations, the repo's name, per-repo checks, and `yass archive`'s citation check. `go test`, `tests/e2e.sh` (636 ok), `tests/examples.sh`, `tests/site.sh` pass. The piece is done.
- Next: none here; piece 9 (`yass-trailers`) can start.
