"""Persistent, stat-validated task index (yak-3fd4.2).

A per-user, derived, never-committed cache that makes full-scan operations
(list/search/next/tangled/stats/rollup and the TUI) fast at scale: cold start
drops from re-parsing every file (~12.6s at 50k) to a single index load plus a
stat-only reconcile (~0.15s), reparsing only the files that actually changed.

Design (git-index-shaped, bulletproof by construction):

- The authoritative state is always the files on disk. The index is a pure
  cache: every load re-stats every task file and reconciles, so a stale,
  corrupt, or missing index simply rebuilds itself. There is no "clear your
  cache" failure mode and no second source of truth.
- One record per task id: its status (which dir it lives in), the file's
  mtime_ns + size (the validation key), and the full parsed task dict
  (description included, so in-memory search needs no reparse).
- A pure status change is a rename that preserves mtime+size, so it is applied
  by moving the record between statuses without reparsing.
- Racy-clean handling like git: any file whose mtime is at or after the index's
  own build time is reparsed, since stat alone can't prove it is unchanged.
- Written atomically (temp + os.replace) so a concurrent TUI/CLI never sees a
  torn index.

The on-disk format (JSON) is an implementation detail; a future Rust/binary
layout can swap in behind the same reconcile contract.
"""

from __future__ import annotations

import json
import os
import time
from pathlib import Path

from yaklib import model
from yaklib.model import _ALL_STATUSES, STATUSES

_INDEX_VERSION = 1


class Index:
    """In-memory task records backed by a stat-validated on-disk cache."""

    def __init__(self, root: Path, path: Path):
        self.root = root
        self.path = path
        # id -> {"status": str, "mtime_ns": int, "size": int, "task": dict}
        self.records: dict[str, dict] = {}
        self.built_ns = 0
        self._synced = False

    # -- disk I/O ----------------------------------------------------------

    def load(self) -> "Index":
        """Populate records from the cache file. Any anomaly => empty (a full
        rebuild then happens on the next sync). Never raises."""
        try:
            data = json.loads(self.path.read_text())
        except (OSError, json.JSONDecodeError, ValueError):
            data = None
        if (isinstance(data, dict) and data.get("v") == _INDEX_VERSION
                and isinstance(data.get("records"), dict)):
            self.built_ns = int(data.get("built_ns") or 0)
            self.records = data["records"]
        else:
            self.built_ns = 0
            self.records = {}
        return self

    def write(self) -> None:
        payload = {"v": _INDEX_VERSION, "built_ns": self.built_ns,
                   "records": self.records}
        self.path.parent.mkdir(parents=True, exist_ok=True)
        model.atomic_write(self.path, json.dumps(payload, separators=(",", ":")))

    # -- sync / query ------------------------------------------------------

    def mark_stale(self) -> None:
        """Force the next ensure_synced() to re-validate against disk."""
        self._synced = False

    def ensure_synced(self) -> "Index":
        if not self._synced:
            self.sync()
        return self

    def sync(self) -> bool:
        """Re-stat every task file, reconcile records, and persist if anything
        changed. Returns True when the record set changed. Reparses only new,
        modified, or racy files; a pure status change reuses the cached task."""
        seen: dict[str, tuple[str, int, int, Path]] = {}
        for status in _ALL_STATUSES:  # hairy, shaving, shorn, dead — first wins
            d = self.root / status
            try:
                entries = list(os.scandir(d))
            except OSError:
                continue
            for e in entries:
                name = e.name
                if not name.endswith(".md"):
                    continue
                tid = name[:-3]
                if tid in seen:
                    continue  # matches find_task_file's status precedence
                try:
                    st = e.stat()
                except OSError:
                    continue
                seen[tid] = (status, st.st_mtime_ns, st.st_size, Path(e.path))

        new_records: dict[str, dict] = {}
        changed = False
        for tid, (status, mtime_ns, size, path) in seen.items():
            old = self.records.get(tid)
            racy = mtime_ns >= self.built_ns
            if (old is not None and old.get("mtime_ns") == mtime_ns
                    and old.get("size") == size and not racy):
                if old.get("status") == status:
                    rec = old
                else:  # pure rename (status change), content intact
                    rec = {**old, "status": status}
                    changed = True
            else:
                try:
                    task = model.load_task(path)
                except OSError:
                    # Vanished/renamed mid-scan; keep any prior record so we
                    # don't lose it to a transient race, else just skip.
                    if old is not None:
                        new_records[tid] = old
                    continue
                except Exception:
                    # Malformed file: skip it (matches all_tasks tolerance).
                    continue
                if not task:
                    continue
                rec = {"status": status, "mtime_ns": mtime_ns,
                       "size": size, "task": task}
                changed = True
            new_records[tid] = rec

        if set(new_records) != set(self.records):
            changed = True
        self.records = new_records
        if changed:
            self.built_ns = time.time_ns()
            try:
                self.write()
            except OSError:
                pass  # a read-only cache dir must not break reads
        self._synced = True
        return changed

    def tasks(self, status: str | None = None) -> list[tuple[str, dict]]:
        """(status, task) pairs mirroring model.all_tasks: a single status when
        given, else the visible STATUSES (dead excluded), sorted by id within
        each status. Returned task dicts are shared cache objects — read-only."""
        scan = (status,) if status is not None else STATUSES
        out: list[tuple[str, dict]] = []
        for s in scan:
            recs = [r for r in self.records.values() if r.get("status") == s]
            recs.sort(key=lambda r: r["task"].get("id", ""))
            out.extend((s, r["task"]) for r in recs)
        return out
