"""Shared fixtures for yaks tests.

The CLI tests shell out to `yak.py` against a temporary `.yaks/` root.
A `yak()` helper wraps subprocess invocation with sensible defaults:
cwd set to the tmp root, stdout captured, non-zero exits raising.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parent.parent
YAK_SCRIPT = REPO_ROOT / "scripts" / "yak.py"


@dataclass
class YakResult:
    stdout: str
    stderr: str
    returncode: int

    def json(self):
        return json.loads(self.stdout)


@pytest.fixture(autouse=True)
def _isolate_cache(tmp_path: Path, monkeypatch):
    """Point the per-user cache (index + UI state) at a temp dir so tests never
    read or pollute the real ~/.cache/yaks, and stay isolated from each other.
    Inherited by the CLI subprocess runner via os.environ."""
    monkeypatch.setenv("XDG_CACHE_HOME", str(tmp_path / ".cache"))


@pytest.fixture
def yak_root(tmp_path: Path):
    """A temp dir with an initialized .yaks/ using prefix 'test'."""
    run = _make_runner(tmp_path)
    run("init", "--prefix", "test")
    return tmp_path


@pytest.fixture
def yak(yak_root: Path):
    """A callable that invokes yak.py inside the temp root."""
    return _make_runner(yak_root)


def _make_runner(cwd: Path):
    def run(*args: str, check: bool = True, stdin: str | None = None) -> YakResult:
        result = subprocess.run(
            [sys.executable, str(YAK_SCRIPT), *args],
            cwd=cwd,
            capture_output=True,
            text=True,
            input=stdin,
            env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"},
        )
        if check and result.returncode != 0:
            raise AssertionError(
                f"yak {' '.join(args)} exited {result.returncode}\n"
                f"stdout: {result.stdout}\nstderr: {result.stderr}"
            )
        return YakResult(result.stdout, result.stderr, result.returncode)

    return run


def create_task(yak, title: str, **kwargs) -> str:
    """Helper: create a task, return its ID (parsed from stdout)."""
    args = ["create", "--title", title]
    for k, v in kwargs.items():
        flag = "--" + k.replace("_", "-")
        if isinstance(v, list):
            args += [flag, *v]
        elif isinstance(v, bool):
            if v:
                args.append(flag)
        else:
            args += [flag, str(v)]
    out = yak(*args).stdout
    # "Created {id}: {title}"
    line = out.splitlines()[0]
    assert line.startswith("Created "), out
    return line.split()[1].rstrip(":")
