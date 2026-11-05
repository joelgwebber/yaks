---
platforms: [all]
source: Jaime, 2026-10-05: does the hook setup blow away existing git hooks? Make it safe and additive, or say how to wire it in
follows: 
blocked:
archived: 2026-10-05T18:25:40Z
---
# yass init --hooks leaves existing hooks alone

## Goal
<!-- Intent. What is true when this change is finished, in a paragraph. Edit it only in a commit without code. -->
Turning on the YASS hook never switches off hooks someone already has. `yass init --hooks` sets `core.hooksPath` only when nothing else is in place; when a repo already has a hooks setup (a `core.hooksPath` of its own, global or local, as husky and others use, or working hooks in `.git/hooks/`), it leaves it alone and prints how to run `yass hook` from that setup. The README's hook section says the same, with snippets for plain git hooks, husky, lefthook and the pre-commit framework.


## Acceptance
<!-- Intent. Observable checks, one behavior each, e.g.
- [ ] Given …, when …, then … — verify: <test, flow, or manual steps> -->
- [x] Given a repo with no `core.hooksPath` and no working hooks in `.git/hooks/` (only `*.sample` files, or none), when you run `yass init --hooks`, then it sets `core.hooksPath` to `tools/yass/githooks`, as today — verify: e2e
- [x] Given a repo whose local `core.hooksPath` points elsewhere (for example `.husky`), when you run `yass init --hooks`, then `core.hooksPath` is unchanged, and it says the repo already has a hooks setup, names it, and prints how to run `yass hook` from it — verify: e2e
- [x] Given a global `core.hooksPath` and no local one, when you run `yass init --hooks`, then it doesn't set a local `core.hooksPath` (which would override the global one for this repo), and says why and what to do — verify: e2e (with a temporary global config)
- [x] Given an executable hook in `.git/hooks/` (any name, not `*.sample`), when you run `yass init --hooks`, then `core.hooksPath` stays unset, and it names the hooks it found and prints the line to add to `.git/hooks/pre-commit` (`tools/yass/githooks/pre-commit "$@"`) — verify: e2e
- [x] Given `core.hooksPath` is already `tools/yass/githooks`, when you run `yass init --hooks` again, then nothing changes and it says the hook is already on — verify: e2e
- [x] Given any of these cases, then `yass init` still exits 0 and writes the hook script and everything else it would; only turning the hook on is skipped — verify: e2e
- [x] Given `yass upgrade`, then it never reads or changes `core.hooksPath` or `.git/hooks/` — verify: e2e (an upgrade with a custom `core.hooksPath` leaves it as it was)
- [x] Given the README's hook section and `docs/install.md`'s `--hooks` row, then they say `--hooks` only turns the hook on when there's no other hooks setup, and show how to call `yass hook` from a plain `.git/hooks/pre-commit`, husky, lefthook and the pre-commit framework — verify: manual: review

## Steps
<!-- Progress. Your working checklist: add, reorder and mark freely: [ ] not started, [/] in progress, [x] done, [-] dropped. -->
- [x] `yass init --hooks`: detect an existing setup (local or global `core.hooksPath`, executable non-sample files in `.git/hooks/`, via `git rev-parse --git-path hooks` so worktrees work)
- [x] The message: what was found, and the snippet for it (a guess at husky from `.husky`, lefthook from `lefthook.yml`, the pre-commit framework from `.pre-commit-config.yaml`; otherwise the plain-git line)
- [x] e2e: each case above
- [x] README hook section ("Already have hooks?") and `docs/install.md`'s `--hooks` row

## Decisions
<!-- Progress. "- <decision> - <why> (<who>)", appended as you go. -->
- Check first and hand over a snippet, rather than make `tools/yass/githooks` a dispatcher that also runs the hooks that were there before - a dispatcher has to know what "before" was and how each hook manager works, and grows with every hook YASS adds; leaving the setup alone and saying how to wire in can't break anyone's hooks (claude, for Jaime's review)
- Today nothing is deleted, but setting `core.hooksPath` silently switches off hooks in `.git/hooks/`, overrides a global `core.hooksPath`, and breaks husky (which sets its own); the README warns about it, `yass init --hooks` doesn't (claude, 2026-10-05)
- The monorepo change's `yass-trailers` piece (two more hooks, `prepare-commit-msg` and `commit-msg`) should follow the same rule; worth a note in that piece when it's picked up (claude)
- The messages guess the hook manager from what's in the repo (`.husky/` or a `core.hooksPath` containing `.husky`, `lefthook.yml`, `.pre-commit-config.yaml`) and fall back to the plain-git line for whatever folder git runs hooks from; for a global `core.hooksPath` they also give the one-line way to use YASS's folder in this repo anyway (claude)

## Log
<!-- Progress. Append before you stop, so anyone can resume:
### YYYY-MM-DD (<who>)
- Did: …
- Next: … -->

### 2026-10-05 (claude)
- Did: `yass init --hooks` checks for a hooks setup first (`internal/yass/hooksetup.go`): a local `core.hooksPath` elsewhere, a global one, or executable non-sample hooks in the clone's hooks folder (found with `git rev-parse --git-path hooks`, so worktrees work). With one, it leaves `core.hooksPath` alone, still writes everything else, and prints how to wire `yass hook` in for husky, lefthook, the pre-commit framework or plain git; already on says so. README: the hook section explains it, with an "Already have hooks?" subsection of snippets; `docs/install.md`'s `--hooks` row too. e2e §28 (17 checks, including `yass upgrade` leaving a custom `core.hooksPath` alone); all 377 e2e checks and `tests/site.sh` pass.
- Next: Jaime reviews locally; when approved, push the plan and code branches as a two-PR stack, then archive after merge.
