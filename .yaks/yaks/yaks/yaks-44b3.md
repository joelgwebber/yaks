---
id: yaks-44b3
title: 'Farm wiki: pages, sources and schemas beside yaks (the data layer under a personal agent)'
status: shaving
type: feature
priority: 1
created: '2026-11-01T18:52:36Z'
updated: '2026-11-01T22:26:18Z'
labels:
- wiki
- design
---

Grow the farm from a task tracker into the data layer for in-repo research, planning and documentation: **pages**, **sources** and **schemas** beside yaks, one link grammar across them, and per-change commits. A personal agent (Hearth, later) then sits on top of a farm rather than owning its own storage.

Origin: Joel and `delta:farm-wiki`, 2026-11-01, after comparing yaks with Hearth's knowledge layer (Hearth repo, `hearth-0555`, `hearth-c858`, `hearth-1c4e`, `hearth-1ed0`).

## Decisions already made (Joel, 2026-11-01)

- **Yaks owns what can be decided from the files alone**; an agent layer owns what needs a clock, a network, a model or knowledge of who is asking. Yaks stays model-free.
- **Working knowledge lives in the farm; product docs stay on `main`.** Research, sources, decision records and design notes belong with the work that produced them, so they stay in historical context instead of drifting in `docs/research/`.
- **Port Hearth's knowledge stack into yaks (Rust).** The seam is complete: Hearth (Go) will call `yaks` for every storage interaction. Hearth itself is out of scope until this lands; Joel's `joel` work-research repo is the first user.
- **Typed pages and schemas as data**, and a light generalization of yak frontmatter so a new field (`commit:`, `due:`) is cheap. No deep normalization.
- **Per-change commits** on the farm branch (the branch exists so we can commit freely).
- **A merge driver** that keeps metadata well formed and may leave conflict markers in unstructured text for an agent to resolve.
- **Policy stays out of yaks.** A personal research repo needs no trust gates; a shared agent server adds them in its own layer.
- **Keep checkouts light.** Code checkouts should not pay for large sources or a busy farm history. Watch this through every layer.

## Code branch

All code for this family lands on the `wiki` branch of the yaks repo, never `main`, and is squash-merged once design and implementation settle. The farm itself (these yaks) stays on the shared `yaks` branch as usual.

## Layers

1. **Document model** (design, first): the design children of this yak. Each holds one open ask for Joel with a lean; work proceeds on the lean on the `wiki` branch and adjusts to the answer.
2. **Core**: frontmatter model, pages, sources, schemas, links and peek, per-change commits, page merge driver.
3. **TUI**: browse and follow links across yaks, pages, sources and code; edit pages.
4. **Workflow**: the research skill, staleness and citation checks, migrating the `joel` repo.
5. **Later**: recurring yaks and due dates; Hearth on a farm (back to `hearth-1ed0`).

## Related yaks outside the family

- `yaks-84e0` (same navigation in text and markdown viewing as in yak details) and `yaks-158d` (global back/forward): the TUI layer builds on both.
- `yaks-8f50` (derived index): search over pages wants it.
- `yaks-3e1d` (embedded web UI): a later reader for pages.
- `yaks-6a11` (note metadata, shorn) and `yaks-bdf9` (commit shas, shorn): the frontmatter design reuses their key/value model.

---
▸ 2026-11-01T18:53:40Z [delta:farm-wiki]

Plan (delta:farm-wiki, 2026-11-01): the family is filed, layer by layer, with dependencies. Design asks are open on the five children of yaks-7708; I'm implementing on the leans on the `wiki` branch of the yaks repo, starting with the field model (yaks-1f34), which everything else needs. Answers that change a lean get folded in as I go.

---
▸ 2026-11-01T18:53:40Z [delta:farm-wiki]
kind: move
from: hairy
to: shaving
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-01T22:26:18Z [delta:farm-wiki]

Fan-out, 2026-11-01 (yakherd delta:farm-wiki), all from `wiki` at a150387:

- `check-1`: yaks-c0ac, `yaks check`; claude-sonnet-5-5; [thread](delta://thread/ksQQeB1CvWFdQrqkwpKQFhpMEJLOAD0oa5IBxBCA0aLJyrhJPreUvVwLftHG).
- `sync-1`: yaks-f0db, TUI auto-sync; claude-sonnet-5-5; [thread](delta://thread/ksQQeB1CvWFdQrqkwpKQFhpMEJLOAD0otJIBxBCA0aLJyrhJPreUvVwLftHG).
- `nav-1`: yaks-84e0, detail navigation in the text viewer, the TUI layer's prerequisite; claude-opus-5-5, because it is a shared-component design inside a big TUI; [thread](delta://thread/ksQQeB1CvWFdQrqkwpKQFhpMEJLOAD0o8pIBxBCA0aLJyrhJPreUvVwLftHG).

Landed by me since the last summary: yaks-8e75 (schemas), yaks-4c8c (merge driver plus `yaks conflicts`), yaks-fd7a (attach encoding), yaks-2b68 (`yaks weight`/`yaks lfs`). The core layer (yaks-f301) is now complete.
