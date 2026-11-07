//! Resolving the actor a note or block should be attributed to. Shared by the
//! CLI (`update --as`, `ask`/`answer`) and the TUI (auto-attributed comments) so
//! both surfaces agree on one precedence: an explicit value wins, then
//! `$YAKS_ACTOR` (a harness/coordinator can pin it once per worker), then the
//! checkout's own `git config yaks.actor` (`yaks whoami --set`, yaks-5d1c: a
//! name that survives harnesses whose every command is a fresh shell), then
//! the identity a known harness exposes in its environment (see [`HARNESSES`]),
//! then the git `user.name` (free attribution for interactive human use).
//! `None` writes a bare, unattributed note. Attribution, never ownership.

use std::sync::atomic::{AtomicBool, Ordering};

/// The git config key holding this checkout's actor name.
pub const CONFIG_KEY: &str = "yaks.actor";

/// Where a resolved actor came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// `--as`.
    Explicit,
    /// `$YAKS_ACTOR`.
    Env,
    /// `git config yaks.actor` in this checkout.
    Config,
    /// Derived from a harness's environment (a Delta thread title or id).
    Harness,
    /// The git `user.name`.
    GitUser,
}

impl Source {
    pub fn as_str(self) -> &'static str {
        match self {
            Source::Explicit => "--as",
            Source::Env => "$YAKS_ACTOR",
            Source::Config => "git config yaks.actor",
            Source::Harness => "harness environment",
            Source::GitUser => "git user.name",
        }
    }
}

/// Set once by the CLI (never the TUI, which owns the terminal): a write whose
/// actor was derived from a harness then says so on stderr, once.
static WARN_DERIVED: AtomicBool = AtomicBool::new(false);
static WARNED: AtomicBool = AtomicBool::new(false);

/// Turn on the one-time stderr hint for harness-derived actors (CLI only).
pub fn warn_on_derived() {
    WARN_DERIVED.store(true, Ordering::Relaxed);
}

/// Longest harness-supplied name kept in a stamp; longer titles are cut.
const MAX_HARNESS_NAME: usize = 40;

/// Harness identities, in priority order: `(env var, stamp prefix)`. The first
/// var that is set and non-empty names the actor as `<prefix><value>`; the
/// prefix marks a derived, non-human actor. Supporting another harness is one
/// more line here.
const HARNESSES: &[(&str, &str)] = &[
    ("DELTA_THREAD_TITLE", "delta:"),
    ("DELTA_CURRENT_THREAD_ID", "delta:"),
];

/// Trim to a non-empty owned string, or `None`. An actor is written inside
/// `[ ]` on a note's marker line and read back by splitting at the LAST `" ["`
/// (`store::split_note_head`), so brackets and line breaks in a name would corrupt
/// the stamp (a thread title like `Fix [bug] in x` read back as `bug] in x`).
/// They are replaced here, for every source, so any actor round-trips.
fn clean(s: &str) -> Option<String> {
    let t: String = s
        .trim()
        .chars()
        .map(|c| match c {
            '[' => '(',
            ']' => ')',
            '\n' | '\r' => ' ',
            c => c,
        })
        .collect();
    let t = t.trim();
    (!t.is_empty()).then(|| t.to_string())
}

/// Collapse whitespace runs to single spaces and cut to `MAX_HARNESS_NAME`
/// characters (re-trimming a cut that lands on a space).
fn tidy_harness_name(s: &str) -> Option<String> {
    let collapsed = s.split_whitespace().collect::<Vec<_>>().join(" ");
    let cut: String = collapsed.chars().take(MAX_HARNESS_NAME).collect();
    clean(&cut)
}

/// The actor derived from the first harness variable that yields a name. A
/// title that starts with a worker name (`door-2: add doors`, the spawn-title
/// convention in `yaks-coordinating`) gives just that name (`delta:door-2`):
/// stable across renames of the rest of the title, and the worker's own name
/// rather than a sentence.
fn harness_actor(get: impl Fn(&str) -> Option<String>) -> Option<String> {
    HARNESSES.iter().find_map(|(var, prefix)| {
        let raw = get(var)?;
        let name = title_name(&raw).or_else(|| tidy_harness_name(&raw))?;
        Some(format!("{prefix}{name}"))
    })
}

