"""Capability-matrix tests (Phase 0 of bf54.11 bidirectional sync)."""

from __future__ import annotations

from yaklib import sync_caps as sc


def test_known_trackers_present():
    for t in ("jira", "linear", "github"):
        assert t in sc.TRACKER_CAPS


def test_jira_description_is_lossy_due_to_adf():
    assert sc.push_capability("jira", "description") == sc.LOSSY


def test_linear_description_uses_normalizer():
    assert sc.push_capability("linear", "description") == sc.NORMALIZER


def test_github_priority_is_na():
    assert sc.push_capability("github", "priority") == sc.NA


def test_github_status_is_binary():
    assert sc.push_capability("github", "status") == sc.BINARY


def test_jira_status_requires_transition():
    assert sc.push_capability("jira", "status") == sc.TRANSITION


def test_attachments_split_by_tracker():
    assert sc.push_capability("jira", "attachments_up") == sc.MANUAL
    assert sc.push_capability("github", "attachments_up") == sc.MANUAL
    assert sc.push_capability("linear", "attachments_up") == sc.OK


def test_unknown_tracker_falls_back_conservatively():
    # Text fields default to ok; attachments default to manual.
    assert sc.push_capability("notion", "title") == sc.OK
    assert sc.push_capability("notion", "attachments_up") == sc.MANUAL


def test_unknown_field_returns_ok():
    # Pre-validation is the caller's job; matrix doesn't fabricate.
    assert sc.push_capability("jira", "made_up_field") == sc.OK


def test_is_pushable_excludes_manual_and_na():
    assert not sc.is_pushable("github", "priority")        # n/a
    assert not sc.is_pushable("github", "attachments_up")  # manual
    assert not sc.is_pushable("jira", "attachments_up")    # manual


def test_is_pushable_includes_lossy_and_transition():
    # Pushable even though degraded — caller decides whether to warn.
    assert sc.is_pushable("jira", "description")  # lossy
    assert sc.is_pushable("jira", "status")       # transition
    assert sc.is_pushable("github", "status")     # binary
    assert sc.is_pushable("linear", "description")  # normalizer


def test_is_diffable_only_excludes_na():
    # Manual attachments are still diffable (they show up in buckets).
    assert sc.is_diffable("github", "attachments_up")
    assert sc.is_diffable("jira", "attachments_up")
    # n/a is the only non-diffable case.
    assert not sc.is_diffable("github", "priority")


def test_capability_constants_match_design_doc():
    # Sanity: the seven values documented in docs/design/sync.md.
    expected = {"ok", "lossy", "normalizer", "transition",
                "binary", "manual", "n/a"}
    actual = set()
    for caps in sc.TRACKER_CAPS.values():
        actual.update(caps.values())
    assert actual <= expected
