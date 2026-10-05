---
id: yaks-814a
title: 'yaks init --mode: explicit farm mode and environment setup'
type: feature
priority: 3
created: '2026-10-03T20:14:31Z'
updated: '2026-10-05T21:16:42Z'
parent: yaks-b5a0
depends_on:
- yaks-3859
labels:
- cli
---

Require choosing the farm mode at init (team/public, private/local, out-of-tree pointer; possibly the environment, e.g. Delta) and perform the setup it implies: .gitignore or .git/info/exclude, pointer file, matching skills installed. Removes the 'figure out which mode you are in' step from the skill. Docs and --help in the same change.

---
▸ 2026-10-05T19:57:23Z [delta-lead]
Design forks before I brief a worker (coordinator delta-lead); covers yaks-3859 too since 814a depends on it. Today: 'yaks init' creates an in-tree committed farm (team) with --herd/--type/--priority; 'yaks skills install' writes yaks + yaks-tracker into ~/.agents/skills (--dir, --force).

1. INSTALL TARGET: make 'yaks skills install' write to ./.agents/skills in the git top-level when run inside a repo (where Delta and Claude load project skills), keep ~/.agents/skills behind '--user' (and --dir). My lean: yes; it prints the destination it chose. Cost: someone who runs it expecting a global install gets a project-local one; the printed destination and '--user' cover it.
2. WHICH SKILLS SHIP: the default set stays yaks + yaks-tracker. Add an opt-in group, e.g. 'yaks skills install --with coordination', for the coordination family (yaks-coordinating core + team + private + worktrees + delta, and yaks-working). The delta skill carries a script (land.sh) so BUNDLED in src/skills.rs grows from 'one SKILL.md per skill' to an explicit file list per skill (still an explicit list, never the directory contents). My lean: yes, but only after you are happy with these skills as shipped surface; they are repo-internal today ('not shipped'). Say if they should stay internal for now and only (1) and (3) go ahead.
3. INIT --MODE: add 'yaks init --mode team|private|pointer'; default team, so nothing existing breaks (a REQUIRED flag would break every script, test and doc and init also runs in plain temp dirs); init prints the chosen mode and the next step. 'private' = the farm in .yaks/ plus a '.yaks' entry in .git/info/exclude (not .gitignore: that is committed and leaks the farm's existence). 'pointer' = '--mode pointer --path <dir> [--herd <prefix>]' writes the '.yaks' pointer file discovery reads, creates the farm at <dir> if it is absent, and excludes the pointer file the same way. Delta is NOT a mode (your 2026-10-03 direction): init does not install skills itself, it prints 'yaks skills install' as the next step, and for private/pointer farms it says a Delta or worktree worker needs YAKS_DIR in its brief (the discovery bound, yaks-b4dc).

Questions: (a) answer (1), (2), (3) yes/no or an alternative; (b) is a 'default team' init acceptable, or do you want the choice forced?

---
▸ 2026-10-05T20:51:41Z [Joel Webber]
1. Yup. 2. Also yup; install them optionally initially. 3. Should we not consider rolling init/skills together?

---
▸ 2026-10-05T21:08:36Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-05T21:08:36Z [delta-lead]
Claim and settled design (coordinator delta-lead). Joel's answers: yaks-3859 decisions landed (c568e09: project-local install by default, --user, opt-in --with coordination). On rolling init and skills together Joel (inline comment, 2026-10-05): 'That works for me. If we do make the skills optional, then we should at least tell people after a plain yaks init that they still need to do this. Even better if they can repeat the init idempotently with the necessary flag.' DESIGN: yaks init [--mode team|private|pointer] [--path <dir> for pointer] [--skills none|default|coordination] plus today's --herd/--type/--priority. (1) NO --mode and NO --skills: exactly today's behaviour (farm only, team) and it ends by printing that skills are not installed and the exact command to run, e.g. 'yaks init --skills default' (safe to re-run) or 'yaks skills install'. (2) --mode and/or --skills given = the full setup: mode setup, then skills via the shared install function from yaks-3859 (project-local in the git top-level; the default set unless --skills none; --skills coordination adds the group). --skills alone means mode team. (3) Modes: team = farm in .yaks/ committed. private = farm in .yaks/ plus a '/.yaks' line in .git/info/exclude (NOT .gitignore: committed, leaks the farm). pointer = --path <dir> [--herd <prefix>]: write the '.yaks' pointer file discovery reads (see store::discover and the existing pointer format), create the farm at <dir> if absent, exclude the pointer file via .git/info/exclude. private and pointer need a git repo (exclude lives in the git dir). (4) Skills in private/pointer modes must not enter shared history: add the installed skill directories to .git/info/exclude too. In team mode they are plain project files the user may commit; say so. (5) IDEMPOTENT: re-running with the same flags changes nothing and says so; with more flags it adds only what is missing (an exclude line, a skill, the pointer); never overwrites an edited skill (same rules as 'skills install' without --force); a DIFFERENT --mode on an existing farm is an error naming what would have to change, never a silent conversion. 'yaks init --skills default' on an existing team farm just installs skills. (6) It prints one line per step done or already-done, and for private/pointer says a Delta or worktree worker needs YAKS_DIR in its brief (yaks-b4dc). Delta is NOT a mode and init never asks about the environment. Owner: worker ini-1. Judge: coordinator re-runs the gate and drives each mode in temp repos (git status --short, yaks list from a subdirectory, a second run). Out of scope: auto-updating project-local skills (yaks-36b6, asked), the coordination skills' prose (coordinator updates it).

---
▸ 2026-10-05T21:09:14Z [ini-1]
ini-1 start: ls .yaks/shaving/yaks-814a.md; date -u Mon Oct 5 21:08:53 UTC 2026

---
▸ 2026-10-05T21:16:42Z [ini-1]
Done. New src/init.rs (plain init unchanged + skills hint; setup = modes team/private/pointer, --skills none|default|coordination, idempotent, one line per step, .git/info/exclude edits incl. linked worktrees via git rev-parse --git-path, YAKS_DIR note). skills.rs: render_installed (extracted from run_skills, same output) so init and skills install share the printing; skills::install reused as is. store.rs: parse_pointer/expand_path/resolve_farm_root made pub(crate), discovery untouched. Judgment calls: (a) --skills alone on an EXISTING farm keeps that farm's mode (private stays private) rather than asserting team; fresh = team; (b) --path DIR: farm is DIR if it is a farm or named .yaks, else DIR/.yaks; pointer path: written as given; (c) settings flags contradicting an existing farm's config.yaml are an error (init never rewrites config); (d) --skills outside git is an error, nothing written; (e) exclude line goes in before the farm is created. Evidence: tests/init_modes.rs 19 tests, all 19 fail on HEAD (pre-change worktree) and pass now; cargo test -p yaks green (414+8+28+19+3). Real runs in temp repos done (private + repeat, pointer). Docs: docs/cli.md, docs/README.md, README.md, yaks SKILL.md, --help. Out of date in coordination skills: yaks-coordinating-private SKILL.md line ~24-25 says 'yaks init --mode private writing it is a separate, unshipped change'; its farm-mode setup prose could now point to init --mode private/pointer, and delta/worktrees skills may cite the printed YAKS_DIR note. finish date -u Mon Oct 5 21:16 UTC 2026.

---
▸ 2026-10-05T21:16:42Z [ini-1]
moved: shaving -> shorn
