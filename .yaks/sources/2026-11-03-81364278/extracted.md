# Summary of Clint's Haks implementation thread (Indent)

Thread: https://app.indent.com/c/01a1a14b-50ca-74e8-af55-9bfc62e4527c (37 turns, 2026-11-03 12:03 to 19:20 UTC).

Produced 2026-11-03 by a read-only Indent sandbox agent (session 01a1a3a7-fcfe-7b94-927b-55ca800e2370) that downloaded the thread and read its `conversation.md`. It did not read `events.jsonl`, so tool calls and subagent internals are not covered, and screenshots were not viewed. A model's summary of a transcript, not the transcript.

Agent's notes: Downloaded the session with download_session and read all 916 lines of conversation.md (37 turns, 2026-11-03 12:03 to 19:20 UTC). I did not read events.jsonl, so I have not seen tool calls or the internals of the subagents Haks launched. Everything above is from the conversation text. Attachments (screenshots) were not viewed. The session is read-only; I sent it nothing and took none of the actions it describes.

---

## Haks session: design history (2026-11-03, 12:03 to 19:20 UTC)

Participants were Clint Ayres, who sent every message, and Joel Webber, who sent none. The session is a continuation of an earlier Haks session (`01a1a0bb…`).

**What Haks is.** It is a Go CLI plus a local web board, built as an agent layer over "yaks". Yaks is Joel's task tracker. A farm is a git branch of markdown files, and coordination happens through notes. Haks reads yaks only through `--json` and writes only through yaks commands. It derives one snapshot of streams, inbox, reviews and agents, and stores no state of its own.

### Phase 0: restore lost work (12:03-12:33)
- **Clint asked:** "continue Haks". The old session's sandbox was gone, so none of its work existed. The checkout was at `f92e662`.
- **Agent proposed:** two parallel agents in Clint's local checkout.
  - Backend (Go): run registry, `haks stop`/`resume`/`runs`, opt-in reviews with a daily budget, `model` config, and serving the board from disk so chat can rebuild it.
  - Board (web): plain-words Inbox, Runs panel, layout fixes, and a "Review this PR" button.
  - It set `review.auto: false` first, because Clint's running server would otherwise start reviews on its own. The stated reason was spend control.
- **Clint:** "you now have access to local haks make all the fixes and restart the app locally". The agent then built and deployed locally.
- **Result at 12:33:** reviews became opt-in, with a budget of $10 per day. The agent flagged what was unchecked:
  - Slaughter and Dismiss had only been tried against a mock or the demo server.
  - `TestJoinAndBrief` was failing (a macOS `/var` vs `/private/var` path).
  - A copied binary was killed until it was codesigned.

### Phase 1: the verbs for ending work (12:14-12:44)
- **Clint (12:14):** he was marking tasks "shorn" just to stop an agent that was asking a question. He asked, "what is my process here to say - done and stop working?"
- **Agent:** "Don't shear it." It explained that `yaks answer --done` clears the question without handing the yak back, and proposed a Dismiss button.
- **Clint (12:18), a pivot to a simpler ask:** "I just need a way to click on an agent and say - slaughter." The agent added Slaughter on every card, with "Slaughter with children" if child yaks are open. Shear stays, but only for finished work.
- **Bug (12:36):** Clint said "I don't see a slaughter?". Slaughter had been hidden on shorn yaks. That was a bug, and it was fixed so Slaughter shows on everything except dead yaks.
- **Commit/push:** Clint asked "commit and push?". It went to a branch, `clint/runs-control` (`500772a`), and no PR was opened. The repo rule is no AI attribution.
- **Clint (12:49):** "no pr just main line push".
- **Cleanup (12:47):** Clint asked to remove "clint" from the README and add a simple get-started. It went to `main` as `2c0f051`. A PR had meanwhile been merged on the remote, so the agent rebased on top of it.
- **Test fixed:** the `TestJoinAndBrief` path failure was fixed.

### Phase 2: sharing with Joel (12:26-13:34)
- **Clint** twice asked for a summary to send to Joel.
- **Agent:** it first gave a short version, then a longer "problem, design, use cases, not done" writeup.
- **Agent restraint:** it declined to claim what Joel might want changed upstream, because "Nothing in the repo says Haks needs a yak change".
- **Clint (13:10):** he asked for learnings from Joel's separate yaks/surveyor session. The agent read most of it and reported seven points:
  - Shorn parents keep open children.
  - A question on a shorn yak gets lost. Joel's fix was to file a new `review:` yak.
  - Joel couldn't follow raw output.
  - Evidence should be recorded at shear time.
  - Agents merged on their own lean and asked later.
  - Binary version drift in yaks.
  - Side-tasks belong in a sub-thread.
