# Large monorepos first, separate plans repo done right: plan
<!-- Intent: how, and how we'll know it's done. With code, only mark boxes here ([/] in progress, [x] done);
     anything else is a plan revision in its own commit. -->

## Approach
The public surface (trailer, `repo:` and `repos:`, `web@sha`, `yass paths`, the branch view, the `yass/` symlink, pieces owning their files, and when plans count as separate) is decided in [design.md](design.md). Everything else builds on the existing pieces: root discovery and `yass.yaml` in `internal/yass/repo.go` and `config.go`, the citation check (`checkCode`, `mergedBranch`), the hook (`internal/yass/hook.go`, `kit/tools/yass/githooks/`), the installer (`install.sh --hooks`), and the end-to-end suite (`tests/e2e.sh`), which gets a numbered section per piece.

**M1, large monorepos.** Mostly documentation, in a new `docs/monorepo.md` linked from the README's Monorepos section, backed by small tooling where a recipe needs it:
- `yass paths` (and `--only <range>`), so any CI can skip plan-only changes with a single gate job.
- A sparse-aware `blocked:` check: `git ls-files` lists entries that aren't checked out, and root discovery already skips folders that aren't on disk.
- Pieces own their files (design.md §5). The hook gives a heads-up when a commit with code touches a parent's files or more than one piece. `yass status <change>` shows parent criteria as *ready to mark* once the pieces delivering them are done, parsed from `Delivers AC…` boxes. The `yass-work` and `yass-plan` skills teach the rule.
- A branch view in `yass status`, on by default (design.md §7): one `git log` over unmerged local and remote-tracking branches, limited to YASS paths and recent branches, showing who is working on which change or piece and how far they are.
- A benchmark script that holds `yass status` and the hook to the PRD's speed targets.

This repo's own `ci.yml` adopts the GitHub recipe, so the recipe is tested every time someone touches a plan here.

**M2, separate plans done correctly.** First, fix path resolution in linked worktrees: when a relative `path:` doesn't exist from the worktree, resolve it from the main worktree, found through `git rev-parse --git-common-dir`. Then add `${VAR:-default}` to path expansion in `config.go`, and the gitignored `yass/` symlink (design.md §1). Then add a notion of "separate", derived as design.md §1 says. It drives:
- a `plans:` line in status;
- notes on the plans repo's uncommitted or behind state (notes, not warnings, so `--strict` is unaffected);
- repo-qualified citations: each repo checks its own and lists the rest as unchecked;
- `yass archive` running the citation check before it moves anything.

**M3, automatic, checked trailers.**
- A `prepare-commit-msg` script adds the `yass:` trailer from `yass use`, the branch name or earlier commits (design.md §2).
- A `commit-msg` script runs `yass hook --commit-msg <file>`, which warns, or rejects under `trailers: require`. Both scripts sit beside `pre-commit` and do nothing when plans are inline.
- `yass hook --range` runs the same check in CI. Where the plans folder isn't available (typical in CI for private plans), it checks only the trailer's grammar and says so.
- `yass status <change>` lists a change's code commits, and each criterion's commits from `ac:`, by reading trailers once from the merged branch, starting at the date of the oldest active change.
- A plans repo's `repos:` map lets status run all of these checks from the plans side.
- Finally, the `yass-work`, `yass-log` and `yass-status` skills, and `docs/skills.md`, get the full separate-plans routine, including `yass use` and trailers in PR bodies.