/// The leading `<name>:` of a title, when it is one token of letters, digits,
/// `.`, `_` or `-` (at most 32 characters) followed by `: ` (or the end):
/// `door-2: add doors` -> `door-2`. `Trial 4A: x` has a space, so `None`.
fn title_name(title: &str) -> Option<String> {
    let (head, rest) = title.trim().split_once(':')?;
    let ok = !head.is_empty()
        && head.chars().count() <= 32
        && head
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
        && (rest.is_empty() || rest.starts_with(char::is_whitespace));
    ok.then(|| head.to_string())
}

/// Thread-id variables, in priority order: `$YAKS_THREAD` (any harness can set
/// it) wins over Delta's `$DELTA_CURRENT_THREAD_ID`.
const THREAD_VARS: &[&str] = &["YAKS_THREAD", "DELTA_CURRENT_THREAD_ID"];

/// The stable id of this conversation, if the harness exposes one (`$YAKS_THREAD`,
/// else Delta's thread id): recorded as `thread:` metadata on state moves and
/// session notes and shown by `yaks whoami`, so a name can be traced back to
/// the thread that wrote it (yaks-5d1c, yaks-8003). An empty value is unset.
pub fn harness_thread() -> Option<String> {
    thread_from(|var| std::env::var(var).ok())
}

/// [`harness_thread`] over an injectable environment.
fn thread_from(get: impl Fn(&str) -> Option<String>) -> Option<String> {
    THREAD_VARS.iter().find_map(|var| {
        get(var)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    })
}

/// Resolve the actor: `--as`/explicit → `$YAKS_ACTOR` → harness identity
/// (`delta:<thread title>`, else `delta:<thread id>`) → git `user.name`.
/// Best-effort — never fails, returning `None` when nothing is configured.
/// Without any of the first three, an agent running under the human's git
/// identity is stamped with the human's name.
pub fn resolve(explicit: Option<&str>) -> Option<String> {
    let (name, source) = resolve_detail(explicit)?;
    if source == Source::Harness
        && WARN_DERIVED.load(Ordering::Relaxed)
        && !WARNED.swap(true, Ordering::Relaxed)
    {
        eprintln!(
            "yaks: stamping this as `{name}`, derived from the harness; name yourself once with \
             `yaks whoami --set <name>` (or YAKS_ACTOR=<name> on every command)"
        );
    }
    Some(name)
}

/// [`resolve`] plus where the name came from (`yaks whoami`).
pub fn resolve_detail(explicit: Option<&str>) -> Option<(String, Source)> {
    resolve_full(
        explicit,
        std::env::var("YAKS_ACTOR").ok().as_deref(),
        || git_config(&["config", "--get", CONFIG_KEY]),
        |var| std::env::var(var).ok(),
        || git_config(&["config", "user.name"]),
    )
}

/// Precedence core, decoupled from the environment and git for testability.
fn resolve_full(
    explicit: Option<&str>,
    env: Option<&str>,
    config: impl FnOnce() -> Option<String>,
    harness_env: impl Fn(&str) -> Option<String>,
    git: impl FnOnce() -> Option<String>,
) -> Option<(String, Source)> {
    let tag = |s| move |n| (n, s);
    explicit
        .and_then(clean)
        .map(tag(Source::Explicit))
        .or_else(|| env.and_then(clean).map(tag(Source::Env)))
        .or_else(|| config().as_deref().and_then(clean).map(tag(Source::Config)))
        .or_else(|| harness_actor(harness_env).map(tag(Source::Harness)))
        .or_else(|| git().map(tag(Source::GitUser)))
}

#[cfg(test)]
fn resolve_with(
    explicit: Option<&str>,
    env: Option<&str>,
    harness_env: impl Fn(&str) -> Option<String>,
    git: impl FnOnce() -> Option<String>,
) -> Option<String> {
    resolve_full(explicit, env, || None, harness_env, git).map(|(n, _)| n)
}

/// A git config value read in the cwd's repository (worktree config
/// included), or `None` if git is absent or the key unset.
fn git_config(args: &[&str]) -> Option<String> {
    let out = std::process::Command::new("git").args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
        .as_deref()
        .and_then(clean)
}

