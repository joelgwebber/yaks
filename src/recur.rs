//! Due dates and recurring yaks (yaks-2fc5, design yaks-9cc1).
//!
//! Three extra yak fields, written through [`crate::fields::Fields`] like any
//! other (the `--due` / `--every` flags are sugar for `--set`):
//!
//! - `due:` a date (`2026-11-15`) or an RFC 3339 datetime: when the yak should
//!   be done. A hairy or shaving yak past it is overdue.
//! - `every:` an ISO 8601 duration (`P1D`, `P2W`, `P1M`, `P1Y`, or a run of them
//!   in Y M W D order): makes the yak recur.
//! - `regrown:` a datetime yaks writes when `regrow-due` reopens the yak.
//!
//! Nothing here runs on a clock: [`plan`] is a pure function of a yak and a
//! `now`, and `Farm::regrow_due` applies it. A harness (cron, the TUI's tick)
//! calls `yaks regrow-due` when it likes.

use crate::fields::Value;
use crate::model::{Status, Task};
use chrono::{DateTime, Days, Months, NaiveDate, NaiveTime, Utc};

pub const DUE: &str = "due";
pub const EVERY: &str = "every";
pub const REGROWN: &str = "regrown";

/// The extra fields yaks itself gives meaning to. `doctor` never reports them
/// as missing from `schemas/yak.md`, and the detail view shows them in their
/// own rows instead of under "Other fields".
pub const KEYS: [&str; 3] = [DUE, EVERY, REGROWN];

/// An ISO 8601 duration as the schemas write them (`P1Y`, `P6M`, `P2W`,
/// `P30D`, or a run of them in that order), as `(months, days)`.
pub fn parse_duration(s: &str) -> Option<(u32, u32)> {
    let rest = s.strip_prefix('P')?;
    if rest.is_empty() {
        return None;
    }
    let (mut months, mut days) = (0u32, 0u32);
    let (mut last, mut num) = (0, String::new());
    for c in rest.chars() {
        if c.is_ascii_digit() {
            num.push(c);
            continue;
        }
        let rank = "YMWD".find(c)? + 1;
        if rank <= last || num.is_empty() {
            return None;
        }
        let n: u32 = num.parse().ok()?;
        match c {
            'Y' => months = months.checked_add(n.checked_mul(12)?)?,
            'M' => months = months.checked_add(n)?,
            'W' => days = days.checked_add(n.checked_mul(7)?)?,
            _ => days = days.checked_add(n)?,
        }
        (last, num) = (rank, String::new());
    }
    num.is_empty().then_some((months, days))
}

/// A moment as `due:` writes it: a whole day, or an instant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct When {
    /// The instant; midnight UTC when `date_only`.
    pub at: DateTime<Utc>,
    pub date_only: bool,
}

impl When {
    /// `2026-11-15` or an RFC 3339 datetime. Nothing else.
    pub fn parse(s: &str) -> Option<When> {
        let s = s.trim();
        if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            return Some(When {
                at: d.and_time(NaiveTime::MIN).and_utc(),
                date_only: true,
            });
        }
        DateTime::parse_from_rfc3339(s).ok().map(|t| When {
            at: t.with_timezone(&Utc),
            date_only: false,
        })
    }

    /// What a person may type for a deadline: what [`When::parse`] reads, or
    /// `+Nd` / `+Nw` (N days or weeks after `today`), which resolves to a whole
    /// day. Nothing else: no weekday words, no `-`, no months (a month has no
    /// fixed length; use a date).
    pub fn parse_input(s: &str, today: NaiveDate) -> Option<When> {
        let s = s.trim();
        let Some(rest) = s.strip_prefix('+') else {
            return When::parse(s);
        };
        let (digits, per) = match (rest.strip_suffix('d'), rest.strip_suffix('w')) {
            (Some(n), _) => (n, 1),
            (_, Some(n)) => (n, 7),
            _ => return None,
        };
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let days = digits.parse::<u64>().ok()?.checked_mul(per)?;
        Some(When {
            at: today
                .checked_add_days(Days::new(days))?
                .and_time(NaiveTime::MIN)
                .and_utc(),
            date_only: true,
        })
    }

    /// How it is written back: `2026-11-15`, or `2026-11-15T08:00:00Z`.
    pub fn render(self) -> String {
        let fmt = if self.date_only {
            "%Y-%m-%d"
        } else {
            "%Y-%m-%dT%H:%M:%SZ"
        };
        self.at.format(fmt).to_string()
    }

    /// The day alone (`2026-11-15`): what a list row shows.
    pub fn day(self) -> String {
        self.at.format("%Y-%m-%d").to_string()
    }

    /// The instant after which the yak is late: the end of the day for a
    /// date, the instant itself for a datetime.
    pub fn deadline(self) -> DateTime<Utc> {
        if self.date_only {
            self.at.checked_add_days(Days::new(1)).unwrap_or(self.at)
        } else {
            self.at
        }
    }
}

