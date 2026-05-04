"""Interactive sidecar review dialog (bf54.9 / bf54.11).

Opens on `~` over a yak that has a `.yaks/.sync-pending/<id>.yaml` sidecar.
Lets the user toggle each row's resolution and direction, then either
apply the local-direction items locally (writes the yak, stamps
`last_synced`, clears the sidecar if all items resolved) or discard the
plan.

Layout: top is a unified row table (fields + bucket items); bottom is a
two-pane Local / Upstream view of the selected row, wrapped to width.

Conservative on purpose: status migrations and any upstream MCP write
require the agent-driven `/yaks:sync` flow. The TUI persists the user's
resolutions verbatim into the sidecar; the skill picks them up and
executes upstream writes on the next /yaks:sync invocation.
"""

from __future__ import annotations

import curses
import textwrap
from dataclasses import dataclass

from yaklib import sync as _sync
from yaklib import sync_caps as _caps
from yaklib.model import find_task_file, load_task, now_iso, save_task
from yaktui.colors import (
    C_GHOST,
    C_HEADER,
    C_HELP,
    C_LABEL,
    C_SEARCH,
    C_SELECTED,
)
from yaktui.dialogs import confirm, safe_addstr


_BUCKETS = ("comments_up", "comments_down", "attachments_up", "attachments_down")


@dataclass
class Row:
    """One navigable row in the dialog — field, comment, or attachment."""
    kind: str            # "field" | "comments_up" | "comments_down" | ...
    item: dict           # the underlying dict from the sidecar
    bucket_idx: int = 0  # 1-based index within its bucket (for display)
    bucket_n: int = 1    # bucket size (for display)


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


def _next_direction(current: str) -> str:
    """Cycle upstream → local → pending → upstream."""
    order = ("upstream", "local", "pending")
    try:
        i = order.index(current)
    except ValueError:
        return "upstream"
    return order[(i + 1) % len(order)]


def _preview(value) -> str:
    """One-line rendering of a field value."""
    if value is None:
        return "(unset)"
    if isinstance(value, list):
        return ", ".join(str(v) for v in value) or "(empty)"
    return str(value).replace("\n", " ⏎ ")


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


def _bucket_label(kind: str) -> str:
    return {
        "comments_up": "comment↑",
        "comments_down": "comment↓",
        "attachments_up": "attach↑",
        "attachments_down": "attach↓",
    }.get(kind, kind)


def _bucket_capability(tracker: str, kind: str) -> str:
    """Capability for a bucket on the given tracker. Down-buckets are
    always local writes (safe), so they're rendered as ``ok``."""
    if kind in ("comments_down", "attachments_down"):
        return _caps.OK
    field = "comments_up" if kind == "comments_up" else "attachments_up"
    return _caps.push_capability(tracker, field)


def _row_capability(row: Row, tracker: str) -> str:
    """Resolve the capability for a row, preferring sidecar-stamped value."""
    if row.kind == "field":
        cap = row.item.get("capability")
        if cap:
            return cap
        return _caps.push_capability(tracker, row.item.get("name") or "")
    return _bucket_capability(tracker, row.kind)


def _build_rows(sidecar: dict) -> list[Row]:
    """Flatten sidecar fields + buckets into a single navigable row list."""
    rows: list[Row] = []
    for f in sidecar.get("fields") or []:
        rows.append(Row(kind="field", item=f))
    for bucket_name in _BUCKETS:
        items = sidecar.get(bucket_name) or []
        for i, item in enumerate(items):
            rows.append(Row(kind=bucket_name, item=item,
                            bucket_idx=i + 1, bucket_n=len(items)))
    return rows


def _row_label(row: Row) -> str:
    if row.kind == "field":
        return str(row.item.get("name", "?"))
    return f"{_bucket_label(row.kind)} {row.bucket_idx}/{row.bucket_n}"


def _row_summary(row: Row) -> str:
    if row.kind == "field":
        local = _preview(row.item.get("local"))
        upstream = _preview(row.item.get("upstream"))
        return f"{local} → {upstream}"
    if row.kind in ("comments_up", "comments_down"):
        body = (row.item.get("body") or "").replace("\n", " ⏎ ")
        author = row.item.get("author")
        prefix = f"@{author}: " if author else ""
        return f"{prefix}{body}"[:200]
    # attachments
    fn = row.item.get("filename") or "?"
    size = row.item.get("size")
    return f"{fn} ({size}b)" if size else fn


