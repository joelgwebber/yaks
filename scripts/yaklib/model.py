"""Core model: status constants, YAML I/O, filesystem layout, task loading."""

from __future__ import annotations

import random
import string
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

import yaml

# ---------------------------------------------------------------------------
# Status constants
# ---------------------------------------------------------------------------

HAIRY = "hairy"
SHAVING = "shaving"
SHORN = "shorn"
DEAD = "dead"

# Normal statuses visible to the UI and default queries. "dead" is hidden;
# slaughtered yaks exist on disk but are excluded unless explicitly requested.
STATUSES = (HAIRY, SHAVING, SHORN)
_ALL_STATUSES = (HAIRY, SHAVING, SHORN, DEAD)

_STATUS_ALIASES = {
    "open": HAIRY, "working": SHAVING, "closed": SHORN,
    "slaughtered": DEAD,
    HAIRY: HAIRY, SHAVING: SHAVING, SHORN: SHORN, DEAD: DEAD,
}


def resolve_status(name: str) -> str:
    return _STATUS_ALIASES.get(name, name)


ALL_STATUS_NAMES = sorted(_STATUS_ALIASES.keys())


# ---------------------------------------------------------------------------
# YAML helpers
# ---------------------------------------------------------------------------

class _BlockScalarDumper(yaml.SafeDumper):
    """Dumper that uses block scalars for multiline strings."""


def _str_representer(dumper: _BlockScalarDumper, data: str):
    if "\n" in data:
        return dumper.represent_scalar("tag:yaml.org,2002:str", data, style="|")
    return dumper.represent_scalar("tag:yaml.org,2002:str", data)


_BlockScalarDumper.add_representer(str, _str_representer)


def dump_yaml(data: dict) -> str:
    return yaml.dump(data, Dumper=_BlockScalarDumper, default_flow_style=False,
                     sort_keys=False, allow_unicode=True)


# ---------------------------------------------------------------------------
# Filesystem layout
# ---------------------------------------------------------------------------

def find_tasks_root(start: Path | None = None) -> Path:
    """Walk up from *start* (default cwd) looking for a `.yaks/` directory."""
    p = (start or Path.cwd()).resolve()
    while True:
        candidate = p / ".yaks"
        if candidate.is_dir():
            _auto_migrate(candidate)
            return candidate
        if p.parent == p:
            break
        p = p.parent
    print("error: no .yaks/ directory found (run /yaks:init first)", file=sys.stderr)
    sys.exit(1)


def _auto_migrate(root: Path) -> None:
    """Migrate legacy .yaml task files to .md with frontmatter."""
    migrated = []
    for s in STATUSES:
        d = root / s
        if not d.exists():
            continue
        for f in sorted(d.glob("*.yaml")):
            task = yaml.safe_load(f.read_text()) or {}
            if not task:
                continue
            md_path = f.with_suffix(".md")
            description = task.pop("description", None)
            fm = dump_yaml(task)
            parts = ["---\n", fm, "---\n"]
            if description:
                parts.append("\n")
                parts.append(description)
                if not description.endswith("\n"):
                    parts.append("\n")
            md_path.write_text("".join(parts))
            f.unlink()
            migrated.append(f"{s}/{f.stem}")
    if migrated:
        print(f"Migrated {len(migrated)} task(s) from .yaml to .md:")
        for name in migrated:
            print(f"  {name}")


def load_config(root: Path) -> dict:
    cfg_path = root / "config.yaml"
    if cfg_path.exists():
        return yaml.safe_load(cfg_path.read_text()) or {}
    return {}


# ---------------------------------------------------------------------------
# Task I/O
# ---------------------------------------------------------------------------

def load_task(path: Path) -> dict:
    text = path.read_text()
    if path.suffix == ".md":
        if not text.startswith("---"):
            return {}
        end = text.find("\n---", 3)
        if end < 0:
            return {}
        fm = text[4:end]
        body = text[end + 4:]
        task = yaml.safe_load(fm) or {}
        body = body.strip()
        if body:
            task["description"] = body
        return task
    return yaml.safe_load(text) or {}


