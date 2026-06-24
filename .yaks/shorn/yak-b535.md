---
id: yak-b535
title: Crash on file modification via yaks tool
type: bug
priority: 1
created: '2026-06-24T20:51:18Z'
updated: '2026-06-24T20:57:07Z'
---

Here's the stack trace. I *think* this happened when an agent modified (removed?) a yak file from underneath it. It *should* have been using the yaks cli tool, but it may have gone rogue and moved the file directly. Either way, the `yaks tui` shouldn't crash.

Traceback (most recent call last):
  File "/Users/joel/.local/bin/yaks", line 10, in <module>
    sys.exit(main())
             ^^^^^^
  File "/Users/joel/.local/share/uv/tools/yaks/lib/python3.12/site-packages/yaklib/cli.py", line 21, in main
    curses.wrapper(tui_main)
  File "/Users/joel/.local/share/uv/python/cpython-3.12.11-macos-aarch64-none/lib/python3.12/curses/__init__.py", line 94, in wrapper
    return func(stdscr, *args, **kwds)
           ^^^^^^^^^^^^^^^^^^^^^^^^^^^
  File "/Users/joel/.local/share/uv/tools/yaks/lib/python3.12/site-packages/tui.py", line 1196, in main
    tui.run()
  File "/Users/joel/.local/share/uv/tools/yaks/lib/python3.12/site-packages/tui.py", line 464, in run
    self._check_fs_changes()
  File "/Users/joel/.local/share/uv/tools/yaks/lib/python3.12/site-packages/tui.py", line 393, in _check_fs_changes
    self._reload_preserving_position()
  File "/Users/joel/.local/share/uv/tools/yaks/lib/python3.12/site-packages/tui.py", line 359, in _reload_preserving_position
    self.reload()
  File "/Users/joel/.local/share/uv/tools/yaks/lib/python3.12/site-packages/tui.py", line 274, in reload
    self._refresh_task_cache()
  File "/Users/joel/.local/share/uv/tools/yaks/lib/python3.12/site-packages/tui.py", line 349, in _refresh_task_cache
    for st, t in _all_tasks(self.root, s):
                 ^^^^^^^^^^^^^^^^^^^^^^^^
  File "/Users/joel/.local/share/uv/tools/yaks/lib/python3.12/site-packages/yaklib/model.py", line 231, in all_tasks
    task = load_task(f)
           ^^^^^^^^^^^^
  File "/Users/joel/.local/share/uv/tools/yaks/lib/python3.12/site-packages/yaklib/model.py", line 188, in load_task
    text = path.read_text()
           ^^^^^^^^^^^^^^^^
  File "/Users/joel/.local/share/uv/python/cpython-3.12.11-macos-aarch64-none/lib/python3.12/pathlib.py", line 1027, in read_text
    with self.open(mode='r', encoding=encoding, errors=errors) as f:
         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  File "/Users/joel/.local/share/uv/python/cpython-3.12.11-macos-aarch64-none/lib/python3.12/pathlib.py", line 1013, in open
    return io.open(self, mode, buffering, encoding, errors, newline)
           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
FileNotFoundError: [Errno 2] No such file or directory: '/Users/joel/src/fs/mn/projects/fullstory/.yaks/hairy/fullstory-e57c.20.md'

---
▸ 2026-06-24T20:57:03Z
Root cause: TOCTOU race in model.all_tasks. It globs *.md then load_task()->read_text() each; if an agent moves/removes a yak file directly between the glob and the read, read_text raises FileNotFoundError and crashes the whole 'yaks tui' FS-poll reload (_check_fs_changes -> reload -> _refresh_task_cache -> all_tasks). Fix: wrap the per-file load_task in try/except (OSError, yaml.YAMLError) and skip vanished/mid-write files; the next reload sees settled state. Broadened to YAMLError so a half-written file can't crash the scan either. Added test_all_tasks_skips_files_that_vanish_mid_scan (monkeypatches load_task to raise FileNotFoundError for one file, asserts survivors load and no exception). 114 tests pass. Bumped plugin 0.1.75->0.1.76 (both manifests).
