---
id: yaks-43dc
title: 'Skills: reconcile pointer-farm shared/private and commit-message id rules'
status: hairy
type: task
priority: 3
created: '2026-10-26T21:00:47Z'
updated: '2026-10-27T22:29:32Z'
parent: yaks-a99f
labels:
- skills
---

Found by the cold-read for yaks-8aeb (2026-10-26), predates it:

- The yaks skill calls a pointer farm a shared out-of-tree farm (Where the farm lives), but hard rule 3 and yaks-working's Finish treat "pointer" as private (ids out of commits).
- yaks-working says to keep the word "yaks" out of private-farm commits; the yaks skill says only ids. The rule appears four times with different wording.
- The yaks skill says `--as <actor>` is a global flag usable on any write, but `yaks --as x create ...` and `yaks create --as x ...` both fail ("unexpected argument '--as'"); `update`/`shave` accept it. Fix the doc or the CLI.

Pick one rule and state it once.

---
▸ 2026-10-27T22:29:32Z [delta:Check for `.yaks` file repo redirect sup]

Folded into yaks-a99f (2026-10-27): pointer files and in-tree private farms are being removed, so the pointer-vs-private wording conflict goes away with them. The commit-id rule and the `--as` finding remain; handle them in that docs sweep.
