---
id: yaks-48f4
title: Diagnose/fix local build failure after toque hoist (let-chains need rustc >= 1.88)
type: chore
priority: 2
created: '2026-10-01T13:51:14Z'
updated: '2026-10-01T13:52:20Z'
labels:
- build
- toque
---

---
▸ 2026-10-01T13:52:20Z [Joel Webber]
Root cause: not a repo bug. toque's hoisted-out code (edition 2024) uses let-chains (e.g. 'if let (Some(w), Some(h)) = ... && let (Ok(w), Ok(h)) = ...' in lib.rs:258-259), stabilized in rustc 1.88. This repo's rust-toolchain.toml pins channel="stable" (unchanged since the very first commit, yaks-94e7) and always has - that's correct and reproducible. The failure was that the local rustup 'stable' alias was stale at 1.87.0 (installed a while back; rustup's 'stable' only moves forward when you run 'rustup update', it isn't live-tracking). CI is unaffected: .github/workflows/*.yml use dtolnay/rust-toolchain@stable, which always fetches the current latest stable at run time. Fix: ran 'rustup update stable' (1.87.0 -> 1.99.0). Verified with the repo's own pinned toolchain (no +version override): cargo build --release --locked succeeds, cargo test --locked -> 319 + 28 pass. No code or config changes needed.
