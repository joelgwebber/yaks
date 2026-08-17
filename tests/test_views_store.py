"""Persistent View list: reconcile / load / save + list mutations (yak-6b51).

The store is an overlay on the code-defined built-ins; these pin down that it
round-trips, self-heals from missing/corrupt/old files, always keeps the
built-ins, and restores custom Views.
"""

from __future__ import annotations

from dataclasses import replace

from yaklib.filter import FilterSpec
from yaklib.model import config_dir
from yaktui.view import View, custom_view, default_views
from yaktui.views_store import (
    can_unpin,
    load_views,
    load_working_set,
    move,
    reconcile,
    save_views,
    save_working_set,
    toggle_working_set,
    views_path,
)


def test_save_load_roundtrip_preserves_order_and_pins(tmp_path):
    root = tmp_path / ".yaks"
    root.mkdir()
    views = default_views()
    # Reorder (Recent first) and unpin Shorn; keep Working set last.
    views = [views[3], views[0], views[1], views[2], views[4]]
    views[3] = replace(views[3], pinned=False)  # unpin Shorn (now at index 3)
    save_views(root, views)

    loaded = load_views(root)
    assert [v.key for v in loaded] == ["recent", "status:hairy", "status:shaving",
                                       "status:shorn", "working-set"]
    assert loaded[0].key == "recent" and loaded[0].pinned
    assert loaded[3].key == "status:shorn" and not loaded[3].pinned
    # Structural bits still come from code (not overlaid).
    assert loaded[0].is_flat and loaded[0].sort_by == "updated"


def test_rename_of_builtin_persists():
    stored = [{"key": "status:hairy", "name": "🦬 Todo", "pinned": True}]
    out = reconcile(stored, default_views())
    hairy = next(v for v in out if v.key == "status:hairy")
    assert hairy.name == "🦬 Todo"
    assert hairy.status == "hairy"  # structure intact


def test_unrenamed_builtin_name_follows_code_default():
    # A null stored name means "never renamed" -> use the current code default,
    # so label/emoji changes propagate to already-customized herds.
    stored = [{"key": "status:shaving", "name": None, "pinned": True}]
    out = reconcile(stored, default_views())
    shaving = next(v for v in out if v.key == "status:shaving")
    code_default = next(v for v in default_views() if v.key == "status:shaving")
    assert shaving.name == code_default.name


def test_save_nulls_unrenamed_builtin_names_but_keeps_renames(tmp_path):
    import json
    root = tmp_path / ".yaks"
    root.mkdir()

    save_views(root, default_views())  # nothing renamed
    names = {e["key"]: e["name"] for e in json.loads(views_path(root).read_text())["views"]}
    assert names["status:hairy"] is None  # un-renamed built-in -> null (tracks code)

    vs = default_views()
    vs[0] = replace(vs[0], name="Custom")  # user renames Hairy
    save_views(root, vs)
    names = {e["key"]: e["name"] for e in json.loads(views_path(root).read_text())["views"]}
    assert names["status:hairy"] == "Custom"  # explicit rename preserved


def test_missing_file_falls_back_to_defaults(tmp_path):
    root = tmp_path / ".yaks"
    root.mkdir()
    assert not views_path(root).exists()
    assert [v.key for v in load_views(root)] == [v.key for v in default_views()]


def test_corrupt_file_falls_back_to_defaults(tmp_path):
    root = tmp_path / ".yaks"
    root.mkdir()
    p = views_path(root)
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text("}{ not json")
    assert [v.key for v in load_views(root)] == [v.key for v in default_views()]


def test_new_builtins_are_appended_even_if_overlay_predates_them():
    # Overlay only knows the three status views; Recent (newer) must still show.
    stored = [{"key": f"status:{s}", "pinned": True}
              for s in ("hairy", "shaving", "shorn")]
    out = reconcile(stored, default_views())
    assert [v.key for v in out][-2:] == ["recent", "working-set"]  # appended in default order
    assert len(out) == 5


