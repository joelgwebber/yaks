"""Artifact storage layout and link parsing.

Artifacts are files stored under .yaks/artifacts/{yak-id}/ and referenced
from a yak's markdown body via standard `![desc](artifacts/{id}/{name})`
links. Only links that occupy a line on their own (and lie outside fenced
code blocks) count as attachments — this keeps prose examples from being
misread as references.
"""

from __future__ import annotations

import re
from pathlib import Path

_ARTIFACT_LINE_RE = re.compile(
    r"^\s*!\[([^\]]*)\]\(artifacts/([^/)]+)/([^)]+)\)\s*$"
)


def artifacts_dir(root: Path, yak_id: str) -> Path:
    return root / "artifacts" / yak_id


def parse_artifacts(body: str, yak_id: str) -> list[tuple[str, str]]:
    """Return [(desc, filename)] for artifact links belonging to yak_id."""
    out = []
    in_fence = False
    for line in (body or "").split("\n"):
        if line.lstrip().startswith("```"):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        m = _ARTIFACT_LINE_RE.match(line)
        if m and m.group(2) == yak_id:
            out.append((m.group(1), m.group(3)))
    return out
