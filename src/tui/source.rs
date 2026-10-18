//! Links to repo source files (yaks-da5e): `[x](../src/foo.rs#L12-L40)`.
//!
//! A yak file lives at `.yaks/<id>.md`, so a plain relative link that climbs
//! one level (`../src/foo.rs`) is a path from the repo root, and resolves the
//! same way on GitHub and in any other markdown renderer. Two optional parts:
//!
//! - a GitHub-style line fragment, `#L12` or `#L12-L40`;
//! - a pinned commit in the GitHub blob-URL shape, `../blob/<sha>/src/foo.rs`
//!   (`<sha>` 7-40 hex digits). Elsewhere it is just a relative link that goes
//!   nowhere; here it names the file as it was at that commit.
//!
//! [`parse_source`] is pure (no disk, no git): the renderer calls it for every
//! link. The opening side ([`App::open_source`]) does the file and git work
//! when a link is followed.

use super::detail::Target;
use super::*;
use std::process::{Command, Stdio};

/// A program for the event loop to run with the terminal suspended (the
/// loop owns the terminal; a key handler only asks).
#[derive(Clone, PartialEq, Debug)]
pub(crate) struct ExternalCmd {
    pub program: String,
    pub args: Vec<String>,
}

/// `path[:L12-L40][@sha7]`, for the status bar and notifications.
pub fn source_label(path: &str, lines: Option<(u32, u32)>, rev: Option<&str>) -> String {
    let mut s = path.to_string();
    match lines {
        Some((a, b)) if a == b => s.push_str(&format!(":L{a}")),
        Some((a, b)) => s.push_str(&format!(":L{a}-L{b}")),
        None => {}
    }
    if let Some(r) = rev {
        s.push('@');
        s.push_str(&r[..r.len().min(7)]);
    }
    s
}

/// Classify a link destination as a repo source file, or `None` (a URL, an
/// attachment, anything that does not climb out of `.yaks/` into the repo).
pub fn parse_source(dest: &str) -> Option<Target> {
    if dest.contains("://") || dest.starts_with("mailto:") {
        return None;
    }
    let (file, frag) = match dest.split_once('#') {
        Some((p, f)) => (p, Some(f)),
        None => (dest, None),
    };
    let rest = file.strip_prefix("../")?;
    let (rev, path) = match rest.strip_prefix("blob/").and_then(|r| r.split_once('/')) {
        Some((sha, path)) if is_rev(sha) => (Some(sha.to_string()), path),
        _ => (None, rest),
    };
    let path = clean_path(&percent_decode(path))?;
    Some(Target::Source {
        path,
        lines: frag.and_then(parse_lines),
        rev,
    })
}

fn is_rev(s: &str) -> bool {
    (7..=40).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_hexdigit())
}

/// `L12` or `L12-L40` (1-based, inclusive; a reversed range is reordered).
fn parse_lines(frag: &str) -> Option<(u32, u32)> {
    let r = frag.strip_prefix('L')?;
    let (a, b) = r.split_once("-L").unwrap_or((r, r));
    let (a, b): (u32, u32) = (a.parse().ok()?, b.parse().ok()?);
    (a > 0 && b > 0).then(|| (a.min(b), a.max(b)))
}

