---
id: yaks-aa49
title: 'id to path resolution: yaks path'
type: feature
priority: 3
created: '2026-08-23T03:43:37Z'
updated: '2026-10-03T23:27:39Z'
parent: yaks-8d53
labels:
- cli
- git
---

yaks path ID prints the current on-disk file path for a yak (and yaks path [filters] for a set), so git add is precise even though transitions move files between status dirs. Motivation: repeatedly hand-built .yaks/status/id.md paths and fought staging drift when committing yak moves alongside code.

---
▸ 2026-10-03T23:24:17Z [delta-lead]
TRIAL 4 (yaks-5c9f): worker 't4-path', SHORT brief, must load yaks-working itself. Claim committed. Scope: new 'yaks path' command (src/main.rs Command + dispatch, a lookup in src/farm.rs), docs/cli.md, --help; add a one-line pointer in the yaks-working pathspec bullet if natural. Evidence: gate cargo test -p yaks; tests incl. a yak in each status dir, several ids, a filter form, an unknown id (non-zero exit); transcript of the built binary. Judge: coordinator re-runs the gate and the scenario.

![t4-path-transcript](artifacts/yaks-aa49/t4-path-transcript.txt)

---
▸ 2026-10-03T23:27:39Z [t4-path]
Added 'yaks path [IDS | filters] [--all]': Farm::path_of/paths (src/farm.rs), run_path + Command::Path (src/main.rs), docs/cli.md, clap --help. Absolute paths; unknown id -> stderr + exit 1 (known ids still print); ids XOR filter else exit 1. Tests: 6 path_* in src/main.rs (each status dir, several ids, transition, unknown id exit 1, filter form, xor). Gate: cargo test -p yaks => 331 + 28 passed. Transcript of built binary attached. Skipped the optional yaks-working pointer (out of the given scope; its SKILL.md frontmatter also fails YAML parsing in Delta).
