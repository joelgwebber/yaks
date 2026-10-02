//! System clipboard helpers (text + PNG image) via `arboard`.
//!
//! Best-effort and cross-platform: calls return a bool/Option so the TUI
//! degrades gracefully (a notification) when no clipboard is available (e.g. a
//! headless session or a machine with no display server).
//!
//! One `arboard::Clipboard` is kept for the life of the process. On Linux
//! (X11, and Wayland via XWayland) the clipboard is not a buffer: the *owning
//! process* serves its contents on request, so dropping the handle right after
//! `set_text` silently loses the copy (yaks-ebd0). Call [`release`] on shutdown
//! so arboard can hand the contents off to a clipboard manager.
//!
//! arboard's `wayland-data-control` feature is enabled so native Wayland
//! sessions get the real Wayland clipboard; without it arboard only speaks X11,
//! which never reaches Wayland apps on compositors whose XWayland bridge doesn't
//! sync the clipboard (e.g. niri + xwayland-satellite).

use std::sync::{Mutex, MutexGuard};

static CLIPBOARD: Mutex<Option<arboard::Clipboard>> = Mutex::new(None);

fn lock() -> MutexGuard<'static, Option<arboard::Clipboard>> {
    CLIPBOARD.lock().unwrap_or_else(|e| e.into_inner())
}

/// Run `f` against the shared clipboard, opening it on first use. `None` when
/// no clipboard is available (a later call retries).
fn with_clipboard<T>(f: impl FnOnce(&mut arboard::Clipboard) -> Option<T>) -> Option<T> {
    let mut guard = lock();
    if guard.is_none() {
        *guard = arboard::Clipboard::new().ok();
    }
    f(guard.as_mut()?)
}

/// Copy `text` to the system clipboard. Returns `true` on success.
pub fn copy_text(text: &str) -> bool {
    with_clipboard(|cb| cb.set_text(text.to_string()).ok()).is_some()
}

/// Drop the shared clipboard handle. On Linux this is when arboard offers the
/// last copy to a clipboard manager, so it survives yaks exiting (if one runs).
pub fn release() {
    lock().take();
}

/// Read an image from the clipboard and encode it as PNG bytes. Returns `None`
/// when the clipboard holds no image or no clipboard is available. Used to
/// paste a screenshot straight onto a yak (artifact attach).
pub fn read_png() -> Option<Vec<u8>> {
    let img = with_clipboard(|cb| cb.get_image().ok())?;
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, img.width as u32, img.height as u32);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().ok()?;
        writer.write_image_data(&img.bytes).ok()?;
    }
    Some(out)
}
