"""Interactive sidecar review dialog (bf54.9).

Opens on `~` over a yak that has a `.yaks/.sync-pending/<id>.yaml` sidecar.
Lets the user toggle each field row's resolution, then either apply
locally (writes the yak, stamps `last_synced`, clears the sidecar) or
discard the plan.

Layout: top half is a compact field-resolution table; bottom half is a
two-pane Local / Upstream view of the currently-selected field, wrapped
to width so long titles or descriptions don't run off the edge.

Conservative on purpose: comments / attachments and any field that needs
an MCP write or status migration are read-only here. Use ``/yaks:sync``
for those. Free-form editing of upstream text would require pushing the
edit back to the tracker, which is firmly out of scope for the TUI —
the dialog is accept / skip / defer only.
"""

from __future__ import annotations

import curses
import textwrap

from yaklib import sync as _sync
from yaklib.model import find_task_file, load_task, now_iso, save_task
from yaktui.colors import C_HEADER, C_HELP, C_LABEL, C_SEARCH, C_SELECTED
from yaktui.dialogs import confirm, safe_addstr


def _next_resolution(current: str) -> str:
    """Cycle approve → skip → pending → approve. ``auto`` cycles into
    ``skip`` so a user who didn't like a silent default can opt out."""
    order = ("approve", "skip", "pending")
    if current == "auto":
        return "skip"
    try:
        i = order.index(current)
    except ValueError:
        return "approve"
    return order[(i + 1) % len(order)]


def _preview(value) -> str:
    """One-line rendering of a field value (used in the compact table)."""
    if value is None:
        return "(unset)"
    if isinstance(value, list):
        return ", ".join(str(v) for v in value) or "(empty)"
    s = str(value).replace("\n", " ⏎ ")
    return s


def _full_text(value) -> str:
    """Multi-line rendering for the detail panel."""
    if value is None:
        return "(unset)"
    if isinstance(value, list):
        if not value:
            return "(empty)"
        return "\n".join(f"- {v}" for v in value)
    return str(value)


def _wrap(text: str, width: int) -> list[str]:
    """Wrap *text* preserving paragraph breaks. Returns a list of lines."""
    if width <= 0:
        return [text]
    lines: list[str] = []
    for para in text.splitlines() or [""]:
        if not para:
            lines.append("")
            continue
        wrapped = textwrap.wrap(para, width=width,
                                break_long_words=False,
                                break_on_hyphens=False) or [""]
        lines.extend(wrapped)
    return lines


def _draw_table(stdscr, fields, sel, y, max_y, w):
    """Draw the compact resolution table. Returns the next free y."""
    if not fields:
        safe_addstr(stdscr, y, 0, "(no field changes)", curses.A_DIM)
        return y + 1
    safe_addstr(stdscr, y, 0,
                "Field            Dir       Res       Local → Upstream",
                curses.color_pair(C_HEADER) | curses.A_BOLD)
    y += 1
    for i, f in enumerate(fields):
        if y >= max_y:
            break
        name = str(f.get("name", "?"))
        direction = str(f.get("direction", "?"))
        resolution = str(f.get("resolution", "?"))
        local = _preview(f.get("local"))
        upstream = _preview(f.get("upstream"))
        is_sel = i == sel
        row_attr = curses.color_pair(C_SELECTED) | curses.A_BOLD if is_sel else 0
        safe_addstr(stdscr, y, 0, " " * w, row_attr)
        safe_addstr(stdscr, y, 0, f" {name:<15}", row_attr)
        safe_addstr(stdscr, y, 17, f"{direction:<10}", row_attr)
        if resolution == "pending":
            res_attr = curses.color_pair(C_SEARCH) | curses.A_BOLD
            if is_sel:
                res_attr |= curses.A_REVERSE
        else:
            res_attr = row_attr
        safe_addstr(stdscr, y, 28, f"{resolution:<10}", res_attr)
        rest = f"{local} → {upstream}"
        safe_addstr(stdscr, y, 39, rest[:max(1, w - 40)], row_attr)
        y += 1
    return y