/// Store `name` as this checkout's actor (`yaks whoami --set`). Written where
/// it applies to this checkout only: the repository's own config for a clone
/// or a primary checkout, the worktree's config (`extensions.worktreeConfig`)
/// for a linked `git worktree`, whose plain config is shared with the others.
/// Returns which file it went to.
pub fn set_config(cwd: &std::path::Path, name: &str) -> anyhow::Result<&'static str> {
    use anyhow::{Context, bail};
    let name = clean(name).context("an actor name cannot be empty")?;
    let git = |args: &[&str]| -> anyhow::Result<String> {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(cwd)
            .args(args)
            .output()
            .context("running git")?;
        if !out.status.success() {
            bail!(
                "git {}: {}",
                args.join(" "),
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    };
    let git_dir = git(&["rev-parse", "--absolute-git-dir"])
        .context("`yaks whoami --set` needs a git checkout")?;
    let common = git(&["rev-parse", "--path-format=absolute", "--git-common-dir"])?;
    let linked = crate::gitrepo::canonical(std::path::Path::new(&git_dir))
        != crate::gitrepo::canonical(std::path::Path::new(&common));
    if linked {
        git(&["config", "extensions.worktreeConfig", "true"])?;
        git(&["config", "--worktree", CONFIG_KEY, &name])?;
        Ok("this worktree's git config")
    } else {
        git(&["config", "--local", CONFIG_KEY, &name])?;
        Ok("this checkout's git config")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn git() -> Option<String> {
        Some("git-name".to_string())
    }

    /// An actor must survive being stamped on a note marker line and parsed back
    /// out unchanged, whatever a harness (or `--as`) hands us.
    #[test]
    fn every_actor_round_trips_through_the_note_stamp() {
        let env = [("DELTA_THREAD_TITLE", "Fix [bug] in the parser")];
        let names = [
            resolve_with(Some("x [y]"), None, |_| None, || None).unwrap(),
            resolve_with(None, None, vars(&env), || None).unwrap(),
            resolve_with(Some("a]b [c"), None, |_| None, || None).unwrap(),
            resolve_with(Some("two\nlines"), None, |_| None, || None).unwrap(),
        ];
        assert_eq!(names[1], "delta:Fix (bug) in the parser");
        for actor in names {
            let head = format!("2026-10-04T00:00:00Z [{actor}]");
            let (ts, parsed) = crate::store::split_note_head(&head);
            assert_eq!(ts, "2026-10-04T00:00:00Z");
            assert_eq!(parsed.as_deref(), Some(actor.as_str()), "{actor:?} did not round-trip");
        }
    }

    /// A fake environment from `(var, value)` pairs.
    fn vars<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |k| {
            pairs
                .iter()
                .find(|(n, _)| *n == k)
                .map(|(_, v)| v.to_string())
        }
    }

    #[test]
    fn checkout_config_sits_between_env_and_the_harness() {
        let h = vars(&[("DELTA_THREAD_TITLE", "My thread")]);
        let cfg = || Some("door-2".to_string());
        assert_eq!(
            resolve_full(None, Some("env"), cfg, &h, git),
            Some(("env".into(), Source::Env))
        );
        assert_eq!(
            resolve_full(None, None, cfg, &h, git),
            Some(("door-2".into(), Source::Config))
        );
        assert_eq!(
            resolve_full(None, None, || Some("  ".into()), &h, git),
            Some(("delta:My thread".into(), Source::Harness))
        );
        assert_eq!(
            resolve_full(None, None, || None, vars(&[]), git),
            Some(("git-name".into(), Source::GitUser))
        );
    }

    #[test]
    fn a_title_that_starts_with_a_worker_name_gives_just_the_name() {
        let derived = |t: &str| harness_actor(vars(&[("DELTA_THREAD_TITLE", t)]));
        assert_eq!(derived("door-2: add doors"), Some("delta:door-2".into()));
        assert_eq!(derived("tex_1.b: x"), Some("delta:tex_1.b".into()));
        assert_eq!(derived("herd:"), Some("delta:herd".into()));
        // Not one token, or no space after the colon: the whole title, as before.
        assert_eq!(derived("Trial 4A: add"), Some("delta:Trial 4A: add".into()));
        assert_eq!(derived("http://x"), Some("delta:http://x".into()));
        assert_eq!(derived("Web Deploy v0"), Some("delta:Web Deploy v0".into()));
    }

    #[test]
    fn explicit_beats_env_beats_harness_beats_git() {
        let h = vars(&[("DELTA_THREAD_TITLE", "My thread")]);
        // Explicit wins over everything.
        assert_eq!(
            resolve_with(Some("flag"), Some("env"), &h, git),
            Some("flag".into())
        );
        // Empty/whitespace explicit falls through to $YAKS_ACTOR.
        assert_eq!(
            resolve_with(Some("  "), Some("env"), &h, git),
            Some("env".into())
        );
        // $YAKS_ACTOR beats the harness identity.
        assert_eq!(resolve_with(None, Some("env"), &h, git), Some("env".into()));
        // Empty $YAKS_ACTOR falls through to the harness identity.
        assert_eq!(
            resolve_with(None, Some(" "), &h, git),
            Some("delta:My thread".into())
        );
        // Harness identity beats git.
        assert_eq!(
            resolve_with(None, None, &h, git),
            Some("delta:My thread".into())
        );
        // No harness -> git.
        assert_eq!(
            resolve_with(None, None, vars(&[]), git),
            Some("git-name".into())
        );
        // Nothing anywhere -> None (bare, unattributed note).
        assert_eq!(resolve_with(None, None, vars(&[]), || None), None);
    }

    #[test]
    fn delta_title_is_preferred_over_thread_id() {
        let h = vars(&[
            ("DELTA_THREAD_TITLE", "Trial 4A: add yaks path"),
            ("DELTA_CURRENT_THREAD_ID", "abc123"),
        ]);
        assert_eq!(
            harness_actor(h),
            Some("delta:Trial 4A: add yaks path".into())
        );
    }

    #[test]
    fn delta_thread_id_is_used_when_title_missing_or_blank() {
        let id = ("DELTA_CURRENT_THREAD_ID", "abc123");
        assert_eq!(harness_actor(vars(&[id])), Some("delta:abc123".into()));
        assert_eq!(
            harness_actor(vars(&[("DELTA_THREAD_TITLE", " \t\n "), id])),
            Some("delta:abc123".into())
        );
        assert_eq!(
            harness_actor(vars(&[("DELTA_THREAD_TITLE", ""), id])),
            Some("delta:abc123".into())
        );
    }

    #[test]
    fn yaks_thread_wins_over_the_delta_thread_id() {
        let delta = ("DELTA_CURRENT_THREAD_ID", "abc123");
        assert_eq!(thread_from(vars(&[delta])), Some("abc123".into()));
        assert_eq!(
            thread_from(vars(&[("YAKS_THREAD", " hearth-9 "), delta])),
            Some("hearth-9".into())
        );
        // Empty or blank `$YAKS_THREAD` is unset: the Delta id still applies.
        assert_eq!(
            thread_from(vars(&[("YAKS_THREAD", "  "), delta])),
            Some("abc123".into())
        );
        assert_eq!(thread_from(vars(&[("YAKS_THREAD", "x")])), Some("x".into()));
        assert_eq!(thread_from(vars(&[])), None);
    }

    #[test]
    fn no_delta_env_means_no_harness_actor() {
        assert_eq!(harness_actor(vars(&[])), None);
        assert_eq!(
            harness_actor(vars(&[
                ("DELTA_THREAD_TITLE", " "),
                ("DELTA_CURRENT_THREAD_ID", ""),
            ])),
            None
        );
    }

    #[test]
    fn harness_title_collapses_whitespace_and_truncates_to_40_chars() {
        assert_eq!(
            harness_actor(vars(&[("DELTA_THREAD_TITLE", "  a \n  b\t\tc  ")])),
            Some("delta:a b c".into())
        );
        let long = "x".repeat(60);
        assert_eq!(
            harness_actor(vars(&[("DELTA_THREAD_TITLE", &long)])),
            Some(format!("delta:{}", "x".repeat(40)))
        );
        // A cut landing on a space is re-trimmed; multibyte chars are not split.
        let spaced = format!("{} tail", "y".repeat(39));
        assert_eq!(
            harness_actor(vars(&[("DELTA_THREAD_TITLE", &spaced)])),
            Some(format!("delta:{}", "y".repeat(39)))
        );
        let wide = "é".repeat(45);
        assert_eq!(
            harness_actor(vars(&[("DELTA_THREAD_TITLE", &wide)])),
            Some(format!("delta:{}", "é".repeat(40)))
        );
    }
}
