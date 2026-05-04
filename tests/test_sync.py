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


# ---------------------------------------------------------------------------
# Sweep / drift-check helpers (bf54.2)
# ---------------------------------------------------------------------------


def test_tracker_for_classifies_known_hosts():
    from yaklib.sync import tracker_for
    assert tracker_for("https://fullstory.atlassian.net/browse/PROJ-1") == "jira"
    assert tracker_for("https://linear.app/team/issue/SUB-42/foo") == "linear"
    assert tracker_for("https://github.com/owner/repo/issues/7") == "github"
    assert tracker_for("https://example.com/issue/1") == "other"
    assert tracker_for("") == "other"
    assert tracker_for(None) == "other"


def test_upstream_key_for_extracts_jira_key():
    from yaklib.sync import upstream_key_for
    assert upstream_key_for("https://fullstory.atlassian.net/browse/SUBTEXT-301") == "SUBTEXT-301"
    # No match if the path doesn't include /browse/.
    assert upstream_key_for("https://fullstory.atlassian.net/projects/SUBTEXT") is None


def test_upstream_key_for_extracts_linear_key():
    from yaklib.sync import upstream_key_for
    assert upstream_key_for("https://linear.app/team/issue/SUB-42/foo") == "SUB-42"


def test_upstream_key_for_extracts_github_owner_repo_num():
    from yaklib.sync import upstream_key_for
    assert upstream_key_for("https://github.com/owner/repo/issues/7") == "owner/repo#7"


def test_iter_synced_yaks_skips_yaks_without_source(yak, yak_root):
    """Sweep only cares about yaks that link to something."""
    from yaklib.sync import iter_synced_yaks
    create_task(yak, "no source", type="task")
    yak("create", "--title", "linked", "--type", "task",
        "--source", "https://fullstory.atlassian.net/browse/PROJ-1")
    rows = iter_synced_yaks(yak_root / ".yaks")
    assert len(rows) == 1
    assert rows[0]["tracker"] == "jira"
    assert rows[0]["key"] == "PROJ-1"
    assert rows[0]["source"] == "https://fullstory.atlassian.net/browse/PROJ-1"


def test_iter_synced_yaks_flags_pending_sidecar(yak, yak_root):
    from yaklib.sync import iter_synced_yaks
    out = yak("create", "--title", "linked", "--type", "task",
              "--source", "https://fullstory.atlassian.net/browse/PROJ-2").stdout
    tid = out.splitlines()[0].split()[1].rstrip(":")
    _write_sidecar(yak_root, tid, {"yak_id": tid})

    rows = iter_synced_yaks(yak_root / ".yaks")
    assert len(rows) == 1
    assert rows[0]["has_pending"] is True


def test_cli_sync_check_json(yak):
    yak("create", "--title", "tracked", "--type", "task",
        "--source", "https://fullstory.atlassian.net/browse/PROJ-9")
    rows = yak("sync", "check", "--json").json()
    assert len(rows) == 1
    assert rows[0]["tracker"] == "jira"
    assert rows[0]["key"] == "PROJ-9"


def test_cli_sync_check_filters_by_tracker(yak):
    yak("create", "--title", "j", "--type", "task",
        "--source", "https://fullstory.atlassian.net/browse/J-1")
    yak("create", "--title", "g", "--type", "task",
        "--source", "https://github.com/o/r/issues/1")
    rows = yak("sync", "check", "--json", "--tracker", "jira").json()
    keys = [r["key"] for r in rows]
    assert keys == ["J-1"]


def test_cli_sync_check_empty_message_when_no_source_yaks(yak):
    create_task(yak, "no source", type="task")
    out = yak("sync", "check").stdout
    assert "No yaks with a `source:`" in out


def test_cli_sync_check_table_includes_local_drift_column(yak):
    """A yak whose updated > last_synced shows local_drift=yes."""
    out = yak("create", "--title", "drifted", "--type", "task",
              "--source", "https://fullstory.atlassian.net/browse/D-1").stdout
    tid = out.splitlines()[0].split()[1].rstrip(":")
    yak("update", tid, "--last-synced", "2020-01-01T00:00:00Z")
    yak("update", tid, "--note", "force updated bump after the stamp")

    table = yak("sync", "check").stdout
    line = next(ln for ln in table.splitlines() if tid in ln)
    assert line.endswith("yes")


# ---------------------------------------------------------------------------
# apply_sidecar_locally — TUI sidecar review (bf54.9)
# ---------------------------------------------------------------------------


def test_apply_sidecar_locally_overwrites_field_when_direction_upstream():
    from yaklib.sync import apply_sidecar_locally
    yak = {"id": "x-0001", "title": "old", "priority": 2}
    sidecar = {
        "fields": [
            {"name": "title", "local": "old", "upstream": "new",
             "direction": "upstream", "resolution": "approve"},
            {"name": "priority", "local": 2, "upstream": 1,
             "direction": "upstream", "resolution": "auto"},
        ],
    }
    res = apply_sidecar_locally(yak, sidecar)
    assert res.new_yak["title"] == "new"
    assert res.new_yak["priority"] == 1
    assert len(res.applied) == 2
    assert not res.deferred and not res.skipped
    assert res.bucket_remaining == 0


