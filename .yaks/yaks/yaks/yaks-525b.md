---
id: yaks-525b
title: Attribute an agent's hand edits committed with yaks commit
status: shorn
type: feature
priority: 3
created: '2026-11-06T04:57:49Z'
updated: '2026-11-06T23:52:13Z'
parent: yaks-1c89
tags:
- cli
---

`yaks commit` writes no `Yaks-Actor` trailer for hand edits (they are the person's by design), so a dok an agent edited in its editor and committed is missing from `yaks session touched` and `undo --by`. The yaks-upkeep skill tells agents to list such edits in `## Saved` themselves. Option: `yaks commit --as <actor>` (or the checkout's whoami name when it is an agent's) stamps them. Found while building yaks-17f5.

---
▸ 2026-11-06T23:12:42Z [herd-upkeep-2]
kind: move
from: hairy
to: shaving
thread: ksQQKUp3Ux8dTNqgb6cSSWpUMpLOAEX_fZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-06T23:13:40Z [herd-upkeep-2]

Claim (yakherd herd-upkeep-2, 2026-11-06): worker `refs-1`, after yaks-2e00, in the same checkout.

- **Decision (yakherd, scope):** explicit only. `yaks commit --as <actor>` stamps the hand-edit commit with a `Yaks-Actor: <actor>` trailer (and the identity `identity_env` gives that actor). Without `--as` nothing changes: hand edits stay the person's, as designed. Rejected: stamping the checkout's `whoami` name by default, since a human can set one too and the commit would then claim their edits for whoever ran it.
- **Scope:** `Command::Commit` in `src/main.rs`, `src/commit.rs`/`src/autocommit.rs` as needed. Then `yaks session touched` and `undo --by` see those commits with no further change (check it). Docs: `docs/cli.md` `### commit`/`Every change commits`, the `yaks` skill's farm block, the `yaks-upkeep` skill's `## Saved` bullet (which tells agents to list hand edits themselves: say to commit them with `yaks commit --as <you>` instead).
- **Judge:** `cargo test -p yaks` with a CLI test: a hand-edited dok committed with `--as tester` appears in `yaks session touched --by tester`.

---
▸ 2026-11-06T23:40:30Z [refs-1]

Start (after yaks-2e00, commit a698509). Implemented `yaks commit --as <actor>`: `commit::run_as` (`run` delegates with None) adds a `Yaks-Actor` trailer as a final `-m` paragraph and uses the actor for `identity_env`; `Plan.actor`, output says `(as <actor>)`. CLI test `commit_as_stamps_a_hand_edit_for_touched_and_undo` in tests/autocommit.rs: a hand-edited dok committed `--as tester` shows in `session touched --by tester` and `undo --by tester`, a plain commit's hand edit does not. No change needed in src/sessions.rs.

Decision: a new `run_as` beside `run` rather than a new parameter on `run`: five callers (init, sync, TUI, status, main) are unchanged. Rejected: threading `Option<&str>` through all of them.

Decision: `--as` that cleans to empty (`--as " "`) is an error, not a fall back to $YAKS_ACTOR. Rejected: fallback, since stamping an unintended name is exactly what the explicit-only decision avoids.

Decision: the trailer is a second `-m` paragraph, and a dry run prints `(as X)` after the unchanged subject; `plan.message` stays the subject only. Rejected: `git commit --trailer` (needs git 2.32).

---
▸ 2026-11-06T23:52:01Z [refs-1]
kind: verify
result: pass
---

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-11-06T23:52:12Z [refs-1]

Summary: `yaks commit --as <actor>` stamps a hand-edit commit with `Yaks-Actor: <actor>` (explicit only; plain commit unchanged). Files: src/commit.rs, src/main.rs, tests/autocommit.rs (CLI test), docs/cli.md, README.md, yaks and yaks-upkeep skills. Full `cargo test` green; `yaks verify` PASS.

---
▸ 2026-11-06T23:52:13Z [refs-1]
kind: move
from: shaving
to: shorn
thread: ksQQKUp3Ux8dTNqgb6cSSWpUMpLOAEjn95IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
