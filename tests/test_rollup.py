"""Tests for the rollup projection: source classification, ancestor
inheritance, and the `yaks rollup` CLI surface."""

from __future__ import annotations

from conftest import create_task
from yaklib.rollup import effective_source, tracker_and_key

# ---------------------------------------------------------------------------
# tracker_and_key — pure URL classification
# ---------------------------------------------------------------------------


def test_tracker_and_key_jira():
    assert tracker_and_key("https://acme.atlassian.net/browse/SUBTEXT-369") == ("jira", "SUBTEXT-369")


def test_tracker_and_key_linear_uppercases():
    assert tracker_and_key("https://linear.app/acme/issue/roc-5/some-slug") == ("linear", "ROC-5")


def test_tracker_and_key_github():
    assert tracker_and_key("https://github.com/acme/widgets/issues/123") == ("github", "acme/widgets#123")


def test_tracker_and_key_other_falls_back_to_url():
    assert tracker_and_key("https://example.com/tickets/42") == ("other", "https://example.com/tickets/42")


def test_tracker_and_key_none():
    assert tracker_and_key(None) == ("none", None)
    assert tracker_and_key("") == ("none", None)


# ---------------------------------------------------------------------------
# effective_source — ancestor inheritance
# ---------------------------------------------------------------------------


def test_effective_source_own_beats_inherited():
    by_id = {"x-1": "PARENT-URL", "x-1.2": "OWN-URL"}
    assert effective_source("x-1.2", by_id) == ("OWN-URL", None)


def test_effective_source_inherits_from_nearest_ancestor():
    by_id = {"x-1": "GRANDPARENT-URL"}
    # x-1.2.3 has no own source; nearest ancestor with one is x-1.
    assert effective_source("x-1.2.3", by_id) == ("GRANDPARENT-URL", "x-1")


def test_effective_source_none_when_no_ancestor_has_source():
    assert effective_source("x-1.2", {}) == (None, None)


# ---------------------------------------------------------------------------
# CLI: yaks rollup
# ---------------------------------------------------------------------------


def test_rollup_groups_by_source_json(yak):
    j = "https://acme.atlassian.net/browse/SUBTEXT-369"
    a = create_task(yak, "alpha", type="task", source=j)
    b = create_task(yak, "beta", type="task", source=j)
    create_task(yak, "no source here", type="task")  # omitted from rollup

    out = yak("rollup", "--json").json()
    assert len(out) == 1
    grp = out[0]
    assert grp["tracker"] == "jira"
    assert grp["key"] == "SUBTEXT-369"
    assert {y["id"] for y in grp["yaks"]} == {a, b}
    assert all(y["inherited"] is False for y in grp["yaks"])


def test_rollup_inherits_source_from_parent(yak):
    j = "https://acme.atlassian.net/browse/SUBTEXT-369"
    parent = create_task(yak, "umbrella", type="feature", source=j)
    child = create_task(yak, "child work", type="task", parent=parent)

    out = yak("rollup", "--json").json()
    assert len(out) == 1
    yaks = {y["id"]: y for y in out[0]["yaks"]}
    assert yaks[parent]["inherited"] is False
    assert yaks[child]["inherited"] is True
    assert yaks[child]["inherited_from"] == parent


def test_rollup_keys_dedupes_and_lists(yak):
    j = "https://acme.atlassian.net/browse/SUBTEXT-369"
    g = "https://github.com/acme/widgets/issues/7"
    create_task(yak, "a", type="task", source=j)
    create_task(yak, "b", type="task", source=j)  # same key
    create_task(yak, "c", type="task", source=g)

    lines = yak("rollup", "--keys").stdout.split()
    assert lines == ["SUBTEXT-369", "acme/widgets#7"]


def test_rollup_empty_when_no_sources(yak):
    create_task(yak, "lonely", type="task")
    out = yak("rollup").stdout
    assert "No yaks with an external source." in out


def test_rollup_respects_label_filter(yak):
    j1 = "https://acme.atlassian.net/browse/SUBTEXT-1"
    j2 = "https://acme.atlassian.net/browse/SUBTEXT-2"
    create_task(yak, "shipping", type="task", source=j1, labels=["pr-1"])
    create_task(yak, "other", type="task", source=j2)

    out = yak("rollup", "--label", "pr-1", "--json").json()
    assert len(out) == 1
    assert out[0]["key"] == "SUBTEXT-1"
