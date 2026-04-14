# /// script
# requires-python = ">=3.10"
# dependencies = ["pyyaml>=6.0"]
# ///
"""Filesystem-native task tracker — CLI entry point.

The implementation lives under scripts/yaklib/ (model, commands, parser,
deps, artifacts, clipboard, format). This file wires argparse to the
dispatch table and is the only thing the plugin invokes directly.
"""

from __future__ import annotations

import subprocess as _sp
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from yaklib.commands import COMMANDS  # noqa: E402
from yaklib.parser import build_parser  # noqa: E402


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
