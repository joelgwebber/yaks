---
id: yaks-2b68
title: 'Checkout weight: LFS attributes on the farm branch, weight report, partial-clone farm fetch'
status: shorn
type: task
priority: 2
created: '2026-11-01T18:53:12Z'
updated: '2026-11-01T22:21:39Z'
parent: yaks-f301
tags:
- wiki
- git
verify: cargo test --workspace
---

Implements yaks-9756. Measure first (pack size, commits, largest blobs) on this farm and the `joel` farm.

---
▸ 2026-11-01T22:12:17Z [delta:farm-wiki]

kind: move
from: hairy
to: shaving
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

[weight-yaks-and-joel](yaks-2b68/weight-yaks-and-joel.txt)

---
▸ 2026-11-01T22:19:22Z [delta:farm-wiki]

kind: checkpoint
branch: wiki
commit: a94991c
---

Done on `wiki`. Joel's three answers on yaks-9756: LFS for binary originals, own-repo farms recommended, and measure before doing more.

- **`yaks weight [--json]`** reports commits, the bytes a code clone fetches for the farm (objects no other branch has), the tree by kind, the largest farm-only blobs, and whether LFS is on.
- **`yaks lfs`** runs `git lfs install --local` and writes case-insensitive rules for about 45 binary extensions under `.yaks/`. It commits them together with the binaries already in the tree, converted to pointers; that commit holds only those paths. Per-change commits then write pointers on their own, because `hash-object -w` runs the clean filter. This was checked in scratch, along with a `yaks sync` that pushed the LFS objects to a bare origin.
- Docs say when to give a heavy farm its own repo (`yaks init --farm`).

## Measurements (attached)

- **yaks farm:** 804 commits. Each clone of the yaks repo fetches 4.6 MiB for it. The tree has 1.9 MiB of yaks and 6.2 MiB of attachments; 5.3 MiB of that is PNGs that `yaks lfs` would convert.
- **joel farm:** 533 commits, but only 840 KiB is farm-only. Its yaks branch shares history with `main`, which reaches 77 MiB through traveler session snapshots. So the weight in `joel` is the code history, not the farm. Its sources (about 8.8 MiB in `raw/`) arrive with the migration (yaks-5550).
- Per-change commits run on this branch only so far, so a before/after count waits until the live farms are on schema 7.

Not done, as decided: partial-clone farm fetch and ref layout, pending the numbers above. LFS is not turned on in any real farm; that is Joel's call per repo.

Tests: `tests/weight.rs` covers the report and LFS end to end, and skips when git-lfs is missing. `cargo test --workspace`: 926 passed.

---
▸ 2026-11-01T22:21:39Z [delta:farm-wiki]

kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-01T22:21:39Z [delta:farm-wiki]

kind: move
from: shaving
to: shorn
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