## Acceptance
### M1
- [x] AC1 (R1) Given a monorepo with `yass/`, `services/payments/yass/` and a `yass.yaml` pointing to `planning/`, when you run `yass paths`, then it prints exactly those yass folders as globs plus each `yass.yaml`, one per line, and no paths for ignored folders or for a plans folder outside the repo — verify: e2e §13
- [x] AC2 (R1) Given a commit range that changes only files under YASS paths, when you run `yass paths --only <range>`, then it exits 0. With any other file in the range it exits 1, and with a range it can't read it exits 2 with a hint to fetch history — verify: e2e §13
- [x] AC3 (R2) Given this repo's `ci.yml` using the documented gate-job recipe, when a PR changes only files under `yass/`, then build and test jobs are skipped, and the required check reports success instead of waiting — verify: manual: open a plan-only PR here and a code PR; record both check runs in the Log
- [-] AC4 (R2) Given `docs/monorepo.md`, then it has a GitHub Actions recipe (gate job, required checks), a Bazel recipe (`.bazelignore` and affected targets, with the yass folders in no target), and a generic recipe using `yass paths --only`, and each recipe's commands are copied from a run that worked — verify: manual: review against the Log of the runs
- [-] AC5 (R3) Given a GitHub merge queue, when a plan-only PR enters it, then the documented `merge_group` setup passes without running the test suite. The docs say how path filters behave for `merge_group` — verify: manual: test on a scratch repo with a merge queue, or say in the Log why that wasn't possible and what the docs rest on
- [x] AC57 (R2) Given `docs/monorepo.md`, then its GitHub Actions and generic recipes are copied from runs that worked (this repo's CI, e2e §13), and its Bazel section says it hasn't been tested end to end yet and where to report results — verify: manual: review against the Log
- [x] AC58 (R3) Given `docs/monorepo.md`, then its merge-queue section says how `merge_group` differs (`branches` filters, no `paths`), cites GitHub's docs, and says it hasn't been tested on a real queue yet and where to report results — verify: manual: review
- [x] AC6 (R4) Given `docs/monorepo.md`, then it shows CODEOWNERS entries routing each team's yass folder to that team, and the root folder to the people who own cross-team changes — verify: manual: review
- [x] AC7 (R5) Given a cone-mode sparse checkout without `services/search/`, when you run `yass status`, then it lists the checked-out yass folders, says nothing about `services/search/yass/`, and exits 0 under `--strict` — verify: e2e §14
- [x] AC8 (R5) Given a checked-out change with `blocked: <a change in a folder that isn't checked out>`, when you run `yass status`, then it says that change is outside this sparse checkout and can't be checked, not that there's no such change, and `--strict` still exits 0 — verify: e2e §14
- [x] AC9 (R6) Given `docs/monorepo.md`, then it has `.gitattributes` LFS rules for images and PDFs under yass folders, and says assets live in the change folder and move with it when archived — verify: manual: review; e2e §14 archives a change with a binary asset and finds it under `archive/`
- [x] AC10 (R7) Given two branches each working on a different piece of the same change, ticking boxes and appending Log entries only in their own piece's folder, when one rebases onto the other, then it finishes without conflicts — verify: e2e §15
- [x] AC11 (R7) Given a commit with code that also changes the parent's `plan.md` or `change.md`, or files in two pieces, when it's committed with the hook on, then the hook gives a heads-up naming the files, and a closing commit with no code that marks parent boxes gets none — verify: e2e §15
- [-] AC12 (R7) Given a parent criterion whose delivering pieces (by their `Delivers AC1, AC3–AC5` boxes) are all done, when you run `yass status <change>`, then it lists that criterion as ready to mark. A criterion with any piece still open isn't listed, and one no piece delivers is left alone — verify: e2e §15; go test for parsing ID lists and ranges
- [-] AC13 (R7) Given a parent whose criteria are all marked done in a closing commit, after its pieces are done, when you run `yass status`, then the change is `done` and `yass archive` accepts it, as today — verify: e2e §15
- [x] AC14 (R7) Given the `yass-plan` and `yass-work` skills, then they say a branch working on a piece changes only that piece's folder, and that the parent needs no closing commit: `yass status` counts the criteria its pieces deliver, and `yass archive` ticks them — verify: manual: review
- [x] AC15 (R8) Given `docs/monorepo.md`, then it gives commands for `git log`, `blame` and diff that leave YASS paths out, built from `yass paths` — verify: manual: run each in this repo
- [x] AC16 (R9) Given a generated repo with 200 yass folders and 2,000 changes, with 100 recently active branches, when you run `yass status` (branch view included), then it finishes in under 1s on a 2024-or-later laptop — verify: `tests/bench.sh`; record the timing in the Log
- [x] AC17 (R9) Given a generated repo of 100,000 files, when you commit a one-file change with the hook on, then the hook adds under 200ms — verify: `tests/bench.sh`; record the timing in the Log

- [x] AC18 (R19) Given a local branch and a pushed branch, each ahead of main with commits under a piece's folder, when you run `yass status`, then each piece lists those branches, with last author, date, the branch's box progress and its latest **Next:** — verify: e2e §21
- [x] AC19 (R19) Given branches that only change code, branches already merged, and branches with no commits in the last 30 days, when you run `yass status`, then none of them are listed — verify: e2e §21
- [x] AC20 (R19) Given a branch last touched 10 days ago, when you run `yass status --since 7d`, then it isn't listed. With `--since <a date before it>`, it is. With `--no-branches`, no branch information is shown and no branch lookup runs — verify: e2e §21
- [x] AC21 (R19) Given any run, then the branch view never fetches, including in a blobless partial clone whose branch-tip files aren't local (it shows those branches without progress instead), and it says when the remote-tracking refs were last fetched, or that there are none. Plain `yass status` with no unmerged branches looks as it does today — verify: e2e §21 (no network: a remote that's a local folder)
- [x] AC22 (R19) Given the benchmark repo with 1,000 recently active remote branches, when you run `yass status`, then it finishes in under 2s — verify: `tests/bench.sh`; record the timing in the Log