/// Drop `.`/empty segments; refuse `..` (a link may not climb back out of the
/// repo) and an empty or directory path.
fn clean_path(p: &str) -> Option<String> {
    if p.ends_with('/') {
        return None;
    }
    let mut parts = Vec::new();
    for seg in p.split('/') {
        match seg {
            "" | "." => {}
            ".." => return None,
            s => parts.push(s),
        }
    }
    (!parts.is_empty()).then(|| parts.join("/"))
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        let hex = |c: u8| (c as char).to_digit(16);
        if b[i] == b'%' && i + 2 < b.len() {
            if let (Some(h), Some(l)) = (hex(b[i + 1]), hex(b[i + 2])) {
                out.push((h * 16 + l) as u8);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| s.to_string())
}

/// How to open `file` (at `line`) with `$VISUAL`/`$EDITOR` text `editor`
/// (which may carry its own arguments, like `code -w`).
pub(crate) fn editor_command(editor: &str, file: &Path, line: Option<u32>) -> Option<ExternalCmd> {
    let mut words = editor.split_whitespace();
    let program = words.next()?.to_string();
    let mut args: Vec<String> = words.map(String::from).collect();
    let name = Path::new(&program)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let file = file.display().to_string();
    match (line, name.as_str()) {
        (Some(n), "code" | "codium" | "code-insiders" | "cursor") => {
            args.extend(["-g".into(), format!("{file}:{n}")])
        }
        (Some(n), "subl" | "zed" | "hx" | "helix") => args.push(format!("{file}:{n}")),
        // vi, vim, nvim, nano, emacs, micro, kak, ...: `+N file`.
        (Some(n), _) => args.extend([format!("+{n}"), file]),
        (None, _) => args.push(file),
    }
    Some(ExternalCmd { program, args })
}

/// Open `arg` (a file or URL) with the OS default application.
pub(crate) fn os_open(arg: &str) -> std::io::Result<std::process::Child> {
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    Command::new(opener).arg(arg).spawn()
}

fn git(base: &Path, args: &[&str]) -> std::io::Result<std::process::Output> {
    Command::new("git")
        .arg("-C")
        .arg(base)
        .args(args)
        .stdin(Stdio::null())
        .output()
}

/// The file to show for a source link: the working copy, or for a pinned
/// link whose file differs from the working tree, a read-only copy of
/// `git show <rev>:<path>` in the temp directory. `Err` is a one-line message.
pub(crate) fn resolve_source_file(
    base: &Path,
    path: &str,
    rev: Option<&str>,
) -> Result<(PathBuf, bool), String> {
    let work = base.join(path);
    let Some(rev) = rev else {
        return if work.is_file() {
            Ok((work, false))
        } else {
            Err(format!("{path}: no such file"))
        };
    };
    let rel = format!("./{path}");
    // `git diff <rev> -- <file>` is clean for an untracked file too, so first
    // ask whether the commit has the file at all.
    let has =
        git(base, &["cat-file", "-e", &format!("{rev}:{rel}")]).map_err(|e| format!("git: {e}"))?;
    if !has.status.success() {
        let short = &rev[..rev.len().min(7)];
        let err = String::from_utf8_lossy(&has.stderr);
        return Err(
            if err.contains("exists on disk, but not in") || err.contains("does not exist in") {
                format!("{path} not in {short}")
            } else {
                format!("{short}: {}", err.lines().next().unwrap_or("not a commit"))
            },
        );
    }
    let diff = git(base, &["diff", "--quiet", rev, "--", &rel]).map_err(|e| format!("git: {e}"))?;
    match diff.status.code() {
        Some(0) if work.is_file() => return Ok((work, false)),
        Some(0 | 1) => {}
        _ => {
            let err = String::from_utf8_lossy(&diff.stderr);
            let line = err.lines().next().unwrap_or("git diff failed");
            return Err(format!("{rev}: {line}"));
        }
    }
    let shown = git(base, &["show", &format!("{rev}:{rel}")]).map_err(|e| format!("git: {e}"))?;
    if !shown.status.success() {
        return Err(format!("{path} not in {}", &rev[..rev.len().min(7)]));
    }
    let tmp = std::env::temp_dir().join("yaks-src").join(rev).join(path);
    let write = || -> std::io::Result<()> {
        std::fs::create_dir_all(tmp.parent().unwrap())?;
        let _ = std::fs::remove_file(&tmp); // a previous copy is read-only
        std::fs::write(&tmp, &shown.stdout)?;
        let mut perm = std::fs::metadata(&tmp)?.permissions();
        perm.set_readonly(true);
        std::fs::set_permissions(&tmp, perm)
    };
    write().map_err(|e| format!("temp copy failed: {e}"))?;
    Ok((tmp, true))
}

fn editor_from_env() -> Option<String> {
    ["VISUAL", "EDITOR"]
        .iter()
        .filter_map(|k| std::env::var(k).ok())
        .find(|v| !v.trim().is_empty())
}

/// The checkout a source link (`../src/foo.rs`) is resolved in. For a farm in
/// its own checkout that is the farm's parent, as the link's `../` says. A
/// LIVE farm (`live_branch:`, yaks-6a34) lives in a worktree that holds only
/// `.yaks/`, so there it is the code checkout instead: the git top-level of
/// `cwd` (where the TUI was started) unless that is the farm worktree itself,
/// else the repository's main checkout (yaks-6c4f). `None` when nothing fits.
pub(crate) fn source_base(farm_root: &Path, cwd: Option<&Path>) -> Option<PathBuf> {
    let own = farm_root.parent().map(Path::to_path_buf);
    if crate::store::read_config(farm_root).live_branch.is_none() {
        return own;
    }
    let farm_wt = own.as_deref().map(crate::sheds::canonical);
    let top = |dir: &Path| {
        Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| crate::sheds::canonical(Path::new(String::from_utf8_lossy(&o.stdout).trim())))
    };
    if let Some(t) = cwd.and_then(top).filter(|t| Some(t) != farm_wt.as_ref()) {
        return Some(t);
    }
    crate::sheds::host(farm_root)
        .and_then(|h| crate::sheds::host_checkout(&h))
        .filter(|t| Some(t) != farm_wt.as_ref())
}

