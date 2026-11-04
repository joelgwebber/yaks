//! `yaks tick` (yaks-5e8d): the one command a person (or, later, a persistent
//! agent) kicks off by hand. It reopens what is due, then says what needs
//! someone, so the one report can be trusted instead of reading the farm.
//!
//! The regrow step is [`Farm::regrow_due`], the same code the TUI's tick calls.
//! Everything here after it is read-only and pure over the loaded yaks, so the
//! digest is testable with a fixed clock ([`build`]).

use crate::model::{Status, Task};
use crate::{json, recur, sheds, store};
use chrono::{DateTime, Duration, Utc};
use serde_json::{Value, json};

/// How far ahead "due soon" looks unless `--within` says otherwise.
pub const DEFAULT_WITHIN: &str = "3d";

/// A claim nobody has touched: a `shaving` yak whose holder went quiet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stale {
    pub actor: String,
    pub id: String,
    pub title: String,
    /// The holder's newest write (an ISO timestamp).
    pub last: String,
}

/// What a tick found, after the regrow step.
#[derive(Debug, Clone)]
pub struct Digest {
    pub now: DateTime<Utc>,
    pub within: Duration,
    pub dry_run: bool,
    /// What the regrow step reopened (or, on a dry run, would) and held.
    pub regrow: recur::Report,
    /// Open yaks past their `due:`, most overdue first.
    pub overdue: Vec<Task>,
    /// Open yaks due within `within`, not yet overdue and not already in the
    /// inbox (they are listed there, with their due date), soonest first.
    pub due_soon: Vec<Task>,
    pub awaiting_human: Vec<Task>,
    pub awaiting_agent: Vec<Task>,
    pub stale: Vec<Stale>,
    /// How many findings `yaks check` has; `None` when it could not run.
    pub findings: Option<usize>,
}

impl Digest {
    /// Nothing for anyone to do or read.
    pub fn quiet(&self) -> bool {
        self.regrow.regrew.is_empty()
            && self.regrow.held.is_empty()
            && self.overdue.is_empty()
            && self.due_soon.is_empty()
            && self.awaiting_human.is_empty()
            && self.awaiting_agent.is_empty()
            && self.stale.is_empty()
            && self.findings.unwrap_or(0) == 0
    }
}

/// The digest of `tasks` (every yak) and `inbox` (the yaks with a `needs`), as
/// of `now`, given what the regrow step did. Pure.
pub fn build(
    tasks: &[Task],
    inbox: Vec<Task>,
    regrow: recur::Report,
    findings: Option<usize>,
    now: DateTime<Utc>,
    within: Duration,
    dry_run: bool,
) -> Digest {
    let by_due = |a: &Task, b: &Task| {
        let key = |t: &Task| recur::due(t).map(|d| d.at);
        key(a).cmp(&key(b)).then_with(|| a.id.cmp(&b.id))
    };
    let mut overdue: Vec<Task> = tasks
        .iter()
        .filter(|t| recur::is_overdue(t, now))
        .cloned()
        .collect();
    overdue.sort_by(by_due);
    let mut due_soon: Vec<Task> = tasks
        .iter()
        .filter(|t| t.status.is_open() && !recur::is_overdue(t, now) && t.needs.is_none())
        .filter(|t| recur::due(t).is_some_and(|d| d.at <= now + within))
        .cloned()
        .collect();
    due_soon.sort_by(by_due);
    let stale = sheds::build(tasks, now)
        .into_iter()
        .filter(|s| s.liveness == sheds::Liveness::Stale)
        .flat_map(|s| {
            let last = s.last_write.clone();
            let name = s.name.clone();
            s.working()
                .map(|t| Stale {
                    actor: name.clone(),
                    id: t.id.clone(),
                    title: t.title.clone(),
                    last: last.clone(),
                })
                .collect::<Vec<_>>()
        })
        .collect();
    Digest {
        now,
        within,
        dry_run,
        regrow,
        overdue,
        due_soon,
        awaiting_human: inbox
            .iter()
            .filter(|t| t.awaiting_human())
            .cloned()
            .collect(),
        awaiting_agent: inbox
            .iter()
            .filter(|t| t.awaiting_agent())
            .cloned()
            .collect(),
        stale,
        findings,
    }
}

