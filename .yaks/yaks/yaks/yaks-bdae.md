---
id: yaks-bdae
title: 'create --key <source-url> and list --source: find or create by source'
status: shaving
type: feature
priority: 2
created: '2026-11-04T22:31:54Z'
updated: '2026-11-04T22:51:08Z'
parent: yaks-7fa6
tags:
- cli
---

Part of the scheduling arc ([design](/doks/design/scheduling.md)). Also the cheapest upstream ask from Haks (`haks reconcile` exists only because this is missing).

- `yaks list --source <url>`: exact match on `source:`; dead yaks with `--all`; `--json` as usual.
- `yaks create --key <url>`: if any yak, in any state including dead, already has that `source:`, print it and exit 0 without creating; otherwise create with `source:` set to the key. `--json` says `created: true|false`. It never resurrects a dead yak: the caller decides.
- Takes the farm lock once, so two callers cannot both create.

Done when: tests incl. a dead and a shorn match; `--help`, `docs/cli.md`, README, and the `yaks` and `yaks-tracker` skills updated.

---
▸ 2026-11-04T22:44:22Z [sched-1]
kind: move
from: hairy
to: shaving
thread: ksQQc0mhDGDdTV-sOKzlcBdkVpLOAEMHM5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-04T22:51:08Z [sched-1]

Done on branch `scheduling`.

- `yaks list --source <url>` (also `next`, `bulk` and the other shared filters): exact match on a yak's own `source:`; dead yaks only with `--all`.
- `yaks create --key <url>`: if any yak in any state already has that `source:`, print `Exists <id>: <title>` and exit 0 with nothing written; otherwise create it with the key as its `source:`. `--json` gains `created: true|false` (only for `--key` runs, so plain `create --json` is unchanged). The lookup and create share one farm lock (`Farm::create`, `NewTask::key`, `CreateOutcome::Exists`).
- Evidence: `tests/source_key.rs` (6 tests, failed first), including shorn and dead matches that are left alone, a longer URL that only starts with the key not matching, and six concurrent `create --key` processes making exactly one yak (stable over three runs). Full `cargo test` green; clippy clean for these files.
- Docs and help: `docs/cli.md` (filter flags, `create`), README, the `yaks` skill (command table and filters), and the `yaks-tracker` skill's import-once step; clap help on `--key` and `--source`.

Decisions: `--key` conflicts with `--source` (the key is the source) and an empty key is refused; a dead or shorn match is never revived, since the watcher contract says the caller decides. The TUI create form and saved views never carry a key or source filter.
