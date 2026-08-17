---
id: yak-6c15
title: Built-in view label changes do not reach an existing views.json (v1 baked names)
type: bug
priority: 2
created: '2026-08-17T14:51:00Z'
updated: '2026-08-17T14:52:52Z'
labels:
- ui
---

The reconcile propagation fix (yak-1bef) compares a stored built-in name to the CURRENT code default to decide 'renamed vs default'. But herds that wrote views.json before a label change have the OLD default baked in (e.g. name '⭐ Working set', '✂️  Shaving'), which differs from the new default, so it's mistaken for a user rename and kept forever -- the Starred rename and the razor TAB label never appear for existing users.

Fix: version the store. Bump _VERSION to 2; load_views accepts v1 and v2 and, for v1 files, ignores stored built-in names entirely (treat as un-renamed -> code default), migrating on the next save to v2 where an un-renamed built-in stores a null name. Order/pins/custom views preserved. Tradeoff: a genuine built-in rename made during the brief 0.1.92-0.1.97 window is dropped (rare; re-nameable via the picker).

---
▸ 2026-08-17T14:52:52Z
Fixed. Versioned the views store: _VERSION -> 2. load_views accepts v1 and v2; for v1 files (which baked resolved built-in labels into name) it ignores stored built-in names via reconcile(ignore_builtin_names=True), so current code labels win; the next save rewrites as v2 (null name = un-renamed). Verified against the real repo views.json: working-set -> '⭐ Starred', shaving -> '🪒 Shaving', order/pins preserved. Test: test_v1_file_ignores_baked_builtin_names. Full suite 168 pass, ruff-clean.
