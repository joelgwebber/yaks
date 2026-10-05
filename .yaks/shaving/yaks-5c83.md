---
id: yaks-5c83
title: 'yaks: tell the user once per release when project-local skills are stale (decision C of yaks-36b6)'
type: feature
priority: 2
created: '2026-10-05T22:39:41Z'
updated: '2026-10-05T22:39:49Z'
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
