---
id: yaks-f04d
title: Known labels in configuration
status: hairy
type: feature
priority: 3
created: '2026-08-26T13:20:21Z'
updated: '2026-10-27T01:10:50Z'
labels:
- config
- labels
needs: agent
---

We don't want to *limit* labels to a predefined set, but it would be helpful to have some of them predefined in the config.
That way we can give them descriptions, which can in turn provide useful context to an agent for picking labels (it tends to make a lot of shit up now).

---
▸ 2026-09-23T21:35:38Z [coordinator]

Shaved as parent of yaks-7cb3 (lightning-round lane 'labels').

---
▸ 2026-09-23T21:45:01Z [lane-labels]

yaks-7cb3 landed label normalization on lane-labels: model::normalize_labels is the shared parser (commas/whitespace split) and doctor's malformed-label check -- a known-labels config can build on is_canonical_label/normalize_labels.

---
▸ 2026-10-12T21:49:45Z [Joel Webber]

moved: shaving -> hairy

---
▸ 2026-10-27T01:10:50Z [Joel Webber]

requested: needs agent

Tidy request: change only the line structure of this yak's description and every note, so each renders as intended under CommonMark.

Put a blank line between blocks, make run-on facts a list, and fence pasted output. Never change a word, a note header (`▸ <ts> [actor]`) or the frontmatter. Edit the file at `yaks path yaks-f04d` directly: `update --description` replaces the whole body, notes included. Then `yaks pickup yaks-f04d --drop-request`, which removes this request note, so no new note is left behind.
