---
id: yaks-ec00
title: 'Research and plan: cloud runners always run a current yaks binary and current skills (one skills location, versions visible)'
status: hairy
type: task
priority: 2
created: '2026-11-01T16:44:44Z'
updated: '2026-11-02T17:44:34Z'
tags:
- skills
- agent
---

Joel (2026-11-01): make sure cloud runners always get an up-to-date `yaks` CLI and up-to-date installed skills. He has hit skills installed in a mix of `~/.agents/skills` and `./.agents/skills`, some not updated, "resulting in all manner of confusion". Runners make it worse: the home dir is ephemeral and environment setup is inconsistent, for both the home dir and the checkouts. This yak holds the research and ends in a plan; implementation goes in child yaks.

## What the tool does today (0.0.27, docs/skills.md)

- The binary embeds the skills matching its version and stamps each install (`metadata: yaks-version`, `yaks-digest`).
- Any ordinary `yaks` command installs missing skills and upgrades cleanly `stale` ones, but **only in `~/.agents/skills`**. A project-local `./.agents/skills` is never rewritten: a stale one gets a one-line stderr note, printed once per version per checkout (marker in the git dir).
- `yaks skills status` and `install` look at **one** directory (project default, `--user` or `--dir`), never both side by side.
- `held` means an install by a yaks "not older" than this one is never touched, so a dev build stamped with the release version pins its copy until the next release.
- Nothing records which yaks version an agent actually ran, so a coordinator cannot check it from the farm (this turn I could not confirm the Indent runner's version: the text it quoted has been the same since v0.0.22).

## Evidence of the confusion, on Joel's laptop today

- `~/src/rs/memcraft`: local `main` has project skills stamped **0.0.24** (eb3babe, "Committing yaks skills in-repo for all runners to use"); `origin/main` has **0.0.27** (48ed523). A checkout that has not pulled runs the old project skills against a 0.0.27 binary.
- `~/.agents/skills`: `yaks-tracker` stamped 0.0.25, the other three 0.0.27. `yaks skills status --user` with the 0.0.27 release reports three as `held` ("installed 0.0.27 is not older"): a dev build stamped 0.0.27 installed them, so the release won't replace them.
- Two `yaks` on `PATH`: `~/Library/pnpm/bin/yaks` (0.0.27) and `/opt/homebrew/bin/yaks` (0.0.22, an npm global under Homebrew's node). Which one runs depends on `PATH` order.

## Questions to answer

1. **Per harness** (Delta local, Delta cloud runner, Indent, Claude Code, Codex): where are skills discovered, which wins when the same name is in both project and user dirs, and are they read once at session start (before any `yaks` command could top them up)?
2. **On a runner:** what persists between sessions (home, checkout, `.git`), how `yaks` is installed (setup script, image, `npm i -g`, `npx`), and is that pinned or `@latest`?
3. **One source of skills per project:** should projects commit `.agents/skills` (works without a home dir, but pins a version per commit and goes stale silently) or not (user-level only, so the binary always matches, but an ephemeral home starts empty)?
4. **Version visibility:** should the farm record the binary version (in the claim/move metadata, or in `yaks brief` and `sheds`), so a coordinator sees who ran what?
5. **Floor:** a farm config `min-version` that makes an older binary warn or refuse, as schema bumps already do?

## Candidate directions (to test, not decided)

- `yaks skills status` (and `doctor`) report **every** location an agent could load (project and user), side by side, flag the same skill at different versions, and say which one each known harness would pick.
- A documented runner setup recipe: install `@j15r/yaks@latest` (or a pinned version from the project), then `yaks skills install --user`, run before the agent starts. Possibly a `yaks setup --runner` that does both and prints what it did.
- A project-local skills policy, one of: don't commit them; or commit them and have `yaks` upgrade a stale, git-clean project copy (a working-tree change, so it needs a commit story); or a committed one-line `SKILL.md` that points at the user-level install.
- `yaks brief` and `yaks whoami` print the binary version and the skill verdicts, so a worker's first step shows a mismatch before it does any work.
- Dev builds stamp a dev version (`0.0.27+<sha>`) so they never `held`-pin a release.

## Related

- yaks-0ef9 (per-user yaks state: where the stale-skills marker lives)
- yaks-7d57 (trial notes: memcraft skills already bumped to 0.0.27 on origin)
- yaks-fe33 (repeatable trials per environment: a runner trial is how to test this)
- yaks-de7a (Linux release binaries for runners' glibc)

---
▸ 2026-11-02T17:44:34Z [delta:skills-upgrade-audit]

Audit (2026-11-02, delta:skills-upgrade-audit): what the skills and the binary tell an agent about upgrading `yaks` and migrating a farm, on 0.0.30 / schema 7.

## What the skills say

- `yaks` skill, "Running yaks": `yaks`, else `npx @j15r/yaks`, else a local build. Nothing about which version, or how to upgrade.
- Command table: `migrate` = "upgrade an old schema". Task format: "A farm on an older schema is refused by every command but `yaks migrate`."
- `yaks-working`, `yaks-coordinating`: nothing on versions or migration.

## What the binary says

- Farm behind the binary: every command refuses with "run `yaks migrate` (`--dry-run` shows what changes), commit the result, then use this yaks again". Good, and enough for an agent to act on, but it never asks whether the agent *should* migrate a shared farm.
- Farm ahead of the binary: "this farm uses schema vN, newer than this yaks supports (vM). Upgrade yaks." It gives no command and no version.
- Nothing ever says a newer release exists. An agent only learns this when a schema refusal hits.

## Gaps

1. No upgrade recipe (`npm i -g @j15r/yaks@latest`, or `npx -y @j15r/yaks@latest`) in the skills or in the refusal.
2. No rule on who migrates a shared farm. Migrating and syncing makes every other machine's older binary refuse, and a long-running TUI does worse (gap 3). This is the step Joel currently gates by hand.
3. The schema gate only runs at `Farm::open`. A running `yaks tui` that auto-syncs (yaks-f0db) can pull a migrated farm and keep writing old-shape files into it. Filed separately as a bug.
4. No session-start version check: `yaks --version` against the latest release and the farm's `schema` are not in the workflow.
