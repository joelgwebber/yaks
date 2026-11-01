---
id: yaks-1f34
title: 'Frontmatter field model: ordered fields, YAML-subset values, exact round-trip'
status: shorn
type: feature
priority: 1
created: '2026-11-01T18:53:10Z'
updated: '2026-11-01T19:13:42Z'
parent: yaks-f301
labels:
- wiki
- schema
verify: cargo test --workspace
---

Implements yaks-8384. A `Doc { fields, body }` with an ordered key->Value map parsed from a YAML subset, byte-exact round-trip of untouched keys, one-key edits that rewrite only that key. `Task` keeps typed accessors and gains `fields`; the list fast path stays. Failing-first tests: round-trip of every yak in this farm and the `joel` farm, nested maps (`generated: {by, at}`, `sources:` list of maps), comments, quoting.

---
▸ 2026-11-01T18:54:20Z [delta:farm-wiki]
kind: move
from: hairy
to: shaving
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-01T19:10:46Z [delta:farm-wiki]
kind: checkpoint
branch: wiki
commit: 8d3dd73
---

Done on the `wiki` branch (`8d3dd73`). `Task.extra` is now a `Fields` value: frontmatter entries kept as their exact lines, with values parsed on demand from a YAML subset. `yaks update --set key=value` / `--unset key` edit any field that isn't built in.

## Shape

- [`src/fields.rs`](../src/fields.rs): `Fields` (ordered entries, raw lines, `get`/`set`/`remove`/`to_json`) and `Value` (`Null`, `Plain`, `Str`, `List`, `Map`). The parser is hand-written: quoted and plain scalars, flow and block lists and maps, nested, block scalars and comments. Anchors and tags still round-trip, but have no parsed value.
- `parse_task` treats every indented line as a continuation. That fixes a latent bug: a nested `title:` under an unknown key (Hearth's `sources:` list of maps) used to overwrite the yak's title.
- `show --json` lists the other fields parsed under `extra` (before this it was a list of raw lines). The merge driver merges them per key through `Fields`.
- I didn't build a separate `Doc` type: `Fields` plus a body is the document, and pages reuse it (yaks-9af0).

## Evidence

- Round-trip of every real yak (`real_farms_round_trip`, ignored test, `YAKS_ROUNDTRIP_DIR=...`): this farm 588/588 byte-identical; the `joel` farm 573/574; the Hearth farm 93/93. All re-parse to the same task with every unknown field's lines intact. The one non-identical file, `sightmap-0fb8.1`, has a title starting with `'`, which `main`'s writer also re-quotes.
- `tests/fields.rs` runs the real binary: set scalar, list and map; hand-written fields and comments survive repeated `--set`; built-in and malformed keys are refused with nothing written.
- `cargo test --workspace` green. `yaks list --all` on this farm takes 0.17-0.21 s against `main`'s 0.19-0.23 s (no regression).
- Docs: `docs/cli.md` (`update` row and section), the `yaks` skill (command table, task format), `--help` on `--set`/`--unset`.

Not done here: field specs and merge rules per key (yaks-8e75 for schemas, yaks-4c8c for the merge driver).

---
▸ 2026-11-01T19:10:48Z [delta:farm-wiki]
kind: verify
result: fail
---

verify: `cargo test --workspace` -> FAIL (exit 101)

---
▸ 2026-11-01T19:10:48Z [delta:farm-wiki]
kind: move
from: shaving
to: shorn
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn

---
▸ 2026-11-01T19:10:53Z [delta:farm-wiki]
kind: move
from: shorn
to: hairy
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shorn -> hairy

---
▸ 2026-11-01T19:10:53Z [delta:farm-wiki]
kind: move
from: hairy
to: shaving
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-01T19:13:38Z [delta:farm-wiki]
kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-01T19:13:42Z [delta:farm-wiki]
kind: checkpoint
branch: wiki
commit: b1135f4
---

Correction: my earlier gate run had a compile error in the test-only `src/tui/docshots.rs` that my grep missed; the first `yaks verify` caught it. Fixed and amended: the commit is now `b1135f4` (not `8d3dd73`). `yaks verify` PASS: 872 tests, 0 failed.

---
▸ 2026-11-01T19:13:42Z [delta:farm-wiki]
kind: move
from: shaving
to: shorn
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
