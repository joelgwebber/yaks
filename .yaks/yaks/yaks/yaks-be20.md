---
id: yaks-be20
title: 'Links inside attachments: a sibling-relative link resolves to the yak''s own attachments; attach without linking'
status: shorn
type: bug
priority: 2
created: '2026-11-03T23:07:23Z'
updated: '2026-11-03T23:47:30Z'
tags:
- links
- cli
---

Reported by Joel from the surveyor farm: `survey-1633` has a markdown attachment `survey-1633/fullstory-FilterChip-evidence.md` that embeds its images as `![..](fullstory-FilterChip-01-crop.png)`. The images are attachments of the same yak, but they do not render when the markdown is opened in `yaks tui`.

## Findings

**Why the links fail.** The attachment viewer reads its document `From::Yak` ([fn AttachView::from](code:src/tui/handlers.rs#L115-L120)). In `links::parse`, a relative link from a yak means *from the farm root* ([fn parse](code:src/links.rs#L88-L157)), so `fullstory-FilterChip-01-crop.png` becomes `Farm{path: "fullstory-FilterChip-01-crop.png"}`, which does not exist. The file is on disk at `<herd>/yaks/<id>/`, beside the markdown, so the link is *correct* as markdown (any other viewer resolves it) and wrong only in yaks.

**A second break behind it.** Even once it resolves as an attachment, the image renderer keeps the raw destination as the `Target::Artifact` ([Tag::Image](code:src/tui/mdrender.rs#L508-L522)), and `image_slot` and `resolve_attachment` split it as `<id>/<file>`; a bare `name.png` would still not open or draw inline.

**The same mistake in other places.** Nothing reads links *inside* attachments except the viewer:

- `doctor` ([links::edges](code:src/links.rs#L444-L478)) scans yak bodies and pages only, so a broken link in an attachment is never reported (`yaks doctor` on the surveyor farm is silent about this one).
- `rename-attachment` and `promote` rewrite links in yak bodies and pages only, so renaming or promoting `x.png` leaves a sibling markdown pointing at nothing.
- `promote --page` moves a markdown attachment to `wiki/..`, where its sibling-relative links would resolve against the page's directory instead.
- A yak rename is safe for bare sibling links (the best argument for the relative form: it survives the id changing).

## Why the images are on the yak at all

The CLI gave the agent no other way. `yaks attach` takes one file and always appends a link to the yak body. The surveyor's `AGENTS.md` tells agents to `yaks attach` every file of an evidence bundle (the markdown and each PNG it links), so each PNG also lands in the yak body as `![..]`, and `survey-1633` shows 4 images plus the markdown link. Seventeen surveyor yaks hold attachments `.md` files. The images are only needed by the evidence markdown; the yak should link the one document.

## Plan

1. A document read from an attachment resolves a bare `name` as a sibling attachment of the same yak (`From::Attachment(id)`), one canonical `Link::Attachment`. Other relative forms from there (`sub/x`, `../x`) stay refused; `/farm/path`, `code:` and URLs work as before.
2. `Target::Artifact` is the canonical `<id>/<file>` for any attachment link, so open and inline images work from a sibling-relative link.
3. `doctor` checks links inside markdown attachments; `rename-attachment` and `promote` rewrite the sibling references; `promote --page` rewrites sibling-relative links to the `<id>/<file>` form so the page keeps working.
4. `yaks attach <id> <file>... --no-link`: store files without linking them in the body, several at once (one commit), so a bundle is its document plus its parts.
5. Docs, `--help`, the `yaks` skill, and the surveyor's `AGENTS.md` / `arbiter` skill say which to use. A decision on tidying the existing seventeen surveyor yaks is a separate ask.

---
▸ 2026-11-03T23:08:12Z [indent-links]
kind: move
from: hairy
to: shaving
thread: ksQQNSYysfbwRGKL-KGL8e-aQpLOAEKHPJIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

[survey-1633-evidence-frame](yaks-be20/survey-1633-evidence-frame.txt)

---
▸ 2026-11-03T23:22:53Z [indent-links]

Fixed in the working tree (not committed or landed: awaiting a go-ahead).

## What changed

- **One rule:** a document read from a yak's attachment resolves a bare `name` as a sibling attachment of that yak (`From::Attachment`, [fn parse](code:src/links.rs#L88)); the `<id>/<file>`, `/farm/path`, `code:` and URL forms are unchanged, and a deeper relative path or a pinned name is not a sibling. In a yak or on the CLI a bare name still means the farm root.
- **Viewer:** `AttachView::from` uses it, and an attachment link is always the canonical `<id>/<file>` target, so inline images, open and rename work from a bare link.
- **`doctor`:** reads the links inside markdown attachments (it reported nothing about them before).
- **`rename-attachment`:** rewrites markdown attachments too, keeping each link's form (bare stays bare).
- **`promote --page`:** a promoted document's bare sibling links become `<id>/<file>`, so the page keeps working; `promote` and links to the moved file are rewritten in markdown attachments too.
- **`yaks attach <id> <file>... [--no-link]`:** several files in one write (one farm commit); `--no-link` stores them and leaves the body alone. `--name` is for one file.
- **Docs and help:** `docs/cli.md`, the `yaks` skill, `src/brief.rs` and the clap help for `attach`. In the surveyor tree: `AGENTS.md` and the `arbiter` skill now say `yaks attach ID DIR/* --no-link`.

## Evidence

- `cargo test`: 947 unit tests and every integration test pass; new ones: `a_bare_name_in_an_attachment_is_a_sibling`, `an_evidence_document_links_its_sibling_attachments_by_bare_name`, `attach_many_links_each_file_in_order`, `a_markdown_attachment_draws_its_sibling_images_in_place`.
- My binary on the real surveyor farm: `yaks doctor` now checks `survey-1633`'s evidence document, its four `fullstory-FilterChip-*` links resolve, and the two findings left are unrelated (`survey-b436` and `survey-2c6d` link to repo paths as farm paths).
- The headless TUI opens `survey-1633/fullstory-FilterChip-evidence.md` and draws its images in place: [frame](yaks-be20/survey-1633-evidence-frame.txt).
- Scratch farm: `yaks attach t-xxxx e.md a.png --no-link` leaves the body untouched, and `doctor` flags only the missing `zz.png`, as `t-xxxx/e.md:L5`.

## Not done

- The seventeen existing surveyor yaks with an evidence bundle keep their per-image links in the body. They render now (the document's links work) but are noisy; tidying them is a separate decision.
- `yaks find` and `yaks refs` do not read attachments (unchanged; `doctor` is the one place that needed to).

---
▸ 2026-11-03T23:47:29Z [indent-links]

Landed as `522ff21` on `main`, pushed to `origin` and `local`. Docs and skill text keep to yaks vocabulary (no surveyor or other repo references), per Joel.

Installed with `cargo install --path .` into `~/.cargo/bin`; a running `yaks tui` keeps the old binary until restarted.

Joel is tidying the surveyor farm and its guidance himself, so `yaks-9173` is his.

---
▸ 2026-11-03T23:47:30Z [indent-links]
kind: move
from: shaving
to: shorn
thread: ksQQNSYysfbwRGKL-KGL8e-aQpLOAEKHPJIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
