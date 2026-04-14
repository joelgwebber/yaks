# /// script
# requires-python = ">=3.10"
# dependencies = ["pyyaml>=6.0"]
# ///
"""Filesystem-native task tracker. Markdown files with YAML frontmatter, no database, no daemon.

This file is an entry shim. The actual implementation lives under scripts/yaklib/
(model, commands, parser, deps, artifacts, clipboard, format). Back-compat
re-exports are preserved here so tui.py's `import yak; yak.X` usage keeps
working until step .8 of the refactor switches tui.py over to yaklib directly.
"""

from __future__ import annotations

import subprocess as _sp
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

# Back-compat re-exports (see tui.py).
from yaklib import deps as deps_mod  # noqa: E402,F401
from yaklib.artifacts import artifacts_dir, parse_artifacts  # noqa: E402,F401
from yaklib.clipboard import read_png as read_clipboard_png  # noqa: E402,F401
from yaklib.commands import COMMANDS  # noqa: E402
from yaklib.model import (  # noqa: E402,F401
    ALL_STATUS_NAMES,
    DEAD,
    HAIRY,
    SHAVING,
    SHORN,
    STATUSES,
    _ALL_STATUSES,
    _BlockScalarDumper,
    _STATUS_ALIASES,
    _auto_migrate,
    all_tasks,
    dump_yaml,
    find_children,
    find_descendants,
    find_task_file,
    find_tasks_root,
    generate_id,
    git_head_short,
    load_config,
    load_task,
    next_child_number,
    now_iso,
    parent_id,
    resolve_status,
    save_task,
)
from yaklib.parser import build_parser  # noqa: E402

_resolve_status = resolve_status


def main():
    parser = build_parser()
    args = parser.parse_args()
    if not args.command:
        parser.print_help()
        sys.exit(1)

    if args.command == "tui":
        _sp.execvp(sys.executable, [sys.executable, str(Path(__file__).parent / "tui.py")])

    COMMANDS[args.command](args)


if __name__ == "__main__":
    main()
