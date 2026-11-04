//! `yaks import-hearth` through the real binary (yaks-5550): a small
//! Hearth-layout repo built here is copied into a scratch farm.

mod common;
use assert_cmd::Command;
use std::path::{Path, PathBuf};

const SRC: &str = "2026-10-26-13846285";
const SHOT: &str = "2026-10-26-1b129a8d";

struct Setup {
    dir: PathBuf,
    repo: PathBuf,
}

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

fn git(dir: &Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@example.com"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?}: {out:?}");
}

impl Setup {
    /// A farm repo and, beside it, a Hearth-layout repo with two pages, a
    /// source with an image, a schema, a session review, and the things that
    /// are not imported.
    fn new(tag: &str) -> Setup {
        let dir = std::env::temp_dir().join(format!("yaks-hearth-{tag}-{}", std::process::id()));
        common::cleanup(&dir);
        let farm_repo = dir.join("farm");
        common::git_init(&farm_repo);
        let repo = dir.join("research");
        common::git_init(&repo);
        write(
            &repo,
            "wiki/topics/signals.md",
            &format!(
                "---\ntype: topic\ntitle: Signals\nsources:\n  - raw/sources/{SRC}/source.md\n---\n\n\
                 See [errors](errors.md) and [the project](../projects/app.md#plan).\n\n\
                 ![shot](../../raw/sources/{SHOT}/shot%201.png)\n\n\
                 Notes live in [notes](../../notes/idea.md), code in [main](../../projects/app/main.rs).\n\n\
                 Inline `[x](../nowhere.md)` stays, and so does [the web](https://example.org/a/../b).\n"
            ),
        );
        write(
            &repo,
            "wiki/topics/errors.md",
            "---\ntype: topic\ntitle: Errors\n---\n\nBack to [signals](signals.md).\n",
        );
        write(
            &repo,
            "wiki/projects/app.md",
            "---\ntype: note\ntitle: App\n---\n\n## Plan\n\nDo it.\n",
        );
        write(
            &repo,
            &format!("raw/sources/{SRC}/source.md"),
            &format!(
                "---\ntype: source\ntitle: Launch page\nfile: page.html\nmedia_type: text/html\nsize: 4\n\
                 sources:\n  - id: file\n    resource: \"/tmp/raw/sources/page.html\"\n---\n\nIts text.\n"
            ),
        );
        write(&repo, &format!("raw/sources/{SRC}/page.html"), "<p>");
        write(
            &repo,
            &format!("raw/sources/{SHOT}/source.md"),
            &format!(
                "---\ntype: source\ntitle: Screenshot\nfile: shot 1.png\nmedia_type: image/png\nsize: 4\n\
                 part_of: \"raw/sources/{SRC}/source.md\"\n---\n\nAn image.\n"
            ),
        );
        std::fs::write(
            repo.join(format!("raw/sources/{SHOT}/shot 1.png")),
            [0x89, b'P', b'N', b'G', 0, 1, 2, 255],
        )
        .unwrap();
        write(
            &repo,
            "schemas/agent-platform.md",
            "---\ntype: Page Type\ntitle: Agent platform\nfields:\n  vendor: {type: text}\n---\n\nRows.\n",
        );
        write(
            &repo,
            "raw/sessions/2026/10/s1.review.md",
            &format!(
                "---\ntype: session-review\ntitle: A session\n---\n\nIt saved [the page](../../../../wiki/topics/errors.md) \
                 and [a source](../../../sources/{SRC}/source.md).\n"
            ),
        );
        write(&repo, "notes/idea.md", "An idea.\n");
        write(&repo, "projects/app/main.rs", "fn main() {}\n");
        write(&repo, "schedules/daily.md", "run\n");
        write(&repo, "core/memory.md", "remember\n");
        write(&repo, "AGENTS.md", "layout\n");
        git(&repo, &["add", "-A"]);
        git(&repo, &["commit", "-q", "-m", "fixture"]);
        let s = Setup { dir, repo };
        s.run(&["init", "--no-skills"]);
        s
    }

