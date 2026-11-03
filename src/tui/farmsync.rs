//! The farm's git state in the TUI (yaks-ce5d): a chip on the tab row saying
//! what the farm has that git does not (uncommitted yaks) and where the farm
//! branch stands against its upstream, plus the two keys that act on it —
//! `W` commits the farm, `U` syncs it (`yaks commit` / `yaks sync`).
//!
//! The chip is read from local git only (never a fetch) when the farm is
//! loaded, reloaded or acted on, and `W`/`U` commit and sync only on the key.
//! The one thing that runs by itself is the interval sync ([`AutoSync`],
//! yaks-f0db): every `sync_interval` the event loop starts `yaks sync` on a
//! background thread, so upstream changes show up with no key pressed.
//! `use super::*` inherits `tui.rs`'s scope.

use super::*;
use crate::status::SyncState;
use crate::{changes, commit, privatefarm, status, store, sync};
use std::sync::mpsc::TryRecvError;
use std::time::Instant;

/// What the chip shows: the farm's uncommitted yak count and, for a farm that
/// is a branch, its standing against the upstream. A farm that is not a farm
/// branch has no `sync` part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FarmSync {
    /// Yaks plus other farm files (config, ...) git does not have yet.
    pub dirty: usize,
    pub sync: Option<SyncState>,
}

impl FarmSync {
    /// Read the farm at `root` (local git only). `None` for a farm outside git
    /// (nothing is tracked) or when git cannot say: no chip then.
    ///
    /// Counts the changes the way `yaks status` does ([`status::collect`]:
    /// `yaks + other` of the classified changes) without its per-yak diff
    /// against HEAD, which would cost a `git show` per dirty yak on every
    /// reload.
    pub(crate) fn read(root: &Path) -> Option<FarmSync> {
        let changes = changes::read(root).ok()??;
        let classified = changes::classify(&changes);
        Some(FarmSync {
            dirty: classified.yaks.len() + classified.other.len(),
            sync: status::sync_state(root),
        })
    }

    /// The chip's segments, in order, each with whether it is a warning:
    /// `● 3 uncommitted`, `not published`, `↑2`, `↓1` (the warning). Empty
    /// when the farm is clean and up to date. `compact` is the short form for
    /// a tab row with little room: `●3`, `unpub`.
    pub(crate) fn segments(&self, compact: bool) -> Vec<(String, bool)> {
        let mut out = Vec::new();
        if self.dirty > 0 {
            out.push((
                if compact {
                    format!("●{}", self.dirty)
                } else {
                    format!("● {} uncommitted", self.dirty)
                },
                false,
            ));
        }
        if let Some(s) = &self.sync {
            if s.upstream.is_none() {
                out.push((
                    if compact { "unpub" } else { "not published" }.to_string(),
                    false,
                ));
            }
            if s.ahead > 0 {
                out.push((format!("↑{}", s.ahead), false));
            }
            if s.behind > 0 {
                out.push((format!("↓{}", s.behind), true));
            }
        }
        out
    }
}

/// The interval sync's state: when the next one is due, the one running (on a
/// background thread, so the UI never waits for the network), and the last
/// failure shown (so a failing remote says so once, not every tick).
pub(crate) struct AutoSync {
    /// `None`: off (`sync_interval: off`, or `YAKS_NO_AUTOSYNC`, or no farm).
    pub interval: Option<Duration>,
    next: Instant,
    running: Option<Receiver<Result<(), String>>>,
    last_error: Option<String>,
}

impl AutoSync {
    pub(crate) fn off() -> Self {
        AutoSync {
            interval: None,
            next: Instant::now(),
            running: None,
            last_error: None,
        }
    }

    /// Syncing every `interval` from `now`; off when `disabled` (the
    /// `YAKS_NO_AUTOSYNC` switch, see [`autosync_disabled_by_env`]).
    pub(crate) fn new(interval: Option<Duration>, disabled: bool, now: Instant) -> Self {
        AutoSync {
            interval: interval.filter(|_| !disabled),
            next: interval.map_or(now, |i| now + i),
            running: None,
            last_error: None,
        }
    }

