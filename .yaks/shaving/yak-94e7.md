---
id: yak-94e7
title: 'Phase 0 spike: bootstrap yaks-rs, port read-only commands over existing .yaks/'
type: task
priority: 2
created: '2026-08-20T02:18:28Z'
updated: '2026-08-20T03:01:59Z'
labels:
- rust
---

GATED ON GO-AHEAD (outcome of research arc yak-2219).

Start a new repo (yaks-rs) and port, rather than convert in-place (keeps the Python tree stable; both impls interoperate via the same .yaks/ files + derived index).

Deliverables:
- yak show / list / next in Rust, reading the SAME .yaks/ files + existing on-disk index (no format change => interop with the Python tool).
- clap CLI skeleton; frontmatter parse (hand-rolled + serde); model/deps/filter minimal port.
- Wire cargo-dist + an npm installer skeleton (Biome-style optionalDependencies platform packages) EARLY, so distribution is proven from day one.
- MEASURED cold-start vs the ~42-48ms Python baseline (target single-digit ms).

Exit: go/no-go for the full port (CLI parity -> ratatui + edtui TUI -> demo via ratatui Buffer + avt -> distribution cutover). Non-blocking to the yak-4473 UI arc. Slaughter if we decide against the conversion.

---
▸ 2026-08-20T02:54:24Z
Starting Phase 0. Bootstrapping a sibling repo ../yaks-rs (separate repo per yak-0446 decision) with a Cargo workspace + placeholder CLI (clap skeleton, show/list/next stubs) that will read the same .yaks/ files. Distribution scaffolding (cargo-dist + npm installer) and the startup benchmark come next.

---
▸ 2026-08-20T03:01:59Z
Placeholder project bootstrapped in ../yaks-rs (git init + initial commit b39f2ed). Landed:
- Cargo project (edition 2024): clap + anyhow; release profile (thin LTO, strip).
- src/model.rs (Status, Task); src/store.rs (.yaks discovery + hand-rolled frontmatter fast-path parser); src/main.rs (clap CLI: list/show/next).
- Reads the SAME .yaks/ files as the Python tool. list default matches Python (non-dead); id-set parity confirmed (213 == 213, empty symmetric diff); show/next verified against the live herd.
- MEASURED cold start on this herd: rust ~4.7ms median vs python ~48.6ms median (~10x). Confirms the core hypothesis (yak-0446 predicted ~10-20x; ~10x realized here, and Rust build is unoptimized for startup so far).
Remaining for Phase 0 exit: wire cargo-dist + npm installer skeleton; add assert_cmd smoke tests + an in-repo hyperfine bench; then go/no-go for Phase 1 (full CLI parity with --json byte-parity).
