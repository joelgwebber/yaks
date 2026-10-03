---
id: yaks-814a
title: 'yaks init --mode: explicit farm mode and environment setup'
type: feature
priority: 3
created: '2026-10-03T20:14:31Z'
updated: '2026-10-03T20:14:31Z'
parent: yaks-b5a0
depends_on:
- yaks-3859
labels:
- cli
---

Require choosing the farm mode at init (team/public, private/local, out-of-tree pointer; possibly the environment, e.g. Delta) and perform the setup it implies: .gitignore or .git/info/exclude, pointer file, matching skills installed. Removes the 'figure out which mode you are in' step from the skill. Docs and --help in the same change.
