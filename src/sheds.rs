//! `yaks sheds`: who is working on what, read from the farm alone.
//!
//! A **shed** is a worker in the farm, one row per ACTOR: the `[actor]` stamp
//! on notes and moves (`▸ <ts> [actor]`). Nothing here looks at git, Delta or
//! the machine, so the same farm reads the same on every machine once synced.
//!
//! - **Working on**: yaks `shaving` now whose latest claim is this actor's.
//!   The claim is the latest move into `shaving`; its claimant is the actor
//!   named by that note's `for:` metadata, else the actor of the most recent
//!   note since (the move included). The most recent writer, not the mover: a
//!   coordinator moves a yak to `shaving` for a worker, and the worker writes
//!   after. A yak is under exactly one actor's "working on".
//! - **Asking**: the actor's yaks with `needs: human`.
//! - **Recent**: every other yak the actor wrote, newest first, with its
//!   status now.
//! - **Liveness**: `active` (wrote within [`ACTIVE_WITHIN`]), `stale` (holds a
//!   claim but has not written for longer), else `idle`.
//!
//! Notes with no actor belong to no row.

use crate::model::{Status, Task};
use crate::status::{self, SyncState};
use crate::store;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

const EVERY: [Status; 4] = [Status::Hairy, Status::Shaving, Status::Shorn, Status::Dead];

/// How recent an actor's last write must be for it to read `active`.
pub const ACTIVE_WITHIN: Duration = Duration::from_secs(2 * 60 * 60);

/// How many `recent` yaks the text table lists per shed.
const RECENT_SHOWN: usize = 5;

/// Whether anyone appears to be working as this actor right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Liveness {
    /// It wrote to the farm within [`ACTIVE_WITHIN`].
    Active,
    /// It holds a claim (a `shaving` yak) but has not written for longer: a
    /// claim nobody has touched in a while.
    Stale,
    /// Neither: no recent write and no claim (the common case).
    Idle,
}

impl Liveness {
    /// The JSON `liveness` value.
    pub fn as_str(self) -> &'static str {
        match self {
            Liveness::Active => "active",
            Liveness::Stale => "stale",
            Liveness::Idle => "idle",
        }
    }
}

/// One yak an actor wrote, as the farm says now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Touched {
    pub id: String,
    pub title: String,
    pub status: Status,
    /// Its `needs:` now.
    pub needs: Option<String>,
    /// The actor's newest note on it (for a claim handed over with `for:`
    /// before the actor wrote anything, the move that handed it over).
    pub last: String,
    /// The actor holds the claim on it ([`Kind::Working`]).
    pub working: bool,
}

impl Touched {
    /// Waiting on a human.
    pub fn asking(&self) -> bool {
        self.needs.as_deref() == Some("human")
    }
}

/// Which part of a shed a yak is listed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// `shaving` and claimed by this actor.
    Working,
    /// Waiting on a human, not claimed by this actor.
    Asking,
    /// Anything else the actor wrote.
    Recent,
}

impl Kind {
    /// The label text mode and the TUI print.
    pub fn label(self) -> &'static str {
        match self {
            Kind::Working => "shaving",
            Kind::Asking => "asking",
            Kind::Recent => "recent",
        }
    }
}

/// One worker in the farm: an actor and what it wrote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shed {
    /// The actor stamp.
    pub name: String,
    /// Every yak it wrote, newest write first.
    pub touched: Vec<Touched>,
    /// Its newest write.
    pub last_write: String,
    pub liveness: Liveness,
}

impl Shed {
    pub fn working(&self) -> impl Iterator<Item = &Touched> {
        self.touched.iter().filter(|t| t.working)
    }

    /// Its yaks waiting on a human (claimed or not).
    pub fn asks(&self) -> impl Iterator<Item = &Touched> {
        self.touched.iter().filter(|t| t.asking())
    }

    /// Nothing asks for attention: idle, with no claim and no open ask. The
    /// text table counts these instead of listing them.
    pub fn quiet(&self) -> bool {
        self.liveness == Liveness::Idle && self.asks().next().is_none()
    }

