---
id: yaks-9a06
title: 'Shed detail: yak changes seem to be arbitrarily ordered'
status: shorn
type: bug
priority: 2
created: '2026-10-16T20:44:03Z'
updated: '2026-10-22T03:49:43Z'
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

---
▸ 2026-10-22T03:26:27Z [delta:sheds-review]

moved: hairy -> shaving

---
▸ 2026-10-22T03:26:28Z [delta:sheds-review]

Delivered by yaks-9eaa: the Sheds list sorts attention first (main, open ask, active, stale, idle), then the most recent writer; a shed's detail lists the yaks it wrote newest first. Evidence on yaks-9eaa.

---
▸ 2026-10-22T03:26:28Z [delta:sheds-review]

moved: shaving -> shorn

---
▸ 2026-10-22T03:49:43Z [delta:sheds-review]

verify: `cargo test --workspace` -> PASS (exit 0)
