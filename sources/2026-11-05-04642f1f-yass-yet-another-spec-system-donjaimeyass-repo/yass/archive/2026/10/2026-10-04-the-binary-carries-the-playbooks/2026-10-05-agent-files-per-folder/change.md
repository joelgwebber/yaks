---
platforms: [all]
source: plan.md
follows: 
blocked:
---
# Agent files per folder

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
yass init <folder> --agents puts the AGENTS.md section and the playbooks (and Claude's copies with --claude) in that folder, for a monorepo part with its own agent files; the hook stays at the repo root. Without --agents, yass init <folder> sets up only the folder's planning and touches no AGENTS.md. yass init on a set-up repo or folder writes only what's missing and points at yass upgrade when files are older.

## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Delivers AC31–AC33 in [plan.md](../plan.md)

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `yass init`: the agent files' folder separate from the repo root's hook; `--agents` for a folder
- [x] `yass init <folder>` without `--agents` leaves `AGENTS.md` alone (it used to write the root's)
- [x] `yass init` on a set-up repo or folder: keep what's there, point at `yass upgrade` when files are older
- [x] README CLI reference and usage: `--agents`
- [x] e2e: monorepo folder with and without `--agents`, `--claude` in a folder, re-running init

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- The version comparison (`golang.org/x/mod/semver`) and the stamp reader arrive here, not in piece 3: AC33's "files older than the binary" needs them. `x/mod` v0.27.0 is pinned because later versions require Go 1.26; it needs Go 1.23, under the 1.24 floor piece 3 sets (claude)
- `yass init` now keeps an existing YASS section in `AGENTS.md` instead of rewriting it on every run: setting up writes what's missing (R16), and updating is `yass upgrade`'s job (claude)
- Known gap until piece 3: `install.sh --upgrade` still copies the playbooks and the hook, but the `AGENTS.md` section it used to refresh through `yass init` now stays as it is. `yass upgrade` (piece 3) closes it, and piece 4 points `--upgrade` there; these branches merge together (claude)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->

### 2026-10-05 (claude)
- Did: `yass init <folder> --agents` puts the `AGENTS.md` section and playbooks (and Claude's copies and import with `--claude`) in the folder, with the hook only at the repo root; `yass init <folder>` alone is planning only and leaves every `AGENTS.md` alone; re-running `yass init` writes what's missing, keeps the rest, and notes `yass upgrade` when kept files are older or unstamped. Added `readStamp` and `olderThanBinary` (go test). e2e §25 (18 checks); all 327 e2e checks, examples and `tests/site.sh` pass. README CLI reference lists `--agents`.
- Next: Jaime reviews; then piece 3, `yass-upgrade` (its first step, the comparison, is already here).

### 2026-10-05 (claude)
- Did: fixed two e2e §25 checks that failed in CI on this PR (donjaime/yass#5). At this commit `go.mod` says Go 1.23, which builds `yass` without a version (`dev`), so "`yass upgrade` updates them" never appeared; one check also hard-coded a `0.0.1` stamp, which a CI build's `v0.0.0-…` sorts below. Both now use a binary built with a known version (`vbin`, the helper the `yass-upgrade` piece adds). e2e passes with an unversioned build (as CI's Go 1.23 makes) and a versioned one: 327 each.
- Next: rebase the stack onto this, and merge #5 once CI is green.
