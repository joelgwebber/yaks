---
id: yaks-0ef9
title: Client-side home for farm worktrees and per-user yaks metadata (XDG data dir)
status: hairy
type: task
priority: 3
created: '2026-10-18T15:54:02Z'
updated: '2026-10-18T15:54:02Z'
parent: yaks-6a34
labels:
- cli
- design
---

Joel (2026-10-18): decide where live-farm worktrees live long-term. Candidate: always under the platform data dir (`$XDG_DATA_HOME/yaks/` on Linux, `~/Library/Application Support/yaks/` on macOS), e.g. `<data>/farms/<repo-key>/` for the sparse worktree, keyed by the repo (origin URL or host git dir), so `yaks init --mode live` (yaks-b1e9) never asks for a path and every machine has the same layout. The same home could centralize the other client-side state now scattered: the TUI's views/starred working set (today `$XDG_CONFIG_HOME/yaks/<hash>/`), the stale-skills marker, and a registry mapping repos to their farm (the fallback discovery needs on a machine whose host repo is Delta's bare managed one, and what quiet mode needs). Rules to keep: rebuildable, never a second source of truth for anything in `.yaks/` (AGENTS.md invariant). Decide config vs data vs state dir per item.
