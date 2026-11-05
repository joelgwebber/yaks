# YASS in a large monorepo

YASS works best with plans next to the code, in the same repo, so progress lands in the same commit as the code it describes. That holds in a monorepo with many teams and expensive CI too. This page shows how to keep plans cheap there. For the folder layout (a `yass/` per team, cross-team changes as links), see [Monorepos in the README](../README.md#monorepos).

## Plan-only commits run no CI

Most YASS edits ride along with code: a box ticked in the same commit as the change it describes. The commits that touch *only* plans are intent changes (a new PRD, a revised plan) and archives. They have nothing to build or test.

`yass paths` prints what's YASS's in this repo, as globs: each yass folder inside the repo, and each `yass.yaml`. Folders a `yass.yaml` ignores aren't listed, and neither is a plans folder outside the repo. With a range, it answers the question CI needs:

```bash
yass paths                       # e.g. yass/**, services/payments/yass/**, yass.yaml
yass paths --only "$BASE...$HEAD"
```

`--only` exits **0** when every file changed in the range is YASS's (skip the build), **1** when anything else changed, or nothing did (run it), and **2** when it can't read the range (usually a shallow clone: fetch history first). Use three dots, so a branch is compared with where it left its base, not with commits that landed there since.

Run the check in one small job at the start, and gate everything else on it. Don't skip whole pipelines with your CI's own path filters unless you've checked what that does to required checks (below).

### GitHub Actions

GitHub treats the two ways of skipping differently ([troubleshooting required status checks](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/collaborating-on-repositories-with-code-quality-features/troubleshooting-required-status-checks)):

- A **workflow** skipped by `paths:`, `branches:` or a commit message leaves its checks **Pending**, and a pull request that requires them can't merge.
- A **job** skipped by `if:` reports **Success**.

So filter with a job, not with `on: paths`, and make one always-running job the check you require:

```yaml
on:
  push:
    branches: [main]   # your default branch; a PR's commits are checked as the PR
  pull_request:
  merge_group:

# A new push to a PR supersedes its checks still running; the default branch and the queue finish theirs.
concurrency:
  group: ci-${{ github.event.pull_request.number || github.ref }}
  cancel-in-progress: ${{ github.event_name == 'pull_request' }}

jobs:
  changes:
    runs-on: ubuntu-latest
    outputs:
      code: ${{ steps.only.outputs.code }}
    steps:
      - uses: actions/checkout@v4
        with: { fetch-depth: 0 }
      - name: Install yass
        run: |
          v=vX.Y.Z   # pin a release that has `yass paths`
          curl -fsSLO "https://github.com/donjaime/yass/releases/download/$v/yass_linux_amd64.tar.gz"
          curl -fsSLO "https://github.com/donjaime/yass/releases/download/$v/checksums.txt"
          grep yass_linux_amd64.tar.gz checksums.txt | sha256sum -c
          tar -xzf yass_linux_amd64.tar.gz && echo "$PWD/yass_linux_amd64" >> "$GITHUB_PATH"
      - id: only
        env:
          BASE: ${{ github.event.pull_request.base.sha || github.event.merge_group.base_sha || github.event.before }}
        run: |
          # Plans only: skip. Anything else, or no usable base (a new branch): run everything.
          if [ -n "$BASE" ] && [ "$BASE" != 0000000000000000000000000000000000000000 ] &&
             yass paths --only "$BASE...$GITHUB_SHA"; then
            echo code=false >> "$GITHUB_OUTPUT"
          else
            echo code=true >> "$GITHUB_OUTPUT"
          fi

  test:
    needs: changes
    if: needs.changes.outputs.code == 'true'
    runs-on: ubuntu-latest
    steps:
      - run: echo "your build and tests"

  # Require this one in branch protection, not the jobs it gates.
  ci-ok:
    if: always()
    needs: [changes, test]
    runs-on: ubuntu-latest
    steps:
      - if: contains(needs.*.result, 'failure') || contains(needs.*.result, 'cancelled')
        run: exit 1
      - run: echo "ok"
```

Limit `push` to your default branch: on any other branch, a first push has no `before` commit to compare with, so the check falls back to running everything, and the PR's own run would check the same commit again. A job that `needs:` a skipped job is skipped too, so a whole chain of jobs behind `test` drops out on a plan-only change, and `ci-ok` still reports. This repo's own [ci.yml](../.github/workflows/ci.yml) works this way, building `yass` from source instead of downloading it.

If you'd rather not install `yass` in CI, paste the output of `yass paths` into a plain diff check, and update it when a team adopts YASS:

```bash
! git diff --name-only "$BASE...$GITHUB_SHA" | grep -qvE '^(yass/|services/payments/yass/|yass\.yaml$|services/[^/]+/yass\.yaml$)'
```

### Merge queues

> **Not tested on a real merge queue yet.** This section follows GitHub's documentation; if you run it, please [tell us how it went](https://github.com/donjaime/yass/issues).

A merge queue runs your checks again on the `merge_group` event. That event takes `branches` filters but no `paths` filters ([events that trigger workflows](https://docs.github.com/en/actions/writing-workflows/choosing-when-your-workflow-runs/events-that-trigger-workflows#merge_group)), so a workflow can't skip itself there by path. The gate job above handles it: on `merge_group` it compares the queue's base with the merge group's commit, so a plan-only pull request still passes through the queue, but without running the build. It still takes its turn in the queue; it just doesn't hold the queue up for long.

### Bazel

> **Not tested end to end yet.** If you try this on a Bazel repo, please [tell us how it went](https://github.com/donjaime/yass/issues).

YASS folders hold Markdown and images, never build files, so:

- **List every yass folder in `.bazelignore`** (the output of `yass paths`, without the `/**`), so `bazel build //...` and `bazel query` never look inside them.
- **Keep them out of targets.** A `glob(["**"])` in a root `BUILD` file (a docs bundle, say) would pull plans into a target, and every plan edit would invalidate it. Exclude them, or glob only what the target needs.
- **Affected-target tools** ([bazel-diff](https://github.com/Tinder/bazel-diff), [target-determinator](https://github.com/bazel-contrib/target-determinator)) then find no affected targets for a plan-only change, so nothing builds. You can also skip running them at all with `yass paths --only` first, the same way as the gate job above.

### Any other CI

Tried YASS with Buck, Pants, Nx, Buildkite, GitLab CI or something else? [Tell us](https://github.com/donjaime/yass/issues) what worked, and we'll list it here.

The same three lines work in Buildkite, Jenkins, GitLab CI or a shell script. Decide on a base (the target branch's merge base for a change under review, the previous commit for a push) and:

```bash
if yass paths --only "$BASE...$HEAD"; then
  echo "plans only: skipping build and tests"
  exit 0
fi
# …the build and tests
```

In Buildkite, run this as the step that uploads the rest of the pipeline, and upload nothing when it exits 0.

## Many teams in one repo

Each team gets its own yass folder (`yass init services/search`), so its changes, archive and `keep:` setting are its own. `yass status` shows them all; `yass new` puts a change in the one nearest where you're standing.

### Who reviews plans: CODEOWNERS

Route each yass folder's plans to the team that owns it, and the root folder (cross-team changes) to whoever owns those:

```
# .github/CODEOWNERS (or CODEOWNERS at the root; GitLab uses the same file)
/yass/                   @acme/architecture
/services/search/yass/   @acme/search
/services/web/yass/      @acme/web
/yass.yaml               @acme/platform
```

A pull request that changes a team's PRD, plan or progress then asks that team to review it, and a cross-team change's plan goes to the people who coordinate across teams. Keep these lines below any broader rule for the same paths: the last matching line wins.

### Seeing who's working on what

`yass status` lists, under each change and piece, the unmerged branches with recent commits to its folder ([README](../README.md#the-cli)). It reads what's already in your clone, in a few git commands whatever the number of branches: the branch refs, one `git log` over them limited to the yass folders, and the changed files at their tips. It never fetches, so it's as fresh as your last `git fetch`, which its last line says; a branch deleted on the server stays listed until `git fetch --prune`. In a blobless partial clone it never downloads a branch's files to read its progress: such branches show without it.

On a large repo, keep git recent (2.44 or later, so a partial clone is never asked to download) and let it keep a commit graph with changed-path filters, which makes walking many branches by path fast:

```bash
git config fetch.writeCommitGraph true             # refresh the commit graph on every fetch
git commit-graph write --reachable --changed-paths # or build it once now
```

With 1,000 recently active remote branches on a repo of 200 yass folders and 2,000 changes, `yass status` takes about 1.5s on a 2024 laptop; with 100, the view adds next to nothing. `--no-branches` leaves it out, for scripts.

### Sparse checkouts

In a sparse checkout, `yass status` reports on the yass folders you have checked out and says nothing about the rest. A change that names one outside your checkout, in `blocked:` or `follows:`, gets a note saying where it is and that it can't be checked from here, not a warning, so `yass status --strict` still passes. To see it, add its folder (`git sparse-checkout add services/search`).

### Images, PDFs and other binaries

A mockup, a screenshot or a spec PDF belongs in the change folder it's about (`yass/changes/<change>/assets/`), and moves with the change when it's archived, so the archive keeps a change's evidence with its decisions. Store them with [Git LFS](https://git-lfs.com), so clones and CI fetch them only when they're read:

```
# .gitattributes
**/yass/**/*.png  filter=lfs diff=lfs merge=lfs -text
**/yass/**/*.jpg  filter=lfs diff=lfs merge=lfs -text
**/yass/**/*.gif  filter=lfs diff=lfs merge=lfs -text
**/yass/**/*.pdf  filter=lfs diff=lfs merge=lfs -text
```

### History without the plans

Plans live next to the code, so `git log` and `git diff` show both. `yass paths` prints the paths that are YASS's; turned into exclusions, they leave the plans out:

```bash
yass paths | sed 's/^/:(exclude,glob)/' | xargs git log --oneline -- .        # commits that touch code
yass paths | sed 's/^/:(exclude,glob)/' | xargs git diff --stat main... -- .  # a branch's code changes
```

As an alias, so it's `git codelog` from then on (any `git log` options pass through):

```bash
git config alias.codelog '!f() { yass paths | sed "s/^/:(exclude,glob)/" | xargs git log "$@" -- .; }; f'
```

`git blame` needs nothing: a commit that touches only plans never changes a line of code, so it never shows up in a code file's blame. The other way around, `git log -- yass/` (or `yass decisions`) is the plans' own history.

## Squash merges

A squash merge folds a pull request into one commit on main. YASS's rules are about commits: intent (a PRD, a design, plan text, a change's Goal or Acceptance), an archive move and a `queue.md` reorder each land apart from code, and progress (boxes, Log, Decisions) lands with the code it describes. With squash merges, "apart from code" means a pull request apart from code:

- **A plan revision, an archive move or a queue reorder goes in a pull request with no code in it.** It can share that pull request with other intent, progress or an archive: a PRD revision with its plan, or a change's last progress with its archive move. These are the pull requests [the recipes above](#plan-only-commits-run-no-ci) let skip build and tests.
- **Progress goes in the pull request of the code it describes,** as it would in a single commit.

The hook checks commit by commit, so it can't see a pull request that will squash an intent commit and a code commit together; keeping them apart is up to whoever opens it. How you get there (separate branches, a stack of pull requests, a stacking tool) is your team's call.

## Big archives

Each yass folder's `archive/` keeps finished changes in `<YYYY>/<MM>/` folders, by the month `yass archive` moved them, so no one folder grows without bound and GitHub can still list them all (it shows up to 1,000 entries in a folder). A team archiving more than about 1,000 changes a month in one yass folder should split it into team folders (`yass init <team folder>`): each team's archive then stays small on its own, and the repo's scales with them. A month that occasionally passes 1,000 is fine.

### Keeping the working tree small

A yass folder keeps up to 2,000 archived changes in its working tree: each one adds a little to every `yass status` (git walks its folders, about +170ms at 2,000) and about 10KB to every checkout, and a monorepo's yass folders add up. That's about a year for a 10-person team. Past that, `yass status` and `yass archive` say so, and `yass evict` moves its oldest whole months out: it deletes each month's folder and leaves `archive/<YYYY>/<MM>.evicted`, a short file naming the month's changes and the commit that still has them. Commit that on its own. Nothing is lost: evicted changes still count for `follows:` and `blocked:`, `yass status --archived` lists how many were evicted and from which months, `yass decisions` reads their decisions back from git when a query reaches them (`--since`, `--change`, or `--cites <sha>`), and `git show <commit>:<path>/<name>/change.md` reads one by hand. A shallow clone lacks the commits; `yass decisions` says so, and `git fetch --unshallow` gets them. The limit is soft: going past it changes nothing until someone runs `yass evict`. Set it per yass folder in its `yass.yaml`:

```yaml
archive:
  keep: 1000   # archived changes to keep in the working tree (default 2000)
```

Eviction keeps the working tree, GitHub's folder views and YASS's own commands small. It doesn't shrink `.git`: every evicted file is still in history, which is what lets you read it back. If clone size matters, use a partial clone (`git clone --filter=blob:none`), which fetches old files only when they're read, or keep the plans in a repo of their own ([`yass.yaml`'s `path:`](../README.md#keeping-plans-out-of-the-repo)).

To never think about it, let CI evict on a schedule and open a pull request for someone to merge:

```yaml
name: yass evict
on:
  schedule: [{ cron: "0 6 * * 1" }]   # Mondays
  workflow_dispatch:
permissions: { contents: write, pull-requests: write }
jobs:
  evict:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with: { fetch-depth: 0 }   # eviction records the last commit that touched each month
      - name: Install yass
        run: |
          v=vX.Y.Z   # pin a release that has `yass evict`
          curl -fsSLO "https://github.com/donjaime/yass/releases/download/$v/yass_linux_amd64.tar.gz"
          curl -fsSLO "https://github.com/donjaime/yass/releases/download/$v/checksums.txt"
          grep yass_linux_amd64.tar.gz checksums.txt | sha256sum -c
          tar -xzf yass_linux_amd64.tar.gz && echo "$PWD/yass_linux_amd64" >> "$GITHUB_PATH"
      - name: Evict, and open a pull request if anything moved
        env: { GH_TOKEN: "${{ github.token }}" }
        run: |
          yass evict
          git diff --cached --quiet && exit 0
          git switch -c "yass-evict-$(date +%Y-%m-%d)"
          git -c user.name="yass evict" -c user.email="actions@users.noreply.github.com" commit -q -m "yass: evict archived months"
          git push -u origin HEAD
          gh pr create --fill
```

A pull request opened with `github.token` doesn't start other workflows; it needs only a plans-only check ([above](#plan-only-commits-run-no-ci)), or use a token of your own if your required checks must run on it.
