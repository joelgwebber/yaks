---
id: yaks-ded9
title: Human-created Delta threads get the creator as their name
status: hairy
type: bug
priority: 3
created: '2026-10-17T14:36:03Z'
updated: '2026-10-22T03:26:28Z'
labels:
- sheds
---

Eg, I created a manual sub-thread in Delta called "Implement yaks-31e2 cross-...", and it got labeled "Joel Webber" in the sheds UI, which is a bit confusing.

---
▸ 2026-10-22T03:26:28Z [delta:sheds-review]

yaks-9eaa: the 'creator's name' fallback is gone (Shed::name no longer uses note actors). An untitled clone (one where no yaks command ran with DELTA_THREAD_TITLE) now shows its Delta dir id, e.g. 9kf39g7cyp2a. The remaining fix is getting the title recorded for human-created threads; still open.
