"""Core model: status constants, YAML I/O, filesystem layout, task loading."""

from __future__ import annotations

import hashlib
import os
import random
import re
import string
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path

import yaml

try:  # libyaml C bindings: 2-4x faster, identical semantics. Not always built.
    from yaml import CSafeLoader as _SafeLoader
except ImportError:  # pragma: no cover - depends on the local libyaml build
    from yaml import SafeLoader as _SafeLoader


def _yaml_load(text: str):
    """safe_load via the fastest available loader (libyaml when present)."""
    return yaml.load(text, Loader=_SafeLoader)


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
        merged.update(_yaml_load(user_cfg.read_text()) or {})
    project_cfg = root / "config.yaml"
    if project_cfg.exists():
        merged.update(_yaml_load(project_cfg.read_text()) or {})
    return merged


# ---------------------------------------------------------------------------
# Task I/O
# ---------------------------------------------------------------------------

# --- fast-path frontmatter parser -----------------------------------------
#
# Our writer (dump_yaml) emits a tiny, regular subset of YAML for frontmatter:
# top-level `key: scalar` lines and `key:` + `- item` block lists, where each
# scalar is either plain or single-quoted (strings that could be mistyped are
# single-quoted by PyYAML, so a *plain* token is genuinely its resolved type).
# _fast_frontmatter parses exactly that subset and returns None on anything
# else (long-line folds, block/flow scalars, nesting, comments, exotic types),
# so load_task can defer to the full loader. Scalar typing reuses PyYAML's own
# implicit resolver, so accepted results are identical to safe_load.

_BAIL = object()
_STR_TAG = "tag:yaml.org,2002:str"
_INT_TAG = "tag:yaml.org,2002:int"
_FM_KEY_RE = re.compile(r"([A-Za-z0-9_-]+):(?: (.*))?$")
_FM_INT_RE = re.compile(r"-?\d+$")
_fm_resolver = yaml.resolver.Resolver()


def _fast_scalar(tok: str):
    """Parse one plain/single-quoted scalar, or return _BAIL to force fallback."""
    tok = tok.rstrip()
    if not tok:
        return _BAIL
    if tok[0] == "'":
        # Single-quoted: only escape is '' -> '. Reject unterminated (wrapped
        # onto the next line) or malformed values.
        if len(tok) < 2 or tok[-1] != "'":
            return _BAIL
        inner = tok[1:-1]
        if "'" in inner.replace("''", ""):
            return _BAIL
        return inner.replace("''", "'")
    # Anything with a leading indicator or comment/mapping punctuation is beyond
    # the plain-scalar subset (double-quote, flow, block, alias, tag, etc.).
    if tok[0] in "\"[]{}|>*&!%@`," or " #" in tok or ": " in tok:
        return _BAIL
    tag = _fm_resolver.resolve(yaml.ScalarNode, tok, (True, False))
    if tag == _STR_TAG:
        return tok
    if tag == _INT_TAG and _FM_INT_RE.fullmatch(tok):
        return int(tok)
    return _BAIL  # bool/null/float/timestamp/exotic-int -> let the loader type it


def _fast_frontmatter(fm: str):
    """Parse the writer's YAML subset; return a dict, or None to fall back."""
    result: dict = {}
    lines = fm.split("\n")
    i, n = 0, len(lines)
    while i < n:
        line = lines[i]
        if not line or line.isspace():
            i += 1
            continue
        m = _FM_KEY_RE.match(line)
        if not m:
            return None
        key, val = m.group(1), m.group(2)
        if val is None:
            # `key:` with no inline value: expect a block list on following
            # lines. A bare key (null/empty) is rare here; defer to the loader.
            items = []
            i += 1
            while i < n and lines[i].startswith("- "):
                sval = _fast_scalar(lines[i][2:])
                if sval is _BAIL:
                    return None
                items.append(sval)
                i += 1
            if not items:
                return None
            result[key] = items
            continue
        sval = _fast_scalar(val)
        if sval is _BAIL:
            return None
        result[key] = sval
        i += 1
    return result


def _lenient_scalar(tok: str):
    """Best-effort scalar for recovery parsing; never raises. Strips matching
    surrounding quotes and coerces plain integers; everything else is raw."""
    tok = tok.strip()
    if len(tok) >= 2 and tok[0] == tok[-1] and tok[0] in "'\"":
        inner = tok[1:-1]
        return inner.replace("''", "'") if tok[0] == "'" else inner
    if _FM_INT_RE.fullmatch(tok):
        return int(tok)
    return tok


