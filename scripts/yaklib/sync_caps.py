"""Per-tracker capability matrix for bidirectional sync.

Single source of truth for "what can we push, where." Both the TUI dialog
(``yaktui/sync_review.py``) and the agent-driven sync skill consult this
matrix when deciding whether a ``direction: local`` row is even feasible.

Capability values:

- ``ok`` — round-trip safe; push freely.
- ``lossy`` — push possible but data may degrade (e.g. md→ADF).
- ``normalizer`` — upstream silently rewrites; we apply the same
  normalization at diff time so round-trip is neutralized.
- ``transition`` — push needs a multi-step lookup (e.g. Jira workflow).
- ``binary`` — lossy compression of state space (e.g. GitHub OPEN/CLOSED).
- ``manual`` — no API; surface a hand-off and stop.
- ``n/a`` — concept doesn't exist upstream; excluded from diff entirely.

For unknown trackers the matrix returns ``ok`` for everything except
attachments (``manual``); callers should also degrade gracefully.
"""

from __future__ import annotations

# Field/bucket names the matrix knows about.
FIELDS = ("title", "description", "priority", "labels", "status")
BUCKETS = ("comments_up", "attachments_up")

# Capability value constants. Plain strings so the matrix renders
# legibly when serialized into a sidecar.
OK = "ok"
LOSSY = "lossy"
NORMALIZER = "normalizer"
TRANSITION = "transition"
BINARY = "binary"
MANUAL = "manual"
NA = "n/a"

# Per-tracker capabilities. See docs/design/sync.md for the full table
# and the rationale behind each non-``ok`` cell.
TRACKER_CAPS: dict[str, dict[str, str]] = {
    "jira": {
        "title": OK,
        "description": LOSSY,       # markdown↔ADF round-trip degrades
        "priority": OK,             # 1↔1 identity
        "labels": OK,
        "status": TRANSITION,       # workflow lookup; may reject
        "comments_up": OK,
        "attachments_up": MANUAL,   # MCP exposes metadata only
    },
    "linear": {
        "title": OK,
        "description": NORMALIZER,  # silent markdown rewrites
        "priority": OK,             # 0=None ↔ yak 3 (default) ambiguity
        "labels": OK,
        "status": OK,               # state lookup
        "comments_up": OK,
        "attachments_up": OK,       # create_attachment (base64)
    },
    "github": {
        "title": OK,
        "description": OK,
        "priority": NA,             # not a concept
        "labels": OK,
        "status": BINARY,           # only OPEN/CLOSED; "shaving" invisible
        "comments_up": OK,
        "attachments_up": MANUAL,   # no public upload API
    },
}

# Fallback for unknown trackers. Conservative: assume text fields
# work, attachments don't.
_DEFAULT_CAPS: dict[str, str] = {
    "title": OK,
    "description": OK,
    "priority": OK,
    "labels": OK,
    "status": OK,
    "comments_up": OK,
    "attachments_up": MANUAL,
}

# Capability values that actually permit a push attempt.
_PUSHABLE = frozenset({OK, LOSSY, NORMALIZER, TRANSITION, BINARY})


def push_capability(tracker: str, field: str) -> str:
    """Return the capability for ``tracker``/``field``.

    Unknown trackers fall back to a conservative default; unknown fields
    return ``ok`` (caller should usually pre-validate the field name).
    """
    caps = TRACKER_CAPS.get(tracker, _DEFAULT_CAPS)
    return caps.get(field, OK)


def is_pushable(tracker: str, field: str) -> bool:
    """True when the field can be pushed upstream (even if degraded).

    ``manual`` and ``n/a`` are not pushable. ``lossy``/``transition``/
    ``binary`` are pushable; the caller decides whether to warn.
    """
    return push_capability(tracker, field) in _PUSHABLE


def is_diffable(tracker: str, field: str) -> bool:
    """True when the field should appear in the plan-phase diff at all.

    Only ``n/a`` excludes a field from the diff entirely. Everything
    else is diffable; capability gates *push*, not *read*.
    """
    return push_capability(tracker, field) != NA
