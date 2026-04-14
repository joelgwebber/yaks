"""Golden CLI tests: artifact attach / detach."""

from __future__ import annotations

from pathlib import Path

from conftest import create_task


def test_attach_copies_file_and_appends_link(yak, yak_root, tmp_path):
    tid = create_task(yak, "with file", type="task")
    src = tmp_path / "shot.png"
    src.write_bytes(b"fake-png")

    yak("attach", tid, str(src), "--desc", "a shot")

    dest = yak_root / ".yaks" / "artifacts" / tid / "shot.png"
    assert dest.is_file()
    assert dest.read_bytes() == b"fake-png"

    body = yak("show", tid, "--json").json()["description"]
    assert f"![a shot](artifacts/{tid}/shot.png)" in body


def test_attach_rejects_overwrite_without_force(yak, yak_root, tmp_path):
    tid = create_task(yak, "dupe", type="task")
    src = tmp_path / "shot.png"
    src.write_bytes(b"v1")
    yak("attach", tid, str(src))

    src.write_bytes(b"v2")
    result = yak("attach", tid, str(src), check=False)
    assert result.returncode != 0
    assert "exists" in result.stderr

    dest = yak_root / ".yaks" / "artifacts" / tid / "shot.png"
    assert dest.read_bytes() == b"v1"

    yak("attach", tid, str(src), "--force")
    assert dest.read_bytes() == b"v2"


def test_attach_custom_name(yak, yak_root, tmp_path):
    tid = create_task(yak, "named", type="task")
    src = tmp_path / "ugly-name.png"
    src.write_bytes(b"png")

    yak("attach", tid, str(src), "--name", "nice.png")
    dest = yak_root / ".yaks" / "artifacts" / tid / "nice.png"
    assert dest.is_file()

    body = yak("show", tid, "--json").json()["description"]
    assert f"artifacts/{tid}/nice.png" in body


def test_detach_removes_file_and_link(yak, yak_root, tmp_path):
    tid = create_task(yak, "removable", type="task")
    src = tmp_path / "shot.png"
    src.write_bytes(b"x")
    yak("attach", tid, str(src))

    yak("detach", tid, "shot.png")

    dest = yak_root / ".yaks" / "artifacts" / tid / "shot.png"
    assert not dest.exists()

    body = yak("show", tid, "--json").json().get("description") or ""
    assert "shot.png" not in body


def test_attach_missing_file_errors_cleanly(yak, tmp_path):
    tid = create_task(yak, "oops", type="task")
    result = yak("attach", tid, str(tmp_path / "nope.png"), check=False)
    assert result.returncode != 0
    assert "not a file" in result.stderr
