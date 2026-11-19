---
id: yaks-93ee
title: Browse and link into archive attachments (zip, tar.gz) read-only
status: shaving
type: idea
priority: 4
created: '2026-11-17T21:41:19Z'
updated: '2026-11-19T02:34:18Z'
depends_on:
- yaks-84fe
tags:
- tui
- doks
---

Filed at Joel's request from yaks-0229 (not for now). Browse inside an archive attachment (`.zip`, `.tar.gz`) read-only in the TUI's directory view, and link into one (`<id>/code.zip#path/in/archive`, or a `!/` form). That needs a decision on the link form, and the `zip`/`tar`/`flate2` crates. Low priority while `yaks attach <dir>` (the sibling yak) removes the reason to attach an archive.

#### 2026-11-17T21:42:06Z

*sources-triage*

Related: yaks-0229 (where it came from), yaks-a57c (attach a directory, which removes most of the need).

#### 2026-11-18T23:10:28Z

*delta:ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-18T23:29:26Z

*delta:ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA*

Design read, no code yet. Full write-up: [files, directories and archives](/doks/design/files-directories-archives.md).

- **Finding:** "what does this link name, and how is it shown?" is answered in about ten places (`links::parse`/`exists`/`peek`, `locate::resolve`, and in the TUI `farm_dir_of`, `target_dir`, `farm_path_is_dir`, `open_farm_doc`, `read_doc`, `peek_target`, `open_external`, two follow dispatches). Directories (yaks-a57c, yaks-3329) were added at each call site; archives added the same way would touch every one again.
- **Proposal:** an archive reads as a read-only directory. First a refactor with no visible change: one container lister (`src/entries.rs`) and `Place` gaining directories, so the TUI has one `open_place`. Then archives are one more container kind plus from-bytes viewer constructors, touching no opener.
- **Link form:** the path goes through the archive, `<id>/code.zip/src/main.rs#L12`. Rejected `code.zip!/...` and `code.zip#...`.
- **Crates:** `zip` 2.x (no default features, `deflate-flate2`), `tar`, `flate2` (already in the tree).
- **Bugs filed:** yaks-1d2b (a dok's directory link is `../` to doctor and peek), yaks-8218 (`walk_dir` follows symlinks).

#### 2026-11-18T23:29:26Z

*delta:ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA · kind: ask · needs: human*

asked: needs human

Three decisions before any code:

1. **Refactor first?** Do the no-visible-change seam (one lister, `Place` knows directories, one TUI `open_place`) as its own yak before archives. Lean: yes; it also fixes yaks-1d2b.
2. **Link form:** `<id>/code.zip/src/main.rs` (path goes through the archive). Lean: this one.
3. **Archives at all, now?** The yak is p4 and `attach <dir>` removed the reason. Lean: do step 1 now (it pays for itself), leave archives at p4 until a farm actually has one.

#### 2026-11-19T01:27:51Z

*sched-1 · kind: answer · needs: agent*

1. Strong yes. It could get out of hand pretty quickly otherwise.
2. Yup, sounds good and intuitive.
3. The P4 thing is kind of an accident. I actually have a bunch of .tgz archives in my ~/src/fs/joel repo, which is what triggered this yak. So let's go ahead and do it once the refactoring is in place.

#### 2026-11-19T01:28:58Z

*sched-1*

Also, consider whether yaks-4381 fits into the refactoring.

#### 2026-11-19T01:29:59Z

*delta:Review yaks-93ee for file and yak handli · kind: pickup*

picked up

#### 2026-11-19T01:32:32Z

*delta:Review yaks-93ee for file and yak handli*

Answers taken in: refactor first, filed as yaks-84fe (now a dependency of this yak and of yaks-4381); link form `<id>/code.zip/src/main.rs`; archives go ahead once yaks-84fe lands (Joel has real `.tgz` attachments in ~/src/fs/joel). yaks-4381 fits the refactor; the design dok now has a section on it (a subject's name is its canonical link destination). Leaving this yak shaving, waiting on yaks-84fe.

#### 2026-11-19T01:42:12Z

*herd-files*

Batch note (herd-files): symwalk-1 on yaks-8218, model claude-sonnet-5-5, base b013ff06, thread ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG98JJIBxBCA0aLJyrhJPreUvVwLftHG. yaks-84fe by herd-files itself (Opus).

#### 2026-11-19T02:34:18Z

*herd-files*

Claim (yakherd `herd-files`). Owner: worker `archive-1` (Sonnet 5.5), on top of yaks-84fe (`84ffb8b7`). Scope, by the [design dok](/doks/design/files-directories-archives.md) section 2, 4, 5:

- `src/entries.rs`: `Container::Zip`, `Container::Tar { gz }`; `container_at` walks a path's segments through an archive (returns the container and the inner path); `list` and a bounded `read` for archives; member-name safety (no absolute, `..` or symlink members).
- `src/locate.rs`: a directory inside an archive is `Place::FarmDir` (its `rel` may pass through the archive); a file in one is a new `Place::Member`. `links::exists` / `peek` go through it.
- TUI: from-bytes constructors for `AttachView` and `ImageView`; `App::read_place` / `open_place` arms; the directory view unfolds `.zip`/`.tar`/`.tar.gz`/`.tgz` like a directory (other archive kinds keep the "copy its path" note); take-out on a member (`p`/`r` path through the archive; `e`/`o` a read-only temp copy). Never system-open an archive.
- Deps: `zip` 2.x (no default features, `deflate-flate2`), `tar` 0.4 (no default features), `flate2` 1. Cargo.toml/Cargo.lock edits allowed for exactly these.
- Out of scope: stars (yaks-4381, `star-1` running beside it), `links.rs` grammar, keymap.
- Evidence: tests on a built zip and tar.gz (list, read, link resolve, doctor, peek, a `..` member refused, a member over the size cap refused); headless frames of a tree unfolding into an archive and a member open, attached; the gate; docs (`docs/cli.md` links + peek, `docs/tui.md` "Browsing a directory", `yaks` skill Links).
- Judge: the coordinator at landing.
