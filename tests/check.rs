//! `yaks check` through the real binary (yaks-c0ac): stale pages, uncited
//! sources, kinds that look alike, and `doctor`'s link, schema, conflict and
//! page findings, as text and JSON, with the exit code.

mod common;
use assert_cmd::Command;
use serde_json::Value;
use std::path::PathBuf;

struct Farm {
    dir: PathBuf,
}

impl Farm {
    fn new(tag: &str) -> Farm {
        let dir = std::env::temp_dir().join(format!("yaks-check-{tag}-{}", std::process::id()));
        common::cleanup(&dir);
        common::git_init(&dir);
        let f = Farm { dir };
        f.run(&["init", "--no-skills"]);
        f
    }

    fn cmd(&self, args: &[&str]) -> Command {
        let mut c = Command::cargo_bin("yaks").unwrap();
        c.current_dir(&self.dir)
            .env("XDG_DATA_HOME", common::data_home(&self.dir))
            .env("YAKS_SKILLS_AUTOSYNC", "0")
            .env("YAKS_ACTOR", "tester")
            .args(args);
        c
    }

    fn run(&self, args: &[&str]) -> String {
        let out = self.cmd(args).output().unwrap();
        assert!(
            out.status.success(),
            "{args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    }

    /// Run `check` and return its exit code with stdout.
    fn check(&self, args: &[&str]) -> (i32, String) {
        let mut a = vec!["check"];
        a.extend_from_slice(args);
        let out = self.cmd(&a).output().unwrap();
        (
            out.status.code().unwrap(),
            String::from_utf8(out.stdout).unwrap(),
        )
    }

    fn root(&self) -> PathBuf {
        PathBuf::from(self.run(&["path"]).trim())
    }

    fn write(&self, rel: &str, text: &str) {
        let p = self.root().join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    /// Save `text` as a source and return its `sources/<id>` directory.
    fn source(&self, name: &str, text: &str, extra: &[&str]) -> String {
        let file = self.dir.join(name);
        std::fs::write(&file, text).unwrap();
        let mut a = vec!["source", "add", file.to_str().unwrap()];
        a.extend_from_slice(extra);
        let out = self.run(&a);
        let path = out.trim().rsplit(' ').next().unwrap().to_string();
        path.strip_suffix("/source.md").unwrap().to_string()
    }
}

impl Drop for Farm {
    fn drop(&mut self) {
        common::cleanup(&self.dir);
    }
}

fn kinds(json: &str) -> Vec<String> {
    let v: Value = serde_json::from_str(json).unwrap();
    v.as_array()
        .unwrap()
        .iter()
        .map(|f| f["kind"].as_str().unwrap().to_string())
        .collect()
}

fn finding<'a>(v: &'a Value, kind: &str) -> Vec<&'a Value> {
    v.as_array()
        .unwrap()
        .iter()
        .filter(|f| f["kind"] == kind)
        .collect()
}

#[test]
fn a_clean_farm_exits_zero() {
    let f = Farm::new("clean");
    f.write("wiki/a.md", "---\ntype: note\ntitle: A\n---\n\nFine.\n");
    let (code, out) = f.check(&[]);
    assert_eq!(
        (code, out.as_str()),
        (0, "All clear: nothing to check up on.\n")
    );
    let (code, out) = f.check(&["--json"]);
    assert_eq!((code, out.trim()), (0, "[]"));
}

#[test]
fn every_kind_of_finding_is_reported_with_exit_one() {
    let f = Farm::new("all");
    // stale: a date in the past (its own), and a schema duration counted from
    // `verified:`.
    f.write(
        "wiki/old.md",
        "---\ntype: note\ntitle: Old\nstale_after: 2020-01-01\n---\n\nOld.\n",
    );
    f.write(
        "wiki/fresh.md",
        "---\ntype: note\ntitle: Fresh\nstale_after: 2999-01-01\n---\n\nFresh.\n",
    );
    f.write(
        "schemas/recipe.md",
        "---\ntype: Page Type\ntitle: Recipe\nstale_after: P6M\n---\n",
    );
    f.write(
        "wiki/soup.md",
        "---\ntype: recipe\ntitle: Soup\nverified: 2020-03-01\n---\n\nSoup.\n",
    );
    f.write(
        "wiki/stew.md",
        "---\ntype: recipe\ntitle: Stew\nverified: 2999-03-01\n---\n\nStew.\n",
    );
    // uncited: one source with nothing citing it, one cited by a page.
    let lone = f.source("lone.md", "# Lone\n", &[]);
    let used = f.source("used.md", "# Used\n", &[]);
    f.write(
        "wiki/cites.md",
        &format!("---\ntype: note\ntitle: Cites\nsources:\n  - {used}/source.md\n---\n\nSee it.\n"),
    );
    // similar kinds: `project` and `projects`.
    f.write("wiki/p1.md", "---\ntype: project\ntitle: P1\n---\n");
    f.write("wiki/p2.md", "---\ntype: projects\ntitle: P2\n---\n");
    // doctor's: a broken link, a page without a title, a merge conflict.
    f.write(
        "wiki/links.md",
        "---\ntype: note\ntitle: Links\n---\n\nSee [gone](/wiki/missing.md).\n",
    );
    f.write("wiki/untitled.md", "---\ntype: note\n---\n\nNo title.\n");
    f.write(
        "wiki/clash.md",
        "---\ntype: note\ntitle: Clash\n---\n\n<<<<<<< here\nours\n=======\ntheirs\n>>>>>>> other side\n",
    );

    let (code, json) = f.check(&["--json"]);
    assert_eq!(code, 1, "{json}");
    assert_eq!(
        kinds(&json),
        [
            "page-problem",
            "broken-link",
            "merge-conflict",
            "stale",
            "stale",
            "uncited",
            "similar-kinds"
        ],
        "{json}"
    );
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(finding(&v, "page-problem")[0]["ids"][0], "wiki/untitled.md");
    assert_eq!(finding(&v, "broken-link")[0]["ids"][0], "wiki/links.md");
    assert_eq!(finding(&v, "merge-conflict")[0]["ids"][0], "wiki/clash.md");
    let stale = finding(&v, "stale");
    assert_eq!(stale[0]["ids"][0], "wiki/old.md");
    assert_eq!(
        stale[0]["message"],
        "wiki/old.md was due to be checked by 2020-01-01"
    );
    assert_eq!(stale[1]["ids"][0], "wiki/soup.md");
    assert_eq!(
        stale[1]["message"],
        "wiki/soup.md was verified 2020-03-01; its type asks for a re-check by 2020-09-01"
    );
    let uncited = finding(&v, "uncited");
    assert_eq!(uncited[0]["ids"][0], lone.as_str());
    assert!(
        !json.contains(&format!("{used}\"")),
        "cited source listed: {json}"
    );
    let similar = &finding(&v, "similar-kinds")[0];
    assert_eq!(similar["ids"], serde_json::json!(["project", "projects"]));
    assert_eq!(
        similar["message"],
        "these look like the same kind of page: `project` (1 page), `projects` (1 page)"
    );

    // The text report groups by kind and says how many.
    let (code, text) = f.check(&[]);
    assert_eq!(code, 1);
    assert!(text.starts_with("Found 7 findings:\n"), "{text}");
    for needle in [
        "Stale page",
        "Uncited source",
        "Similar kinds",
        "Broken link",
        "Merge conflict",
        "Page problem",
        "wiki/old.md was due to be checked by 2020-01-01",
    ] {
        assert!(text.contains(needle), "{needle} missing from:\n{text}");
    }
}

#[test]
fn a_source_and_its_parts_count_as_one_and_marks_keep_it_off_the_list() {
    let f = Farm::new("parts");
    let whole = f.source("whole.md", "# Whole\n", &[]);
    let part_of = format!("{whole}/source.md");
    let shot = f.source("shot.md", "# Shot\n", &["--part-of", &part_of]);
    let (_, json) = f.check(&["--json"]);
    let v: Value = serde_json::from_str(&json).unwrap();
    let u = finding(&v, "uncited");
    assert_eq!(u.len(), 1, "{json}");
    assert_eq!(u[0]["ids"][0], whole.as_str());
    assert!(
        u[0]["message"].as_str().unwrap().contains("1 part"),
        "{json}"
    );

    // Citing only the part cites the whole.
    f.write(
        "wiki/c.md",
        &format!("---\ntype: note\ntitle: C\n---\n\n![shot](/{shot}/shot.md)\n"),
    );
    let (code, out) = f.check(&[]);
    assert_eq!(
        (code, out.as_str()),
        (0, "All clear: nothing to check up on.\n")
    );

    // A mark in source.md keeps an otherwise uncited source off the list.
    let marked = f.source("marked.md", "# Marked\n", &[]);
    assert_eq!(f.check(&["--json"]).0, 1);
    let note = f.root().join(&marked).join("source.md");
    let text = std::fs::read_to_string(&note).unwrap();
    std::fs::write(
        &note,
        text.replacen(
            "type: source\n",
            "type: source\nuncited: kept on purpose\n",
            1,
        ),
    )
    .unwrap();
    assert_eq!(f.check(&["--json"]).0, 0);
}

#[test]
fn a_page_with_no_verified_date_counts_from_its_last_commit() {
    let f = Farm::new("commit");
    f.write(
        "schemas/howto.md",
        "---\ntype: Page Type\ntitle: Howto\nstale_after: P1Y\n---\n",
    );
    f.write("wiki/h.md", "---\ntype: howto\ntitle: H\n---\n\nSteps.\n");
    // Not committed yet: no history, so the page is skipped.
    assert_eq!(f.check(&[]).0, 0);
    let out = f
        .cmd(&["commit", "-m", "old"])
        .env("GIT_COMMITTER_DATE", "2020-02-03T12:00:00Z")
        .env("GIT_AUTHOR_DATE", "2020-02-03T12:00:00Z")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let (code, out) = f.check(&[]);
    assert_eq!(code, 1, "{out}");
    assert!(
        out.contains(
            "wiki/h.md was last changed 2020-02-03; its type asks for a re-check by 2021-02-03"
        ),
        "{out}"
    );
}
