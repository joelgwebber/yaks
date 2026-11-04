---
id: yaks-bdae
title: 'create --key <source-url> and list --source: find or create by source'
status: hairy
type: feature
priority: 2
created: '2026-11-04T22:31:54Z'
updated: '2026-11-04T22:31:54Z'
parent: yaks-7fa6
tags:
- cli
---

Part of the scheduling arc ([design](/doks/design/scheduling.md)). Also the cheapest upstream ask from Haks (`haks reconcile` exists only because this is missing).

- `yaks list --source <url>`: exact match on `source:`; dead yaks with `--all`; `--json` as usual.
- `yaks create --key <url>`: if any yak, in any state including dead, already has that `source:`, print it and exit 0 without creating; otherwise create with `source:` set to the key. `--json` says `created: true|false`. It never resurrects a dead yak: the caller decides.
- Takes the farm lock once, so two callers cannot both create.

Done when: tests incl. a dead and a shorn match; `--help`, `docs/cli.md`, README, and the `yaks` and `yaks-tracker` skills updated.