/// The yaks a digest reads: open ones only, since a shorn or dead yak is
/// neither overdue nor claimed.
pub fn open_tasks(root: &std::path::Path) -> anyhow::Result<Vec<Task>> {
    store::load(root, &[Status::Hairy, Status::Shaving])
}

fn count(n: usize, one: &str, many: &str) -> Option<String> {
    match n {
        0 => None,
        1 => Some(format!("1 {one}")),
        n => Some(format!("{n} {many}")),
    }
}

/// The text report: a one-line summary, then a section per thing found.
pub fn render(d: &Digest, row: impl Fn(&Task) -> String) -> String {
    let mut out = String::new();
    let regrew = d.regrow.regrew.len();
    let parts: Vec<String> = [
        count(
            regrew,
            if d.dry_run { "would regrow" } else { "regrew" },
            if d.dry_run { "would regrow" } else { "regrew" },
        ),
        count(d.regrow.held.len(), "held", "held"),
        count(d.overdue.len(), "overdue", "overdue"),
        count(d.due_soon.len(), "due soon", "due soon"),
        count(d.awaiting_human.len(), "awaiting you", "awaiting you"),
        count(
            d.awaiting_agent.len(),
            "awaiting an agent",
            "awaiting an agent",
        ),
        count(d.stale.len(), "stale claim", "stale claims"),
        d.findings
            .and_then(|n| count(n, "check finding", "check findings")),
    ]
    .into_iter()
    .flatten()
    .collect();
    if d.quiet() {
        out.push_str("tick: nothing needs you.\n");
    } else {
        out.push_str(&format!("tick: {}.\n", parts.join(", ")));
    }
    if d.dry_run {
        out.push_str("(dry run \u{2014} nothing written)\n");
    }
    let section = |out: &mut String, title: &str, rows: Vec<String>| {
        if rows.is_empty() {
            return;
        }
        out.push_str(&format!("\n{title}\n"));
        for r in rows {
            out.push_str(&r);
            out.push('\n');
        }
    };
    let verb = if d.dry_run { "Would regrow" } else { "Regrew" };
    section(
        &mut out,
        &format!("{verb}:"),
        d.regrow
            .regrew
            .iter()
            .map(|g| {
                let waits = match &g.awaiting {
                    Some(a) if a.already => format!(", already awaiting {}", a.who),
                    Some(a) => format!(", awaiting {}", a.who),
                    None => String::new(),
                };
                format!(
                    "  {}  {} (for {}, due {}{waits})",
                    g.id,
                    g.title,
                    g.plan.occurrence.render(),
                    g.plan.due.render()
                )
            })
            .collect(),
    );
    section(
        &mut out,
        "Held (could not regrow):",
        d.regrow
            .held
            .iter()
            .map(|(id, why)| format!("  {id}: {why}"))
            .collect(),
    );
    section(&mut out, "Overdue:", d.overdue.iter().map(&row).collect());
    section(
        &mut out,
        &format!("Due within {}:", span(d.within)),
        d.due_soon.iter().map(&row).collect(),
    );
    section(
        &mut out,
        "Awaiting you (`yaks answer <id>`):",
        d.awaiting_human.iter().map(&row).collect(),
    );
    section(
        &mut out,
        "Awaiting an agent (`yaks pickup <id>`):",
        d.awaiting_agent.iter().map(&row).collect(),
    );
    section(
        &mut out,
        "Stale claims (shaving, holder quiet):",
        d.stale
            .iter()
            .map(|s| {
                format!(
                    "  {}  {} ({}, last wrote {})",
                    s.id,
                    s.title,
                    s.actor,
                    sheds::ago(&s.last, d.now)
                )
            })
            .collect(),
    );
    if let Some(n) = d.findings.filter(|n| *n > 0) {
        out.push_str(&format!("\n{n} finding(s): run `yaks check`.\n"));
    }
    out
}