    /// Display order: claimed yaks, then unclaimed asks, then the rest, each
    /// newest first.
    pub fn rows(&self) -> Vec<(Kind, &Touched)> {
        let kind = |t: &Touched| {
            if t.working {
                Kind::Working
            } else if t.asking() {
                Kind::Asking
            } else {
                Kind::Recent
            }
        };
        let mut rows: Vec<(Kind, &Touched)> = self.touched.iter().map(|t| (kind(t), t)).collect();
        rows.sort_by_key(|(k, _)| match k {
            Kind::Working => 0,
            Kind::Asking => 1,
            Kind::Recent => 2,
        });
        rows
    }

    /// Attention order: an open ask, active, stale, idle.
    fn rank(&self) -> u8 {
        if self.asks().next().is_some() {
            0
        } else {
            match self.liveness {
                Liveness::Active => 1,
                Liveness::Stale => 2,
                Liveness::Idle => 3,
            }
        }
    }
}

/// Everything `yaks sheds` shows.
#[derive(Debug, Clone)]
pub struct Report {
    /// When it was read (what "ago" is measured from).
    pub now: DateTime<Utc>,
    /// The farm branch against its upstream as of the last fetch; `None`
    /// when the farm has no upstream to compare with.
    pub farm: Option<SyncState>,
    /// In display order: attention first (an open ask, active, stale, idle),
    /// then the most recent writer, then by name.
    pub sheds: Vec<Shed>,
}

/// Read the farm at `root`. Read-only; never fetches.
pub fn collect(root: &Path) -> Result<Report> {
    let tasks = store::load(root, &EVERY)?;
    let now = Utc::now();
    Ok(Report {
        now,
        farm: status::sync_state(root).filter(|s| s.upstream.is_some()),
        sheds: build(&tasks, now),
    })
}

/// The status a transition note (`moved: a -> b`) moves a yak into.
fn moved_to(text: &str) -> Option<Status> {
    if !store::is_transition_text(text) {
        return None;
    }
    Status::parse(text.strip_prefix("moved: ")?.split_once(" -> ")?.1)
}

/// Who holds the claim on a yak that is `shaving` now, and the timestamp that
/// stamps it for them: see the module docs.
fn claimant(notes: &[store::NoteEntry]) -> Option<(String, String)> {
    let start = notes
        .iter()
        .rposition(|n| moved_to(&n.text) == Some(Status::Shaving));
    if let Some(i) = start {
        if let Some(who) = notes[i].get("for").map(str::trim).filter(|w| !w.is_empty()) {
            return Some((who.to_string(), notes[i].ts.clone()));
        }
    }
    notes[start.unwrap_or(0)..]
        .iter()
        .rev()
        .find_map(|n| Some((n.actor.clone()?, n.ts.clone())))
}

/// The sheds of `tasks`, in display order, as of `now`. Pure.
pub fn build(tasks: &[Task], now: DateTime<Utc>) -> Vec<Shed> {
    let mut by_actor: BTreeMap<String, Vec<Touched>> = BTreeMap::new();
    for t in tasks {
        let notes = store::parse_notes(&t.body);
        let claim = (t.status == Status::Shaving)
            .then(|| claimant(&notes))
            .flatten();
        // Each actor's newest note on this yak.
        let mut last: BTreeMap<&str, &str> = BTreeMap::new();
        for n in &notes {
            if let Some(a) = n.actor.as_deref() {
                if last.get(a).is_none_or(|p| *p <= n.ts.as_str()) {
                    last.insert(a, &n.ts);
                }
            }
        }
        if let Some((who, ts)) = &claim {
            let e = last.entry(who).or_insert(ts);
            if *e < ts.as_str() {
                *e = ts;
            }
        }
        for (actor, ts) in last {
            by_actor
                .entry(actor.to_string())
                .or_default()
                .push(Touched {
                    id: t.id.clone(),
                    title: t.title.clone(),
                    status: t.status,
                    needs: t.needs.clone(),
                    last: ts.to_string(),
                    working: claim.as_ref().is_some_and(|(w, _)| w == actor),
                });
        }
    }
    let active_within = chrono::Duration::from_std(ACTIVE_WITHIN).unwrap_or_default();
    let mut sheds: Vec<Shed> = by_actor
        .into_iter()
        .map(|(name, mut touched)| {
            touched.sort_by(|a, b| b.last.cmp(&a.last).then_with(|| a.id.cmp(&b.id)));
            let last_write = touched[0].last.clone();
            let recent = store::parse_ts(&last_write).is_some_and(|t| now - t <= active_within);
            let liveness = if recent {
                Liveness::Active
            } else if touched.iter().any(|t| t.working) {
                Liveness::Stale
            } else {
                Liveness::Idle
            };
            Shed {
                name,
                touched,
                last_write,
                liveness,
            }
        })
        .collect();
    sheds.sort_by(|a, b| {
        a.rank()
            .cmp(&b.rank())
            .then_with(|| b.last_write.cmp(&a.last_write))
            .then_with(|| a.name.cmp(&b.name))
    });
    sheds
}