    fn cmd(&self, args: &[&str]) -> std::process::Output {
        Command::cargo_bin("yaks")
            .unwrap()
            .current_dir(self.dir.join("farm"))
            .env("XDG_DATA_HOME", common::data_home(&self.dir.join("farm")))
            .env("YAKS_SKILLS_AUTOSYNC", "0")
            .env("YAKS_ACTOR", "tester")
            .args(args)
            .output()
            .unwrap()
    }

    fn run(&self, args: &[&str]) -> String {
        let out = self.cmd(args);
        assert!(
            out.status.success(),
            "{args:?} failed: {}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    }

    fn import(&self, extra: &[&str]) -> String {
        let repo = self.repo.to_str().unwrap();
        let mut a = vec!["import-hearth", repo];
        a.extend_from_slice(extra);
        self.run(&a)
    }

    fn root(&self) -> PathBuf {
        PathBuf::from(self.run(&["path"]).trim())
    }

    fn farm_git(&self, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(self.root())
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}: {out:?}");
        String::from_utf8(out.stdout).unwrap()
    }
}

impl Drop for Setup {
    fn drop(&mut self) {
        common::cleanup(&self.dir.join("farm"));
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn imports_rewrites_links_commits_once_and_is_idempotent() {
    let s = Setup::new("main");
    let root = s.root();
    let commits_before = s.farm_git(&["rev-list", "--count", "HEAD"]);

    // A dry run says what it would do and writes nothing.
    let dry = s.import(&["--dry-run"]);
    assert!(dry.contains("would import"), "{dry}");
    assert!(
        dry.contains("3 doks") || dry.contains("wiki/ -> doks/  3 doks"),
        "{dry}"
    );
    assert!(!root.join("doks").exists() && !root.join("sources").exists());
    assert_eq!(s.farm_git(&["rev-list", "--count", "HEAD"]), commits_before);

    let report = s.import(&[]);
    assert!(report.contains("imported from"), "{report}");
    assert!(report.contains("sources/    2 sources"), "{report}");
    assert!(report.contains("doctor: all clear"), "{report}");
    for skipped in ["schedules", "core", "AGENTS.md", "notes", "projects"] {
        assert!(
            report.contains(&format!("  {skipped}:")),
            "{skipped} in {report}"
        );
    }
    assert!(
        report.contains("(../../notes/idea.md) -> (code:notes/idea.md)")
            && report.contains("(../../projects/app/main.rs) -> (code:projects/app/main.rs)"),
        "{report}"
    );

    // Layout.
    for rel in [
        "doks/topics/signals.md",
        "doks/projects/app.md",
        &format!("sources/{SRC}/page.html"),
        &format!("sources/{SHOT}/shot 1.png"),
        "schemas/agent-platform.md",
        "sessions/2026/10/s1.review.md",
    ] {
        assert!(root.join(rel).is_file(), "{rel}");
    }
    assert!(!root.join("schedules").exists() && !root.join("notes").exists());

    // Links are rewritten by byte range; everything else is as it was.
    let page = std::fs::read_to_string(root.join("doks/topics/signals.md")).unwrap();
    let want = format!(
        "---\ntype: topic\ntitle: Signals\nsources:\n  - sources/{SRC}/source.md\n---\n\n\
         See [errors](errors.md) and [the project](/doks/projects/app.md#plan).\n\n\
         ![shot](/sources/{SHOT}/shot%201.png)\n\n\
         Notes live in [notes](code:notes/idea.md), code in [main](code:projects/app/main.rs).\n\n\
         Inline `[x](../nowhere.md)` stays, and so does [the web](https://example.org/a/../b).\n"
    );
    assert_eq!(page, want);
    let session = std::fs::read_to_string(root.join("sessions/2026/10/s1.review.md")).unwrap();
    assert!(
        session.contains("(/doks/topics/errors.md)")
            && session.contains(&format!("(/sources/{SRC}/source.md)")),
        "{session}"
    );
    let part = std::fs::read_to_string(root.join(format!("sources/{SHOT}/source.md"))).unwrap();
    assert!(
        part.contains(&format!("part_of: \"sources/{SRC}/source.md\"")),
        "{part}"
    );
    // The rest of source.md, including a resource that is not a repo path.
    let top = std::fs::read_to_string(root.join(format!("sources/{SRC}/source.md"))).unwrap();
    assert!(
        top.contains("resource: \"/tmp/raw/sources/page.html\""),
        "{top}"
    );
    // Binary files are byte for byte.
    assert_eq!(
        std::fs::read(root.join(format!("sources/{SHOT}/shot 1.png"))).unwrap(),
        std::fs::read(s.repo.join(format!("raw/sources/{SHOT}/shot 1.png"))).unwrap()
    );

    // One commit, naming the repo and its HEAD, holding nothing else.
    assert_eq!(
        s.farm_git(&["rev-list", "--count", "HEAD"])
            .trim()
            .parse::<u32>()
            .unwrap(),
        commits_before.trim().parse::<u32>().unwrap() + 1
    );
    let head = s.farm_git(&["rev-parse", "--short", "HEAD"]);
    let subject = s.farm_git(&["log", "-1", "--format=%s"]);
    let repo_head = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(&s.repo)
        .output()
        .unwrap();
    let repo_head = String::from_utf8(repo_head.stdout).unwrap();
    assert!(
        subject.contains("from research @") && subject.contains(repo_head.trim()),
        "{subject}"
    );
    assert!(s.farm_git(&["status", "--porcelain"]).is_empty());
    assert!(report.contains(head.trim()), "{report}");

    // The dok, source and doctor commands read what moved.
    assert_eq!(s.run(&["dok", "list"]).lines().count(), 3);
    assert_eq!(s.run(&["source", "list"]).lines().count(), 2);
    s.run(&["doctor"]);

    // The repo is untouched.
    let out = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&s.repo)
        .output()
        .unwrap();
    assert!(out.stdout.is_empty(), "{out:?}");

    // Again: nothing to write, no new commit.
    let again = s.import(&[]);
    assert!(again.contains("0 file(s) written"), "{again}");
    assert!(again.contains("nothing to import"), "{again}");
    assert_eq!(s.farm_git(&["rev-parse", "--short", "HEAD"]), head);
}

#[test]
fn refuses_to_overwrite_a_different_file_and_writes_nothing() {
    let s = Setup::new("refuse");
    s.import(&[]);
    let root = s.root();
    // The farm's copy was edited; the repo's changed too.
    std::fs::write(
        root.join("doks/projects/app.md"),
        "---\ntype: note\ntitle: Mine\n---\n",
    )
    .unwrap();
    write(
        &s.repo,
        "wiki/topics/new.md",
        "---\ntype: note\ntitle: New\n---\n",
    );
    let out = s.cmd(&["import-hearth", s.repo.to_str().unwrap()]);
    assert!(!out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("REFUSED") && text.contains("doks/projects/app.md"),
        "{text}"
    );
    assert!(
        !root.join("doks/topics/new.md").exists(),
        "nothing is written on a refusal"
    );
    let kept = std::fs::read_to_string(root.join("doks/projects/app.md")).unwrap();
    assert!(kept.contains("Mine"));
}

#[test]
fn a_directory_that_is_not_a_hearth_repo_is_refused() {
    let s = Setup::new("notrepo");
    let other = s.dir.join("empty");
    std::fs::create_dir_all(&other).unwrap();
    let out = s.cmd(&["import-hearth", other.to_str().unwrap()]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("not a Hearth-layout repo"));
}