/// A yak's `due:`, when it has one that reads.
pub fn due(t: &Task) -> Option<When> {
    When::parse(t.extra.get(DUE)?.as_str()?)
}

/// Whether a hairy or shaving yak is past its `due:` at `now`. A shorn or dead
/// yak is never overdue: its work is done (or dropped).
pub fn is_overdue(t: &Task, now: DateTime<Utc>) -> bool {
    t.status.is_open() && due(t).is_some_and(|d| now >= d.deadline())
}

/// `list --due`: keep the yaks that have a `due:` (before `before`, when
/// given), sorted by it, then by id.
pub fn due_rows(rows: Vec<Task>, before: Option<DateTime<Utc>>) -> Vec<Task> {
    let mut keyed: Vec<(DateTime<Utc>, Task)> = rows
        .into_iter()
        .filter_map(|t| Some((due(&t)?.at, t)))
        .filter(|(at, _)| before.is_none_or(|b| *at < b))
        .collect();
    keyed.sort_by(|(a, x), (b, y)| a.cmp(b).then_with(|| x.id.cmp(&y.id)));
    keyed.into_iter().map(|(_, t)| t).collect()
}

/// What a CLI row says about the due date: ` due 2026-11-15`, with ` overdue`
/// after it when the yak is late; empty without a readable `due:`.
pub fn due_mark(t: &Task, now: DateTime<Utc>) -> String {
    let Some(d) = due(t) else {
        return String::new();
    };
    let late = if is_overdue(t, now) { " overdue" } else { "" };
    format!(" due {}{late}", d.day())
}

/// The problem with `value` as the field `key`, when `key` is one of ours and
/// the value does not read; `None` otherwise (including for a cleared field).
/// What `--due`, `snooze --to` and the TUI accept, said once.
pub const DUE_INPUT_HELP: &str =
    "a date (2026-11-15), an RFC 3339 datetime (2026-11-15T09:00:00Z), or +2d / +1w from today";

/// A typed deadline as `due:` stores it, or why it is refused. The one door
/// for `--due`, `snooze --to` and the TUI.
pub fn resolve_due(text: &str, today: NaiveDate) -> Result<String, String> {
    When::parse_input(text, today)
        .map(When::render)
        .ok_or_else(|| format!("{text:?} is not {DUE_INPUT_HELP}"))
}

/// Today on the calendar a person reads, which is what `+2d` counts from.
pub fn today() -> NaiveDate {
    chrono::Local::now().date_naive()
}

pub fn field_problem(key: &str, value: &Value) -> Option<String> {
    let text = match value {
        Value::Null => return None,
        v => v.as_str().unwrap_or_default(),
    };
    match key {
        DUE if When::parse(text).is_none() => Some(format!("{text:?} is not {DUE_INPUT_HELP}")),
        EVERY if Every::parse(text).is_none() => Some(format!(
            "{text:?} is not an ISO 8601 duration (P1D, P2W, P1M, P1Y; a run of them in Y M W D order, not zero)"
        )),
        _ => None,
    }
}

/// A recurrence period: calendar months plus days (a week is seven days).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Every {
    months: u32,
    days: u32,
}

impl Every {
    /// An ISO 8601 duration that is not zero.
    pub fn parse(s: &str) -> Option<Every> {
        let (months, days) = parse_duration(s.trim())?;
        (months > 0 || days > 0).then_some(Every { months, days })
    }

    /// `start` plus `k` periods, counted from `start` (not step by step), so a
    /// month-end start keeps its day: Jan 31 + 1 month is Feb 28, + 2 is Mar 31.
    /// `None` when the date overflows.
    pub fn after(self, start: DateTime<Utc>, k: u32) -> Option<DateTime<Utc>> {
        let months = self.months.checked_mul(k)?;
        let days = u64::from(self.days) * u64::from(k);
        start
            .checked_add_months(Months::new(months))?
            .checked_add_days(Days::new(days))
    }

