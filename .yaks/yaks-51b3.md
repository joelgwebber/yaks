---
id: yaks-51b3
title: Parent/child references in yak details broken across sheds
status: hairy
type: bug
priority: 2
created: '2026-10-17T14:03:33Z'
updated: '2026-10-17T14:04:18Z'
labels:
- tui
- sheds
- cli
---

When yaks are created and statuses are updated across sheds, parent and child references in the detail view can get broken. I've seen a few different cases that are a little hard to quickly characterize, so we'll probably need to research a taxonomy of failure modes to deal with.

As always (and per yaks-991c), please ensure all lib/CLI affordances take these issues into account in their own interfaces.
