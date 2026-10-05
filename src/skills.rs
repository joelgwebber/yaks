//! The agent skills, embedded in the binary so the single self-contained
//! `yaks` can install them without cloning the repo (yaks-45b2). The SKILL.md
//! files are baked in at build time via `include_str!`, so the shipped binary
//! always carries the skill that matches its version.
//!
//! Installed copies carry a **provenance stamp** in their frontmatter (see
//! [`stamp`]). Without one, an installed SKILL.md is an anonymous copy: nothing
//! records which yaks wrote it or what it looked like when written, so "the
//! tool moved on" (yaks-bd9a) can't be told apart from "a human edited this"
//! (yaks-d8e9) and the only available answer is to clobber. The stamp turns
//! that into the four decidable states of [`SkillState`].

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// One file of a bundled skill, relative to the skill's directory.
pub struct SkillFile {
    pub path: &'static str,
    pub content: &'static str,
    /// Installed with mode 0755 (a script the skill tells the agent to run).
    pub executable: bool,
}

/// One bundled skill: a name, the opt-in group it belongs to, and its
/// **explicit file list**.
pub struct Skill {
    pub name: &'static str,
    /// `None` = part of the default install; `Some(g)` = installed only by
    /// `--with g` (see [`GROUPS`]).
    pub group: Option<&'static str>,
    /// Every file the skill ships. `SKILL.md` must be listed first; it carries
    /// the provenance stamp (see [`stamp`]). Other files are compared by content
    /// against the digests recorded in that stamp.
    pub files: &'static [SkillFile],
}

impl Skill {
    /// The pristine `SKILL.md` content.
    pub fn skill_md(&self) -> &'static str {
        self.files[0].content
    }

    /// Every file but `SKILL.md`.
    fn extras(&self) -> &'static [SkillFile] {
        &self.files[1..]
    }
}

const fn md(content: &'static str) -> SkillFile {
    SkillFile {
        path: "SKILL.md",
        content,
        executable: false,
    }
}

/// The opt-in skill groups `yaks skills install --with <group>` accepts.
pub const GROUPS: &[&str] = &["coordination"];

/// Every bundled skill. Paths are relative to this source file
/// (`src/skills.rs`), i.e. the repo's own `.agents/skills/` directory — the
/// single, real source of this repo's skills (yaks-0576).
///
/// **This explicit list — skills and, per skill, files — not the contents of
/// `.agents/skills/`, decides what is embedded and installed.** The directory
/// may hold any other project-local skill, and none is embedded or installed
/// unless it is added here on purpose. A test fails if a bundled skill's
/// directory holds a file this list leaves out.
///
/// The default set is `yaks` + `yaks-tracker`; the `coordination` group (the
/// multi-agent skills plus `yaks-working`) is opt-in.
pub const BUNDLED: &[Skill] = &[
    Skill {
        name: "yaks",
        group: None,
        files: &[md(include_str!("../.agents/skills/yaks/SKILL.md"))],
    },
    Skill {
        name: "yaks-tracker",
        group: None,
        files: &[md(include_str!("../.agents/skills/yaks-tracker/SKILL.md"))],
    },
    Skill {
        name: "yaks-coordinating",
        group: Some("coordination"),
        files: &[md(include_str!(
            "../.agents/skills/yaks-coordinating/SKILL.md"
        ))],
    },
    Skill {
        name: "yaks-coordinating-team",
        group: Some("coordination"),
        files: &[md(include_str!(
            "../.agents/skills/yaks-coordinating-team/SKILL.md"
        ))],
    },
    Skill {
        name: "yaks-coordinating-private",
        group: Some("coordination"),
        files: &[md(include_str!(
            "../.agents/skills/yaks-coordinating-private/SKILL.md"
        ))],
    },
    Skill {
        name: "yaks-coordinating-worktrees",
        group: Some("coordination"),
        files: &[md(include_str!(
            "../.agents/skills/yaks-coordinating-worktrees/SKILL.md"
        ))],
    },
    Skill {
        name: "yaks-coordinating-delta",
        group: Some("coordination"),
        files: &[
            md(include_str!(
                "../.agents/skills/yaks-coordinating-delta/SKILL.md"
            )),
            SkillFile {
                path: "land.sh",
                content: include_str!("../.agents/skills/yaks-coordinating-delta/land.sh"),
                executable: true,
            },
        ],
    },
    Skill {
        name: "yaks-working",
        group: Some("coordination"),
        files: &[md(include_str!("../.agents/skills/yaks-working/SKILL.md"))],
    },
];

/// The skills an install selects: the default set plus every skill of each
/// group in `with` (names from [`GROUPS`]).
pub fn select(with: &[String]) -> Vec<&'static Skill> {
    BUNDLED
        .iter()
        .filter(|s| s.group.is_none_or(|g| with.iter().any(|w| w == g)))
        .collect()
}

/// What `status` reports: the default set, the requested groups, and any other
/// bundled skill that is actually present in `base` (so an opt-in install is
/// tracked without having to repeat `--with`).
pub fn select_for_status(base: &Path, with: &[String]) -> Vec<&'static Skill> {
    BUNDLED
        .iter()
        .filter(|s| {
            s.group.is_none_or(|g| with.iter().any(|w| w == g)) || base.join(s.name).exists()
        })
        .collect()
}

/// This binary's version, stamped into skills it installs.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// The complete set of frontmatter keys the Agent Skills specification allows
/// (<https://agentskills.io/specification>). `name` and `description` are
/// required; the rest are optional.
///
/// This matters because the leniency is asymmetric: Claude Code silently
/// *ignores* an unrecognized key, but claude.ai upload, the Skills API, and
/// packaging with `package_skill.py` reject one with a hard error — so a
/// non-spec key is invisible locally and fatal downstream. That is exactly how
/// a bogus `activation:` key rode along in both bundled skills (yaks-e233).
/// Custom data belongs under `metadata:`, which the spec reserves for it.
#[allow(dead_code)] // enforced by the spec-compliance test over BUNDLED
pub const SPEC_FRONTMATTER_KEYS: &[&str] = &[
    "name",
    "description",
    "license",
    "compatibility",
    "metadata",
    "allowed-tools",
];

/// Environment variable that disables everything yaks does about skills on an
/// ordinary command: the user-level auto-sync (see [`auto_sync`]) and the
/// stale project-local notice (see [`project_stale_notice`]).
pub const AUTOSYNC_ENV: &str = "YAKS_SKILLS_AUTOSYNC";

/// Default install target: `~/.agents/skills`. Overridable so other agents'
/// skills directories (e.g. `~/.claude/skills`) can be targeted.
///
/// `~/.agents/skills` is the cross-client interoperability convention the
/// Agent Skills client-implementation guide tells agents to scan, alongside
/// their own native directory — so it is the widest-reach default. (The spec
/// itself mandates no install location.) Claude Code reads `~/.claude/skills`
/// and does *not* scan `.agents`, which is why `--dir` stays necessary.
pub fn default_dir() -> PathBuf {
    for var in ["HOME", "USERPROFILE"] {
        if let Ok(home) = std::env::var(var) {
            if !home.is_empty() {
                return PathBuf::from(home).join(".agents").join("skills");
            }
        }
    }
    PathBuf::from(".agents").join("skills")
}

/// Expand a leading `~/` in a user-supplied path against `$HOME`.
pub fn expand_tilde(p: &str) -> PathBuf {
    if let Some(rest) = p.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            if !home.is_empty() {
                return PathBuf::from(home).join(rest);
            }
        }
    }
    PathBuf::from(p)
}

// -- where to install -----------------------------------------------------

/// How an install/status target directory was chosen (printed so the user
/// always sees where files go).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetKind {
    /// `--dir <path>`.
    Dir,
    /// `--user`: [`default_dir`].
    User,
    /// No flag, inside a git repo: `<git top-level>/.agents/skills`.
    Project { root: PathBuf },
    /// No flag, not inside a git repo: [`default_dir`], as before.
    UserFallback,
}

/// The chosen skills directory and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub dir: PathBuf,
    pub kind: TargetKind,
}

impl Target {
    /// One line saying where, and why, for CLI output.
    pub fn describe(&self) -> String {
        let why = match &self.kind {
            TargetKind::Dir => "--dir".to_string(),
            TargetKind::User => "--user".to_string(),
            TargetKind::Project { .. } => format!(
                "project-local, in the git top-level; --user for {}",
                default_dir().display()
            ),
            TargetKind::UserFallback => {
                "not inside a git repo, so the user directory; --dir for another".to_string()
            }
        };
        format!("{} ({why})", self.dir.display())
    }
}

/// The nearest ancestor of `cwd` (itself included) holding a `.git` entry (a
/// directory, or the file a linked worktree has) — the git top-level, found the
/// same way `store::discover` bounds a farm search, without shelling out.
pub fn git_toplevel(cwd: &Path) -> Option<PathBuf> {
    cwd.ancestors()
        .find(|d| d.join(".git").exists())
        .map(Path::to_path_buf)
}

