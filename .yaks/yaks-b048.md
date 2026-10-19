---
id: yaks-b048
title: 'Rewrite the skills for one farm per machine: collapse the farm-mode and environment layers'
status: hairy
type: task
priority: 2
created: '2026-10-18T05:15:36Z'
updated: '2026-10-19T04:30:38Z'
parent: yaks-6a34
depends_on:
- yaks-7bfd
labels:
- skills
---

If the trial (yaks-6a34) holds, the private skill's runtime rules (one live farm, live claims and asks, one writer per yak, workers stage nothing under `.yaks/`) apply to every farm, and the team skill's landing rules move to "the yakherd commits the farm on `farm` and merges it into main". One farm-mode skill, with a short paragraph per remaining difference (private: ids out of git). Cold-read with a cheap scout per the skills README rule. Blocked on the trial's verdict.

---
▸ 2026-10-19T04:30:31Z [delta:herd-pass]

Audit (herd pass, 2026-10-19), against the skills at v0.0.21. Nothing in `yaks init` writes AGENTS.md boilerplate (grep: no AGENTS/CLAUDE in src/); the yaks sections in canon/hearth/j15r AGENTS.md were hand-written by agents from the skills, so fixing the skills fixes the source, and the repos need a one-time edit (see yaks-7cd1 for giving that guidance a home in the farm instead).

## What is stale
- **yaks** (bundled): Hard rule 2 and "Two workflows" still frame team mode as `.yaks/` committed beside the code, with the farm branch as an exception paragraph; `git ls-files .yaks` as THE mode signal is wrong for a branch farm (prints nothing, which reads as private). "Local-only across machines" (nested repo) and much of the hiding-method list are superseded by a never-pushed farm branch. The sheds/changes/answer@shed rows describe per-shed copies. `preflight` git-state fails on a branch farm's normal uncommitted state (seen at the 0.0.21 landing).
- **yaks-coordinating** (core): section 1 routes on `git ls-files .yaks`; section 3's watch list (`changes <shed>`, `inbox --sheds`, `show --sheds`, `answer <id>@<shed>`) and section 8's answer-in-the-shed rule assume per-shed farms.
- **yaks-coordinating-team**: everything after its first section (claim commits, worker stages `.yaks/<id>.md`, squash with human drift, asks in a shed's copy, provenance via file-follow, PR churn) is the retired model.
- **yaks-coordinating-private**: its runtime rules (one live farm, live claims/asks, one writer per yak, workers stage nothing) are now the universal rules; only "ids out of git" and the YAKS_DIR plumbing are private-specific.
- **yaks-coordinating-delta**: `yaks changes <dir id>`, `inbox --sheds`, `answer <id>@<worker>`, "in a team farm commit the claim", and the stale-yak-copy hazards; land.sh mechanics stay until yaks-e5fb.
- **yaks-coordinating-worktrees**: "team farm: cut the shed after the claim commit; the shed has its own copy of .yaks/".
- **yaks-working**: the team-mode finish (stage `.yaks/<id>.md` with the code).

## Proposed structure (four skills, from eight)
- `yaks` (bundled): one model. The farm is wherever discovery says (`yaks path`); the common case is the farm branch, reached from every checkout. Two orthogonal settings, each one paragraph: is the farm shared (pushed) or private (never pushed / out-of-tree pointer), and may ids appear in commit messages. Workflow, evidence, ask/answer, commands.
- `yaks-tracker` (bundled): unchanged.
- `yaks-working` (opt-in): one finish: never stage `.yaks/`, commit your code (ids in the message if the farm allows), the yakherd commits the farm.
- `yaks-coordinating` (opt-in): one coordinator skill; the private/team split disappears (live claims and asks everywhere), and the environment differences shrink to a section each: git worktrees (cut, file-tool pitfalls) and Delta (spawn, brief, landing by SHA via land.sh / `yaks land`, landing in the human's checkout). If the Delta section stays long, keep `yaks-coordinating-delta` as the one environment companion (it carries land.sh), and drop team/private/worktrees.

Also fold in: yaks-248d (linking yaks, now stable paths), yaks-8f2e (large-farm command-line traps), the "every surface" docs (README, docs/*, .agents/skills/README.md, BUNDLED in src/skills.rs + its guard test). Check with the README's cold-read procedure.

Order: decide yaks-9eaa's fate for `changes`/`inbox --sheds`/`answer @shed` first (or write the skills as if they are gone), so the skills never teach a command we are about to delete.
