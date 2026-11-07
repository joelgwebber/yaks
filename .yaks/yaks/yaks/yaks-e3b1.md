---
id: yaks-e3b1
title: The --json shapes as a stated contract for harnesses
status: hairy
type: idea
priority: 3
created: '2026-11-07T16:57:33Z'
updated: '2026-11-07T16:57:33Z'
parent: yaks-44b3
tags:
- cli
---

Two harnesses now build on yaks's `--json` output: Haks ([the Haks dok](/doks/research/haks-orchestration-layer.md), which pins yaks `main` daily) and Hearth (hearth-1ed0 in the Hearth farm, which will call `tick --json`, `inbox --json`, `dok show --json`, `find --json`, `list --json`). A changed shape breaks them silently.

Proposal:

- Say in `docs/cli.md` which `--json` shapes are a contract (fields only added, never renamed or removed without a release note) and which are diagnostic.
- Add a top-level `yaks --version --json` (or `yaks api`) reporting the version and a contract number a harness can check at start.
- A test per contract shape that fails when a field disappears.

Open question carried from the Haks dok: is this worth doing before a second outside consumer exists? Hearth makes two.
