---
id: yaks-a838
title: 'Per-change commits: scoped commit per write, trailers, yaks undo, config commit: change|batch'
status: shorn
type: feature
priority: 1
created: '2026-11-01T18:53:11Z'
updated: '2026-11-01T21:19:00Z'
parent: yaks-f301
tags:
- wiki
- git
verify: cargo test --workspace
---

Implements yaks-63c3 (commit half). Every mutating command commits exactly its own paths under the farm lock through a temporary index; `Yaks-Actor` trailer; hand edits committed first and separately; `yaks undo`.

---
▸ 2026-11-01T20:51:43Z [delta:farm-wiki]

kind: move
from: hairy
to: shaving
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-01T21:17:07Z [delta:farm-wiki]

kind: checkpoint
branch: wiki
commit: db3ad28
---

Done on the `wiki` branch.

## Shape

- [`src/autocommit.rs`](code:src/autocommit.rs) hangs off the farm lock. `store::lock` snapshots the farm's dirty files (path to blob), and the lock's `Drop` commits whatever changed since, while the lock is still held. Every mutation already holds the lock, so every write path is covered without touching the 25 call sites.
- The commit is built in a temporary index on HEAD, so other dirty or staged files in the farm (a page being edited by hand) stay exactly as they were.
- A hand edit that a yaks write rewrites is committed first, alone, so an undo of the write never takes it.
- Every commit carries a `Yaks-Actor:` trailer.
- The TUI folds a burst of saves (same actor and files, within 5 s, unpushed, made by this process) into one commit.
- A failure never fails the command: it's reported, and `yaks commit` picks the files up later.
- `YAKS_NO_AUTOCOMMIT=1` opts out. `sync` and `commit` take the lock without auto-committing.
- `yaks init` makes the branch's first commit, `yaks: new farm`.
- `yaks undo <commits> | --by <actor> [--since 2h] [--yes]` reverts as one new commit, through the yak merge driver (registered on demand). It refuses on a dirty farm or a revert that doesn't apply.
- Commit messages now name pages (`yaks: pages wiki/x.md`), where before they said `farm config`.

## Evidence

- Scratch branch farm: `create`, `create`, `shave`, a hand edit then `update --note` on the same yak gave exactly the commits `created`, `created`, `shaving`, `hand edits to ...`, `updated`, each with its `Yaks-Actor`. An unrelated hand edit and a scratch file stayed uncommitted.
- Undo: `undo --by bot --yes` removed the bot's yak, note and page and kept alice's later note on the same yak; doctor was clean afterwards.
- TUI headless: `P 1`, `P 2`, `P 4` on one yak gave one commit (`priority: 3` to `4`), not three, and the CLI's earlier `created` commit was left alone.
- `tests/autocommit.rs` (4 tests through the binary); the sync tests now converge across two machines with per-change commits. `cargo test --workspace`: 900 passed.

## Notes for later

- `yaks status` and the TUI's uncommitted chip and `W` now matter only for hand edits; I left them.
- Auto-sync on an interval is yaks-f0db.
- The shared `yaks` farm (this farm) isn't converted until the branch lands, as with schema 7.

---
▸ 2026-11-01T21:19:00Z [delta:farm-wiki]

kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-01T21:19:00Z [delta:farm-wiki]

kind: move
from: shaving
to: shorn
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
