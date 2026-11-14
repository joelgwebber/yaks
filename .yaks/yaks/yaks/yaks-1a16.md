---
id: yaks-1a16
title: Keyboard tweaks for tag auto-completion
status: shaving
type: task
priority: 3
created: '2026-11-11T20:23:06Z'
updated: '2026-11-14T13:25:15Z'
tags:
- tui
---

Currently, the tag autocomplete affordance eats `<tab>` right after you select a tag. Looks like this is because it leaves the cursor right after the tag just added, so `<tab>` pushes you back into the completion overlay.

"Fixing" this may be as simple as just adding a space after the selected tag, so that you can tab to the next field.

---
▸ 2026-11-14T11:32:22Z [linkplan-lead]
kind: move
from: hairy
to: shaving
thread: ksQQw3f_HRXkT-eF6oV8kx3Nn5LOAFi59ZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-14T13:25:15Z [tags-1]
kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)
