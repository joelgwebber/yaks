---
id: yaks-db66
title: 'Images and files in doks and session notes: a sibling-directory convention'
status: hairy
type: feature
priority: 2
created: '2026-11-06T21:37:11Z'
updated: '2026-11-06T21:48:01Z'
parent: yaks-3f35
tags:
- doks
- design
needs: agent
---

From yaks-3f35, item 4. Yaks have attachments (`yaks attach`, stored in `<herd>/yaks/<id>/`); doks and session notes have nothing. The schism import put each image in a directory named after its document, beside it:

- `doks/screenshots/2026-03-08-composite-objects.md` embeds `2026-03-08-composite-objects/composite_objects.png`
- `sessions/2025/03/<note>.md` has `<note>/screenshot.png`

That works (a relative link going down resolves), but no skill describes it and no command knows it. The `yaks-research` skill only says to embed an image saved as a source (`/sources/<id>/shot.png`), which is wrong for a screenshot the farm itself made.

## To decide

Bless the sibling directory as the attachment directory of any farm document, mirroring yaks (`<id>.md` + `<id>/`), or send all images through sources. Lean: bless it.

If blessed:

- `yaks attach <dok-or-session-path> <file>...` stores into it and writes the link, with the same `--caption` / `--no-link`.
- `yaks mv` moves the directory with its dok, and `yaks doctor` checks links into it (check what `mv` does today).
- `yaks lfs` / `yaks weight` count it.
- The `yaks` skill ("Doks", "Links") and `yaks-research` say when to attach to a dok versus save a source: the farm's own artifact is an attachment; outside material is a source.

## Evidence

A test attaching to a dok and moving it; the skill and `docs/` text.

---
▸ 2026-11-06T21:37:49Z [schism-feedback]
kind: ask
needs: human
---

asked: needs human

Images and other files beside a dok or session note: bless the sibling directory (`doks/x.md` + `doks/x/`, as yaks have `<id>.md` + `<id>/`), with `yaks attach <dok-path>` writing into it and `yaks mv` moving it? Or route every image through `yaks source add`?

Lean: bless the sibling directory for the farm's own artifacts (screenshots it made, diagrams), and keep sources for outside material.

---
▸ 2026-11-06T21:48:01Z [sched-1]
kind: answer
needs: agent
---

Agreed, I think that's a very easy pattern to follow, and also easy to browse on the filesystem.
