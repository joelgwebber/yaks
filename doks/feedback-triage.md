---
type: topic
title: Feedback triage ledger
created: '2026-11-25T04:21:11Z'
description: 'Which feedback-to-yaks entries from other farms have been triaged, and to what: read before collecting feedback, so no entry is triaged twice'
---

Every entry the collector (yaks-feaf) has triaged from another farm's `/doks/feedback-to-yaks.md`, one row each, whatever came of it. Design: yaks-24be.

- **entry** (the row key): `<owner/repo>#<heading>`, the farm's `remote.origin.url` as `owner/repo` and the entry's `##` heading exactly. Not the farm directory name: that changes with each clone.
- **outcome**: `new` (a yak was filed; it also carries the key as `--key feedback-to-yaks:<entry>`), `linked` (noted on an existing yak), `not-a-bug` (say why), `duplicate` (of another entry: name it), `done` (already fixed), `wontfix`.
- **yak**: the yak it went to, when there is one.
- **triaged**: the day it was triaged. An entry whose newest `**Update YYYY-MM-DD:**` bullet is later than this is triaged again, and its row updated.

Write a row with `yaks dok row set 'doks/feedback-triage.md#<entry>' --set outcome=... --set yak=... --set triaged=YYYY-MM-DD --set 'note=...'`; list them with `yaks dok query --rows --where 'entry=~<owner/repo>#'`.

| entry | outcome | yak | triaged | note |
| --- | --- | --- | --- | --- |
| cowpaths/joel#2026-11-10: `dok new` and `source add` fail when run from outside a farm checkout | linked | yaks-76a3 | 2026-11-18 | second farm to hit it; raised to p3 |
| cowpaths/joel#2026-11-10: `session touched` missed sources committed under the harness actor | linked | yaks-d11e | 2026-11-18 | thread selection would catch them |
| cowpaths/joel#2026-11-10: an unset actor stamps commits with the harness name, even after `whoami --set` | new | yaks-5250 | 2026-11-18 | whoami --set writes only the git config of the checkout it ran in |
| cowpaths/joel#2026-11-12: `yaks check` flags an email autolink as a broken link | new | yaks-9dc2 | 2026-11-18 |  |
| cowpaths/joel#2026-11-13: `dok row set` stamps `as_of` on every row it touches, even when only one field was checked | not-a-bug |  | 2026-11-18 | row set leaves other cells alone; the agent's helper passed --set as_of on every call |
