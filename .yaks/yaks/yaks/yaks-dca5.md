---
id: yaks-dca5
title: 'dok fields: nested maps in schemas and --where'
status: hairy
type: bug
priority: 3
created: '2026-11-08T21:41:05Z'
updated: '2026-11-08T21:41:05Z'
---

Found importing a real record into a scratch farm (Hearth's design dok /doks/design/calcifer-import.md in the Hearth farm).

A dok whose frontmatter has a nested map (the college records: `engagement: {housing_upperclass_pct: 57, coop_participation_pct: 93, ...}`) is kept as written, but:
- the schema field types (`text number bool date datetime money link list`) have no map, so `yaks doctor` warns "fields the college schema does not list: engagement" until `engagement` is declared as `text`, which is not what it is;
- `yaks dok query --where 'engagement.coop_participation_pct=>90'` and `--where 'engagement=~coop'` both return nothing, with no error.

Decide: support a `map` type with dotted `--where` paths, or say plainly in `dok query` (an error for a path into a map) and in the yaks-research skill that fields are flat. Whichever, the silent empty result should go.