    /// The largest `k >= first` with `start + k * every` not after `now`, and
    /// that instant; `None` when even `first` is still in the future. Gallops
    /// then bisects (`after` only grows with k), so a yak shorn years ago costs
    /// a handful of steps, not one per missed occurrence.
    pub fn latest(
        self,
        start: DateTime<Utc>,
        first: u32,
        now: DateTime<Utc>,
    ) -> Option<(DateTime<Utc>, u32)> {
        let reached = |k: u32| self.after(start, k).filter(|t| *t <= now);
        reached(first)?;
        let (mut lo, mut step) = (first, 1u32);
        let mut hi = loop {
            match lo.checked_add(step) {
                Some(c) if reached(c).is_some() => {
                    lo = c;
                    step = step.saturating_mul(2);
                }
                Some(c) => break c,
                None => break u32::MAX,
            }
        };
        while hi - lo > 1 {
            let mid = lo + (hi - lo) / 2;
            if reached(mid).is_some() {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        Some((reached(lo)?, lo))
    }
}

/// What `regrow-due` does for one shorn yak that has `every:`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// The occurrence the yak regrows for.
    pub occurrence: When,
    /// Earlier occurrences that arrived while the yak stayed shorn: one regrow
    /// covers them all.
    pub skipped: u32,
    /// The next deadline: the occurrence plus one period.
    pub due: When,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// Not a shorn yak with `every:`: nothing to do.
    NotRecurring,
    /// Recurring, but its next occurrence has not arrived.
    Waiting,
    /// `every:`, `due:` or `regrown:` does not read.
    Invalid(String),
    Regrow(Plan),
}

/// When the yak was last shorn: its latest move into `shorn`, else `updated`.
fn shorn_at(t: &Task) -> Option<When> {
    let moved = crate::store::parse_notes(&t.body)
        .into_iter()
        .rfind(|n| {
            n.meta.iter().any(|(k, v)| k == "kind" && v == "move")
                && n.meta
                    .iter()
                    .any(|(k, v)| k == "to" && v == Status::Shorn.as_str())
        })
        .map(|n| n.ts);
    When::parse(&moved.or_else(|| t.updated.clone())?)
}

enum Date {
    Absent,
    Reads(When),
    Bad,
}

fn date_field(t: &Task, key: &str) -> Date {
    match t.extra.get(key) {
        None | Some(Value::Null) => Date::Absent,
        Some(v) => v
            .as_str()
            .and_then(When::parse)
            .map_or(Date::Bad, Date::Reads),
    }
}

/// The schedule, as one rule. The occurrences are `anchor + k * every`; the
/// yak regrows for the latest one that is not after `now`.
///
/// - A yak that has been regrown carries `due:` = its last occurrence plus one
///   period, which is exactly the next occurrence: the anchor is that `due:`
///   and k starts at 0. (Counting from `regrown:`, which is the moment yaks
///   ran, would let every regrow's delay push the schedule later.)
/// - A yak never regrown counts from its `due:` (else the time it was shorn),
///   k starting at 1: the due date is the deadline of the round just shorn.
/// - `regrown:` with no `due:` (someone cleared it) is the anchor, k from 1.
pub fn plan(t: &Task, now: DateTime<Utc>) -> Verdict {
    let Some(every) = t.extra.get(EVERY) else {
        return Verdict::NotRecurring;
    };
    if t.status != Status::Shorn {
        return Verdict::NotRecurring;
    }
    let text = every.as_str().unwrap_or_default();
    let Some(every) = Every::parse(text) else {
        return Verdict::Invalid(format!("every: {text:?} is not an ISO 8601 duration"));
    };
    let (start, first) = match (date_field(t, REGROWN), date_field(t, DUE)) {
        (Date::Bad, _) => return Verdict::Invalid("regrown: does not read as a date".into()),
        (_, Date::Bad) => return Verdict::Invalid("due: does not read as a date".into()),
        (Date::Reads(_), Date::Reads(due)) => (due, 0),
        (Date::Reads(regrown), Date::Absent) => (regrown, 1),
        (Date::Absent, Date::Reads(due)) => (due, 1),
        (Date::Absent, Date::Absent) => match shorn_at(t) {
            Some(shorn) => (shorn, 1),
            None => {
                return Verdict::Invalid("no due:, regrown: or shear time to count from".into());
            }
        },
    };
    let Some((at, k)) = every.latest(start.at, first, now) else {
        return Verdict::Waiting;
    };
    let Some(next) = every.after(at, 1) else {
        return Verdict::Invalid("the next deadline is out of range".into());
    };
    let when = |at| When {
        at,
        date_only: start.date_only,
    };
    Verdict::Regrow(Plan {
        occurrence: when(at),
        skipped: k - first,
        due: when(next),
    })
}

/// The text of the `kind: regrow` note.
pub fn regrow_note(p: &Plan) -> String {
    format!(
        "regrew for the {} occurrence ({} skipped)",
        p.occurrence.render(),
        p.skipped
    )
}

/// One yak `regrow-due` reopened (or, on a dry run, would).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Regrew {
    pub id: String,
    pub title: String,
    pub plan: Plan,
    /// Shorn ancestors reopened first, outermost first.
    pub reopened: Vec<String>,
}

