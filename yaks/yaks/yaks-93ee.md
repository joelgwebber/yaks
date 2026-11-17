---
id: yaks-93ee
title: Browse and link into archive attachments (zip, tar.gz) read-only
status: hairy
type: idea
priority: 4
created: '2026-11-17T21:41:19Z'
updated: '2026-11-17T21:42:06Z'
tags:
- tui
- doks
---

Filed at Joel's request from yaks-0229 (not for now). Browse inside an archive attachment (`.zip`, `.tar.gz`) read-only in the TUI's directory view, and link into one (`<id>/code.zip#path/in/archive`, or a `!/` form). That needs a decision on the link form, and the `zip`/`tar`/`flate2` crates. Low priority while `yaks attach <dir>` (the sibling yak) removes the reason to attach an archive.

#### 2026-11-17T21:42:06Z

*sources-triage*

Related: yaks-0229 (where it came from), yaks-a57c (attach a directory, which removes most of the need).
