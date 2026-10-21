---
id: yaks-e5fb
title: 'Port worker landing (land.sh) into the binary: yaks land <sha>'
status: shaving
type: feature
priority: 2
created: '2026-10-18T15:54:02Z'
updated: '2026-10-21T22:10:38Z'
parent: yaks-6a34
labels:
- cli
- git
---

Joel (2026-10-18): shell scripts for landing are stinky, especially across platforms. Move `.agents/skills/yaks-coordinating-delta/land.sh` into the binary as `yaks land <sha> [--dry-run] [--force-discard]`: pending-state report, ancestry check (shallow-aware), `merge-tree --write-tree`, the unexplained-changes refusal, clearing Delta's applied state, the squash commit (the linear landing from its sibling yak), the tree check, and, for a live farm, the farm landing (`yaks commit --land`) in the same command. git stays a subprocess (no libgit2 dependency); the selftest scenarios become Rust tests with real temp repos. Then land.sh is deleted from the skill and from `BUNDLED`, and the Delta/worktrees skills say `yaks land`. Feasible: every step is git plumbing the script already calls.

---
▸ 2026-10-21T22:10:09Z [delta:herd-dirs]

Claim (delta:herd-dirs, yakherd). Owner: `land-1`. Also covers yaks-3bcd (same code).

**Goal:** `yaks land <sha> [--dry-run] [--force-discard]` does exactly what `.agents/skills/yaks-coordinating-delta/land.sh` does today, in Rust, and the script goes away.

**Scope:**
- New `src/land.rs` + ONE `Command::Land { sha, dry_run, force_discard }` variant and dispatch arm in `src/main.rs` (runs in the cwd's git checkout, the code checkout; needs no farm: handle it before the farm opens, like `discover`).
- Port every behaviour of land.sh (read it end to end, including `--selftest`): the pending-state report (MERGE_HEAD, applied files), the shallow-aware ancestry check, `git merge-tree --write-tree HEAD <sha>`, the refusal when the working tree holds changes the worker commit does not explain (unless `--force-discard`), clearing Delta's applied state, landing as ONE single-parent squash commit on HEAD with the worker's message + author + a `Squashed-from: <full sha>` trailer, the cherry-pick fallback (base rewritten or the merge would conflict), and the final check that HEAD's tree equals the computed tree. Never pushes. Same output lines where sensible (`landed: <sha>` etc.).
- yaks-3bcd: when `<sha>` is ALREADY an ancestor of HEAD, verify `git merge-tree --write-tree HEAD <sha>` equals HEAD's tree; otherwise refuse ("its commit is an ancestor but its change is missing from HEAD"). Add a unit test reproducing the evil merge (a merge commit that has the worker as a parent but dropped its change).
- From yaks-7bfd: when the worker commit touches `.yaks/` and HEAD has no tracked `.yaks/` (the farm lives on its own branch), refuse naming the paths (a worker must never stage yak files under a branch farm).
- Remove land.sh: delete the file, its `BUNDLED` entry in `src/skills.rs`, and the tests that exercise it (`delta_land_script_selftest_passes`, the land.sh executable-bit and stale-script install tests: replace any that test generic "a skill's extra file" behaviour with an equivalent on another fixture only if the behaviour is still needed; if no bundled skill has extra files left, say so in a Decision note). Port the selftest scenarios to Rust tests in `src/land.rs`.
- Minimal skill edits: every `sh .agents/skills/yaks-coordinating-delta/land.sh <sha>` / `land.sh` mention in `.agents/skills/` and `.agents/skills/README.md` becomes `yaks land <sha>` (the full skills rewrite is yaks-b048; do not restructure). `docs/cli.md` row + clap help.

**Out of scope:** `yaks sync`, init, discovery, skills restructuring.

**Evidence:** the ported selftest scenarios + the evil-merge test + the `.yaks/` refusal, each failing before; full `cargo test --release`; a real run of `./target/release/yaks land <sha> --dry-run` against a scratch two-branch repo, pasted in a note.

**Judge:** the yakherd (re-runs the tests and lands its own next worker with `yaks land`).

---
▸ 2026-10-21T22:10:10Z [delta:herd-dirs]

moved: hairy -> shaving

---
▸ 2026-10-21T22:10:38Z [land-1]

Start (land-1): date -u = Wed Oct 21 22:10:34 UTC 2026
- yaks path yaks-e5fb = /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-e5fb.md