/// What `regrow-due` did, and what it left alone and why.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Report {
    pub regrew: Vec<Regrew>,
    /// A recurring yak that is due but could not regrow: `(id, reason)`.
    pub held: Vec<(String, String)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(y: i32, m: u32, d: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, 0, 0, 0).unwrap()
    }

    fn shorn(extra: &[(&str, &str)]) -> Task {
        let mut t = Task {
            id: "t-0001".into(),
            title: "Salt".into(),
            kind: "task".into(),
            priority: 3,
            status: Status::Shorn,
            created: None,
            updated: Some("2026-10-01T09:00:00Z".into()),
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
        for (k, v) in extra {
            t.extra.set(k, &Value::Plain((*v).into()));
        }
        t
    }

    fn regrow(t: &Task, now: DateTime<Utc>) -> Plan {
        match plan(t, now) {
            Verdict::Regrow(p) => p,
            other => panic!("expected a regrow, got {other:?}"),
        }
    }

    #[test]
    fn durations_read_iso_8601_in_order() {
        assert_eq!(parse_duration("P1Y"), Some((12, 0)));
        assert_eq!(parse_duration("P6M"), Some((6, 0)));
        assert_eq!(parse_duration("P2W"), Some((0, 14)));
        assert_eq!(parse_duration("P30D"), Some((0, 30)));
        assert_eq!(parse_duration("P1Y2M1W3D"), Some((14, 10)));
        for bad in [
            "",
            "P",
            "1Y",
            "P1D1Y",
            "PY",
            "P1",
            "P1H",
            "p1y",
            "P1Y1Y",
            "monday",
            "2026-12-01",
            "P99999999999Y",
        ] {
            assert_eq!(parse_duration(bad), None, "{bad}");
        }
    }

    #[test]
    fn every_refuses_zero_and_words() {
        assert!(Every::parse("P0D").is_none());
        assert!(Every::parse("P1W").is_some());
        assert!(Every::parse("monday").is_none());
    }

    #[test]
    fn when_reads_dates_and_datetimes_only() {
        let d = When::parse("2026-11-15").unwrap();
        assert!(d.date_only);
        assert_eq!(d.render(), "2026-11-15");
        let t = When::parse("2026-11-15T09:30:00+02:00").unwrap();
        assert!(!t.date_only);
        assert_eq!(t.render(), "2026-11-15T07:30:00Z");
        for bad in ["", "soon", "2026-13-01", "11/15/2026", "2026-11-15 09:00"] {
            assert!(When::parse(bad).is_none(), "{bad}");
        }
    }

    #[test]
    fn input_reads_relative_days_and_weeks_from_today() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 4).unwrap();
        let day = |s: &str| When::parse_input(s, today).map(|w| (w.render(), w.date_only));
        assert_eq!(day("+0d"), Some(("2026-11-04".into(), true)));
        assert_eq!(day("+2d"), Some(("2026-11-06".into(), true)));
        assert_eq!(day(" +1w "), Some(("2026-11-11".into(), true)));
        assert_eq!(day("+30d"), Some(("2026-12-04".into(), true)));
        // Absolute forms pass through unchanged.
        assert_eq!(day("2026-11-15"), Some(("2026-11-15".into(), true)));
        assert_eq!(
            day("2026-11-15T09:00:00Z"),
            Some(("2026-11-15T09:00:00Z".into(), false))
        );
        for bad in [
            "+", "+d", "+2", "+2m", "-2d", "+2d3", "+ 2d", "+1.5w", "+-1d", "monday", "",
        ] {
            assert!(day(bad).is_none(), "{bad}");
        }
    }

    #[test]
    fn a_date_is_overdue_once_its_day_has_passed() {
        let mut t = shorn(&[("due", "2026-11-15")]);
        t.status = Status::Hairy;
        assert!(!is_overdue(&t, at(2026, 11, 15)));
        assert!(!is_overdue(
            &t,
            at(2026, 11, 15) + chrono::Duration::hours(23)
        ));
        assert!(is_overdue(&t, at(2026, 11, 16)));
        // A datetime is late the moment it passes.
        let t = Task {
            extra: shorn(&[("due", "2026-11-15T09:00:00Z")]).extra,
            status: Status::Shaving,
            ..t
        };
        assert!(!is_overdue(
            &t,
            at(2026, 11, 15) + chrono::Duration::hours(8)
        ));
        assert!(is_overdue(
            &t,
            at(2026, 11, 15) + chrono::Duration::hours(10)
        ));
    }

    #[test]
    fn finished_yaks_are_never_overdue() {
        for status in [Status::Shorn, Status::Dead] {
            let mut t = shorn(&[("due", "2020-01-01")]);
            t.status = status;
            assert!(!is_overdue(&t, at(2026, 1, 1)), "{status:?}");
        }
    }

    #[test]
    fn rows_say_due_and_overdue() {
        let mut t = shorn(&[("due", "2026-11-15")]);
        t.status = Status::Hairy;
        assert_eq!(due_mark(&t, at(2026, 11, 15)), " due 2026-11-15");
        assert_eq!(due_mark(&t, at(2026, 11, 16)), " due 2026-11-15 overdue");
        assert_eq!(due_mark(&shorn(&[]), at(2026, 11, 16)), "");
        // A shorn yak keeps its date but is not late.
        t.status = Status::Shorn;
        assert_eq!(due_mark(&t, at(2027, 1, 1)), " due 2026-11-15");
    }

    #[test]
    fn due_rows_keep_dated_yaks_in_date_order() {
        let mk = |id: &str, due: Option<&str>| {
            let mut t = shorn(&[]);
            t.id = id.into();
            if let Some(d) = due {
                t.extra.set("due", &Value::Plain(d.into()));
            }
            t
        };
        let rows = vec![
            mk("b", Some("2026-11-20")),
            mk("none", None),
            mk("c", Some("2026-11-01")),
            mk("a", Some("2026-11-20")),
        ];
        let ids = |v: Vec<Task>| v.into_iter().map(|t| t.id).collect::<Vec<_>>();
        assert_eq!(ids(due_rows(rows.clone(), None)), ["c", "a", "b"]);
        // `before` is exclusive.
        assert_eq!(ids(due_rows(rows.clone(), Some(at(2026, 11, 20)))), ["c"]);
        assert_eq!(ids(due_rows(rows, Some(at(2026, 11, 21)))), ["c", "a", "b"]);
    }

    #[test]
    fn regrows_for_the_latest_occurrence_and_counts_the_skipped() {
        // Due Nov 1, weekly, never regrown: Nov 8 and Nov 15 have both
        // arrived, so one regrow covers both.
        let t = shorn(&[("every", "P1W"), ("due", "2026-11-01")]);
        let p = regrow(&t, at(2026, 11, 15));
        assert_eq!(p.occurrence.render(), "2026-11-15");
        assert_eq!(p.skipped, 1);
        assert_eq!(p.due.render(), "2026-11-22");
        assert_eq!(
            regrow_note(&p),
            "regrew for the 2026-11-15 occurrence (1 skipped)"
        );
    }

    #[test]
    fn the_first_occurrence_skips_nothing_and_a_future_one_waits() {
        let t = shorn(&[("every", "P1W"), ("due", "2026-11-01")]);
        let p = regrow(&t, at(2026, 11, 9));
        assert_eq!((p.occurrence.render(), p.skipped), ("2026-11-08".into(), 0));
        assert_eq!(plan(&t, at(2026, 11, 7)), Verdict::Waiting);
        // The occurrence itself counts: it is not in the future.
        assert!(matches!(plan(&t, at(2026, 11, 8)), Verdict::Regrow(_)));
    }

    #[test]
    fn a_long_gap_is_found_without_walking_it() {
        let t = shorn(&[("every", "P1D"), ("due", "2000-01-01")]);
        let p = regrow(&t, at(2026, 11, 15));
        assert_eq!(p.occurrence.render(), "2026-11-15");
        assert_eq!(p.skipped, 9_815 - 1);
    }

    #[test]
    fn months_count_from_the_start_so_month_end_keeps_its_day() {
        let t = shorn(&[("every", "P1M"), ("due", "2026-01-31")]);
        // Feb 28 (Jan 31 + 1 month, clamped), then Mar 31, not Mar 28.
        assert_eq!(regrow(&t, at(2026, 3, 1)).occurrence.render(), "2026-02-28");
        let p = regrow(&t, at(2026, 4, 1));
        assert_eq!(p.occurrence.render(), "2026-03-31");
        assert_eq!(p.skipped, 1);
        // The next deadline is the occurrence plus one period.
        assert_eq!(p.due.render(), "2026-04-30");
    }

    #[test]
    fn a_regrown_yak_regrows_at_its_due_without_drifting() {
        // Regrown late (Nov 16 14:03) for the Nov 15 occurrence: due is Nov 22.
        // The next regrow is Nov 22 itself, not a week after the 14:03.
        let t = shorn(&[
            ("every", "P1W"),
            ("due", "2026-11-22"),
            ("regrown", "2026-11-16T14:03:00Z"),
        ]);
        assert_eq!(plan(&t, at(2026, 11, 21)), Verdict::Waiting);
        let p = regrow(&t, at(2026, 11, 22));
        assert_eq!(p.occurrence.render(), "2026-11-22");
        assert_eq!(p.skipped, 0);
        assert_eq!(p.due.render(), "2026-11-29");
        // Missed rounds are counted from that due too.
        assert_eq!(regrow(&t, at(2026, 12, 7)).skipped, 2);
    }

    #[test]
    fn regrown_alone_is_an_anchor_counted_from_one() {
        let t = shorn(&[("every", "P1W"), ("regrown", "2026-11-10T08:00:00Z")]);
        assert_eq!(plan(&t, at(2026, 11, 17)), Verdict::Waiting);
        let p = regrow(&t, at(2026, 11, 18));
        assert_eq!(p.occurrence.render(), "2026-11-17T08:00:00Z");
        assert_eq!(p.due.render(), "2026-11-24T08:00:00Z");
    }

    #[test]
    fn with_no_dates_it_counts_from_the_shear() {
        let mut t = shorn(&[("every", "P1W")]);
        t.body = "---\n\u{25b8} 2026-10-20T10:00:00Z [a]\nkind: move\nfrom: shaving\nto: shorn\n---\n\nmoved: shaving -> shorn".into();
        let p = regrow(&t, at(2026, 10, 28));
        assert_eq!(p.occurrence.render(), "2026-10-27T10:00:00Z");
        // Without a move note, `updated` stands in.
        let t = shorn(&[("every", "P1D")]);
        let p = regrow(&t, at(2026, 10, 3));
        assert_eq!(p.occurrence.render(), "2026-10-02T09:00:00Z");
    }

    #[test]
    fn only_shorn_yaks_with_a_reading_every_are_candidates() {
        let mut t = shorn(&[("every", "P1W"), ("due", "2026-01-01")]);
        assert_eq!(
            plan(&shorn(&[("due", "2026-01-01")]), at(2026, 11, 1)),
            Verdict::NotRecurring
        );
        for status in [Status::Hairy, Status::Shaving, Status::Dead] {
            t.status = status;
            assert_eq!(
                plan(&t, at(2026, 11, 1)),
                Verdict::NotRecurring,
                "{status:?}"
            );
        }
        for bad in ["monday", "P0D", "1W"] {
            let t = shorn(&[("every", bad), ("due", "2026-01-01")]);
            assert!(
                matches!(plan(&t, at(2026, 11, 1)), Verdict::Invalid(_)),
                "{bad}"
            );
        }
        let t = shorn(&[("every", "P1W"), ("due", "soon")]);
        assert!(matches!(plan(&t, at(2026, 11, 1)), Verdict::Invalid(_)));
    }

    #[test]
    fn fields_are_checked_as_they_are_set() {
        let v = |s: &str| Value::Plain(s.into());
        assert!(field_problem("due", &v("2026-11-15")).is_none());
        assert!(
            field_problem("due", &v("next week"))
                .unwrap()
                .contains("not a date")
        );
        assert!(field_problem("every", &v("P1W")).is_none());
        assert!(
            field_problem("every", &v("weekly"))
                .unwrap()
                .contains("duration")
        );
        // Clearing is always fine, and other keys are not ours.
        assert!(field_problem("due", &Value::Null).is_none());
        assert!(field_problem("owner", &v("anything")).is_none());
    }
}