def _lenient_frontmatter(fm: str) -> dict:
    """Recover top-level ``key: value`` scalars and ``key:`` / ``- item`` lists
    from frontmatter that strict YAML rejects (e.g. an unescaped colon in a
    title). Never raises; unreadable lines are skipped. This keeps one bad file
    from bricking the tool and keeps the yak visible so it can be fixed.
    Priority is coerced to int so mixed-type sorts stay safe."""
    out: dict = {}
    lines = fm.split("\n")
    i, n = 0, len(lines)
    while i < n:
        line = lines[i]
        if not line.strip():
            i += 1
            continue
        m = _FM_KEY_RE.match(line)
        if not m:
            i += 1
            continue
        key, val = m.group(1), m.group(2)
        if val is None:
            items = []
            i += 1
            while i < n and lines[i].startswith("- "):
                items.append(_lenient_scalar(lines[i][2:]))
                i += 1
            if items:
                out[key] = items
            continue
        out[key] = _lenient_scalar(val)
        i += 1
    return out


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
        task = _fast_frontmatter(fm)
        if task is None:  # subset miss -> full loader (same result, just slower)
            try:
                task = _yaml_load(fm) or {}
            except yaml.YAMLError as e:
                # Malformed frontmatter (e.g. an unescaped colon). Recover what
                # we can instead of raising: a single bad file must never crash
                # a scan or a point read (yak-cae6). Flag it so the UI can say
                # so; the id comes from the filename when unparseable.
                task = _lenient_frontmatter(fm)
                # The filename is authoritative for the id (the index and
                # find_task_file key on it); a recovered id may be garbage.
                task["id"] = path.stem
                task["_error"] = getattr(e, "problem", None) or e.__class__.__name__
        body = body.strip()
        if body:
            task["description"] = body
        return task
    return _yaml_load(text) or {}


def save_task(path: Path, task: dict) -> None:
    # Drop private, derived keys (e.g. the _error recovery flag) so they never
    # get written back into the file.
    task = {k: v for k, v in task.items() if not k.startswith("_")}
    description = task.pop("description", None)
    fm = dump_yaml(task)
    parts = ["---\n", fm, "---\n"]
    if description:
        parts.append("\n")
        parts.append(description)
        if not description.endswith("\n"):
            parts.append("\n")
    atomic_write(path, "".join(parts))
    _mark_index_stale()


# ---------------------------------------------------------------------------
# Per-user derived cache (index + UI state). Never committed; rebuildable.
# ---------------------------------------------------------------------------

def _project_slug(root: Path) -> str:
    return hashlib.sha1(str(root.resolve()).encode("utf-8")).hexdigest()[:12]


def cache_dir(root: Path) -> Path:
    """Per-project cache directory under XDG_CACHE_HOME (default ~/.cache), for
    rebuildable/derived state (the index, collapsed ids). Keyed by a hash of the
    absolute root so distinct herds never collide."""
    cache_home = os.environ.get("XDG_CACHE_HOME") or str(Path.home() / ".cache")
    return Path(cache_home) / "yaks" / _project_slug(root)


def config_dir(root: Path) -> Path:
    """Per-project config directory under XDG_CONFIG_HOME (default ~/.config),
    for DURABLE user intent (saved/pinned Views, working-set pins) that must
    survive a cache wipe. Distinct from cache_dir on purpose."""
    config_home = os.environ.get("XDG_CONFIG_HOME") or str(Path.home() / ".config")
    return Path(config_home) / "yaks" / _project_slug(root)


# Process-lifetime index singletons, keyed by resolved root. One stat-validated
# sync per read phase; mutations mark it stale so the next scan re-validates.
_INDEX_CACHE: dict[str, object] = {}


def _shared_index(root: Path):
    from yaklib.index import Index
    key = str(root.resolve())
    idx = _INDEX_CACHE.get(key)
    if idx is None:
        idx = Index(root, cache_dir(root) / "index.json").load()
        _INDEX_CACHE[key] = idx
    idx.ensure_synced()
    return idx


def _mark_index_stale() -> None:
    """Invalidate every loaded index so the next all_tasks re-validates. Called
    after any task-file mutation that goes through save_task/move_task."""
    for idx in _INDEX_CACHE.values():
        idx.mark_stale()


def refresh_index(root: Path) -> None:
    """Force a re-validation of *root*'s index on the next scan. The TUI calls
    this before a reload so externally-made changes (including direct file
    deletes) are picked up."""
    _mark_index_stale()


def _all_tasks_direct(root: Path, status: str | None = None) -> list[tuple[str, dict]]:
    """Reference full scan: read + parse every task file. Used when the index
    is disabled (YAKS_NO_INDEX) and by tests to cross-check the index."""
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


def all_tasks(root: Path, status: str | None = None) -> list[tuple[str, dict]]:
    """Return (status, task_dict) for tasks in the given status dir(s).

    When status is None, only the visible STATUSES are scanned — dead yaks
    are excluded. Pass status=DEAD to inspect slaughtered tasks.

    Backed by the persistent stat-validated index (yaklib.index): the first
    call in a process loads + reconciles it, later calls reuse it until a
    mutation marks it stale. Task dicts are shared cache objects — treat the
    results as read-only (mutating commands re-read via find_task_file +
    load_task). Set YAKS_NO_INDEX=1 to force the direct scan.
    """
    if os.environ.get("YAKS_NO_INDEX"):
        return _all_tasks_direct(root, status)
    return _shared_index(root).tasks(status)


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