impl App {
    /// Follow a source link: `$VISUAL`/`$EDITOR` at the line, else the OS
    /// opener. The editor is queued ([`App::pending_editor`]) for the event
    /// loop to run with the terminal suspended.
    pub(crate) fn open_source(&mut self, path: &str, lines: Option<(u32, u32)>, rev: Option<&str>) {
        self.open_source_with(editor_from_env().as_deref(), path, lines, rev)
    }

    pub(crate) fn open_source_with(
        &mut self,
        editor: Option<&str>,
        path: &str,
        lines: Option<(u32, u32)>,
        rev: Option<&str>,
    ) {
        let label = source_label(path, lines, rev);
        let Some(base) = self
            .farm
            .as_ref()
            .and_then(|f| source_base(f.root(), self.sheds.cwd.as_deref()))
        else {
            self.notification = Some("no code checkout to resolve the link against".into());
            return;
        };
        let (file, snapshot) = match resolve_source_file(&base, path, rev) {
            Ok(f) => f,
            Err(e) => {
                self.notification = Some(e);
                return;
            }
        };
        let note = if snapshot { " (read-only copy)" } else { "" };
        match editor.and_then(|e| editor_command(e, &file, lines.map(|l| l.0))) {
            Some(cmd) => {
                self.pending_editor = Some(cmd);
                self.notification = Some(format!("edited {label}{note}"));
            }
            None => {
                self.notification = Some(match os_open(&file.display().to_string()) {
                    Ok(_) => format!("opened {label}{note}"),
                    Err(e) => format!("open failed: {e}"),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn src(path: &str, lines: Option<(u32, u32)>, rev: Option<&str>) -> Option<Target> {
        Some(Target::Source {
            path: path.into(),
            lines,
            rev: rev.map(String::from),
        })
    }

    #[test]
    fn plain_relative_paths_are_repo_files() {
        assert_eq!(parse_source("../src/foo.rs"), src("src/foo.rs", None, None));
        assert_eq!(parse_source("../README.md"), src("README.md", None, None));
        assert_eq!(parse_source("../a/./b.rs"), src("a/b.rs", None, None));
        assert_eq!(
            parse_source("../my%20dir/f.rs"),
            src("my dir/f.rs", None, None)
        );
    }

    #[test]
    fn line_fragments_follow_github() {
        assert_eq!(
            parse_source("../src/foo.rs#L12"),
            src("src/foo.rs", Some((12, 12)), None)
        );
        assert_eq!(
            parse_source("../src/foo.rs#L12-L40"),
            src("src/foo.rs", Some((12, 40)), None)
        );
        assert_eq!(
            parse_source("../src/foo.rs#L40-L12"),
            src("src/foo.rs", Some((12, 40)), None)
        );
        // Not a line fragment: still the file, no lines.
        for f in ["#intro", "#L", "#L0", "#L1-2", "#L1:2", "#Lx"] {
            assert_eq!(
                parse_source(&format!("../src/foo.rs{f}")),
                src("src/foo.rs", None, None),
                "{f}"
            );
        }
    }

    #[test]
    fn pinned_links_use_the_blob_shape() {
        assert_eq!(
            parse_source("../blob/abc1234/src/foo.rs#L3-L9"),
            src("src/foo.rs", Some((3, 9)), Some("abc1234"))
        );
        let sha = "0123456789abcdef0123456789abcdef01234567";
        assert_eq!(
            parse_source(&format!("../blob/{sha}/src/foo.rs")),
            src("src/foo.rs", None, Some(sha))
        );
        // Too short / not hex: an ordinary `blob/` directory in the repo.
        assert_eq!(
            parse_source("../blob/abc12/x.rs"),
            src("blob/abc12/x.rs", None, None)
        );
        assert_eq!(
            parse_source("../blob/zzzzzzz/x.rs"),
            src("blob/zzzzzzz/x.rs", None, None)
        );
    }

    #[test]
    fn everything_else_is_not_a_source_link() {
        for d in [
            "https://x.io/a",
            "mailto:a@b.c",
            "yaks-0001/log.txt", // an attachment
            "docs/x.md",
            "/etc/passwd",
            "../../etc/passwd", // climbs out of the repo
            "../src/../../x",
            "../src/",
            "../",
            "../blob/abc1234/",
        ] {
            assert_eq!(parse_source(d), None, "{d}");
        }
    }

    #[test]
    fn labels() {
        assert_eq!(source_label("a.rs", None, None), "a.rs");
        assert_eq!(source_label("a.rs", Some((5, 5)), None), "a.rs:L5");
        assert_eq!(
            source_label("a.rs", Some((5, 9)), Some("abcdef0123")),
            "a.rs:L5-L9@abcdef0"
        );
    }

    #[test]
    fn editor_arguments_by_editor() {
        let f = Path::new("/r/src/foo.rs");
        let cmd = |e, l| editor_command(e, f, l).unwrap();
        assert_eq!(cmd("vim", Some(7)).args, ["+7", "/r/src/foo.rs"]);
        assert_eq!(cmd("/usr/bin/nvim", None).args, ["/r/src/foo.rs"]);
        let c = cmd("code -w", Some(7));
        assert_eq!(
            (c.program.as_str(), c.args),
            (
                "code",
                vec!["-w".into(), "-g".into(), "/r/src/foo.rs:7".into()]
            )
        );
        assert_eq!(cmd("hx", Some(7)).args, ["/r/src/foo.rs:7"]);
        assert!(editor_command("  ", f, None).is_none());
    }

    fn git_ok(dir: &Path, args: &[&str]) -> String {
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["-c", "user.name=t", "-c", "user.email=t@t"])
            .args(args)
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}: {out:?}");
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    }

    #[test]
    fn a_pinned_link_opens_a_read_only_copy_only_when_the_file_changed() {
        let (proj, farm) = crate::tui::test_support::temp_farm(&[]);
        std::fs::create_dir_all(proj.join("src")).unwrap();
        git_ok(&proj, &["init", "-q"]);
        std::fs::write(proj.join("src/foo.rs"), "one\ntwo\n").unwrap();
        git_ok(&proj, &["add", "src"]);
        git_ok(&proj, &["commit", "-q", "-m", "first"]);
        let rev = git_ok(&proj, &["rev-parse", "HEAD"]);

        // Unchanged since the pin: the working copy itself.
        let (f, snap) = resolve_source_file(&proj, "src/foo.rs", Some(&rev)).unwrap();
        assert_eq!((f.clone(), snap), (proj.join("src/foo.rs"), false));
        // No pin: the working copy; a missing file is a message, not a panic.
        assert_eq!(
            resolve_source_file(&proj, "src/foo.rs", None).unwrap().0,
            proj.join("src/foo.rs")
        );
        assert!(resolve_source_file(&proj, "src/nope.rs", None).is_err());

        // Edited since: the pinned content, in a read-only temp copy.
        std::fs::write(proj.join("src/foo.rs"), "changed\n").unwrap();
        let (f, snap) = resolve_source_file(&proj, "src/foo.rs", Some(&rev[..10])).unwrap();
        assert!(snap);
        assert_eq!(std::fs::read_to_string(&f).unwrap(), "one\ntwo\n");
        assert!(std::fs::metadata(&f).unwrap().permissions().readonly());
        // Opening it again (a read-only copy already there) still works.
        assert!(resolve_source_file(&proj, "src/foo.rs", Some(&rev[..10])).is_ok());

        // A file the commit did not have, and a commit that does not exist.
        std::fs::write(proj.join("src/new.rs"), "x\n").unwrap();
        let err = resolve_source_file(&proj, "src/new.rs", Some(&rev)).unwrap_err();
        assert!(err.contains("not in"), "{err}");
        assert!(resolve_source_file(&proj, "src/foo.rs", Some("deadbeef")).is_err());

        // Through the app: the editor is queued with the line, on that copy.
        let mut app = App::with_farm(farm).unwrap();
        app.open_source_with(Some("vim"), "src/foo.rs", Some((2, 3)), Some(&rev[..10]));
        let cmd = app.pending_editor.take().expect("editor queued");
        assert_eq!(cmd.program, "vim");
        assert_eq!(cmd.args[0], "+2");
        assert!(cmd.args[1].contains("yaks-src"), "{cmd:?}");
        assert!(
            app.notification
                .as_deref()
                .unwrap()
                .contains("src/foo.rs:L2-L3@")
        );
    }

    #[test]
    fn a_live_farm_resolves_source_links_in_the_code_checkout_not_the_farm_worktree() {
        let base = std::env::temp_dir().join(format!("yaks-srclive-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let code = base.join("code");
        std::fs::create_dir_all(code.join("src")).unwrap();
        std::fs::create_dir_all(code.join(".yaks")).unwrap();
        std::fs::write(code.join("src/foo.rs"), "a\nb\n").unwrap();
        std::fs::write(
            code.join(".yaks/config.yaml"),
            "herd: yak\nlive_branch: farm\n",
        )
        .unwrap();
        git_ok(&code, &["init", "-q", "-b", "main"]);
        git_ok(&code, &["add", "."]);
        git_ok(&code, &["commit", "-q", "-m", "init"]);
        let wt = base.join("farm");
        git_ok(
            &code,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                "farm",
                wt.to_str().unwrap(),
                "main",
            ],
        );
        git_ok(&wt, &["sparse-checkout", "set", "--no-cone", "/.yaks/"]);
        let root = wt.join(".yaks");
        let code_c = crate::sheds::canonical(&code);
        // From the code checkout, and from the farm worktree (falls back to the
        // main checkout), the base is the code; never the sparse farm worktree.
        assert_eq!(
            source_base(&root, Some(&code.join("src"))),
            Some(code_c.clone())
        );
        assert_eq!(source_base(&root, Some(&wt)), Some(code_c.clone()));
        assert_eq!(source_base(&root, None), Some(code_c.clone()));
        // The old rule (the farm's parent) is what broke: no src there.
        assert!(!wt.join("src/foo.rs").exists());
        assert!(
            resolve_source_file(&source_base(&root, None).unwrap(), "src/foo.rs", None).is_ok()
        );
        // A farm in its own checkout keeps resolving against its parent.
        let (proj, _farm) = crate::tui::test_support::temp_farm(&[]);
        assert_eq!(
            source_base(&proj.join(".yaks"), Some(&code)),
            Some(proj.clone())
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn following_a_source_link_queues_the_editor_at_the_line() {
        let (proj, farm) = crate::tui::test_support::temp_farm(&[]);
        std::fs::create_dir_all(proj.join("src")).unwrap();
        std::fs::write(proj.join("src/foo.rs"), "a\nb\nc\n").unwrap();
        let mut app = App::with_farm(farm).unwrap();
        app.open_source_with(Some("nvim"), "src/foo.rs", Some((2, 2)), None);
        let cmd = app.pending_editor.take().expect("editor queued");
        let file = proj.join("src/foo.rs").display().to_string();
        assert_eq!(cmd.args, ["+2".to_string(), file]);
        // A missing file: a message, nothing queued.
        app.open_source_with(Some("nvim"), "src/gone.rs", None, None);
        assert!(app.pending_editor.is_none());
        assert_eq!(
            app.notification.as_deref(),
            Some("src/gone.rs: no such file")
        );
    }
}
