---
platforms: [all]
source: Jaime, 2026-10-05: sanity check of CI while reviewing the binary-carries-the-playbooks stack; fix after the stack lands
follows: 
blocked:
archived: 2026-10-05T19:11:56Z
---
# Tighten CI: fewer duplicate runs, pinned release actions, required checks

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
CI stays as safe for outside contributors as it is today (fork PRs get a read-only token and no secrets, first-time contributors need approval, and only `main` and `v*` tags reach anything that writes), and gets cheaper and harder to tamper with. Each commit on a PR is tested once, superseded runs stop when a branch is pushed again, the release cross-build runs only where it's worth its cost, the actions that can write releases are pinned to commit hashes, and `main` requires the `ci-ok` gate before a PR merges.


## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Given a push to a PR's branch, then `ci` runs once for that commit (for the PR), not also for the branch push; pushes to `main` and merge-queue runs still run it — verify: manual: push to a PR branch and list the runs for its commit (`gh run list --commit <sha>`)
- [x] Given a PR branch pushed again while its `ci` run is in progress, then the older run is cancelled; runs on `main` are never cancelled — verify: manual: push twice in a row to a PR branch and check the first run's status
- [x] Given a PR that touches none of `.goreleaser.yaml`, `go.mod`, `go.sum`, `cmd/`, `kit/` or `.github/workflows/`, then the GoReleaser snapshot build is skipped and `ci-ok` still passes; on `main`, and on a PR that touches any of them, it runs — verify: manual: one PR of each kind
- [x] Given `release.yml`, then every third-party action is pinned to a full commit hash (with the version in a comment), and the GoReleaser version is pinned exactly — verify: manual: review; `grep -E 'uses: .*@' .github/workflows/release.yml` shows only 40-character hashes
- [x] Given `ci.yml` and `pages.yml`, then their actions are pinned the same way — verify: manual: review
- [x] Given a PR into `main` whose `ci-ok` hasn't passed, then GitHub won't merge it — verify: manual: a ruleset on `main` requiring `ci-ok`; try merging a PR while CI runs
- [x] Given a new PR branch that changes only plans (for example, an archive move), when it's pushed and its PR opens, then no run tests or builds anything: only the `changes` gate and `ci-ok` run — verify: manual: open a plans-only PR from a new branch and list the jobs of every run for its commit
- [x] Given the change, then no workflow uses `pull_request_target` or `workflow_run`, workflow tokens default to read, and fork PRs from first-time contributors still need approval — verify: manual: `gh api repos/donjaime/yass/actions/permissions/workflow` and `…/fork-pr-contributor-approval`; review

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `ci.yml`: `push` on `main` only; `concurrency` per PR with `cancel-in-progress` (not on `main`)
- [x] `ci.yml`: the snapshot build only on `main`, the merge queue, or PRs touching release inputs (a paths check in the `changes` job); `ci-ok` treats a skipped build as passing
- [x] Pin actions to commit hashes in `release.yml`, `ci.yml` and `pages.yml`; GoReleaser version exact
- [x] Repo settings (Jaime): a ruleset on `main` requiring `ci-ok`; optionally a `v*` tag ruleset, and approval for all outside contributors rather than first-timers only
- [x] Note the required check in `docs/monorepo.md`'s CI recipe if it changes
- [x] Look at the gate's own cost (a runner, `setup-go`, building `yass` for `yass paths --only`, about 10 s plus queueing) and whether a cache or a prebuilt binary is worth it; it's also this repo's dogfood of the `docs/monorepo.md` recipe, so keep that working

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- A separate small change, after the binary-carries-the-playbooks stack lands - found during a CI sanity check while reviewing that stack; nothing in it is a security hole today, so it doesn't block the stack (Jaime)
- Findings it starts from (claude, 2026-10-05): no `pull_request_target` or `workflow_run`; `ci.yml` has `contents: read` and the repo default is read; fork-PR approval is `first_time_contributors`; `release.yml` runs only on `v*` tags and `pages.yml` only on `main`, whose environment allows only `main`; the one event value used in a script goes through `env:`. Costs and gaps: `ci` runs on both `push` (every branch) and `pull_request`, so each PR commit runs twice; no `concurrency`; the six-target snapshot build runs on every PR; actions are pinned by movable tags, which matters most in `release.yml` (`contents: write`, `id-token: write`, `attestations: write`); `main` has no branch protection or ruleset, so `ci-ok` isn't required
- Seen on donjaime/yass#10, an archive-only PR (Jaime, 2026-10-05): its `pull_request` run did the right thing (gate about 10 s, tests and build skipped, `ci-ok` 3 s), but the `push` run for the new branch ran the full tests and the release build, because a branch's first push has no `before` commit to compare with, so the gate falls back to running everything. `push` on `main` only fixes it; the new criterion checks it (claude)
- The test job also cross-compiles `cmd/yass` for windows/amd64, linux/arm64 and darwin/arm64 on Ubuntu: with the snapshot cross-build now limited to release inputs, an ordinary code PR could otherwise break a platform unnoticed until main; the check takes seconds (claude)
- Actions are pinned to the commit their major tag pointed at on 2026-10-05, with the full version in a comment (checkout v4.4.0, setup-go v5.6.0, upload-artifact v4.6.2, goreleaser-action v6.4.0, attest-build-provenance v2.4.0, configure-pages v5.0.0, upload-pages-artifact v3.0.1, deploy-pages v4.0.5), and GoReleaser at v2.18.2; moving to a new version is now a deliberate edit (claude)
- `docs/monorepo.md`'s GitHub recipe gets the same `push`-on-the-default-branch and `concurrency` settings, so teams copying it don't get the duplicate runs; the required check is still `ci-ok` (claude)
- The gate's cost stays as it is: `setup-go` already caches modules and the build, so building `yass` is a few seconds, and the rest is a runner and a full-history checkout, which `--only` needs for its base; a prebuilt release binary would stop this repo's gate from exercising its own `yass paths` (claude)
- Settings, applied in the GitHub UI: a branch ruleset "main: require ci-ok" on the default branch, active, requiring `ci-ok` from GitHub Actions (not "up to date"), with repository admins able to bypass always so direct pushes to main keep working; it also restricts deleting main and blocks force pushes to it (Jaime, 2026-10-05)
- Fork-PR approval stays at first-time contributors: once someone's first run is approved, their later runs go ahead, and Jaime approves with care (Jaime)
- Deferred: the Actions setting "Require actions to be pinned to a full-length commit SHA" and a `v*` tag ruleset, for a later change (Jaime)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->