/// Choose the skills directory: `--dir` wins; else `--user` means
/// [`default_dir`]; else `<git top-level>/.agents/skills` when `cwd` is inside a
/// git repo, falling back to [`default_dir`] outside one.
pub fn resolve_target(dir: Option<&str>, user: bool, cwd: &Path) -> Target {
    if let Some(d) = dir {
        return Target {
            dir: expand_tilde(d),
            kind: TargetKind::Dir,
        };
    }
    if user {
        return Target {
            dir: default_dir(),
            kind: TargetKind::User,
        };
    }
    match git_toplevel(cwd) {
        Some(root) => Target {
            dir: root.join(".agents").join("skills"),
            kind: TargetKind::Project { root },
        },
        None => Target {
            dir: default_dir(),
            kind: TargetKind::UserFallback,
        },
    }
}

// -- provenance -----------------------------------------------------------

/// A 64-bit FNV-1a digest, rendered as 16 lowercase hex chars.
///
/// Hand-rolled on purpose: this only has to answer "did these bytes change
/// since we wrote them", which needs no cryptographic strength, and it keeps
/// yaks a self-contained binary with no hashing dependency (the same reason
/// the frontmatter parser is hand-rolled).
fn digest(s: impl AsRef<[u8]>) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.as_ref() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x1000_0000_01b3);
    }
    format!("{h:016x}")
}

/// Key names inside the stamped `metadata:` map.
const K_VERSION: &str = "yaks-version";
const K_DIGEST: &str = "yaks-digest";
/// Digests of the skill's other files as written (`path=digest,path=digest`),
/// present only for a skill that ships more than `SKILL.md`. It is what lets a
/// script edited since install be told apart from one an older yaks wrote.
const K_FILES: &str = "yaks-files";

/// Render the provenance block: a `metadata:` map holding the writing yaks'
/// version and a digest of the content as written.
///
/// It lives under `metadata:` because that is the one spec field reserved for
/// "additional properties not defined by the Agent Skills spec". Values are
/// quoted strings because the spec defines metadata as a map of string keys to
/// *string* values. It is emitted last in the frontmatter so it can be removed
/// again deterministically by [`read_stamp`].
fn stamp_block(version: &str, source_digest: &str, files: &[(String, String)]) -> String {
    let mut s = String::from("metadata:\n");
    s.push_str("  ");
    s.push_str(K_VERSION);
    s.push_str(": \"");
    s.push_str(version);
    s.push_str("\"\n  ");
    s.push_str(K_DIGEST);
    s.push_str(": \"");
    s.push_str(source_digest);
    s.push('"');
    if !files.is_empty() {
        let list: Vec<String> = files.iter().map(|(p, d)| format!("{p}={d}")).collect();
        s.push_str(&format!("\n  {K_FILES}: \"{}\"", list.join(",")));
    }
    s
}

/// Stamp `content` (the pristine embedded skill) for installation at `version`.
///
/// The recorded digest is of `content` itself — the text *before* stamping —
/// so verification is a clean round trip: strip the stamp back off an
/// installed file and re-digest what remains.
#[cfg(test)]
fn stamp(content: &str, version: &str) -> String {
    stamp_with_files(content, version, &[])
}

/// [`stamp`], also recording the digests of the skill's other files.
fn stamp_with_files(content: &str, version: &str, files: &[(String, String)]) -> String {
    let block = stamp_block(version, &digest(content), files);
    let Some(rest) = content.strip_prefix("---\n") else {
        // No frontmatter to extend; never corrupt the file.
        return content.to_string();
    };
    match rest.find("\n---") {
        Some(i) => {
            let (fm, body) = rest.split_at(i + 1); // the newline stays with fm
            format!("---\n{fm}{block}\n{body}")
        }
        None => content.to_string(),
    }
}

/// Provenance recovered from an installed skill.
pub struct Stamp {
    pub version: String,
    pub digest: String,
    /// `(path, digest)` of each non-`SKILL.md` file as written.
    pub files: Vec<(String, String)>,
}

/// Split an installed skill into its stamp and the content as originally
/// written (the stamp lines removed).
///
/// Returns `None` when the file carries no recognizable stamp — an *unmanaged*
/// file (hand-written, or produced by another tool), which is never clobbered.
fn read_stamp(installed: &str) -> Option<(Stamp, String)> {
    let mut version: Option<String> = None;
    let mut dig: Option<String> = None;
    let mut files: Vec<(String, String)> = Vec::new();
    let mut kept: Vec<&str> = Vec::new();
    let mut fence = 0usize; // how many `---` delimiters seen
    let mut in_block = false; // inside our metadata: block

    for line in installed.lines() {
        let t = line.trim();
        if t == "---" {
            fence += 1;
            in_block = false;
            kept.push(line);
            continue;
        }
        // Only the frontmatter (between fence 1 and 2) can hold the stamp.
        if fence == 1 {
            if t == "metadata:" {
                in_block = true;
                continue; // drop
            }
            if in_block {
                // Our own keys are dropped; anything else ends the block and is
                // kept, so a skill with its own metadata entries is left alone.
                if let Some(v) = t.strip_prefix(K_VERSION).and_then(|r| r.strip_prefix(':')) {
                    version = Some(unquote(v));
                    continue;
                }
                if let Some(v) = t.strip_prefix(K_DIGEST).and_then(|r| r.strip_prefix(':')) {
                    dig = Some(unquote(v));
                    continue;
                }
                if let Some(v) = t.strip_prefix(K_FILES).and_then(|r| r.strip_prefix(':')) {
                    files = unquote(v)
                        .split(',')
                        .filter_map(|e| e.split_once('='))
                        .map(|(p, d)| (p.to_string(), d.to_string()))
                        .collect();
                    continue;
                }
                in_block = false;
            }
        }
        kept.push(line);
    }

    let (version, digest) = (version?, dig?);
    let mut body = kept.join("\n");
    // `lines()` drops a trailing newline; restore it so the round trip is exact.
    if installed.ends_with('\n') {
        body.push('\n');
    }
    Some((
        Stamp {
            version,
            digest,
            files,
        },
        body,
    ))
}

fn unquote(s: &str) -> String {
    s.trim().trim_matches('"').to_string()
}

/// Compare dotted numeric versions (`0.0.10` > `0.0.9`). Non-numeric or
/// unparseable components compare as 0, so a weird version never wins.
fn version_gt(a: &str, b: &str) -> bool {
    let parts = |v: &str| -> Vec<u64> {
        v.split(['.', '-', '+'])
            .map(|p| p.parse::<u64>().unwrap_or(0))
            .collect()
    };
    let (x, y) = (parts(a), parts(b));
    for i in 0..x.len().max(y.len()) {
        let (l, r) = (
            x.get(i).copied().unwrap_or(0),
            y.get(i).copied().unwrap_or(0),
        );
        if l != r {
            return l > r;
        }
    }
    false
}

// -- state ----------------------------------------------------------------

/// What an installed skill looks like relative to the binary's embedded copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillState {
    /// Nothing installed at that path.
    Absent,
    /// Byte-identical to what this binary would write. Nothing to do.
    Current,
    /// Untouched since we wrote it, and this binary is strictly newer — the
    /// only state that is safe to upgrade automatically.
    Upgradable { from: String },
    /// Untouched since we wrote it, but this binary is *not* newer while the
    /// content differs. Left alone deliberately: auto-writing here is what
    /// makes two co-installed binaries (a release on PATH and a dev build)
    /// ping-pong, each "fixing" the other forever.
    Held { installed: String },
    /// Edited since we wrote it. Never overwritten without an explicit force.
    Modified { installed: String },
    /// Present with no stamp, but byte-identical to what this binary would
    /// write: a pre-stamp install that nobody has touched. Adopting it (writing
    /// the same content, now stamped) destroys nothing, so it needs no force.
    ///
    /// Without this, every skill installed before stamping existed would sit
    /// `Unmanaged` forever and auto-sync could never help it — which is most of
    /// the installed base, and the whole point of the feature.
    Adoptable,
    /// Present, unstamped, and *not* what we would write — hand-written, or
    /// another tool's. Treated like `Modified`: never overwritten without an
    /// explicit force.
    Unmanaged,
    /// The target resolves into a yaks checkout's own `.agents/skills/` —
    /// the repo itself, or a `~/.agents/skills/yaks` symlink pointing at it, a
    /// common dev setup. There is nothing to install: the installed skill *is* the source.
    /// Never written, not even with `--force` (yaks-d8e9).
    SourceLinked,
}

impl SkillState {
    /// True when writing would override something this binary has no business
    /// overwriting silently: a local edit, another tool's file, or an install
    /// from a yaks at least as new as this one (a downgrade).
    pub fn needs_force(&self) -> bool {
        matches!(
            self,
            SkillState::Modified { .. }
                | SkillState::Unmanaged
                | SkillState::Held { .. }
                | SkillState::SourceLinked
        )
    }

    /// True when the blocker is local content we'd destroy, as opposed to a
    /// merely newer install.
    pub fn has_local_edits(&self) -> bool {
        matches!(self, SkillState::Modified { .. } | SkillState::Unmanaged)
    }

    /// Short word for CLI output.
    pub fn word(&self) -> &'static str {
        match self {
            SkillState::Absent => "absent",
            SkillState::Current => "current",
            SkillState::Upgradable { .. } => "stale",
            SkillState::Held { .. } => "held",
            SkillState::Modified { .. } => "modified",
            SkillState::Adoptable => "adoptable",
            SkillState::Unmanaged => "unmanaged",
            SkillState::SourceLinked => "source",
        }
    }
}

