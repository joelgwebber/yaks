"""View model for the TUI list pane (yak-4473 substrate).

A View is a named, ordered entry in the list pane's tab strip. In this first
tranche (yak-5892) a View is just a label + the status it scopes to, and the
only Views are the three built-in status Views — so behavior is identical to
the old fixed tabs. Later tranches extend View with a FilterSpec, sort_by /
sort_dir, a limit, and a tree/flat layout, and make the list user-editable,
pinnable, and persistent.
"""

from __future__ import annotations

from dataclasses import dataclass

from yaklib.model import HAIRY, SHAVING, SHORN


@dataclass
class View:
    name: str                  # tab-strip label (may include an emoji)
    status: str | None = None  # status Views scope to one status dir
    builtin: bool = False      # built-in Views can't be deleted, only reordered

    @property
    def is_status(self) -> bool:
        return self.status is not None


# Label strings are byte-identical to the former render.TABS so the tab row
# renders exactly as before.
_STATUS_VIEWS = [
    ("\U0001f9ac Hairy", HAIRY),
    ("\u2702\ufe0f  Shaving", SHAVING),
    ("\U0001f411 Shorn", SHORN),
]


def builtin_status_views() -> list[View]:
    """The three always-present, un-deletable, first-in-order status Views."""
    return [View(name=name, status=status, builtin=True)
            for name, status in _STATUS_VIEWS]