/// `14m ago`, `3h ago`, `2d ago`; `just now` under a minute (or ahead of
/// `now`); the input itself when it is not a timestamp.
pub fn ago(ts: &str, now: DateTime<Utc>) -> String {
    let Some(t) = store::parse_ts(ts) else {
        return ts.to_string();
    };
    let secs = (now - t).num_seconds().max(0);
    match secs {
        0..60 => "just now".into(),
        60..3600 => format!("{}m ago", secs / 60),
        3600..86400 => format!("{}h ago", secs / 3600),
        _ => format!("{}d ago", secs / 86400),
    }
}

/// `farm: up to date with origin/yaks`, or how far apart they are.
pub fn farm_line(s: &SyncState) -> Option<String> {
    let up = s.upstream.as_ref()?;
    Some(if s.ahead == 0 && s.behind == 0 {
        format!("farm: up to date with {up}")
    } else {
        format!(
            "farm: {} ahead, {} behind {up} (as of the last fetch)",
            s.ahead, s.behind
        )
    })
}

/// `<name>  <liveness> · <ago>`, the header of a shed (and the TUI's detail
/// title), without the padding.
pub fn status_text(s: &Shed, now: DateTime<Utc>) -> String {
    format!("{} \u{b7} {}", s.liveness.as_str(), ago(&s.last_write, now))
}

/// One yak of a shed: `shaving  <id>  <title>`, `asking   <id>  <status>
/// <title>  <ago>`, `recent   <id>  <status>  <title>  <ago>`; `asking you`
/// ends a claimed yak that waits on a human. Shared with the TUI's detail.
pub fn yak_line(kind: Kind, t: &Touched, now: DateTime<Utc>) -> String {
    let label = format!("{:<8}", kind.label());
    let status = t.status.as_str();
    match kind {
        Kind::Working => {
            let ask = if t.asking() { "  asking you" } else { "" };
            format!("{label} {}  {}{ask}", t.id, t.title)
        }
        Kind::Asking | Kind::Recent => {
            format!(
                "{label} {}  {status}  {}  {}",
                t.id,
                t.title,
                ago(&t.last, now)
            )
        }
    }
}

/// The compact human view: the farm line, then one block per shed. Idle sheds
/// with no claim and no open ask are only counted unless `all`.
pub fn render(r: &Report, all: bool) -> String {
    let mut out = String::new();
    if let Some(line) = r.farm.as_ref().and_then(farm_line) {
        out.push_str(&line);
        out.push_str("\n\n");
    }
    if r.sheds.is_empty() {
        out.push_str("No actor has written to this farm yet.\n");
        return out;
    }
    let shown: Vec<&Shed> = r.sheds.iter().filter(|s| all || !s.quiet()).collect();
    let width = shown
        .iter()
        .map(|s| s.name.chars().count())
        .max()
        .unwrap_or(0)
        .min(40);
    for s in &shown {
        out.push_str(&format!("{:<width$}  {}\n", s.name, status_text(s, r.now)));
        let mut recent = 0;
        let mut more = 0;
        for (kind, t) in s.rows() {
            if kind == Kind::Recent {
                recent += 1;
                if recent > RECENT_SHOWN {
                    more += 1;
                    continue;
                }
            }
            out.push_str(&format!("  {}\n", yak_line(kind, t, r.now)));
        }
        if more > 0 {
            out.push_str(&format!("  (+{more} more; --json lists them all)\n"));
        }
    }
    let hidden = r.sheds.len() - shown.len();
    if hidden > 0 {
        out.push_str(&format!(
            "({hidden} idle {}; --all lists them)\n",
            if hidden == 1 { "actor" } else { "actors" }
        ));
    }
    out
}