### M2
- [x] AC23 (R11) Given a code repo whose `yass.yaml` says `path: ../plans`, when you run `yass root` and `yass status` from a linked worktree elsewhere on disk, or nested under `.claude/worktrees/`, then both resolve to the same plans folder as the main checkout — verify: e2e §16
- [x] AC24 (R11) Given a linked worktree where the plans folder can't be found by either route, when you run `yass status`, then the warning names both places it looked and doesn't suggest `yass init` — verify: e2e §16
- [x] AC25 (R22) Given a code repo whose `yass.yaml` points outside it, when you run `yass init` or `yass status` in the main checkout or a linked worktree, then `yass/` there is a symlink to the resolved plans folder, the clone's `.git/info/exclude` lists it (added once, shared by all worktrees), and `git status` shows nothing new, with `.gitignore` untouched — verify: e2e §16
- [x] AC26 (R22) Given that link, when you run `yass status`, then each change is listed once and there are no warnings about the link. When the plans folder moves and `yass.yaml` is updated, the next `yass` command repoints the link — verify: e2e §16
- [x] AC27 (R22) Given a real `yass/` folder or file already in place, when a `yass` command runs, then it's left alone and YASS warns once. Where symlinks can't be created, YASS skips the link with a note and everything else works — verify: e2e §16; go test with a failing symlink call
- [x] AC28 (R22) Given Claude Code with the code repo as project root, when an agent reads and edits `yass/changes/...` through the link, then the docs say what works and what permission it needs (`--add-dir` or not) — verify: manual: one session; record the result in the Log
- [x] AC29 (R23) Given `path: ${YASS_PLANS:-../plans}`, when `YASS_PLANS` is unset or empty, then `../plans` is used, and when it's set, its value is. `${VAR}` with no default and unset still fails with today's "isn't set" message. `repos:` entries resolve the same way — verify: go test (config_test.go); e2e §16
- [x] AC30 (R10) Given plans inside the repo, when you run `yass status`, then its output is unchanged, and `yass root -v` prints each folder followed by `inline` — verify: e2e §17
- [x] AC31 (R10) Given plans in another git repo or in no repo, when you run `yass status`, then it ends with `plans: separate (<folder>)`, and `yass root -v` prints `separate`. Plain `yass root` output is unchanged — verify: e2e §17
- [x] AC32 (R14) Given separate plans with uncommitted changes in the plans folder, when you run `yass status`, then it notes how many files are uncommitted there, and `--strict` still exits 0 — verify: e2e §17
- [x] AC33 (R14) Given a separate plans repo behind its upstream as of the last fetch, when you run `yass status`, then it notes how far behind it is and suggests pulling, without fetching itself — verify: e2e §17
- [x] AC34 (R12) Given two code repos named `api` and `web` sharing a plans repo, with boxes citing `api@<sha>`, `web@<sha>` and `web@deadbee`, when you run `yass status` in `api`, then it checks only the `api` citations, warns about none of the others, and says how many it left to `web` — verify: e2e §18
- [x] AC35 (R12) Given the same setup, when you run `yass status` in `web`, then it warns that `web@deadbee` isn't a commit there — verify: e2e §18
- [x] AC36 (R12) Given no `repo:` key, then a code repo's name comes from its `origin` URL, and failing that from its main worktree's folder name, the same from every linked worktree. `repo:` overrides both — verify: e2e §18; go test
- [x] AC37 (R12) Given a single code repo with unqualified citations, when you run `yass status`, then it behaves exactly as before, and the existing §9 assertions still pass — verify: e2e §9
- [x] AC38 (R13) Given a change whose `[x]` box cites an unmerged or missing commit in this repo, when you run `yass archive`, then it refuses and names the box and commit. `--force` archives it anyway — verify: e2e §18
- [x] AC39 (R13) Given a change whose citations belong to another repo, when you run `yass archive` here, then it archives it and lists the citations it couldn't check and where to check them — verify: e2e §18