def _draw_table(stdscr, rows, sel, y, max_y, w):
    """Draw the unified row table. Returns the next free y."""
    if not rows:
        safe_addstr(stdscr, y, 0, "(no items)", curses.A_DIM)
        return y + 1

    header = "Item              Dir       Res       Cap          Detail"
    safe_addstr(stdscr, y, 0, header[:w],
                curses.color_pair(C_HEADER) | curses.A_BOLD)
    y += 1

    for i, row in enumerate(rows):
        if y >= max_y:
            break
        is_sel = i == sel
        row_attr = curses.color_pair(C_SELECTED) | curses.A_BOLD if is_sel else 0
        safe_addstr(stdscr, y, 0, " " * w, row_attr)

        label = _row_label(row)
        if row.kind == "field":
            direction = str(row.item.get("direction", "?"))
            resolution = str(row.item.get("resolution", "?"))
        else:
            direction = "—"
            resolution = str(row.item.get("resolution", "?"))
        capability = row.item.get("_cap_display", "?")

        safe_addstr(stdscr, y, 0, f" {label:<17}", row_attr)
        safe_addstr(stdscr, y, 19, f"{direction:<10}", row_attr)
        if resolution == "pending":
            res_attr = curses.color_pair(C_SEARCH) | curses.A_BOLD
            if is_sel:
                res_attr |= curses.A_REVERSE
        else:
            res_attr = row_attr
        safe_addstr(stdscr, y, 30, f"{resolution:<10}", res_attr)

        if capability == _caps.OK:
            cap_attr = curses.color_pair(C_GHOST)
            if is_sel:
                cap_attr |= curses.A_BOLD
        else:
            cap_attr = curses.color_pair(C_SEARCH)
            if is_sel:
                cap_attr |= curses.A_REVERSE
        safe_addstr(stdscr, y, 41, f"{capability:<12}", cap_attr)

        safe_addstr(stdscr, y, 54, _row_summary(row)[:max(1, w - 55)], row_attr)
        y += 1

    return y


