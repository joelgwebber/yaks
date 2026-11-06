---
id: yaks-17f5
title: 'Command: write a session note into sessions/'
status: shorn
type: feature
priority: 2
created: '2026-11-06T04:28:30Z'
updated: '2026-11-06T04:58:08Z'
parent: yaks-1c89
depends_on:
- yaks-bca6
- yaks-c5bd
tags:
- cli
- doks
verify: cargo test --test sessions && cargo test --bin yaks sessions
---

A command that writes a session note into the farm: one file under `sessions/YYYY/MM/`, `type: session-review`, in the shape settled on the design yak, committed under the actor's name like any other write. Model-free: the agent supplies the narrative, turns and lessons; yaks supplies the path, the frontmatter (actor, host, harness, day, time, verdict counts) and the commit, and prints the path.

Mechanical help worth having: list the yaks, notes and doks the actor wrote during the session (from its farm commits since a given time), so the note can cite them instead of restating them.

Keep the file shape compatible with Hearth's `session-review` notes, so the ones `import-hearth` brought in read the same.

Docs and help in the same change: `docs/cli.md`, clap help, the `yaks` skill's command table.

---
▸ 2026-11-06T04:57:02Z [herd-upkeep]
kind: move
from: hairy
to: shaving
thread: ksQQKUp3Ux8dTNqgb6cSSWpUMpLOAEX_fZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-06T04:57:12Z [herd-upkeep]
kind: verify
result: pass
---

verify: `cargo test --test sessions && cargo test --bin yaks sessions` -> PASS (exit 0)

---
▸ 2026-11-06T04:58:06Z [herd-upkeep]

Done on branch `upkeep` (505d2058): `yaks session note|touched|list|promote` ([src/sessions.rs](code:src/sessions.rs@505d2058)).

- `session note --title T --body -` checks the body (Summary, Turns with a verdict on every item, Lessons; optional Discussion and Saved; 12,000-character cap), writes `sessions/YYYY/MM/<actor>-<day>-<hhmm>-<slug>.md` with Hearth's `session-review` frontmatter plus `actor`, `harness` and the Delta `thread`, and commits it.
- Without `## Saved`, yaks writes it from `session touched`: each yak the actor created, moved or noted with a link to each note, then the doks, sources and schemas its commits wrote. The default window is since the actor's last session note, at most 12 hours.
- `doctor` now checks links in `sessions/`, and the farm commit subject names session notes.
- Follow-ups: yaks-2e00 (`refs --incoming` over doks and sessions), yaks-525b (hand edits committed with `yaks commit` carry no actor, so `touched` misses them).

Evidence: `yaks verify` PASS; the first real note is this session's own, written with the installed build (linked from yaks-1c89).

---
▸ 2026-11-06T04:58:08Z [herd-upkeep]
kind: move
from: shaving
to: shorn
thread: ksQQKUp3Ux8dTNqgb6cSSWpUMpLOAEX_fZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