    /// Whether a tick at `now` should start a sync: it is on, due, and none is
    /// running. (The farm and UI conditions are [`App::tick_auto_sync`]'s.)
    pub(crate) fn due(&self, now: Instant) -> bool {
        self.interval.is_some() && self.running.is_none() && now >= self.next
    }

    fn postpone(&mut self, now: Instant) {
        if let Some(i) = self.interval {
            self.next = now + i;
        }
    }
}

/// `YAKS_NO_AUTOSYNC=1` (anything but empty, `0` or `false`) keeps the TUI
/// from syncing by itself: tests and headless runs set it.
pub(crate) fn autosync_disabled_by_env() -> bool {
    autosync_disabled(std::env::var("YAKS_NO_AUTOSYNC").ok().as_deref())
}

fn autosync_disabled(value: Option<&str>) -> bool {
    value.is_some_and(|v| !matches!(v.trim(), "" | "0" | "false"))
}

/// Whether `yaks sync` has a remote to talk to: the farm branch's configured
/// remote, else `origin` (the same rule as `sync::run`'s), and the farm is a
/// branch at all.
fn has_remote(root: &Path) -> bool {
    let (Some(top), Some(branch)) = (root.parent(), store::farm_branch(root)) else {
        return false;
    };
    let git = |args: &[&str]| {
        crate::branchfarm::git(top, args)
            .ok()
            .filter(|s| !s.is_empty())
    };
    let remote = git(&["config", "--get", &format!("branch.{branch}.remote")])
        .unwrap_or_else(|| "origin".to_string());
    git(&["remote", "get-url", &remote]).is_some()
}

/// A farm operation a key asked for. Run by the event loop after the frame
/// that shows its `committing…` / `syncing…` notification is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FarmOp {
    Commit,
    Sync,
}

impl App {
    /// Re-read the chip's state from git. Cheap (local git, no fetch); called
    /// wherever the farm is (re)loaded.
    pub(crate) fn refresh_farm_sync(&mut self) {
        if let Some(f) = &self.farm {
            self.farm_sync = FarmSync::read(f.root());
        }
    }

    /// `W` / `U`: queue `op` and say so; the event loop draws that frame, then
    /// calls [`App::run_pending_farm_op`]. A no-op without a farm.
    pub(crate) fn request_farm_op(&mut self, op: FarmOp) {
        if self.farm.is_none() {
            return;
        }
        self.notification = Some(match op {
            FarmOp::Commit => "committing…".to_string(),
            FarmOp::Sync => "syncing…".to_string(),
        });
        self.pending_farm_op = Some(op);
    }

    /// One turn of the event loop for the interval sync. Collects a finished
    /// background sync (a failure becomes the notification, once per distinct
    /// message), and starts one when it is due, the UI is idle (no overlay, no
    /// queued `W`/`U`) and the farm has a remote. Returns whether a sync just
    /// finished, so the caller reloads the farm (a pull rewrites yak files).
    ///
    /// A private farm pulls but never pushes (`no_push`); `sync::run` would
    /// skip the push anyway, this keeps it from even trying.
    pub(crate) fn tick_auto_sync(&mut self, now: Instant) -> bool {
        // A finished sync may have pulled others' shears: reopen what recurs.
        let finished = if self.collect_auto_sync(now, false) {
            if !self.warn_newer_schema() {
                self.regrow_due();
            }
            true
        } else {
            false
        };
        if !self.auto_sync.due(now)
            || !matches!(self.overlay, Overlay::None)
            || self.pending_farm_op.is_some()
        {
            return finished;
        }
        let Some(root) = self.farm.as_ref().map(|f| f.root().to_path_buf()) else {
            return finished;
        };
        self.auto_sync.postpone(now);
        if !has_remote(&root) {
            // No sync to wait for: the tick itself is the clock.
            return self.regrow_due() || finished;
        }
        let opts = sync::Opts {
            no_push: root.parent().is_some_and(privatefarm::is_private),
            ..sync::Opts::default()
        };
        let (tx, rx) = mpsc::channel();
        self.auto_sync.running = Some(rx);
        std::thread::spawn(move || {
            let _ = tx.send(
                sync::run(&root, &opts)
                    .map(|_| ())
                    .map_err(|e| e.to_string()),
            );
        });
        finished
    }

