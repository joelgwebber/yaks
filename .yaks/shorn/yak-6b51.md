---
id: yak-6b51
title: View picker + pin/reorder/rename/delete + persistence
type: feature
priority: 2
created: '2026-08-16T23:15:40Z'
updated: '2026-08-17T01:13:49Z'
parent: yak-4473
labels:
- ui
depends_on:
- yak-5892
---

Tranche 2 of yak-4473 (deps yak-5892). The View management surface + persistence.

Key v opens a picker listing every View. Pin/unpin, reorder, rename, delete happen there (unlimited space). Pinned Views = the tab bar; pin order = tab order, so 'reorder tabs' and 'manage pins' are one operation. Non-pinned Views live only in the picker; the picker is also the tab-strip overflow affordance. Built-in status Views are un-deletable (pin/reorder only).

Persistence: pins + order + any custom View defs go to USER-SPECIFIC DURABLE storage under $XDG_CONFIG_HOME/yaks/<slug>/ (default ~/.config/yaks). Rationale: these are user-authored intent (losing them loses work), unlike the rebuildable ~/.cache (index, collapsed_ids). Atomic temp+rename writes, mirroring the index.

Done when: status Views can be reordered/pinned via v and the order persists across restarts.

---
▸ 2026-08-17T01:13:49Z
Done. View picker + durable persistence.

PERSISTENCE (tested foundation):
- model.config_dir(root) -> $XDG_CONFIG_HOME/yaks/<slug>/ (default ~/.config), for DURABLE user intent, distinct from the rebuildable cache_dir (~/.cache). Shared _project_slug with cache_dir.
- yaktui/views_store.py: views.json overlay. load_views reconciles the stored overlay with code-defined default_views(): built-in structure always from code (only name/pinned/order overlaid); built-ins the overlay omits are appended (new built-ins always appear); custom Views restored from their def; a stored 'builtin' key we no longer ship is dropped. Missing/corrupt/old-version file -> defaults. Atomic writes; reads never raise. FilterSpec (de)serialized (frozensets<->lists). Pure list ops move()/can_unpin() for the picker.
- View gained key (stable id: status:<s>, recent) + pinned (default True).
- conftest now also sandboxes XDG_CONFIG_HOME.

UI:
- TUI loads self.views = views_store.load_views(root) on startup.
- Tab bar renders only PINNED views (render.pinned_indices); _switch_tab cycles the pinned set; mouse hit-testing maps tab position -> pinned view.
- _open_view_picker (key v): modal listing ALL views with pin/active/lock markers + counts. Enter opens/activates, p pin/unpin (guards the last pinned tab), J/K reorder, r rename (built-ins allowed), d delete (built-ins blocked). Every mutation persists immediately.

Tests: tests/test_views_store.py (save/load roundtrip incl. order+pins, rename overlay, missing/corrupt fallback, new-builtin append, custom-view roundtrip, stale-builtin drop, config_dir location, move/can_unpin). Full suite 160 pass; new files ruff-clean (only pre-existing E402/E741/F401 in the shared files, +1 structural E402 for a new import in tui's post-sys.path block). Picker is curses UI -> unit-tested at the logic layer; the modal itself wants a live smoke (v to open; pin/reorder/rename; restart to confirm persistence). Unblocks yak-597c (working set) + yak-a373 (saved views), which build on this store.
