---
id: yak-bbd7
title: 'README restructure: CLI-first install, UIs up front, screenshots'
type: task
priority: 3
created: '2026-07-25T23:57:30Z'
updated: '2026-07-25T23:59:50Z'
---

Restructure README: (1) generalize the Zed install section to 'Agent Skills (any agent)' and make CLI install (uv tool install yakherder) the primary step so users can 'yaks tui'; (2) clarify that plugin/skill install does NOT provide the 'yaks' command (needs the CLI step); (3) move TUI + Web board up under 'How it works' and merge Commands+Filtering into a single 'CLI' section after them; (4) add the tui.png/wui.png screenshots; (5) ensure Web board section current + add deep-link/self-host notes; (6) set repo homepageUrl to the board.

---
▸ 2026-07-25T23:59:50Z
Done. README restructured: Install is now CLI-first (step 1: uv tool install yakherder -> 'yaks'/'yaks tui'; uvx fallback) + 'Teach your agent' step 2 with an explicit callout that the plugin/skill does NOT install the yaks command. Generalized the Zed section to 'Zed and other Agent-Skills agents' (npx skills add / manual copy; nothing Zed-specific). Added 'Interfaces' section (TUI + Web board) right after 'How it works', each with a screenshot (assets/tui.png, assets/web.png via raw.githubusercontent so they render on GitHub + PyPI). Merged Commands+Filtering into one 'CLI' section after Interfaces. Web board section: added deep-link (#owner/repo) and self-host (copy docs/index.html -> Pages auto-detects your repo) notes. Broadened tagline to 'humans and AI coding agents'. Set repo homepageUrl to the board. Bumped 0.1.80->0.1.81 (README shipped). Tests pass.
