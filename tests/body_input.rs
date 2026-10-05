//! Multi-line text for `--description` / `--note` (create, update, ask,
//! answer): `-` reads stdin, `--description-file` / `--note-file` read a file.
//! The text must land byte-for-byte (trailing newline trimmed), and every
//! misuse must fail loudly rather than store an empty or partial note.

use assert_cmd::Command;
use std::path::{Path, PathBuf};

/// Backticks, both quote kinds, `$`, a blank line, a fenced block, indentation
/// and a trailing space: everything a shell-quoted `--note "..."` mangles.
const TEXT: &str = "## Heading\n\n- item with `code`, \"double\", 'single' and $HOME $(not run)\n\n```sh\n  echo \"$x\" | grep `y`\n```\nlast line ends with a space \n\nafter blank";

struct Farm {
    dir: PathBuf,
}

impl Farm {
    fn new(tag: &str) -> Farm {
        let dir = std::env::temp_dir().join(format!("yaks-body-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let farm = Farm { dir };
        farm.ok(&["init"], None);
        farm
    }

    fn cmd(&self, args: &[&str]) -> Command {
        let mut cmd = Command::cargo_bin("yaks").unwrap();
        cmd.current_dir(&self.dir)
            .env("YAKS_SKILLS_AUTOSYNC", "0")
            .env("YAKS_ACTOR", "tester")
            .args(args);
        cmd
    }

    /// Run with `stdin` piped (an empty pipe when `None`); must succeed.
    fn ok(&self, args: &[&str], stdin: Option<&str>) -> String {
        let out = self
            .cmd(args)
            .write_stdin(stdin.unwrap_or(""))
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    }

    /// Run with `stdin` piped; must fail, returning stderr.
    fn err(&self, args: &[&str], stdin: &str) -> String {
        let out = self.cmd(args).write_stdin(stdin).output().unwrap();
        assert!(!out.status.success(), "{args:?} unexpectedly succeeded");
        String::from_utf8(out.stderr).unwrap()
    }

    fn create(&self, title: &str) -> String {
        let out = self.ok(&["create", title, "--json"], None);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        v["id"].as_str().unwrap().to_string()
    }

    fn file(&self, id: &str) -> String {
        let path = self.ok(&["path", id], None);
        std::fs::read_to_string(Path::new(path.trim())).unwrap()
    }

    fn write(&self, name: &str, text: &str) -> String {
        let p = self.dir.join(name);
        std::fs::write(&p, text).unwrap();
        p.to_string_lossy().into_owned()
    }
}

impl Drop for Farm {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// The text the file ends with, exactly once (notes append at the end).
fn ends_with_text(file: &str) -> bool {
    file.ends_with(&format!("{TEXT}\n"))
}

#[test]
fn create_description_from_stdin_and_file() {
    let f = Farm::new("create");
    let out = f.ok(
        &["create", "A", "--description", "-", "--json"],
        Some(&format!("{TEXT}\n")),
    );
    let id = serde_json::from_str::<serde_json::Value>(&out).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(ends_with_text(&f.file(&id)), "{}", f.file(&id));

    let path = f.write("body.md", &format!("{TEXT}\n\n"));
    let out = f.ok(
        &["create", "B", "--description-file", &path, "--json"],
        None,
    );
    let id = serde_json::from_str::<serde_json::Value>(&out).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(ends_with_text(&f.file(&id)), "{}", f.file(&id));
}

#[test]
fn update_description_and_note_from_stdin_and_file() {
    let f = Farm::new("update");
    let id = f.create("U");
    f.ok(
        &["update", &id, "--description", "-"],
        Some(&format!("{TEXT}\n")),
    );
    assert!(ends_with_text(&f.file(&id)));

    let path = f.write("d.md", TEXT);
    f.ok(&["update", &id, "--description-file", &path], None);
    assert!(ends_with_text(&f.file(&id)));

    f.ok(
        &["update", &id, "--note", "-"],
        Some(&format!("{TEXT}\r\n")),
    );
    let file = f.file(&id);
    assert!(ends_with_text(&file), "{file}");
    assert!(file.contains("[tester]"), "{file}");

    let path = f.write("n.md", &format!("{TEXT}\n"));
    f.ok(&["update", &id, "--note-file", &path], None);
    let file = f.file(&id);
    assert!(ends_with_text(&file));
    assert_eq!(file.matches("## Heading").count(), 3, "{file}");
}

#[test]
fn update_applies_the_same_text_to_every_id() {
    let f = Farm::new("multi");
    let (a, b) = (f.create("A"), f.create("B"));
    f.ok(&["update", &a, &b, "--note", "-"], Some(TEXT));
    assert!(ends_with_text(&f.file(&a)));
    assert!(ends_with_text(&f.file(&b)));
}

#[test]
fn ask_and_answer_take_stdin_and_file() {
    let f = Farm::new("askanswer");
    let id = f.create("Q");
    f.ok(&["ask", &id, "--note", "-"], Some(&format!("{TEXT}\n")));
    let file = f.file(&id);
    assert!(ends_with_text(&file), "{file}");
    assert!(file.contains("needs: human"), "{file}");

    let path = f.write("a.md", &format!("{TEXT}\n"));
    f.ok(&["answer", &id, "--note-file", &path], None);
    let file = f.file(&id);
    assert!(ends_with_text(&file));
    assert!(!file.contains("needs:"), "{file}");

    f.ok(&["ask", &id, "--note-file", &path], None);
    f.ok(&["answer", &id, "--note", "-"], Some(TEXT));
    assert!(ends_with_text(&f.file(&id)));
}

#[test]
fn file_dash_means_stdin() {
    let f = Farm::new("filedash");
    let id = f.create("D");
    f.ok(&["update", &id, "--note-file", "-"], Some(TEXT));
    assert!(ends_with_text(&f.file(&id)));
}

#[test]
fn inline_text_is_unchanged() {
    let f = Farm::new("inline");
    let id = f.create("I");
    f.ok(&["update", &id, "--description", "one line"], None);
    f.ok(&["update", &id, "--note", "inline note"], None);
    assert!(f.file(&id).ends_with("inline note\n"));
    // An explicit empty inline description still clears the body.
    f.ok(&["update", &id, "--description", ""], None);
    assert!(!f.file(&id).contains("inline note"));
}

#[test]
fn empty_stdin_or_file_is_an_error_not_an_empty_note() {
    let f = Farm::new("empty");
    let id = f.create("E");
    let before = f.file(&id);
    let empty = f.write("empty.md", "");
    let blank = f.write("blank.md", " \n\n");
    for (args, stdin) in [
        (vec!["update", &id, "--note", "-"], ""),
        (vec!["update", &id, "--note", "-"], "\n"),
        (vec!["update", &id, "--description", "-"], "  \n"),
        (vec!["ask", &id, "--note", "-"], ""),
        (vec!["answer", &id, "--note", "-"], ""),
        (vec!["create", "X", "--description", "-"], ""),
        (vec!["update", &id, "--note-file", &empty], ""),
        (vec!["update", &id, "--note-file", &blank], ""),
        (vec!["create", "X", "--description-file", &empty], ""),
    ] {
        let e = f.err(&args, stdin);
        assert!(e.contains("empty"), "{args:?}: {e}");
    }
    assert_eq!(
        f.file(&id),
        before,
        "a failed command must not touch the yak"
    );
}

#[test]
fn dash_for_both_note_and_description_is_an_error() {
    let f = Farm::new("both");
    let id = f.create("B");
    let before = f.file(&id);
    let e = f.err(&["update", &id, "--note", "-", "--description", "-"], TEXT);
    assert!(e.contains("only once"), "{e}");
    let e = f.err(
        &["update", &id, "--note-file", "-", "--description", "-"],
        TEXT,
    );
    assert!(e.contains("only once"), "{e}");
    assert_eq!(f.file(&id), before);
}

#[test]
fn dash_note_with_a_description_file_is_fine() {
    let f = Farm::new("mixed");
    let id = f.create("M");
    let path = f.write("d.md", "from a file");
    f.ok(
        &["update", &id, "--description-file", &path, "--note", "-"],
        Some("from stdin"),
    );
    let file = f.file(&id);
    assert!(
        file.contains("from a file") && file.ends_with("from stdin\n"),
        "{file}"
    );
}

#[test]
fn inline_and_file_for_the_same_flag_is_an_error() {
    let f = Farm::new("conflict");
    let id = f.create("C");
    let path = f.write("n.md", TEXT);
    for args in [
        vec!["update", &id, "--note", "x", "--note-file", &path],
        vec!["update", &id, "--note", "-", "--note-file", &path],
        vec![
            "update",
            &id,
            "--description",
            "x",
            "--description-file",
            &path,
        ],
        vec!["ask", &id, "--note", "x", "--note-file", &path],
        vec!["answer", &id, "--note", "x", "--note-file", &path],
        vec![
            "create",
            "X",
            "--description",
            "x",
            "--description-file",
            &path,
        ],
    ] {
        let e = f.err(&args, "");
        assert!(e.contains("cannot be used with"), "{args:?}: {e}");
    }
}

#[test]
fn missing_or_unreadable_file_names_the_path() {
    let f = Farm::new("missing");
    let id = f.create("F");
    let missing = f.dir.join("nope.md").to_string_lossy().into_owned();
    let e = f.err(&["update", &id, "--note-file", &missing], "");
    assert!(e.contains("--note-file") && e.contains(&missing), "{e}");
    // A directory is unreadable as text, and so is non-UTF-8 bytes.
    let dir = f.dir.to_string_lossy().into_owned();
    let e = f.err(&["create", "X", "--description-file", &dir], "");
    assert!(e.contains(&dir), "{e}");
    let bin = f.dir.join("bin.md");
    std::fs::write(&bin, [0xff, 0xfe, 0x00]).unwrap();
    let bin = bin.to_string_lossy().into_owned();
    let e = f.err(&["answer", &id, "--note-file", &bin], "");
    assert!(e.contains(&bin), "{e}");
}
