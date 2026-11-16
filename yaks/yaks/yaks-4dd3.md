---
id: yaks-4dd3
title: 'source add: do not record the absolute path and host of the machine'
status: hairy
type: bug
priority: 3
created: '2026-11-08T22:23:02Z'
updated: '2026-11-08T22:23:02Z'
---

Found by Hearth's Calcifer import: every source's source.md records the absolute path and host the file was added from. For a household farm in a repository that others can read, that is a leak with no use (the readme says never to describe a source by a path on your machine). Record the file name only unless --url is given. Also: the 20 MiB per-file source limit is not in 'yaks source add --help' or the yaks-research skill (both say 5 MiB for a directory); say it.
