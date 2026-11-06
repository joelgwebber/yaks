---
id: yaks-525b
title: Attribute an agent's hand edits committed with yaks commit
status: shaving
type: feature
priority: 3
created: '2026-11-06T04:57:49Z'
updated: '2026-11-06T23:13:40Z'
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
