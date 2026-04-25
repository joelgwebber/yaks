"""Pending-sync sidecar IO.

A sidecar is a YAML file at ``.yaks/.sync-pending/<yak-id>.yaml`` that
captures the proposed changes from a sync *plan* phase: silent auto-applies,
prompts the user must resolve, and a snapshot of upstream at plan time so
the *apply* phase can detect "remote changed under us, re-plan."

The format and semantics live in the yak-sync skill; this module is the
plumbing — read, write, list, delete. Plan and apply are agent-driven
(they need MCP access); the CLI exposes only bookkeeping.
"""

from __future__ import annotations

from pathlib import Path

import yaml

from yaklib.model import _BlockScalarDumper

PENDING_DIR = ".sync-pending"


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
