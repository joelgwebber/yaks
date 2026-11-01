---
id: yaks-1f34
title: 'Frontmatter field model: ordered fields, YAML-subset values, exact round-trip'
status: hairy
type: feature
priority: 1
created: '2026-11-01T18:53:10Z'
updated: '2026-11-01T18:53:10Z'
parent: yaks-f301
labels:
- wiki
- schema
---

Implements yaks-8384. A `Doc { fields, body }` with an ordered key->Value map parsed from a YAML subset, byte-exact round-trip of untouched keys, one-key edits that rewrite only that key. `Task` keeps typed accessors and gains `fields`; the list fast path stays. Failing-first tests: round-trip of every yak in this farm and the `joel` farm, nested maps (`generated: {by, at}`, `sources:` list of maps), comments, quoting.
