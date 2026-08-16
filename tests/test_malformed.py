"""Malformed frontmatter must never crash or vanish (yak-cae6).

An unescaped colon (or any strict-YAML breakage) previously raised
ScannerError out of load_task, crashing scans and point reads. load_task now
recovers best-effort, keeps the yak visible, and flags it.
"""

from __future__ import annotations

from pathlib import Path

from yaklib.model import (
    all_tasks,
    find_children,
    find_task_file,
    load_task,
    save_task,
)

# The exact shape from the bug report: a second, unescaped colon in the title.
BAD = ("---\n"
       "id: yak-0002\n"
       "title: fix the s-matched entities; add messages: broken\n"
       "parent: yak-0001\n"
       "priority: 2\n"
       "---\n"
       "body text\n")


def _mk_root(tmp_path: Path) -> Path:
    root = tmp_path / ".yaks"
    for s in ("hairy", "shaving", "shorn", "dead"):
        (root / s).mkdir(parents=True)
    return root


def test_load_task_recovers_instead_of_raising(tmp_path):
    root = _mk_root(tmp_path)
    p = root / "hairy" / "yak-0002.md"
    p.write_text(BAD)

    task = load_task(p)  # must not raise
    # The whole string is recovered as the title (YAML only choked on the colon).
    assert task["title"] == "fix the s-matched entities; add messages: broken"
    assert task["id"] == "yak-0002"
    assert task["parent"] == "yak-0001"
    assert task["priority"] == 2 and isinstance(task["priority"], int)
    assert task["description"] == "body text"
    assert task["_error"]  # flagged for the UI


def test_malformed_stays_visible_in_scans(tmp_path):
    root = _mk_root(tmp_path)
    save_task(root / "hairy" / "yak-0001.md",
              {"id": "yak-0001", "title": "parent", "type": "task", "priority": 2})
    (root / "hairy" / "yak-0002.md").write_text(BAD)

    ids = {t["id"] for _, t in all_tasks(root, "hairy")}
    assert ids == {"yak-0001", "yak-0002"}  # bad file no longer silently dropped
    kids = {t["id"] for _, t in find_children(root, "yak-0001")}
    assert kids == {"yak-0002"}  # still attached to its parent


def test_point_read_does_not_crash(tmp_path):
    root = _mk_root(tmp_path)
    (root / "hairy" / "yak-0002.md").write_text(BAD)
    status, path = find_task_file(root, "yak-0002")
    assert status == "hairy"
    assert load_task(path)["id"] == "yak-0002"  # must not raise


def test_id_falls_back_to_filename_when_unparseable(tmp_path):
    root = _mk_root(tmp_path)
    # Even the id line is wrecked; the filename is authoritative.
    p = root / "hairy" / "yak-9999.md"
    p.write_text("---\nid: broken: bad\ntitle: whatever: nope\n---\n")
    task = load_task(p)
    assert task["id"] == "yak-9999"
    assert task["_error"]


def test_save_task_strips_error_flag(tmp_path):
    root = _mk_root(tmp_path)
    p = root / "hairy" / "yak-0002.md"
    p.write_text(BAD)
    task = load_task(p)
    assert task["_error"]

    # Fixing the title and saving must not persist the private _error flag.
    task["title"] = "fixed title"
    save_task(p, task)
    text = p.read_text()
    assert "_error" not in text
    reloaded = load_task(p)
    assert reloaded["title"] == "fixed title"
    assert "_error" not in reloaded  # clean parse, no flag


def test_cli_show_and_list_survive_malformed(yak, yak_root):
    (yak_root / ".yaks" / "hairy" / "test-bad.md").write_text(BAD.replace(
        "id: yak-0002", "id: test-bad").replace("parent: yak-0001\n", ""))
    # Neither command may crash (check=True raises on non-zero exit).
    assert yak("list").returncode == 0
    out = yak("show", "test-bad").stdout
    assert "add messages: broken" in out
