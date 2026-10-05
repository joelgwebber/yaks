---
id: yaks-814a
title: 'yaks init --mode: explicit farm mode and environment setup'
type: feature
priority: 3
created: '2026-10-03T20:14:31Z'
updated: '2026-10-05T19:57:23Z'
parent: yaks-b5a0
depends_on:
- yaks-3859
labels:
- cli
needs: human
---

Require choosing the farm mode at init (team/public, private/local, out-of-tree pointer; possibly the environment, e.g. Delta) and perform the setup it implies: .gitignore or .git/info/exclude, pointer file, matching skills installed. Removes the 'figure out which mode you are in' step from the skill. Docs and --help in the same change.

---
▸ 2026-10-05T19:57:23Z [delta-lead]
Design forks before I brief a worker (coordinator delta-lead); covers yaks-3859 too since 814a depends on it. Today: 'yaks init' creates an in-tree committed farm (team) with --herd/--type/--priority; 'yaks skills install' writes yaks + yaks-tracker into ~/.agents/skills (--dir, --force). (1) INSTALL TARGET: make 'yaks skills install' write to ./.agents/skills in the git top-level when run inside a repo (where Delta and Claude load project skills), keep ~/.agents/skills behind '--user' (and --dir). My lean: yes; it prints the destination it chose. Cost: someone who runs it expecting a global install gets a project-local one; the printed destination and '--user' cover it. (2) WHICH SKILLS SHIP: the default set stays yaks + yaks-tracker. Add an opt-in group, e.g. 'yaks skills install --with coordination', for the coordination family (yaks-coordinating core + team + private + worktrees + delta, and yaks-working). The delta skill carries a script (land.sh) so BUNDLED in src/skills.rs grows from 'one SKILL.md per skill' to an explicit file list per skill (still an explicit list, never the directory contents). My lean: yes, but only after you are happy with these skills as shipped surface; they are repo-internal today ('not shipped'). Say if they should stay internal for now and only (1) and (3) go ahead. (3) INIT --MODE: add 'yaks init --mode team|private|pointer'; default team, so nothing existing breaks (a REQUIRED flag would break every script, test and doc and init also runs in plain temp dirs); init prints the chosen mode and the next step. 'private' = the farm in .yaks/ plus a '.yaks' entry in .git/info/exclude (not .gitignore: that is committed and leaks the farm's existence). 'pointer' = '--mode pointer --path <dir> [--herd <prefix>]' writes the '.yaks' pointer file discovery reads, creates the farm at <dir> if it is absent, and excludes the pointer file the same way. Delta is NOT a mode (your 2026-10-03 direction): init does not install skills itself, it prints 'yaks skills install' as the next step, and for private/pointer farms it says a Delta or worktree worker needs YAKS_DIR in its brief (the discovery bound, yaks-b4dc). Questions: (a) answer (1), (2), (3) yes/no or an alternative; (b) is a 'default team' init acceptable, or do you want the choice forced?
