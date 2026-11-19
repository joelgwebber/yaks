---
type: session-review
title: 'Files, directories and archives: one seam, then archives and stars'
source: session-note
actor: herd-files
person: Joel Webber
host: joel
harness: delta
thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG
day: 2026-11-19
written: '2026-11-19T04:13:29Z'
good: 3
---

## Summary

Joel asked for yaks-93ee (browse and link into archives) with care for the growing file/dok/yak handling. A design pass found the question "what does this link name, and how is it shown" answered in about ten places; the [design dok](/doks/design/files-directories-archives.md) proposed one lister, one resolver and one TUI opener first. Joel agreed and added yaks-4381 (star anything), which fit as "a subject's name is its link destination". Landed in order: yaks-8218 (`walk_dir` followed symlinks; symwalk-1, Sonnet), yaks-84fe (the seam, herd-files; also fixed yaks-1d2b), then in parallel yaks-93ee (archive-1, Sonnet) and yaks-4381 (star-1, Sonnet). Gate on `973a47e1`: 1522 passed, fmt clean; pushed to `main` locally and installed.

## Turns

- good: design read of yaks-93ee; design dok, two bugs filed (yaks-1d2b, yaks-8218), three asks answered.
- good: yaks-4381 folded into the design; refactor yak yaks-84fe filed, dependencies wired.
- good: delegated and landed the batch; every worker change re-run by the coordinator with its own build.

## Saved

- yaks-1d2b: Links to a dok's or source's directory (x/sketches/, /doks/x/sketches/) are reported as ../ by doctor and refused by peek (hairy -> shorn)
  - [Fixed by yaks-84fe (herd-files): `links::parse` keeps a trailing `/` on a far...](/yaks/yaks/yaks-1d2b.md#2026-11-19t023314z)
- yaks-4381: Ability to pin/star any file/resource that can be rendered in the detail view (hairy -> shaving)
  - [Claim (yakherd `herd-files`). Owner: worker `star-1` (Sonnet 5.5), on top of...](/yaks/yaks/yaks-4381.md#2026-11-19t023418z)
  - [landed: 58eb9a62 yaks-4381: star anything the pane shows; Starred lists place...](/yaks/yaks/yaks-4381.md#2026-11-19t040106z)
  - [Coordinator review (herd-files): landed as `58eb9a62`, plus two follow-ups of...](/yaks/yaks/yaks-4381.md#2026-11-19t041134z)
- yaks-8218: walk_dir follows symlinks: yaks attach <dir> and source add <dir> copy a link's target from outside the tree (hairy -> shaving)
  - [Claim (yakherd `herd-files`). Owner: worker `symwalk-1` (Sonnet 5.5). Scope:...](/yaks/yaks/yaks-8218.md#2026-11-19t014144z)
  - [landed: 22b1c30d yaks-8218: walk_dir never follows symlinks; attach reports w...](/yaks/yaks/yaks-8218.md#2026-11-19t021137z)
  - [Coordinator review (herd-files): landed as `22b1c30d`, plus `543403df` (rewra...](/yaks/yaks/yaks-8218.md#2026-11-19t021711z)
- yaks-84fe: Refactor: one container lister and one resolver for files, directories and subjects (open_place, links::dest) (hairy -> shorn)
  - [Claim (yakherd `herd-files`): the coordinator does this one itself (shared de...](/yaks/yaks/yaks-84fe.md#2026-11-19t014145z)
  - [Decision: an attachment link with an empty name, `<id>/`, is the yak's whole...](/yaks/yaks/yaks-84fe.md#2026-11-19t023313z)
  - [Done (herd-files). The shape, per the design:](/yaks/yaks/yaks-84fe.md#2026-11-19t023313z-1)
- yaks-93ee: Browse and link into archive attachments (zip, tar.gz) read-only
  - [Batch note (herd-files): symwalk-1 on yaks-8218, model claude-sonnet-5-5, bas...](/yaks/yaks/yaks-93ee.md#2026-11-19t014212z)
  - [Claim (yakherd `herd-files`). Owner: worker `archive-1` (Sonnet 5.5), on top...](/yaks/yaks/yaks-93ee.md#2026-11-19t023418z)
  - [Batch note (herd-files), base 84ffb8b7: archive-1 on yaks-93ee (claude-sonnet...](/yaks/yaks/yaks-93ee.md#2026-11-19t023533z)
  - [landed: 91d46c73 yaks-93ee: browse and link into zip/tar/tgz attachments read...](/yaks/yaks/yaks-93ee.md#2026-11-19t033640z)
  - [Coordinator review (herd-files): landed as `91d46c73`.](/yaks/yaks/yaks-93ee.md#2026-11-19t034234z)

## Lessons

- The seam paid off at once: archives touched `entries`, `locate` and the viewer constructors and no opener; the two parallel workers met only in `follow.rs` and one `match` (star-1's explicit arms made the merge a compile error, not a silent miss).
- Cheap-model workers' tests passed, but the coordinator's own runs still found things: two over-long help lines (8218), and a starred archive member labelled by its herd path (4381 merge). Budget the review.
- `<id>/` was linked by the TUI but meant a farm path to the grammar; the old TUI hid it with a special case. Moving decisions into one resolver surfaces such mismatches.
- Friction: headless TUI tab numbers differ per farm (`6` hit Starred, not Doks); `yaks dok set --old` needs `--old=...` when the text starts with `-`; BSD `sed -i` needs a suffix.