- **Agent offered:** to add items 1, 2 and 4 to the skill. There is no record in the thread that this was done.

### Phase 3: usability fixes and review features (13:34-15:00)
- **Clint (13:34):** a question said "asking clint ... review.md (on this yak)", but he couldn't see the yak. The agent fixed the cause: `review.md` and yak ids became links, and links were moved above the question text. It admitted it didn't know which view he meant.
- **Clint (13:40):** "can we use side panels instead of modals, the modals are killing me". It was done in `976ccd0`.
- **Clint (13:42), configurable review approach:** "my default position is, if there is a finding to fix we'll fix it". He also asked for "a 'reviews' panel like indent does for the changes on a branch". The agent added:
  - `review.approach` set to `fix`, `comment` or `ask`, with `fix` as Clint's default, plus `review.instructions` and per-repo overrides.
  - A Changes panel showing the PR diff, with finding annotations and a Fixes tab fed by `fixes.patch`.
  - The decision that the reviewer writes fixes into a separate worktree and "asks you before pushing".
- **Confusion and reversal:**
  - Clint: "what am I looking at?" and "I don't see a 'fixes tab' anywhere?". The agent made the tab always visible.
  - Clint (14:15): "ok i'm so confused now … I asked to make changes together and now I have no idea what's going on". The board had shown first-pass buttons because it recognised only options starting with "post" or "dismiss". "Make changes together" was misread by the reviewer, whose run ended with nothing posted.
  - The agent changed the answer box to show the options the agent actually asked. It did not click anything for Clint.
- **Clint had to unblock the agent:** "direnv allow from FS_HOME iiuc". The agent ran it and `go test`. One test the reviewer had added (`TestResolveSelectorBounds/iframes/...`) failed because the reviewer had guessed how frames work.
- **Clint (14:33-14:43), a batch of requests, each met by an agent change:**
  - "stop working on this forever" became "Skip this PR". A slaughtered review yak had been recreated by the 60-second GitHub sync, so dead review yaks now stay suppressed.
  - Review filters by author and repo.
  - A useless "Question page" link was removed.
  - A rule allowing push and approve on someone else's branch, only on an explicit answer.
  - Stale installed skills, with `haks serve` now re-syncing skills on start.
  - A review stepper (Not started, Reviewing, Your call, Posting, Done).
  - Filters cut to Author and Repo, because Clint said they were "too busy".
- **Agent warning:** the fixes patch was uncompiled and had a failing test, so don't push yet.
- **Clint (15:05):** "update the review.md with a new instruction: first help get the pr green". The agent edited `prompts/review.md`, which it assumed was what he meant. It also widened the reviewer's command allowlist to `go`, `direnv`, `make` and `npm`, flagged as "a real change in what a review can do", and offered to revert it.

### Phase 4: the design principle (15:13-16:15)
- **Clint (15:13), the key design rule:** "if everything is a yak I can run the review anywhere and have it stay in sync. that's the design philosophy to follow. yaks and haks as tools for agent swarms getting real work done."
- **Agent:** it wrote this into `AGENTS.md`. The test is whether another machine could continue the work from the farm alone. It listed places where Haks violated the rule:
  - Run state lives in a local directory.
  - Run details are not on the yak.
  - "Make changes together" can leave edits only in `/tmp`.
- **Agent proposed:** start and end notes on every run.
- **Clint (15:37):** "reviews need to be ordered … if they are in stack, then show me that". The agent ordered reviews oldest request first and kept stacks together bottom-up. It found two real stacks, and listed what it misses (same-base siblings and non-reviewed bases). Chips and header clipping were then fixed.

### Phase 5: the lego-brick plan and its build (16:15-18:25)
- **Clint (16:15):** he asked for a more "lego brick" design.
  - Software factory as a team yak farm, plus a personal farm.
  - Every assigned PR becomes a yak.
  - Context from earlier yaks is consulted.
  - Reviews open in the UI or CLI with an agent tied to the session.
  - The UI is a thin slice of what the CLI already shows.
  - No coordination beyond what yaks do today.
- **Agent's plan** (`docs/design.md`, uncommitted at the time):
  - Farms with personal, team and watch roles.
  - One yak per PR, keyed by URL, with a `reviewer:<login>` tag.
  - A review state machine written as notes, with heartbeats and takeover after silence.
  - `haks context`.
  - Review sessions.
  - Logic moved from TypeScript to Go.
  - Idempotent verbs and append-only notes.
  - Build order of six steps.
