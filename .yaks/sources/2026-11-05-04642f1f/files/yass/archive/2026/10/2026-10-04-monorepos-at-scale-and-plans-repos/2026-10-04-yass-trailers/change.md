---
platforms: [all]
source: 
follows: 
blocked: 2026-10-04-separate-plans-mode, 2026-10-04-repo-qualified-citations, 2026-10-04-the-binary-carries-the-playbooks
---
# yass: trailers

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
With separate plans, every code commit names its change in a `yass:` trailer that the hooks add automatically and check (a warning by default, enforced with `trailers: require`), and yass status lists each change's and each criterion's code from those trailers.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC40–AC52 in [plan.md](../plan.md)
- [x] `yass init` (with `--hooks` to turn them on) installs and upgrades the new `prepare-commit-msg` and `commit-msg` scripts beside `pre-commit`

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] 9a: `trailers.go`: the grammar (`parseTrailer`), targets (active, archived, evicted, `<change>/<piece>`, an unambiguous piece; the closest names suggested), when trailers apply (`plansSeparate`, `trailers:` warn, require, off)
- [x] 9a: `yass hook --prepare-msg` (from `yass use`, the branch name, an earlier commit's trailer; never over yours, never on merges), `--commit-msg` (heads-up, or refuse under `require`), `--range` (reported; fails with `--strict` or `require`; form only without the plans)
- [x] 9a: `yass use [<change>] [--ac] [--clear]`, kept in the worktree's own git folder
- [x] 9a: `prepare-commit-msg` and `commit-msg` scripts, installed by `yass init` and added by `yass upgrade`
- [x] 9a: `git` calls for a particular folder drop the hook's `GIT_DIR` (and `GIT_INDEX_FILE`), so a hook in a linked worktree finds the plans repo rather than the code worktree
- [x] 9a: go test (`TestParseTrailer`, `TestCheckTarget`); e2e §45 (681 ok); husky wired in a scratch repo; `docs/separate-plans.md`, README
- [x] 9a: Jaime reviews the docs (AC49)
- [x] 9b: `yass status <change>` lists its merged commits from trailers, its pieces' included (AC50), and each criterion's, warning about an `ac:` its plan doesn't have (AC51); ten per list, then a count; one `git log --grep` on the merged branch
- [x] 9b: e2e §46 (691 ok); `tests/bench.sh` with 50,000 commits, 5,001 with trailers: `yass status` 77ms, `yass status <change>` 229ms (AC52, < 1s); `docs/separate-plans.md`

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- The commit lists are only in `yass status <change>`, not in plain `yass status`, so the overview pays nothing for them (claude, 2026-10-10)
- 9b's bench caught a regression from piece 7 (#66): deciding inline or separate ran two git processes per yass folder, so `yass status` on the 200-folder bench went from ~0.9s to ~3.2s. A folder inside this repo's work tree (and not a private one) is now inline without asking git, and the code repo's own answer is cached: back to 858ms. Piece 7 and 8 weren't benchmarked when they landed; they should have been (claude, 2026-10-10)
- Lists cap at ten with a count of the rest: the bench's change had 5,001 commits, all printed (claude, 2026-10-10)
- Built in two pull requests: 9a, the trailer itself (AC40–AC49), and 9b, status listing code from trailers (AC50–AC52) - thirteen criteria and two new hooks are more than one reviewable pull request (claude, 2026-10-10)
- Trailers apply only when every yass folder the code repo uses is outside it (or its `yass.yaml` points outside to a folder that isn't checked out, as in CI); a repo with inline plans and a private separate folder, like this one, treats them as off (claude, 2026-10-10)
- The branch-name source matches the whole branch or the part after its last `/`, against change and piece folder names, not the slug without the date - names are what the trailer writes, and a loose match could pick the wrong change (claude, 2026-10-10)
- Found by e2e: hooks in a linked worktree run with `GIT_DIR` set, so every `git -C <plans repo>` call was answered by the code worktree; `git` now drops the repo-pinning variables when it's given a folder, which fixes it for every hook, not just trailers (claude, 2026-10-10)
- AC49's manual check used husky (installed with npm in a scratch repo): `yass use` gave the commit its trailer through `.husky/prepare-commit-msg`, and without it `.husky/commit-msg` gave the heads-up. lefthook and pre-commit weren't available; their configs follow those tools' documented formats, and the doc says husky is the one tested (claude, 2026-10-10)
- Waits for 2026-10-04-the-binary-carries-the-playbooks and ships its hook scripts through `yass init`, which installs and upgrades everything a repo gets once that change lands; this criterion said `install.sh` before (Jaime)
- The trailer hooks skip a `yass` that doesn't know them (`yass hook --help` without the option) - setting up AC56 showed that once `yass upgrade` installs them, anyone with an older `yass` on PATH couldn't commit: it rejected `--prepare-msg` and git aborted (claude, 2026-10-11)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-10 (claude)
- Did: 9a, the `yass:` trailer: grammar, hooks, `yass use`, range checks, scripts, docs. `go test`, `tests/e2e.sh` (681 ok), `tests/examples.sh`, `tests/site.sh` pass.
- Next: 9b: `yass status <change>` lists commits from trailers, per change and per criterion, and the 50,000-commit bench. Jaime reviews the docs (AC49).
### 2026-10-10 (claude)
- Did: 9b: code and criteria from trailers in `yass status <change>`, the 50,000-commit bench, docs; fixed the per-folder git lookup piece 7 added to every `yass status`. `go test`, `tests/e2e.sh` (691 ok), `tests/examples.sh`, `tests/bench.sh` (all targets) pass. Jaime reviewed 9a's docs (#70). The piece is done.
- Next: none here; piece 10 (`check-from-the-plans-repo`) can start.
### 2026-10-11 (claude)
- Did: fixed the trailer hooks blocking commits under an older `yass` (found setting up AC56); e2e §48 with a stand-in older binary, and a manual check under the real 0.4.0 release (702 ok).
- Next: none here.
