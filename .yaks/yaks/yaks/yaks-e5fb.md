---
id: yaks-e5fb
title: 'Port worker landing (land.sh) into the binary: yaks land <sha>'
status: shorn
type: feature
priority: 2
created: '2026-10-18T15:54:02Z'
updated: '2026-10-27T01:18:52Z'
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

---
▸ 2026-10-21T22:13:11Z [land-1]

Decision: yaks-3bcd's check as written (`git merge-tree --write-tree HEAD <sha>` equals HEAD's tree) is VACUOUS when <sha> is an ancestor of HEAD: the merge base is then <sha> itself, so the result is always HEAD's tree. Observed in a scratch repo with a real evil merge (merge commit with the worker as a parent, changes dropped): `merge-tree --write-tree HEAD $W` printed exactly HEAD^{tree}.
- Used `git merge-tree --write-tree --merge-base=<worker parent> HEAD <sha>` instead: same scratch repo gives a different tree (HEAD plus the dropped change); a worker commit that really is in HEAD gives HEAD's tree.
- Cost: `--merge-base` needs git 2.40 (the rest needs 2.38). Only the already-an-ancestor path needs it; on an older git that path refuses with a message saying so. Rejected: reverse-applying the diff with `git apply -R --check` against a temp index (works on any git but needs temp-index plumbing and fuzz).
- If later commits edited the same lines the merge conflicts; that is also a refusal (cannot confirm), naming the conflicts.

---
▸ 2026-10-21T22:15:43Z [land-1]

asked: needs human

