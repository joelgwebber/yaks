---
id: yaks-85bb
title: 'Private farm guard: yaks.private refuses sync''s push and a pre-push hook blocks a manual push of the farm branch'
status: shorn
type: feature
priority: 3
created: '2026-10-21T20:31:48Z'
updated: '2026-10-27T01:10:50Z'
parent: yaks-472e
labels:
- git
- cli
needs: agent
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

---
▸ 2026-10-21T20:34:02Z [private-1]

Start: Wed Oct 21 20:33:58 UTC 2026 (date -u)
yaks path yaks-85bb: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-85bb.md

---
▸ 2026-10-21T20:38:17Z [private-1]

Decision: `yaks private` is handled before a farm is opened (like `init`/`migrate --to-branch`), so it works from the code checkout, the farm worktree or a repo with no `.yaks/`. Rejected: running it through `Farm::open` (would fail where no farm resolves, and it touches the repository, not a farm).
- Decision: the hook re-reads `yaks.private` and `yaks.branch` at push time rather than baking the branch in. Rejected: a generated per-branch hook (stale after a `yaks.branch` change; `--off` would have to rewrite it). `--off` still removes our hook, but a leftover hook is inert once the flag is unset (tested).
- Decision: a foreign `pre-push` hook gives exit 0, the flag IS set (sync is guarded), and a stdout warning says a manual push is NOT blocked, plus the hook body to fold in. Rejected: non-zero exit (the claim note says "left untouched and reported"); chaining by renaming their hook (rewrites user files).
- Decision: ownership is the `# yaks-private-guard` marker line; `--off` and re-install only touch files carrying it.
- Decision: `is_private` is true if the repo OR its Delta host has `yaks.private true` (mirrors `configured_farm_branch`); `yaks private` itself writes only the local repo's config (worktrees share it). `--off` warns if the flag is still true from another config.
- Decision: hook path via `git rev-parse --git-path hooks` (honors `core.hooksPath`, shared by worktrees; tested).
- Known gap: the hook matches the pushed LOCAL ref name `refs/heads/<farm branch>`, so `git push origin yaks~1:refs/heads/x` (a non-branch local ref spelling) is not blocked. A Delta clone has its own hooks dir; run `yaks private` inside the clone to guard it.
- `yaks sync` does not exist in this tree yet: clap help, docs/cli.md and the skill say "`yaks sync`, once it exists" must refuse on `yaks.private` (`privatefarm::is_private`); the dead-code warning is avoided because `--off` uses it. Whoever lands sync should reword.
- Evidence: 5 unit tests in src/privatefarm.rs (scratch repo + bare origin + orphan farm worktree): baseline farm push works before `yaks private`; after it `git push origin yaks` (from the farm worktree) and `git push origin --all` (code checkout) both exit non-zero with the message and the bare remote has no refs; `git push origin main` then works; idempotent re-run; `--off` restores pushing and removes the hook; foreign hook untouched/reported and not removed by `--off`; core.hooksPath; custom yaks.branch; non-repo errors. Manually with ./target/release/yaks on a throwaway repo: private -> push --all rc=1, farm push rc=1, main rc=0; --off -> farm push rc=0, remote has both refs.

---
▸ 2026-10-21T20:41:45Z [private-1]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-21T20:54:29Z [private-1]

Summary: new src/privatefarm.rs (is_private, install_guard, remove_guard, run + 5 tests), Command::Private { off } + early dispatch + `mod privatefarm;` in src/main.rs, rows in docs/cli.md and the yaks skill (table + farm-on-a-branch bullet). `yaks verify` PASS recorded above; full `cargo test --release` green on rerun.
- Observed flake, not mine: tests/sheds_layouts.rs `a_flat_shed_forked_before_the_migration_falls_back_to_the_next_base_not_to_nothing` failed twice in full-suite runs (left "checkout", right "merge-base"; reflog second-boundary/timing under load) and passed 6/6 alone both with and without my change (stash -u baseline).

---
▸ 2026-10-21T20:54:30Z [private-1]

moved: shaving -> shorn

---
▸ 2026-10-21T20:58:21Z [delta:herd-dirs]

Landed (delta:herd-dirs): 0e1775b (single-parent, Squashed-from 5a7c0ae). Review with my build on a scratch repo + bare origin: `yaks private` -> `git push origin yaks` and `git push --all` refused, remote empty; `--off` removes the hook and the push goes through. Full gate green (19 suites). Sync will reword the 'once it exists' lines.

---
▸ 2026-10-27T01:10:50Z [Joel Webber]

requested: needs agent

Tidy request: change only the line structure of this yak's description and every note, so each renders as intended under CommonMark.

Put a blank line between blocks, make run-on facts a list, and fence pasted output. Never change a word, a note header (`▸ <ts> [actor]`) or the frontmatter. Edit the file at `yaks path yaks-85bb` directly: `update --description` replaces the whole body, notes included. Then `yaks pickup yaks-85bb --drop-request`, which removes this request note, so no new note is left behind.
