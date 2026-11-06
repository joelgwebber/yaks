---
id: yaks-2e00
title: 'refs --incoming on a yak: include doks and session notes'
status: shaving
type: feature
priority: 3
created: '2026-11-06T04:57:49Z'
updated: '2026-11-06T23:15:36Z'
parent: yaks-1c89
tags:
- cli
---

`yaks refs <id> --incoming` reads only yaks (`backrefs_in` in src/farm.rs). Doks and session notes link yaks too, now often to a single note, and a session-note review wants to see them. Today `yaks find <id> --kind sessions` stands in. Found while building yaks-17f5.

---
▸ 2026-11-06T23:12:42Z [herd-upkeep-2]
kind: move
from: hairy
to: shaving
thread: ksQQKUp3Ux8dTNqgb6cSSWpUMpLOAEX_fZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-06T23:13:39Z [herd-upkeep-2]

Claim (yakherd herd-upkeep-2, 2026-11-06): worker `refs-1`, then yaks-525b in the same thread.

- **Scope:** `yaks refs <id> --incoming` (text and `--json`) also lists doks and session notes that link or mention the yak, each with its path and line, and for a note link the note it names (`<id>#<ts>`). Yak entries are unchanged.
- **Files:** `backrefs_in` / `Farm::backrefs` in `src/farm.rs` (a new entry kind), `render_backrefs`/`backrefs_json` in `src/main.rs`; use `links::edges` (it already reads doks and `sessions/`) rather than a second scanner. Docs: `docs/cli.md` `### refs`, the `yaks` skill's Links bullet and the `yaks-upkeep` skill's line that says to use `yaks find <id> --kind sessions` instead.
- **Judge:** `cargo test -p yaks` with a CLI test: a dok and a session note citing a yak note both show up.

---
▸ 2026-11-06T23:15:36Z [refs-1]

Start Fri Nov  6 23:15:35 UTC 2026. Checkout: /Users/joel/src/yaks/.delta/worktrees/t269kax3x3s6/yaks. Yak path: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-2e00.md.
