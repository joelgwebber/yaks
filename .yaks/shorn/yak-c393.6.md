---
id: yak-c393.6
title: Overlay/annotation affordance (bottom band)
type: feature
priority: 2
created: '2026-08-13T01:24:51Z'
updated: '2026-08-15T03:27:27Z'
parent: yak-c393
---

An overlay primitive to narrate what's happening — use the empty space at the bottom (above the help bar) for a caption/annotation band tied to beats/markers.

---
▸ 2026-08-13T01:56:51Z
render_annotation: caption band above the help bar (top rule + '> ' caption), clears its rows as a clean overlay. Director.annotate()/mark(note=...) drive it per beat.

---
▸ 2026-08-13T03:46:03Z
Redesigned the annotation from a full-width bottom band to a FLOATING CARD (render_overlay): a tinted panel with a cyan left spine, narrower-but-taller (~36 cols), anchored near its subject (agent/board/center) so it sits physically close to what it describes. Annotations now PERSIST (each replaced by the next, no blank gaps) and the screenplay keeps one up throughout with anchors + slightly longer beats.
