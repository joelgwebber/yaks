---
id: yak-7cd3.5
title: Extract yaklib/commands.py + yaklib/parser.py
type: task
priority: 2
created: '2026-04-14T13:22:29Z'
updated: '2026-04-14T16:21:20Z'
depends_on:
- yak-7cd3.3
commit: 6a7d605
parent: yak-7cd3
---

Move all cmd_* functions into yaklib/commands.py and build_parser() into yaklib/parser.py. yak.py collapses to an entry shim: PEP 723 header, sys.path setup, argparse dispatch, main(). Target ~50 lines. Verify 'uv run scripts/yak.py --help' still works and all 20 CLI tests pass unchanged.