### 2026-10-05 (claude)
- Did: `ci.yml` runs on pushes to `main` only (plus PRs and the merge queue), with a per-PR `concurrency` group that cancels superseded PR runs and never main's; the snapshot cross-build runs only on main, the queue, or PRs touching `.goreleaser.yaml`, `go.mod`/`go.sum`, `cmd/`, `kit/` or `.github/workflows/` (checked against #15, #16, #7 and #6: false, false, true, true); a cross-compile check in the test job; every action in the three workflows pinned to a commit, GoReleaser exact; `docs/monorepo.md`'s recipe updated to match. YAML parses; the cross-compile loop builds locally; `tests/site.sh` passes.
- Next: Jaime reviews locally. The behavior criteria can only be checked on GitHub once this is pushed (one run per PR commit, cancelled superseded runs, the build skip, a plans-only PR). The `ci-ok` ruleset and the optional settings are Jaime's to approve.

### 2026-10-05 (claude)
- Did: checked the ruleset through the API (id 24523865: active, `~DEFAULT_BRANCH`, `ci-ok` required, admin bypass "always", deletion and non-fast-forward rules). Early evidence on #20: its branch gets only `pull_request` runs, and the rebase's push cancelled the older run. #19's branch still gets `push` runs, since it carries the old `ci.yml` until #20 merges.
- Next: Jaime reviews #19 and #20. To finish, check on GitHub: a PR whose `ci-ok` is still running shows as blocked from merging; after #20 merges, a new plans-only PR runs only the gate; a PR without release inputs skips `build`. Then archive.

### 2026-10-05 (claude)
- Did: #19 and #20 merged (602d44a, 6b4b94e). On #20 before merging: its branch got only `pull_request` runs, never `push` ones, and the rebase's push cancelled the run for the older commit (one run per PR commit; superseded runs cancelled). On main after the merge, `ci` ran everything, `build` included, and `pages` redeployed because `pages.yml` itself changed. Still true: no `pull_request_target` or `workflow_run` in any workflow, workflow tokens default to read, fork-PR approval is `first_time_contributors`.
- Next: this progress PR is plans-only, so it should run only `changes` and `ci-ok`, and stay blocked from merging until `ci-ok` passes; check both on it. The build-skip criterion needs a code PR that touches no release inputs (the next one will do). Then archive.

### 2026-10-05 (claude)
- Did: on #22, this plans-only progress PR from a new branch: one `ci` run, from the PR only (no branch-push run), in which `changes` and `ci-ok` passed and `test` and `build` were skipped; and GitHub reported it `BLOCKED` while `ci-ok` ran, `CLEAN` after, so the ruleset enforces it.
- Next: the last open criterion, a code PR that touches no release inputs skipping `build`, is checked on the next such PR; then archive.

### 2026-10-05 (claude)
- Did: the last criterion, seen on donjaime/yass#24 (a code PR touching `internal/yass/`, `tests/` and docs, no release inputs): one `pull_request` run in which the tests ran on Ubuntu and macOS, cross-compile check included, and `build` was skipped; #23, plans-only, ran only `changes` and `ci-ok`. Every box is done.
- Next: done. Archive it.
