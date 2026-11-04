---
id: yaks-63c3
title: 'Design: per-change farm commits, undo, and a merge driver for every farm document'
status: shorn
type: idea
priority: 1
created: '2026-11-01T18:52:37Z'
updated: '2026-11-04T20:03:37Z'
parent: yaks-7708
tags:
- git
- doks
---

How farm writes become commits, and how concurrent writes merge, now that the farm holds pages an agent rewrites as well as yaks.

## Today

- Commands write files under the farm lock; `yaks commit` (or `W`) commits whatever is dirty as one batch; `yaks sync` commits, pulls with the yak merge driver, and pushes.
- Attribution lives in note stamps; a batch commit can mix several actors' writes.
- `mergedriver` merges a yak field by field; a scalar both sides changed keeps ours and records theirs in a `merge-driver` note.

## Proposal: one commit per change

- Every mutating `yaks` command commits exactly the paths it wrote, while still holding the farm lock, using a temporary index so other dirty files (a human's edit in progress) are left alone. This is Hearth's scoped writer (`hearth-0555`), moved into yaks.
- Message: `yaks: <verb> <subject>` (`yaks: shave yaks-1a2b`, `yaks: page set wiki/x.md`), with trailers `Yaks-Actor: <actor>` and `Yaks-Command: <argv summary>`. When the command touches a file someone edited by hand, that edit is committed first, by itself, attributed to the git user.
- Hand edits to pages (an agent's editor tools, a human's editor) are committed by `yaks commit`, which also stays as it is for anything else dirty.
- **Undo** is `yaks undo <commit>...` or `yaks undo --by <actor> --since 2h` (git revert of those commits, refusing if a later commit touched the same paths without `--force`).
- **Sync stays manual.** No background push.
- Config: `commit: change` (default for branch farms) or `commit: batch` (today's behaviour, for anyone who wants it), in `config.yaml`.
- The TUI commits each saved edit the same way; a burst of field edits on one yak inside a few seconds can coalesce into one commit.

## Merge driver for every farm document

- Built on the field model (frontmatter design yak): each field merges by its spec's rule; unknown keys merge three-way by value.
- Prose (a page body, a yak description) merges with `git merge-file`; **if it conflicts, the conflict markers stay in the file** and the merge succeeds, so the farm stays parseable and an agent or human resolves the text afterwards. `yaks doctor` and a new `yaks conflicts` list files holding markers.
- A yak's notes log still merges as a union of notes, sorted by timestamp.
- A scalar field both sides set differently: a yak keeps ours plus a `merge-driver` note (today's rule); a page keeps ours and records theirs under a `merge_conflicts:` frontmatter list that doctor reports, since pages have no notes log.

## Costs

- Many more farm commits (this farm has about 790 today). Commits are small and pack well; the checkout-weight design yak measures it.
- A commit per command adds a few git processes (tens of ms) to each write; agents won't notice, and the TUI is the place to measure.

---
▸ 2026-11-01T18:53:39Z [delta:farm-wiki]

kind: ask
needs: human
---

asked: needs human

1. **Per-change commits by default on branch farms**, `commit: batch` in `config.yaml` to keep today's behaviour. Lean: yes, and convert this farm too.
2. **Prose conflicts leave markers in the file** (the merge itself succeeds), listed by `yaks conflicts` and doctor. Lean: yes, for page bodies and yak descriptions; notes stay a union and fields follow their merge rule, so frontmatter is never left with markers.
3. **TUI coalescing:** a burst of edits to one document within a few seconds becomes one commit. Lean: yes, about 5 seconds.

---
▸ 2026-11-01T20:07:39Z [Joel Webber]

kind: answer
needs: agent
---

1. I think that's fine, and keeps things nice and simple. Don't bother with the batch behavior -- if this becomes problematic, we can add it back in, and/or give the CLI flags to control batching when really needed for large changes.
2. Exactly. I think this will be a very effective pattern, giving us the flexibility to always merge forward without creating metadata inconsistencies, allowing humans and agents to fix up semantic conflicts later.
3. That's fine as well -- we can see where that gets us, and tweak as needed. It would also be good to auto-sync with some reasonable regularity, so upstream changes get reflected even in the absence of local ones.

---
▸ 2026-11-01T20:09:15Z [delta:farm-wiki]

kind: pickup
---

picked up

Decision (Joel, 2026-11-01):

1. Per-change commits always, **no `commit: batch` mode**. If it hurts, add CLI flags for batching large changes later.
2. Prose conflicts keep their markers; fields merge by rule, so frontmatter stays well formed. `yaks conflicts` and doctor list the files.
3. TUI coalescing of a burst of edits (about 5 s) into one commit.
4. **New:** the TUI auto-syncs on a regular interval, so upstream changes show up without local ones. Filed under yaks-a838's family.

---
▸ 2026-11-01T20:09:15Z [delta:farm-wiki]

kind: move
from: hairy
to: shorn
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shorn
