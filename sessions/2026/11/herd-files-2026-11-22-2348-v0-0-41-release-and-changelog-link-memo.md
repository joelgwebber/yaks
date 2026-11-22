---
type: session-review
title: v0.0.41 release and changelog link memo (yaks-bfb2)
source: session-note
actor: herd-files
person: Joel Webber
host: joel
harness: delta
thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG
day: 2026-11-22
written: '2026-11-22T23:48:08Z'
good: 3
---

## Summary

Released v0.0.41 (tagged at 64d2ebf9, npm `latest`, GitHub Release from `yaks release changelog --release v0.0.41`), then fixed yaks-bfb2 and landed it on `origin/main` at 7c1452bd. It is not in a release yet; the user will cut one later.

## What changed

- yaks-bfb2: `locate::Space` memoizes the git facts `web_url` needs for each repository (branch, remote/host, private, commit_of, remote_has, remote_branch_exists). `file_web` is now a `Space` method. A full CHANGELOG.md takes 7.7 s instead of 64 s, with identical output. Gate: `many_links_cost_about_one_git_call_each` (12 links: 158 git calls before, 48 after, limit 60).

## Saved

- yaks-0cfb: Docshots: regenerate every doc SVG once to absorb toque renderer drift; refresh the Sheds tab shots (hairy -> dead)
  - [Duplicate of yaks-daa4 (same docshot refresh, filed twice); worked there in t...](/yaks/yaks/yaks-0cfb.md#2026-11-22t164059z)
- yaks-20a5: Take the image pane from toque
  - [change: No user-facing change.](/yaks/yaks/yaks-20a5.md#2026-11-22t175051z)
- yaks-2141: Ask for one rendered frame on every TUI yak
  - [change: `yaks tui --headless` scripts can end with `svg <path>` or `png <path>` to wr...](/yaks/yaks/yaks-2141.md#2026-11-22t175052z)
- yaks-2e7c: Take the command table mechanism from toque
  - [change: No user-facing change.](/yaks/yaks/yaks-2e7c.md#2026-11-22t175051z)
- yaks-4ed6: update --as stamps the note's actor but the farm commit's Yaks-Actor trailer comes from the environment (created)
- yaks-6554: TUI markdown: hide HTML comments (release dok block markers show as literal text) (hairy -> shaving)
  - [Claim (herd-files): worker html-1 (Sonnet), base 6ae40a60, batch before v0.0....](/yaks/yaks/yaks-6554.md#2026-11-22t164102z)
  - [landed: eed4dd5e yaks-6554: TUI markdown hides HTML comments (block, multi-li...](/yaks/yaks/yaks-6554.md#2026-11-22t170544z)
  - [Landed eed4dd5e (herd-files). Reproduced: the real v0.0.41 release dok in the...](/yaks/yaks/yaks-6554.md#2026-11-22t171450z)
  - [Gate at eed4dd5e: fmt clean, 1585 passed, 0 failed.](/yaks/yaks/yaks-6554.md#2026-11-22t171502z)
  - [change: The TUI hides HTML comments in markdown, as GitHub does (a release dok's bloc...](/yaks/yaks/yaks-6554.md#2026-11-22t173408z)
- yaks-67ef: Take theme surfaces from toque
  - [change: No user-facing change.](/yaks/yaks/yaks-67ef.md#2026-11-22t175050z)
- yaks-7ed9: Archive member and directory headers show the farm path (yak/yaks/yak-1a2b/code.zip/...) where an attachment shows yak-1a2b/file (created)
- yaks-879e: yaks commit errors 'git commit failed ... nothing to commit' when an autocommit already took the change (hairy -> shaving)
  - [Claim (herd-files): worker commit-1 (Sonnet), base 6ae40a60, batch before v0....](/yaks/yaks/yaks-879e.md#2026-11-22t164102z)
  - [Coordinator datapoint, 2026-11-22 ~17:10Z, real farm: `yaks update yaks-6554...](/yaks/yaks/yaks-879e.md#2026-11-22t171510z)
  - [landed: 876170b5 yaks-879e: yaks commit takes the farm lock and treats an emp...](/yaks/yaks/yaks-879e.md#2026-11-22t172446z)
  - [Landed 876170b5 (herd-files). Reproduced on the real farm with my build: 15 r...](/yaks/yaks/yaks-879e.md#2026-11-22t172633z)
  - [change: `yaks commit` no longer fails with "nothing to commit" when another write's c...](/yaks/yaks/yaks-879e.md#2026-11-22t173408z)
- yaks-bfb2: release changelog: one web_url costs ~12 git calls; cache per-run remote facts so a 41-release CHANGELOG.md isn't 64 s (hairy -> shorn)
  - [change: `yaks release changelog` asks git for each repository's remote, host and HEAD...](/yaks/yaks/yaks-bfb2.md#2026-11-22t232418z-1)
- yaks-cae0: yaks init still writes a fresh farm under .yaks/ (schema 10 in the pre-9 layout) (hairy -> shaving)
  - [Claim (herd-files): worker init-1 (Sonnet), base 6ae40a60, batch before v0.0....](/yaks/yaks/yaks-cae0.md#2026-11-22t164100z)
  - [landed: fe2b473d yaks-cae0: yaks init writes a fresh farm at the branch root,...](/yaks/yaks/yaks-cae0.md#2026-11-22t173006z)
  - [Landed fe2b473d (herd-files). Reproduced: fresh repo, `yaks init --no-skills`...](/yaks/yaks/yaks-cae0.md#2026-11-22t173359z)
  - [change: `yaks init` writes a new farm at the root of its branch (`config.yaml`, `sche...](/yaks/yaks/yaks-cae0.md#2026-11-22t173407z)
- yaks-d3df: Cut v0.0.41 (created, hairy -> shorn)
  - [Batch (herd-files, base 6ae40a60, all claude-sonnet-5-5): init-1 yaks-cae0 (t...](/yaks/yaks/yaks-d3df.md#2026-11-22t164227z)
  - [Prepared (herd-files): batch landed (eed4dd5e, c763a393, 876170b5, fe2b473d)...](/yaks/yaks/yaks-d3df.md#2026-11-22t174353z)
  - [Ready to publish (herd-files), main f47196f6 on local main, installed:](/yaks/yaks/yaks-d3df.md#2026-11-22t175719z)
  - [Released v0.0.41 (herd-files, Joel 2026-11-22: "let's do it").](/yaks/yaks/yaks-d3df.md#2026-11-22t185619z)
  - [change: No user-facing change.](/yaks/yaks/yaks-d3df.md#2026-11-22t185620z-1)
- yaks-daa4: Refresh every docshot: docs/assets SVGs drift from the current toque output (hairy -> shaving)
  - [Claim (herd-files): worker shots-1 (Sonnet), base 6ae40a60, batch before v0.0...](/yaks/yaks/yaks-daa4.md#2026-11-22t164101z)
  - [landed: c763a393 yaks-daa4: refresh every docshot; add archive-browsing and l...](/yaks/yaks/yaks-daa4.md#2026-11-22t171940z)
  - [Landed c763a393 (herd-files). Rasterized and read tui-archive-tree, tui-archi...](/yaks/yaks/yaks-daa4.md#2026-11-22t173400z)
  - [change: The documentation's TUI screenshots are refreshed, with new frames for browsi...](/yaks/yaks/yaks-daa4.md#2026-11-22t173409z)
- changed [/doks/releases/v0.0.41.md](/doks/releases/v0.0.41.md): yaks v0.0.41

## Lessons

- To count the git calls a command makes, run it under `GIT_TRACE=<file>` and grep for `built-in: git`. That needs no PATH shim.
- A `Space` is now the batch scope: make a fresh one to see a push or fetch. Every current caller builds one per command or per action.
- CI on macOS: an empty HOME is not enough. Also set `GIT_CONFIG_GLOBAL` to a file with `useConfigOnly`.

## Open

- yaks-e61e: quadratic TUI startup (p1).
- yaks-7ed9: archive headers show the farm path.
- yaks-4ed6: `--as` does not set the commit's `Yaks-Actor` trailer.
- pnpm's global yaks is still 0.0.39 (`minimumReleaseAge`): run `pnpm add -g @j15r/yaks@latest` after 24 h.

## Turns

- good: cut and published v0.0.41 (one CI dry-run failure: a test had no git identity; fixed and re-run).
- good: yaks-bfb2 fixed with byte-identical output and a spawn-count gate.
- good: landed 7c1452bd on origin/main; no tag, so nothing published.
