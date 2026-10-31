---
id: yaks-bdf9
title: 'Design: commit hashes in yaks, now that the farm has its own branch'
status: hairy
type: idea
priority: 2
created: '2026-10-31T15:18:15Z'
updated: '2026-10-31T15:18:27Z'
parent: yaks-039b
labels:
- git
- design
---

`yaks-2610` chose not to store commit hashes in yaks because, with the farm on `main`, recording a hash was itself a commit (circular). A branch farm removes that: writing a sha into a yak changes only the farm branch.

## Found while exploring: `yaks commits` is broken on branch farms

`Farm::commits` in `src/farm.rs` runs `git log --grep=<id>` and `--follow` in the farm root, which is the farm branch's worktree. So it lists `yaks: shorn …` farm commits and misses the code.

Example: `yaks commits yaks-e5fb` lists four farm commits, while the real code commit `e99754d` on `main` is not there. This needs fixing regardless of the rest.

## What recorded hashes would buy

- **Landing record.** `yaks land` already makes one squash commit with a `Squashed-from:` trailer; it could note the landed sha (and the worker sha) on the yak automatically.
- **Private farms.** They must keep ids out of commit messages, so grepping messages can never link them. A sha recorded in the yak is the only link that works there.
- **`yaks commits`** reads recorded shas first, falling back to grepping the code repo (not the farm).
- **Checkpoints for remote visibility** (sibling design yak): a worker's pushed branch and head sha.

## Caveats and open questions

- Worker shas die on squash or rebase; only the landed sha is durable. Record both, and mark which is which?
- Frontmatter (`landed: [sha]`) vs a note event vs both (see the structured comments design).
- Relation to `yaks-d695` (PRs field): the same "where is this yak's code" family of fields.

---
▸ 2026-10-31T15:18:27Z [delta:Cloud Runners]

The `yaks commits` bug found here is filed separately as yaks-7a11, so it can be fixed now, independent of this design.
