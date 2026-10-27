//! `yaks wait`: block until the inbox has something for an audience.
//!
//! A poll loop over [`crate::farm::Farm::inbox`] (plain file reads, no farm
//! lock, no daemon or watcher). The loop is generic over how rows are fetched
//! and over a [`Clock`], so tests drive it without sleeping for real.

use std::collections::HashSet;
use std::time::{Duration, Instant};

use anyhow::Result;

use crate::model::Task;

/// Time as the loop sees it: the real clock in the binary, a fake in tests.
pub trait Clock {
    fn now(&self) -> Instant;
    fn sleep(&self, d: Duration);
}

/// The wall clock and `std::thread::sleep`.
pub struct RealClock;

impl Clock for RealClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
    fn sleep(&self, d: Duration) {
        std::thread::sleep(d);
    }
}

/// How long and how often to poll, and whether rows present at start count.
pub struct WaitOpts {
    /// `None` waits forever. `Some(ZERO)` checks once.
    pub timeout: Option<Duration>,
    pub interval: Duration,
    /// Ignore rows (by id and `needs` value) already present on the first poll.
    pub new_only: bool,
}

/// The identity `--new` compares: a yak that flips `needs` (human to agent)
/// is a different row.
fn key(t: &Task) -> (String, Option<String>) {
    (t.id.clone(), t.needs.clone())
}

