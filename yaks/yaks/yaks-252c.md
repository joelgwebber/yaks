---
id: yaks-252c
title: Hash-checked dok edits (no silent overwrite from a stale read)
status: shorn
type: feature
priority: 1
created: '2026-11-06T04:28:31Z'
updated: '2026-11-07T00:00:24Z'
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

---
▸ 2026-11-06T23:53:00Z [doks-1]
kind: verify
result: fail
---

verify: `cargo test -p yaks` -> FAIL (exit 101)

---
▸ 2026-11-06T23:53:49Z [doks-1]
kind: verify
result: fail
---

verify: `cargo test -p yaks` -> FAIL (exit 101)

---
▸ 2026-11-07T00:00:14Z [doks-1]
kind: verify
result: pass
---

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-11-07T00:00:23Z [doks-1]

Decision: `dok show` prints the hash on stderr (`hash: <h>`) in text mode, and as a `hash` field in `--json`. Rejected: a header line on stdout (breaks `dok show | ...` consumers that expect the file text).

Decision: the hash is the sha256 of the file bytes as on disk (first 12 hex), read under the farm lock for `--if-hash`; the hash `dok set` prints is that of the text it wrote. Rejected: hashing `Dok::render()`, which differs from the file for a hand-written non-canonical dok.

Decision: `dok set` always prints `hash: <h>` on stdout after `Updated`/`No changes to`, with or without the new flags. Rejected: only when `--if-hash` or `--old` was used (an inconsistent output shape).

Decision: `--old`/`--new` take `-`/`--old-file`/`--new-file` through the existing `resolve_text`, so trailing newlines are dropped and an empty file or stdin is refused; an empty inline `--new ''` deletes. They match the body only (not frontmatter), conflict with `--body/--body-file`, must be given together, and combine with `--set/--unset/--if-hash`. Rejected: a separate edit command.

Decision: `json::dok_value` is unchanged; a new `dok_value_hashed` adds `hash`, so `list`/`query` JSON is unchanged (the hash would cost a file read per row, and yaks-39f9 owns query). `dok new` does not print a hash (not in scope).

Summary: `doks::text_hash`/`load_hashed`, `Farm::dok_hashed`, `Farm::dok_set` (+`DokEdit`, `DokSet`) refuse a stale `--if-hash` or an `--old` matching 0 or n times, writing nothing. CLI test `stale_if_hash_is_refused_and_old_new_edits_chain_by_hash` in tests/doks.rs. Docs: docs/cli.md `### dok`, `yaks` skill Doks section, `yaks-research` skill edit line, clap help. README has no dok rows (nothing to change).

Evidence: `cargo test` all green except the known `init_modes` farm_works_from_a_git_worktree... flake (fails here intermittently, passes on rerun; also passed on baseline once). One `yaks verify` failed in the `--bin yaks` unit tests under load and did not reproduce in 4 reruns; the latest `yaks verify` is PASS.

---
▸ 2026-11-07T00:00:24Z [doks-1]
kind: move
from: shaving
to: shorn
thread: ksQQKUp3Ux8dTNqgb6cSSWpUMpLOAEjUD5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
