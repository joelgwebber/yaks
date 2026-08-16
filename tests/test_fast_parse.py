"""Fast-path frontmatter parser (yak-3fd4.3).

The fast path must be *indistinguishable* from yaml.safe_load for the subset it
accepts, and must return None (defer to the loader) for everything else. These
tests pin both halves, plus a differential check against the repo's own corpus.
"""

from __future__ import annotations

from pathlib import Path

import yaml
from conftest import REPO_ROOT
from yaklib import model
from yaklib.model import _fast_frontmatter, dump_yaml, load_task, save_task


def _fm(task: dict) -> str:
    """Frontmatter text as the writer would emit it (no description)."""
    t = dict(task)
    t.pop("description", None)
    return dump_yaml(t)


# --- accepted subset: identical to safe_load, and actually fast-pathed --------

ACCEPTED = [
    {"id": "yak-a1b2", "title": "Fix the login crash", "type": "bug", "priority": 2},
    {"id": "yak-c3d4", "title": "(Y)ank in details, on a url", "priority": 5},
    {"id": "yak-e5f6", "created": "2026-08-15T17:25:41Z", "parent": "yak-3fd4"},
    {"id": "yak-1", "depends_on": ["yak-2", "yak-3"], "labels": ["perf", "ui"]},
    {"id": "yak-q", "title": "it's a 'quoted' title: with colon"},
    {"id": "yak-neg", "priority": -1},
    {"id": "yak-one", "labels": ["solo"]},
    {"id": "yak-src", "source": "https://jira.example.com/browse/PROJ-123"},
]


def test_accepted_subset_matches_safe_load_and_is_fast_pathed():
    for task in ACCEPTED:
        fm = _fm(task)
        fast = _fast_frontmatter(fm)
        assert fast is not None, f"expected fast path for:\n{fm}"
        assert fast == yaml.safe_load(fm), f"mismatch for:\n{fm}"


def test_priority_is_int_not_string():
    fast = _fast_frontmatter(_fm({"id": "x", "priority": 3}))
    assert fast["priority"] == 3 and isinstance(fast["priority"], int)


def test_single_quote_escaping():
    # A mid-word apostrophe stays a plain scalar (legal YAML); still parses.
    assert _fast_frontmatter(_fm({"id": "x", "title": "it's fine"}))["title"] == "it's fine"
    # A leading quote forces single-quoted style with '' escaping.
    fm = _fm({"id": "x", "title": "'lead' and it's"})
    assert "''" in fm
    assert _fast_frontmatter(fm)["title"] == "'lead' and it's"


# --- bail cases: fast path returns None, fallback still equals safe_load ------

BAIL_CASES = {
    "double_quoted": 'id: x\ntitle: "quoted"\n',
    "flow_list": "id: x\nlabels: [a, b]\n",
    "flow_map": "id: x\nmeta: {a: 1}\n",
    "block_scalar": "id: x\nnote: |\n  line one\n  line two\n",
    "wrapped_plain": ("id: x\ntitle: a very long title that PyYAML folds across "
                      "the default width boundary\n  onto a second line\n"),
    "nested_map": "id: x\nchild:\n  a: 1\n",
    "plain_bool": "id: x\nflag: yes\n",
    "plain_null": "id: x\nthing: null\n",
    "plain_float": "id: x\nratio: 1.5\n",
    "plain_date": "id: x\nwhen: 2026-08-15\n",
    "trailing_comment": "id: x\ntitle: value # comment\n",
    "exotic_int": "id: x\nn: 1_000\n",
    "bare_key_null": "id: x\nempty:\n",
}


def test_bail_cases_defer_and_fallback_is_correct():
    for name, fm in BAIL_CASES.items():
        assert _fast_frontmatter(fm) is None, f"{name} should have bailed:\n{fm}"
        # load_task's fallback path must reproduce safe_load exactly.
        assert (model._yaml_load(fm) or {}) == (yaml.safe_load(fm) or {}), name


def test_empty_frontmatter_is_empty_dict():
    assert _fast_frontmatter("") == {}
    assert _fast_frontmatter("\n") == {}


# --- integration: save_task -> load_task round-trip ---------------------------

def test_load_task_roundtrip(tmp_path: Path):
    task = {
        "id": "yak-rt", "title": "round: trip's edge", "type": "task",
        "priority": 4, "created": "2026-08-16T00:00:00Z",
        "parent": "yak-root", "depends_on": ["yak-a", "yak-b"],
        "labels": ["perf"], "description": "Body text\nsecond line.",
    }
    p = tmp_path / "yak-rt.md"
    save_task(p, task)
    assert load_task(p) == task


# --- differential against the repo's own corpus -------------------------------

def _corpus_frontmatters():
    yaks_dir = REPO_ROOT / ".yaks"
    for md in yaks_dir.glob("*/*.md"):
        text = md.read_text()
        if not text.startswith("---"):
            continue
        end = text.find("\n---", 3)
        if end < 0:
            continue
        yield md, text[4:end]


def test_corpus_fast_path_matches_safe_load():
    total = fast = 0
    for md, fm in _corpus_frontmatters():
        total += 1
        parsed = _fast_frontmatter(fm)
        expected = yaml.safe_load(fm)
        if parsed is not None:
            fast += 1
            assert parsed == expected, f"fast-path mismatch in {md.name}:\n{fm}"
    assert total > 0, "no corpus tasks found — is this running in the repo?"
    # The vast majority of real tasks should hit the fast path; if this drops,
    # the writer's shape and the parser's subset have drifted apart.
    assert fast / total > 0.9, f"only {fast}/{total} tasks used the fast path"
