---
id: yaks-da5e
title: Allow markdown links to repo files / source
status: hairy
type: feature
priority: 2
created: '2026-10-17T14:13:34Z'
updated: '2026-10-17T14:13:34Z'
parent: yaks-faf5
labels:
- markdown
- tui
---

Existing markdown links can only reference files relative to the `.yaks` dir. This is fine for attachments, but it would be really convenient to be able to reference other files in the repo as well.

## Link format
Not entirely sure about link format -- propose something that fits reasonably cleanly into other markdown renderers, so it's likely to do something reasonable. Maybe as dumb as just `[code](../src/whatever.rs)`, which ought to "just work".

## Line ranges
In the case of source code, it would also be helpful to support link number/range references. Again, preferably in a way that won't break other markdown renderers. Eg, is it the norm to use `#fragments` for this, like `[code](../src/whatever.rs#L123:145)`?

## Commit-specific
Ooh, it would be _really_ cool if you could reference repo files by `git` hash as well, so a yak can maintain historical connection to the _actual_ code, before it's changed. Again, open to format ideas; this case matters less for other markdown renderers, though it would be nice if the path still hit the right spot in the filesystem.
