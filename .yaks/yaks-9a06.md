---
id: yaks-9a06
title: 'Shed detail: yak changes seem to be arbitrarily ordered'
status: hairy
type: bug
priority: 2
created: '2026-10-16T20:44:03Z'
updated: '2026-10-19T04:28:52Z'
parent: yaks-9eaa
labels:
- sheds
---

I think it would be most useful if they could be reverse-update-time ordered, so that it's easy to drop in and see what was changed most recently.

---
▸ 2026-10-17T14:44:16Z [Joel Webber]

It just struck me that we could also do with some grouping on these. A few observations:
- When I'm looking at a shed, I very often want to know first what yaks are actually shorn in this shed.
- Asks (`needs: human`) seem like they should be higher-priority.
- Asks (`needs:` state) are highlighted, but not answers. It would be useful to be able to see the ask/answer threads within each yak.

---
▸ 2026-10-19T04:28:52Z [delta:herd-pass]

Herd pass 2026-10-19: moved under yaks-9eaa; with the sheds view built from per-write shed stamps, this ordering is a plain sort there.
