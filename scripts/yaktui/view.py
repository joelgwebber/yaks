"""View model for the TUI list pane (yak-4473 substrate).

A View is a named, ordered entry in the list pane's tab strip. In this first
tranche (yak-5892) a View is just a label + the status it scopes to, and the
only Views are the three built-in status Views — so behavior is identical to
the old fixed tabs. Later tranches extend View with a FilterSpec, sort_by /
sort_dir, a limit, and a tree/flat layout, and make the list user-editable,
pinnable, and persistent.
"""

from __future__ import annotations

from dataclasses import dataclass, field

from yaklib.filter import FilterSpec
from yaklib.model import HAIRY, SHAVING, SHORN


@dataclass
class View:
    name: str                  # tab-strip label (may include an emoji)
    status: str | None = None  # status Views scope to one status dir
    builtin: bool = False      # built-in Views can't be deleted, only reordered
    # The View's saved filter. Activating a View loads this into the single
    # live filter (yak-1b89); status Views carry a spec that scopes to their
    # status, so status is just another (removable) filter axis at runtime.
    spec: FilterSpec = field(default_factory=FilterSpec)

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
    """The three always-present, un-deletable, first-in-order status Views.
    Each carries a spec scoping to its status, so activating it loads that
    status into the live filter."""
    return [View(name=name, status=status, builtin=True,
                 spec=FilterSpec(statuses=frozenset({status})))
            for name, status in _STATUS_VIEWS]
