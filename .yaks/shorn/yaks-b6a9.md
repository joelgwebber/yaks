---
id: yaks-b6a9
title: 'yaks status: what yak changes has git not got yet'
type: feature
priority: 2
created: '2026-10-06T21:48:15Z'
updated: '2026-10-06T21:54:59Z'
parent: yaks-ee0a
labels:
- cli
- git
---

Step 2 of the build order in yaks-ee0a (Joel, 2026-10-06): surface "dirty yaks". He keeps forgetting to commit yak updates because he forgets to `git status`. `yaks status` answers "what have I changed in the farm that git does not have yet?" in yak terms, and points at `yaks commit`.

## Behaviour
`yaks status` (read-only; nothing is staged, committed or written):
- In a TEAM farm (something under the farm directory is tracked by git) it lists the farm's uncommitted changes, ONE LINE PER YAK, in yak terms: `created`, `moved hairy -> shaving`, `notes +2`, `edited` (a field or the body changed), `removed`, plus `artifacts <id>` for a new or changed file under `artifacts/<id>/` and `config` for config.yaml. A yak with several kinds of change gets one line with all of them. Mark what is staged (`*` or a column) versus only in the working tree. Order: by id.
- It uses the SAME classification as `yaks commit` (src/commit.rs builds the generated message from the changed files): extract or share that code so the verbs cannot drift, and add a test that the status lines and the commit message agree on a fixture.
- Ends with what to do: `yaks commit` (and the message it would use), or, when a merge or cherry-pick is in progress, a line saying `yaks commit` will refuse (git forbids the partial commit) and why.
- A clean farm prints `farm clean: nothing to commit` and exits 0.
- A PRIVATE farm (nothing tracked) prints `private farm: not tracked by git, nothing to commit` and exits 0.
- `--check` exits 1 when the farm is dirty (silent apart from the same lines), for scripts and hooks; without it the exit code is 0 either way.
- `--json`: an object with `clean` (bool), `merge_in_progress` (bool), and `yaks` (array of `{id, changes: [...], staged}`) plus `other` for farm files that are not yaks (config, artifacts not tied to a yak).
- Other uncommitted files in the repo (code) are NOT listed: say `N other changed files outside the farm` in one line only if there are any, so the human knows `yaks commit` will not touch them.

## Out of scope
`--sheds` (status across other checkouts; a later step), the TUI, any write, committing.

## Evidence
Tests with temp git repos: clean; one created yak; a moved yak (status dir change); a yak with a new note; a field edit; a removed yak; a new artifact; config change; staged versus unstaged; a yak with several kinds of change on one line; private farm; a merge in progress; `--check` exit codes; `--json` shape; the agreement test with `yaks commit`. A real run in this repo (clean, then after making a note and a move). Docs parity: docs/cli.md, README.md, `.agents/skills/yaks/SKILL.md` command table, the `--help` text; the coordination skills' prose is the coordinator's (they should say to run `yaks status` before a landing or a spawn).

---
▸ 2026-10-06T21:48:23Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-06T21:48:23Z [delta-lead]
Claim (coordinator delta-lead). Owner: worker sta-1. Scope: a new src/status.rs (or the smallest home that fits), a shared classification extracted from src/commit.rs so `yaks status` and `yaks commit` agree, the `Status` subcommand and dispatch in src/main.rs, docs/cli.md, README.md, the commands table in `.agents/skills/yaks/SKILL.md`, `--help`. Judge: the coordinator re-runs the gate and runs `yaks status` in this repo before and after a note and a move. Forbidden: any write to git or the farm from `status`, changing what `yaks commit` does (only share its classification), the TUI, `--sheds`, the coordination skills' prose.

---
▸ 2026-10-06T21:48:45Z [sta-1]
Start: Tue Oct  6 21:48:42 UTC 2026. yaks path: /Users/joel/src/yaks/.delta/worktrees/yar6r29tjc78/yaks/.yaks/shaving/yaks-b6a9.md

---
▸ 2026-10-06T21:54:35Z [sta-1]
Decision: classification lives in a new `src/changes.rs` (Change, parse_status, `classify` into per-yak Kind Created/Moved/Updated/Removed + artifacts flag, git helpers, pending merge check). `commit::generate_message` and `yaks status` both call `classify`. Rejected: making `status` import private helpers from commit.rs (wrong dependency direction) and duplicating the verbs.

---
▸ 2026-10-06T21:54:35Z [sta-1]
Decision: `notes +N` / `edited` come from comparing the file with its HEAD version (`git show HEAD:<path>`), ignoring the re-stamped `updated:` line and the `moved: a -> b` transition entry a move writes. "Only appended note blocks" = notes; anything else (a field, the description, an edited existing note, or only the `updated:` stamp) = `edited`. Both read as `updated` in the commit message, so they agree. Rejected: counting the move's own note (every move would print `notes +1`), and parsing frontmatter field by field.

---
▸ 2026-10-06T21:54:35Z [sta-1]
Decision: a yak's `staged` (`*`, JSON bool) is true only when every changed path of that yak is staged with nothing pending in the working tree (so a partly staged yak shows unstaged). Rejected: a third "partial" state; `yaks commit` stages everything anyway.

---
▸ 2026-10-06T21:54:35Z [sta-1]
Decision: artifacts are one more token on the yak's own line (`artifacts`, no count), not a separate `artifacts <id>` line, since a yak with several kinds of change gets one line; an artifacts-only yak gets a line `<id>  artifacts`. `config` is config.yaml; any other non-yak farm file shows as its path. JSON adds `private`, `outside_farm` (count) and `message` to the requested keys; `other` is `[{path, staged}]`.

---
▸ 2026-10-06T21:54:35Z [sta-1]
Decision: `--check` exit 1 only on uncommitted farm changes (code outside the farm and a pending merge with a clean farm are exit 0); private farm exits 0 with or without `--check`. In a merge the final line says `yaks commit` will refuse instead of offering the message; the farm lines are still listed.

---
▸ 2026-10-06T21:54:50Z [sta-1]
verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-06T21:54:59Z [sta-1]
Shorn summary: added `yaks status [--check] [--json]` (src/status.rs) over a new shared `src/changes.rs` (classification extracted from src/commit.rs; `yaks commit` behaviour unchanged, its 8 tests pass untouched apart from the Change struct gaining `staged`). Tests: 21 unit tests in status.rs (clean, created, moved, notes, field edit, updated-stamp-only, removed, artifact, config, staged vs unstaged, several kinds on one line, staged rename, private, merge in progress, code counted not listed, json shape, writes nothing, agreement with the commit message incl. `commit --dry-run` message equality) and 2 binary tests in tests/status.rs (`--check` exit codes 0/1, clean -> dirty -> `yaks commit` -> clean, `--json`). Docs: docs/cli.md, README.md, SKILL.md command table, clap --help. Real run in this checkout: the state before the move is `yaks-b6a9  notes +1` (exit 0; `--check` exit 1); the moved state is attached as artifacts/status-after-move.txt.

---
▸ 2026-10-06T21:54:59Z [sta-1]
moved: shaving -> shorn

![status-after-move](artifacts/yaks-b6a9/status-after-move.txt)
