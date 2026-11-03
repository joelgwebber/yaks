---
id: yaks-ec00
title: 'Research and plan: cloud runners always run a current yaks binary and current skills (one skills location, versions visible)'
status: shaving
type: task
priority: 2
created: '2026-11-01T16:44:44Z'
updated: '2026-11-03T14:30:09Z'
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

---
▸ 2026-11-02T17:44:54Z [delta:skills-upgrade-audit]
kind: ask
needs: human
---

asked: needs human

Upgrade and migrate guidance for agents, from the audit above. Four questions; my lean on each.

1. **Upgrade recipe in the skill.** Add a short "Keeping yaks current" block to the `yaks` skill's session start: `yaks --version`, then `npm i -g @j15r/yaks@latest` (or `npx -y @j15r/yaks@latest <cmd>` where a global install isn't possible) whenever it is behind the latest release or a refusal says "Upgrade yaks". Lean: yes, and put the same command in the "newer than this yaks supports" error.
2. **Who migrates a shared farm.** Options: (a) any agent migrates when refused; (b) agents never migrate a farm with a remote, they `yaks ask` and stop; (c) agents migrate only on a machine nobody sits at, and only when a yakherd/brief says so. Lean: (b) for a shared farm and (a) for a private or unpublished one, because migrating and syncing breaks every other machine's binary at once. Migrating is a coordinator's or human's step, done once, with a `--dry-run` first and its own commit.
3. **Make it automatic in the binary instead.** A cheap "a newer yaks is available" notice (once per day, cached, never on `--json`), and/or a farm `min_version` in `config.yaml` that warns an older binary before the schema bump hits. Lean: `min_version` first: it is local, needs no network, and the coordinator who migrates sets it. Leave the registry check for later.
4. **Order of operations for a schema bump**, written into `yaks-coordinating`: upgrade every machine's binary, then migrate once, then sync, then restart TUIs. Lean: yes, plus the TUI fix in yaks-9786 so a stale TUI can't corrupt the farm meanwhile.

---
▸ 2026-11-03T02:38:56Z [Joel Webber]
kind: answer
needs: agent
---

1. SGTM.
2. Let's just always defer to a human; don't try and separate private from public herds for this.
3. Agreed.
4. :thumbs-up:

---
▸ 2026-11-03T14:21:51Z [delta:skills-upgrade-audit]
kind: pickup
---

picked up

First pass (Joel's answers): upgrade recipe in skill + newer-schema error; agents never migrate, always ask a human; farm min_version warning; schema-bump order in yaks-coordinating; TUI fix yaks-9786. Children below.

---
▸ 2026-11-03T14:21:59Z [delta:skills-upgrade-audit]
kind: move
from: hairy
to: shaving
child: yaks-c2fc
thread: ksQQGwTub5pWSfWv5Hlp3m1Av5LOAD8vcpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-03T14:30:09Z [delta:skills-upgrade-audit]

First pass landed locally in 7ed9402 (yaks-c2fc, yaks-c69d, yaks-9786 shorn). Remaining from the original research questions: skills in two locations, version visibility in the farm, runner setup recipe, dev-build version stamps.
