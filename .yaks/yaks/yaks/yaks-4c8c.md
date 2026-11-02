---
id: yaks-4c8c
title: 'Merge driver for every farm document: field rules, prose conflict markers kept, yaks conflicts'
status: shorn
type: feature
priority: 2
created: '2026-11-01T18:53:11Z'
updated: '2026-11-01T21:58:59Z'
parent: yaks-f301
depends_on:
- yaks-1f34
- yaks-a838
tags:
- wiki
- git
verify: cargo test --workspace
---

Implements yaks-63c3 (merge half) on yaks-1f34: per-field merge rules, `git merge-file` for bodies leaving markers in place, `merge_conflicts:` for pages, `yaks conflicts` and a doctor check.

---
▸ 2026-11-01T21:40:37Z [delta:farm-wiki]

kind: move
from: hairy
to: shaving
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-01T21:56:47Z [delta:farm-wiki]

kind: checkpoint
branch: wiki
commit: c3cfecc
---

Done on the `wiki` branch.

## What it does

Joel's rule from yaks-63c3: fields merge by rule and never get conflict markers; prose may keep them for a person or agent to resolve.

- **Yaks:** the description now merges line by line (`git merge-file`), so edits to different paragraphs both land. Same-line edits keep both versions between `<<<<<<< here` and `>>>>>>> other side`, and a `merge-driver` note says so. Any list field both sides changed merges as a set.
- **Pages and any other farm markdown:** frontmatter merges by the same field rule (one `merge_fields` for both). A value both sides set differently keeps ours, and `{field, theirs}` goes under `merge_conflicts:`. The body merges as text.
- **Routing is by path**: a page that happens to carry `id:` never merges as a yak.
- **`yaks conflicts [--json]`** lists marker regions (fences skipped) and `merge_conflicts:` entries, and exits 1 if any. Doctor reports `merge-conflict`.

## Evidence

Real `yaks sync` between two clones of a bare origin, built from this branch. Clone `a` set `status: review`, tag `pg`, and plan Y. Clone `b` set `status: final`, tag `fts`, a new intro, and plan Z. Merged page on `b`:

```
status: review
tags: [db, pg, fts]   (block style as written)
merge_conflicts:
  - {field: status, theirs: final}
...
A better intro.

<<<<<<< here
The plan is Y.
=======
The plan is Z.
>>>>>>> other side
```

`yaks conflicts` printed `wiki/topics/search.md: conflict markers at L17; merge_conflicts: status (...)` and exited 1, and doctor printed the same. During a rebase, `here` is the remote's side, which is documented.

Tests: `tests/merge_driver.rs` gains a page case through a real `git merge`, and 21 unit tests are in `mergedriver::tests`. `cargo test --workspace`: 920 passed.

---
▸ 2026-11-01T21:58:59Z [delta:farm-wiki]

kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-01T21:58:59Z [delta:farm-wiki]

kind: move
from: shaving
to: shorn
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
