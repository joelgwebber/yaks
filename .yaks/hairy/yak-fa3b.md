---
id: yak-fa3b
title: Replace asciinema-player with a lightweight cast renderer
type: idea
priority: 4
created: '2026-08-14T03:13:51Z'
updated: '2026-08-14T03:13:51Z'
---

The vendored asciinema-player ships a Rust->WASM terminal emulator (avt, ~73KB wasm inlined as base64) plus a DOM renderer, just to replay a .cast of a few hundred styled cells. Overkill for our needs and the source of the emoji-width headaches (color-emoji glyphs don't advance exactly 2 monospace cells in its flowing-DOM renderer; avt's width table lives in an uneditable WASM blob). We control the cast format end to end, so a tiny custom player (parse asciicast v2, apply a minimal SGR/CSI subset to a fixed grid rendered on a <canvas> or a CSS grid of cells) would be smaller, hackable, and let us fix width handling ourselves (incl. putting wide glyphs in fixed cells so emoji never shift neighbors). Scope: play/pause/scrub, markers + pauseOnMarkers, click-to-advance, our theme. Not urgent - the ASCII workaround holds - but revisit before we lean harder on the demo.
