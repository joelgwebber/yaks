---
id: yaks-be32
title: 'AGENTS.md/README/RELEASING: local builds via cargo install --path ., no personal env'
status: shorn
type: task
priority: 2
created: '2026-10-26T21:15:10Z'
updated: '2026-10-26T21:16:09Z'
labels:
- docs
---

---
▸ 2026-10-26T21:15:14Z [delta:Triage starred yaks and design yaks-33c8]

moved: hairy -> shaving

---
▸ 2026-10-26T21:16:09Z [delta-lead]

Done (delta-lead), at Joel's request in the coordinating thread.

- `AGENTS.md`: the "hand the human a build" step is now `cargo install --path .` into `~/.cargo/bin`, replacing the `~/go/bin` override. "Joel's call" on sync is now "the human's call".
- The repo-local `land` skill's step 5 says the same.
- `README.md`: the from-source install uses `cargo install --path .`, with a note on putting `~/.cargo/bin` ahead of npm's bin directory, and `cargo build`/`cargo test` as the inner loop.
- `RELEASING.md`: a new "Before tagging: try the build locally" step.

Evidence: running `cargo install --path .` installed `~/.cargo/bin/yaks`; a grep for `~/go/bin` and "Joel" across `AGENTS.md`, `README.md`, `RELEASING.md` and `.agents/` finds no hits left (the `joelgwebber` GitHub URLs are the real repo and stay).

---
▸ 2026-10-26T21:16:09Z [delta-lead]

moved: shaving -> shorn
