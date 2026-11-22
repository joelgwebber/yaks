---
id: yaks-d3df
title: Cut v0.0.41
status: shorn
type: task
priority: 1
created: '2026-11-22T16:42:25Z'
updated: '2026-11-22T18:56:20Z'
tags:
- release
---

After the pre-release batch (yaks-cae0, yaks-daa4, yaks-6554, yaks-879e): `yaks release changes --check`, rerun `yaks release dok v0.0.41` and refresh its headlines (pin a docshot), bump Cargo.toml/Cargo.lock/npm package.json to 0.0.41, `yaks release changelog -o CHANGELOG.md` committed with the bump, then (Joel's go-ahead) push main + tag v0.0.41 to origin; GitHub Release body from `yaks release changelog --release v0.0.41`.

#### 2026-11-22T16:42:27Z

*herd-files*

Batch (herd-files, base 6ae40a60, all claude-sonnet-5-5): init-1 yaks-cae0 (thread ksQQ7I2z8nJeT6a6hDJ07WHX45LOAHgO5JIBxBCA0aLJyrhJPreUvVwLftHG); shots-1 yaks-daa4 (…HgPcJ…); html-1 yaks-6554 (…HgP4Z…); commit-1 yaks-879e (…HgQcZ…). Removed Joel's stale 0.0.22 global npm yaks from Homebrew's node (Joel: uninstall it).

#### 2026-11-22T17:34:00Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-22T17:43:53Z

*herd-files*

Prepared (herd-files): batch landed (eed4dd5e, c763a393, 876170b5, fe2b473d) plus 668232a1 (the release being cut leads CHANGELOG.md with its dok's date); `release changes --check` ok; v0.0.41 dok has headlines and a pinned archive-browser image (farm f4211aaa, on origin; raw URL serves image/png); bump + CHANGELOG.md committed as 7986845c, pushed to local main, installed (yaks 0.0.41). Gate: fmt clean, 1587 passed, 0 failed. Waiting on Joel's go-ahead to push main and tag v0.0.41 to origin (publishes to npm).

#### 2026-11-22T17:57:19Z

*herd-files*

Ready to publish (herd-files), main f47196f6 on local main, installed:

- `origin/main` had five commits of Joel's not in our main (the toque extraction: yaks-67ef, yaks-2e7c, yaks-20a5, yaks-2141). Merged in c0e1ba76; conflicts in imageview.rs (toque's ImageView kept, plus this branch's `rev` and `from_bytes`), keymap.rs (toque's rule with RowKind::Place), the yaks-working skill (both changes) and Cargo.lock (origin's, rebuilt at 0.0.41). After the merge: docshots regenerate byte-identical, the pinned LFS image still opens in the image viewer, `cargo test --workspace --locked` with an empty HOME passes (1580, 0 failed), fmt clean.
- Change lines: three toque yaks `--no-change`, yaks-2141 a line (headless `svg`/`png` frames). `release changes --check` ok; release dok up to date; CHANGELOG.md regenerated (f47196f6).
- main fast-forwards origin/main.

#### 2026-11-22T18:56:19Z

*herd-files*

Released v0.0.41 (herd-files, Joel 2026-11-22: "let's do it").

- The first dry run (run 45054387367 on f47196f6) failed in `test`: `tests/land_change_warning.rs` (yaks-045c) ran `yaks land` with no git identity, which CI lacks; an empty HOME on macOS had hidden it. Fixed in 64d2ebf9 (the test sets GIT_AUTHOR/COMMITTER like tests/land_continue.rs) and the whole suite was re-run under `user.useConfigOnly` with an empty HOME: 1580 passed, 0 failed.
- Dry run 45055040927 on 64d2ebf9: test, five builds and publish dry-run all green.
- Tag `v0.0.41` on 64d2ebf9, pushed to origin and local; release run https://github.com/joelgwebber/yaks/actions/runs/45055792612 succeeded (test, 5 builds, publish). `npm view @j15r/yaks` gives version 0.0.41, latest 0.0.41.
- GitHub Release https://github.com/joelgwebber/yaks/releases/tag/v0.0.41 with the body from `yaks release changelog --release v0.0.41`.
- Release dok rerun after tagging: range v0.0.40..v0.0.41, tag recorded.

#### 2026-11-22T18:56:20Z

*herd-files · kind: move · from: shaving · to: shorn · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: shaving -> shorn

#### 2026-11-22T18:56:20Z

*herd-files · kind: change · section: none*

No user-facing change.
