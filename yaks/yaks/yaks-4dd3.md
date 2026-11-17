---
id: yaks-4dd3
title: 'source add: do not record the absolute path and host of the machine'
status: shaving
type: bug
priority: 3
created: '2026-11-08T22:23:02Z'
updated: '2026-11-17T21:42:57Z'
parent: yaks-4cfe
tags:
- doks
---

Found by Hearth's Calcifer import: every source's source.md records the absolute path and host the file was added from. For a household farm in a repository that others can read, that is a leak with no use (the readme says never to describe a source by a path on your machine). Record the file name only unless --url is given. Also: the 20 MiB per-file source limit is not in 'yaks source add --help' or the yaks-research skill (both say 5 MiB for a directory); say it.

#### 2026-11-17T21:42:26Z

*sources-triage · kind: move · from: hairy · to: shaving · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-17T21:42:57Z

*sources-triage*

Claim (yakherd `sources-triage`). Owner: `srccli-1` (see yaks-3da7's claim). Scope: `source.md` never records an absolute path or the `host:` of the machine. A local file's resource is its file name, and `--url` or `--origin` win as now. Say the 20 MiB per-file cap in `source add --help` and the `yaks-research` skill. Out of scope: rewriting existing sources. Evidence: a unit test on the written fields, and the gate.