/// Classify the installed copy of `content` at `path`.
pub fn inspect(path: &Path, content: &str) -> SkillState {
    // Checked first: writing here would rewrite the source of truth, which is
    // true regardless of what the file currently contains.
    if path.parent().is_some_and(is_source_tree) {
        return SkillState::SourceLinked;
    }
    let Ok(installed) = std::fs::read_to_string(path) else {
        return SkillState::Absent;
    };
    let Some((s, written)) = read_stamp(&installed) else {
        // Unstamped. If it is exactly what we would install anyway, stamping it
        // is lossless; otherwise it is someone else's file.
        return if installed == content {
            SkillState::Adoptable
        } else {
            SkillState::Unmanaged
        };
    };
    if digest(&written) != s.digest {
        return SkillState::Modified {
            installed: s.version,
        };
    }
    if written == content {
        return SkillState::Current;
    }
    if version_gt(version(), &s.version) {
        SkillState::Upgradable { from: s.version }
    } else {
        SkillState::Held {
            installed: s.version,
        }
    }
}

/// Classify a skill's non-`SKILL.md` file `rel`, installed at `path`.
///
/// Such a file carries no stamp of its own; it is compared by content, and the
/// digest the skill's `SKILL.md` stamp recorded for it (`recorded`) tells a
/// copy an older yaks wrote (untouched → upgradable) from one edited since
/// (`Modified`). With no record it is nobody's we know of (`Unmanaged`).
fn inspect_extra(path: &Path, rel: &str, content: &str, recorded: Option<&Stamp>) -> SkillState {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return SkillState::Absent,
        Err(_) => return SkillState::Unmanaged,
    };
    if bytes == content.as_bytes() {
        return SkillState::Current;
    }
    let Some(stamp) = recorded else {
        return SkillState::Unmanaged;
    };
    match stamp.files.iter().find(|(p, _)| p == rel) {
        None => SkillState::Unmanaged,
        Some((_, d)) if *d != digest(&bytes) => SkillState::Modified {
            installed: stamp.version.clone(),
        },
        Some(_) if version_gt(version(), &stamp.version) => SkillState::Upgradable {
            from: stamp.version.clone(),
        },
        Some(_) => SkillState::Held {
            installed: stamp.version.clone(),
        },
    }
}

/// How much attention a state wants; a skill reports its most demanding file.
fn severity(s: &SkillState) -> u8 {
    match s {
        SkillState::Current => 0,
        SkillState::Absent => 1,
        SkillState::Adoptable => 2,
        SkillState::Upgradable { .. } => 3,
        SkillState::Held { .. } => 4,
        SkillState::Unmanaged => 5,
        SkillState::Modified { .. } => 6,
        SkillState::SourceLinked => 7,
    }
}

/// Classify an installed skill: the most demanding state among its files, and
/// the path of the file that state is about. (Skills are installed, upgraded
/// and protected as a unit.)
pub fn inspect_skill(base: &Path, skill: &Skill) -> (PathBuf, SkillState) {
    let dir = base.join(skill.name);
    let md_path = dir.join("SKILL.md");
    let mut worst = (md_path.clone(), inspect(&md_path, skill.skill_md()));
    if worst.1 == SkillState::SourceLinked {
        return worst;
    }
    let recorded = std::fs::read_to_string(&md_path)
        .ok()
        .and_then(|t| read_stamp(&t))
        .map(|(s, _)| s);
    for f in skill.extras() {
        let path = dir.join(f.path);
        let state = inspect_extra(&path, f.path, f.content, recorded.as_ref());
        if severity(&state) > severity(&worst.1) {
            worst = (path, state);
        }
    }
    worst
}

/// One skill's situation in a skills directory.
pub struct Status {
    pub name: String,
    /// The file `state` is about (`SKILL.md` unless another file is the issue).
    pub path: PathBuf,
    pub state: SkillState,
}

/// Classify each of `skills` in `base`.
pub fn status(base: &Path, skills: &[&Skill]) -> Vec<Status> {
    skills
        .iter()
        .map(|skill| {
            let (path, state) = inspect_skill(base, skill);
            Status {
                name: skill.name.to_string(),
                path,
                state,
            }
        })
        .collect()
}

// -- the source-tree guard ------------------------------------------------

/// True when `dir` is the skills directory of a yaks source tree: either
/// `<checkout>/.agents/skills` (where this repo's skills live and the source of
/// truth — yaks-0576) or a plain `<checkout>/skills` (the layout before
/// yaks-0576; still recognised so an older checkout or branch stays protected).
/// `<checkout>` is identified by a `Cargo.toml` naming the package `yaks`.
fn is_yaks_skills_dir(dir: &Path) -> bool {
    if dir.file_name().and_then(|s| s.to_str()) != Some("skills") {
        return false;
    }
    let Some(parent) = dir.parent() else {
        return false;
    };
    let root = if parent.file_name().and_then(|s| s.to_str()) == Some(".agents") {
        match parent.parent() {
            Some(root) => root,
            None => return false,
        }
    } else {
        parent
    };
    let Ok(text) = std::fs::read_to_string(root.join("Cargo.toml")) else {
        return false;
    };
    text.lines()
        .any(|l| l.trim().replace(' ', "") == "name=\"yaks\"")
}

/// True when `path` — **after following symlinks** — lives inside a yaks source
/// tree's skills directory (see [`is_yaks_skills_dir`]: `.agents/skills/`).
///
/// Writing there overwrites the *source of truth* with the binary's baked-in
/// copy: it silently reverts real edits and, in `git status`, looks exactly
/// like an authored change (yaks-d8e9, which happened twice).
///
/// Resolving symlinks is the whole point. A common dev setup symlinks
/// `~/.agents/skills/yaks` at the repo's `.agents/skills/yaks`, so a perfectly
/// innocent-looking `yaks skills install` (default dir, no `--dir` at all)
/// lands on the source through the link. Checking only the path we were handed
/// misses that entirely — which is exactly how the original incident happened.
pub fn is_source_tree(path: &Path) -> bool {
    // Resolve as much of the path as exists; a not-yet-created target can't be
    // a symlink to anything, so the literal path is the right fallback.
    let resolved = path.canonicalize().unwrap_or_else(|_| {
        match path.parent().and_then(|p| p.canonicalize().ok()) {
            Some(parent) => match path.file_name() {
                Some(name) => parent.join(name),
                None => path.to_path_buf(),
            },
            None => path.to_path_buf(),
        }
    });
    resolved.ancestors().any(is_yaks_skills_dir)
}

// -- install --------------------------------------------------------------

/// Result of considering one bundled skill for installation.
pub struct Installed {
    pub name: String,
    /// The file `before` is about (`SKILL.md` unless another file is the issue).
    pub path: PathBuf,
    /// What the target looked like before we acted (the most demanding state
    /// among the skill's files).
    pub before: SkillState,
    /// Whether the skill was (re)written.
    pub wrote: bool,
}

impl Installed {
    /// True when the file was left alone because it would need `--force`.
    pub fn blocked(&self) -> bool {
        !self.wrote && self.before.needs_force()
    }
}

/// One result of [`install`] as the line(s) the CLI prints, shared by
/// `skills install` and `init --skills` so both say the same thing.
/// `force_hint` is appended to a skip that `--force` would override (and
/// omitted where nothing could, i.e. a source-linked target).
pub fn render_installed(i: &Installed, force_hint: &str) -> String {
    if i.wrote {
        let verb = match &i.before {
            SkillState::Upgradable { from } => format!("upgraded (from {from})"),
            SkillState::Absent => "installed".to_string(),
            _ => "rewrote".to_string(),
        };
        format!("{verb} {} -> {}", i.name, i.path.display())
    } else if i.blocked() {
        let why = match &i.before {
            SkillState::SourceLinked => "it resolves into yaks' own \
                 .agents/skills/ source \u{2014} the installed skill IS the \
                 source, so there is nothing to install"
                .to_string(),
            SkillState::Held { installed } => format!(
                "it was installed by yaks {installed}, which is not older \
                 than this one \u{2014} refusing to downgrade"
            ),
            SkillState::Unmanaged => {
                "it has no yaks stamp (hand-written, or another tool's)".to_string()
            }
            _ => "it was edited since it was installed".to_string(),
        };
        // Only offer --force where it would actually help; it can never
        // override a source-linked target.
        let hint = if matches!(i.before, SkillState::SourceLinked) {
            ""
        } else {
            force_hint
        };
        format!(
            "skip {} [{}]: {}\n       {why}{hint}",
            i.name,
            i.before.word(),
            i.path.display(),
        )
    } else {
        format!("ok {} is already current", i.name)
    }
}

/// Write `content` to `path` atomically, so parallel `yaks` invocations (the
/// coordinator spawns many) can never observe or leave a half-written file.
fn write_atomic(path: &Path, content: &str, executable: bool) -> Result<()> {
    let tmp = path.with_extension(format!("tmp{}", std::process::id()));
    std::fs::write(&tmp, content).with_context(|| format!("writing {}", tmp.display()))?;
    #[cfg(unix)]
    if executable {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755))
            .with_context(|| format!("making {} executable", tmp.display()))?;
    }
    #[cfg(not(unix))]
    let _ = executable;
    std::fs::rename(&tmp, path).with_context(|| format!("installing {}", path.display()))?;
    Ok(())
}

