---
id: yak-90b8
title: Supporting ecosystem (CLI, YAML, fs-events, clipboard, locking, testing, packaging)
type: task
priority: 3
created: '2026-08-16T23:19:19Z'
updated: '2026-08-20T02:17:45Z'
parent: yak-2219
labels:
- rust
---

Round out the port: clap (CLI), frontmatter/YAML parse, notify (fs-events), arboard (clipboard), fs4/fd-lock (cross-process locking), insta/assert_cmd (testing), cargo-dist/binstall (distribution).

---
▸ 2026-08-16T23:25:12Z
FINDINGS — the rest of the port, by subsystem (all mainstream, low-risk crates).

CLI (replaces yaklib/parser.py argparse): clap 4.x derive — subcommands, repeatable flags, --json, generated help. Near-zero startup cost vs argparse import.

YAML / frontmatter (replaces model.py YAML I/O + the yak-3fd4.3 fast-path parser): frontmatter is simple scalars/lists, so hand-roll the --- splitter and parse the block with serde. General parser options: serde_yaml_ng 0.10 (fork of dtolnay archived serde-yaml, YAML 1.1, 2M/mo) OR saphyr (pure-Rust YAML 1.2, active successor to yaml-rust). gray_matter crate exists specifically for front-matter. Body after the closing --- stays raw markdown, as today.

FS-EVENTS (new; replaces TUI stat-poll, also a yak-3fd4 follow-up): notify 6/7 — FSEvents/inotify/kqueue/ReadDirectoryChangesW behind one API. Ties to yak-1622 index invalidation.

CLIPBOARD (replaces yaklib/clipboard.py pbcopy/xclip shellouts + read_png): arboard — cross-platform text AND image clipboard (macOS/Windows/X11/Wayland). Deletes the OS-specific subprocess code.

CROSS-PROCESS SAFETY (matches yak-3fd4 atomic-write decision): tempfile persist() for atomic temp+rename of the index; fs4 (successor to fs2) or fd-lock for advisory locks if a write barrier is ever needed. Rename is the primary primitive.

TERMINAL HYGIENE (replaces curses wrapper + the def_prog_mode/endwin/IXON dance in editor.py): crossterm raw-mode guard + a panic hook that restores the terminal on crash. RAII guard means no leaked raw mode.

TESTING (ports tests/ ~1:1): assert_cmd + predicates for subprocess CLI tests (mirrors current suite), trycmd/snapbox for golden CLI output, insta for snapshots of ratatui TestBackend buffers AND of list/rollup/--json output. Existing pytest subprocess tests translate directly.

DATES (model.py ISO8601 + format.humanize_date): chrono or time + a small humanizer.

PATHS/CONFIG (platformdirs equivalent — already transitive today): directories crate for ~/.cache/yaks and ~/.config/yaks resolution.

CLI COLOR: owo-colors / anstream with NO_COLOR support. unicode-width for display width (same rules as avt/castkit).

DISTRIBUTION — the biggest downstream change. Single static binary: musl target for fully-static Linux; macOS universal via lipo; cargo-dist ("dist") to produce per-platform artifacts, installers, and GitHub Releases; plus cargo-binstall + a Homebrew tap + cargo install. This REPLACES the PyPI/yakherder + uvx path entirely. IMPORTANT: the yak skill "Running yaks" stanza and the Claude/Codex plugin + marketplace manifests must switch their install instructions from uvx/PyPI to a binary installer (curl script / brew / binstall). The skill CONTENT (markdown workflow) is unaffected; only the install lines and the version-bump mechanics change. Keep the command name yaks.

RECOMMENDATION: none of these are blockers; all are standard. The only real migration surface beyond code is distribution + the skill/plugin install stanzas (tracked here so the synthesis in yak-0446 accounts for it).

---
▸ 2026-08-20T02:17:45Z
REFINEMENT (per owner): replicate the Go + goreleaser + npm distribution that has worked well. This pattern IS common and available for Rust CLIs.

- The pattern: publish N platform-specific npm packages (e.g. yaks-darwin-arm64, yaks-linux-x64-gnu, yaks-win32-x64), each containing ONE prebuilt binary, declared as optionalDependencies of a thin main package yaks whose bin shim execs whichever optional dep npm installed (npm skips optionalDependencies whose os/cpu do not match the host). esbuild (Go) pioneered it; the canonical RUST reference implementations are Biome, oxc/oxlint, swc, Rolldown, and tailwindcss-oxide — all ship exactly this way. Users run npm i -g yaks (or npx) and never see os/arch.

- Tooling: cargo-dist / dist (axodotdev, alive, 2.1k stars, self-hosting) generates the whole release pipeline from a git tag — multi-platform builds, tarballs, installers, GitHub Release, publish to package managers — INCLUDING an npm installer target. So one dist config yields the goreleaser-equivalent npm packages PLUS shell/powershell/brew installers and cargo-binstall metadata. napi-rs is the other npm-adjacent option but it targets node-addon (.node) builds, not standalone CLIs — not what we want.

- Fit with our surfaces: ideal, because the skill and Claude/Codex plugins already assume a zero-install story (uvx today). Swapping the Running-yaks stanza to npm i -g yaks / npx yaks is a small edit, and dist ALSO gives brew + curl installer + cargo install + binstall for non-npm users. Command name stays yaks.

RECOMMENDATION: use cargo-dist to drive releases; enable the npm installer (Biome-style optionalDependencies platform packages) as the primary agent/user path, with brew + shell installer as alternates. Mirrors the proven Go + goreleaser + npm workflow.