### M3
- [x] AC40 (R15) Given separate plans with `trailers:` unset or `warn` and the hooks on, when a commit has no `yass:` trailer, then it goes through with a heads-up saying how to add one (`yass use <change>`, or `yass: none`) — verify: e2e §19
- [x] AC41 (R15) Given `trailers: require`, when a commit has no `yass:` trailer, or one whose target names no change, then it's rejected. The message suggests the closest change names — verify: e2e §19
- [x] AC42 (R15) Given separate plans, when a trailer names an active change, an archived change, `<change>/<piece>`, a piece alone, or `none`, in any letter case, then it's accepted. A malformed trailer, or one whose piece name is ambiguous, is treated as invalid. An unknown `key:value` gives a warning only — verify: e2e §19; go test for the grammar
- [x] AC43 (R15) Given inline plans, or `trailers: off`, with the hooks on, when you commit, then nothing about trailers is added, checked or printed — verify: e2e §19
- [x] AC44 (R15) Given separate plans and `yass hook --range A..B`, when a non-merge commit in the range lacks a valid trailer, then it's reported, and with `--strict` or `trailers: require` it exits 1. A squash commit whose message carries the trailers passes — verify: e2e §19
- [x] AC45 (R15) Given CI where the plans folder isn't available, when `yass hook --range` runs, then it checks only the trailers' grammar and says the change names weren't checked — verify: e2e §19
- [x] AC46 (R20) Given `yass use <change>` in one worktree, a different change in another, and the hooks on, when each commits with `git commit -m`, then each commit gets `yass: <its change>` in lower case. `yass use` with no argument prints the current change, and `--clear` clears it — verify: e2e §19
- [x] AC47 (R20) Given no `yass use`, when the branch is named after a change (exactly, or with a prefix like `jaime/`), then the trailer comes from the branch name. With neither, it comes from an earlier commit's trailer on the branch. With none of the three, nothing is added and the commit-msg check reports it — verify: e2e §19
- [x] AC48 (R20) Given a commit message that already has a `yass:` trailer, or a merge commit, when the prepare-commit-msg hook runs, then the message is left as it is — verify: e2e §19
- [x] AC49 (R20) Given `docs/` for separate plans, then they show the `trailers:` setting, `yass use`, and calling `yass hook --prepare-msg` and `--commit-msg` from husky, lefthook and pre-commit — verify: manual: wire one manager in a scratch repo, and record it in the Log
- [x] AC50 (R16) Given merged commits carrying `yass: <change>`, when you run `yass status <change>` in that code repo, then it lists them (short SHA, subject) under the change, including after a squash merge, and unmerged commits aren't listed — verify: e2e §20
- [x] AC51 (R21) Given merged commits whose trailers carry `ac:ac1` and `ac:ac1,ac3`, when you run `yass status <change>`, then AC1 lists both commits and AC3 lists one. An `ac:` naming a criterion the change doesn't have gives a warning — verify: e2e §20
- [x] AC52 (R16) Given the benchmark repo plus 50,000 commits, when you run `yass status`, then the trailer lookup keeps it under the AC16 target — verify: `tests/bench.sh`; record the timing in the Log
- [x] AC53 (R17) Given a plans repo whose `yass.yaml` maps `api` and `web` to local checkouts, when you run `yass status --strict` there, then every qualified citation is checked against its repo. An unqualified citation is checked when the map has exactly one repo, and listed as unchecked otherwise — verify: e2e §20
- [x] AC54 (R17) Given a `repos:` entry whose checkout is missing, or whose `${VAR}` has no default and is unset, when you run `yass status` in the plans repo, then it notes that repo's citations as unchecked and doesn't fail `--strict` — verify: e2e §20
- [x] AC55 (R18) Given the `yass-work` skill, then its separate-plans section covers the whole routine. At session start, pull the plans and check their state. Commit code first, with the trailer, then plans. Cross-link the PR and the change. Git latitude is per repo. Never run `yass init` for a missing plans folder. Worktrees. `yass-log` and `yass-status` use trailers to find code — verify: manual: review; e2e §1 still finds the installed skills
- [x] AC56 (R18) Given a fresh agent session in a linked worktree of a code repo with separate plans, when it's pointed at a change with "keep going", then it pulls the plans, builds, commits code with the trailer, and records progress in the plans repo, without running `yass init` — verify: manual: one agent run; summarize it in the Log

