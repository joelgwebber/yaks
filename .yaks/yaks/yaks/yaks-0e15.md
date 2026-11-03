---
id: yaks-0e15
title: Links from documents to yaks failing
status: shorn
type: task
priority: 3
created: '2026-11-03T04:11:38Z'
updated: '2026-11-03T21:20:56Z'
parent: yaks-f677
tags:
- tui
---

Eg, `wiki/design/knowledge-store.md` links to `yaks-44b3`, but attempting to follow it yields;
> `wiki/design/yaks-44b3: not in this farm`

---
▸ 2026-11-03T21:13:55Z [delta:starred-batch]
kind: move
from: hairy
to: shaving
thread: ksQQGwTub5pWSfWv5Hlp3m1Av5LOAD8vcpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

[doclink-frames](yaks-0e15/doclink-frames.txt)

---
▸ 2026-11-03T21:20:54Z [delta:starred-batch]

Fixed (with yaks-d0b0, the same bug): the link grammar now reads a destination that is a known yak id as `Link::Yak`, from anywhere, before any page-relative resolution. A page's `[yaks-44b3](yaks-44b3)` was being joined to the page's directory (`wiki/design/yaks-44b3`).

- [fn parse](code:src/links.rs#L85) checks `is_yak` on a bare, unpinned, slash-free destination first; `exists`, `label` and `peek` know the new variant (`yaks peek yaks-44b3` prints the yak's head).
- The TUI maps it to `Target::Task`, so Enter opens the yak.

Evidence: `the_grammar` (all three `From`s), the new TUI test `a_markdown_link_to_a_yak_from_a_page_opens_the_yak`, and a headless run on the real farm (attached): Wiki tab, open `wiki/design/knowledge-store.md`, Tab to the link, Enter, and frame 8 shows `sel=yaks-44b3` / `→ yaks-44b3`.

---
▸ 2026-11-03T21:20:56Z [delta:starred-batch]
kind: move
from: shaving
to: shorn
thread: ksQQGwTub5pWSfWv5Hlp3m1Av5LOAD8vcpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
