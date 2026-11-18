---
type: source
title: DeltaDB product page (delta.dev, Zed Industries)
description: Saved copy of https://delta.dev/deltadb
media_type: text/markdown
size: 8397
sha256: e5d7f4769c2ff1c427ec897a20af90feaf88f3836092914c3f6bb43b5d48b11a
sources:
  - id: web
    resource: https://delta.dev/deltadb
    host: joel
    last_modified: 2026-11-16T21:32:54Z
    retrieved: 2026-11-16T21:33:19Z
generated:
  by: delta:ksQQM-tlBqyPR_q2SQ147vmMIZLOAGl2rZIBxBCA
  at: 2026-11-16T21:33:19Z
---

Skip to content

[](/)

[](/)

Now in Beta[Home](/)[Download](/download)[CLI](/cli)[DeltaDB](/deltadb)[Docs](/docs/getting-started)[Roadmap](/roadmap)[Pricing](/pricing)[Blog](/blog)

DeltaDB

# Keep the context behind every change

The version control system that records every edit with the conversation behind it.

[Get started`G`](/docs/concepts/delta-and-git#what-is-deltadb)[Download Delta`D`](/download)[Install the CLI`C`](/cli)

## Git keeps the commits. DeltaDB keeps everything in between.

Git records a snapshot each time you commit. DeltaDB records work as a stream of fine-grained deltas as it happens, alongside the conversation that produced it. Commits, branches, and remotes stay in Git, so you push and open pull requests as usual.

[How Delta and Git work together](/docs/concepts/delta-and-git)

Git keeps the commits; DeltaDB keeps everything in between. Three Git commits on a line. Around them, the reasoning behind the work surfaces and falls away: the ask, dead ends, rationale, tradeoffs, test results, pushback. Then a DeltaDB timeline appears beneath the same commits, and that reasoning lands in it as deltas, alongside the code edits. Finally, links show a worktree starting from the first commit, and its work going back to Git as a commit and a pull request.GitA snapshot at each commit4a7c2e19d03b5fc61e8a4the askdead endsrationaletradeoffstest resultspushbackLost between commitsDeltaDBEvery step in betweenthe askdead endsrationaletradeoffstest resultspushbackKept: every message and edit, with the codeWorktreeCommitPull request↻ Replay

## How DeltaDB works

DeltaDB records each change as it happens and keeps every copy in sync.

  1. 1

### Every edit becomes a delta

DeltaDB records each edit, yours or an agent's, as a delta with a stable identity, its author, and the version it builds on. There's nothing to stage or commit.

Each edit, from you or an agent, becomes one delta in the history123456Historyfn retry(attempts: u32) {You for n in 0..attempts {Agent if try_once().is_ok() { return; }Agent sleep(backoff(n)); }Agent}You▶Hover to playTap to play

  2. 2

### The conversation shares the history

Messages, comments, and file tree changes are deltas too, so each message sits in the same history as the edit it produced.

Messages, code edits, comments, and file changes all land in one historyMessagesCode editsCommentsFile changesOne history▶Hover to playTap to play

  3. 3

### Every copy stays in sync

Every Delta client holds a replica. Deltas sync as you type, and concurrent edits resolve the same way on every machine, in any order.

Every replica converges on the same history. A laptop, a cloud runner, and a browser each receive the same four changes through the sync server, in different orders. Each change takes its agreed place, so all three end with the same history and fingerprint.Sync serverYour laptopfingerprint 9c41 ✓Cloud runnerfingerprint 9c41 ✓Teammate's browserfingerprint 9c41 ✓▶Hover to playTap to play

  4. 4

### Files appear where you need them

Agents and browsers use the replicated files directly. On your machine, Delta mounts them into a checkout for your terminal, builds, and editor. The checkout is temporary, and the work stays in DeltaDB.

[Read about Delta worktrees →](/docs/concepts/worktrees)

One Delta worktree, used from a laptop checkout, an agent on a cloud runner, and a browserWorktreein DeltaDBCheckout on your laptopTemporaryAgent on a cloud runnerBrowser▶Hover to playTap to play




## What a continuous history makes possible

### [Rewind to any message ](/docs/agents/threads#revert-a-conversation)

Revert a thread to an earlier message, and DeltaDB restores its files to match. You don't need a commit.

### [Share work before you commit it ](/docs/collaboration/collaborate-thread)

Teammates can join while the agent is still working, read the conversation, and comment on changes before anyone commits.

### Start new work without the bookkeeping

Each thread gets an isolated worktree, and you can fork one to try another approach. There are no directories or branches to clean up.

### Know where each change came from

Each delta records whether a person or an agent made it. From the code, you can get back to the conversation behind it.

## Under the hood

DeltaDB is built on CRDTs, loosely based on diamond-types and adapted to work with Git.

[Read the deep dive](/blog/future-of-version-control)[Watch the talk](https://www.youtube.com/watch?v=yif2rday2bw)

History graphConvergenceFragment indexStructural sharing

### Each delta records what it was based on

A delta carries the text it inserts, the ranges it replaces, and the version it builds on: earlier deltas, or a Git blob for a file's starting text. Without a shared clock, order comes from the causal graph.

Each batch points to the versions it builds on. Five batches of changes from actors A and B. Two are written concurrently on the same parent; the next batch names both as its parents. Each batch's Lamport timestamp is higher than its parents'.(1, A)(2, A)(2, B)(3, A)(4, B)parents: (2, A) and (2, B)1234Lamport timestamp →
    
    
    struct HistoryDeltaBatch {
        id: DeltaBatchId,
        parent: HistoryVersion,
        payloads: Vec<HistoryDelta>,
    }
    
    struct DeltaId {
        timestamp: u64,
        actor_id: ActorId,
    }

### Concurrent edits converge

Start with TIVITY. You insert REA at the start while I insert C at the same spot. Both machines break the tie by actor ID, so we both get REACTIVITY, whichever edit arrives first.

Concurrent inserts converge on REACTIVITY. Both replicas start with TIVITY. One person inserts REA at the start while another inserts C at the same spot. Replica 1 receives REA first, replica 2 receives C first, and both end with REACTIVITY. REA is (4, B), C is (4, A). Same timestamp, so the higher actor goes first.Replica 1TIVITYREA arrivesREATIVITYC arrivesREACTIVITYReplica 2TIVITYC arrivesCTIVITYREA arrivesREACTIVITY= same resultREA is (4, B), C is (4, A). Same timestamp, so the higher actor goes first.

### The fragment index

Each file is stored as text plus a B-tree recording which delta created each character. When an edit based on an older version arrives, DeltaDB looks up where that position is now.

The fragment index behind REACTIVITY. The text REACTIVITY sits above a tree of four fragments: REA, C, a deleted S that is kept but invisible, and TIVITY from the Git blob. Each node stores how much visible text is beneath it, so finding character 4 walks from the root to the right branch, past the deleted fragment, to the T in TIVITY.REACTIVITY1046REA · 3from (4, B)C · 1from (4, A)S · 0deletedTIVITY · 6Git blobFind character 4

### Structural sharing

Versions are immutable trees. A new version adds only the nodes on the path to the change and shares the rest, so storage grows with the size of the edits instead of the number of edits squared.

A new snapshot copies only the path to the change. Version 1 is a tree with two branches and four leaves. Editing the last leaf creates version 2 with three new nodes: the edited leaf, its parent, and a new root. Version 2 shares the other branch and leaf with version 1.v1v2editv2 adds 3 nodes and shares 4 with v1

## Frequently asked questions

Does DeltaDB replace Git?

No. Your project still needs a Git remote, commits stay in Git, and you push and open pull requests as usual. DeltaDB records the work between commits.

What does DeltaDB record?

Messages, comments, file tree changes, and edits to files Git tracks or doesn't ignore. Ignored files, like build output and local .env files, stay on the machine that created them.

How do I use DeltaDB?

DeltaDB comes with Delta. Download the app, or open a shared thread in your browser.

## Read more about DeltaDB

  * ### [Software Is Made Between Commits ](https://zed.dev/blog/introducing-deltadb)

Jun 11, 2026

  * ### [DeltaDB: The Future of Version Control](/blog/future-of-version-control)

Oct 15, 2026

  * ### [Escaping the tyranny of worktrees](/blog/escaping-the-tyranny-of-worktrees)

Oct 6, 2026

  * ### [Xanadu Was Waiting for Agents ](https://zed.dev/blog/agentic-xanadu)

Sep 1, 2026




[Download Delta Now`D`](/download)[Open Delta`D`](/threads)

Zed Industries (C) 2026

[Terms](https://zed.dev/terms)[Privacy](https://zed.dev/privacy)[Brand](/brand)Cookies
