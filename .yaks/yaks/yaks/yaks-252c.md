---
id: yaks-252c
title: Hash-checked dok edits (no silent overwrite from a stale read)
status: shaving
type: feature
priority: 1
created: '2026-11-06T04:28:31Z'
updated: '2026-11-06T23:50:34Z'
parent: yaks-fcdc
tags:
- cli
- doks
---

Hearth's `wiki_read` returns a hash and `wiki_edit` refuses an edit whose `base_hash` is stale; yaks has nothing like it. The farm lock orders writes, but `dok set --body` built from a stale read replaces a newer body without a word. With several agent sessions on one farm this loses work.

Lean: `dok show` reports the file's hash, and `dok set` (and any whole-body write) takes `--if-hash <h>`, refusing with a message naming the current hash when it differs. Consider an exact-text replace (`--old/--new`, must match once) so agents never rewrite a whole body.

---
▸ 2026-11-06T23:12:44Z [herd-upkeep-2]
kind: move
from: hairy
to: shaving
thread: ksQQKUp3Ux8dTNqgb6cSSWpUMpLOAEX_fZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-06T23:13:37Z [herd-upkeep-2]

Claim (yakherd herd-upkeep-2, 2026-11-06): worker `doks-1`.

- **Scope:** `dok show` reports the file's hash (text: a `hash:` line on stderr or a header; `--json`: a `hash` field), and `dok set` takes `--if-hash <h>`, refusing (exit 1, nothing written) with the current hash when the file differs. Also an exact-text edit: `dok set <path> --old <text> --new <text>` (`-` reads stdin for one of them, or `--old-file`/`--new-file`), which must match exactly once in the body, else refused saying how many matched. It prints the new hash so a second edit can chain. Hash: sha256 of the file bytes, first 12 hex (say so in help).
- **Files:** `src/doks.rs`, `Farm::dok_set` in `src/farm.rs`, `DokAction::Show`/`Set` in `src/main.rs`, `json::dok_value`; docs: `docs/cli.md` `### dok`, the `yaks` skill's Doks section and the `yaks-research` skill's "Edit an existing dok" line.
- **Out of scope:** yak bodies (`update --description` already has its own base check), table rows (yaks-6cdd), query (yaks-39f9).
- **Judge:** `cargo test -p yaks`, plus a CLI test in `tests/doks.rs` showing a stale `--if-hash` refused and an `--old/--new` edit chained by the printed hash. The yakherd re-runs it at landing.

---
▸ 2026-11-06T23:14:41Z [doks-1]

Start: 2026-11-06T23:14:38Z

- yaks path yaks-252c: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-252c.md
- checkout: /Users/joel/src/yaks/.delta/worktrees/sr37a33ha7db/yaks

---
▸ 2026-11-06T23:27:34Z [doks-1]
kind: verify
result: fail
---

verify: `cargo test -p yaks` -> FAIL (exit 101)

---
▸ 2026-11-06T23:39:08Z [doks-1]
kind: verify
result: fail
---

verify: `cargo test -p yaks` -> FAIL (exit 101)

---
▸ 2026-11-06T23:50:34Z [doks-1]
kind: verify
result: pass
---

verify: `cargo test -p yaks` -> PASS (exit 0)