/// `3d`, `36h`, `90m`: a window the way `--within` is written.
fn span(d: Duration) -> String {
    match d.num_seconds() {
        s if s % 604_800 == 0 && s > 0 => format!("{}w", s / 604_800),
        s if s % 86_400 == 0 && s > 0 => format!("{}d", s / 86_400),
        s if s % 3_600 == 0 && s > 0 => format!("{}h", s / 3_600),
        s => format!("{}m", s / 60),
    }
}

/// The stable shape a harness reads: the `regrow-due --json` keys, then one
/// array per section (task objects as `list --json` and `inbox --json` give).
pub fn to_json(d: &Digest) -> Value {
    let mut v = json::regrow_due_value(&d.regrow, d.dry_run);
    fn kept(ts: &[Task]) -> Vec<&Task> {
        ts.iter().collect()
    }
    if let Value::Object(m) = &mut v {
        m.insert("within".into(), json!(span(d.within)));
        m.insert("overdue".into(), json::tasks_array(&d.overdue));
        m.insert("due_soon".into(), json::tasks_array(&d.due_soon));
        m.insert(
            "awaiting_human".into(),
            json::inbox_array(&kept(&d.awaiting_human)),
        );
        m.insert(
            "awaiting_agent".into(),
            json::inbox_array(&kept(&d.awaiting_agent)),
        );
        m.insert(
            "stale".into(),
            Value::Array(
                d.stale
                    .iter()
                    .map(
                        |s| json!({"actor": s.actor, "id": s.id, "title": s.title, "last": s.last}),
                    )
                    .collect(),
            ),
        );
        m.insert("findings".into(), json!(d.findings));
        m.insert("quiet".into(), json!(d.quiet()));
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fields::Value as F;
    use chrono::TimeZone;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 4, 12, 0, 0).unwrap()
    }

    fn yak(id: &str, status: Status, due: Option<&str>) -> Task {
        let mut t = Task {
            id: id.into(),
            title: format!("{id} title"),
            kind: "task".into(),
            priority: 2,
            status,
            created: None,
            updated: None,
            parent: None,
            tags: vec![],
            depends_on: vec![],
            source: None,
            prs: vec![],
            needs: None,
            verify: None,
            extra: Default::default(),
            body: String::new(),
        };
        if let Some(d) = due {
            t.extra.set(recur::DUE, &F::Plain(d.into()));
        }
        t
    }

    fn digest(tasks: &[Task], inbox: Vec<Task>) -> Digest {
        build(
            tasks,
            inbox,
            recur::Report::default(),
            None,
            now(),
            Duration::days(3),
            false,
        )
    }

    #[test]
    fn overdue_and_due_soon_are_open_yaks_sorted_by_due() {
        let tasks = vec![
            yak("a-0001", Status::Hairy, Some("2026-11-02")),
            yak("a-0002", Status::Shaving, Some("2026-11-01")),
            // A date due today is not overdue until its day is over.
            yak("a-0003", Status::Hairy, Some("2026-11-04")),
            yak("a-0004", Status::Hairy, Some("2026-11-07")),
            yak("a-0005", Status::Hairy, Some("2026-11-08")),
            yak("a-0006", Status::Hairy, None),
            yak("a-0007", Status::Shorn, Some("2026-10-01")),
        ];
        let d = digest(&tasks, vec![]);
        let ids = |ts: &[Task]| ts.iter().map(|t| t.id.clone()).collect::<Vec<_>>();
        assert_eq!(ids(&d.overdue), ["a-0002", "a-0001"]);
        assert_eq!(ids(&d.due_soon), ["a-0003", "a-0004"]);
        assert!(!d.quiet());
    }

    #[test]
    fn a_yak_already_in_the_inbox_is_not_also_due_soon() {
        let mut asked = yak("a-0001", Status::Hairy, Some("2026-11-05"));
        asked.needs = Some("agent".into());
        let plain = yak("a-0002", Status::Hairy, Some("2026-11-05"));
        let d = digest(&[asked.clone(), plain], vec![asked]);
        assert_eq!(d.due_soon.len(), 1);
        assert_eq!(d.due_soon[0].id, "a-0002");
        assert_eq!(d.awaiting_agent[0].id, "a-0001");
    }

    #[test]
    fn the_inbox_splits_by_who_is_awaited() {
        let mut h = yak("a-0001", Status::Hairy, None);
        h.needs = Some("human".into());
        let mut g = yak("a-0002", Status::Hairy, None);
        g.needs = Some("agent".into());
        let d = digest(&[], vec![h, g]);
        assert_eq!(d.awaiting_human.len(), 1);
        assert_eq!(d.awaiting_agent.len(), 1);
    }

    #[test]
    fn a_claim_whose_holder_went_quiet_is_stale() {
        let mut t = yak("a-0001", Status::Shaving, None);
        t.body = store::append_note_meta(
            "",
            "2026-11-04T01:00:00Z",
            Some("worker-1"),
            &[("kind", "move"), ("from", "hairy"), ("to", "shaving")],
            &store::transition_text(Status::Hairy, Status::Shaving),
        );
        let d = digest(&[t.clone()], vec![]);
        assert_eq!(d.stale.len(), 1);
        assert_eq!(
            (d.stale[0].actor.as_str(), d.stale[0].id.as_str()),
            ("worker-1", "a-0001")
        );
        let text = render(&d, |t| t.id.clone());
        assert!(
            text.contains("Stale claims") && text.contains("worker-1"),
            "{text}"
        );

        // Written to within the active window: not stale.
        let fresh = build(
            &[t],
            vec![],
            recur::Report::default(),
            None,
            Utc.with_ymd_and_hms(2026, 11, 4, 2, 0, 0).unwrap(),
            Duration::days(3),
            false,
        );
        assert!(fresh.stale.is_empty());
    }

    #[test]
    fn a_quiet_farm_says_so_and_a_busy_one_leads_with_counts() {
        let quiet = digest(&[], vec![]);
        assert!(quiet.quiet());
        assert_eq!(
            render(&quiet, |t| t.id.clone()),
            "tick: nothing needs you.\n"
        );

        let tasks = vec![
            yak("a-0001", Status::Hairy, Some("2026-11-01")),
            yak("a-0002", Status::Hairy, Some("2026-11-05")),
        ];
        let text = render(&digest(&tasks, vec![]), |t| format!("  {}", t.id));
        assert!(text.starts_with("tick: 1 overdue, 1 due soon.\n"), "{text}");
        assert!(text.contains("\nOverdue:\n  a-0001\n"), "{text}");
        assert!(text.contains("\nDue within 3d:\n  a-0002\n"), "{text}");
    }

    #[test]
    fn findings_count_and_a_dry_run_are_said() {
        let mut d = digest(&[], vec![]);
        d.findings = Some(2);
        d.dry_run = true;
        let text = render(&d, |t| t.id.clone());
        assert!(
            text.starts_with("tick: 2 check findings.\n(dry run"),
            "{text}"
        );
        assert!(text.contains("2 finding(s): run `yaks check`"), "{text}");
        assert!(!d.quiet());
    }

    #[test]
    fn spans_read_back_the_way_they_are_written() {
        assert_eq!(span(Duration::days(3)), "3d");
        assert_eq!(span(Duration::weeks(2)), "2w");
        assert_eq!(span(Duration::hours(36)), "36h");
        assert_eq!(span(Duration::minutes(90)), "90m");
    }
}
