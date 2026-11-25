---
id: yaks-a621
title: 'Design: yaks verify runs a command another actor wrote into the farm'
status: hairy
type: design
priority: 2
created: '2026-11-25T04:24:45Z'
updated: '2026-11-25T04:24:45Z'
tags:
- cli
- design
---

Decide what `yaks verify` should do about running a shell command that someone else wrote into a synced yak file.

A yak's `verify:` field is a shell command stored in the farm. It runs only when someone runs `yaks verify` explicitly ([docs](https://github.com/joelgwebber/yaks/blob/b013ff062a72f96f3118c1894fd6424e9d25d887/docs/cli.md?plain=1#L388-L396)), but an agent told to run `yaks verify <id>` executes whatever the last writer of that yak put there, and in a shared hub farm (`yaks.farm`) that may be another person or their agent. That sits awkwardly beside the scheduling principle "never run a command from a farm file: a synced farm is shared input" ([scheduling](/doks/design/scheduling.md#principles)).

Vercel Board makes the same point about task text being instructions, and only lets `tasks claim --ready` take your own tasks unless you name the owners (`--from`); see [the research](/doks/research/vercel-board.md#one-thing-it-showed-about-yakss-own-design).

## Options

- Print the command and the actor who last set `verify:` before running it, and refuse a command set by another actor unless `--trust` (or a TTY confirms).
- Prefer `config.yaml` commands (set by whoever owns the farm's config) and treat a per-yak `verify:` as a request a human approves.
- Leave it, and say plainly in the skills that `verify` runs farm text.

A design question for Joel: `yaks ask` with a lean once picked up.