def test_custom_view_roundtrips(tmp_path):
    root = tmp_path / ".yaks"
    root.mkdir()
    custom = View(name="Auth bugs", key="view:auth", builtin=False, pinned=True,
                  spec=FilterSpec(labels=("auth",), types=frozenset({"bug"})),
                  sort_by="priority", sort_dir="asc", limit=None)
    save_views(root, [*default_views(), custom])

    loaded = load_views(root)
    got = next(v for v in loaded if v.key == "view:auth")
    assert got.name == "Auth bugs" and not got.builtin
    assert got.spec.labels == ("auth",) and got.spec.types == frozenset({"bug"})
    assert got.sort_by == "priority" and got.sort_dir == "asc"


def test_custom_view_factory_and_roundtrip(tmp_path):
    root = tmp_path / ".yaks"
    root.mkdir()
    v1 = custom_view("My filter", FilterSpec(search="foo"),
                     sort_by="updated", sort_dir="desc", limit=20)
    v2 = custom_view("Other", FilterSpec())
    assert not v1.builtin and v1.pinned and v1.key.startswith("view:")
    assert v1.key != v2.key  # generated keys are unique
    assert v1.is_flat and v1.sort_by == "updated"

    save_views(root, [*default_views(), v1])
    got = next(v for v in load_views(root) if v.key == v1.key)
    assert got.name == "My filter" and got.spec.search == "foo" and got.limit == 20


def test_stale_builtin_key_is_dropped_not_resurrected():
    # An overlay entry marked builtin whose key we no longer ship is discarded.
    stored = [
        {"key": "status:hairy", "pinned": True},
        {"key": "status:ancient", "name": "gone", "builtin": True, "pinned": True},
    ]
    out = reconcile(stored, default_views())
    keys = [v.key for v in out]
    assert "status:ancient" not in keys
    assert "status:hairy" in keys and "recent" in keys  # rest intact


def test_config_dir_is_under_xdg_config(tmp_path, monkeypatch):
    monkeypatch.setenv("XDG_CONFIG_HOME", str(tmp_path / "cfg"))
    root = tmp_path / "proj" / ".yaks"
    assert str(config_dir(root)).startswith(str(tmp_path / "cfg" / "yaks"))


# -- pure list mutations -------------------------------------------------------

def test_move_reorders_and_clamps():
    vs = default_views()
    keys = [v.key for v in vs]
    assert move(vs, 0, +1) == 1
    assert [v.key for v in vs] == [keys[1], keys[0], keys[2], keys[3], keys[4]]
    # Clamp at the top.
    assert move(vs, 0, -1) == 0
    assert [v.key for v in vs][0] == keys[1]


def test_default_views_include_recent_and_working_set():
    keys = [v.key for v in default_views()]
    assert keys == ["status:hairy", "status:shaving", "status:shorn",
                    "recent", "working-set"]


def test_toggle_working_set_adds_removes_and_keeps_order():
    ids = []
    ids = toggle_working_set(ids, "yak-a")
    ids = toggle_working_set(ids, "yak-b")
    assert ids == ["yak-a", "yak-b"]           # new stars append
    ids = toggle_working_set(ids, "yak-a")      # toggle off
    assert ids == ["yak-b"]
    ids = toggle_working_set(ids, "yak-a")      # re-add -> goes to the end
    assert ids == ["yak-b", "yak-a"]


def test_working_set_roundtrip_and_fallback(tmp_path):
    root = tmp_path / ".yaks"
    root.mkdir()
    assert load_working_set(root) == []          # missing -> empty
    save_working_set(root, ["yak-1", "yak-2", "yak-3"])
    assert load_working_set(root) == ["yak-1", "yak-2", "yak-3"]


def test_can_unpin_guards_last_pinned():
    vs = default_views()
    # All pinned initially -> any can be unpinned.
    assert can_unpin(vs, 0)
    # Unpin all but one; the last pinned cannot be unpinned.
    for i in range(1, len(vs)):
        vs[i] = replace(vs[i], pinned=False)
    assert not can_unpin(vs, 0)
    assert can_unpin(vs, 1)  # already unpinned -> trivially fine
