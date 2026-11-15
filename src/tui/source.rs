//! Links to repo source files (yaks-da5e): `[x](../src/foo.rs#L12-L40)`.
//!
//! `../src/foo.rs` is a path from the repo root: one `../` for the `.yaks/`
//! the link conventionally climbs out of (it predates herd directories, when
//! a yak file sat at `.yaks/<id>.md`, and is kept so existing links keep
//! working). yaks resolves it against the code checkout; other renderers
//! no longer resolve it by plain relative path. Two optional parts:
//!
//! - a GitHub-style line fragment, `#L12` or `#L12-L40`;
//! - a pinned commit in the GitHub blob-URL shape, `../blob/<sha>/src/foo.rs`
//!   (`<sha>` 7-40 hex digits). Elsewhere it is just a relative link that goes
//!   nowhere; here it names the file as it was at that commit.
//!
//! `code:src/foo.rs` is the current form (yaks-101d); see [`crate::links`],
//! which owns the grammar; the renderer classifies every link with it
//! (`mdrender::link_target`, yaks-7395). The opening side
//! ([`App::open_source`]) does the file and git work when a link is followed.

use super::*;
use std::process::Command;

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
#[cfg_attr(test, allow(dead_code))] // a test never launches the opener (`App::os_open`)
pub(crate) fn os_open(arg: &str) -> std::io::Result<std::process::Child> {
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    Command::new(opener).arg(arg).spawn()
}

pub(crate) fn editor_from_env() -> Option<String> {
    ["VISUAL", "EDITOR"]
        .iter()
        .filter_map(|k| std::env::var(k).ok())
        .find(|v| !v.trim().is_empty())
}

/// The checkout a source link is resolved in: [`crate::links::code_base`].
pub(crate) fn source_base(farm_root: &Path, cwd: Option<&Path>) -> Option<PathBuf> {
    crate::links::code_base(farm_root, cwd)
}

impl App {
    /// Follow a `code:` link: open the file in the read-only viewer
    /// ([`App::read_code_view`]) at the lines the link names (a jump: `o`
    /// comes back). Taking it out to an editor is `O` then `e`
    /// ([`super::takeout`]).
    pub(crate) fn open_code(&mut self, path: &str, lines: Option<(u32, u32)>, rev: Option<&str>) {
        let v = match self.read_code_view(path, rev, lines) {
            Ok(v) => v,
            Err(e) => {
                self.notification = Some(e);
                return;
            }
        };
        let failed = matches!(v.doc, super::handlers::DocKind::Code { error: Some(_), .. });
        self.follow_to_doc(DocView::Text(v));
        if failed {
            return;
        }
        if let Some((a, _)) = lines {
            self.land_on_line(a as usize);
        }
        self.notification = Some(format!("viewing {}", source_label(path, lines, rev)));
    }

    /// The viewer for code file `path` at `rev`. A file or commit that cannot
    /// be read still gives a viewer, whose one row says why (and from which
    /// `O` still takes the link out: `g` needs nothing local). `Err` only
    /// when there is no code checkout to look in.
    pub(crate) fn read_code_view(
        &self,
        path: &str,
        rev: Option<&str>,
        marked: Option<(u32, u32)>,
    ) -> Result<super::handlers::AttachView, String> {
        let root = self
            .farm
            .as_ref()
            .map(|f| f.root().to_path_buf())
            .ok_or("no farm to resolve the link against")?;
        let base = source_base(&root, self.sheds.cwd.as_deref())
            .ok_or("no code checkout to resolve the link against")?;
        let space = crate::locate::Space::new(&root, Some(base.clone()));
        let too_big = rev.is_none()
            && std::fs::metadata(base.join(path)).is_ok_and(|m| m.len() > CODE_MAX_BYTES);
        let text = if too_big {
            Err(format!(
                "{path}: too big to view here (O then e opens it in your editor)"
            ))
        } else {
            space.code_text(path, rev).and_then(|t| {
                if t.contains('\0') || t.starts_with("(binary file") {
                    Err(format!("{path}: not a text file (O then o opens it)"))
                } else {
                    Ok(t)
                }
            })
        };
        Ok(super::handlers::AttachView::code(
            path,
            base.join(path),
            text,
            rev.map(String::from),
            marked,
        ))
    }

    /// Put the open viewer's cursor on 1-based `line` at the top of the
    /// page, as a `#L12` link lands on its row.
    fn land_on_line(&mut self, line: usize) {
        let Some((rows, vp)) = self.attachment_rows() else {
            return;
        };
        let Some(v) = self.attach_mut() else { return };
        let to = line.saturating_sub(1).min(rows.len().saturating_sub(1));
        v.nav.line = to;
        v.nav.scroll = to.min(vp.max_scroll(rows.len())) as u16;
    }
}

