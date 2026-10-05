---
id: yaks-5c83
title: 'yaks: tell the user once per release when project-local skills are stale (decision C of yaks-36b6)'
type: feature
priority: 2
created: '2026-10-05T22:39:41Z'
updated: '2026-10-05T22:49:12Z'
parent: yaks-36b6
labels:
- skills
- cli
---

Decision C of yaks-36b6 (Joel, 2026-10-05): "I'm good with (C) as long as notification is automatic, and a `yaks skills` re-run fixes it easily." So project-local skills are NEVER auto-written (yaks-3859 stays as landed), but an ordinary yaks command tells the user when they are stale.

Behaviour:
- When an ordinary yaks command runs inside a git repo whose project-local skills install (`./.agents/skills` in the git top-level; only skills from BUNDLED that are actually installed there) holds a skill whose state is `stale` (untouched since install, and this yaks is newer: the states `yaks skills status` already computes), print ONE line to stderr, once per yaks version per checkout, e.g. `yaks: the skills in .agents/skills are from yaks 0.0.12, this is 0.0.13: run yaks skills install to update them`. Add `--with coordination` to the advice when an opt-in skill is among the stale ones. Never when the command was given `--json`, and never for the `skills` and `init` commands themselves (they print their own status).
- Where "once per version" is remembered: NOT in the working tree and nothing tracked. Use a small marker file inside the checkout's git dir (`git rev-parse --git-dir`, e.g. `<git-dir>/yaks-skills-notified` holding the yaks version that was announced), or a per-user cache dir if you find a reason; decide, and state it in a `Decision:` note. Each Delta clone or worktree has its own git dir, so each lane is told once. Writing the marker must never fail a command.
- Silence it with the same switch as the user-level sync: YAKS_SKILLS_AUTOSYNC=0 (document that). Also silent in a yaks source checkout (state `source`), and for `modified`, `unmanaged` and `held` skills (those are the user's call; `yaks doctor` and `yaks skills status` already say so).
- `yaks skills install` already upgrades a cleanly-stale project-local skill when re-run (yaks-3859): verify that with a test, and make the notice name exactly that command. The `yaks doctor` advisory and `yaks skills status` must say the same thing in the same words.

Evidence: tests with temp git repos and a temp HOME: a stale project-local skill gives one stderr line, then silence on the next command, and again after a simulated version change; silent for current, modified, held and source; silent with YAKS_SKILLS_AUTOSYNC=0 and with --json; running install after the notice makes it current and quiet; nothing in the working tree changes (compare `git status --porcelain` and the mtimes under .agents/skills before and after). A real run in a temp repo.

Out of scope: auto-writing anything project-local (explicitly rejected), the user-level sync, init.

---
▸ 2026-10-05T22:39:49Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-05T22:39:49Z [delta-lead]
Claim (coordinator delta-lead). Owner: worker ntf-1.

Scope: src/skills.rs (reuse the stale check, add the notice), the hook in src/main.rs where auto_sync runs on ordinary commands, docs/skills.md, docs/cli.md, README.md, the status row in .agents/skills/yaks/SKILL.md, the doctor advisory wording.

Judge: coordinator re-runs the gate and a temp-repo run.

Forbidden: writing into any working tree, auto-updating project-local skills, coordination skills prose.

---
▸ 2026-10-05T22:44:56Z [ntf-1]
Worker ntf-1 started.

- `ls .yaks/*/yaks-5c83.md`: `.yaks/shaving/yaks-5c83.md`
- `date -u`: Mon Oct  5 22:40:07 UTC 2026
- Own binary: `cargo build --release` -> `./target/release/yaks`.

---
▸ 2026-10-05T22:48:46Z [ntf-1]
Decision: where "announced" is remembered, and what the notice does.

- **Marker**: `<git-dir>/yaks-skills-notified` holding the yaks version announced (git dir resolved without shelling out: `.git` itself, or the `gitdir:` line of a linked worktree/submodule `.git` file, relative paths against the top-level). Untracked, outside the working tree, per clone/worktree. Rejected: a per-user cache dir (`~/.cache/yaks`): one marker would silence every checkout/Delta lane after the first, but each lane has its own skills files and must be told; keyed-by-path caches add a second index to rebuild. Rejected: a marker under `.yaks/` or `.agents/` (working tree, would show in git status, forbidden).
- **Announce rule**: print only when this yaks version is strictly newer than the marker's (`version_gt`), not merely different, so a release build and a dev build co-installed do not take turns nagging (same ping-pong reasoning as the `held` state).
- **Stale** = exactly `SkillState::Upgradable` (the `stale` word of `skills status`), over the BUNDLED skills actually present in `./.agents/skills` of the git top-level (`select_for_status(base, [])`: default set plus any installed opt-in skill; absent ones never count). A skill whose worst file is modified/unmanaged/held/source is not stale, per skill. `adoptable` (unstamped, identical to ours) is not stale: nothing is out of date. One shared `Stale::of(&[Status])` feeds the notice, `skills status` and the `doctor` advisory so the three cannot disagree.
- **Version in the line** = the oldest `from` among stale skills; `--with coordination` added when a stale skill belongs to that group (`Stale::command`).
- **Marker write failure => no notice** (read-only `.git`, e.g. sandboxes): a notice that cannot be remembered would repeat on every command. Rejected: print anyway (nags agents in sandboxes every call). `status`/`doctor` still report. The write is best-effort and never fails the command.
- **Exempt**: `skills` and `init` (they return before the hook, as before); `--json` detected as `--json` anywhere in argv (`env::args_os()`), not per command variant: the hook stays one self-contained block in main.rs and covers every present and future `--json` command without touching the arg structs another lane is editing. Cost: an argument value equal to `--json` also suppresses it once (harmless). `YAKS_SKILLS_AUTOSYNC` 0/false/never silences the notice too: one switch (`skills_notices_enabled`, shared with `auto_sync`). Rejected: a second env var. Not exempted: tui, path, preflight (stderr is not their data channel). The notice runs before `Farm::open`, so it also appears when there is no farm; it needs only a git repo.
- **Wording**: `note: the skills in .agents/skills are from yaks 0.0.12, this is 0.0.13: run \`yaks skills install\` to update them`. Prefix `note:` matches the existing auto-sync line (`note: updated the ...`) rather than the brief's `yaks:`; shared sentence is `Stale::message()`, used verbatim by the notice and the doctor advisory; `skills status` ends `N stale; run \`yaks skills install[...]\` to update them.` (was `... to upgrade.`, no test or doc depended on it).
- **doctor** now also runs the project-local check from the cwd's git top-level (it previously only looked at `~/.agents/skills` for edited skills). No exit-code effect.
- `install` after the notice: verified by test that a cleanly stale project-local skill upgrades with no `--force` (unit `rerunning_install_after_the_notice_makes_it_current_and_quiet`, integration `a_stale_install_is_announced_once_then_install_fixes_it`).

---
▸ 2026-10-05T22:49:12Z [ntf-1]
Shorn summary (ntf-1).

- Added `Stale` / `project_stale` / `project_stale_notice` (src/skills.rs) and the hook after `auto_sync` in src/main.rs; `skills status` and the doctor advisory use the same `Stale` value/words. Docs updated: docs/skills.md, docs/cli.md, README.md, status row of .agents/skills/yaks/SKILL.md, the `skills install` tail line.
- Tests: 8 unit tests in src/skills.rs (once per version, silent for current/modified/held/unmanaged/absent/source/no-repo, opt-in `--with coordination`, install fixes it, linked-worktree git dir, unwritable marker) and tests/skills_notice.rs (3 end-to-end tests: one stderr line then silence, again after marker version change, silent for --json / YAKS_SKILLS_AUTOSYNC=0|false|never / `skills`, working tree + git status + mtime unchanged, status/doctor same words, install then quiet). The 3 integration tests fail against the pre-change src (verified by stashing src/main.rs and src/skills.rs).
- Gate: `cargo test -p yaks` green (421 + 8 + 28 + 19 + 3 + 3).
- Real run in a temp repo + temp HOME: one `note:` line on the first command, nothing on the second / from a subdirectory / with --json; `git status --porcelain` and SKILL.md mtime unchanged; `skills install` upgraded the stale skill; the next command quiet.
- Surprise: the digest constant in skills.rs is `0x1000_0000_01b3` (12 hex digits), not the standard FNV-64 prime `0x100000001b3`; harmless (internal), left as is; tests copy the actual constant.

---
▸ 2026-10-05T22:49:12Z [ntf-1]
moved: shaving -> shorn