    /// Reopen the recurring yaks whose next occurrence has arrived
    /// (`yaks regrow-due`, yaks-2fc5): at load and on each auto-sync tick. The
    /// clock lives outside yaks; the TUI is one harness that calls it. Local
    /// and cheap (a read, then a write per yak that regrows); a failure is
    /// not worth interrupting for. True when something regrew, so the caller
    /// reloads.
    pub(crate) fn regrow_due(&mut self) -> bool {
        let Some(farm) = &self.farm else {
            return false;
        };
        let actor = crate::actor::resolve(None);
        let Ok(report) = farm.regrow_due(chrono::Utc::now(), false, actor.as_deref()) else {
            return false;
        };
        if report.regrew.is_empty() {
            return false;
        }
        self.notification = Some(regrew_notice(&report));
        true
    }

    /// Take a finished background sync's result (waiting for it when `wait`).
    /// True when one finished.
    fn collect_auto_sync(&mut self, now: Instant, wait: bool) -> bool {
        let Some(rx) = &self.auto_sync.running else {
            return false;
        };
        let result = if wait {
            rx.recv().map_err(|_| TryRecvError::Disconnected)
        } else {
            rx.try_recv()
        };
        let result = match result {
            Ok(r) => r,
            Err(TryRecvError::Empty) => return false,
            Err(TryRecvError::Disconnected) => Err("the sync thread stopped".to_string()),
        };
        self.auto_sync.running = None;
        self.auto_sync.postpone(now);
        match result {
            Ok(()) => self.auto_sync.last_error = None,
            Err(e) => {
                let msg = one_line(&format!("auto-sync failed: {e}"));
                if self.auto_sync.last_error.as_deref() != Some(&msg) {
                    self.notification = Some(msg.clone());
                }
                self.auto_sync.last_error = Some(msg);
            }
        }
        true
    }

    /// Run the queued commit/sync synchronously (sync touches the network),
    /// reload, and leave a one-line result in the notification.
    pub(crate) fn run_pending_farm_op(&mut self) {
        let Some(op) = self.pending_farm_op.take() else {
            return;
        };
        let Some(root) = self.farm.as_ref().map(|f| f.root().to_path_buf()) else {
            return;
        };
        let result = match op {
            FarmOp::Commit => commit::run(&root, None, false).map(|outcome| match outcome {
                commit::Outcome::Nothing => "nothing to commit".to_string(),
                commit::Outcome::Done(plan) => format!(
                    "committed {} {}",
                    plan.commit.as_deref().unwrap_or("?"),
                    plan.message.lines().next().unwrap_or_default()
                ),
            }),
            FarmOp::Sync => sync::run(&root, &sync::Opts::default()).map(|report| {
                report
                    .render()
                    .lines()
                    .rfind(|l| !l.trim().is_empty())
                    .unwrap_or("synced")
                    .to_string()
            }),
        };
        // A sync can rewrite yak files (the rebase), so re-read them too.
        self.reload_preserving_selection();
        self.notification = Some(match result {
            Ok(line) => one_line(&line),
            Err(e) => one_line(&format!("error: {e}")),
        });
        self.warn_newer_schema();
    }

