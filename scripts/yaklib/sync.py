"""Pending-sync sidecar IO + sweep helpers.

A sidecar is a YAML file at ``.yaks/.sync-pending/<yak-id>.yaml`` that
captures the proposed changes from a sync *plan* phase: silent auto-applies,
prompts the user must resolve, and a snapshot of upstream at plan time so
the *apply* phase can detect "remote changed under us, re-plan."

This module also exposes sweep helpers — enumerate yaks with a ``source:``
URL, classify by tracker — so an agent can answer "which yaks might need
syncing?" without re-implementing the walk.

Plan, apply, and the upstream-drift query for sweep are all agent-driven
(they need MCP access); the CLI exposes only bookkeeping.
"""

from __future__ import annotations

import re
from pathlib import Path
from urllib.parse import urlparse

import yaml

from yaklib.model import _BlockScalarDumper, all_tasks

PENDING_DIR = ".sync-pending"

# Tracker classification by source-URL host.
_TRACKER_HOST_PATTERNS = [
    ("jira", re.compile(r".*\.atlassian\.net$")),
    ("linear", re.compile(r"(www\.)?linear\.app$")),
    ("github", re.compile(r"(www\.)?github\.com$")),
]

# Per-tracker key extraction from URL path. Each returns the upstream
# identifier (e.g. "SUBTEXT-301") or None if the URL doesn't match.
_KEY_EXTRACTORS: dict[str, "re.Pattern"] = {
    "jira": re.compile(r"/browse/([A-Z][A-Z0-9_]*-\d+)"),
    "linear": re.compile(r"/issue/([A-Z]+-\d+)"),
    # GitHub: owner/repo#N — capture full "owner/repo/issues/N" so it's
    # actionable (the issue number alone doesn't identify the repo).
    "github": re.compile(r"/([^/]+/[^/]+)/issues/(\d+)"),
}


def pending_root(root: Path) -> Path:
    return root / PENDING_DIR


def sidecar_path(root: Path, yak_id: str) -> Path:
    return pending_root(root) / f"{yak_id}.yaml"


def load_sidecar(path: Path) -> dict:
    return yaml.safe_load(path.read_text()) or {}


def save_sidecar(path: Path, data: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    text = yaml.dump(data, Dumper=_BlockScalarDumper, default_flow_style=False,
                     sort_keys=False, allow_unicode=True)
    path.write_text(text)


def list_pending(root: Path) -> list[str]:
    """Return yak IDs (sorted) that have a sidecar."""
    d = pending_root(root)
    if not d.exists():
        return []
    return sorted(p.stem for p in d.glob("*.yaml"))


def has_pending(root: Path, yak_id: str) -> bool:
    return sidecar_path(root, yak_id).exists()


def clear_sidecar(root: Path, yak_id: str) -> bool:
    """Remove the sidecar. Returns True if a file was deleted, False if none."""
    p = sidecar_path(root, yak_id)
    if not p.exists():
        return False
    p.unlink()
    return True


# ---------------------------------------------------------------------------
# Sweep helpers (bf54.2)
# ---------------------------------------------------------------------------


def tracker_for(source_url: str) -> str:
    """Classify a source URL: 'jira' | 'linear' | 'github' | 'other'."""
    if not source_url:
        return "other"
    try:
        host = (urlparse(source_url).hostname or "").lower()
    except Exception:
        return "other"
    for name, pat in _TRACKER_HOST_PATTERNS:
        if pat.match(host):
            return name
    return "other"


def upstream_key_for(source_url: str, tracker: str | None = None) -> str | None:
    """Return the tracker-native identifier from a source URL, or None.

    For Jira this is the issue key (``SUBTEXT-301``). For Linear, same
    shape (``SUB-42``). For GitHub Issues, ``owner/repo#N`` for legibility.
    """
    if not source_url:
        return None
    tracker = tracker or tracker_for(source_url)
    pat = _KEY_EXTRACTORS.get(tracker)
    if not pat:
        return None
    try:
        path = urlparse(source_url).path or ""
    except Exception:
        return None
    m = pat.search(path)
    if not m:
        return None
    if tracker == "github":
        return f"{m.group(1)}#{m.group(2)}"
    return m.group(1)


def iter_synced_yaks(root: Path) -> list[dict]:
    """Enumerate yaks that have a ``source:`` URL.

    Returns a list of dicts (one per yak) with: id, status, tracker, key,
    source, last_synced, local_updated, has_pending. Stable order by id.
    Caller is responsible for any upstream-side drift detection (needs MCP).
    """
    out = []
    for status, t in all_tasks(root):
        src = t.get("source")
        if not src:
            continue
        tracker = tracker_for(src)
        out.append({
            "id": t["id"],
            "status": status,
            "tracker": tracker,
            "key": upstream_key_for(src, tracker),
            "source": src,
            "last_synced": t.get("last_synced"),
            "local_updated": t.get("updated"),
            "has_pending": has_pending(root, t["id"]),
        })
    out.sort(key=lambda r: r["id"])
    return out
