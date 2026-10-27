---
id: yaks-71c2
title: Search tokenization improvements
status: hairy
type: feature
priority: 3
created: '2026-08-23T02:49:59Z'
updated: '2026-10-27T01:10:50Z'
labels:
- search
needs: agent
---

It would be really helpful if yak searches could be a bit more contextually aware. Some examples:
- When I have a field like `source: ...github/issues/123`, it would be great if `123` matched this.
  We might also consider special forms like `source:123` that use tokenization rules to limit the scope of the search.

---
▸ 2026-08-29T17:34:35Z

Also: multiple tokens vs n-grams, so you can clarify "this exact sequence of tokens" vs "this", "that", and "the other" dispersed throughout the text.

---
▸ 2026-10-27T01:10:50Z [Joel Webber]

requested: needs agent

Tidy request: change only the line structure of this yak's description and every note, so each renders as intended under CommonMark.

Put a blank line between blocks, make run-on facts a list, and fence pasted output. Never change a word, a note header (`▸ <ts> [actor]`) or the frontmatter. Edit the file at `yaks path yaks-71c2` directly: `update --description` replaces the whole body, notes included. Then `yaks pickup yaks-71c2 --drop-request`, which removes this request note, so no new note is left behind.
