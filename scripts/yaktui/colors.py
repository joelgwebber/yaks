"""Curses color pair constants + init_colors + per-status badge helper."""

from __future__ import annotations

import curses

from yaklib.model import HAIRY, SHORN

C_ID = 1
C_P1 = 2
C_P2 = 3
C_P3 = 4
C_TAB_ACTIVE = 5
C_GHOST = 6
C_TYPE = 7
C_SELECTED = 8
C_LABEL = 9
C_HEADER = 10
C_HELP = 11
C_SEARCH = 12
C_LINK = 13
C_LINK_SEL = 14
C_MATCH = 15
C_GHOST_HAIRY = 16
C_GHOST_SHAVING = 17
C_GHOST_SHORN = 18
C_CODE = 19
C_MD_HEADING = 20
C_P4 = 21
C_P5 = 22


def init_colors():
    curses.use_default_colors()
    curses.init_pair(C_ID, curses.COLOR_BLUE, -1)
    # 5-level priority palette: P1 Urgent, P2 High, P3 Medium, P4 Low, P5 Lowest.
    curses.init_pair(C_P1, curses.COLOR_RED, -1)
    curses.init_pair(C_P2, curses.COLOR_MAGENTA, -1)
    curses.init_pair(C_P3, curses.COLOR_YELLOW, -1)
    curses.init_pair(C_P4, curses.COLOR_GREEN, -1)
    curses.init_pair(C_P5, curses.COLOR_BLUE, -1)
    curses.init_pair(C_TAB_ACTIVE, curses.COLOR_BLACK, curses.COLOR_WHITE)
    curses.init_pair(C_GHOST, 8, -1)
    curses.init_pair(C_TYPE, curses.COLOR_CYAN, -1)
    if curses.COLORS >= 256:
        curses.init_pair(C_SELECTED, -1, 237)
        curses.init_pair(C_LINK_SEL, curses.COLOR_BLUE, 237)
    else:
        curses.init_pair(C_SELECTED, curses.COLOR_WHITE, curses.COLOR_BLUE)
        curses.init_pair(C_LINK_SEL, curses.COLOR_CYAN, curses.COLOR_BLUE)
    curses.init_pair(C_LABEL, curses.COLOR_MAGENTA, -1)
    curses.init_pair(C_HEADER, curses.COLOR_WHITE, -1)
    curses.init_pair(C_HELP, curses.COLOR_BLACK, curses.COLOR_WHITE)
    curses.init_pair(C_SEARCH, curses.COLOR_YELLOW, -1)
    curses.init_pair(C_LINK, curses.COLOR_BLUE, -1)
    curses.init_pair(C_MATCH, curses.COLOR_BLACK, curses.COLOR_YELLOW)
    curses.init_pair(C_GHOST_HAIRY, curses.COLOR_YELLOW, -1)
    curses.init_pair(C_GHOST_SHAVING, 8, -1)
    curses.init_pair(C_GHOST_SHORN, curses.COLOR_GREEN, -1)
    curses.init_pair(C_CODE, curses.COLOR_CYAN, -1)
    curses.init_pair(C_MD_HEADING, curses.COLOR_WHITE, -1)


def ghost_badge_attr(status):
    """Return the color pair + attributes for a ghost badge of the given state."""
    if status == HAIRY:
        return curses.color_pair(C_GHOST_HAIRY) | curses.A_BOLD
    if status == SHORN:
        return curses.color_pair(C_GHOST_SHORN)
    return curses.color_pair(C_GHOST_SHAVING) | curses.A_DIM
