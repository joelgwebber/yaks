---
id: yak-7cd3.2
title: Extract yaklib/artifacts.py and yaklib/clipboard.py
type: task
priority: 2
created: '2026-04-14T13:22:29Z'
updated: '2026-04-14T14:25:05Z'
depends_on:
- yak-7cd3.1
commit: e3ce05d
parent: yak-7cd3
---

Move artifacts_dir, parse_artifacts, _ARTIFACT_LINE_RE from yak.py into yaklib/artifacts.py. Move read_clipboard_png (yak.py) and tui.py's _copy_to_clipboard into yaklib/clipboard.py. Both are self-contained and have test coverage (test_cli_artifacts.py). Wire back-compat re-exports from yak.py.
