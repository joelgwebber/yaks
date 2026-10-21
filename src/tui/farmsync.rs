//! The farm's git state in the TUI (yaks-ce5d): a chip on the tab row saying
//! what the farm has that git does not (uncommitted yaks) and where the farm
//! branch stands against its upstream, plus the two keys that act on it —
//! `W` commits the farm, `U` syncs it (`yaks commit` / `yaks sync`).
//!
//! Nothing here runs by itself: the state is read from local git only (never a
//! fetch) when the farm is loaded, reloaded or acted on, and commit/sync run
//! only on the key. `use super::*` inherits `tui.rs`'s scope.

use super::*;
use crate::status::SyncState;
use crate::{changes, commit, status, sync};

/// What the chip shows: the farm's uncommitted yak count and, for a farm that
/// is a branch, its standing against the upstream. An in-tree or private farm
/// has no `sync` part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FarmSync {
    /// Yaks plus other farm files (config, ...) git does not have yet.
    pub dirty: usize,
    pub sync: Option<SyncState>,
}

impl FarmSync {
    /// Read the farm at `root` (local git only). `None` for a private farm
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
    }
}

/// Collapse a message to one line (newlines become spaces).
fn one_line(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}
