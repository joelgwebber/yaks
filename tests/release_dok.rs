//! `yaks release dok` (yaks-c708): the release dok `doks/releases/<version>.md`
//! with a generated changes block, through the real binary on a temp repo with
//! a scratch farm and tags.

mod common;
use common::Repo;
use std::path::PathBuf;

struct Rel {
    r: Repo,
    feat: String,
    kid: String,
    gap: String,
    umbrella: String,
}

/// `v0.1.0`, then a range that has an entry, a family, a gap, an internal yak
/// and an open yak, tagged `v0.2.0`.
fn release(tag: &str) -> Rel {
    let r = Repo::new(tag);
    r.commit("initial");
    r.git(&["tag", "v0.1.0"]);
    let feat = r.create("A feature", "feature", None);
    let umbrella = r.create("Umbrella", "feature", None);
    let kid = r.create("Kid", "task", Some(&umbrella));
    let gap = r.create("Gap", "bug", None);
    let internal = r.create("Internal", "task", None);
    let open = r.create("Open", "task", None);
    r.run(&["shorn", &feat, "--change", "Adds the feature."]);
    r.run(&["shorn", &kid]);
    r.run(&["shorn", &umbrella, "--change", "Umbrella lands."]);
    r.run(&["shorn", &gap]);
    r.run(&["shorn", &internal, "--no-change"]);
    r.commit(&format!("{feat}: the feature"));
    r.commit(&format!("the kid\n\nRefs {kid}."));
    r.commit(&format!("{umbrella}: umbrella"));
    r.commit(&format!("the gap ({gap})"));
    r.commit(&format!("internal [[{internal}]]"));
    r.commit(&format!("partial work on {open}"));
    r.commit("stray work");
    r.git(&["tag", "v0.2.0"]);
    Rel {
        r,
        feat,
        kid,
        gap,
        umbrella,
    }
}

fn farm(r: &Repo) -> PathBuf {
    PathBuf::from(r.run(&["path"]).trim())
}

fn read(r: &Repo, rel: &str) -> String {
    std::fs::read_to_string(farm(r).join(rel)).unwrap()
}

fn link(id: &str) -> String {
    format!("[{id}](/yak/yaks/{id}.md)")
}

#[test]
fn it_creates_the_dok_with_its_fields_schema_and_block() {
    let s = release("create");
    let out = s.r.run(&["release", "dok", "v0.2.0"]);
    assert!(out.starts_with("Created doks/releases/v0.2.0.md"), "{out}");
    let dok = read(&s.r, "doks/releases/v0.2.0.md");
    let front = dok.split("\n---\n").next().unwrap();
    assert!(front.contains("type: release"), "{dok}");
    assert!(front.contains("version: v0.2.0"), "{dok}");
    assert!(front.contains("tag: v0.2.0"), "{dok}");
    assert!(front.contains("range: v0.1.0..v0.2.0"), "{dok}");
    assert!(front.contains("date: 20"), "{dok}");
    assert!(dok.contains("# yak v0.2.0\n\n<!-- Headlines:"), "{dok}");
    // Entries by section, each linking its yak (their order within a section
    // follows the yaks' creation, which these tests make in one second).
    assert!(dok.contains("## Added\n\n- "), "{dok}");
    assert!(
        dok.contains(&format!("- Adds the feature. ({})", link(&s.feat))),
        "{dok}"
    );
    assert!(
        dok.contains(&format!("- Umbrella lands. ({})", link(&s.umbrella))),
        "{dok}"
    );
    // A covered child nests under its umbrella, by title.
    assert!(
        dok.contains(&format!(
            "- Umbrella lands. ({})\n  - Kid ({})",
            link(&s.umbrella),
            link(&s.kid)
        )),
        "{dok}"
    );
    // A gap is listed by title, marked; open, internal and no-yak work is not.
    assert!(
        dok.contains(&format!(
            "## Not described\n\n- Gap ({}) (no change line)",
            link(&s.gap)
        )),
        "{dok}"
    );
    for left_out in ["Internal", "Open", "stray work", "Fixed", "Security"] {
        assert!(!dok.contains(left_out), "{left_out} in {dok}");
    }
    assert!(dok.contains("<!-- yaks:changes -->") && dok.contains("<!-- /yaks:changes -->"));
    // The schema was written the first time, with the brief's fields.
    let schema = read(&s.r, "schemas/release.md");
    for f in ["version:", "tag:", "date:", "range:", "yanked:"] {
        assert!(schema.contains(f), "{schema}");
    }
    assert!(s.r.cmd(&["doctor"]).status.success());
    // The dok is a real dok of kind release, found by a query.
    let q = s.r.run(&["dok", "query", "--type", "release"]);
    assert!(q.contains("doks/releases/v0.2.0.md"), "{q}");
}

