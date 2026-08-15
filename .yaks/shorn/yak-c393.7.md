---
id: yak-c393.7
title: 'Player behavior: no autoplay/loop, slower, pause at markers'
type: task
priority: 2
created: '2026-08-13T01:24:52Z'
updated: '2026-08-15T03:27:27Z'
---

docs/demo.html: disable autoPlay + loop (jarring); slow playback; ideally auto-pause at chapter/annotation boundaries so the viewer absorbs each beat and advances when ready. Investigate asciinema-player pause-at-marker options; fall back to a custom control if needed.

---
▸ 2026-08-13T01:56:51Z
demo.html player: autoPlay=false, loop=false, pauseOnMarkers=true (auto-pause at each chapter), speed=0.85, poster npt:0:01. Caption updated to explain the pause-to-advance UX.

---
▸ 2026-08-13T04:02:56Z
Adding click-to-advance: an overlay that appears when the player auto-pauses at a marker; clicking anywhere resumes to the next chapter.
