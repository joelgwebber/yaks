"""System clipboard helpers (text + PNG image, macOS + Linux)."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def copy_text(text: str) -> bool:
    """Copy a string to the clipboard. Returns True on success."""
    try:
        if sys.platform == "darwin":
            proc = subprocess.Popen(["pbcopy"], stdin=subprocess.PIPE)
        else:
            proc = subprocess.Popen(
                ["xclip", "-selection", "clipboard"], stdin=subprocess.PIPE)
        proc.communicate(text.encode())
        return True
    except FileNotFoundError:
        return False


def read_png() -> bytes | None:
    """Attempt to read a PNG image from the system clipboard."""
    if sys.platform == "darwin":
        try:
            r = subprocess.run(
                ["osascript", "-e",
                 'try\nset png to (the clipboard as «class PNGf»)\nset fp to open for access '
                 '(POSIX file "/tmp/yak-clip.png") with write permission\nset eof of fp to 0\n'
                 'write png to fp\nclose access fp\nend try'],
                capture_output=True, timeout=5,
            )
            p = Path("/tmp/yak-clip.png")
            if r.returncode == 0 and p.exists() and p.stat().st_size > 0:
                data = p.read_bytes()
                p.unlink()
                return data
        except (FileNotFoundError, subprocess.TimeoutExpired):
            pass
        return None
    try:
        r = subprocess.run(
            ["xclip", "-selection", "clipboard", "-t", "image/png", "-o"],
            capture_output=True, timeout=5,
        )
        if r.returncode == 0 and r.stdout:
            return r.stdout
    except (FileNotFoundError, subprocess.TimeoutExpired):
        pass
    return None
