"""Core model: status constants, YAML I/O, filesystem layout, task loading."""

from __future__ import annotations

import os
import random
import re
import string
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path

import yaml

# Strict ISO8601 timestamp at column 0 inside an h3 heading. Matches the
# old yak comment-block format (`### <iso> [@author] [(from tracker:key)]`)
# and captures the iso + the rest-of-line for replacement. Only date+time
# with timezone matches; a bare `### 2026-04-25` stays untouched.
_LEGACY_COMMENT_RE = re.compile(
    r"^### (\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?"
    r"(?:Z|[+-]\d{2}:?\d{2})?)(.*)$",
    re.MULTILINE,
)

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
# Atomic writes + schema versioning
# ---------------------------------------------------------------------------

def atomic_write(path: Path, text: str) -> None:
    """Write *text* to *path* atomically: a temp file in the same directory is
    written, then os.replace()'d over the target. A crash can leave a stray
    .tmp file (harmless) but never a torn or half-written target. Keeping the
    temp in the same directory guarantees the rename stays on one filesystem.
    """
    path = Path(path)
    fd, tmp = tempfile.mkstemp(dir=str(path.parent), prefix=path.name + ".", suffix=".tmp")
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as fh:
            fh.write(text)
        os.replace(tmp, path)
    except BaseException:
        try:
            os.unlink(tmp)
        except OSError:
            pass
        raise


# Bump when a new on-disk migration step is added; each step's version must be
# <= this. v1: legacy .yaml -> .md. v2: `### <iso>` comment blocks -> sigil
# format. v3: backfill the `parent` field from legacy dotted IDs.
CURRENT_SCHEMA_VERSION = 3

_SCHEMA_FILE = "schema"


def read_schema_version(root: Path) -> int:
    """Herd schema version; 0 when unmarked, unreadable, or legacy."""
    try:
        return int((root / _SCHEMA_FILE).read_text().strip())
    except (OSError, ValueError):
        return 0


def write_schema_version(root: Path, version: int) -> None:
    atomic_write(root / _SCHEMA_FILE, f"{version}\n")


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
    print("error: no .yaks/ directory found (run 'yaks init' first)", file=sys.stderr)
    sys.exit(1)


def _auto_migrate(root: Path) -> None:
    """Version-gated on-disk migration, run once from find_tasks_root().

    Cheap no-op when the herd is already at CURRENT_SCHEMA_VERSION (a single
    small read of the schema file). Otherwise it runs the ordered, idempotent
    migration steps whose version is newer than the herd's, stamping the schema
    file after each step so an interrupted run resumes where it left off. An
    unmarked herd reads as version 0, so every step runs once (each is a no-op
    on already-current content) and the herd is then stamped.
    """
    current = read_schema_version(root)
    if current >= CURRENT_SCHEMA_VERSION:
        return
    for version, step in _MIGRATIONS:
        if version > current:
            step(root)
            write_schema_version(root, version)


def _migrate_comment_blocks(text: str) -> str:
    """Rewrite ``### <iso8601>`` comment headings to ``---\\n▸ <iso8601>``.

    Pure function (no IO). Only operates on the description body — the
    frontmatter never carries h3 headings, but we slice it off explicitly
    so the regex can't be fooled by an iso-shaped value buried in YAML.
    Idempotent: if no headings match, returns *text* unchanged.
    """
    if not text.startswith("---\n"):
        return text
    end = text.find("\n---\n", 4)
    if end < 0:
        return text
    head = text[:end + 5]
    body = text[end + 5:]

    def repl(m: re.Match) -> str:
        return f"---\n▸ {m.group(1)}{m.group(2)}"

    new_body = _LEGACY_COMMENT_RE.sub(repl, body)
    if new_body == body:
        return text
    # Collapse `---\n---\n` runs in case the user already had a thematic
    # break right above an old `### iso` heading.
    new_body = re.sub(r"(?m)^---\n---\n", "---\n", new_body)
    return head + new_body


def _migrate_v1_yaml_to_md(root: Path) -> None:
    """v1: convert legacy ``.yaml`` task files to ``.md`` with frontmatter."""
    migrated = []
    for s in _ALL_STATUSES:
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
            atomic_write(md_path, "".join(parts))
            f.unlink()
            migrated.append(f"{s}/{f.stem}")
    if migrated:
        print(f"Migrated {len(migrated)} task(s) from .yaml to .md:")
        for name in migrated:
            print(f"  {name}")


def _migrate_v2_comment_blocks(root: Path) -> None:
    """v2: rewrite legacy ``### <iso>`` comment headings to the sigil format."""
    migrated = []
    for s in _ALL_STATUSES:
        d = root / s
        if not d.exists():
            continue
        for f in sorted(d.glob("*.md")):
            text = f.read_text()
            new = _migrate_comment_blocks(text)
            if new != text:
                atomic_write(f, new)
                migrated.append(f"{s}/{f.stem}")
    if migrated:
        print(f"Migrated {len(migrated)} task(s) to new comment format:")
        for name in migrated:
            print(f"  {name}")


