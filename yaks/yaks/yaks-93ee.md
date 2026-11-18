---
id: yaks-93ee
title: Browse and link into archive attachments (zip, tar.gz) read-only
status: shaving
type: idea
priority: 4
created: '2026-11-17T21:41:19Z'
updated: '2026-11-18T23:10:28Z'
tags:
- tui
- doks
---

Filed at Joel's request from yaks-0229 (not for now). Browse inside an archive attachment (`.zip`, `.tar.gz`) read-only in the TUI's directory view, and link into one (`<id>/code.zip#path/in/archive`, or a `!/` form). That needs a decision on the link form, and the `zip`/`tar`/`flate2` crates. Low priority while `yaks attach <dir>` (the sibling yak) removes the reason to attach an archive.

#### 2026-11-17T21:42:06Z

*sources-triage*

Related: yaks-0229 (where it came from), yaks-a57c (attach a directory, which removes most of the need).

#### 2026-11-18T23:10:28Z

*delta:ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving
