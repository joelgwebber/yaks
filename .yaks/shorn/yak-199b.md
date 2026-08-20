---
id: yak-199b
title: Virtual terminal lib for TUI abstraction + cast recording
type: task
priority: 3
created: '2026-08-16T23:19:19Z'
updated: '2026-08-20T02:17:59Z'
parent: yak-2219
labels:
- rust
---

Assess a Rust virtual-terminal (avt/vt100/termwiz) to abstract TUI rendering and deterministically compose demo casts, replacing demo/castkit.py's hand-rolled Screen + asciicast emitter (cf yak-3291, yak-fa3b).

---
▸ 2026-08-16T23:24:50Z
FINDINGS (versions verified 2026-08).

TWO needs, both served by a virtual terminal model: (a) abstracting TUI rendering, and (b) deterministic demo-cast generation + TUI snapshot testing. Today demo/castkit.py hand-rolls BOTH: a display-column grid Screen with double-width/emoji handling, and an asciicast v2 emitter — and it EXPLICITLY mirrors asciinema avt unicode-width rules to stay column-aligned with the player.

avt 0.18.0 (May 2026) is asciinema OWN virtual terminal, in Rust: a Williams-state-machine ANSI parser + primary/alt screen buffers as a char grid with color/style attrs + an API to feed bytes and query the grid + cursor. Its scope is parse+buffer only (no input, no rendering) — precisely castkit Screen role, maintained by the very project whose width rules we currently copy by hand. Adopting avt makes our width handling correct-by-construction against the player instead of a hand-maintained mirror. (Alternatives: vt100 crate — older Rust parser; termwiz has a full surface/terminal model. avt is the closest fit and lowest drift.)

KEY INSIGHT for rendering abstraction: ratatui Buffer + TestBackend already IS a virtual screen (cells + style). So yak-3291 (shared Surface/Style painter for TUI + demo) collapses to: render the REAL ratatui widgets to a TestBackend Buffer, then serialize Buffer -> ANSI frames into an asciicast. ONE painter drives both the live TUI and the demo frames, deleting castkit parallel reimplementation and the alignment drift it fights. avt then serves as the verification oracle (feed our emitted frames back through avt and assert the resulting grid) and/or the basis for a renderer.

CAST FORMAT: asciicast v2 is trivial line-delimited JSON; keep our own ~40-line writer (castkit.Cast is already that) rather than take a dep. This also feeds yak-fa3b (replace asciinema-player with a lightweight renderer): because frames would come from ratatui Buffers we control the exact cell grid and can render to SVG/GIF (avt has a companion gif path) without shipping the JS player.

RECOMMENDATION: (1) live TUI and demo share one ratatui painter -> TestBackend Buffer; (2) a tiny asciicast v2 writer we own; (3) avt as the parse/verify oracle and future GIF/SVG rendering base. Net: castkit hand-rolled Screen goes away; width correctness comes from avt/unicode-width; yak-3291 and yak-fa3b both get materially easier.

---
▸ 2026-08-20T02:17:59Z
REFINEMENT (per owner): the virtual terminal is not just for demos — it is a first-class TESTING + agent-introspection surface. Because the live TUI renders to a ratatui Buffer (and TestBackend yields that Buffer with no real terminal), and avt can parse any emitted ANSI back into a grid, we get:
- deterministic golden/snapshot tests of full rendered frames (insta over the Buffer), catching layout/wrap/width regressions that unit tests miss today;
- headless agent/CI introspection: feed the app scripted key events, dump the Buffer grid as text, and let a coding assistant READ BACK the exact on-screen state to verify its own output.
Several birds, one rock: a single painter -> Buffer serves live render, demo casts (yak-3291/yak-fa3b), regression snapshots, AND machine-readable introspection for agents. This raises TUI testability dramatically vs curses, and makes the TUI safe for an AI to iterate on.