def test_apply_sidecar_locally_records_skipped_rows():
    from yaklib.sync import apply_sidecar_locally
    yak = {"id": "x-0001", "title": "old"}
    sidecar = {"fields": [
        {"name": "title", "local": "old", "upstream": "new",
         "direction": "upstream", "resolution": "skip"},
    ]}
    res = apply_sidecar_locally(yak, sidecar)
    assert res.new_yak == yak
    assert len(res.skipped) == 1
    assert not res.applied and not res.deferred


def test_apply_sidecar_locally_defers_pending_resolution():
    """Unresolved rows stay in the sidecar — TUI never silently applies them."""
    from yaklib.sync import apply_sidecar_locally
    sidecar = {"fields": [
        {"name": "title", "direction": "upstream", "resolution": "pending"},
    ]}
    res = apply_sidecar_locally({"title": "x"}, sidecar)
    assert not res.applied
    assert len(res.deferred) == 1


def test_apply_sidecar_locally_defers_local_direction():
    """direction: local needs an MCP write — defers to /yaks:sync."""
    from yaklib.sync import apply_sidecar_locally
    sidecar = {"fields": [
        {"name": "title", "direction": "local", "resolution": "approve"},
    ]}
    res = apply_sidecar_locally({"title": "x"}, sidecar)
    assert not res.applied
    assert len(res.deferred) == 1


def test_apply_sidecar_locally_defers_status_field():
    """status mapping is lossy — defer to /yaks:sync even when resolved."""
    from yaklib.sync import apply_sidecar_locally
    sidecar = {"fields": [
        {"name": "status", "direction": "upstream", "resolution": "approve",
         "local": "hairy", "upstream": "In Progress"},
    ]}
    res = apply_sidecar_locally({"id": "x-1"}, sidecar)
    assert not res.applied
    assert len(res.deferred) == 1


def test_apply_sidecar_locally_partial_apply_with_mixed_rows():
    """A mix of appliable + deferred + skipped rows: apply what's safe,
    keep the rest for /yaks:sync."""
    from yaklib.sync import apply_sidecar_locally
    yak = {"id": "x-1", "title": "old", "priority": 2}
    sidecar = {"fields": [
        # Applies locally.
        {"name": "title", "upstream": "new",
         "direction": "upstream", "resolution": "approve"},
        # Defers — needs MCP push.
        {"name": "priority", "upstream": 1,
         "direction": "local", "resolution": "approve"},
        # Skipped — drops out.
        {"name": "labels", "upstream": ["new"],
         "direction": "upstream", "resolution": "skip"},
    ]}
    res = apply_sidecar_locally(yak, sidecar)
    assert res.new_yak["title"] == "new"
    assert res.new_yak["priority"] == 2  # unchanged; deferred to skill
    assert len(res.applied) == 1
    assert len(res.deferred) == 1
    assert len(res.skipped) == 1


def test_apply_sidecar_locally_counts_bucket_remaining():
    """Buckets aren't applied, just counted — and skipped items don't count."""
    from yaklib.sync import apply_sidecar_locally
    sidecar = {
        "fields": [],
        "comments_down": [
            {"body": "ferry me", "resolution": "auto"},
            {"body": "skip me", "resolution": "skip"},
        ],
        "attachments_up": [
            {"filename": "a.png", "resolution": "pending"},
        ],
    }
    res = apply_sidecar_locally({"id": "x"}, sidecar)
    # 1 comment_down (auto) + 1 attachment_up (pending) = 2; skip drops out.
    assert res.bucket_remaining == 2


def test_apply_sidecar_locally_drops_field_when_upstream_empty():
    """Upstream cleared a label list / description → yak loses the field."""
    from yaklib.sync import apply_sidecar_locally
    yak = {"id": "x-1", "labels": ["a", "b"]}
    sidecar = {"fields": [
        {"name": "labels", "local": ["a", "b"], "upstream": [],
         "direction": "upstream", "resolution": "auto"},
    ]}
    res = apply_sidecar_locally(yak, sidecar)
    assert "labels" not in res.new_yak
    assert len(res.applied) == 1


def test_apply_sidecar_locally_uses_merged_value_when_set():
    """merged_value supersedes upstream — it's the user's reconciliation."""
    from yaklib.sync import apply_sidecar_locally
    yak = {"id": "x-1", "title": "old"}
    sidecar = {"fields": [
        {"name": "title", "local": "old", "upstream": "raw upstream",
         "merged_value": "merged result",
         "direction": "upstream", "resolution": "approve"},
    ]}
    res = apply_sidecar_locally(yak, sidecar)
    assert res.new_yak["title"] == "merged result"


def test_apply_sidecar_locally_merged_value_none_falls_through():
    """merged_value: null (the schema default) shouldn't suppress upstream."""
    from yaklib.sync import apply_sidecar_locally
    yak = {"id": "x-1", "title": "old"}
    sidecar = {"fields": [
        {"name": "title", "local": "old", "upstream": "new",
         "merged_value": None,
         "direction": "upstream", "resolution": "approve"},
    ]}
    res = apply_sidecar_locally(yak, sidecar)
    assert res.new_yak["title"] == "new"
