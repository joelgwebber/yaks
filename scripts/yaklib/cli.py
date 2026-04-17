"""CLI entry point for the `yaks` command (installed via `uv tool install`)."""
from __future__ import annotations

import sys

from yaklib.commands import COMMANDS
from yaklib.parser import build_parser


def main():
    parser = build_parser()
    args = parser.parse_args()
    if not args.command:
        parser.print_help()
        sys.exit(1)

    if args.command == "tui":
        import curses
        from tui import main as tui_main
        try:
            curses.wrapper(tui_main)
        except KeyboardInterrupt:
            pass
        return

    COMMANDS[args.command](args)
