---
id: yaks-735a
title: yaks land records landed shas on the yak; yaks commits reads recorded shas
status: shaving
type: feature
priority: 2
created: '2026-10-31T16:17:01Z'
updated: '2026-10-31T16:17:16Z'
parent: yaks-bdf9
depends_on:
- yaks-7a11
labels:
- git
- cli
---

Implements the decision on `yaks-bdf9`, using the note metadata core from `7f78a28`.

## Scope

1. **`yaks land <sha>` records what it landed.** After a successful landing, for each yak id named in the landed commit's message (same id rule as `yaks scan-ids`), append a note `kind: landed`, `commit: <landed sha>`, `source: <worker sha>` with the body `landed: <short sha> <subject>`. `--dry-run` writes nothing. A private farm keeps ids out of commits, so add a repeatable `--yak <id>` that names the yak explicitly.
2. **`yaks commits <id>` reads recorded shas first:** list every note with a `commit:` key (kind, sha, branch if any, subject resolved through `git log -1` in the code repo when the commit is present), then the message-grep fallback. Run the grep in the code repository (the checkout yaks was started from), never in the farm worktree; that is the `yaks-7a11` bug. The yak file's own history on the farm branch is labelled as farm history.
3. **Docs and help:** `docs/cli.md` (`land`, `commits`) and `--help`. Not `.agents/skills/`.

---
▸ 2026-10-31T16:17:15Z [yakherd]

moved: hairy -> shaving

---
▸ 2026-10-31T16:17:16Z [yakherd]

Claim (yakherd): done by `commits-1` after `yaks-7a11`. Evidence: a land test that checks the `kind: landed` note (commit, source); `commits` lists recorded shas first.