def _draw_diff_panel(stdscr, field, y_start, height, w):
    """Two-column Local / Upstream view of a single field, wrapped."""
    if height < 3 or field is None:
        return
    safe_addstr(stdscr, y_start, 0, "─" * w, curses.A_DIM)
    panel_y = y_start + 1
    panel_h = height - 1

    col_w = max(10, (w - 3) // 2)
    left_x = 0
    right_x = col_w + 3

    safe_addstr(stdscr, panel_y, left_x,
                f"Local — {field.get('name', '?')}",
                curses.color_pair(C_LABEL) | curses.A_BOLD)
    safe_addstr(stdscr, panel_y, right_x,
                f"Upstream — {field.get('name', '?')}",
                curses.color_pair(C_LABEL) | curses.A_BOLD)
    # Vertical divider.
    for yy in range(panel_y, panel_y + panel_h):
        try:
            stdscr.addch(yy, col_w + 1, curses.ACS_VLINE, curses.A_DIM)
        except curses.error:
            pass

    local_lines = _wrap(_full_text(field.get("local")), col_w)
    upstream_lines = _wrap(_full_text(field.get("upstream")), col_w)
    body_h = panel_h - 1
    for i in range(body_h):
        yy = panel_y + 1 + i
        if i < len(local_lines):
            safe_addstr(stdscr, yy, left_x, local_lines[i][:col_w], 0)
        if i < len(upstream_lines):
            safe_addstr(stdscr, yy, right_x, upstream_lines[i][:col_w], 0)


def open_review(app, yak_id: str) -> None:
    """Open the sidecar review dialog for *yak_id*. No-op if no sidecar."""
    sidecar_path = _sync.sidecar_path(app.root, yak_id)
    if not sidecar_path.exists():
        app.notification = f"no sidecar for {yak_id}"
        return

    sidecar = _sync.load_sidecar(sidecar_path)
    fields = list(sidecar.get("fields") or [])

    buckets = [(k, len(sidecar.get(k) or []))
               for k in ("comments_up", "comments_down",
                         "attachments_up", "attachments_down")]
    has_buckets = any(n for _, n in buckets)

    res = find_task_file(app.root, yak_id)
    if not res:
        app.notification = f"{yak_id} not found"
        return
    _, yak_path = res
    yak = load_task(yak_path)

    sel = 0
    message = ""

    while True:
        h, w = app.stdscr.getmaxyx()
        app.stdscr.erase()

        title = f"Sync review: {yak_id} — {yak.get('title', '')}"
        safe_addstr(app.stdscr, 0, 0, title[:w],
                    curses.color_pair(C_HEADER) | curses.A_BOLD)
        src = sidecar.get("source") or yak.get("source") or ""
        if src:
            safe_addstr(app.stdscr, 1, 0, src[:w], curses.A_DIM)

        # Reserve space: title (1) + source (1) + blank (1) +
        # buckets summary (variable) + message (1) + footer (1).
        bucket_lines = (1 + sum(1 for _, n in buckets if n) + 1
                        if has_buckets else 0)
        chrome = 3 + bucket_lines + (1 if message else 0) + 1

        # Split remaining space: ~40% table, ~60% diff panel.
        avail = max(6, h - chrome)
        table_max_h = min(len(fields) + 1 + 1, max(3, avail // 2))
        table_y0 = 3
        table_y_end = _draw_table(app.stdscr, fields, sel,
                                  table_y0, table_y0 + table_max_h, w)

        # Diff panel below the table.
        panel_y = table_y_end + 1
        panel_h = h - chrome - (table_y_end - table_y0) - 1
        if fields and 0 <= sel < len(fields) and panel_h > 2:
            _draw_diff_panel(app.stdscr, fields[sel], panel_y, panel_h, w)
            y = panel_y + panel_h
        else:
            y = panel_y

        if has_buckets:
            safe_addstr(app.stdscr, y, 0,
                        "Read-only here — apply via /yaks:sync:",
                        curses.color_pair(C_HEADER) | curses.A_BOLD)
            y += 1
            for name, n in buckets:
                if not n or y >= h - 2:
                    continue
                safe_addstr(app.stdscr, y, 2, f"{name}: {n}",
                            curses.color_pair(C_LABEL))
                y += 1

        if message:
            safe_addstr(app.stdscr, h - 2, 0, message[:w],
                        curses.color_pair(C_SEARCH) | curses.A_BOLD)

        footer = ("space/Enter: cycle  s: skip  p: pending  "
                  "A: apply  D: discard  q/Esc: leave")
        safe_addstr(app.stdscr, h - 1, 0, footer[:w],
                    curses.color_pair(C_HELP))

        app.stdscr.refresh()
        key = app.stdscr.getch()
        if key == -1:
            continue
        if key in (ord("q"), 27):
            sidecar["fields"] = fields
            _sync.save_sidecar(sidecar_path, sidecar)
            app.notification = f"sidecar for {yak_id} left as-is"
            return
        if key in (ord("j"), curses.KEY_DOWN):
            if fields:
                sel = min(len(fields) - 1, sel + 1)
        elif key in (ord("k"), curses.KEY_UP):
            sel = max(0, sel - 1)
        elif key in (ord(" "), ord("\n"), curses.KEY_ENTER, 10, 13):
            if fields and 0 <= sel < len(fields):
                cur = fields[sel].get("resolution", "pending")
                fields[sel]["resolution"] = _next_resolution(cur)
                message = ""
        elif key == ord("s"):
            if fields and 0 <= sel < len(fields):
                fields[sel]["resolution"] = "skip"
                message = ""
        elif key == ord("p"):
            if fields and 0 <= sel < len(fields):
                fields[sel]["resolution"] = "pending"
                message = ""
        elif key == ord("D"):
            if confirm(app.stdscr,
                       f"Discard sidecar for {yak_id}? (y/N): "):
                _sync.clear_sidecar(app.root, yak_id)
                app.reload()
                app.notification = f"discarded sidecar for {yak_id}"
                return
            message = "discard cancelled"
        elif key == ord("A"):
            sidecar["fields"] = fields
            try:
                new_yak = _sync.apply_sidecar_locally(yak, sidecar)
            except _sync.SidecarApplyError as e:
                message = f"can't apply locally: {e}"
                continue
            new_yak["updated"] = now_iso()
            new_yak["last_synced"] = new_yak["updated"]
            save_task(yak_path, new_yak)
            _sync.clear_sidecar(app.root, yak_id)
            app.reload()
            app.notification = f"applied sidecar for {yak_id}"
            return
