"""Inter-yak link detection in free-form text.

Supports two forms:
- Bare `yak-abcd` tokens — auto-detected.
- `[[yak-abcd]]` wiki-link form — brackets are stripped for display.

Rationale for the bracket form: CommonMark / GFM / Hugo / Jekyll all render
[[...]] as literal text, so no conflict. Obsidian, Roam and Logseq use [[...]]
as wiki-link syntax, so the choice composes well for anyone using yaks
alongside those tools. The #yak-N and @yak-N forms were rejected: GFM
renders #N as an issue reference and @user as a mention.
"""

from __future__ import annotations

import re
from pathlib import Path

from yaklib.model import find_task_file

_ID_CORE = r"[a-z][a-z0-9-]*-[0-9a-f]{4}(?:\.\d+)*"

# Wiki-link form: [[yak-abcd]] → strip the brackets for display. Keep the
# captured ID so callers can rewrite the text in one pass.
EXPLICIT_LINK_RE = re.compile(rf"\[\[({_ID_CORE})\]\]")

# Word-boundaried scan for bare IDs in already-unbracketed text. Avoids
# matching mid-identifier and common substring false positives.
BARE_LINK_RE = re.compile(rf"(?<![\w-])({_ID_CORE})(?![\w])")


def strip_explicit_brackets(text: str) -> str:
    """Rewrite [[yak-abcd]] → yak-abcd so downstream code can treat both
    forms uniformly."""
    return EXPLICIT_LINK_RE.sub(r"\1", text or "")


def find_link_spans(text: str) -> list[tuple[int, int, str]]:
    """Return [(start, end, task_id)] for bare yak-ID occurrences in *text*.

    Caller is expected to have already passed the text through
    strip_explicit_brackets if mixing explicit and bare forms.
    """
    return [(m.start(1), m.end(1), m.group(1)) for m in BARE_LINK_RE.finditer(text or "")]


def resolve_spans(root: Path, text: str, self_id: str,
                  exclude: set[str] | None = None
                  ) -> list[tuple[int, int, str]]:
    """Return [(start, end, task_id)] for spans whose task_id resolves on disk,
    skipping *self_id* and anything in *exclude*."""
    skip = set(exclude or ())
    skip.add(self_id)
    out = []
    for start, end, tid in find_link_spans(text):
        if tid in skip:
            continue
        if find_task_file(root, tid) is None:
            continue
        out.append((start, end, tid))
    return out
