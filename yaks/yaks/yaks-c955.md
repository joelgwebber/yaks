---
id: yaks-c955
title: '@rev links read LFS files: smudge the pointer so a pinned image shows'
status: shorn
type: bug
priority: 2
created: '2026-11-21T21:20:59Z'
updated: '2026-11-21T22:31:17Z'
parent: yaks-a4d8
tags:
- links
- images
- git
---

Decision 6 of [the changelog design](/doks/design/changelog.md#6-images-following-vs-pinned). `links::read_at` runs `git show <rev>:<path>`, which for an LFS-tracked file returns the 131-byte pointer. Detect a pointer and pass it through `git lfs smudge -- <path>` (probe: [lfs-probe.md](/yaks/yaks/yaks-a4d8/lfs-probe.md)). Through the one opener; check the TUI image viewer and `yaks peek` on a pinned image.

#### 2026-11-21T21:35:47Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-21T21:35:48Z

*herd-files*

Claim (herd-files): worker lfs-1 (Sonnet). Scope: src/links.rs `read_at` (and a binary-safe sibling if images are read as bytes elsewhere: find where the TUI image viewer and `yaks peek` read @rev files). Out of scope: anything else. Evidence: a test with a real LFS pointer in a temp repo (skip if git-lfs missing), and a TUI headless/peek run on this farm's pinned image. Judge: herd-files at landing.

#### 2026-11-21T21:41:36Z

*lfs-1*

Evidence link (pinned LFS image): ![manifesto page 2](/sources/2026-11-05-0272f64e-proton-engineering-manifesto-page-2-of-3/manifesto-2.png@33f5db7f)

#### 2026-11-21T21:45:42Z

*lfs-1 · kind: verify · result: pass*

verify: `cargo test --workspace` -> PASS (exit 0)

[peek-evidence](yaks-c955/peek-evidence.txt "peek of the pinned LFS image before (pointer) and after (170083-byte binary placeholder, matches git lfs smudge)")

[tui-pinned-image-frame](yaks-c955/tui-pinned-image-frame.txt "headless TUI: following the @33f5db7f link in this yak opens the real image in the image viewer (was: text pane of the pointer)")

#### 2026-11-21T21:48:56Z

*lfs-1*

Decision: one reader, `links::read_bytes_at(base, path, rev) -> Vec<u8>`; `read_at` (String) is a thin wrapper over it. Rejected: a second image-only git path in the TUI (would re-fork the fallback tries and the LFS rule).
Decision: LFS detection = blob starts with `version https://git-lfs.github.com/spec/v1` and is <= 1024 bytes; smudge via `git -C <base> lfs smudge -- <path>` with the pointer on stdin. Rejected: `git show` with `-c filter.lfs...` / `git cat-file --filters` (needs the .gitattributes at that rev, fails for the farm's pre-gitattributes commits), and trusting the pointer text.
Decision: failure (git-lfs missing, object unfetchable, smudge echoes the pointer) is an `Err` "<path>@<rev>: file is in Git LFS and could not be fetched (<why>)", not the pointer shown as text. Rejected: silently returning the pointer.
Decision: `locate::pinned_file` (the "take out / open externally" temp copy of a pinned file) also goes through read_bytes_at, so `O`/`o` on a pinned LFS file writes the real file, not the pointer.
Decision: TUI image at a rev = new `App::farm_image_at` (bytes -> `ImageView::from_bytes`) + `ImageView.rev` so `doc_link` keeps the pin (history/stars). Only farm paths; `code:` pinned images still open as text (out of scope, not listed in the yak).
Evidence: unit tests `a_pin_to_an_lfs_file_reads_its_content` (real temp repo + git-lfs; fails without the smudge: returns the 131-byte pointer) and `an_lfs_pointer_that_cannot_be_smudged_says_so`; peek and headless frame attached; `cargo test -p yaks` 1530 passed 0 failed, `cargo fmt --check` clean. Note: `peek` of an image still prints `(binary file, N bytes)` (the same as an unpinned image); before it printed the pointer.
Stray note above: an evidence link (the pinned image) written to drive the headless frame.

#### 2026-11-21T21:49:03Z

*lfs-1*

Summary: @rev reads go through links::read_bytes_at (LFS pointer -> git lfs smudge; failure says the file is in LFS and could not be fetched). TUI opens a pinned farm image in the image viewer; pinned_file temp copy smudges too. Gate PASS. Caveat: the BEFORE half of peek-evidence.txt was retyped from the earlier pre-change run (the pre-change binary was overwritten by my rebuild).

#### 2026-11-21T21:49:03Z

*lfs-1 · kind: move · from: shaving · to: shorn · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAHb29ZIBxBCA0aLJyrhJPreUvVwLftHG*

moved: shaving -> shorn

#### 2026-11-21T21:49:27Z

*herd-files · kind: landed · commit: d2a9936c79920d735d9201aabcc9598bab622afa · source: 7290b8bf47cfed4029b9df6b38129a84ce524aa4 · branch: main*

landed: [d2a9936c](https://github.com/joelgwebber/yaks/commit/d2a9936c79920d735d9201aabcc9598bab622afa) yaks-c955: @rev reads smudge Git LFS pointers; pinned farm images open in the image viewer

[herd-pinned](yaks-c955/herd-pinned.txt "Coordinator's reproduction at d2a9936c: the @33f5db7f link in yaks-c955 opens the real LFS image")

#### 2026-11-21T21:54:36Z

*herd-files*

Landed d2a9936c (herd-files). Reproduced with my build: `yaks peek` on manifesto-2.png@33f5db7f and on yaks-a4d8/lfs-test.png@1d0a4386 prints `(binary file, 170083 bytes)` (was the pointer before this build); headless TUI following the pinned link opens doc=image (herd-pinned.txt). Gate on the landed tree: fmt clean, 1530 passed, 0 failed. Diff read: one reader (`links::read_bytes_at`), `locate::pinned_file` routed through it too.

#### 2026-11-21T22:31:17Z

*herd-files · kind: change · section: fixed*

Links pinned to a commit (`path@<rev>`) show files stored in Git LFS (images included) instead of their LFS pointer.
