---
id: yaks-10ce
title: 'toque README: real agent-driven yaks session (protocol transcript + SVG frames); fix canon link'
type: chore
priority: 3
created: '2026-10-01T21:45:38Z'
updated: '2026-10-01T22:19:49Z'
labels:
- docs
- toque
---

---
▸ 2026-10-01T22:19:49Z [Joel Webber]
toque README: real agent session against yaks (added docshots_session generator in src/tui/docshots.rs: Session::step over 'key Tab','key j','key l' on the docshots farm at 96x32 -> docs/assets/session.txt + session-N.svg). Frames 1-3 copied to toque/docs/assets/yaks-session-N.svg, embedded with trimmed protocol excerpts (README notes abridging). Joel committed/pushed the toque side (801a1cd); canon link fixed to joelgwebber/canon. yaks README: added tui-list.svg + tui-detail.svg to the Interactive TUI section plus a pointer to yaks tui --headless / toque. Looked at rasterized frames (headless Chrome) for session 2+3, tui-list: render correctly. Evidence: cargo test --locked 319+28 pass; docshots --ignored leaves existing SVGs byte-identical; docs/tui.md documents the new generator.
