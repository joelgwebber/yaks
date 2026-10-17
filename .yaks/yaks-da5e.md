---
id: yaks-da5e
title: Allow markdown links to repo files / source
status: shaving
type: feature
priority: 2
created: '2026-10-17T14:13:34Z'
updated: '2026-10-17T18:14:58Z'
parent: yaks-faf5
labels:
- markdown
- tui
---

Existing markdown links can only reference files relative to the `.yaks` dir. This is fine for attachments, but it would be really convenient to be able to reference other files in the repo as well.

## Link format
Not entirely sure about link format -- propose something that fits reasonably cleanly into other markdown renderers, so it's likely to do something reasonable. Maybe as dumb as just `[code](../src/whatever.rs)`, which ought to "just work".

## Line ranges
In the case of source code, it would also be helpful to support link number/range references. Again, preferably in a way that won't break other markdown renderers. Eg, is it the norm to use `#fragments` for this, like `[code](../src/whatever.rs#L123:145)`?

## Commit-specific
Ooh, it would be _really_ cool if you could reference repo files by `git` hash as well, so a yak can maintain historical connection to the _actual_ code, before it's changed. Again, open to format ideas; this case matters less for other markdown renderers, though it would be nice if the path still hit the right spot in the filesystem.

---
▸ 2026-10-17T18:14:58Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-17T18:14:58Z [delta-lead]

Joel agreed the format (2026-10-17, in the Delta thread): plain relative links, GitHub-style `#L` line fragments, and the GitHub blob-URL shape for pinned commits. Showing a resolved git blob can be deferred to a follow-up yak; it may dovetail with yaks-699b (viewing files inside the TUI).

Claim (yakherd delta-lead): owner **srclinks-1**.

- **Format.** `[x](../src/foo.rs)`, `[x](../src/foo.rs#L12)`, `[x](../src/foo.rs#L12-L40)`. Paths are relative to the yak file (`.yaks/`), so they also work on GitHub. Pinned: `[x](../blob/<sha>/src/foo.rs#L12)`, where `../blob/<sha>/` is a GitHub-like prefix that yaks recognises (decide the exact recognised shape and record it as a `Decision:` note; it must stay a harmless relative link elsewhere).
- **Rendering (src/tui/mdrender.rs, `link_target` + `Tag::Link`).** A new `Target::Source { path, lines: Option<(u32, u32)>, rev: Option<String> }` in src/tui/detail.rs for links that resolve to a repo file (outside `.yaks/`, inside the repo); attachments stay `Artifact`. Show the link text; the status-bar destination shows `path:L12-L40` (and `@<sha7>` when pinned). Reuse yaks-3b8d's status-bar line.
- **Following (src/tui/detail_nav.rs, `follow_link` / `O`).** Open the file in `$VISUAL`/`$EDITOR` at the line when set (suspend and restore the TUI around it, like other external opens; check src/tui/runtime.rs for the terminal setup). Otherwise use the OS opener (`xdg-open`/`open`) as attachments do. For a pinned rev: if the file at that rev differs from the working tree (`git diff --quiet <rev> -- <path>`), write `git show <rev>:<path>` to a temp file and open that read-only; otherwise open the working copy. If this turns out hairy, defer the pinned part to a follow-up yak under yaks-da5e and say so.
- **Docs.** docs/tui.md, help_content, the yaks skill's "What the TUI renders" line (add a one-line example of a source link).

Out of scope: an in-TUI file viewer (yaks-699b).

Evidence: failing-first tests for parsing (path, both fragment forms, pinned prefix), Target classification, the status-bar text; a headless frame; a manual run with `EDITOR` set to a script that records its arguments. Judge: coordinator at landing.
