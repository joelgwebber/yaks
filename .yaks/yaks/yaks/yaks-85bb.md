---
id: yaks-85bb
title: 'Private farm guard: yaks.private refuses sync''s push and a pre-push hook blocks a manual push of the farm branch'
status: shaving
type: feature
priority: 3
created: '2026-10-21T20:31:48Z'
updated: '2026-10-21T20:33:02Z'
parent: yaks-472e
labels:
- git
- cli
---

---
▸ 2026-10-21T20:33:01Z [delta:herd-dirs]

moved: hairy -> shaving

---
▸ 2026-10-21T20:33:02Z [delta:herd-dirs]

Claim (delta:herd-dirs, yakherd). Owner: `private-1`.

**Goal:** a farm marked private never leaves the machine through yaks or a stray `git push`.

**Scope:**
- New `src/privatefarm.rs`: `pub fn is_private(repo: &Path) -> bool` (`git config --bool yaks.private` in the host repo, as `store::configured_farm_branch` reads `yaks.branch`), and `pub fn install_guard(repo: &Path) -> Result<GuardOutcome>`: writes a `pre-push` hook into the repository's hooks dir (`git rev-parse --git-path hooks`, respecting `core.hooksPath`) that reads git's stdin ref lines and exits 1 with a message naming the farm branch when any pushed local ref is `refs/heads/<farm branch>` and `yaks.private` is true. If a pre-push hook already exists and is not ours (no `# yaks-private-guard` marker), do not overwrite: return `GuardOutcome::Foreign(path)` and print how to chain it. Ours: rewrite idempotently. POSIX sh, no bash-isms.
- `src/main.rs`: ONE new `Command::Private { off: bool }` variant (`yaks private` marks the farm private: sets `yaks.private true` and installs the guard; `yaks private --off` unsets it and removes OUR hook only). Prints what it did.
- Expose `is_private` for `yaks sync` (yaks-e3b2) to refuse pushing; do not build sync.
- Docs: `docs/cli.md` row, clap help, one line in the yaks skill's farm-on-a-branch section.

**Out of scope:** sync, TUI, init integration.

**Evidence:** test with a scratch repo + bare remote + farm branch: after `yaks private`, `git push origin yaks` from the farm worktree AND `git push --all` from the code checkout are both refused and the remote's branch is unchanged; pushing `main` alone still works; `yaks private --off` restores pushing; a foreign pre-push hook is left untouched and reported.

**Judge:** the yakherd.
