---
id: yaks-2e00
title: 'refs --incoming on a yak: include doks and session notes'
status: shaving
type: feature
priority: 3
created: '2026-11-06T04:57:49Z'
updated: '2026-11-06T23:18:30Z'
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

---
▸ 2026-11-06T23:18:30Z [refs-1]

Progress: implemented. `Farm::backrefs` now also fills `TaskBackRefs.docs` (new `DocBackRef {path, line, note}`) from `links::edges`, filtered to dok paths and `sessions/`; `render_backrefs`/`backrefs_json` print/emit them. CLI test `refs_incoming_lists_doks_and_session_notes` in tests/links.rs passes. Docs/help/skills updated. Next: full gate.

Decision: doks/session notes go in a separate `docs` list (JSON key `docs`, text lines after the yaks), not as a new `BackRefVia` variant or extra `incoming` objects. Rejected: a `BackRefVia` variant, since `BackRef` needs a yak id/title/status and the TUI picker (`backrefs_in`) consumes it; extra `incoming` objects without an `id` would break JSON consumers of the yak entries.

Decision: only markdown links count (what `links::edges` reads, as the claim says), so a bare id in a dok's prose is not listed. `yaks find <id> --kind sessions` still finds those; the yaks-upkeep skill line says so. Rejected: also running `refs::scan` over dok bodies: a second scanner, and the claim named edges.
