---
id: yaks-4c93
title: 'Retire the four coordination companion skills: drop them from BUNDLED, remove untouched installed copies'
status: shorn
type: task
priority: 2
created: '2026-10-21T22:59:46Z'
updated: '2026-10-27T01:10:50Z'
parent: yaks-b048
labels:
- skills
- cli
needs: agent
---

The skills rewrite (yaks-b048) folds `yaks-coordinating-team`, `-private`, `-worktrees` and `-delta` into `yaks-coordinating`. Dropping them from the binary is not enough: anyone who ran `yaks skills install --with coordination` keeps the old copies, which then teach the retired per-shed model forever (nothing upgrades a skill the binary no longer knows).

---
▸ 2026-10-21T23:00:07Z [delta:herd-dirs]

moved: hairy -> shaving

---
▸ 2026-10-21T23:00:07Z [delta:herd-dirs]

Claim (yakherd delta:herd-dirs). Owner: `retire-1` (Sonnet worker).

**Goal.** Add a retired-skills rule to `src/skills.rs`, then drop the four companion skills from `BUNDLED`.

- A `RETIRED: &[&str]` const holding `yaks-coordinating-team`, `yaks-coordinating-private`, `yaks-coordinating-worktrees` and `yaks-coordinating-delta`, each with a one-line doc comment saying what replaced it (`yaks-coordinating`).
- `install` (the shared function used by `yaks skills install` and `yaks init --skills`): for each retired name present in the target dir, if its `SKILL.md` carries our stamp and is unedited (the same digest check `inspect` uses), remove that skill directory and report it; if it is edited or unstamped, leave it and report that it was kept and why (remove it by hand). Add whatever field or variant you need to `Installed` (or a sibling result type) and render the lines in `render_installed` or next to it, so both callers print them.
- `status`/`select_for_status` (`yaks skills status`): a retired skill present in the dir is listed with a new state word `retired`.
- `Stale::of`/the project-local notice and the `yaks doctor` advisory: a present, unedited retired skill counts as needing `yaks skills install --with coordination`, the same advice text path as stale.
- Auto-sync (`auto_sync_in`) stays as is (it never touched the coordination group).
- Remove the four entries from `BUNDLED` and fix every test that names them: `SIX` and `with_coordination_adds_the_six_skills` in `src/skills.rs` (now two skills: `yaks-coordinating` and `yaks-working`), `tests/skills_install.rs` and `tests/init_modes.rs` (use `yaks-coordinating` where they used `-delta`). The `.agents/skills/yaks-coordinating-*` directories are deleted by the yakherd in parallel; do NOT edit anything under `.agents/skills/`, `docs/` or `README.md`.
- Update the `--with` help on `skills install` and `init` in `src/main.rs` (the `yaks-coordinating (+ -team, ...)` text) to read `yaks-coordinating and yaks-working`.

