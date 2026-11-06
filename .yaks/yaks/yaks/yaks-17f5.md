---
id: yaks-17f5
title: 'Command: write a session note into sessions/'
status: hairy
type: feature
priority: 2
created: '2026-11-06T04:28:30Z'
updated: '2026-11-06T04:28:33Z'
parent: yaks-1c89
depends_on:
- yaks-bca6
tags:
- cli
- doks
---

A command that writes a session note into the farm: one file under `sessions/YYYY/MM/`, `type: session-review`, in the shape settled on the design yak, committed under the actor's name like any other write. Model-free: the agent supplies the narrative, turns and lessons; yaks supplies the path, the frontmatter (actor, host, harness, day, time, verdict counts) and the commit, and prints the path.

Mechanical help worth having: list the yaks, notes and doks the actor wrote during the session (from its farm commits since a given time), so the note can cite them instead of restating them.

Keep the file shape compatible with Hearth's `session-review` notes, so the ones `import-hearth` brought in read the same.

Docs and help in the same change: `docs/cli.md`, clap help, the `yaks` skill's command table.
