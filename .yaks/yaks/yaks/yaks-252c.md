---
id: yaks-252c
title: Hash-checked dok edits (no silent overwrite from a stale read)
status: hairy
type: feature
priority: 1
created: '2026-11-06T04:28:31Z'
updated: '2026-11-06T04:28:31Z'
parent: yaks-fcdc
tags:
- cli
- doks
---

Hearth's `wiki_read` returns a hash and `wiki_edit` refuses an edit whose `base_hash` is stale; yaks has nothing like it. The farm lock orders writes, but `dok set --body` built from a stale read replaces a newer body without a word. With several agent sessions on one farm this loses work.

Lean: `dok show` reports the file's hash, and `dok set` (and any whole-body write) takes `--if-hash <h>`, refusing with a message naming the current hash when it differs. Consider an exact-text replace (`--old/--new`, must match once) so agents never rewrite a whole body.