#[test]
fn a_rerun_rewrites_only_the_block_and_keeps_the_prose_and_other_fields() {
    let s = release("rerun");
    s.r.run(&["release", "dok", "v0.2.0"]);
    let path = farm(&s.r).join("doks/releases/v0.2.0.md");
    // A person writes the headlines, adds a field and a closing paragraph.
    let edited = std::fs::read_to_string(&path)
        .unwrap()
        .replace(
            "<!-- Headlines: what matters in this release, with a screenshot if there is one (pin images with @<sha>). -->",
            "The big news: **the feature**.",
        )
        .replace("range: v0.1.0..v0.2.0", "range: v0.1.0..HEAD")
        .replace("type: release\n", "type: release\nyanked: true\n")
        + "\nUpgrade notes: none.\n";
    std::fs::write(&path, &edited).unwrap();
    // A change line is corrected afterwards.
    s.r.run(&[
        "update",
        &s.feat,
        "--note",
        "Adds the better feature.",
        "--meta",
        "kind=change",
    ]);
    let out = s.r.run(&["release", "dok", "v0.2.0"]);
    assert!(out.starts_with("Updated doks/releases/v0.2.0.md"), "{out}");
    let after = std::fs::read_to_string(&path).unwrap();
    assert!(after.contains("The big news: **the feature**."), "{after}");
    assert!(after.ends_with("\nUpgrade notes: none.\n"), "{after}");
    assert!(after.contains("yanked: true"), "{after}");
    assert!(after.contains("Adds the better feature."), "{after}");
    assert!(!after.contains("Adds the feature."), "{after}");
    // The derived `range:` is brought back to what it resolves to.
    assert!(after.contains("range: v0.1.0..v0.2.0"), "{after}");
    // Nothing else changed: the text outside the block is the same.
    let outside = |t: &str| {
        let a = t.find("<!-- yaks:changes -->").unwrap();
        let b = t.find("<!-- /yaks:changes -->").unwrap();
        format!("{}{}", &t[..a], &t[b..])
    };
    assert_eq!(
        outside(&after),
        outside(&edited).replace("range: v0.1.0..HEAD", "range: v0.1.0..v0.2.0")
    );
    // A second rerun has nothing to do and commits nothing.
    let again = s.r.run(&["release", "dok", "v0.2.0"]);
    assert!(again.starts_with("Already up to date:"), "{again}");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), after);
}