/// Write every file of `skill` under `base`, `SKILL.md` last: it carries the
/// stamp (version, its own digest, and the digest of each other file), so a
/// half-finished write leaves no stamp claiming the skill is current.
fn write_skill(base: &Path, skill: &Skill) -> Result<()> {
    let dir = base.join(skill.name);
    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    let mut digests = Vec::new();
    for f in skill.extras() {
        write_atomic(&dir.join(f.path), f.content, f.executable)?;
        digests.push((f.path.to_string(), digest(f.content)));
    }
    let stamped = stamp_with_files(skill.skill_md(), version(), &digests);
    write_atomic(&dir.join("SKILL.md"), &stamped, false)
}

/// Install `skills` (see [`select`]) into `base`.
///
/// A skill is installed, upgraded and protected as a unit: files we wrote and
/// that are still untouched are upgraded freely, but if any file of it is
/// `Modified`, `Unmanaged` or `Held`, none is written unless `force` is set.
/// Writing into this repo's own `.agents/skills/` is refused outright (see
/// [`is_source_tree`]).
pub fn install(base: &Path, force: bool, skills: &[&Skill]) -> Result<Vec<Installed>> {
    if is_source_tree(base) {
        anyhow::bail!(
            "refusing to install into {} \u{2014} that is yaks' own .agents/skills/ source, \
             and overwriting it would revert the real files to this binary's \
             baked-in copy (yaks-d8e9). Install somewhere else: `--user` \
             (~/.agents/skills) or `--dir <path>` (e.g. `--dir ~/.claude/skills`).",
            base.display()
        );
    }
    let mut out = Vec::new();
    for skill in skills {
        let (path, before) = inspect_skill(base, skill);
        let wrote = match &before {
            SkillState::Current => false,
            // Never written, force or not: it resolves onto the source.
            SkillState::SourceLinked => false,
            s if s.needs_force() && !force => false,
            _ => {
                write_skill(base, skill)?;
                true
            }
        };
        out.push(Installed {
            name: skill.name.to_string(),
            path,
            before,
            wrote,
        });
    }
    Ok(out)
}

/// Bring the user-level skills directory up to date on startup, quietly.
///
/// The overwhelmingly common failure is running against stale skills, so this
/// runs on ordinary `yaks` invocations. It is deliberately narrow:
///
/// - only `Absent` and `Upgradable` are written — never `Modified`,
///   `Unmanaged`, or `Held`, so it can neither clobber your edits nor
///   downgrade a newer install;
/// - only the user-level [`default_dir`], never a project-local
///   `.agents/skills` (those are files in someone's working tree; they get
///   [`project_stale_notice`] instead) and never a `--dir` target;
/// - only the default skill set, never the opt-in groups;
/// - skipped entirely when [`AUTOSYNC_ENV`] is `0`/`false`/`never`, for CI and
///   sandboxes.
///
/// Returns the names it wrote, for a one-line notice; empty means it did
/// nothing, which is the normal case.
pub fn auto_sync() -> Vec<String> {
    if !skills_notices_enabled() {
        return Vec::new();
    }
    auto_sync_in(&default_dir())
}

/// False when [`AUTOSYNC_ENV`] is `0`/`false`/`never`: the one switch for
/// everything yaks does about skills on an ordinary command, both the
/// user-level sync ([`auto_sync`]) and the project-local notice
/// ([`project_stale_notice`]).
fn skills_notices_enabled() -> bool {
    match std::env::var(AUTOSYNC_ENV) {
        Ok(v) => !matches!(v.trim(), "0" | "false" | "never"),
        Err(_) => true,
    }
}

/// [`auto_sync`] against an explicit user-level directory (the seam its tests use).
fn auto_sync_in(base: &Path) -> Vec<String> {
    if is_source_tree(base) {
        return Vec::new();
    }
    let mut done = Vec::new();
    for skill in select(&[]) {
        // Never write through a symlink into a yaks checkout (see install).
        if is_source_tree(&base.join(skill.name)) {
            continue;
        }
        match inspect_skill(base, skill).1 {
            SkillState::Absent | SkillState::Upgradable { .. } | SkillState::Adoptable => {
                if write_skill(base, skill).is_ok() {
                    done.push(skill.name.to_string());
                }
            }
            // Current / Held / Modified / Unmanaged: leave it alone.
            _ => {}
        }
    }
    done
}

// -- the stale project-local notice ---------------------------------------

/// A project-local install that has fallen behind this yaks: one or more of
/// its skills is `stale` ([`SkillState::Upgradable`]). Built from the same
/// states `yaks skills status` prints, so the notice, `status` and `yaks
/// doctor` cannot disagree about what is stale or what to run.
pub struct Stale {
    /// How many skills are stale.
    pub count: usize,
    /// The oldest yaks version that wrote a stale skill.
    pub from: String,
    /// Opt-in groups with a stale skill, so the advice repeats `--with <group>`
    /// (a bare `install` would skip them).
    pub with: Vec<&'static str>,
}

impl Stale {
    /// The stale skills among `statuses`, or `None` when nothing is stale.
    /// A skill whose most demanding file is `modified`, `unmanaged`, `held` or
    /// `source` is not stale: it is the user's call and says so elsewhere.
    pub fn of(statuses: &[Status]) -> Option<Stale> {
        let mut from: Option<String> = None;
        let mut with: Vec<&'static str> = Vec::new();
        let mut count = 0;
        for s in statuses {
            let SkillState::Upgradable { from: f } = &s.state else {
                continue;
            };
            count += 1;
            if from.as_deref().is_none_or(|cur| version_gt(cur, f)) {
                from = Some(f.clone());
            }
            let group = BUNDLED
                .iter()
                .find(|b| b.name == s.name)
                .and_then(|b| b.group);
            if let Some(g) = group {
                if !with.contains(&g) {
                    with.push(g);
                }
            }
        }
        from.map(|from| Stale { count, from, with })
    }

    /// The command that brings these up to date, with `target_flag` (`" --user"`,
    /// `" --dir <path>"`, or empty for the project-local default) after `install`.
    pub fn command(&self, target_flag: &str) -> String {
        let mut c = format!("yaks skills install{target_flag}");
        for g in &self.with {
            c.push_str(&format!(" --with {g}"));
        }
        c
    }

    /// The one-sentence advice for a project-local install, shared by the
    /// startup notice (`note: ` in front) and the `yaks doctor` advisory.
    pub fn message(&self) -> String {
        format!(
            "the skills in {PROJECT_SKILLS_DIR} are from yaks {}, this is {}: \
             run `{}` to update them",
            self.from,
            version(),
            self.command("")
        )
    }
}

/// Where a project-local install lives, relative to the git top-level.
const PROJECT_SKILLS_DIR: &str = ".agents/skills";

/// File in the checkout's git dir recording the newest yaks version that
/// already told the user about a stale project-local install.
const NOTIFIED_FILE: &str = "yaks-skills-notified";

/// The stale skills of the project-local install in `cwd`'s git repo, if any.
/// Only skills of [`BUNDLED`] that are actually installed there count (see
/// [`select_for_status`]); an absent skill is not stale.
pub fn project_stale(cwd: &Path) -> Option<Stale> {
    let root = git_toplevel(cwd)?;
    let base = root.join(PROJECT_SKILLS_DIR);
    Stale::of(&status(&base, &select_for_status(&base, &[])))
}

/// The checkout's git dir: `<root>/.git` itself, or, for a linked worktree or
/// submodule where `.git` is a file, the directory its `gitdir:` line names.
/// Per checkout, so each Delta clone or worktree has its own.
fn git_dir(root: &Path) -> Option<PathBuf> {
    let dot_git = root.join(".git");
    if dot_git.is_dir() {
        return Some(dot_git);
    }
    let text = std::fs::read_to_string(&dot_git).ok()?;
    let target = text.lines().find_map(|l| l.strip_prefix("gitdir:"))?.trim();
    Some(root.join(target)) // an absolute `target` replaces `root`
}

/// The notice an ordinary command should print to stderr, or `None`.
///
/// Project-local skills are files in someone's working tree, so yaks never
/// rewrites them on its own (yaks-3859); instead it says so, **once per yaks
/// version per checkout**. "Told" is remembered in a marker file in the
/// checkout's git dir (see [`NOTIFIED_FILE`]): untracked, outside the working
/// tree, and per clone/worktree. Silent when [`AUTOSYNC_ENV`] is off, when
/// nothing is stale (see [`Stale::of`]), when the marker already holds this
/// version or a newer one (so two co-installed yaks don't take turns nagging),
/// and when the marker cannot be written (a read-only `.git`, as in some
/// sandboxes: a notice we cannot remember would repeat on every command;
/// `status` and `doctor` still report it). The marker write is best-effort and
/// never fails the command.
pub fn project_stale_notice(cwd: &Path) -> Option<String> {
    if !skills_notices_enabled() {
        return None;
    }
    notice_in(cwd)
}

