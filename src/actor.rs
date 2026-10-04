//! Resolving the actor a note or block should be attributed to. Shared by the
//! CLI (`update --as`, `ask`/`answer`) and the TUI (auto-attributed comments) so
//! both surfaces agree on one precedence: an explicit value wins, then
//! `$YAKS_ACTOR` (a harness/coordinator can pin it once per worker), then the
//! identity a known harness exposes in its environment (see [`HARNESSES`]), then
//! the git `user.name` (free attribution for interactive human use). `None`
//! writes a bare, unattributed note. Attribution, never ownership.

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

/// The actor derived from the first harness variable that yields a name.
fn harness_actor(get: impl Fn(&str) -> Option<String>) -> Option<String> {
    HARNESSES.iter().find_map(|(var, prefix)| {
        let name = tidy_harness_name(&get(var)?)?;
        Some(format!("{prefix}{name}"))
    })
}

/// Resolve the actor: `--as`/explicit → `$YAKS_ACTOR` → harness identity
/// (`delta:<thread title>`, else `delta:<thread id>`) → git `user.name`.
/// Best-effort — never fails, returning `None` when nothing is configured.
/// Without any of the first three, an agent running under the human's git
/// identity is stamped with the human's name.
pub fn resolve(explicit: Option<&str>) -> Option<String> {
    resolve_with(
        explicit,
        std::env::var("YAKS_ACTOR").ok().as_deref(),
        |var| std::env::var(var).ok(),
        git_user_name,
    )
}

/// Precedence core, decoupled from the environment and git for testability.
fn resolve_with(
    explicit: Option<&str>,
    env: Option<&str>,
    harness_env: impl Fn(&str) -> Option<String>,
    git: impl FnOnce() -> Option<String>,
) -> Option<String> {
    explicit
        .and_then(clean)
        .or_else(|| env.and_then(clean))
        .or_else(|| harness_actor(harness_env))
        .or_else(git)
}

/// The git `user.name`, or `None` if git is absent/unconfigured.
fn git_user_name() -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["config", "user.name"])
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
        .as_deref()
        .and_then(clean)
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