## Pieces
In order. Each leaves main green and is one PR. This plan follows its own rule: a piece's branch changes only its own folder. The boxes here count as delivered once their pieces are done (`stacked-prs-and-squash-merges`, #59); `yass archive` ticks them.
1. `2026-10-04-worktrees-find-the-plans` (R11, R22, R23): first, because it breaks agents in the desktop app's worktrees today, and every later piece is tested from worktrees.
2. `2026-10-04-yass-paths-and-ci-recipes` (R1–R3): the gate job, dogfooded in this repo's CI. It also starts `docs/monorepo.md`.
3. `2026-10-04-speed-at-scale` (R9): the benchmark comes early, so later pieces are measured against it rather than tuned afterwards.
4. `2026-10-04-team-scale-docs` (R4–R6, R8): sparse checkouts, CODEOWNERS, LFS and history filters. Needs the doc from piece 2.
5. `2026-10-04-pieces-own-their-files` (R7): before the branch view, so the view reports progress from piece folders only.
6. `2026-10-04-branches-view` (R19): needs piece 5. Benchmarked with piece 3's script.
7. `2026-10-04-separate-plans-mode` (R10, R14): the "separate" notion that pieces 8–10 rely on.
8. `2026-10-04-repo-qualified-citations` (R12, R13): needs piece 7.
9. `2026-10-04-yass-trailers` (R15, R16, R20, R21): needs piece 7. It reads repo names from piece 8.
10. `2026-10-04-check-from-the-plans-repo` (R17): needs pieces 8 and 9.
11. `2026-10-04-agent-guidance-for-separate-plans` (R18): last, so it describes what shipped.

Pieces 4, 6 and 8–11 set `blocked:` on the pieces they need, so `yass status` shows what each is waiting on.

## Validation
- `go vet ./...`, `go test ./...`, `tests/e2e.sh` and `tests/examples.sh` pass on macOS and Linux in CI, including the new sections §13–§21.
- `tests/bench.sh` meets the AC16, AC17, AC22 and AC52 targets, with the timings in the Log.
- This repo's own CI runs the GitHub recipe (AC3), so the recipe is checked on every plan-only PR from then on.
- One manual end-to-end pass of each setup before archiving:
  - a monorepo with three team folders, a sparse checkout, and two agents on different pieces of one change in parallel, watched with the branch view;
  - two code repos sharing a private plans repo, with worktrees, trailers, a squash merge and the plans repo's own `--strict` check (AC56).
- `install.sh --upgrade` on an existing install picks up the new `prepare-commit-msg` and `commit-msg` hooks only with `--hooks`, and changes nothing else.