def _draw_diff_panel(stdscr, row, y_start, height, w):
    """Two-column view of the selected row, wrapped to width."""
    if height < 3 or row is None:
        return
    safe_addstr(stdscr, y_start, 0, "─" * w, curses.A_DIM)
    panel_y = y_start + 1
    panel_h = height - 1

    col_w = max(10, (w - 3) // 2)
    left_x = 0
    right_x = col_w + 3

    if row.kind == "field":
        name = row.item.get("name", "?")
        left_label = f"Local — {name}"
        right_label = f"Upstream — {name}"
        left_text = _full_text(row.item.get("local"))
        right_text = _full_text(row.item.get("upstream"))
    elif row.kind in ("comments_up", "comments_down"):
        direction_label = ("would be posted upstream"
                           if row.kind == "comments_up"
                           else "would be appended to yak body")
        author = row.item.get("author")
        ts = row.item.get("timestamp", "")
        meta_bits = [b for b in (ts, f"@{author}" if author else "") if b]
        left_label = "Comment — " + " ".join(meta_bits) if meta_bits else "Comment"
        right_label = direction_label
        left_text = _full_text(row.item.get("body"))
        right_text = ""
    else:
        # attachments
        direction_label = ("would be uploaded upstream"
                           if row.kind == "attachments_up"
                           else "would be downloaded into .yaks/artifacts/")
        left_label = f"Attachment — {row.item.get('filename', '?')}"
        right_label = direction_label
        meta = []
        if row.item.get("size") is not None:
            meta.append(f"size: {row.item.get('size')} bytes")
        if row.item.get("local_path"):
            meta.append(f"local_path: {row.item.get('local_path')}")
        if row.item.get("upstream_url"):
            meta.append(f"upstream_url: {row.item.get('upstream_url')}")
        left_text = "\n".join(meta) if meta else "(no metadata)"
        right_text = ""

    safe_addstr(stdscr, panel_y, left_x, left_label[:col_w],
                curses.color_pair(C_LABEL) | curses.A_BOLD)
    safe_addstr(stdscr, panel_y, right_x, right_label[:col_w],
                curses.color_pair(C_LABEL) | curses.A_BOLD)
    for yy in range(panel_y, panel_y + panel_h):
        try:
            stdscr.addch(yy, col_w + 1, curses.ACS_VLINE, curses.A_DIM)
        except curses.error:
            pass

    left_lines = _wrap(left_text, col_w)
    right_lines = _wrap(right_text, col_w)
    body_h = panel_h - 1
    for i in range(body_h):
        yy = panel_y + 1 + i
        if i < len(left_lines):
            safe_addstr(stdscr, yy, left_x, left_lines[i][:col_w], 0)
        if i < len(right_lines):
            safe_addstr(stdscr, yy, right_x, right_lines[i][:col_w], 0)


def _writeback_sidecar(sidecar: dict, rows: list[Row]) -> None:
    """Mutate sidecar in place from the row list (rows are dict refs)."""
    sidecar["fields"] = [r.item for r in rows if r.kind == "field"]
    for bucket in _BUCKETS:
        sidecar[bucket] = [r.item for r in rows if r.kind == bucket]


def _stamp_capabilities(rows: list[Row], tracker: str) -> None:
    """Stash the resolved capability on each row item for render. Doesn't
    overwrite a sidecar-stamped value; just mirrors it under a private key
    so the table draw is one lookup."""
    for r in rows:
        cap = _row_capability(r, tracker)
        r.item["_cap_display"] = cap


def _strip_capability_display(sidecar: dict) -> None:
    """Remove the private `_cap_display` key before saving."""
    for f in sidecar.get("fields") or []:
        f.pop("_cap_display", None)
    for bucket in _BUCKETS:
        for item in sidecar.get(bucket) or []:
            item.pop("_cap_display", None)


def _drop_skipped_buckets(sidecar: dict) -> None:
    """Filter resolution=skip items out of every bucket in place."""
    for bucket in _BUCKETS:
        items = sidecar.get(bucket) or []
        sidecar[bucket] = [it for it in items if it.get("resolution") != "skip"]
        if not sidecar[bucket]:
            sidecar.pop(bucket, None)


def open_review(app, yak_id: str) -> None:
    """Open the sidecar review dialog for *yak_id*."""
    sidecar_path = _sync.sidecar_path(app.root, yak_id)
    if not sidecar_path.exists():
        app.notification = f"no sidecar for {yak_id}"
        return

    sidecar = _sync.load_sidecar(sidecar_path)
    notes = [str(n) for n in (sidecar.get("notes") or []) if str(n).strip()]

    res = find_task_file(app.root, yak_id)
    if not res:
        app.notification = f"{yak_id} not found"
        return
    _, yak_path = res
    yak = load_task(yak_path)

    tracker = _sync.tracker_for(sidecar.get("source") or yak.get("source") or "")
    rows = _build_rows(sidecar)
    _stamp_capabilities(rows, tracker)

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

        notes_y0 = 3
        notes_drawn = 0
        for note in notes:
            if notes_y0 + notes_drawn >= h - 6:
                break
            safe_addstr(app.stdscr, notes_y0 + notes_drawn, 0,
                        f"⚠ {note}"[:w],
                        curses.color_pair(C_SEARCH))
            notes_drawn += 1
        notes_block_h = notes_drawn + (1 if notes_drawn else 0)

        chrome = 3 + notes_block_h + (1 if message else 0) + 1
        avail = max(6, h - chrome)
        table_max_h = min(len(rows) + 1 + 1, max(3, avail // 2))
        table_y0 = 3 + notes_block_h
        table_y_end = _draw_table(app.stdscr, rows, sel,
                                  table_y0, table_y0 + table_max_h, w)

        panel_y = table_y_end + 1
        panel_h = h - chrome - (table_y_end - table_y0) - 1
        if rows and 0 <= sel < len(rows) and panel_h > 2:
            _draw_diff_panel(app.stdscr, rows[sel], panel_y, panel_h, w)

        if message:
            safe_addstr(app.stdscr, h - 2, 0, message[:w],
                        curses.color_pair(C_SEARCH) | curses.A_BOLD)

        footer = ("space/Enter: cycle res  d: cycle dir  s: skip  p: pending  "
                  "[/]: jump bucket  A: apply  D: discard  q/Esc: leave")
        safe_addstr(app.stdscr, h - 1, 0, footer[:w],
                    curses.color_pair(C_HELP))

        app.stdscr.refresh()
        key = app.stdscr.getch()
        if key == -1:
            continue

        if key in (ord("q"), 27):
            _writeback_sidecar(sidecar, rows)
            _strip_capability_display(sidecar)
            _sync.save_sidecar(sidecar_path, sidecar)
            app.notification = f"sidecar for {yak_id} left as-is"
            return

        if key in (ord("j"), curses.KEY_DOWN):
            if rows:
                sel = min(len(rows) - 1, sel + 1)
        elif key in (ord("k"), curses.KEY_UP):
            sel = max(0, sel - 1)
        elif key == ord("["):
            # Jump to previous bucket boundary (or first row).
            sel = _jump_bucket(rows, sel, -1)
        elif key == ord("]"):
            sel = _jump_bucket(rows, sel, +1)
        elif key in (ord(" "), ord("\n"), curses.KEY_ENTER, 10, 13):
            if rows and 0 <= sel < len(rows):
                cur = rows[sel].item.get("resolution", "pending")
                rows[sel].item["resolution"] = _next_resolution(cur)
                message = ""
        elif key == ord("s"):
            if rows and 0 <= sel < len(rows):
                rows[sel].item["resolution"] = "skip"
                message = ""
        elif key == ord("p"):
            if rows and 0 <= sel < len(rows):
                rows[sel].item["resolution"] = "pending"
                message = ""
        elif key == ord("d"):
            if rows and 0 <= sel < len(rows):
                row = rows[sel]
                if row.kind != "field":
                    message = "direction is field-only"
                else:
                    cap = row.item.get("_cap_display", _caps.OK)
                    cur = row.item.get("direction", "pending")
                    nxt = _next_direction(cur)
                    if nxt == "local" and cap == _caps.NA:
                        message = (f"{row.item.get('name')}: not pushable "
                                   f"to {tracker} (n/a)")
                    else:
                        row.item["direction"] = nxt
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
            _writeback_sidecar(sidecar, rows)
            _strip_capability_display(sidecar)
            result = _sync.apply_sidecar_locally(yak, sidecar)

            if not result.applied and not result.skipped:
                # Nothing was actionable — don't churn the yak. Tell
                # the user what's blocking and stay in the dialog.
                _stamp_capabilities(rows, tracker)
                if result.deferred or result.bucket_remaining:
                    message = ("nothing to apply locally; "
                               f"{len(result.deferred) + result.bucket_remaining}"
                               " items need /yaks:sync")
                else:
                    message = "nothing to apply"
                continue

            if result.applied:
                result.new_yak["updated"] = now_iso()
                result.new_yak["last_synced"] = result.new_yak["updated"]
                save_task(yak_path, result.new_yak)

            sidecar["fields"] = result.deferred
            _drop_skipped_buckets(sidecar)
            remaining = len(result.deferred) + result.bucket_remaining

            if remaining == 0:
                _sync.clear_sidecar(app.root, yak_id)
            else:
                _sync.save_sidecar(sidecar_path, sidecar)

            app.reload()
            bits = []
            if result.applied:
                bits.append(f"{len(result.applied)} applied locally")
            if result.skipped:
                bits.append(f"{len(result.skipped)} skipped")
            if remaining:
                bits.append(f"{remaining} need /yaks:sync")
            app.notification = f"{yak_id}: " + "; ".join(bits)
            return


def _jump_bucket(rows: list[Row], sel: int, delta: int) -> int:
    """Move sel to the next/prev row whose kind differs from the current."""
    if not rows:
        return sel
    cur_kind = rows[sel].kind
    if delta > 0:
        for j in range(sel + 1, len(rows)):
            if rows[j].kind != cur_kind:
                return j
        return len(rows) - 1
    for j in range(sel - 1, -1, -1):
        if rows[j].kind != cur_kind:
            # Jump to the first row of that previous group.
            target_kind = rows[j].kind
            while j > 0 and rows[j - 1].kind == target_kind:
                j -= 1
            return j
    return 0
