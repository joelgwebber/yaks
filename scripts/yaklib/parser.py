"""argparse wiring for the yak CLI."""

from __future__ import annotations

import argparse

from yaklib.model import ALL_STATUS_NAMES


def _add_filter_flags(sp, exclude: tuple = ()) -> None:
    """Attach the unified FilterSpec flags to a subparser."""
    if "status" not in exclude:
        sp.add_argument("--status", choices=ALL_STATUS_NAMES,
                        action="append", help="Filter by status (repeatable)")
    if "type" not in exclude:
        sp.add_argument("--type", action="append",
                        help="Filter by type (repeatable)")
    if "priority" not in exclude:
        sp.add_argument("--priority", type=int, action="append",
                        help="Filter by priority (repeatable)")
    if "label" not in exclude:
        sp.add_argument("--label", action="append",
                        help="Filter by label — OR within labels (repeatable)")
    if "search" not in exclude:
        sp.add_argument("--search", help="Substring match on title/description/id")
    if "ready" not in exclude:
        sp.add_argument("--ready", action="store_true",
                        help="Only tasks whose deps are all resolved")
    if "tangled" not in exclude:
        sp.add_argument("--tangled", action="store_true",
                        help="Only tasks with at least one unresolved dep")
    if "parent" not in exclude:
        sp.add_argument("--parent-of", dest="parent_of",
                        help="Only descendants of the given task ID")


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="yaks", description="Filesystem-native task tracker")
    sub = p.add_subparsers(dest="command")

    sp = sub.add_parser("init", help="Initialize .yaks/ in the current directory")
    sp.add_argument("--prefix", help="Task ID prefix (default: directory name)")
    sp.add_argument("--agents", action="store_true",
                    help="Write guidance to AGENTS.md instead of CLAUDE.md")

    sp = sub.add_parser("create", help="Create a new task")
    sp.add_argument("--title", required=True, help="Task title")
    sp.add_argument("--type", help="Task type (bug, feature, task, idea, etc.)")
    sp.add_argument("--priority", type=int, help="Priority (1=highest)")
    sp.add_argument("--description", help="Task description")
    sp.add_argument("--labels", nargs="+", help="Labels")
    sp.add_argument("--depends-on", nargs="+", help="Dependency task IDs")
    sp.add_argument("--parent", help="Parent task ID (creates a child task)")
    sp.add_argument("--source", help="External issue URL (e.g. Jira, GitHub, Linear)")

    sp = sub.add_parser("list", help="List tasks")
    _add_filter_flags(sp)
    sp.add_argument("--json", action="store_true", help="JSON output")

    sp = sub.add_parser("show", help="Show a task")
    sp.add_argument("id", help="Task ID")
    sp.add_argument("--json", action="store_true", help="JSON output")

    sp = sub.add_parser("update", help="Update a task")
    sp.add_argument("id", help="Task ID")
    sp.add_argument("--title", help="New title")
    sp.add_argument("--type", help="New type")
    sp.add_argument("--priority", type=int, help="New priority")
    sp.add_argument("--description", help="New description")
    sp.add_argument("--add-label", nargs="+", help="Add labels")
    sp.add_argument("--remove-label", nargs="+", help="Remove labels")
    sp.add_argument("--note", help="Append a timestamped progress note to the description")
    sp.add_argument("--source", help="External issue URL")
    sp.add_argument("--last-synced",
                    help="Stamp last_synced timestamp (ISO8601, or 'now'). "
                         "Written by /yaks:sync after a successful merge.")

    for name in ("shave", "work"):
        sp = sub.add_parser(name, help="Start shaving a yak")
        sp.add_argument("id", help="Task ID")

    for name in ("shorn", "close"):
        sp = sub.add_parser(name, help="Mark a yak as shorn")
        sp.add_argument("id", help="Task ID")

    for name in ("regrow", "reopen"):
        sp = sub.add_parser(name, help="Regrow a shorn yak")
        sp.add_argument("id", help="Task ID")

    sp = sub.add_parser("slaughter",
                        help="Slaughter a yak (move to hidden 'dead' state)")
    sp.add_argument("id", help="Task ID")

    sp = sub.add_parser("revive", help="Revive a dead yak (back to hairy)")
    sp.add_argument("id", help="Task ID")

    for name in ("next", "ready"):
        sp = sub.add_parser(name, help="Show yaks ready to shave "
                            "(shortcut for `list --status hairy --ready`)")
        _add_filter_flags(sp, exclude=("ready", "tangled", "status"))
        sp.add_argument("--json", action="store_true", help="JSON output")

    for name in ("tangled", "blocked"):
        sp = sub.add_parser(name, help="Show tangled yaks "
                            "(shortcut for `list --status hairy --tangled`)")
        _add_filter_flags(sp, exclude=("ready", "tangled", "status"))
        sp.add_argument("--json", action="store_true", help="JSON output")

    sp = sub.add_parser("dep", help="Manage dependencies")
    sp.add_argument("action", choices=["add", "remove"], help="Add or remove dependency")
    sp.add_argument("id", help="Task ID")
    sp.add_argument("dep_id", help="Dependency task ID")

    sp = sub.add_parser("reparent", help="Move a task to a new parent or to top-level")
    sp.add_argument("id", help="Task ID to reparent")
    group = sp.add_mutually_exclusive_group(required=True)
    group.add_argument("--parent", help="New parent task ID")
    group.add_argument("--unparent", action="store_true", help="Promote to top-level task")

    sp = sub.add_parser("attach", help="Attach a file (or clipboard image) to a yak")
    sp.add_argument("id", help="Task ID")
    sp.add_argument("path", nargs="?", help="Path to file (omit with --paste)")
    sp.add_argument("--paste", action="store_true", help="Read PNG image from clipboard")
    sp.add_argument("--name", help="Override stored filename")
    sp.add_argument("--desc", help="Alt text / description for the markdown link")
    sp.add_argument("--force", action="store_true", help="Overwrite if file already exists")

    sp = sub.add_parser("detach", help="Detach an artifact from a yak")
    sp.add_argument("id", help="Task ID")
    sp.add_argument("name", help="Artifact filename")

    sp = sub.add_parser("search", help="Search tasks by keyword "
                        "(shortcut for `list --search QUERY`)")
    sp.add_argument("query", help="Search term")
    _add_filter_flags(sp, exclude=("search",))
    sp.add_argument("--json", action="store_true", help="JSON output")

    sp = sub.add_parser("stats", help="Show task statistics")
    sp.add_argument("--json", action="store_true", help="JSON output")

    sub.add_parser("tui", help="Open interactive TUI")

    sp = sub.add_parser("import-beads", help="Import tasks from a beads issues.jsonl file")
    sp.add_argument("--file", help="Path to issues.jsonl (default: auto-detect .beads/issues.jsonl)")
    sp.add_argument("--dry-run", action="store_true",
                    help="Print what would be created without writing")

    return p
