"""Persistent, user-specific View list (yak-6b51).

Durable user intent — View order, pins, renames, and (later) custom Views —
lives under $XDG_CONFIG_HOME/yaks/<slug>/views.json, NOT the rebuildable
~/.cache. The built-in Views are always defined in code; the stored file is an
overlay recording the user's customizations, reconciled against the current
built-ins on every load:

- built-in Views always exist (structure from code; name/pinned/order from the
  overlay), so a missing/corrupt file, or a file written by an older version,
  degrades gracefully to sensible defaults;
- built-ins the overlay doesn't mention (e.g. a newly shipped one) are appended,
  so new built-ins always appear;
- custom Views are restored from their stored definition.

Writes are atomic (temp + rename); reads never raise.
"""

from __future__ import annotations

import json
from dataclasses import replace
from pathlib import Path

from yaklib.filter import FilterSpec
from yaklib.model import atomic_write, config_dir

from yaktui.view import View, default_views

_VERSION = 1


def views_path(root: Path) -> Path:
    return config_dir(root) / "views.json"


# -- (de)serialization ---------------------------------------------------------

def _spec_to_dict(spec: FilterSpec) -> dict:
    return {
        "statuses": sorted(spec.statuses),
        "types": sorted(spec.types),
        "priorities": sorted(spec.priorities),
        "labels": list(spec.labels),
        "search": spec.search,
        "ready_only": spec.ready_only,
        "tangled_only": spec.tangled_only,
        "parent": spec.parent,
    }


def _spec_from_dict(d: dict | None) -> FilterSpec:
    d = d or {}
    return FilterSpec(
        statuses=frozenset(d.get("statuses") or []),
        types=frozenset(d.get("types") or []),
        priorities=frozenset(d.get("priorities") or []),
        labels=tuple(d.get("labels") or []),
        search=d.get("search") or "",
        ready_only=bool(d.get("ready_only")),
        tangled_only=bool(d.get("tangled_only")),
        parent=d.get("parent") or "",
    )


def _view_to_dict(v: View) -> dict:
    return {
        "key": v.key,
        "name": v.name,
        "status": v.status,
        "builtin": v.builtin,
        "pinned": v.pinned,
        "spec": _spec_to_dict(v.spec),
        "sort_by": v.sort_by,
        "sort_dir": v.sort_dir,
        "limit": v.limit,
    }


def _view_from_dict(d: dict) -> View:
    return View(
        name=d["name"],
        key=d.get("key", ""),
        status=d.get("status"),
        builtin=bool(d.get("builtin")),
        pinned=bool(d.get("pinned", True)),
        spec=_spec_from_dict(d.get("spec")),
        sort_by=d.get("sort_by"),
        sort_dir=d.get("sort_dir") or "desc",
        limit=d.get("limit"),
    )


# -- reconcile / load / save ---------------------------------------------------

def reconcile(entries: list[dict], defaults: list[View]) -> list[View]:
    """Merge a stored overlay with the code-defined default Views.

    Order follows the overlay; built-in structure always comes from code (only
    name/pinned are overlaid); built-ins the overlay omits are appended in their
    default order; custom Views are rebuilt from their stored definition. A
    stored entry claiming to be a built-in whose key we no longer ship is
    dropped (not resurrected as a custom View).
    """
    by_key = {v.key: v for v in defaults}
    out: list[View] = []
    seen: set[str] = set()
    for e in entries:
        if not isinstance(e, dict):
            continue
        key = e.get("key")
        if not key or key in seen:
            continue
        seen.add(key)
        base = by_key.get(key)
        if base is not None:  # built-in: canonical structure + overlaid pin.
            # Name follows the code default unless the user actually renamed it
            # (un-renamed built-ins store a null name), so code-side label/emoji
            # changes reach already-customized herds, not just fresh installs.
            stored_name = e.get("name")
            out.append(replace(
                base,
                name=base.name if stored_name is None else stored_name,
                pinned=bool(e.get("pinned", base.pinned)),
            ))
        elif not e.get("builtin"):  # custom View
            out.append(_view_from_dict(e))
        # else: a built-in key we no longer ship -> drop it
    for v in defaults:  # ensure every built-in is present (new ones appended)
        if v.key not in seen:
            out.append(v)
    return out


def load_views(root: Path) -> list[View]:
    """The View list for *root*: the stored overlay reconciled with the current
    built-ins. Missing/corrupt/old files fall back to defaults. Never raises."""
    try:
        data = json.loads(views_path(root).read_text())
    except (OSError, json.JSONDecodeError, ValueError):
        data = None
    defaults = default_views()
    if (isinstance(data, dict) and data.get("v") == _VERSION
            and isinstance(data.get("views"), list)):
        return reconcile(data["views"], defaults)
    return defaults


def save_views(root: Path, views: list[View]) -> None:
    """Persist the View list (order + pins + custom defs) atomically. An
    un-renamed built-in stores a null name so it tracks the code default."""
    defaults = {v.key: v for v in default_views()}
    entries = []
    for v in views:
        d = _view_to_dict(v)
        base = defaults.get(v.key)
        if base is not None and v.builtin and d.get("name") == base.name:
            d["name"] = None
        entries.append(d)
    payload = {"v": _VERSION, "views": entries}
    path = views_path(root)
    path.parent.mkdir(parents=True, exist_ok=True)
    atomic_write(path, json.dumps(payload, indent=2))


# -- pure list mutations (used by the picker; unit-tested) ---------------------

def move(views: list[View], i: int, direction: int) -> int:
    """Swap view *i* with its neighbor in *direction* (-1 up / +1 down).
    Returns the view's new index (unchanged at the ends)."""
    j = i + direction
    if 0 <= i < len(views) and 0 <= j < len(views):
        views[i], views[j] = views[j], views[i]
        return j
    return i


def can_unpin(views: list[View], i: int) -> bool:
    """A view may be unpinned unless it is the last pinned one (the tab bar must
    keep at least one tab)."""
    if not views[i].pinned:
        return True
    return sum(1 for v in views if v.pinned) > 1


# -- working set: an ordered list of starred yak ids (yak-597c) -----------------
#
# Durable, user-specific, and ORDERED, so it is a plain id list in config rather
# than labels (which are unordered) or a frontmatter flag (which would bump
# `updated` and pollute the Recent view). Backs the built-in Working set View.

def working_set_path(root: Path) -> Path:
    return config_dir(root) / "working_set.json"


def load_working_set(root: Path) -> list[str]:
    """The ordered starred-id list for *root*; [] when absent/corrupt. Never raises."""
    try:
        data = json.loads(working_set_path(root).read_text())
    except (OSError, json.JSONDecodeError, ValueError):
        return []
    if isinstance(data, dict) and isinstance(data.get("ids"), list):
        return [str(x) for x in data["ids"]]
    return []


def save_working_set(root: Path, ids: list[str]) -> None:
    payload = {"v": _VERSION, "ids": list(ids)}
    path = working_set_path(root)
    path.parent.mkdir(parents=True, exist_ok=True)
    atomic_write(path, json.dumps(payload, indent=2))


def toggle_working_set(ids: list[str], tid: str) -> list[str]:
    """Return a new list with *tid* removed if present, else appended to the end
    (new stars go to the bottom, preserving existing order)."""
    if tid in ids:
        return [i for i in ids if i != tid]
    return [*ids, tid]