    /// After a pull: if it brought in a farm on a newer schema (another
    /// machine ran `yaks migrate`), say so. Every write is already refused
    /// under the farm lock ([`store::lock`]); this tells the user why, and
    /// that this TUI needs an upgraded yaks and a restart (yaks-9786). True
    /// when it warned.
    pub(crate) fn warn_newer_schema(&mut self) -> bool {
        let Some(why) = self
            .farm
            .as_ref()
            .and_then(|f| store::newer_schema_refusal(f.root()))
        else {
            return false;
        };
        self.notification = Some(one_line(&format!("read-only: {why}")));
        true
    }
}

/// `regrew yaks-ab12: Check the salt`, or a count for several.
pub(super) fn regrew_notice(r: &crate::recur::Report) -> String {
    match r.regrew.as_slice() {
        [one] => one_line(&format!("regrew {}: {}", one.id, one.title)),
        many => format!("regrew {} recurring yaks", many.len()),
    }
}

/// Collapse a message to one line (newlines become spaces).
fn one_line(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::test_support::*;

    fn git_in(dir: &Path, args: &[&str]) {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}: {out:?}");
    }

    /// A farm that is the branch `yaks` of its own repo, published to a bare
    /// origin; returns (project dir, the TUI on it, origin dir). `tmp` names
    /// the scratch directory so tests do not collide.
    fn published_farm(tmp: &str) -> (PathBuf, App, PathBuf) {
        let (proj, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        let origin = proj.with_file_name(format!("{tmp}-origin.git"));
        let _ = std::fs::remove_dir_all(&origin);
        git_in(&proj, &["init", "-q", "-b", "yaks"]);
        git_in(&proj, &["config", "user.name", "t"]);
        git_in(&proj, &["config", "user.email", "t@t"]);
        git_in(&proj, &["add", "."]);
        git_in(&proj, &["commit", "-q", "-m", "init"]);
        git_in(&proj, &["init", "-q", "--bare", origin.to_str().unwrap()]);
        git_in(
            &proj,
            &["remote", "add", "origin", origin.to_str().unwrap()],
        );
        git_in(&proj, &["push", "-q", "-u", "origin", "yaks"]);
        (proj, App::with_farm(farm).unwrap(), origin)
    }

    /// Tick until the background sync (if any) has finished; the final
    /// `finished` result. Fails the test rather than hang.
    fn settle_auto_sync(app: &mut App, start: Instant) -> bool {
        for i in 0..400 {
            let now = start + Duration::from_millis(i);
            let finished = app.tick_auto_sync(now);
            if finished || app.auto_sync.running.is_none() {
                return finished;
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        panic!("the interval sync did not finish");
    }

    #[test]
    fn env_switch_reads_like_a_flag() {
        assert!(!autosync_disabled(None));
        for off in ["", "0", "false", " 0 "] {
            assert!(!autosync_disabled(Some(off)), "{off:?}");
        }
        for on in ["1", "true", "yes"] {
            assert!(autosync_disabled(Some(on)), "{on:?}");
        }
    }

    #[test]
    fn due_only_when_on_idle_and_the_interval_has_passed() {
        let t0 = Instant::now();
        let every = Duration::from_secs(60);
        let a = AutoSync::new(Some(every), false, t0);
        assert!(!a.due(t0));
        assert!(!a.due(t0 + Duration::from_secs(59)));
        assert!(a.due(t0 + every));
        // Off by config, off by the env switch.
        assert!(!AutoSync::new(None, false, t0).due(t0 + every));
        assert!(!AutoSync::new(Some(every), true, t0).due(t0 + every));
        assert!(!AutoSync::off().due(t0 + every));
        // One already running is never joined by a second.
        let mut running = AutoSync::new(Some(every), false, t0);
        running.running = Some(mpsc::channel().1);
        assert!(!running.due(t0 + every));
    }

    #[test]
    fn a_farm_with_no_remote_never_starts_a_sync() {
        let (proj, farm) = temp_farm(&[task("t0", "solo", Status::Hairy, 3, None)]);
        git_in(&proj, &["init", "-q", "-b", "yaks"]);
        let mut app = App::with_farm(farm).unwrap();
        let t0 = Instant::now();
        app.auto_sync = AutoSync::new(Some(Duration::from_secs(1)), false, t0);
        let due = t0 + Duration::from_secs(1);
        assert!(!app.tick_auto_sync(due));
        assert!(app.auto_sync.running.is_none());
        assert!(app.notification.is_none());
        // The tick was spent: the next try is one interval later, not every frame.
        assert!(!app.auto_sync.due(due));
        assert!(app.auto_sync.due(due + Duration::from_secs(1)));
    }

    #[test]
    fn an_open_overlay_or_queued_op_holds_the_sync_until_idle() {
        let (_proj, mut app, _origin) = published_farm("yaks-f0db-idle");
        let t0 = Instant::now();
        app.auto_sync = AutoSync::new(Some(Duration::from_secs(1)), false, t0);
        let due = t0 + Duration::from_secs(1);
        app.overlay = Overlay::Confirm {
            prompt: "?".into(),
            action: ConfirmAction::Slaughter("t0".into()),
        };
        app.tick_auto_sync(due);
        assert!(app.auto_sync.running.is_none());
        assert!(app.auto_sync.due(due), "still due once the overlay closes");
        app.overlay = Overlay::None;
        app.pending_farm_op = Some(FarmOp::Commit);
        app.tick_auto_sync(due);
        assert!(app.auto_sync.running.is_none());
        app.pending_farm_op = None;
        app.tick_auto_sync(due);
        assert!(app.auto_sync.running.is_some(), "idle: it starts");
        settle_auto_sync(&mut app, due);
    }

    #[test]
    fn a_commit_pushed_from_another_clone_appears_after_a_tick() {
        let (proj, mut app, origin) = published_farm("yaks-f0db-pull");
        assert_eq!(app.all.len(), 1);

        // Another machine adds a yak and pushes it.
        let other = proj.with_file_name("yaks-f0db-pull-other");
        let _ = std::fs::remove_dir_all(&other);
        git_in(
            &proj,
            &[
                "clone",
                "-q",
                "-b",
                "yaks",
                origin.to_str().unwrap(),
                other.to_str().unwrap(),
            ],
        );
        git_in(&other, &["config", "user.name", "o"]);
        git_in(&other, &["config", "user.email", "o@t"]);
        store::write::save(
            &other.join(".yaks"),
            &task("t9", "from afar", Status::Hairy, 3, None),
        )
        .unwrap();
        git_in(&other, &["add", "."]);
        git_in(&other, &["commit", "-q", "-m", "yaks: add t9"]);
        git_in(&other, &["push", "-q", "origin", "yaks"]);
        assert!(
            app.farm_sync
                .as_ref()
                .is_some_and(|f| f.sync.as_ref().is_some_and(|s| s.behind == 0))
        );

        // A short interval, and nothing pressed: the tick pulls it in.
        let t0 = Instant::now();
        app.auto_sync = AutoSync::new(Some(Duration::from_millis(10)), false, t0);
        assert!(!app.tick_auto_sync(t0), "not due yet");
        let due = t0 + Duration::from_millis(10);
        assert!(!app.tick_auto_sync(due), "starting is not finishing");
        assert!(app.auto_sync.running.is_some());
        assert!(settle_auto_sync(&mut app, due));
        app.reload_preserving_selection(); // the loop's reload after `finished`
        assert!(
            app.all.iter().any(|t| t.id == "t9"),
            "the pulled yak is listed"
        );
        assert!(app.notification.is_none(), "a clean sync is silent");
        assert_eq!(
            app.farm_sync
                .as_ref()
                .and_then(|f| f.sync.as_ref())
                .map(|s| s.behind),
            Some(0)
        );
    }

    #[test]
    fn a_private_farm_pulls_but_does_not_push() {
        let (proj, mut app, origin) = published_farm("yaks-f0db-private");
        git_in(&proj, &["config", "--bool", "yaks.private", "true"]);
        // A local commit the origin lacks: a private farm keeps it local.
        git_in(
            &proj,
            &["commit", "-q", "--allow-empty", "-m", "local only"],
        );
        let t0 = Instant::now();
        app.auto_sync = AutoSync::new(Some(Duration::from_millis(1)), false, t0);
        let due = t0 + Duration::from_millis(1);
        app.tick_auto_sync(due);
        settle_auto_sync(&mut app, due);
        let ahead = std::process::Command::new("git")
            .arg("-C")
            .arg(&origin)
            .args(["log", "--oneline", "yaks"])
            .output()
            .unwrap();
        assert!(!String::from_utf8_lossy(&ahead.stdout).contains("local only"));
    }

    #[test]
    fn a_failing_remote_says_so_once_not_every_tick() {
        let (proj, mut app, origin) = published_farm("yaks-f0db-fail");
        // The remote goes away.
        std::fs::remove_dir_all(&origin).unwrap();
        let every = Duration::from_millis(1);
        let t0 = Instant::now();
        app.auto_sync = AutoSync::new(Some(every), false, t0);
        let mut at = t0 + every;
        app.tick_auto_sync(at);
        settle_auto_sync(&mut app, at);
        let first = app.notification.clone().expect("the failure is shown");
        assert!(first.starts_with("auto-sync failed: "), "{first}");
        assert!(!first.contains('\n'));
        // The same failure on the next tick stays quiet.
        app.notification = None;
        at += Duration::from_secs(1);
        app.tick_auto_sync(at);
        settle_auto_sync(&mut app, at);
        assert!(app.notification.is_none(), "{:?}", app.notification);
        // Once the remote is back, a clean tick forgets the error, so a later
        // failure is shown again.
        git_in(&proj, &["init", "-q", "--bare", origin.to_str().unwrap()]);
        at += Duration::from_secs(1);
        app.tick_auto_sync(at);
        settle_auto_sync(&mut app, at);
        assert!(app.auto_sync.last_error.is_none());
    }

    /// yaks-9786: another machine migrates the farm to a newer schema and
    /// pushes; after this TUI's interval sync pulls it, the TUI says it is
    /// read-only and every write is refused, naming the upgrade.
    #[test]
    fn a_pulled_newer_schema_makes_the_tui_read_only() {
        let (proj, mut app, origin) = published_farm("yaks-9786-newer");
        let other = proj.with_file_name("yaks-9786-newer-other");
        let _ = std::fs::remove_dir_all(&other);
        git_in(
            &proj,
            &[
                "clone",
                "-q",
                "-b",
                "yaks",
                origin.to_str().unwrap(),
                other.to_str().unwrap(),
            ],
        );
        git_in(&other, &["config", "user.name", "o"]);
        git_in(&other, &["config", "user.email", "o@t"]);
        std::fs::write(
            other.join(".yaks/schema"),
            format!("{}\n", store::SCHEMA + 1),
        )
        .unwrap();
        git_in(&other, &["add", "."]);
        git_in(&other, &["commit", "-q", "-m", "yaks: migrate"]);
        git_in(&other, &["push", "-q", "origin", "yaks"]);

        let t0 = Instant::now();
        app.auto_sync = AutoSync::new(Some(Duration::from_millis(1)), false, t0);
        let due = t0 + Duration::from_millis(1);
        app.tick_auto_sync(due);
        assert!(settle_auto_sync(&mut app, due));
        let note = app.notification.clone().expect("the TUI says why");
        assert!(note.starts_with("read-only: "), "{note}");
        assert!(note.contains("npm i -g @j15r/yaks@latest"), "{note}");

        let farm = app.farm.as_ref().unwrap();
        let err = farm
            .transition("t0", Status::Shaving, None)
            .expect_err("a write into a newer-schema farm is refused");
        assert!(err.to_string().contains("newer than this yaks"), "{err}");
        let t0_file = std::fs::read_to_string(store::task_path(farm.root(), "t0")).unwrap();
        assert!(t0_file.contains("status: hairy"), "nothing was written");
    }
}