/// Largest code file the viewer reads (as for attachments).
const CODE_MAX_BYTES: u64 = 1 << 20;

#[cfg(test)]
mod tests {
    use super::super::detail::Target;
    use super::*;

    /// A destination as the renderer classifies it in a yak, when it is a
    /// code file (`mdrender::link_target`, which delegates to `links::parse`).
    fn parse_source(dest: &str) -> Option<Target> {
        match super::super::mdrender::link_target(
            dest,
            crate::links::From::Yak,
            &Default::default(),
        ) {
            t @ Target::Source { .. } => Some(t),
            _ => None,
        }
    }

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
        let (f, snap) = crate::locate::pinned_file(&proj, "src/foo.rs", Some(&rev)).unwrap();
        assert_eq!((f.clone(), snap), (proj.join("src/foo.rs"), false));
        // No pin: the working copy; a missing file is a message, not a panic.
        assert_eq!(
            crate::locate::pinned_file(&proj, "src/foo.rs", None)
                .unwrap()
                .0,
            proj.join("src/foo.rs")
        );
        assert!(crate::locate::pinned_file(&proj, "src/nope.rs", None).is_err());

        // Edited since: the pinned content, in a read-only temp copy.
        std::fs::write(proj.join("src/foo.rs"), "changed\n").unwrap();
        let (f, snap) = crate::locate::pinned_file(&proj, "src/foo.rs", Some(&rev[..10])).unwrap();
        assert!(snap);
        assert_eq!(std::fs::read_to_string(&f).unwrap(), "one\ntwo\n");
        assert!(std::fs::metadata(&f).unwrap().permissions().readonly());
        // Opening it again (a read-only copy already there) still works.
        assert!(crate::locate::pinned_file(&proj, "src/foo.rs", Some(&rev[..10])).is_ok());

        // A file the commit did not have, and a commit that does not exist.
        std::fs::write(proj.join("src/new.rs"), "x\n").unwrap();
        let err = crate::locate::pinned_file(&proj, "src/new.rs", Some(&rev)).unwrap_err();
        assert!(err.contains("not in"), "{err}");
        assert!(crate::locate::pinned_file(&proj, "src/foo.rs", Some("deadbeef")).is_err());

        // Through the menu: `e` queues the editor with the line, on that copy.
        let mut app = App::with_farm(farm).unwrap();
        let t = crate::tui::takeout::TakeOut {
            place: crate::locate::Place::CodeFile {
                rel: "src/foo.rs".into(),
                rev: Some(rev[..10].into()),
                anchor: None,
            },
            line: Some(2),
            label: "src/foo.rs:L2-L3@".into(),
        };
        app.take_out_with(Some("vim"), &t, 'e');
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
    fn a_farm_branch_resolves_source_links_in_the_code_checkout_not_the_farm_worktree() {
        let base = std::env::temp_dir().join(format!("yaks-srclive-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let code = base.join("code");
        std::fs::create_dir_all(code.join("src")).unwrap();
        std::fs::write(code.join("src/foo.rs"), "a\nb\n").unwrap();
        git_ok(&code, &["init", "-q", "-b", "main"]);
        git_ok(&code, &["add", "."]);
        git_ok(&code, &["commit", "-q", "-m", "init"]);
        // The farm on an orphan branch, in its own worktree (yaks-73f4).
        let wt = base.join("farm");
        git_ok(
            &code,
            &[
                "worktree",
                "add",
                "-q",
                "--orphan",
                "-b",
                "yaks",
                wt.to_str().unwrap(),
            ],
        );
        std::fs::create_dir_all(wt.join(".yaks")).unwrap();
        std::fs::write(wt.join(".yaks/config.yaml"), "herd: yak\n").unwrap();
        let root = wt.join(".yaks");
        let code_c = crate::gitrepo::canonical(&code);
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
            crate::locate::pinned_file(&source_base(&root, None).unwrap(), "src/foo.rs", None)
                .is_ok()
        );
        // A farm in its own checkout keeps resolving against its parent.
        let (proj, _farm) = crate::tui::test_support::temp_farm(&[]);
        assert_eq!(
            source_base(&proj.join(".yaks"), Some(&code)),
            Some(proj.clone())
        );
        let _ = std::fs::remove_dir_all(&base);
    }
}