#[test]
fn a_version_that_is_not_a_tag_yet_covers_the_latest_tag_to_head() {
    let s = release("untagged");
    let late = s.r.create("Late", "bug", None);
    s.r.run(&["shorn", &late, "--change", "Fixes the late thing."]);
    s.r.commit(&format!("{late}: late fix"));
    s.r.run(&["release", "dok", "v0.3.0"]);
    let dok = read(&s.r, "doks/releases/v0.3.0.md");
    assert!(dok.contains("range: v0.2.0..HEAD"), "{dok}");
    // No tag: no `tag:` field.
    assert!(!dok.contains("\ntag:"), "{dok}");
    assert!(dok.contains("Fixes the late thing."), "{dok}");
    assert!(!dok.contains("Adds the feature."), "{dok}");
    // An explicit range overrides both rules.
    s.r.run(&["release", "dok", "v0.4.0", "v0.1.0..v0.2.0"]);
    let dok = read(&s.r, "doks/releases/v0.4.0.md");
    assert!(dok.contains("range: v0.1.0..v0.2.0"), "{dok}");
    assert!(dok.contains("Adds the feature."), "{dok}");
    // Tagging it later turns `tag:` on and the range into the tag's.
    s.r.git(&["tag", "v0.3.0"]);
    s.r.run(&["release", "dok", "v0.3.0"]);
    let dok = read(&s.r, "doks/releases/v0.3.0.md");
    assert!(dok.contains("tag: v0.3.0"), "{dok}");
    assert!(dok.contains("range: v0.2.0..v0.3.0"), "{dok}");
}

#[test]
fn dry_run_prints_the_dok_and_writes_nothing() {
    let s = release("dry");
    let before = s.r.git(&["rev-parse", "yaks"]);
    let out = s.r.run(&["release", "dok", "v0.2.0", "--dry-run"]);
    assert!(out.starts_with("---\ntype: release\n"), "{out}");
    assert!(out.contains("Adds the feature."), "{out}");
    assert!(!farm(&s.r).join("doks/releases").exists());
    assert!(!farm(&s.r).join("schemas/release.md").exists());
    assert_eq!(s.r.git(&["rev-parse", "yaks"]), before);
}

#[test]
fn a_write_is_one_farm_commit() {
    let s = release("commit");
    let count = |r: &Repo| {
        r.git(&["rev-list", "--count", "yaks"])
            .parse::<u32>()
            .unwrap()
    };
    let before = count(&s.r);
    s.r.run(&["release", "dok", "v0.2.0"]);
    assert_eq!(count(&s.r), before + 1);
    let files = s.r.git(&["show", "--name-only", "--format=", "yaks"]);
    assert!(files.contains("doks/releases/v0.2.0.md"), "{files}");
    assert!(files.contains("schemas/release.md"), "{files}");
}

#[test]
fn check_flags_a_stale_block_until_the_dok_is_rewritten() {
    let s = release("check");
    s.r.run(&["release", "dok", "v0.2.0"]);
    let check = |r: &Repo| {
        let out = r.cmd(&["check"]);
        String::from_utf8_lossy(&out.stdout).into_owned()
    };
    let clean = check(&s.r);
    assert!(!clean.contains("release dok"), "{clean}");
    // Correcting a change line makes the block out of date.
    s.r.run(&[
        "update",
        &s.gap,
        "--note",
        "Closes the gap.",
        "--meta",
        "kind=change",
    ]);
    let out = check(&s.r);
    assert!(
        out.contains(
            "release dok doks/releases/v0.2.0.md is out of date with its yaks: run `yaks release dok v0.2.0`"
        ),
        "{out}"
    );
    s.r.run(&["release", "dok", "v0.2.0"]);
    assert!(!check(&s.r).contains("release dok"));
}

#[test]
fn a_dok_without_its_block_is_refused_not_overwritten() {
    let s = release("noblock");
    s.r.run(&["release", "dok", "v0.2.0"]);
    let path = farm(&s.r).join("doks/releases/v0.2.0.md");
    let t = std::fs::read_to_string(&path)
        .unwrap()
        .replace("<!-- yaks:changes -->", "")
        .replace("<!-- /yaks:changes -->", "");
    std::fs::write(&path, &t).unwrap();
    let out = s.r.cmd(&["release", "dok", "v0.2.0"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("no generated block"));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), t);
}

#[test]
fn it_needs_a_code_checkout() {
    let s = release("nocode");
    let out = assert_cmd::Command::cargo_bin("yaks")
        .unwrap()
        .current_dir(farm(&s.r))
        .env("XDG_DATA_HOME", common::data_home(&s.r.dir))
        .env("YAKS_ACTOR", "tester")
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .args(["release", "dok", "v0.2.0"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("code repository"));
}
