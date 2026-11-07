---
id: yaks-1c89
title: 'Upkeep: session notes, session review and housekeeping (Hearth''s knowledge procedures in yaks)'
status: shaving
type: feature
priority: 2
created: '2026-11-06T04:27:55Z'
updated: '2026-11-07T00:00:18Z'
parent: yaks-44b3
tags:
- doks
- skills
---

Move the knowledge-*maintenance* half of Hearth into yaks. The storage half (doks, sources, schemas, `find`, `check`) landed under yaks-44b3; what Hearth's `hearth-housekeeping` and `hearth-session-notes` skills and its `session-review` job do has no yaks equivalent yet. Hearth will drop its own versions and keep only the outer agent loop, server and Hearth-specific tools.

Origin: assessment by `herd-upkeep` with Joel, 2026-11-05, comparing `.agents/skills/yaks-research` with Hearth's `internal/defaults/files/skills/` and its yaks (`hearth-e1c5`, `hearth-d740`, `hearth-6e7e`, `hearth-6e98`, `hearth-3285`).

## Rule for the split

As yaks-44b3 decided: what can be decided from the files alone is a command; judgement is a skill; the clock, model, network and trust policy stay in the harness.

## Gaps

- **Session notes:** `sessions/` is reserved, imported and searchable, but nothing writes a note and no skill asks for one.
- **Session review:** nothing finds unreviewed notes, promotes their facts into doks, or stamps `promoted:`.
- **Housekeeping:** `yaks check` finds loose ends; nothing says how to tidy them (Hearth's rules: merge look-alike kinds, fix only unambiguous formats, never choose a date or number, fix clearly renamed links, never delete or rewrite, give an account).
- **Jobs:** recurring yaks with `--wake agent` replace Hearth's job registry, but nothing registers the upkeep ones.
- **Tools an agent layer needs** on top of a farm (hash-checked edits, table row writes, ...).
- **Weekly reflection:** not built in Hearth either; decide the split.

Stays in Hearth: `core/` memory injection, protected paths and trust, fetching (web, Drive, mail), the clock, transcripts, the morning summary.

---
▸ 2026-11-06T04:28:55Z [herd-upkeep]

Filed the family (herd-upkeep, 2026-11-05):

- yaks-bca6 design: session notes in a farm (Joel's direction recorded; four details in an open ask)
- yaks-c5bd links to one note of a yak
- yaks-17f5 command to write a session note
- yaks-e58f session review mechanics
- yaks-3fa5 the `yaks-upkeep` skill (session notes, review, housekeeping, registering recurring yaks)
- yaks-fcdc tools an agent layer needs: yaks-252c hash-checked edits (p1), yaks-6cdd table row writes, yaks-39f9 query operators, yaks-80bd small gaps
- yaks-7168 design: weekly reflection

Order: design, then note links and the command, then review, then the skill. The tool gaps are independent.

---
▸ 2026-11-06T04:57:01Z [herd-upkeep]
kind: move
from: hairy
to: shaving
child: yaks-c5bd
thread: ksQQKUp3Ux8dTNqgb6cSSWpUMpLOAEX_fZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-06T05:10:09Z [herd-upkeep]

Built the session-notes path on branch `upkeep` of the yaks repo (505d2058, 40dea8f0, d78b25a7), not yet landed on `main`:

- yaks-c5bd note links, yaks-17f5 `yaks session note|touched`, yaks-e58f `session list --unreviewed|promote`, yaks-3fa5 the `yaks-upkeep` skill: all shorn, each with a `verify` PASS.
- The first real session note: [/sessions/2026/11/herd-upkeep-2026-11-06-0008-upkeep-in-yaks-assessment-note-links-ses.md](/sessions/2026/11/herd-upkeep-2026-11-06-0008-upkeep-in-yaks-assessment-note-links-ses.md).
- Installed with `cargo install --path .` from the branch, and `yaks skills install --user --force` so `~/.agents/skills` has the new skill text.
- New follow-ups: yaks-b6a7 (TUI lands on the note), yaks-2e00 (`refs --incoming` over doks and sessions), yaks-525b (actor on hand edits).

Still open here: the agent-layer tools (yaks-fcdc) and the reflection design (yaks-7168). Next step is Joel trying the review and housekeeping procedures on real examples.

---
▸ 2026-11-06T21:55:36Z [herd-upkeep]

Landed on `main` as d78b25a7 (Joel, 2026-11-06): note links, `yaks session`, and the `yaks-upkeep` skill. Installed from the main checkout with `cargo install --path .`.

The flaky `tests/init_modes.rs` test (`farm_works_from_a_git_worktree_and_a_shared_clone_of_the_sharing_repo`) also fails on the previous `main` (08b86fca), 5 of 6 runs here: it predates this work.

Still open in this family: yaks-fcdc (agent-layer tools), yaks-7168 (reflection), yaks-b6a7, yaks-2e00, yaks-525b.

---
▸ 2026-11-06T23:15:35Z [herd-upkeep-2]

Fan-out, 2026-11-06 (yakherd `herd-upkeep-2`), all from `main` at 671320f9 (which removes import-hearth, yaks-ff4c), profile default model:

- `doks-1`: yaks-252c, hash-checked dok edits ([thread](delta://thread/ksQQKUp3Ux8dTNqgb6cSSWpUMpLOAEjUD5IBxBCA0aLJyrhJPreUvVwLftHG)).
- `tools-1`: yaks-80bd, `source mark` and `dok define` ([thread](delta://thread/ksQQKUp3Ux8dTNqgb6cSSWpUMpLOAEjWoJIBxBCA0aLJyrhJPreUvVwLftHG)).
- `tui-1`: yaks-b6a7, note links land on their note ([thread](delta://thread/ksQQKUp3Ux8dTNqgb6cSSWpUMpLOAEjb85IBxBCA0aLJyrhJPreUvVwLftHG)).
- `query-2`: yaks-39f9 then yaks-6cdd ([thread](delta://thread/ksQQKUp3Ux8dTNqgb6cSSWpUMpLOAEjhJJIBxBCA0aLJyrhJPreUvVwLftHG)).
- `refs-1`: yaks-2e00 then yaks-525b ([thread](delta://thread/ksQQKUp3Ux8dTNqgb6cSSWpUMpLOAEjn95IBxBCA0aLJyrhJPreUvVwLftHG)).

I take yaks-7168 (reflection design). yaks-3f35 is another thread's; its children own `src/sessions.rs`.

---
▸ 2026-11-07T00:00:18Z [herd-upkeep-2]

Landed on `main` at 30653149 and installed (`cargo install --path .` from the main checkout; `yaks skills install --user --force`):

- yaks-ff4c (671320f9) import-hearth removed.
- yaks-b6a7 (6006e665, `tui-1`) note links land on their note in the TUI.
- yaks-80bd (22076525, `tools-1`) `yaks source mark`, `yaks dok define`.
- yaks-7168 (aad2377b, me) reflection in `yaks-upkeep`; agents read `/doks/lessons.md` at session start. Follow-up yaks-323b (`session list --since`).
- yaks-2e00 (487a2508, `refs-1`) `refs --incoming` lists doks and session notes.
- yaks-525b (30653149, `refs-1`) `yaks commit --as`.

Gate on 30653149: full `cargo test`, 1179 passed, 0 failed. Under load from running workers, `tests/sync.rs` `a_private_farm_syncs_locally_but_never_pushes` failed once and passed 3 of 3 alone: load-sensitive, like the `init_modes` one.

Still running: `doks-1` (yaks-252c), `query-2` (yaks-39f9, yaks-6cdd).
