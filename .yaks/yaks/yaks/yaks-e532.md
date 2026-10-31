---
id: yaks-e532
title: Unify skills install
status: shorn
type: task
priority: 1
created: '2026-10-31T14:38:15Z'
updated: '2026-10-31T15:05:04Z'
labels:
- skills
verify: cargo test --no-fail-fast
---

Drop all the optionality in `yaks skills install` -- everyone gets all the tasks.

Let's also drop the scaffolding used to support install optionality -- we won't want to go back to that now that we're simplifying the deployment modes dramatically.

---
▸ 2026-10-31T14:39:24Z [delta:ksQQhJ4mOC_KTlKDZQwbRNBUD5LOADjEEpIBxBCA]

moved: hairy -> shaving

---
▸ 2026-10-31T14:42:24Z [delta:ksQQhJ4mOC_KTlKDZQwbRNBUD5LOADjEEpIBxBCA]

Claimed. Plan (yaks-a99f's commit `3bb5937` is the base; it removed the farm modes, so the tree I started from was stale):

- Drop `--with`/`GROUPS`/`Skill.group`/`select`/`select_for_status`, `init --skills <set>` (`SkillSet`) and the `Stale.with` advice. Every install and every status covers all of `BUNDLED`.
- `init` keeps one opt-out for scratch repos and tests: `--no-skills`.
- Sweep leftovers of the old mode structure: `brief::Mode`, "private-mode" wording, the coordination companion-router test, "opt-in" in skill descriptions, docs and help.

---
▸ 2026-10-31T14:54:57Z [delta:ksQQhJ4mOC_KTlKDZQwbRNBUD5LOADjEEpIBxBCA]

Done, uncommitted in this worktree on top of `3bb5937` (yaks-a99f).

- **One install.** `yaks skills install` and `yaks skills status` always cover all four bundled skills. Gone: `--with`, `GROUPS`, `Skill.group`, `select`, `select_for_status`, `Stale.with` (the stale advice is always plain `yaks skills install`), and the auto-sync's default-set-only rule. `install(base, force)` and `status(base)` take no skill list; `install_one` stays for the script-fixture tests.
- **`init`.** `--skills none|default|coordination` is replaced by `--no-skills` (`SkillSet` deleted); with no flag it installs everything. The old flag now errors with a `--no-skills` hint.
- **Mode leftovers.** `brief::Mode::{Branch,Private}` became `Publication::{Published,Private}` (it only ever said whether ids may reach git); `detect_mode` is now `detect_publication`. "private-mode" wording is gone from `scan-ids` help and a test comment, and `init`'s doc comment no longer says "Branch mode".
- **Dead test.** The router check for `yaks-coordinating-*` companions is removed; those skills are retired.
- **Docs and help.** `README.md`, `docs/cli.md`, `docs/skills.md`, `.agents/skills/README.md`, `AGENTS.md`, the `yaks`, `yaks-working` and `yaks-coordinating` skills (the "Opt-in skill, installed by --with" text is out of their descriptions), and the clap help are all updated. The `skills install` hint no longer says the skill "activates when a .yaks/ directory is present", which stopped being true with farm branches.

## Evidence

- `cargo test --no-fail-fast`: every target green except `sheds::tests::inspecting_a_shed_changes_nothing_in_it`. That one fails the same way with my changes stashed, so it is not from this change. The diff shows an `fsmonitor--daemon/cookies` mtime moving, which looks like macOS fsmonitor.
- Real binary in a scratch repo: `yaks init` installed four skills and `skills status` listed four as `current`. `skills install --with coordination` and `init --skills none` are both rejected.
- Parity grep for `--with`, `--skills`, `SkillSet`, `GROUPS`, `opt-in`, `--mode` and `Mode::` across README, docs, skills, src and tests: only the `--mode team` rejection test and `--skills none` rejection assertions remain.

Not done: a cold-read scout of the changed skill text, and `cargo install` (waits for landing).

---
▸ 2026-10-31T14:54:57Z [delta:ksQQhJ4mOC_KTlKDZQwbRNBUD5LOADjEEpIBxBCA]

moved: shaving -> shorn

---
▸ 2026-10-31T15:05:04Z [delta:Clean up skills install and mode remnant]

Landed on `main` as `c2b640f`, on top of `3bb5937` (yaks-a99f), via `git push local main:main`.

- `cargo test --no-fail-fast` on that exact commit: all targets green, 734 unit tests. That run set `core.fsmonitor=false` for the process, because `sheds::tests::inspecting_a_shed_changes_nothing_in_it` fails under the macOS fsmonitor daemon (it moves a `cookies` mtime). It fails identically on the base without these changes. A follow-up yak tracks it.
- `yaks preflight --push-main`: ok.
- Verified in the user's checkout: `main` is at `c2b640f` and `git status --short` is empty.