- **Agent's asks for Joel**, stated as unconfirmed: create-by-key, cross-farm references, heartbeat note kinds, and queries by note metadata. Until then, a `haks reconcile` step merges duplicates.
- **Wrong-thread detour (16:55-16:57):** Clint pasted a Surveyor service proto asking about "single job interface" pushback. The agent critiqued it at length, saying it was not sure the plan covered all use cases, and that Run, Decide and Publish are different things. Clint then said "oops wrong thread ignore previous post", and the agent changed nothing.
- **Clint (16:59):** "implement the design". The agent committed everything (`7d25a08`), then ran three parallel agents in git worktrees under `~/src/haks-wt/`:
  - `review-model`, covering steps 1 and 2.
  - `context`, covering step 3.
  - `farms`, covering steps 4 and 6 in simulation.
- **Defaults the agent picked itself, for open questions:** team farm is whatever repo is configured, evidence stays shared, and a run is stale after 5 minutes.
- **Clint (17:35):** he needed a personal to-do herd, using a pasted meeting note about validating search on fs-mcp. The agent created `me-2a5c`, a `me` herd of kind `todo`, and later added `haks todo`.
- **Steps 1 to 4 merged** (`9b3fb25`): one review model in Go, `haks reviews`, a generic verb endpoint, start and end notes, `haks context`, multi-farm registry, reconcile, and a simulated two-machine test.
- **Step 5 merged** (`0fa7a62`): `haks review open` and a docked Session panel, with the session recorded on the yak and "Save to yak" writing `fixes.patch`. A launchd gotcha was found: killing `haks serve` by hand let launchd restart the old binary, so deploy is now `launchctl kickstart -k`.

### Phase 6: wrap-up (18:51-19:20)
- **Clint (18:51):** "is it all running locally? what's my new workflow". The agent gave an eight-step workflow. While checking, it found `haks todo --help` silently created a yak, and fixed that in `bb6f2f1`.
- **Clint (19:00):** "push and update readme". Done in `8fbe3ec`.
- **Clint (19:02):** he asked for a lifecycle summary. The agent gave one covering yaks, Haks orchestration, the inbox, and reviews from intake through Done.

### What the human asked for vs. what the agent proposed
- **Clint-driven:**
  - Slaughter and Skip.
  - Side panels.
  - Fix-by-default review approach.
  - A reviews panel.
  - Filters.
  - Get-the-PR-green prompt.
  - Stack ordering.
  - Personal to-do herd.
  - The "everything is a yak" principle.
  - The lego-brick and team-farm vision.
  - Pushing straight to `main`.
  - No AI attribution.
- **Agent-driven:**
  - The two-agent split.
  - The Dismiss button.
  - The `review.daily_budget_usd` cap.
  - The widened command allowlist.
  - Skill auto-sync on start.
  - Notes written on the yak for every run.
  - Context assembly.
  - The state-machine and heartbeat model.
  - Reconcile.
  - Step ordering and defaults.
  - The wish-list for Joel's yaks.

### Pivots and reversals
- Dismiss-only, then a Slaughter-everywhere button, then "Skip this PR" for reviews.
- A Fixes tab that was hidden, then always visible.
- Five review filters cut to two.
- A plain-text question, then link-first rows.
- Modals replaced by docks.
- A "push only to your own PRs" rule relaxed to push on an explicit answer.

### Unbuilt or unverified at the end
- **No real team farm.** The team path was proven only with a simulated two-machine test against a local bare repo.
- **Agent's own known gaps:**
  - Creating a new team-farm branch is racy.
  - Create-by-key is unavailable in yaks.
  - Cross-farm references, heartbeat and note-metadata queries remain asks for Joel.
- **Never run for real:**
  - A fix-mode review on a real PR.
  - A Session panel on a real review. It was checked only on the demo server.
  - Skip on a live yak, and Slaughter and Dismiss on the real backend.
- **Minor gaps:**
  - Stacking misses same-base siblings and bases that aren't open reviews.
  - The Inbox is not ordered or filtered like Reviews.
  - Resuming a Claude session on another machine needs its transcript.
  - Codex and Cursor sessions resume only on the originating machine.
  - The launchd agent is not created by `haks install`.
- **Uncertain upstream state:** whether the yaks `doctor` checks Joel's agent proposed ever landed, and whether `shorn` refuses with open children.
- **Open from Clint's earlier asks:** the offer to add Joel-session learnings 1, 2 and 4 to `skills/haks/SKILL.md`. The thread shows no approval or build.
- **Open questions** put to Clint were asked once and not clearly answered: where the team farm lives, whether personal drafts are shared, and the heartbeat interval.