Question (non-blocking, FYI): `tests/init_modes.rs` (init-1's area) asserts `yaks-coordinating-delta/land.sh` exists after `init --skills coordination`, which cannot hold once land.sh is deleted. I changed that ONE assertion to check `yaks-coordinating-delta/SKILL.md` instead (same for `tests/skills_install.rs`), because the gate is red otherwise. If init-1 also edits that test, expect a trivial conflict on those 3 lines. Tell me if you want it done differently; I am continuing meanwhile.

---
▸ 2026-10-21T22:21:22Z [land-1]

Decision: the farm landing (`yaks commit --land`) the yak description mentions is NOT in `yaks land`. The claim note's scope does not list it, `yaks commit` has no `--land` flag, and land.sh never did it. Rejected: adding it unasked. Left as a follow-up.
- land.sh's last step (`doctor: <last line>`) IS ported: `land::cli` runs this binary's `path` then `doctor` in the checkout top and prints the last line. `land::run` (what the tests call) does not, so tests never spawn the test binary.

---
▸ 2026-10-21T22:21:22Z [land-1]

Decision: already-ancestor SHA that IS in HEAD stops after saying `landed: <HEAD oneline>` (no commit, no clearing of a pending merge; it notes the pending state is still set). Rejected: carrying on to the squash path (land.sh would have made an empty commit) and clearing Delta's applied state (destructive, nothing needs landing).

Decision: landing a worker whose merged tree already equals HEAD's tree (a squash-landed worker landed twice) clears the applied state, then prints `HEAD's tree already equals the computed merge: nothing to commit` + `landed:`. land.sh made an empty duplicate commit. Rejected: erroring (idempotent re-run is harmless).

Decision: clearing the applied state uses `git reset -q --hard HEAD` where land.sh used `git checkout -q -- .`. checkout restores from the index, so a staged copy of a worker file survives and then blocks the ff merge; reset --hard makes index and tree HEAD's, which is what the "tracked changes will be discarded" message promises. Untracked files are untouched either way (identical-to-worker ones removed, others exit 4, as before).

Decision: everything runs from the checkout top-level (`rev-parse --show-toplevel`), so `yaks land` from a subdirectory works; land.sh's `git checkout -- .` only restored the cwd subtree. Test: runs_from_a_subdirectory_of_the_checkout.

Decision: `git status` is read with `--porcelain=v1 -z` (renames count both names; no quoting problems), and the squash message trailer is added with `git interpret-trailers` (joins an existing trailer block), falling back to a plain appended paragraph.

Decision: exit codes: land.sh's 3 (unexplained), 4 (untracked differs), 5 (commit/cherry-pick failed) kept; new 6 (ancestor but change missing/unconfirmable, yaks-3bcd), 7 (`.yaks/` files with no tracked `.yaks/` in HEAD, yaks-7bfd). Usage errors are clap's.

Decision: skills.rs — no bundled skill has extra files after land.sh goes. The generic machinery (SkillFile, per-file stamp digests, executable bit, edited/stale extras) is still live code, so I kept its three tests, ported onto a test-only `FIXTURE_NEW` static skill with a `run.sh` (plus a new install/executable-bit test) instead of deleting them. Only `delta_land_script_selftest_passes` was deleted (its scenarios are in src/land.rs). `the_file_list_matches_the_source_directories` still guards BUNDLED.

Decision: also touched `tests/skills_install.rs` and `tests/init_modes.rs` (one assertion each: `land.sh` is_file -> `SKILL.md` is_file) and `docs/skills.md` (the land.sh example text). Not touched, flagged for the yakherd: root `AGENTS.md` line ~157 still says "like `yaks-coordinating-delta` carries `land.sh`".

---
▸ 2026-10-21T22:21:23Z [land-1]

Evidence: real run of ./target/release/yaks in a scratch repo (main moved after the worker branched; Delta-like state: `git merge --no-commit` then `git read-tree HEAD`, so MERGE_HEAD set, a.txt modified, new/ untracked):
```
== dry-run   (yaks land <sha> --dry-run)
worker commit: 2e01d10 worker change
pending state: merge
worker base 4f90bc7 is an ancestor of HEAD: squashing onto HEAD
correct merged tree: 548a0f38fd54
dry run: would clear the applied state and land 2e01d10 by squash
== land   (yaks land <sha>)
...same four lines, then:
result is identical to the computed merge
landed: 63b00cb worker change
* 63b00cb t worker change      <- one single-parent commit
* fba5940 t main moves
* 4f90bc7 t base
message: "worker change\n\nbody line\n\nSquashed-from: 2e01d10031a1e01fd920a89f0c6b36348bf7acc4"; git status clean
```
Second scratch repo, same binary:
```
$ yaks land <sha>        (again)       -> "HEAD's tree already equals the computed merge: nothing to commit" / "landed: f7e879c worker change", rc=0, no new commit
$ yaks land <sha>        (evil merge)  -> land: worker commit daa7b62 is an ancestor of HEAD but its change is missing from HEAD (a merge that dropped it, or a later revert); it was NOT landed
                                          the first commit after it that lacks the change: 733ba85 evil merge        rc=6
$ yaks land <sha>        (.yaks/h/yaks/yaks-1.md in the worker commit, HEAD tracks none) -> land: the worker commit touches yak files, but HEAD tracks no `.yaks/` ...: .yaks/h/yaks/yaks-1.md   rc=7
```
Failing-before: with `--merge-base={parent}` swapped for `--merge-base={full}` (= the literal, vacuous yaks-3bcd check) and the `.yaks/` refusal disabled, `cargo test --release land::tests` fails 3: the_evil_merge_that_dropped_the_workers_change_is_refused, an_ancestor_whose_change_was_edited_over_later_cannot_be_confirmed, a_worker_commit_with_yak_files_is_refused_when_head_tracks_none. Restored afterwards.

---
▸ 2026-10-21T22:25:34Z [land-1]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-21T22:31:00Z [land-1]

Summary (land-1): `yaks land <sha> [--dry-run] [--force-discard]` is in the binary (`src/land.rs`, one `Command::Land` variant + arm in main.rs, handled before the farm opens). It ports every land.sh behaviour and adds the yaks-3bcd ancestor check (with `--merge-base`, see the Decision note) and the yaks-7bfd `.yaks/` refusal. land.sh, its BUNDLED entry and its selftest test are gone; its scenarios are 18 Rust tests in src/land.rs against real temp repos. `land.sh` mentions replaced in .agents/skills/ (README, delta, team), docs/cli.md (row + skills install), docs/skills.md. Gate: `cargo test --release` green (803 unit + all integration suites, 0 failed). Not run: the TUI/`?` help (no TUI change); a real Delta-applied state (scratch repos only); git older than 2.54.

---
▸ 2026-10-21T22:31:01Z [land-1]

moved: shaving -> shorn

---
▸ 2026-10-21T22:49:46Z [delta:herd-dirs]

Landed (delta:herd-dirs): e99754d (squash by SHA of 6479e79, one docs/cli.md conflict resolved by hand: both the branch-mode init docs and the land row kept). Fixed the AGENTS.md land.sh line it flagged. Real run with my build in a scratch repo: --dry-run reports the plan, the land is a single-parent squash ('result is identical to the computed merge'), a second land of the same SHA says 'landed:' and adds no commit. Its ask on tests it touched outside scope: accepted (they had to change once land.sh was gone).

---
▸ 2026-10-21T22:49:47Z [delta:herd-dirs]

Yakherd: accepted, the two test-assertion edits were forced by deleting land.sh.
