---
name: land
description: "Land finished work on main in the yaks repo, tracked by its yak. Only invoke when the user has asked to land or merge the change (the Land Changes button or an explicit request), not merely to review, prepare, run checks or install this skill."
disable-model-invocation: true
metadata:
  delta-action: land
---

# Land (yaks repo)

Personal project: no PRs, no code CI, `main` is the destination. The yak workflow
is the point, so work is tracked before it lands and closed out after. Being
invoked is the request to land: proceed; stop only for a real blocker, and then
report plainly that nothing landed and why.

Conflicts: pause. Report the conflict and wait for the user; do not resolve it.

Sources for commands: `AGENTS.md` (Verifying changes, Keeping docs current,
Delegating), `.yaks/config.yaml` on the farm branch (`yaks path`) for `verify:`,
`yaks land --help`, `yaks preflight --help`, `.agents/skills/yaks-coordinating/SKILL.md` §6, §9.

## 1. Track the work

1. Load the `yaks` skill (pass the worktree argument). `yaks show` the yak for
   this change. If there is none, `yaks create` one (check `yaks search` first);
   it must be `shaving` (`yaks shave <id>`, and its parent if still hairy).
   Never land code that has no yak.
2. Record follow-ups you found as new yaks; note decisions as `Decision:` notes.
3. Parity check, in this same change (`AGENTS.md`, Keeping docs current): `docs/`,
   `README.md`, the bundled skills, clap `--help` for CLI changes, `help_content`
   / `help_hint` in `src/tui/render.rs` for TUI keys. Grep for old names/keys.

## 2. Gate

1. Commit the code (never stage `.yaks/`; stage explicit paths). Put the yak id
   in the commit message: the farm is published.
2. `cargo fmt -- <files you touched>`, then `git checkout src/actor.rs
   src/tui/docshots.rs src/tui/tests.rs` unless you changed them.
3. Run the yak's `verify:` (`yaks verify <id>`; labels map to `cargo test -p yaks`
   or `cargo test -p yaks skills`; default `cargo test --workspace`) on the exact
   commit being landed. Failing, or not run, means not landed.
4. TUI change: drive `toque` to the state, look at the frame, `yaks attach` it
   to the yak.

## 3. Land on main

- **From a Delta worker subthread** (the commit is on no branch in the user's
  checkout): in the main checkout, `yaks land <sha> --dry-run`, then
  `yaks land <sha>`. It never pushes.
- **Already in the user's checkout on `main`:** the commit from step 2 is the landing.
- **From a Delta thread whose clone is not the user's checkout:** `yaks preflight
  --push-main`, then `git push local <branch>:main` (follow its `pr/<name>`
  fallback if it says so).
- Never push `origin` or open a PR without the user's say-so.
- Verify: `main` contains the change (`git log main -1`, `git merge-base
  --is-ancestor <sha-or-landed> main`), and `git status --short` shows no stray
  changes. A pending `MERGE_HEAD` that `yaks land` cannot explain is a blocker.

## 4. Close out the yak

1. `yaks update <id> --note` with a short summary: what changed, the landed SHA,
   the evidence (the checks you actually ran), and any yaks spawned. Never claim
   what you did not observe.
2. `yaks shorn <id>` (and its parent if this was the last open child).
3. `yaks doctor`, `yaks preflight`, then `yaks commit`. Leave `yaks sync`
   (publishing) to the user.

## 5. Hand over a build

After a significant landing, from the main checkout:

```sh
cargo build --release && mkdir -p ~/go/bin && install -m 755 target/release/yaks ~/go/bin/yaks
```

This is the only place a test build goes. Say you did it; for a TUI change add
that a running `yaks tui` needs a restart.