def _migrate_v3_dot_to_parent(root: Path) -> None:
    """v3: record parentage in a ``parent`` field derived from legacy dotted
    IDs, so hierarchy no longer depends on the filename. IDs are left as-is
    (their dots become inert); only the field is added, and only when absent."""
    migrated = []
    for s in _ALL_STATUSES:
        d = root / s
        if not d.exists():
            continue
        for f in sorted(d.glob("*.md")):
            stem = f.stem
            dot = stem.rfind(".")
            if dot < 0 or not stem[dot + 1:].isdigit():
                continue  # not a legacy child id
            task = load_task(f)
            if not task or task.get("parent"):
                continue  # already has explicit parentage
            task["parent"] = stem[:dot]
            save_task(f, task)
            migrated.append(f"{s}/{stem}")
    if migrated:
        print(f"Migrated {len(migrated)} task(s) to parent-field hierarchy:")
        for name in migrated:
            print(f"  {name}")


# Ordered migration steps: (schema_version, step_fn). Each step is idempotent
# and advances the herd to its version. Append new steps here and bump
# CURRENT_SCHEMA_VERSION in lockstep.
_MIGRATIONS = [
    (1, _migrate_v1_yaml_to_md),
    (2, _migrate_v2_comment_blocks),
    (3, _migrate_v3_dot_to_parent),
]


def load_config(root: Path) -> dict:
    """Load config with user-global → per-project layering.
    Per-project keys override user-global keys (shallow merge)."""
    merged: dict = {}
    user_cfg = Path.home() / ".config" / "yaks" / "config.yaml"
    if user_cfg.exists():
        merged.update(yaml.safe_load(user_cfg.read_text()) or {})
    project_cfg = root / "config.yaml"
    if project_cfg.exists():
        merged.update(yaml.safe_load(project_cfg.read_text()) or {})
    return merged


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
    atomic_write(path, "".join(parts))


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
            try:
                task = load_task(f)
            except (OSError, yaml.YAMLError):
                # The file vanished or was mid-write between the glob and the
                # read — e.g. an agent moved/removed a yak directly while a
                # live `yaks tui` scan was running. Skip it instead of crashing
                # the whole scan; the next reload sees the settled state.
                continue
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

def parent_of(task: dict) -> str | None:
    """Return a task's parent ID from its ``parent`` field, or None if it is
    top-level. Hierarchy lives in the frontmatter, not the ID (yak-3fd4.6)."""
    return task.get("parent") or None


def find_children(root: Path, task_id: str) -> list[tuple[str, dict]]:
    """Return (status, task) for all direct children (tasks whose ``parent``
    field is *task_id*), sorted by creation time then ID."""
    children = []
    for s in STATUSES:
        for st, t in all_tasks(root, s):
            if (t.get("parent") or None) == task_id:
                children.append((st, t))
    children.sort(key=lambda x: (x[1].get("created", ""), x[1].get("id", "")))
    return children


def descendant_ids(root: Path, task_id: str, include_dead: bool = False) -> set[str]:
    """IDs of all descendants of *task_id* at any depth, following ``parent``
    pointers. include_dead extends the walk into slaughtered yaks."""
    statuses = _ALL_STATUSES if include_dead else STATUSES
    children_of: dict[str, list[str]] = {}
    for s in statuses:
        for _st, t in all_tasks(root, s):
            p = t.get("parent")
            if p:
                children_of.setdefault(p, []).append(t.get("id", ""))
    out: set[str] = set()
    stack = list(children_of.get(task_id, []))
    while stack:
        cur = stack.pop()
        if cur and cur not in out:
            out.add(cur)
            stack.extend(children_of.get(cur, []))
    return out


# ---------------------------------------------------------------------------
# Git integration
# ---------------------------------------------------------------------------

def move_task(root: Path, task_id: str, dest_status: str,
              extra_fields: dict | None = None) -> tuple[bool, str]:
    """Move a task's file into *dest_status* and update timestamps.

    Returns (ok, message) where `ok` is False if the task is missing or
    already at dest_status. No stdout/stderr — callers render the message.
    """
    result = find_task_file(root, task_id)
    if not result:
        return False, f"task {task_id} not found"
    status, path = result
    if status == dest_status:
        return False, f"{task_id} already at {dest_status}"
    dest_dir = root / dest_status
    dest_dir.mkdir(exist_ok=True)
    dest = dest_dir / path.name
    path.rename(dest)
    task = load_task(dest)
    task["updated"] = now_iso()
    if extra_fields:
        task.update(extra_fields)
    save_task(dest, task)
    return True, f"{task_id} → {dest_status}"
