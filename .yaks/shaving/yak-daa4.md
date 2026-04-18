---
id: yak-daa4
title: Source field missing in tui
type: bug
priority: 2
created: '2026-04-18T23:05:08Z'
updated: '2026-04-18T23:07:51Z'
---

If present in the source, it should be shown as a clickable link in the detail view.

### 2026-04-18T23:07:51Z
Fixed: source field now renders as a navigable 'link' DetailLine in the TUI detail pane. Relaxed _open_externally to skip the existence check for http(s):// URLs and dispatch them via open/xdg-open. Pressing Enter or O on the Source line opens the URL in the default browser.
