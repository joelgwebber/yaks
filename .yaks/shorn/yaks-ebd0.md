---
id: yaks-ebd0
title: Copying to clipboard fails silently on linux
type: task
priority: 1
created: '2026-10-02T05:05:36Z'
updated: '2026-10-02T05:09:00Z'
labels:
- linux
---

Not sure why, but copying to the clipboard doesn't seem to work on linux. It thinks it copied a value, but nothing ends up in the system clipboard.

Sometimes it fails with this:
```
Clipboard was dropped very quickly after writing (100ms); clipboard managers may not have seen the contents.
Consider keeping `Clipboard` in more persistent state somewhere or keeping the contents alive longer using `SetLinuxExt` and/or threads.
```

---
▸ 2026-10-02T05:09:00Z [Joel Webber]
Root cause, two layers:
1. src/clipboard.rs::copy_text created an arboard::Clipboard, set_text, and dropped it immediately. On Linux the owning process serves clipboard contents, so the copy died with the handle (set_text still returned Ok -> 'copied' notification; arboard's 'dropped very quickly' warning). Fix: one process-lifetime Clipboard in a static Mutex<Option<_>>, released explicitly at TUI teardown (runtime::run) so arboard can hand off to a clipboard manager.
2. arboard was built without 'wayland-data-control', so it only used X11 (:0). On niri (xwayland-satellite) that never reached the Wayland clipboard: even with the persistent handle, wl-paste still returned the old contents. Enabled the feature in Cargo.toml; wl-clipboard-rs is target-gated to linux/bsd, so mac/windows release builds are unaffected.
edtui's vim yank already held a persistent ArboardClipboard; it gets the Wayland feature through the shared arboard crate.
Evidence (niri, Wayland): scratch test copy_text -> wl-paste: before = old contents, after = copied value. End-to-end: seeded wl-copy 'stale', ran target/debug/yaks tui in a pty, pressed y -> 'copied yaks-ebd0' notification, quit -> wl-paste = 'yaks-ebd0' (survives exit). cargo test: 319+28 pass. No docs/help change: behavior fix, no new keys/flags.
