---
platforms: [all]
source: 
follows: 
blocked: 2026-10-07-archive-names-only
created: 2026-10-07T17:34:55Z
---
# Keep and evict

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
A soft keep limit per yass folder, notes when it's passed, and yass evict removing oldest whole months with a write-once manifest each, so evicted names still resolve.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC31–AC42, AC48 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `yass.yaml` `archive: { keep: N }` (default 2,000, after the revision), unknown keys under it warn; `Root.keep()`
- [x] `.evicted` manifests read at load: evicted changes resolve by name (`Change.Evicted`), never read from disk
- [x] Notes past `keep` in `yass status` and `yass archive`; `status --archived` lists the tree and summarizes what's evicted
- [x] `yass evict` (`evict.go`): plans every month first and refuses before changing anything (outside git, uncommitted or never-committed months); oldest whole months until at or under `keep`, never this month; `git rm`, write the manifest, stage it, print the commit
- [x] Hook: a deletion passes only when the same commit adds its month's `.evicted` naming it; a new `.evicted` passes, not alongside code
- [x] Docs: `docs/monorepo.md` "Keeping the working tree small" (keep, evict, `.git` and partial clones, scheduled CI recipe); rule 5 in the templates, this repo's `yass/README.md`, `README.md`, `AGENTS.md` section
- [x] go test (`TestArchiveKeep`); e2e section 33 (482 ok): notes, eviction, manifest, hook both ways, shallow clone, refusals, two branches merging, a squash merge
- [x] `tests/bench.sh`: main 834ms, this branch 840ms on the same repo; the noise between runs is larger
- [x] Jaime reviews the docs (AC42)

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- Evicted changes join `r.Archived` as name-only entries marked `Evicted`, rather than a list of their own - every name lookup (follows, blocked, the queue, `yass archive`'s duplicate check, `status --archived <name>`) already walks the archive, so they resolve with no new code paths (claude, 2026-10-08)
- `yass evict` acts on every yass folder past its keep, each with its own commit command - one command for "make the repo fit", and each folder's repo commits its own (claude, 2026-10-08)
- It checks every month it means to evict before deleting any, so a refusal leaves the tree as it was (claude, 2026-10-08)
- Changes still sitting flat in `archive/` count toward `keep` but can't be evicted; `yass evict` says to run `yass upgrade` first when they're what's left (claude, 2026-10-08)
- The CI recipe opens a pull request with `gh` rather than a third-party action, and says that a PR opened with `github.token` doesn't start other workflows (claude, 2026-10-08)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->
### 2026-10-08 (claude)
- Did: built and tested `keep`, `yass evict` and its manifests, the hook's eviction rule, notes and docs. `go test`, `tests/e2e.sh` (482 ok), `tests/examples.sh`, `tests/bench.sh` pass.
- Did: fixed a marking slip from piece 1: AC1's `[x]` had landed on the commented-out example at the top of plan.md's Acceptance, not on AC1 itself.
- Next: Jaime reviews the docs (AC42); then this piece is done and `decisions-from-history` can start.
### 2026-10-08 (claude)
- Did: Jaime lowered the default `keep` to 2,000 after measuring (revision underneath this PR); constant and docs follow.
- Next: Jaime reviews the docs (AC42).
### 2026-10-08 (claude)
- Did: Jaime reviewed the docs and merged it (#44, #45). The piece is done.
- Next: none here.
