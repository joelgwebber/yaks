"""Shared formatting helpers for CLI and TUI output."""

from __future__ import annotations

from datetime import datetime, timezone

# Single-character status tag used in tight layouts (list view, detail links).
# Imported lazily to avoid a hard dependency on yak.py during the refactor —
# callers pass the status string directly.
STATUS_CHAR = {
    "hairy": "H",
    "shaving": "S",
    "shorn": "N",
    "dead": "D",
}


def status_char(status: str) -> str:
    """Return the one-character tag for a status, or '?' if unknown."""
    return STATUS_CHAR.get(status, "?")


def humanize_date(value) -> str:
    """Render an ISO8601 timestamp as a relative + absolute local string.

    Examples: '5 minutes ago (14:16)', 'yesterday at 14:16',
    '3 days ago (Apr 2, 14:16)', 'Jan 15, 2025 14:16'.
    """
    if not value or not isinstance(value, str):
        return str(value) if value else "-"
    try:
        s = value.replace("Z", "+00:00")
        dt = datetime.fromisoformat(s)
    except ValueError:
        return value
    if dt.tzinfo is None:
        dt = dt.replace(tzinfo=timezone.utc)
    local = dt.astimezone()
    now = datetime.now(local.tzinfo)
    secs = (now - local).total_seconds()

    time_str = local.strftime("%H:%M")
    if secs < 0:
        return local.strftime("%b %-d, %Y %H:%M")
    if secs < 60:
        rel = "just now"
    elif secs < 3600:
        m = int(secs // 60)
        rel = f"{m} minute{'s' if m != 1 else ''} ago"
    elif secs < 86400 and local.date() == now.date():
        h = int(secs // 3600)
        rel = f"{h} hour{'s' if h != 1 else ''} ago"
    else:
        days = (now.date() - local.date()).days
        if days == 1:
            return f"yesterday at {time_str}"
        if days < 7:
            return f"{days} days ago ({local.strftime('%b %-d')}, {time_str})"
        if local.year == now.year:
            return local.strftime("%b %-d, %H:%M")
        return local.strftime("%b %-d, %Y %H:%M")
    return f"{rel} ({time_str})"
