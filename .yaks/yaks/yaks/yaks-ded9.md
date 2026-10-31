---
id: yaks-ded9
title: Human-created Delta threads get the creator as their name
status: hairy
type: bug
priority: 3
created: '2026-10-17T14:36:03Z'
updated: '2026-10-31T16:52:21Z'
parent: yaks-0071
labels:
- sheds
---

Eg, I created a manual sub-thread in Delta called "Implement yaks-31e2 cross-...", and it got labeled "Joel Webber" in the sheds UI, which is a bit confusing.

---
▸ 2026-10-22T03:26:28Z [delta:sheds-review]

yaks-9eaa: the 'creator's name' fallback is gone (Shed::name no longer uses note actors). An untitled clone (one where no yaks command ran with DELTA_THREAD_TITLE) now shows its Delta dir id, e.g. 9kf39g7cyp2a. The remaining fix is getting the title recorded for human-created threads; still open.

---
▸ 2026-10-31T15:56:25Z [delta:Cloud Runners]

Adopted into `yaks-039b` under `yaks-0071` (2026-10-31): with sheds built from note actors, this becomes the actor-name problem. A human-created thread's agent should set `YAKS_ACTOR`; see also `yaks-8003`.

---
▸ 2026-10-31T16:52:21Z [yakherd]

With sheds now one row per actor (`7393e45`), a human-created Delta thread shows under the name its agent writes as: `delta:<thread title>`, or `YAKS_ACTOR` if set. The 'creator's name' row only appears for notes the human wrote. Leaving this open until the skills rule (`YAKS_ACTOR` at session start) has been seen working; then shear.