def save_task(path: Path, task: dict) -> None:
    task = dict(task)
    description = task.pop("description", None)
    fm = dump_yaml(task)
    parts = ["---\n", fm, "---\n"]
    if description:
        parts.append("\n")
        parts.append(description)
        if not description.endswith("\n"):
            parts.append("\n")
    path.write_text("".join(parts))


def all_tasks(root: Path, status: str | None = None) -> list[tuple[str, dict]]:
    """Return (status, task_dict) for tasks in the given status dir(s).

    When status is None, only the visible STATUSES are scanned — dead yaks
    are excluded. Pass status=DEAD to inspect slaughtered tasks.
    """
    scan = (status,) if status is not None else STATUSES
    results = []
    for s in scan:
        d = root / s
        if not d.exists():
            continue
        for f in sorted(d.glob("*.md")):
            task = load_task(f)
            if task:
                results.append((s, task))
    return results


def find_task_file(root: Path, task_id: str) -> tuple[str, Path] | None:
    """Locate a task file by ID across all status dirs (including dead)."""
    for status_dir in _ALL_STATUSES:
        p = root / status_dir / f"{task_id}.md"
        if p.exists():
            return status_dir, p
    return None


def generate_id(root: Path, prefix: str) -> str:
    """Generate a collision-free task ID against all dirs (including dead)."""
    existing = set()
    for d in (root / s for s in _ALL_STATUSES):
        if d.exists():
            for f in d.glob("*.md"):
                existing.add(f.stem)
    for _ in range(100):
        suffix = "".join(random.choices(string.hexdigits[:16], k=4))
        tid = f"{prefix}-{suffix}"
        if tid not in existing:
            return tid
    print("error: could not generate unique ID after 100 attempts", file=sys.stderr)
    sys.exit(1)


def now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


# ---------------------------------------------------------------------------
# Parent / child ID arithmetic
# ---------------------------------------------------------------------------

def parent_id(task_id: str) -> str | None:
    """Return the parent ID if task_id is a child (e.g. 'foo-abc.2' → 'foo-abc')."""
    dot = task_id.rfind(".")
    if dot < 0:
        return None
    suffix = task_id[dot + 1:]
    if suffix.isdigit():
        return task_id[:dot]
    return None


def find_children(root: Path, task_id: str) -> list[tuple[str, dict]]:
    """Return (status, task) for all direct children, sorted by child number."""
    prefix = task_id + "."
    children = []
    for s in STATUSES:
        d = root / s
        if not d.exists():
            continue
        for f in d.glob(f"{prefix}*.md"):
            suffix = f.stem[len(prefix):]
            if suffix.isdigit():
                task = load_task(f)
                if task:
                    children.append((s, task, int(suffix)))
    children.sort(key=lambda x: x[2])
    return [(s, t) for s, t, _ in children]


def next_child_number(root: Path, task_id: str) -> int:
    """Return the next available child number for task_id."""
    prefix = task_id + "."
    max_n = 0
    for s in STATUSES:
        d = root / s
        if not d.exists():
            continue
        for f in d.glob(f"{prefix}*.md"):
            suffix = f.stem[len(prefix):]
            if suffix.isdigit():
                max_n = max(max_n, int(suffix))
    return max_n + 1


def find_descendants(root: Path, task_id: str) -> list[tuple[str, Path]]:
    """Return (status, path) for all descendants at any depth.

    Includes dead descendants so reparenting updates them too.
    """
    prefix = task_id + "."
    descendants = []
    for s in _ALL_STATUSES:
        d = root / s
        if not d.exists():
            continue
        for f in d.glob(f"{prefix}*.md"):
            descendants.append((s, f))
    return descendants


# ---------------------------------------------------------------------------
# Git integration
# ---------------------------------------------------------------------------

def git_head_short() -> str | None:
    """Return the short hash of HEAD, or None if not in a git repo."""
    try:
        result = subprocess.run(
            ["git", "rev-parse", "--short", "HEAD"],
            capture_output=True, text=True, timeout=5,
        )
        if result.returncode == 0:
            return result.stdout.strip()
    except (FileNotFoundError, subprocess.TimeoutExpired):
        pass
    return None
