---
id: yaks-4ed6
title: update --as stamps the note's actor but the farm commit's Yaks-Actor trailer comes from the environment
status: hairy
type: bug
priority: 3
created: '2026-11-22T17:26:33Z'
updated: '2026-11-22T17:26:33Z'
tags:
- git
- agent
---

Seen 2026-11-22 (herd-files): `yaks update <id> --note x --as herd-probe` wrote notes stamped `herd-probe`, but each per-change commit carries `Yaks-Actor: herd-files` (the checkout's whoami), so `yaks undo --by herd-probe` found nothing. The trailer should be the write's resolved actor (`--as` first), the same one the note gets; `yaks session touched` and `undo --by` depend on it. Check every command with `--as`.
