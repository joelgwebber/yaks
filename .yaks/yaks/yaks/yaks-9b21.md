---
id: yaks-9b21
title: Config.yaml still has `prefix:`
status: shorn
type: bug
priority: 3
created: '2026-09-21T20:15:03Z'
updated: '2026-10-27T01:10:50Z'
labels:
- herds
needs: agent
---

Maybe skip it altogether? Or require `herds: [herd-name]:` as a baseline config?
Note that this will affect things like `yaks init`.

---
▸ 2026-09-21T20:59:33Z [agent]

config.yaml default-herd key is now 'herd:' (was 'prefix:'). read_config and parse_pointer accept both ('herd' | 'prefix' alias) so on-disk farms keep working; set_config_prefix and init now WRITE 'herd:' (a rename-herd migration also upgrades a legacy prefix: line in place). Migrated .yaks/config.yaml and tests/fixtures/farm/.yaks/config.yaml to herd:. Smoked: init --herd writes herd:, a legacy prefix: config still reads.

---
▸ 2026-09-21T21:02:13Z [agent]

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-10-27T01:10:50Z [Joel Webber]

requested: needs agent

Tidy request: change only the line structure of this yak's description and every note, so each renders as intended under CommonMark.

Put a blank line between blocks, make run-on facts a list, and fence pasted output. Never change a word, a note header (`▸ <ts> [actor]`) or the frontmatter. Edit the file at `yaks path yaks-9b21` directly: `update --description` replaces the whole body, notes included. Then `yaks pickup yaks-9b21 --drop-request`, which removes this request note, so no new note is left behind.