/// [`project_stale_notice`] without the environment switch (the seam its unit
/// tests use; the switch itself is covered end to end).
fn notice_in(cwd: &Path) -> Option<String> {
    let root = git_toplevel(cwd)?;
    let marker = git_dir(&root)?.join(NOTIFIED_FILE);
    if let Ok(told) = std::fs::read_to_string(&marker) {
        if !version_gt(version(), told.trim()) {
            return None;
        }
    }
    let stale = project_stale(cwd)?;
    std::fs::write(&marker, format!("{}\n", version())).ok()?;
    Some(stale.message())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Top-level frontmatter keys of a SKILL.md (the `key:` lines between the
    /// opening and closing `---`, ignoring nested/indented lines and comments).
    fn frontmatter_keys(src: &str) -> Vec<String> {
        let mut lines = src.lines();
        assert_eq!(
            lines.next().map(str::trim),
            Some("---"),
            "SKILL.md must open with frontmatter"
        );
        let mut keys = Vec::new();
        for line in lines {
            if line.trim() == "---" {
                break;
            }
            // Only top-level keys: skip indented (nested) lines and comments.
            if line.starts_with(' ') || line.starts_with('\t') || line.trim_start().starts_with('#')
            {
                continue;
            }
            if let Some((k, _)) = line.split_once(':') {
                keys.push(k.trim().to_string());
            }
        }
        keys
    }

    #[test]
    fn bundled_skills_use_only_spec_frontmatter_keys() {
        // Guards yaks-e233: a non-spec key (we shipped `activation:`) is
        // silently ignored by Claude Code but HARD-ERRORS on claude.ai upload /
        // the Skills API / package_skill.py, so it can't be caught by using the
        // skill locally. Put custom data under `metadata:` instead.
        for skill in BUNDLED {
            let (name, content) = (skill.name, skill.skill_md());
            let keys = frontmatter_keys(content);
            for k in &keys {
                assert!(
                    SPEC_FRONTMATTER_KEYS.contains(&k.as_str()),
                    "skill {name:?} has non-spec frontmatter key {k:?}; \
                     allowed: {SPEC_FRONTMATTER_KEYS:?} (put custom data under `metadata:`)"
                );
            }
            for required in ["name", "description"] {
                assert!(
                    keys.iter().any(|k| k == required),
                    "skill {name:?} is missing required frontmatter key {required:?}"
                );
            }
            let declared = content
                .lines()
                .find_map(|l| l.strip_prefix("name:"))
                .map(str::trim)
                .unwrap_or_default();
            assert_eq!(
                declared, name,
                "skill {name:?} declares a `name` that doesn't match its directory"
            );
        }
    }

    #[test]
    fn bundled_source_is_never_itself_stamped() {
        // The embedded skills must be PRISTINE: a stamp belongs only to an
        // installed copy. If a stamp ever leaks back into .agents/skills/*/SKILL.md,
        // the build embeds it, every install double-stamps, and `inspect`
        // starts comparing stamped content against stamped content. It has
        // happened (a stray install pointed at the repo), so assert it loudly
        // rather than trusting the write-path guard alone.
        for skill in BUNDLED {
            let (name, content) = (skill.name, skill.skill_md());
            assert!(
                read_stamp(content).is_none(),
                ".agents/skills/{name}/SKILL.md carries a provenance stamp; the source \
                 must stay unstamped \u{2014} run `git checkout -- .agents/skills/`"
            );
            assert!(
                !content.contains(K_VERSION)
                    && !content.contains(K_DIGEST)
                    && !content.contains(K_FILES),
                ".agents/skills/{name}/SKILL.md mentions a stamp key; the source must stay pristine"
            );
        }
    }

    #[test]
    fn stamp_round_trips_and_stays_spec_legal() {
        let src = BUNDLED[0].skill_md();
        let out = stamp(src, "1.2.3");
        // The stamp lands in the frontmatter, under the spec's metadata: field.
        assert!(out.contains("metadata:\n  yaks-version: \"1.2.3\""));
        assert_eq!(frontmatter_keys(&out).last().unwrap(), "metadata");
        for k in frontmatter_keys(&out) {
            assert!(SPEC_FRONTMATTER_KEYS.contains(&k.as_str()), "key {k:?}");
        }
        // Stripping it back recovers the pristine source exactly.
        let (s, written) = read_stamp(&out).expect("stamped file parses");
        assert_eq!(s.version, "1.2.3");
        assert_eq!(written, src);
        assert_eq!(s.digest, digest(src));
    }

    #[test]
    fn version_comparison_is_numeric_not_lexical() {
        assert!(version_gt("0.0.10", "0.0.9")); // the lexical trap
        assert!(version_gt("0.1.0", "0.0.99"));
        assert!(!version_gt("0.0.9", "0.0.9"));
        assert!(!version_gt("0.0.9", "0.0.10"));
    }

    fn temp_base(tag: &str) -> PathBuf {
        let mut base = std::env::temp_dir();
        base.push(format!(
            "yaks-skills-{tag}-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&base);
        base
    }

    #[test]
    fn states_absent_current_modified_and_unmanaged() {
        let base = temp_base("states");
        let (name, content) = (BUNDLED[0].name, BUNDLED[0].skill_md());
        let path = base.join(name).join("SKILL.md");

        assert_eq!(inspect(&path, content), SkillState::Absent);

        install(&base, false, &select(&[])).unwrap();
        assert_eq!(inspect(&path, content), SkillState::Current);

        // A hand edit is detected and protected.
        let edited = std::fs::read_to_string(&path).unwrap() + "\nlocal note\n";
        std::fs::write(&path, &edited).unwrap();
        assert!(matches!(
            inspect(&path, content),
            SkillState::Modified { .. }
        ));
        let res = install(&base, false, &select(&[])).unwrap();
        let me = res.iter().find(|i| i.name == name).unwrap();
        assert!(!me.wrote && me.blocked(), "modified file is protected");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), edited);
        // ...until forced.
        let res = install(&base, true, &select(&[])).unwrap();
        assert!(res.iter().find(|i| i.name == name).unwrap().wrote);
        assert_eq!(inspect(&path, content), SkillState::Current);

        // An unstamped file is someone else's; also protected.
        std::fs::write(&path, "---\nname: yaks\ndescription: hand rolled\n---\n").unwrap();
        assert_eq!(inspect(&path, content), SkillState::Unmanaged);
        let res = install(&base, false, &select(&[])).unwrap();
        assert!(res.iter().find(|i| i.name == name).unwrap().blocked());

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn untouched_prestamp_install_is_adopted_without_force() {
        // The migration path: skills installed before stamping existed carry no
        // stamp. When the content is still exactly ours, adopting it is
        // lossless, so it must not demand --force -- otherwise the whole
        // installed base stays unmanaged and auto-sync never helps it.
        let base = temp_base("adopt");
        let (name, content) = (BUNDLED[0].name, BUNDLED[0].skill_md());
        let dir = base.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("SKILL.md");
        // Exactly what an old yaks would have written: our content, no stamp.
        std::fs::write(&path, content).unwrap();
        assert_eq!(inspect(&path, content), SkillState::Adoptable);
        assert!(!inspect(&path, content).needs_force());

        let res = install(&base, false, &select(&[])).unwrap();
        assert!(
            res.iter().find(|i| i.name == name).unwrap().wrote,
            "an untouched pre-stamp install adopts with no --force"
        );
        assert_eq!(inspect(&path, content), SkillState::Current);

        // But a DIVERGENT unstamped file is someone else's and stays protected.
        std::fs::write(&path, "---\nname: yaks\ndescription: theirs\n---\n").unwrap();
        assert_eq!(inspect(&path, content), SkillState::Unmanaged);
        assert!(inspect(&path, content).needs_force());

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn older_binary_holds_instead_of_downgrading() {
        // The ping-pong guard (yaks-1d51): a file stamped by a NEWER yaks must
        // not be rewritten by this one, or two co-installed binaries flip-flop.
        let base = temp_base("hold");
        let (name, content) = (BUNDLED[0].name, BUNDLED[0].skill_md());
        let dir = base.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("SKILL.md");
        // Stamp altered content at an implausibly high version.
        let future = format!("{content}\n<!-- from the future -->\n");
        std::fs::write(&path, stamp(&future, "99.0.0")).unwrap();

        assert!(matches!(inspect(&path, content), SkillState::Held { .. }));
        let before = std::fs::read_to_string(&path).unwrap();
        // Neither auto-sync nor a plain install may touch it.
        let res = install(&base, false, &select(&[])).unwrap();
        assert!(!res.iter().find(|i| i.name == name).unwrap().wrote);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), before);

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn stale_install_is_upgraded_without_force() {
        let base = temp_base("stale");
        let (name, content) = (BUNDLED[0].name, BUNDLED[0].skill_md());
        let dir = base.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("SKILL.md");
        // An old, unmodified install of different content.
        let old = format!("{content}\n<!-- old -->\n");
        std::fs::write(&path, stamp(&old, "0.0.0")).unwrap();

        assert!(matches!(
            inspect(&path, content),
            SkillState::Upgradable { .. }
        ));
        let res = install(&base, false, &select(&[])).unwrap();
        assert!(
            res.iter().find(|i| i.name == name).unwrap().wrote,
            "a clean stale copy upgrades with no --force"
        );
        assert_eq!(inspect(&path, content), SkillState::Current);

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn refuses_to_install_into_the_yaks_source_tree() {
        // yaks-d8e9: not bypassable by --force, since --force caused it.
        // `.agents/skills` is this repo's source of truth (yaks-0576); the
        // pre-0576 plain `skills/` layout stays protected too.
        for layout in [".agents/skills", "skills"] {
            let root = temp_base("srctree");
            let base = root.join(layout);
            std::fs::create_dir_all(&base).unwrap();
            std::fs::write(
                root.join("Cargo.toml"),
                "[package]\nname = \"yaks\"\nversion = \"0.0.1\"\n",
            )
            .unwrap();
            assert!(is_source_tree(&base), "{layout} is a yaks source tree");
            assert!(install(&base, false, &select(&[])).is_err(), "{layout}");
            assert!(
                install(&base, true, &select(&[])).is_err(),
                "--force must NOT be the escape hatch here ({layout})"
            );
            let _ = std::fs::remove_dir_all(&root);
        }
    }

    #[test]
    fn a_skills_dir_outside_a_yaks_checkout_is_not_the_source_tree() {
        // `.agents/skills` is also the normal install target (home dir, other
        // projects): only a parent Cargo.toml naming the package `yaks` makes
        // it the source.
        let root = temp_base("notsrc");
        let base = root.join(".agents").join("skills");
        std::fs::create_dir_all(&base).unwrap();
        assert!(!is_source_tree(&base), "no Cargo.toml");
        std::fs::write(root.join("Cargo.toml"), "[package]\nname = \"other\"\n").unwrap();
        assert!(!is_source_tree(&base), "a different package");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn install_writes_both_then_is_idempotent_then_forces() {
        let base = temp_base("install");

        let first = install(&base, false, &select(&[])).unwrap();
        assert_eq!(first.len(), 2);
        assert!(first.iter().all(|i| i.wrote));
        assert!(base.join("yaks/SKILL.md").is_file());
        assert!(base.join("yaks-tracker/SKILL.md").is_file());
        let yak = std::fs::read_to_string(base.join("yaks/SKILL.md")).unwrap();
        assert!(yak.contains("name: yaks"));

        // Second run is a no-op: identical content needs no rewrite.
        let again = install(&base, false, &select(&[])).unwrap();
        assert!(
            again
                .iter()
                .all(|i| !i.wrote && i.before == SkillState::Current)
        );

        // Force on an identical file stays a no-op: there is nothing to
        // refresh, and rewriting would only churn mtimes.
        let forced = install(&base, true, &select(&[])).unwrap();
        assert!(forced.iter().all(|i| !i.wrote));

        // And status agrees.
        assert!(
            status(&base, &select(&[]))
                .iter()
                .all(|s| s.state == SkillState::Current)
        );

        let _ = std::fs::remove_dir_all(&base);
    }

    fn coordination() -> Vec<String> {
        vec!["coordination".to_string()]
    }

    const SIX: [&str; 6] = [
        "yaks-coordinating",
        "yaks-coordinating-team",
        "yaks-coordinating-private",
        "yaks-coordinating-worktrees",
        "yaks-coordinating-delta",
        "yaks-working",
    ];

    #[test]
    fn default_set_is_exactly_yaks_and_yaks_tracker() {
        let names: Vec<_> = select(&[]).iter().map(|s| s.name).collect();
        assert_eq!(names, ["yaks", "yaks-tracker"]);
        let base = temp_base("default-set");
        install(&base, false, &select(&[])).unwrap();
        let mut dirs: Vec<_> = std::fs::read_dir(&base)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        dirs.sort();
        assert_eq!(dirs, ["yaks", "yaks-tracker"]);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn with_coordination_adds_the_six_skills_and_the_land_script() {
        let names: Vec<_> = select(&coordination()).iter().map(|s| s.name).collect();
        assert_eq!(names.len(), 8);
        for n in ["yaks", "yaks-tracker"].into_iter().chain(SIX) {
            assert!(names.contains(&n), "{n}");
        }
        let base = temp_base("with-coord");
        let res = install(&base, false, &select(&coordination())).unwrap();
        assert!(res.iter().all(|i| i.wrote));
        for n in SIX {
            assert!(base.join(n).join("SKILL.md").is_file(), "{n}");
        }
        let land = base.join("yaks-coordinating-delta/land.sh");
        assert_eq!(
            std::fs::read_to_string(&land).unwrap(),
            include_str!("../.agents/skills/yaks-coordinating-delta/land.sh")
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&land).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o755, "land.sh must stay executable");
        }
        // Idempotent, and status sees all eight as current.
        let again = install(&base, false, &select(&coordination())).unwrap();
        assert!(again.iter().all(|i| !i.wrote));
        let st = status(&base, &select_for_status(&base, &[]));
        assert_eq!(
            st.len(),
            8,
            "installed opt-in skills are tracked without --with"
        );
        assert!(st.iter().all(|s| s.state == SkillState::Current));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn an_edited_script_is_reported_and_kept_unless_forced() {
        let base = temp_base("script-edit");
        let skills = select(&coordination());
        install(&base, false, &skills).unwrap();
        let land = base.join("yaks-coordinating-delta/land.sh");
        let edited = std::fs::read_to_string(&land).unwrap() + "\n# local tweak\n";
        std::fs::write(&land, &edited).unwrap();

        let delta = BUNDLED
            .iter()
            .find(|s| s.name == "yaks-coordinating-delta")
            .unwrap();
        let (path, state) = inspect_skill(&base, delta);
        assert!(matches!(state, SkillState::Modified { .. }), "{state:?}");
        assert_eq!(path, land, "the report names the edited file");

        let res = install(&base, false, &skills).unwrap();
        let r = res
            .iter()
            .find(|i| i.name == "yaks-coordinating-delta")
            .unwrap();
        assert!(!r.wrote && r.blocked());
        assert_eq!(std::fs::read_to_string(&land).unwrap(), edited);

        let res = install(&base, true, &skills).unwrap();
        assert!(
            res.iter()
                .find(|i| i.name == "yaks-coordinating-delta")
                .unwrap()
                .wrote
        );
        assert_ne!(std::fs::read_to_string(&land).unwrap(), edited);
        assert_eq!(inspect_skill(&base, delta).1, SkillState::Current);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn an_untouched_script_from_an_older_yaks_is_upgraded_without_force() {
        let base = temp_base("script-stale");
        let delta = BUNDLED
            .iter()
            .find(|s| s.name == "yaks-coordinating-delta")
            .unwrap();
        let dir = base.join(delta.name);
        std::fs::create_dir_all(&dir).unwrap();
        // What yaks 0.0.0 would have written: an older script, stamped with its digest.
        let old_script = "#!/bin/sh\necho old\n";
        std::fs::write(dir.join("land.sh"), old_script).unwrap();
        std::fs::write(
            dir.join("SKILL.md"),
            stamp_with_files(
                delta.skill_md(),
                "0.0.0",
                &[("land.sh".to_string(), digest(old_script))],
            ),
        )
        .unwrap();
        assert!(matches!(
            inspect_skill(&base, delta).1,
            SkillState::Upgradable { .. }
        ));
        let res = install(&base, false, &[delta]).unwrap();
        assert!(res[0].wrote);
        assert_eq!(inspect_skill(&base, delta).1, SkillState::Current);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn stamp_records_extra_file_digests_and_round_trips() {
        let src = BUNDLED[0].skill_md();
        let files = vec![("a.sh".to_string(), "00ff".to_string())];
        let out = stamp_with_files(src, "1.2.3", &files);
        let (s, written) = read_stamp(&out).unwrap();
        assert_eq!(s.files, files);
        assert_eq!(written, src);
        assert_eq!(frontmatter_keys(&out).last().unwrap(), "metadata");
    }

    #[test]
    fn the_file_list_matches_the_source_directories() {
        // BUNDLED is explicit, never directory contents — but a file added to a
        // bundled skill's directory and forgotten here would silently not ship.
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(".agents/skills");
        for skill in BUNDLED {
            assert_eq!(skill.files[0].path, "SKILL.md", "{}", skill.name);
            let mut on_disk: Vec<String> = std::fs::read_dir(root.join(skill.name))
                .unwrap()
                .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                .collect();
            on_disk.sort();
            let mut listed: Vec<String> = skill.files.iter().map(|f| f.path.to_string()).collect();
            listed.sort();
            assert_eq!(on_disk, listed, "{}: BUNDLED file list drifted", skill.name);
            #[cfg(unix)]
            for f in skill.files {
                use std::os::unix::fs::PermissionsExt;
                let mode = std::fs::metadata(root.join(skill.name).join(f.path))
                    .unwrap()
                    .permissions()
                    .mode();
                assert_eq!(
                    mode & 0o100 != 0,
                    f.executable,
                    "{}/{}: executable flag disagrees with the source file mode",
                    skill.name,
                    f.path
                );
            }
        }
        for g in BUNDLED.iter().filter_map(|s| s.group) {
            assert!(GROUPS.contains(&g), "group {g:?} missing from GROUPS");
        }
    }

    fn fake_repo(tag: &str) -> PathBuf {
        let root = temp_base(tag);
        std::fs::create_dir_all(root.join(".git")).unwrap();
        std::fs::create_dir_all(root.join("sub/deeper")).unwrap();
        root
    }

    #[test]
    fn default_target_is_project_local_at_the_git_top_level() {
        let root = fake_repo("tgt-project");
        let t = resolve_target(None, false, &root.join("sub/deeper"));
        assert_eq!(t.dir, root.join(".agents/skills"));
        assert_eq!(t.kind, TargetKind::Project { root: root.clone() });
        assert!(t.describe().contains("project-local"));
        // A linked worktree has a `.git` *file*, which counts too.
        let wt = temp_base("tgt-wt");
        std::fs::create_dir_all(&wt).unwrap();
        std::fs::write(wt.join(".git"), "gitdir: /elsewhere\n").unwrap();
        assert_eq!(
            resolve_target(None, false, &wt).dir,
            wt.join(".agents/skills")
        );
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&wt);
    }

    #[test]
    fn user_flag_targets_the_user_dir_even_inside_a_repo() {
        let root = fake_repo("tgt-user");
        let t = resolve_target(None, true, &root);
        assert_eq!(t.dir, default_dir());
        assert_eq!(t.kind, TargetKind::User);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn dir_flag_wins_over_everything() {
        let root = fake_repo("tgt-dir");
        for user in [false, true] {
            let t = resolve_target(Some("/some/where"), user, &root);
            assert_eq!(t.dir, PathBuf::from("/some/where"));
            assert_eq!(t.kind, TargetKind::Dir);
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn outside_a_git_repo_the_default_falls_back_to_the_user_dir() {
        let cwd = temp_base("tgt-nogit");
        std::fs::create_dir_all(&cwd).unwrap();
        assert!(
            git_toplevel(&cwd).is_none(),
            "temp dir must not be in a repo"
        );
        let t = resolve_target(None, false, &cwd);
        assert_eq!(t.dir, default_dir());
        assert_eq!(t.kind, TargetKind::UserFallback);
        assert!(t.describe().contains("not inside a git repo"));
        let _ = std::fs::remove_dir_all(&cwd);
    }

    #[test]
    fn in_a_yaks_checkout_the_default_target_is_refused_and_names_the_way_out() {
        let root = fake_repo("tgt-yaks");
        std::fs::write(root.join("Cargo.toml"), "[package]\nname = \"yaks\"\n").unwrap();
        let t = resolve_target(None, false, &root.join("sub"));
        let err = install(&t.dir, false, &select(&coordination()))
            .err()
            .expect("must refuse")
            .to_string();
        assert!(err.contains("--user") && err.contains("--dir"), "{err}");
        // ...and --force is still not the way out.
        assert!(install(&t.dir, true, &select(&[])).is_err());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn autosync_never_touches_a_project_local_install_or_the_opt_in_group() {
        // Auto-sync works on the one user-level directory it is handed; a stale
        // project-local install elsewhere is left exactly as it was.
        let project = temp_base("as-project");
        let user = temp_base("as-user");
        let skills = select(&coordination());
        install(&project, false, &skills).unwrap();
        // Make the project copy stale: an older yaks' stamp over older content.
        let md = project.join("yaks/SKILL.md");
        let old = format!("{}\n<!-- old -->\n", BUNDLED[0].skill_md());
        std::fs::write(&md, stamp(&old, "0.0.0")).unwrap();
        let before = std::fs::read_to_string(&md).unwrap();

        let wrote = auto_sync_in(&user);
        assert_eq!(wrote, ["yaks", "yaks-tracker"], "default set only");
        assert!(
            !user.join("yaks-working").exists(),
            "opt-in group not synced"
        );
        assert_eq!(std::fs::read_to_string(&md).unwrap(), before);
        let st = status(&project, &select(&[]));
        assert!(
            matches!(st[0].state, SkillState::Upgradable { .. }),
            "status reports the project install stale instead"
        );
        let _ = std::fs::remove_dir_all(&project);
        let _ = std::fs::remove_dir_all(&user);
    }

    // -- the stale project-local notice ------------------------------------

    /// Overwrite `name`'s SKILL.md in `base` with an older yaks' untouched copy
    /// of older content: the `stale` state.
    fn make_stale(base: &Path, name: &str) {
        let skill = BUNDLED.iter().find(|s| s.name == name).unwrap();
        let dir = base.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        let old = format!("{}\n<!-- old -->\n", skill.skill_md());
        std::fs::write(dir.join("SKILL.md"), stamp(&old, "0.0.0")).unwrap();
    }

    /// Every file under `dir` with its mtime and content, to prove a notice
    /// wrote nothing into the working tree.
    fn snapshot(dir: &Path) -> Vec<(PathBuf, std::time::SystemTime, Vec<u8>)> {
        fn walk(d: &Path, out: &mut Vec<(PathBuf, std::time::SystemTime, Vec<u8>)>) {
            let Ok(rd) = std::fs::read_dir(d) else { return };
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else {
                    let m = std::fs::metadata(&p).unwrap().modified().unwrap();
                    out.push((p.clone(), m, std::fs::read(&p).unwrap()));
                }
            }
        }
        let mut out = Vec::new();
        walk(dir, &mut out);
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    fn marker(root: &Path) -> PathBuf {
        root.join(".git").join(NOTIFIED_FILE)
    }

    #[test]
    fn a_stale_project_install_is_announced_once_per_version() {
        let root = fake_repo("ntf-once");
        let base = root.join(".agents/skills");
        install(&base, false, &select(&[])).unwrap();
        make_stale(&base, "yaks");
        let tree_before = snapshot(&root.join(".agents"));
        let sub = root.join("sub/deeper");

        let first = notice_in(&sub).expect("stale install is announced");
        assert_eq!(
            first,
            format!(
                "the skills in .agents/skills are from yaks 0.0.0, this is {}: \
                 run `yaks skills install` to update them",
                version()
            )
        );
        assert_eq!(
            std::fs::read_to_string(marker(&root)).unwrap().trim(),
            version()
        );
        assert_eq!(notice_in(&sub), None, "the second command is silent");
        assert_eq!(notice_in(&root), None);

        // A newer yaks (a marker from an older one) announces again...
        std::fs::write(marker(&root), "0.0.0\n").unwrap();
        assert!(
            notice_in(&sub).is_some(),
            "announced again after a version change"
        );
        // ...but an older yaks does not re-announce over a newer yaks' marker.
        std::fs::write(marker(&root), "99.0.0\n").unwrap();
        assert_eq!(notice_in(&sub), None);

        assert_eq!(
            snapshot(&root.join(".agents")),
            tree_before,
            "the notice writes nothing into the working tree"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn the_notice_is_silent_unless_a_skill_is_cleanly_stale() {
        // current
        let root = fake_repo("ntf-quiet");
        let base = root.join(".agents/skills");
        install(&base, false, &select(&[])).unwrap();
        assert_eq!(notice_in(&root), None, "current");

        // modified: edited after install
        let md = base.join("yaks/SKILL.md");
        let mut t = std::fs::read_to_string(&md).unwrap();
        t.push_str("\nmy own line\n");
        std::fs::write(&md, t).unwrap();
        assert_eq!(notice_in(&root), None, "modified");

        // held: installed by a yaks that is not older than this one
        let future = format!("{}\n<!-- from the future -->\n", BUNDLED[0].skill_md());
        std::fs::write(&md, stamp(&future, "99.0.0")).unwrap();
        assert_eq!(notice_in(&root), None, "held");

        // unmanaged: hand-written, no stamp
        std::fs::write(&md, "---\nname: yaks\ndescription: mine\n---\n").unwrap();
        assert_eq!(notice_in(&root), None, "unmanaged");

        // absent: nothing installed at all
        let bare = fake_repo("ntf-absent");
        assert_eq!(notice_in(&bare), None, "absent");

        // a stale skill beside a modified one is the user's call, not stale
        make_stale(&base, "yaks-tracker");
        make_stale(&base, "yaks");
        std::fs::write(
            base.join("yaks/SKILL.md"),
            format!(
                "{}\nedited\n",
                std::fs::read_to_string(base.join("yaks/SKILL.md")).unwrap()
            ),
        )
        .unwrap();
        assert!(
            notice_in(&root).is_some(),
            "the clean stale one still counts"
        );

        // not in a git repo at all
        let nogit = temp_base("ntf-nogit");
        std::fs::create_dir_all(&nogit).unwrap();
        assert_eq!(notice_in(&nogit), None, "no repo");

        for d in [&root, &bare, &nogit] {
            let _ = std::fs::remove_dir_all(d);
        }
    }

    #[test]
    fn the_notice_is_silent_in_a_yaks_source_checkout() {
        let root = fake_repo("ntf-source");
        std::fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"yaks\"\nversion = \"0.0.1\"\n",
        )
        .unwrap();
        // Even content that would be stale anywhere else.
        make_stale(&root.join(".agents/skills"), "yaks");
        assert!(project_stale(&root).is_none());
        assert_eq!(notice_in(&root), None);
        assert!(!marker(&root).exists());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn an_opt_in_stale_skill_adds_with_coordination_to_the_advice() {
        let root = fake_repo("ntf-optin");
        let base = root.join(".agents/skills");
        install(&base, false, &select(&coordination())).unwrap();
        make_stale(&base, "yaks-working");
        let msg = notice_in(&root).expect("stale");
        assert!(
            msg.ends_with("run `yaks skills install --with coordination` to update them"),
            "{msg}"
        );
        // The default-set-only case keeps the short form (asserted above), and
        // `status` words the same advice from the same value.
        let st = status(&base, &select_for_status(&base, &[]));
        let stale = Stale::of(&st).unwrap();
        assert_eq!(stale.count, 1);
        assert_eq!(
            stale.command(" --user"),
            "yaks skills install --user --with coordination"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn rerunning_install_after_the_notice_makes_it_current_and_quiet() {
        let root = fake_repo("ntf-fix");
        let base = root.join(".agents/skills");
        install(&base, false, &select(&[])).unwrap();
        make_stale(&base, "yaks");
        assert!(notice_in(&root).is_some());

        // `yaks skills install` (no flags, default set) is exactly what the
        // notice names, and it upgrades a cleanly stale skill with no --force.
        let res = install(&base, false, &select(&[])).unwrap();
        assert!(res.iter().find(|i| i.name == "yaks").unwrap().wrote);
        assert!(project_stale(&root).is_none());
        // Quiet even if the marker is forgotten.
        std::fs::remove_file(marker(&root)).unwrap();
        assert_eq!(notice_in(&root), None);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn the_marker_lives_in_a_linked_worktrees_own_git_dir() {
        let root = temp_base("ntf-wt");
        let gitdir = temp_base("ntf-wt-gitdir");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&gitdir).unwrap();
        // `.git` is a file in a linked worktree; an absolute gitdir...
        std::fs::write(root.join(".git"), format!("gitdir: {}\n", gitdir.display())).unwrap();
        let base = root.join(".agents/skills");
        install(&base, false, &select(&[])).unwrap();
        make_stale(&base, "yaks");
        assert!(notice_in(&root).is_some());
        assert!(gitdir.join(NOTIFIED_FILE).is_file());
        assert_eq!(notice_in(&root), None);
        // ...and a relative one resolves against the worktree root.
        std::fs::create_dir_all(root.join("meta")).unwrap();
        std::fs::write(root.join(".git"), "gitdir: meta\n").unwrap();
        assert!(notice_in(&root).is_some());
        assert!(root.join("meta").join(NOTIFIED_FILE).is_file());
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&gitdir);
    }

    #[test]
    fn an_unrecordable_notice_is_not_printed() {
        // A notice that cannot be remembered would repeat on every command.
        let root = fake_repo("ntf-unwritable");
        let base = root.join(".agents/skills");
        install(&base, false, &select(&[])).unwrap();
        make_stale(&base, "yaks");
        std::fs::create_dir_all(marker(&root)).unwrap(); // a directory: the write fails
        assert_eq!(notice_in(&root), None);
        let _ = std::fs::remove_dir_all(&root);
    }
}

/// The symlink case gets its own module so the guard that actually failed in
/// production is impossible to lose in a refactor of the big test module.
#[cfg(all(test, unix))]
mod symlink_guard_tests {
    use super::*;

    /// The real mechanism behind yaks-d8e9, found the hard way: a dev setup
    /// symlinks `~/.agents/skills/yaks` at the repo's copy of the skill (then
    /// `skills/yaks`, now `.agents/skills/yaks`), so a plain install with **no
    /// `--dir`** lands on the source through the link. Guarding only the
    /// literal path we were handed misses this entirely.
    #[test]
    fn a_symlink_onto_the_source_tree_is_never_written() {
        let mut tmp = std::env::temp_dir();
        tmp.push(format!("yaks-srclink-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);

        // A fake yaks checkout, with a precious source file.
        let src_dir = tmp.join("repo").join(".agents").join("skills").join("yaks");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(
            tmp.join("repo").join("Cargo.toml"),
            "[package]\nname = \"yaks\"\n",
        )
        .unwrap();
        let precious = src_dir.join("SKILL.md");
        std::fs::write(&precious, "PRECIOUS SOURCE").unwrap();

        // An innocent-looking skills dir whose `yaks` entry is a symlink to it.
        let base = tmp.join("home").join(".agents").join("skills");
        std::fs::create_dir_all(&base).unwrap();
        std::os::unix::fs::symlink(&src_dir, base.join("yaks")).unwrap();

        // The base itself is NOT a source tree; only the resolved entry is.
        assert!(!is_source_tree(&base), "base looks innocent");
        assert_eq!(
            inspect(&base.join("yaks").join("SKILL.md"), BUNDLED[0].skill_md()),
            SkillState::SourceLinked,
        );

        // Neither a plain install nor a forced one may write through the link.
        for force in [false, true] {
            install(&base, force, &select(&[])).unwrap();
            assert_eq!(
                std::fs::read_to_string(&precious).unwrap(),
                "PRECIOUS SOURCE",
                "source overwritten through the symlink (force={force})"
            );
        }
        // Auto-sync must not either (it targets the user dir, so prove the
        // state it keys off is the protective one).
        assert!(
            inspect(&base.join("yaks").join("SKILL.md"), BUNDLED[0].skill_md()).needs_force(),
            "source-linked must never be auto-written"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// The repo's own skills (`.agents/skills`) stay consistent, since agents load
    /// them by name and follow the references between them (yaks-5c9f).
    #[test]
    fn repo_skills_are_well_formed() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(".agents/skills");
        let mut names: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        assert!(!names.is_empty(), "no skills found in {}", dir.display());

        let field = |front: &str, key: &str| -> Option<String> {
            front
                .lines()
                .find_map(|l| l.strip_prefix(&format!("{key}:")))
                .map(|v| v.trim().to_string())
        };
        for n in &names {
            let text = std::fs::read_to_string(dir.join(n).join("SKILL.md"))
                .unwrap_or_else(|_| panic!("{n}: missing SKILL.md"));
            let rest = text
                .strip_prefix("---\n")
                .unwrap_or_else(|| panic!("{n}: SKILL.md must start with frontmatter"));
            let (front, _body) = rest
                .split_once("\n---\n")
                .unwrap_or_else(|| panic!("{n}: frontmatter is not closed"));
            assert_eq!(
                field(front, "name").as_deref(),
                Some(n.as_str()),
                "{n}: name must equal its directory"
            );
            let desc = field(front, "description").unwrap_or_default();
            assert!(
                desc.to_lowercase().contains("when"),
                "{n}: description must say when to use the skill"
            );
            // A plain (unquoted) YAML scalar may not contain `: ` or ` #`. The harness
            // refuses to load such a skill ("mapping values are not allowed"), and a
            // line-based read like this one cannot see that, so check it explicitly.
            if !(desc.starts_with('"') || desc.starts_with('\'')) {
                assert!(
                    !desc.contains(": ") && !desc.contains(" #"),
                    "{n}: an unquoted description may not contain ': ' or ' #' (invalid YAML, \
                     the skill will not load); wrap it in double quotes"
                );
            }
            // A skill another skill must be able to load alongside others stays short.
            if n.starts_with("yaks-coordinating") || n == "yaks-working" {
                assert!(text.lines().count() <= 200, "{n}: over the 200-line budget");
            }
            // Every skill-shaped reference (`yaks-` + 5 or more letters, so not a
            // 4-character yak id) must name a skill that exists.
            let bytes = text.as_bytes();
            let mut i = 0;
            while let Some(p) = text[i..].find("yaks-") {
                let start = i + p + 5;
                let mut end = start;
                while end < bytes.len() && (bytes[end].is_ascii_lowercase() || bytes[end] == b'-') {
                    end += 1;
                }
                let tok = text[start..end].trim_end_matches('-');
                if tok.len() >= 5 {
                    let full = format!("yaks-{tok}");
                    assert!(
                        names.contains(&full),
                        "{n}: refers to unknown skill `{full}`"
                    );
                }
                i = end.max(start);
            }
        }
        // The core's router must name every companion it routes to.
        let core = std::fs::read_to_string(dir.join("yaks-coordinating").join("SKILL.md")).unwrap();
        for n in names.iter().filter(|n| n.starts_with("yaks-coordinating-")) {
            assert!(
                core.contains(n.as_str()),
                "yaks-coordinating does not route to {n}"
            );
        }
    }

    /// The Delta landing helper is only trustworthy if its own scenarios keep passing
    /// (clean merge by SHA, refusing unexplained edits, dry run, cherry-pick fallback).
    /// It needs `git merge-tree --write-tree` (git 2.38+); skip on an older git.
    #[test]
    fn delta_land_script_selftest_passes() {
        let ver = std::process::Command::new("git")
            .arg("--version")
            .output()
            .unwrap();
        let ver = String::from_utf8_lossy(&ver.stdout).to_string();
        let nums: Vec<u32> = ver
            .split_whitespace()
            .nth(2)
            .unwrap_or("0")
            .split('.')
            .filter_map(|p| p.parse().ok())
            .collect();
        if (
            nums.first().copied().unwrap_or(0),
            nums.get(1).copied().unwrap_or(0),
        ) < (2, 38)
        {
            eprintln!("skipping: git {ver:?} lacks merge-tree --write-tree");
            return;
        }
        let script = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(".agents/skills/yaks-coordinating-delta/land.sh");
        let out = std::process::Command::new("sh")
            .arg(&script)
            .arg("--selftest")
            .output()
            .unwrap();
        let text = String::from_utf8_lossy(&out.stdout);
        assert!(out.status.success(), "land.sh --selftest failed:\n{text}");
        assert!(text.contains("selftest: all passed"), "{text}");
    }
}
