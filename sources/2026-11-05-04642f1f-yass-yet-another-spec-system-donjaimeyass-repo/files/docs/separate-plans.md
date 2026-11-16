# Plans in another repo: commits name their change

When a change's plans live outside the code repo (a `yass.yaml` whose `path:` points elsewhere, see the [README](../README.md#keeping-plans-out-of-the-repo)), no commit can carry its progress next to the code. So each code commit for it says which change it belongs to instead, in a git trailer at the end of its message:

```
feat: disable Save while saving

yass: 2026-09-14-offline-sync/2026-09-16-offline-queue
```

The trailer survives squash merges when the pull request's description or commit list keeps it, and git tools read it (`git log --format='%(trailers:key=yass)'`). Changes whose plans are in the repo's own `yass/` need none: their progress travels in the commits.

## The trailer

```
yass: <target> [ac:<criteria>]
```

- **target** is a change's folder name, `<change>/<piece>`, a piece's own name when only one change has a piece by that name, or `none` for a commit that belongs to no change (a dependency bump, a typo fix). Archived changes count too.
- **`ac:`** names the criteria the commit works toward, comma-separated: `yass: 2026-09-14-offline-sync ac:ac2,ac5`.
- **Case doesn't matter;** YASS writes lower case. A commit for two changes gets two `yass:` lines. A key this version of YASS doesn't know is a note, not an error, so newer trailers don't break older binaries.

## You don't type it

`yass init --hooks` installs three hooks in `tools/yass/githooks/`: `pre-commit` (YASS's usual check), `prepare-commit-msg`, which adds the trailer, and `commit-msg`, which checks it. The hook works out the change from, in order:

1. **`yass use <change>`**, which sets this worktree's change until you change or clear it (`yass use` shows it, `yass use --clear` clears it). It's kept in the worktree's own git folder, so it's never committed, and each worktree can work on a different change. `--ac ac2,ac5` adds criteria.
2. **The branch name**, when it's a change's or piece's folder name, on its own or after a prefix: `jaime/2026-09-14-offline-sync`.
3. **An earlier commit's trailer** on the branch, since it left the main branch.

It adds the trailer when that change lives outside the repo, and leaves alone a trailer you wrote and merge commits. The `commit-msg` hook points out a trailer that's malformed or names no change, with the closest change names; it never asks for a missing one and never refuses a commit.

## In CI

`yass hook --range <base>..<head>` checks the trailers of every non-merge commit in the range, alongside its usual check. `--require-trailers` also reports commits without one, and `--strict` fails the job on anything it reports. A squash commit whose message carries the trailers passes. Where CI doesn't check out the plans, it checks only the trailers' form and says the change names weren't checked.

## Finding a change's code

`yass status <change>`, run in the code repo, lists the commits on the main branch whose trailers name that change or one of its pieces, and, when they name criteria with `ac:`, which commits worked toward each one:

```
code on origin/main (from yass: trailers):
  9f2c1aa Cart totals (#7)
  4e1b2c3 feat: cart ui
criteria:
  AC1: 4e1b2c3
  AC3: 9f2c1aa, 4e1b2c3
```

Because it reads trailers, not commit hashes cited in the plans, it still works after a squash merge. Unmerged commits aren't listed, and an `ac:` naming a criterion the plan doesn't have is a warning. It shows ten commits per list and says how many more there are.

## With a hook manager

If your repo already runs hooks through a manager, call YASS from it rather than pointing `core.hooksPath` at YASS's folder.

**husky** (tested): add a line to each hook file.

```sh
# .husky/prepare-commit-msg
yass hook --prepare-msg "$@"
# .husky/commit-msg
yass hook --commit-msg "$1"
# .husky/pre-commit
yass hook
```

**lefthook**, in `lefthook.yml`:

```yaml
prepare-commit-msg:
  commands:
    yass:
      run: yass hook --prepare-msg {1} {2}
commit-msg:
  commands:
    yass:
      run: yass hook --commit-msg {1}
pre-commit:
  commands:
    yass:
      run: yass hook
```

**pre-commit**, in `.pre-commit-config.yaml` (it passes the message file to both message hooks):

```yaml
default_install_hook_types: [pre-commit, prepare-commit-msg, commit-msg]
repos:
  - repo: local
    hooks:
      - id: yass
        name: yass hook
        entry: yass hook
        language: system
        pass_filenames: false
        always_run: true
      - id: yass-trailer
        name: yass trailer
        entry: yass hook --prepare-msg
        language: system
        stages: [prepare-commit-msg]
      - id: yass-trailer-check
        name: yass trailer check
        entry: yass hook --commit-msg
        language: system
        stages: [commit-msg]
```