**Out of scope.** Skill prose and docs (the yakherd's job), the brief text in `src/brief.rs`, any other command.

**Evidence.** Tests that fail before the change: (1) install into a temp dir holding an untouched stamped copy of a retired skill removes it; (2) an edited copy is kept and reported; (3) `status` reports `retired`; (4) the notice and `Stale` include it. Gate: `cargo build --release && cargo test --release --no-fail-fast`, all suites green. Also run the real binary: `./target/release/yaks skills install --dir <tmp> --with coordination` after planting an old stamped copy, and paste the output.

**Judge.** The yakherd, at landing.

---
▸ 2026-10-21T23:00:35Z [retire-1]

Start: Wed Oct 21 23:00:32 UTC 2026; yaks path yaks-4c93 = /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-4c93.md

---
▸ 2026-10-21T23:03:00Z [retire-1]

verify: `cargo test -p yaks skills` -> PASS (exit 0)

---
▸ 2026-10-21T23:06:41Z [retire-1]

Decision: retired state is two `SkillState` variants, `Retired { from }` (stamped + digest matches: `install` removes it) and `RetiredKept { stamped }` (edited or unstamped: kept). Rejected: one variant plus a flag, and reusing `Modified`/`Unmanaged` (they `needs_force`, which would make `--force` rewrite a skill the binary no longer has content for). Both show the word `retired`.
Decision: `--force` never removes a retired skill. It is edited or foreign content; the report says to remove it by hand.
Decision: `status()` always appends present retired skills (not only with `--with`), and `install()` always handles them whatever skills it was given, so `yaks init --skills default` also cleans up. Rejected: only under `--with coordination` (a leftover would hide).
Decision: `Installed` gets a `removed: bool` plus `changed()` (wrote || removed); `init.rs` uses `changed()` for its report. Rejected: a separate result type.
Decision: `Stale::of` counts `Retired` (untouched) but not `RetiredKept`, and always adds group `coordination` to the advice. `skills status` footer now reads "N stale or retired; run ...".
Decision: removal failure is an error (`remove_dir_all` with context), not silent.
Decision: retired skills that resolve into a yaks source tree are never reported or touched (`inspect_retired` returns None).
Evidence: the four new tests (`install_removes_an_untouched_retired_skill`, `install_keeps_and_reports_an_edited_or_unstamped_retired_skill`, `status_reports_a_present_retired_skill_as_retired`, `an_untouched_retired_skill_is_stale_in_the_notice_and_stale_of`) did not compile before the change (`Installed.removed`, `RETIRED` absent), so they could not pass; they pass after. `yaks verify yaks-4c93` PASS. Full gate: `cargo build --release && cargo test --release --no-fail-fast`: 808 unit + 20 integration suites, 0 failed.
Docs parity: grep for `coordinating-(team|private|worktrees|delta)` outside src/skills.rs finds only `docs/skills.md:35` and a comment in `src/land.rs:18` (both out of my scope; the coordinator is rewriting docs). `--with` help in `src/main.rs` updated (only the `skills install` one carried the old text; `init`'s `--skills` doc has no skill names).
Real binary (./target/release/yaks, tmp dir, old copies planted by installing with the system yaks 0.0.21, then one edited):
- `skills status`: 4 `current`, `retired yaks-coordinating-team  (retired; installed by 0.0.21; ...)`, `-private  (retired, and edited since install - remove it by hand)`, `-worktrees`, `-delta`; footer `3 stale or retired; run ... --with coordination`.
- `skills install --with coordination`: `removed yaks-coordinating-team (retired, now part of yaks-coordinating) -> ...` x3, and `kept yaks-coordinating-private [retired]: ...  it was edited since it was installed; yaks retired it (now part of yaks-coordinating), remove it by hand`. Afterwards only the edited directory remained and `status` listed it as `retired`.

---
▸ 2026-10-21T23:06:45Z [retire-1]

verify: `cargo test -p yaks skills` -> PASS (exit 0)

---
▸ 2026-10-21T23:06:45Z [retire-1]

moved: shaving -> shorn

---
▸ 2026-10-21T23:11:04Z [delta:herd-dirs]

Landed as 00fbc8c (yaks land, squash of 16edef2). Yakherd re-ran the full gate: 20 suites green (unit 808 passed). Reproduced: 0.0.21 coordination install in a temp dir, one companion edited; new binary removed the three untouched, kept the edited one with 'remove it by hand'.

---
▸ 2026-10-27T01:10:50Z [Joel Webber]

requested: needs agent

Tidy request: change only the line structure of this yak's description and every note, so each renders as intended under CommonMark.

Put a blank line between blocks, make run-on facts a list, and fence pasted output. Never change a word, a note header (`▸ <ts> [actor]`) or the frontmatter. Edit the file at `yaks path yaks-4c93` directly: `update --description` replaces the whole body, notes included. Then `yaks pickup yaks-4c93 --drop-request`, which removes this request note, so no new note is left behind.
