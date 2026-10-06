---
id: yaks-2c22
title: 'Process rails: yaks brief (worker brief from a yak) and yaks preflight (landing checks)'
type: feature
priority: 2
created: '2026-10-03T21:13:29Z'
updated: '2026-10-06T03:58:20Z'
parent: yaks-b5a0
labels:
- cli
- skills
- agent
---

Prose rules in the skills have not kept agents on the rails (yaks-df61 O5, O6, O9). Candidate commands that take the checklist out of prose: (1) yaks brief <id> --as <actor>: print a worker brief from the yak itself (id, how to get a binary, actor, scope from notes, evidence contract = verify command or required attachments, forbidden moves, commit and return format); (2) yaks preflight [<id>...]: before landing, check each shorn yak has its verify last PASS, no untracked or unstaged files under .yaks/ (evidence not staged), the yak file and code are in the same commit, and the commit message names the id; exit non-zero with one actionable line per failure. Design test (yaks-coordinating): both are expressible as queries over files plus git status, so they may live in the tool; anything that assumes Delta or worktrees stays in the skill. Open: whether these generalise beyond Delta, and whether brief should be a template skill instead. Decide with Joel before building.

---
▸ 2026-10-03T22:48:20Z [delta-lead]
Design fork: build 'yaks brief <id>' and 'yaks preflight' in the tool, or keep the checklist in a skill? My lean: build 'preflight' first (pure file + git-status queries: verify last PASS, nothing untracked or unstaged under .yaks/, duplicate status dirs, yak move and code in one commit, id in the message), since trials 1-3 showed agents and coordinators miss exactly those checks and doctor already catches duplicate status dirs. Defer 'brief' until the skills restructure (yaks-5c9f) settles what a brief contains, because the template is still changing. Alternative: skill-only. Answer: preflight now / brief later / skill-only.

---
▸ 2026-10-03T23:19:12Z [Joel Webber]
Agreed on both. The preflight gate will be a big help.

---
▸ 2026-10-05T22:44:37Z [delta-lead]
Design fork before I brief a worker (coordinator delta-lead). You agreed on 2026-10-03 to build `yaks brief <id>`; what is open is WHERE the template lives.

Today the fill-in worker brief is prose in `.agents/skills/yaks-coordinating-delta/SKILL.md`. Building the command puts a second copy in the binary, so one of them has to be the source of truth.

