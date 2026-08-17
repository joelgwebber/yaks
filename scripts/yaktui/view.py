"""View model for the TUI list pane (yak-4473 substrate).

A View is a named, ordered entry in the list pane's tab strip. In this first
tranche (yak-5892) a View is just a label + the status it scopes to, and the
only Views are the three built-in status Views — so behavior is identical to
the old fixed tabs. Later tranches extend View with a FilterSpec, sort_by /
sort_dir, a limit, and a tree/flat layout, and make the list user-editable,
pinnable, and persistent.
"""

from __future__ import annotations

import uuid
from dataclasses import dataclass, field

from yaklib.filter import FilterSpec
from yaklib.model import HAIRY, SHAVING, SHORN

# Fields a View may sort by (yak-b601). ISO timestamps sort chronologically as
# plain strings, so date sorts need no parsing.
SORT_FIELDS = ("updated", "created", "priority", "title", "id")


@dataclass
class View:
    name: str                  # tab-strip label (may include an emoji)
    key: str = ""              # stable id for persistence (e.g. status:hairy)
    status: str | None = None  # status Views scope to one status dir
    builtin: bool = False      # built-in Views can't be deleted, only reordered
    pinned: bool = True        # pinned Views appear on the tab bar (yak-6b51)
    # The View's saved filter. Activating a View loads this into the single
    # live filter (yak-1b89); status Views carry a spec that scopes to their
    # status, so status is just another (removable) filter axis at runtime.
    spec: FilterSpec = field(default_factory=FilterSpec)
    # Sorting (yak-b601). A View that sorts renders FLAT (a sort order can't
    # coexist with the parent/child tree); status Views leave sort_by None and
    # stay tree views. limit caps the rows shown (None = unlimited).
    sort_by: str | None = None
    sort_dir: str = "desc"     # "asc" | "desc"
    limit: int | None = None

    @property
    def is_status(self) -> bool:
        return self.status is not None

    @property
    def is_flat(self) -> bool:
        """Sorted Views are flat; unsorted (status) Views render as a tree."""
        return self.sort_by is not None


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
    return [View(name=name, key=f"status:{status}", status=status, builtin=True,
                 spec=FilterSpec(statuses=frozenset({status})))
            for name, status in _STATUS_VIEWS]


# How many rows the Recent view shows. Recent is a working list, not an
# archive, so it is capped; the exact number is a tunable presentation choice.
RECENT_LIMIT = 50


def recent_view() -> View:
    """The built-in Recent View (yak-b601): every task, most-recently-updated
    first, flat, capped. Derived purely from the `updated` field — navigating
    TO a yak does not bump `updated`, so the list does not churn under you.
    Pinned by default, so a new user meets the View affordance on day one."""
    return View(name="\U0001f552 Recent", key="recent", status=None, builtin=True,
                spec=FilterSpec(), sort_by="updated", sort_dir="desc",
                limit=RECENT_LIMIT)


def default_views() -> list[View]:
    """The Views a fresh TUI starts with: the three status Views plus Recent.
    (Once yak-6b51 lands, user-defined/pinned Views merge in from storage.)"""
    return [*builtin_status_views(), recent_view()]


def custom_view(name: str, spec: FilterSpec, sort_by: str | None = None,
                sort_dir: str = "desc", limit: int | None = None) -> View:
    """A user-created (saved) View with a generated stable key (yak-a373).
    Pinned by default so it lands on the tab bar; not built-in, so the picker
    can rename or delete it."""
    return View(name=name, key=f"view:{uuid.uuid4().hex[:8]}", builtin=False,
                pinned=True, spec=spec, sort_by=sort_by, sort_dir=sort_dir,
                limit=limit)
