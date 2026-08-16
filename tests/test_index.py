"""Persistent stat-validated index (yak-3fd4.2).

The index is a pure cache: it must be indistinguishable from the direct scan,
self-heal from any corruption, and reconcile external edits/deletes/moves via
stat alone — reparsing only what actually changed.
"""

from __future__ import annotations

import os
from pathlib import Path

from yaklib import model
from yaklib.index import Index
from yaklib.model import (
    _all_tasks_direct,
    all_tasks,
    cache_dir,
    save_task,
)


def _mk_root(tmp_path: Path) -> Path:
    root = tmp_path / ".yaks"
    for s in ("hairy", "shaving", "shorn", "dead"):
        (root / s).mkdir(parents=True)
    return root


def _put(root: Path, status: str, tid: str, **fields) -> Path:
    task = {"id": tid, "title": fields.pop("title", tid), "type": "task",
            "priority": 3, "created": "2026-08-16T00:00:00Z",
            "updated": "2026-08-16T00:00:00Z", **fields}
    p = root / status / f"{tid}.md"
    save_task(p, task)
    return p


def _seed(root: Path):
    _put(root, "hairy", "yak-0001", labels=["perf"])
    _put(root, "hairy", "yak-0002", depends_on=["yak-0003"])
    _put(root, "shaving", "yak-0003", title="in progress")
    _put(root, "shorn", "yak-0004", description="finished\nwith a body")
    _put(root, "dead", "yak-0005", title="slaughtered")


# --- parity with the direct scan ---------------------------------------------

def test_index_matches_direct_scan(tmp_path):
    root = _mk_root(tmp_path)
    _seed(root)
    for status in (None, "hairy", "shaving", "shorn", "dead"):
        assert all_tasks(root, status) == _all_tasks_direct(root, status), status


def test_description_is_indexed_for_search(tmp_path):
    root = _mk_root(tmp_path)
    _put(root, "shorn", "yak-0004", description="finished\nwith a body")
    (_, task), = all_tasks(root, "shorn")
    assert task["description"] == "finished\nwith a body"


def test_no_index_env_uses_direct(tmp_path, monkeypatch):
    root = _mk_root(tmp_path)
    _seed(root)
    monkeypatch.setenv("YAKS_NO_INDEX", "1")
    assert all_tasks(root) == _all_tasks_direct(root)
    # And with the env set, no index file is written.
    assert not (cache_dir(root) / "index.json").exists()


# --- the cache file itself ----------------------------------------------------

def test_index_file_written_under_cache_dir(tmp_path):
    root = _mk_root(tmp_path)
    _seed(root)
    all_tasks(root)  # materialize + persist
    assert (cache_dir(root) / "index.json").exists()


def test_self_heals_from_corrupt_index(tmp_path):
    root = _mk_root(tmp_path)
    _seed(root)
    idx_path = cache_dir(root) / "index.json"
    Index(root, idx_path).load().sync()  # write a good index
    idx_path.write_text("}{ not json at all")
    fresh = Index(root, idx_path).load()  # tolerates the garbage
    fresh.sync()                          # rebuilds from the files
    ids = {t["id"] for _, t in fresh.tasks()}
    assert ids == {"yak-0001", "yak-0002", "yak-0003", "yak-0004"}


def test_missing_index_rebuilds(tmp_path):
    root = _mk_root(tmp_path)
    _seed(root)
    idx = Index(root, cache_dir(root) / "index.json").load()  # no file yet
    idx.sync()
    assert {t["id"] for _, t in idx.tasks("hairy")} == {"yak-0001", "yak-0002"}


# --- reconcile: a fresh instance simulates a new process reading a stale index

def test_reconcile_picks_up_edit(tmp_path):
    root = _mk_root(tmp_path)
    _put(root, "hairy", "yak-0001", title="before")
    Index(root, cache_dir(root) / "index.json").load().sync()

    _put(root, "hairy", "yak-0001", title="after")  # external edit (new mtime)
    idx2 = Index(root, cache_dir(root) / "index.json").load()
    idx2.sync()
    (_, task), = idx2.tasks("hairy")
    assert task["title"] == "after"


def test_reconcile_picks_up_delete(tmp_path):
    root = _mk_root(tmp_path)
    _put(root, "hairy", "yak-0001")
    _put(root, "hairy", "yak-0002")
    Index(root, cache_dir(root) / "index.json").load().sync()

    (root / "hairy" / "yak-0001.md").unlink()  # external delete
    idx2 = Index(root, cache_dir(root) / "index.json").load()
    idx2.sync()
    assert {t["id"] for _, t in idx2.tasks("hairy")} == {"yak-0002"}


def test_pure_status_rename_reuses_record_without_reparse(tmp_path, monkeypatch):
    root = _mk_root(tmp_path)
    src = _put(root, "hairy", "yak-0001", title="movable")
    idx_path = cache_dir(root) / "index.json"
    Index(root, idx_path).load().sync()  # index now knows yak-0001 @ hairy

    # A pure status change is a rename that preserves mtime + size.
    dst = root / "shorn" / "yak-0001.md"
    os.rename(src, dst)

    calls = []
    real = model.load_task
    monkeypatch.setattr(model, "load_task", lambda p: calls.append(p) or real(p))

    idx2 = Index(root, idx_path).load()
    idx2.sync()
    assert calls == []  # status change detected by stat alone, no reparse
    (status, task), = idx2.tasks("shorn")
    assert status == "shorn" and task["title"] == "movable"
    assert idx2.tasks("hairy") == []


def test_racy_file_is_reparsed(tmp_path, monkeypatch):
    """A file whose mtime is at/after the index build time can't be trusted by
    stat, so it must be reparsed (git's racy-clean rule)."""
    root = _mk_root(tmp_path)
    p = _put(root, "hairy", "yak-0001", title="v1")
    idx = Index(root, cache_dir(root) / "index.json").load()
    idx.sync()

    # Rewrite content but forge mtime/size back to the recorded values so only
    # the racy rule (mtime >= built_ns) can catch the change.
    rec = idx.records["yak-0001"]
    p.write_text(p.read_text().replace("v1", "v2"))
    os.utime(p, ns=(idx.built_ns, idx.built_ns))  # mtime == build time => racy
    # keep size identical (v1/v2 are the same length)
    assert p.stat().st_size == rec["size"]

    idx.mark_stale()
    idx.sync()
    (_, task), = idx.tasks("hairy")
    assert task["title"] == "v2"