/// Poll `fetch` until some row passes `keep` (and, with `new_only`, was not
/// present at start), then return that poll's full row set for the caller to
/// render. `Ok(None)` on timeout.
///
/// With `new_only`, the baseline only holds rows that stay continuously
/// present: a row that leaves the inbox (picked up) and later comes back with
/// the same `needs` counts as new.
pub fn wait_for(
    mut fetch: impl FnMut() -> Result<Vec<Task>>,
    keep: impl Fn(&Task) -> bool,
    opts: &WaitOpts,
    clock: &impl Clock,
) -> Result<Option<Vec<Task>>> {
    let deadline = opts.timeout.map(|t| clock.now() + t);
    let mut baseline: Option<HashSet<(String, Option<String>)>> = None;
    loop {
        let rows = fetch()?;
        let present: HashSet<_> = rows.iter().filter(|t| keep(t)).map(key).collect();
        let hit = match (&baseline, opts.new_only) {
            (_, false) => !present.is_empty(),
            // First poll under --new: everything there is the baseline.
            (None, true) => false,
            (Some(seen), true) => present.iter().any(|k| !seen.contains(k)),
        };
        if hit {
            return Ok(Some(rows));
        }
        if opts.new_only {
            baseline = Some(match baseline {
                None => present,
                Some(seen) => seen.intersection(&present).cloned().collect(),
            });
        }
        let now = clock.now();
        let nap = match deadline {
            Some(d) if now >= d => return Ok(None),
            Some(d) => opts.interval.min(d - now),
            None => opts.interval,
        };
        clock.sleep(nap);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::farm::Farm;
    use crate::filter::FilterSpec;
    use crate::model::Status;
    use crate::store;
    use std::cell::{Cell, RefCell};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A clock that advances only when slept on.
    struct FakeClock {
        base: Instant,
        offset: Cell<Duration>,
        sleeps: Cell<usize>,
    }

    impl FakeClock {
        fn new() -> Self {
            FakeClock {
                base: Instant::now(),
                offset: Cell::new(Duration::ZERO),
                sleeps: Cell::new(0),
            }
        }
    }

    impl Clock for FakeClock {
        fn now(&self) -> Instant {
            self.base + self.offset.get()
        }
        fn sleep(&self, d: Duration) {
            self.offset.set(self.offset.get() + d);
            self.sleeps.set(self.sleeps.get() + 1);
        }
    }

    fn yak(id: &str, needs: Option<&str>) -> Task {
        Task {
            id: id.into(),
            title: format!("title {id}"),
            kind: "task".into(),
            priority: 3,
            status: Status::Shaving,
            created: Some("2026-01-01T00:00:00Z".into()),
            updated: Some("2026-01-01T00:00:00Z".into()),
            parent: None,
            labels: vec![],
            depends_on: vec![],
            source: None,
            needs: needs.map(str::to_string),
            verify: None,
            extra: Vec::new(),
            body: String::new(),
        }
    }

    fn opts(timeout: Option<u64>, new_only: bool) -> WaitOpts {
        WaitOpts {
            timeout: timeout.map(Duration::from_secs),
            interval: Duration::from_secs(2),
            new_only,
        }
    }

    fn ids(rows: &[Task]) -> Vec<&str> {
        rows.iter().map(|t| t.id.as_str()).collect()
    }

    #[test]
    fn returns_immediately_when_the_inbox_has_a_row() {
        let clock = FakeClock::new();
        let rows = wait_for(
            || Ok(vec![yak("y-1", Some("agent"))]),
            |t| t.awaiting_agent(),
            &opts(Some(30), false),
            &clock,
        )
        .unwrap()
        .expect("a hit");
        assert_eq!(ids(&rows), vec!["y-1"]);
        assert_eq!(clock.sleeps.get(), 0, "no sleep before the first poll");
    }

    #[test]
    fn times_out_when_nothing_matches_the_audience() {
        let clock = FakeClock::new();
        let polls = Cell::new(0);
        let out = wait_for(
            || {
                polls.set(polls.get() + 1);
                // A row for the other audience does not count.
                Ok(vec![yak("y-1", Some("human"))])
            },
            |t| t.awaiting_agent(),
            &opts(Some(5), false),
            &clock,
        )
        .unwrap();
        assert!(out.is_none());
        // Polls at 0, 2, 4, 5 (the last nap is clipped to the deadline).
        assert_eq!(polls.get(), 4);
        assert_eq!(clock.offset.get(), Duration::from_secs(5));
    }

    #[test]
    fn zero_timeout_checks_once() {
        let clock = FakeClock::new();
        let out = wait_for(|| Ok(vec![]), |_| true, &opts(Some(0), false), &clock).unwrap();
        assert!(out.is_none());
        assert_eq!(clock.sleeps.get(), 0);
    }

    #[test]
    fn new_ignores_rows_present_at_start_but_sees_a_needs_flip() {
        let clock = FakeClock::new();
        let poll = Cell::new(0);
        let script = |n: usize| match n {
            0 | 1 => vec![yak("y-1", Some("human"))],
            // y-1 is answered: same id, new `needs` value, so it is new.
            _ => vec![yak("y-1", Some("agent"))],
        };
        let rows = wait_for(
            || {
                let n = poll.get();
                poll.set(n + 1);
                Ok(script(n))
            },
            |_| true,
            &opts(Some(60), true),
            &clock,
        )
        .unwrap()
        .expect("the flip is a hit");
        assert_eq!(rows[0].needs.as_deref(), Some("agent"));
        assert_eq!(poll.get(), 3);
    }

    #[test]
    fn new_times_out_when_only_old_rows_remain() {
        let clock = FakeClock::new();
        let out = wait_for(
            || Ok(vec![yak("y-1", Some("agent"))]),
            |t| t.awaiting_agent(),
            &opts(Some(10), true),
            &clock,
        )
        .unwrap();
        assert!(out.is_none());
    }

    #[test]
    fn new_counts_a_row_that_left_and_came_back() {
        let clock = FakeClock::new();
        let poll = RefCell::new(0usize);
        let rows = wait_for(
            || {
                let mut n = poll.borrow_mut();
                *n += 1;
                Ok(match *n {
                    1 => vec![yak("y-1", Some("agent"))],
                    2 => vec![],                          // picked up
                    _ => vec![yak("y-1", Some("agent"))], // answered again
                })
            },
            |_| true,
            &opts(Some(60), true),
            &clock,
        )
        .unwrap();
        assert!(rows.is_some());
        assert_eq!(*poll.borrow(), 3);
    }

    static SEQ: AtomicUsize = AtomicUsize::new(0);

    fn temp_farm() -> (PathBuf, Farm) {
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        let parent = std::env::temp_dir().join(format!("yaksrs-wait-{}-{n}", std::process::id()));
        let root = parent.join(".yaks");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("schema"), format!("{}\n", store::SCHEMA)).unwrap();
        let farm = Farm::at(&root).unwrap_or_else(|_| panic!("open {parent:?}"));
        (root, farm)
    }

    /// Against real files and the real clock: another writer answers while
    /// `wait` is polling, and the wait wakes with the answered yak.
    #[test]
    fn wakes_when_another_writer_answers_mid_wait() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &yak("yak-0001", None)).unwrap();
        farm.ask("yak-0001", "human", Some("agt"), Some("which?"))
            .unwrap();

        let writer_root = root.parent().unwrap().to_path_buf();
        let writer = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(150));
            let farm = Farm::at(&writer_root.join(".yaks")).unwrap_or_else(|_| panic!("open"));
            farm.answer("yak-0001", Some("joel"), Some("this one"), false)
                .unwrap();
        });

        let start = Instant::now();
        let rows = wait_for(
            || farm.inbox(FilterSpec::default()),
            |t| t.awaiting_agent(),
            &WaitOpts {
                timeout: Some(Duration::from_secs(10)),
                interval: Duration::from_millis(20),
                new_only: false,
            },
            &RealClock,
        )
        .unwrap()
        .expect("woken by the answer");
        writer.join().unwrap();
        assert_eq!(ids(&rows), vec!["yak-0001"]);
        assert!(rows[0].awaiting_agent());
        assert!(start.elapsed() >= Duration::from_millis(150));
    }
}