/// JSON for `--json`: the farm freshness and every shed (idle ones too), with
/// the same data as the table.
pub fn to_json(r: &Report) -> Value {
    json!({
        "farm": r.farm.as_ref().map(|s| json!({
            "branch": s.branch,
            "upstream": s.upstream,
            "ahead": s.ahead,
            "behind": s.behind,
        })),
        "sheds": r.sheds.iter().map(|s| json!({
            "name": s.name,
            "liveness": s.liveness.as_str(),
            "last_write": s.last_write,
            // Claimed yaks, and yaks waiting on a human, by id.
            "working": s.working().map(|t| &t.id).collect::<Vec<_>>(),
            "asking": s.asks().map(|t| &t.id).collect::<Vec<_>>(),
            // Every yak it wrote, newest write first.
            "touched": s.touched.iter().map(|t| json!({
                "id": t.id,
                "title": t.title,
                "status": t.status.as_str(),
                "needs": t.needs,
                "last": t.last,
                "working": t.working,
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: &str = "2026-10-31T16:00:00Z";

    fn now() -> DateTime<Utc> {
        store::parse_ts(NOW).unwrap()
    }

    fn yak(id: &str, status: Status) -> Task {
        Task {
            id: id.into(),
            title: format!("{id} title"),
            kind: "task".into(),
            priority: 2,
            status,
            created: None,
            updated: None,
            parent: None,
            labels: vec![],
            depends_on: vec![],
            source: None,
            needs: None,
            verify: None,
            extra: vec![],
            body: "A description.".into(),
        }
    }

    /// A note by `actor` (`None`: a bare, pre-actor note) at `ts`.
    fn note(t: &mut Task, ts: &str, actor: Option<&str>, text: &str) {
        t.body = store::append_note(&t.body, ts, actor, text);
    }

    /// A move into `to`, as `yaks move` writes it, with note metadata.
    fn moved(t: &mut Task, ts: &str, actor: &str, from: Status, to: Status, meta: &[(&str, &str)]) {
        t.body = store::append_note_meta(
            &t.body,
            ts,
            Some(actor),
            meta,
            &store::transition_text(from, to),
        );
        t.status = to;
    }

    fn shed<'a>(sheds: &'a [Shed], name: &str) -> &'a Shed {
        sheds.iter().find(|s| s.name == name).unwrap()
    }

    fn ids<'a>(it: impl Iterator<Item = &'a Touched>) -> Vec<&'a str> {
        it.map(|t| t.id.as_str()).collect()
    }

    /// A coordinator claims a yak for a worker; the worker writes after. The
    /// worker, not the mover, is working on it.
    #[test]
    fn the_most_recent_writer_since_the_move_is_working_on_it() {
        let mut a = yak("y-a", Status::Hairy);
        note(&mut a, "2026-10-31T10:00:00Z", Some("boss"), "plan");
        moved(
            &mut a,
            "2026-10-31T11:00:00Z",
            "boss",
            Status::Hairy,
            Status::Shaving,
            &[],
        );
        note(
            &mut a,
            "2026-10-31T11:00:01Z",
            Some("boss"),
            "Claim: worker w1.",
        );
        note(&mut a, "2026-10-31T11:20:00Z", Some("w1"), "started");
        // Notes with no actor belong to nobody.
        note(&mut a, "2026-10-31T11:30:00Z", None, "an old bare note");
        let sheds = build(&[a], now());
        assert_eq!(ids(shed(&sheds, "w1").working()), ["y-a"]);
        assert!(shed(&sheds, "boss").working().next().is_none());
        // The coordinator still wrote it: recent, not claimed.
        assert_eq!(
            shed(&sheds, "boss")
                .rows()
                .iter()
                .map(|(k, _)| *k)
                .collect::<Vec<_>>(),
            [Kind::Recent]
        );
        assert_eq!(sheds.len(), 2, "{sheds:?}");
        assert_eq!(shed(&sheds, "w1").last_write, "2026-10-31T11:20:00Z");
    }

    /// Until the worker writes, the coordinator's own note is the latest: the
    /// rule has no `for:` to go on.
    #[test]
    fn a_claim_nobody_has_written_to_is_the_movers() {
        let mut a = yak("y-a", Status::Hairy);
        moved(
            &mut a,
            "2026-10-31T11:00:00Z",
            "boss",
            Status::Hairy,
            Status::Shaving,
            &[],
        );
        let sheds = build(&[a], now());
        assert_eq!(ids(shed(&sheds, "boss").working()), ["y-a"]);
    }

    #[test]
    fn for_names_the_claimant_before_it_has_written_anything() {
        let mut a = yak("y-a", Status::Hairy);
        moved(
            &mut a,
            "2026-10-31T15:00:00Z",
            "boss",
            Status::Hairy,
            Status::Shaving,
            &[("for", "w2")],
        );
        note(&mut a, "2026-10-31T15:05:00Z", Some("boss"), "Brief sent.");
        let sheds = build(&[a], now());
        let w2 = shed(&sheds, "w2");
        assert_eq!(ids(w2.working()), ["y-a"]);
        // Stamped with the move that handed it over, so it reads active.
        assert_eq!(w2.last_write, "2026-10-31T15:00:00Z");
        assert_eq!(w2.liveness, Liveness::Active);
        assert!(shed(&sheds, "boss").working().next().is_none());
    }

    #[test]
    fn only_writes_since_the_latest_move_into_shaving_count() {
        let mut a = yak("y-a", Status::Hairy);
        moved(
            &mut a,
            "2026-10-30T09:00:00Z",
            "w1",
            Status::Hairy,
            Status::Shaving,
            &[],
        );
        note(&mut a, "2026-10-30T09:30:00Z", Some("w1"), "progress");
        moved(
            &mut a,
            "2026-10-30T10:00:00Z",
            "w1",
            Status::Shaving,
            Status::Hairy,
            &[],
        );
        moved(
            &mut a,
            "2026-10-31T09:00:00Z",
            "w3",
            Status::Hairy,
            Status::Shaving,
            &[],
        );
        let sheds = build(&[a], now());
        assert_eq!(ids(shed(&sheds, "w3").working()), ["y-a"]);
        assert!(shed(&sheds, "w1").working().next().is_none());
        assert_eq!(ids(shed(&sheds, "w1").touched.iter()), ["y-a"]);
    }

    #[test]
    fn a_yak_that_is_not_shaving_is_nobodys_claim() {
        let mut a = yak("y-a", Status::Hairy);
        moved(
            &mut a,
            "2026-10-31T09:00:00Z",
            "w1",
            Status::Hairy,
            Status::Shaving,
            &[],
        );
        moved(
            &mut a,
            "2026-10-31T09:10:00Z",
            "w1",
            Status::Shaving,
            Status::Shorn,
            &[],
        );
        let sheds = build(&[a], now());
        let w1 = shed(&sheds, "w1");
        assert!(w1.working().next().is_none());
        assert_eq!(w1.touched[0].status, Status::Shorn);
    }

    #[test]
    fn an_asking_yak_ranks_its_actor_first_even_when_idle() {
        let mut ask = yak("y-ask", Status::Hairy);
        ask.needs = Some("human".into());
        note(
            &mut ask,
            "2026-10-01T09:00:00Z",
            Some("old-1"),
            "asked: needs human",
        );
        let mut busy = yak("y-busy", Status::Shaving);
        moved(
            &mut busy,
            "2026-10-31T15:50:00Z",
            "w1",
            Status::Hairy,
            Status::Shaving,
            &[],
        );
        let sheds = build(&[ask, busy], now());
        let names: Vec<_> = sheds.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, ["old-1", "w1"]);
        let old = &sheds[0];
        assert_eq!(old.liveness, Liveness::Idle);
        assert!(!old.quiet());
        assert_eq!(ids(old.asks()), ["y-ask"]);
        assert_eq!(old.rows()[0].0, Kind::Asking);
    }

    #[test]
    fn liveness_is_active_stale_or_idle_and_orders_the_rest() {
        let mut fresh = yak("y-fresh", Status::Shaving);
        moved(
            &mut fresh,
            "2026-10-31T15:00:00Z",
            "fresh",
            Status::Hairy,
            Status::Shaving,
            &[],
        );
        let mut quiet_claim = yak("y-claim", Status::Shaving);
        moved(
            &mut quiet_claim,
            "2026-10-31T12:00:00Z",
            "stale",
            Status::Hairy,
            Status::Shaving,
            &[],
        );
        let mut done = yak("y-done", Status::Shorn);
        moved(
            &mut done,
            "2026-10-31T13:00:00Z",
            "idle-new",
            Status::Shaving,
            Status::Shorn,
            &[],
        );
        let mut older = yak("y-older", Status::Shorn);
        moved(
            &mut older,
            "2026-10-20T13:00:00Z",
            "idle-old",
            Status::Shaving,
            Status::Shorn,
            &[],
        );
        let sheds = build(&[older, done, quiet_claim, fresh], now());
        let got: Vec<_> = sheds
            .iter()
            .map(|s| (s.name.as_str(), s.liveness))
            .collect();
        assert_eq!(
            got,
            [
                ("fresh", Liveness::Active),
                ("stale", Liveness::Stale),
                ("idle-new", Liveness::Idle),
                ("idle-old", Liveness::Idle),
            ]
        );
    }

    #[test]
    fn ago_reads_like_a_chat_timestamp() {
        let n = now();
        assert_eq!(ago("2026-10-31T15:59:30Z", n), "just now");
        assert_eq!(ago("2026-10-31T15:46:00Z", n), "14m ago");
        assert_eq!(ago("2026-10-31T13:00:00Z", n), "3h ago");
        assert_eq!(ago("2026-10-29T16:00:00Z", n), "2d ago");
        assert_eq!(ago("2026-11-01T00:00:00Z", n), "just now");
        assert_eq!(ago("garbage", n), "garbage");
    }

    /// A fixture farm with interleaved actors, a coordinator's claim, an ask
    /// and idle actors.
    fn farm() -> Vec<Task> {
        let mut a = yak("mem-f13a", Status::Hairy);
        moved(
            &mut a,
            "2026-10-31T15:30:00Z",
            "boss",
            Status::Hairy,
            Status::Shaving,
            &[("for", "walk-1")],
        );
        note(&mut a, "2026-10-31T15:51:00Z", Some("walk-1"), "mode added");
        let mut b = yak("mem-7aba", Status::Hairy);
        b.needs = Some("human".into());
        moved(
            &mut b,
            "2026-10-31T15:00:00Z",
            "tex-1",
            Status::Hairy,
            Status::Shaving,
            &[],
        );
        note(
            &mut b,
            "2026-10-31T15:46:00Z",
            Some("tex-1"),
            "asked: needs human\n\nWhich licence?",
        );
        // `ask` leaves the yak shaving.
        b.status = Status::Shaving;
        let mut done = yak("mem-99d0", Status::Shorn);
        moved(
            &mut done,
            "2026-10-28T10:00:00Z",
            "walk-1",
            Status::Shaving,
            Status::Shorn,
            &[],
        );
        let mut old = yak("mem-0001", Status::Shorn);
        moved(
            &mut old,
            "2026-10-01T10:00:00Z",
            "gone-1",
            Status::Shaving,
            Status::Shorn,
            &[],
        );
        let mut old2 = yak("mem-0002", Status::Dead);
        moved(
            &mut old2,
            "2026-10-02T10:00:00Z",
            "gone-2",
            Status::Hairy,
            Status::Dead,
            &[],
        );
        vec![a, b, done, old, old2]
    }

    #[test]
    fn the_text_table_lists_attention_first_and_counts_the_idle() {
        let r = Report {
            now: now(),
            farm: Some(SyncState {
                branch: "yaks".into(),
                upstream: Some("origin/yaks".into()),
                ahead: 0,
                behind: 0,
            }),
            sheds: build(&farm(), now()),
        };
        let want = "\
farm: up to date with origin/yaks

tex-1   active · 14m ago
  shaving  mem-7aba  mem-7aba title  asking you
walk-1  active · 9m ago
  shaving  mem-f13a  mem-f13a title
  recent   mem-99d0  shorn  mem-99d0 title  3d ago
boss    active · 30m ago
  recent   mem-f13a  shaving  mem-f13a title  30m ago
(2 idle actors; --all lists them)
";
        assert_eq!(render(&r, false), want);
        let all = render(&r, true);
        assert!(
            all.contains("gone-1  idle · 30d ago\n  recent   mem-0001  shorn"),
            "{all}"
        );
        assert!(all.contains("gone-2") && !all.contains("--all"), "{all}");
    }

    #[test]
    fn recent_is_capped_with_a_pointer_to_json() {
        let tasks: Vec<Task> = (0..8)
            .map(|i| {
                let mut t = yak(&format!("y-{i}"), Status::Shorn);
                moved(
                    &mut t,
                    &format!("2026-10-31T15:{:02}:00Z", 10 + i),
                    "w1",
                    Status::Shaving,
                    Status::Shorn,
                    &[],
                );
                t
            })
            .collect();
        let r = Report {
            now: now(),
            farm: None,
            sheds: build(&tasks, now()),
        };
        let text = render(&r, false);
        assert_eq!(text.matches("recent ").count(), 5, "{text}");
        assert!(text.contains("(+3 more; --json lists them all)"), "{text}");
        // Newest first.
        assert!(text.find("y-7").unwrap() < text.find("y-3").unwrap());
        // JSON lists them all.
        assert_eq!(
            to_json(&r)["sheds"][0]["touched"].as_array().unwrap().len(),
            8
        );
    }

    #[test]
    fn json_carries_everyone_and_the_farm_freshness() {
        let r = Report {
            now: now(),
            farm: Some(SyncState {
                branch: "yaks".into(),
                upstream: Some("origin/yaks".into()),
                ahead: 2,
                behind: 1,
            }),
            sheds: build(&farm(), now()),
        };
        let v = to_json(&r);
        assert_eq!(v["farm"]["ahead"], 2);
        assert_eq!(v["farm"]["upstream"], "origin/yaks");
        let sheds = v["sheds"].as_array().unwrap();
        assert_eq!(sheds.len(), 5);
        assert_eq!(sheds[0]["name"], "tex-1");
        assert_eq!(sheds[0]["asking"], json!(["mem-7aba"]));
        assert_eq!(sheds[0]["working"], json!(["mem-7aba"]));
        assert_eq!(sheds[1]["name"], "walk-1");
        assert_eq!(sheds[1]["touched"][1]["working"], false);
        assert_eq!(sheds[2]["name"], "boss");
        assert_eq!(sheds[3]["liveness"], "idle");
        let line = farm_line(r.farm.as_ref().unwrap()).unwrap();
        assert_eq!(
            line,
            "farm: 2 ahead, 1 behind origin/yaks (as of the last fetch)"
        );
        // No upstream: no farm entry and no line.
        let r = Report { farm: None, ..r };
        assert!(to_json(&r)["farm"].is_null());
        assert!(!render(&r, false).contains("farm:"));
    }

    #[test]
    fn an_empty_farm_says_so() {
        let r = Report {
            now: now(),
            farm: None,
            sheds: build(&[yak("y-a", Status::Hairy)], now()),
        };
        assert_eq!(
            render(&r, false),
            "No actor has written to this farm yet.\n"
        );
    }
}