## Options
- **A. The binary owns the yak-derived core, the skill owns orchestration.** `yaks brief <id> --as <name>` prints what is derivable from the yak and the farm:
  - the yak id and title, "already `shaving`", the `YAKS_ACTOR=<name>` prefix rule
  - how to get a binary, scope pointers (read `yaks show`), the evidence contract (the yak's `verify:` command, required attachments)
  - forbidden moves and how to finish, chosen by farm mode (team: stage `yaks path <id>` plus artifacts; private: no yak ids in commits)
  - the `Decision:` note rule, the multi-line markdown rule, and the final-message format
  - `--yaks-dir <path>` adds the `YAKS_DIR=` line for private/pointer farms
  The Delta skill's template shrinks to "run `yaks brief`, then add the spawn title and anything environment-specific", so the two cannot drift.
- **B. The skill stays the only template**; `yaks brief` prints it from a copy embedded at build time (same text, generated). Single text, but it ties the binary to the optional coordination skills.
- **C. Do not build it**; keep the template in the skill and rely on the cold-read gate.

## Lean
A. It follows the design test in `yaks-coordinating` section 2: what is true of working a given yak is a query over the farm and belongs in the tool; spawning, titles and landing are orchestration and stay in the skill. It also fixes the failure the observation log keeps showing (O5, O6, O9): a coordinator forgets a clause when it fills a template by hand.

Cost: a user-visible command and a stable-ish output format, and the Delta skill must be edited to point at it. Say A, B, C or your own variant.

---
▸ 2026-10-06T03:40:48Z [Joel Webber]
I'm good with putting these into the tools now. Can't always trust those sub-agents.

---
▸ 2026-10-06T03:57:00Z [delta-lead]
Recording the answer (coordinator delta-lead, on Joel's note above, 2026-10-06T03:40Z). Joel wrote: "I'm good with putting these into the tools now. Can't always trust those sub-agents."

I read that as option A: the binary owns the yak-derived core of the worker brief (`yaks brief <id> --as <name>`), the Delta skill keeps orchestration and shrinks to a pointer. The reason he gives is that a prose template filled in by hand gets clauses dropped. If that is not what you meant, say so and I will redo the brief before landing.

Clearing `needs` because the reply was a plain note, which leaves the block set (this is exactly the gap in yaks-c061).

---
▸ 2026-10-06T03:58:20Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-06T03:58:20Z [delta-lead]
Claim and settled design (coordinator delta-lead). Owner: worker brf-1. Joel: "I'm good with putting these into the tools now" (read as option A, recorded above): the binary owns the yak-derived core of the worker brief; the Delta skill keeps orchestration and its template shrinks to a pointer (the coordinator edits the skill after landing).

## `yaks brief <id> --as <name> [--yaks-dir <path>]`
Prints the worker brief to stdout (plain text, deterministic). It errors (non-zero, nothing on stdout) if the yak does not exist, and if it is not `shaving` it prints the brief but warns on stderr that the coordinator must claim it first. Contents, each derivable from the yak, the config or the farm:
1. The yak id and title, and that it is already `shaving`; do not shave, shear, regrow or edit any other yak.
2. Every yaks command prefixed `YAKS_ACTOR=<name>` (a fresh shell per terminal call). With `--yaks-dir <path>` (or when the farm is out-of-tree/pointer or private) also the `YAKS_DIR=<path>` line for every command.
3. How to get a binary: say to use the binary the coordinator names; default to the absolute path of the running executable (`current_exe`) and add "never npx or an installer".
4. Read the yak first: `yaks show <id>` and every note; the scope, the judge and the evidence contract live in the claim note.
5. Evidence: the yak's own `verify:` command (resolved with the config default by label, as `yaks verify` does) and any attachment the yak requires; "paste real output; claim nothing unobserved".
6. Forbidden moves: edits to Cargo.toml/Cargo.lock/.gitignore/config, path dependencies, `git push`, installs, history rewrites.
7. How to finish, chosen by farm mode (detect as the skills do: tracked `.yaks` = team): TEAM: `yaks shorn <id>` (there is no `shear` subcommand) after the evidence exists, ONE commit with explicit paths including `yaks path <id>` output, the removed old status path and `.yaks/artifacts/<id>` if anything was attached (a missing pathspec aborts `git add`), message `<id>: ...`. PRIVATE (untracked or out-of-tree farm): no yak ids or the word yaks in commits, stage only your own files, plain-English message.
8. Blocked or a decision needed: `yaks ask <id>`, leave edits in the working tree, never revert, never park in $TMPDIR, say which in the final message.
9. Notes: record every choice a reviewer could argue about as a note starting `Decision:` naming the rejected alternatives; multi-line notes as markdown piped on stdin (`--note - <<'EOF'`), not one long argument (the same text as the yaks skill).
10. The final-message format: commit SHA and `git show --stat`; gate command and last lines; start/finish `date -u`; the Decision notes in one line each; docs-parity grep; anything surprising.
It must NOT contain orchestration the yak cannot know (spawn titles, Delta landing, which model); the coordinator adds those around it.

## Constraints
- The text lives in one place in the binary (src/brief.rs is fine), is built from small pieces so mode/yak differences are explicit, and is covered by tests (team, private, pointer/--yaks-dir, non-shaving warning, unknown id, `verify:` present/absent).
- `--json` is optional; if you add it, same sections as keys.
- Docs parity: docs/cli.md, README.md, `.agents/skills/yaks/SKILL.md` command table, `--help`. Leave `.agents/skills/yaks-coordinating*` and `yaks-working` to the coordinator, but report which sentences of the brief template in the Delta skill the command now replaces.
- Evidence: gate green; a real run of `yaks brief` for a shaving yak in a team farm and in a private farm (paste both); then check the output against the Delta skill's current template and list any clause present in one and missing in the other.
Judge: coordinator re-runs the gate and gives the output to a fresh scout as a worker brief.
