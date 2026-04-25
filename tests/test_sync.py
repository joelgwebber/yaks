"""Pending-sync sidecar tests — IO + CLI bookkeeping."""

from __future__ import annotations

from pathlib import Path

import yaml

from conftest import create_task
from yaklib.sync import (
    clear_sidecar,
    has_pending,
    list_pending,
    load_sidecar,
    pending_root,
    save_sidecar,
    sidecar_path,
)


def _yaks(root: Path) -> Path:
    return root / ".yaks"


def _write_sidecar(root: Path, yak_id: str, data: dict) -> Path:
    p = sidecar_path(_yaks(root), yak_id)
    save_sidecar(p, data)
    return p


def test_pending_root_is_under_dot_yaks(yak, yak_root):
    """Sidecars live in .yaks/.sync-pending — invisible to find_tasks_root
    and the normal status walks."""
    assert pending_root(_yaks(yak_root)).name == ".sync-pending"


def test_save_load_sidecar_roundtrip(yak, yak_root):
    tid = create_task(yak, "round trip", type="task")
    data = {"yak_id": tid, "fields": [{"name": "title", "resolution": "pending"}]}
    p = _write_sidecar(yak_root, tid, data)

    loaded = load_sidecar(p)
    assert loaded == data


def test_list_pending_empty_returns_empty(yak, yak_root):
    assert list_pending(_yaks(yak_root)) == []


def test_list_pending_returns_sorted_ids(yak, yak_root):
    a = create_task(yak, "alpha")
    b = create_task(yak, "beta")
    _write_sidecar(yak_root, a, {"yak_id": a})
    _write_sidecar(yak_root, b, {"yak_id": b})

    assert list_pending(_yaks(yak_root)) == sorted([a, b])


def test_has_pending_reflects_file_existence(yak, yak_root):
    tid = create_task(yak, "present")
    assert not has_pending(_yaks(yak_root), tid)
    _write_sidecar(yak_root, tid, {"yak_id": tid})
    assert has_pending(_yaks(yak_root), tid)


def test_clear_sidecar_returns_true_when_present(yak, yak_root):
    tid = create_task(yak, "to clear")
    _write_sidecar(yak_root, tid, {"yak_id": tid})

    assert clear_sidecar(_yaks(yak_root), tid) is True
    assert not has_pending(_yaks(yak_root), tid)


def test_clear_sidecar_returns_false_when_absent(yak, yak_root):
    assert clear_sidecar(_yaks(yak_root), "nope-1234") is False


def test_cli_sync_ls_empty(yak):
    out = yak("sync", "ls").stdout
    assert "No pending sync sidecars" in out


def test_cli_sync_ls_lists_ids(yak, yak_root):
    a = create_task(yak, "alpha")
    _write_sidecar(yak_root, a, {"yak_id": a})

    out = yak("sync", "ls").stdout.strip().splitlines()
    assert a in out


def test_cli_sync_show_prints_yaml(yak, yak_root):
    tid = create_task(yak, "show me")
    data = {"yak_id": tid, "source": "https://example/PROJ-1"}
    _write_sidecar(yak_root, tid, data)

    out = yak("sync", "show", tid).stdout
    assert tid in out
    assert "https://example/PROJ-1" in out
    # Round-trips through YAML
    parsed = yaml.safe_load(out)
    assert parsed == data


def test_cli_sync_show_missing_errors(yak):
    res = yak("sync", "show", "nope-1234", check=False)
    assert res.returncode == 1
    assert "no sidecar" in res.stderr.lower()


def test_cli_sync_clear_removes_file(yak, yak_root):
    tid = create_task(yak, "clear me")
    _write_sidecar(yak_root, tid, {"yak_id": tid})

    out = yak("sync", "clear", tid).stdout
    assert tid in out
    assert not has_pending(_yaks(yak_root), tid)


def test_cli_sync_clear_when_absent_is_a_noop(yak):
    out = yak("sync", "clear", "nope-1234").stdout
    assert "No pending sync" in out


def test_cli_list_marks_pending_yaks_with_tilde(yak, yak_root):
    """The list view shows '~' before the ID for yaks with a pending sidecar
    (and ' ' otherwise, to keep columns aligned)."""
    a = create_task(yak, "no sidecar")
    b = create_task(yak, "with sidecar")
    _write_sidecar(yak_root, b, {"yak_id": b})

    out = yak("list").stdout
    a_line = next(ln for ln in out.splitlines() if a in ln)
    b_line = next(ln for ln in out.splitlines() if b in ln)
    assert f"~{b}" in b_line
    assert f"~{a}" not in a_line


def test_pending_dir_does_not_pollute_status_walks(yak, yak_root):
    """Sidecars live alongside hairy/shaving/shorn but must not appear in
    `yak list` as tasks."""
    tid = create_task(yak, "real task")
    _write_sidecar(yak_root, tid, {"yak_id": tid})

    out = yak("list", "--json").json()
    ids = [t["id"] for t in out]
    # Only the real task — no phantom from the sidecar.
    assert ids == [tid]
