---
id: yaks-a57c
title: 'yaks attach <dir>: attach a directory with its layout'
status: shaving
type: feature
priority: 2
created: '2026-11-17T21:41:18Z'
updated: '2026-11-17T21:42:27Z'
parent: yaks-0229
tags:
- cli
- doks
---

Part of yaks-0229 (scope agreed by Joel). `yaks attach <id> <dir>` fails with `error reading d: Is a directory`, so an agent that wants to attach a small code directory attaches a `.tgz` instead.

## Want

- `yaks attach <id|dok|session> <dir>` stores the directory with its layout under the attachment directory (`<herd>/yaks/<id>/<dir-name>/...`), in one write and one commit.
- Use the same limits and skip rules as a directory source (`sources::MAX_DIR_FILES`, `MAX_DIR_BYTES`, its skip list: `.git`, `node_modules`, `target`, ...). Reuse that walk; don't write a second one.
- The link appended to the body names the directory (`[<dir-name>/](<id>/<dir-name>/)`), unless `--no-link`.
- Refuse `--name` with a directory only if it can't be made to mean "store under this directory name".
- The `yaks` skill says: attach a directory, never an archive. Update `--help`, `docs/cli.md`, and `.agents/skills/yaks` in the same change.

## Evidence

Tests: a nested directory attached to a yak and to a dok keeps its layout. Over-limit refuses and writes nothing. Skip names are skipped. A real run of the built binary, shown in the note.

#### 2026-11-17T21:42:27Z

*sources-triage · kind: move · from: hairy · to: shaving · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving
