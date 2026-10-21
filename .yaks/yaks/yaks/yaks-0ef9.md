---
id: yaks-0ef9
title: 'Per-user yaks state: one home for TUI views, the stale-skills marker and a repo-to-farm registry (config vs data vs state dirs)'
status: hairy
type: task
priority: 3
created: '2026-10-18T15:54:02Z'
updated: '2026-10-21T22:54:32Z'
labels:
- cli
- design
---

Joel (2026-10-18): decide where live-farm worktrees live long-term. Candidate: always under the platform data dir (`$XDG_DATA_HOME/yaks/` on Linux, `~/Library/Application Support/yaks/` on macOS), e.g. `<data>/farms/<repo-key>/` for the sparse worktree, keyed by the repo (origin URL or host git dir), so `yaks init --mode live` (yaks-b1e9) never asks for a path and every machine has the same layout. The same home could centralize the other client-side state now scattered: the TUI's views/starred working set (today `$XDG_CONFIG_HOME/yaks/<hash>/`), the stale-skills marker, and a registry mapping repos to their farm (the fallback discovery needs on a machine whose host repo is Delta's bare managed one, and what quiet mode needs). Rules to keep: rebuildable, never a second source of truth for anything in `.yaks/` (AGENTS.md invariant). Decide config vs data vs state dir per item.

---
▸ 2026-10-18T22:32:58Z [delta-farm]

Scope note (delta-farm): the farm-worktree location (`store::farm_home`, `$XDG_DATA_HOME/yaks/farms/<repo>-<hash>`) is built under yaks-bd7e, since the migrate command needs its default now. What remains here: views/starred working set, the stale-skills marker, a repo->farm registry (for bare-host machines and quiet farms), config vs data vs state dirs.

---
▸ 2026-10-21T22:54:32Z [delta:herd-dirs]

Joel (2026-10-21): clarified the title and hoisted out of yaks-6a34; hygiene work for later. The farm-worktree location part is done (store::farm_home, yaks-bd7e).
