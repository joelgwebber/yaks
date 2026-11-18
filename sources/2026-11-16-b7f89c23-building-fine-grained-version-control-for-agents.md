---
type: source
title: Building fine-grained version control for agents with Rust and Durable Objects (Scandurra, Delta blog, 2026-10-29)
description: Saved copy of https://delta.dev/blog/fine-grained-version-control-for-agents
media_type: text/markdown
size: 29873
sha256: b7f89c2310ab06a6e761a4ca3d984832fae8aa3c9984fa9ba3d034c91b7645f9
sources:
  - id: web
    resource: https://delta.dev/blog/fine-grained-version-control-for-agents
    host: joel
    last_modified: 2026-11-16T21:32:54Z
    retrieved: 2026-11-16T21:33:20Z
generated:
  by: delta:ksQQM-tlBqyPR_q2SQ147vmMIZLOAGl2rZIBxBCA
  at: 2026-11-16T21:33:20Z
---

Skip to content

[](/)

[](/)

Now in Beta[Home](/)[Download](/download)[CLI](/cli)[DeltaDB](/deltadb)[Docs](/docs/getting-started)[Roadmap](/roadmap)[Pricing](/pricing)[Blog](/blog)

Blog

# Building fine-grained version control for agents with Rust and Durable Objects

Antonio ScandurraOctober 29, 2026

Back in 2021, Nathan, Max and I founded Zed Industries with two goals: build a [very fast code editor](https://zed.dev/blog/videogame), and [fundamentally change how teams collaborate on software](https://zed.dev/blog/introducing-deltadb). Millions of people now use Zed, and [I’m proud of what we built](https://zed.dev/blog/zed-1-0).

For the past year, we’ve been working on the second goal.

I believe that writing software is the process of continuously clarifying a problem, in your head or with your teammates, until the code becomes inevitable. Clear thinking leads to clear code; messy thinking leads to messy code. Agents haven’t changed that. We’re just producing code way faster, and more and more of the clarifying happens in conversations with agents. In the best case, the context behind that code is trapped inside the agent harness of the week. In the worst case, it’s lost forever in Slack or compressed into a commit message that the agent wrote and that you didn’t read.

[DeltaDB](https://delta.dev/deltadb) is our answer. It’s a version control system for agent context. You can point at a piece of code, recover the reasoning behind it, and ask the agent or human who wrote it why it looks the way it does.

In this post, we’ll dive into how we built a system that streams every change from humans and agents in real time, merges their edits into one monotonically growing causal graph, and keeps every edit linked to the conversation that produced it.

As always at Zed Industries, we started with an empty repo.

## Architecture

DeltaDB is a collection of [CRDTs](https://zed.dev/blog/crdts), all stored locally in SQLite and linked together in a directed acyclic graph (DAG). Every node in the graph is called a _delta_ , and it’s the smallest unit of change that can be represented. Combined, this lets us model the essential state of an [agent thread](/docs/agents/threads): its conversation, its files, and everything around them.

Using CRDTs lets clients make progress on their own and stay resilient to network failures or partitions. We eventually synchronize all the state to a server, but we don’t need clients to be constantly connected: changes are applied locally right away and synchronized when possible. This makes it straightforward to implement the entire spectrum of collaboration, from handing off a thread asynchronously to [collaborating in real time with your teammates](/docs/collaboration/collaborate-thread).

The DAG enables representing every possible intermediate state in the system. This has a few nice properties. For example, we can easily scrub back and forth through the history of a thread and its worktree. It also lets us express the entire state of the system with a very compact version: the deltas at the tips of the graph (also called a _frontier_). Its size grows with how many branches are open at the same time, not with how many writers there have ever been. Our design closely resembles the work pioneered by Gentle and Kleppmann in the [Eg-walker paper](https://arxiv.org/abs/2409.14252).

The primary downside of using a DAG is that we need to navigate the graph to detect whether a delta already includes the impact of some other delta. If we used a [version vector](https://en.wikipedia.org/wiki/Version_vector), we could answer that by comparing counters. Even though that would be cheaper than walking the parent links, a version vector assumes that actors are few and long-lived. In a thread, however, people and agents come and go all the time, and once an actor makes some changes, the vector can’t drop the counter. So the vector can only ever grow, even after the actor has left. [Dotted version vectors](https://arxiv.org/abs/1011.5808) have similar issues, as illustrated in the animation below.

Your client writes deltas 1, 2, and 3 on the main timeline, each numbered by the client's single clock. With one timeline, a version vector, a dotted version vector, and a DAG frontier all describe main correctly. Then a subagent forks a timeline on the same client, so its deltas 5 and 7 take numbers between main's 4, 6, and 8. A version vector describes main as you: 8, which covers every delta of yours up to 8, including the subagent's 5 and 7. A dotted version vector keeps 8 as its dot, but its context is still counters, so it must say you: 4 plus an exception for 6, and every further interleaving adds another exception. The DAG frontier for main is just 8, and walking parent links from 8 visits 6, 4, 3, 2, and 1, exactly main. Next, three Cloud Runner sandboxes fork from 8. Each opens its own database, so each is a new writer. Merging their work as delta 9 collapses four tips into one, but both vectors gain a counter for every sandbox. After the sandboxes shut down and you write delta 10, the vectors still carry those counters, while the frontier is the single delta 10. The frontier's price is that causal questions become walks over parent links, which DeltaDB stores anyway.your clock10MainSubagentsandbox A, gonesandbox B, gonesandbox C, goneA1A1B1B1C1C1A2A2B2B2C2C21122334455667788991010tap any deltaMain's versionVersion vectoryou 10A 2B 2C 2also claims 5, 7Dotted vectordot 10you 4+6 +8 +9A 2B 2C 23 exceptionsDAG frontier910exact

Replay

8 / 8

Ancestor queries are a fine tradeoff for us because we need to store every version anyway. They’re also fast in practice, because we store a delta’s ancestry separately from its payload. The ancestry data is tiny and stored in causal order, so these queries can leverage a lot of locality.

Making each delta a node in the DAG also lets us decouple its network representation from how the CRDT expresses its impact internally. For example, this is how we define text edits:
    
    
    struct EditTextDelta {
        parent: Version,
        offset: usize,
        text: String,
    }

Note how `parent` and `offset` are sufficient to uniquely identify a position in a buffer, even in the presence of concurrent edits. Most of the time, the buffer is already at `parent`, so the edit applies directly. When it isn’t (e.g., if a concurrent edit happened in the meantime), the client _reifies_ the version at `parent`, reconstructing the buffer as it was at that point. It then translates the offset into an internal representation (fragment IDs in our case, similar to how we do it in Zed) and locates that fragment ID in the buffer at the version we’re applying the edit to.

This makes deltas very cheap on the wire and it also lets us change the internal fragment representation and page out unused fragments.

## The mighty B+ tree

We store all of our data in SQLite. This data can roughly be partitioned into two categories: relational and versioned.

Relational data is pretty straightforward. For example, on the client we store all the threads that you’ve created or opened. This goes in a traditional table and there’s nothing special about it.

Versioned data is where it gets interesting. The whole system hinges on the ability to quickly reify any version. As we saw above, applying a concurrent delta depends on it, and we also don’t want to wait a long time when scrubbing through history or mounting the worktree of an old version of the thread on the file system.

To make that fast, we cache a snapshot of the whole state for recently used nodes in the DAG (plus a few more important versions that we pin). Then, starting from the version we want, we walk back through the DAG to the closest ancestor that has a cached snapshot and apply every delta in between.

Storing a full copy of the CRDTs for every cached version would be prohibitively expensive. Instead, we express the whole state as copy-on-write B+ trees. This means we can reuse most of the nodes from a previous snapshot and only pay for the paths that the replayed deltas touch.

A DAG of deltas. The history so far is folded into delta P. Your timeline writes deltas 1 to 4, 6, and 8, a subagent forks after 3 and writes 5 and 7, and delta 9 merges both before 10 to 12 follow. P's snapshot is pinned, and versions 2 and 4 have cached snapshots. To reify version 9, a walk goes back from 9 along parent links on both branches. It reaches delta 4, which has a snapshot that covers everything 4 has seen, so the walk through the subagent's branch stops at delta 3. That leaves deltas 5, 6, 7, 8, and 9 to replay. The DAG moves up, and version 4's B+ tree unfolds out of its snapshot: a root, three internal nodes, and nine leaves of text fragments. Its root is cloned into version 9's root, which shares every node with version 4. Each delta then travels from the new root down to its leaf. Delta 5 clones the internal node and the leaf on its path, and the originals stay behind for version 4. Delta 6 lands in the same leaf, finds its path already cloned, and writes in place. Delta 7 clones a path in the next subtree, and delta 8 writes into it in place. Delta 9 reuses the internal node delta 7 cloned and clones only its own leaf. Read left to right, the leaves hold deltas 1 to 9 in order. Version 9 ends up with six nodes of its own and shares the other seven with version 4. Its snapshot joins the cache, and version 2's, the least recently used, falls out. Tapping any delta reifies its version the same way, from whichever cached snapshot is closest.pinned snapshotcached snapshot12345678910111212345678949tap any delta

Replay

8 / 8

Our B+ tree is a variation of [Zed’s SumTree](https://zed.dev/blog/zed-decoded-rope-sumtree). The `SumTree` in Zed is an append-only B+ tree whose internal nodes summarize their children. To edit a `SumTree`, you have to slice a prefix of the tree up until the insertion point, push new items and skip the ones you want to delete, and finally append the suffix, which rebuilds every node along the cut. In contrast, DeltaDB’s B+ tree edits in place through a mutable cursor.

It’s a lot faster because we perform way fewer adjustments to the tree (see the animation below). In-place mutation also lets us precisely track which nodes were allocated, changed, or released. On commit, new nodes become inserts, changed nodes get rewritten in their existing row, and released nodes only get their refcount decremented. All nodes get stored in a generic `nodes` table.

Two copies of the same B+ tree hold the text "Hello, wld!" in leaves of up to four characters, and each internal node counts the characters under each of its children. Snapshot v1 is a clone of both, so it shares every node. Typing "o" at offset 8, each tree seeks with the counts: the root skips the first 7 characters, and the cursor lands in the third leaf. The tree on top edits the way Zed's SumTree does. It slices the tree at the offset, so each of the three nodes on the cut splits into a prefix half and a suffix half. It pushes "o" onto the prefix and appends the suffix, and when the halves rejoin, the suffix's three nodes are thrown away. That is six nodes built and three thrown away for one character. The tree below edits the way DeltaDB's mutable cursor does. It walks the same three nodes, and because v1 still shares each one, it copies the node before writing to it. The originals stay behind for v1, which still reads "Hello, wld!", and the nodes the edit didn't touch stay shared. Typing "r" next, the rebuild builds six nodes and throws six away again, while the cursor finds its whole path already unshared and changes the three nodes in place, building nothing.textHello, world!v1 Hello, wld!Zed SumTreeslice, push, appendbuilt12thrown away9+6+67422wl344276Hello,␣world!v1DeltaDB B+ treemutable cursorbuilt3thrown away0+0+07422wl344276Hello,␣world!v1

Type "b"Take snapshot

Replay

5 / 5

## Timelines and synchronization

Clients can track as many frontiers as they want in the DAG using _timelines_. For example, when kicking off a subagent, we create a new timeline parked at the parent thread’s frontier and start making changes in parallel. All timelines share the same DAG so they can be merged back together at any later point in time. None of this touches Git and, crucially, doesn’t require committing for cross-agent coordination.

Timelines are also used for synchronization among clients. When a thread is shared, the client establishes a WebSocket connection to the server and subscribes to the timelines it is interested in. For every timeline, we perform a synchronization step to exchange all the deltas each side has missed since the last connection (or everything if it’s the first time we’re connecting to the server). This is accomplished by walking the DAG and sending a logarithmically spaced list of versions to find a recent common ancestor. Every new delta is then sent incrementally over the WebSocket, saved by the server, and pushed to every other client subscribed to that timeline.

Three rows show the same timeline on your client, the server, and your teammate's client, with a WebSocket between each client and the server. You start offline: your three edits stay on your machine, while your teammate's two edits reach the server over their open socket and are saved there. When you reconnect, your client opens a WebSocket for the history and subscribes to the timeline. It then sends five ancestors from its timeline, dense near the tip and sparse further back. The server checks them newest first, doesn't know your three recent edits, and stops at the first shared version, the common ancestor. It replies with a list of every delta after that ancestor, without contents. Your client already has the first one, so it fetches only your teammate's two edits over the same socket. Then it uploads its own three edits; the server saves them and pushes them to your teammate. Everyone now has the same history, and both sockets stay open. From then on, a new edit is saved where it's written, sent over the socket, saved by the server, and pushed to the other subscribed client, with no probes and no fetches. Your teammate's next edit travels back to you the same way. Reconnecting after time offline, or missing an update, runs the catch-up round again.YouonlineServerDurable ObjectTeammateonlineWebSocketWebSocket

Edit as youEdit as teammateGo offline

Replay

9 / 9

Reconnections are handled the same way, which makes it straightforward to reason about the correctness of delta delivery.

## One Durable Object per thread

We envision a world where every human creates and interacts with more and more agents. In that future, it’s hard to imagine keeping all the thread state in a single server.

In DeltaDB, every thread gets synchronized to its own [Cloudflare Durable Object](https://developers.cloudflare.com/durable-objects/). The object comes to life the first time a client connects to it, and creating a thread costs nothing on the server. Each client establishes one WebSocket connection for every thread it’s interested in.

Other than authentication (which only happens when establishing the connection), this is essentially a share-nothing architecture. This means we scale horizontally with the number of threads, as far as Cloudflare can serve the traffic.

You create a thread, and at first it exists only on your machine. When your client connects, the thread's Durable Object wakes up. It is named after the thread's history, keeps that history in its own SQLite database, and receives the deltas you wrote offline. Your teammate, a local agent, and a Cloud Runner sandbox open the same thread, and each connects one WebSocket to the same object. When the agent writes a delta, the object saves it and then pushes it to everyone else connected to the thread, but not back to the agent. Zooming out, every other thread works the same way, each with its own object, database, and clients, sharing nothing. When a swarm of agents piles onto one thread, that thread's object gets busy and deltas queue up there, while every other thread keeps its own pace. New thread adds another thread with its own object, and Swarm a thread floods the newest calm thread with agents.10 threads · 10 Durable Objects · 36 WebSocketsDurable ObjectSQLite9 deltasYouTeammateAgentCloud RunnerDurable ObjectSQLite4 deltasAgentTeammateTeammateAgentDurable ObjectSQLite5 deltasTeammateTeammateCloud RunnerDurable ObjectSQLite28 deltasbusyAgentCloud RunnerAgentAgentAgentAgentAgentAgentAgentAgentDurable ObjectSQLite4 deltasTeammateAgentDurable ObjectSQLite5 deltasTeammateCloud RunnerDurable ObjectSQLite4 deltasTeammateAgentTeammateCloud RunnerDurable ObjectSQLite5 deltasAgentAgentDurable ObjectSQLite4 deltasTeammateAgentDurable ObjectSQLite5 deltasAgentTeammateTeammate

New threadSwarm a thread

Replay

6 / 6

This lets us scale across threads, but multiple agents and humans can also issue many simultaneous writes within a single thread. We put a lot of work into making that efficient and cost-effective in a Durable Object, and we will discuss how we did it in a future post.

Cloudflare is DeltaDB’s first deployment target. As of today, it runs on our multi-tenant deployment, as well as on single-tenant deployments for customers who want to “bring their own Cloudflare”. However, the whole system has been designed to be portable to other platforms, so long as they provide an object store, a fault-tolerant Postgres database for authentication, and a way to spin up addressable single-threaded processes with their own transactional storage.

## `#[derive(Replicate)]`

In practice, we never need to think about any of the above when interacting with DeltaDB. The whole system is exposed via a higher-level client API with the following primitives:

  * `Register<T>`, a last-writer-wins container.
  * `Map<K, V>`, an add-wins map.
  * `Buffer`, a collaborative piece of text backed by a rope.
  * `Worktree`, a [tree of files layered on top of Git](/docs/concepts/worktrees) that you can mount anywhere on disk. A worktree starts at a commit, and each file is either pinned to its committed blob or carries the deltas written since. Creating a commit produces a delta too, which is what ties our DAG to Git’s.



We have a long-lived `deltadb::Client` struct that can be used to interact with the above primitives:
    
    
    client.create(|cx| {
        let register = cx.create_register(42);
        register.set(43, cx);
    
        let map = cx.create_map();
        map.insert("a", 1, cx);
        map.insert("b", 2, cx);
    
        let buffer = cx.create_buffer("hello, world");
        buffer.edit([(0..5, "goodbye")], cx);
    });

Updating a primitive automatically mints new deltas and adds them to the DAG, which is then synchronized to disk and to the server on the user’s behalf. You can also compose them into more complex structs using the `#[derive(Replicate)]` macro, and that’s indeed what we do to represent an agent thread.
    
    
    #[derive(Clone, Replicate)]
    struct Thread {
        #[field(0)]
        transcript: Buffer,
        #[field(1)]
        name: Register<String>,
        #[field(2)]
        worktree: Worktree,
        #[field(3)]
        comments: Map<u64, Comment>,
    }
    
    #[derive(Clone, Replicate)]
    struct Comment {
        #[field(0)]
        body: Buffer,
        #[field(1)]
        resolved: Register<bool>,
    }

If we peek under the hood, these types are just pointers into a larger, flat CRDT that we call a `Snapshot`. These pointers can be created at compile-time via the `field` attribute, or at runtime by taking the ID of the delta that created it.
    
    
    #[derive(Clone)]
    struct Register<T> { id: RegisterId, _type: PhantomData<T> }
    
    #[derive(Clone)]
    struct Map<K, V> { id: MapId, _type: PhantomData<(K, V)> }
    
    #[derive(Clone)]
    struct Buffer { id: BufferId }
    
    #[derive(Clone)]
    struct Worktree { id: WorktreeId }
    
    #[derive(Clone)]
    struct Snapshot {
        registers: CowBTree<RegisterId, RegisterSnapshot>,
        maps: CowBTree<MapId, MapSnapshot>,
        buffers: CowBTree<BufferId, BufferSnapshot>,
        worktrees: CowBTree<WorktreeId, WorktreeSnapshot>,
    }

The client API doesn’t expose the flat snapshot directly, but it exposes its data through the pointer mechanism described above.

An exploded view of #[derive(Replicate)], driven by code. A code panel at the top runs one line at a time, and below it a Thread struct is drawn with four fields: transcript, name, worktree, and comments. When #[derive(Replicate)] runs, the struct tilts up and a floor appears underneath it: the snapshot, with one flat table each for buffers, registers, worktrees, and maps. Each #[field(n)] line then sends the ID [n] from the code into its field, and a line drops from the field to a new row in its table, holding a default value: an empty transcript and name, an empty worktree, and an empty map. Then the code inside client.create runs. thread.transcript.edit sends delta Δ1 through [0] into the transcript's row, which now reads "You: fix it", and thread.name.set("Fix login") sends Δ2 through [1] into the name's row. cx.create_buffer("Check Safari") is delta Δ3, and it lands directly in the buffers table as a new row whose ID is Δ3, the ID of the delta that created it; the variable body now holds Δ3. cx.create_register(false) does the same as Δ4, held by resolved. thread.comments.insert(1, …) sends Δ5 through [3], and the map gains entry 1, which holds only the IDs Δ3 and Δ4, with arrows to their rows. thread.name.set("Fix redirect") sends Δ6 through [1], and the row changes while the struct still holds [1]. Finally resolved.set(true) sends Δ7 straight from the handle to row Δ4, which flips to true. Buttons append more lines: renaming the thread again, or adding a second comment, which creates two more rows and a second map entry.buffersregistersworktreesmaps[0]"You: fix it"[1]"Fix redirect"[2]empty[3]{1}Δ3"Check Safari"Δ4true1 →Δ3Δ4Threadtranscript[0]name[1]worktree[2]comments[3]10 thread.transcript.edit([(0..0, "You: fix it")], cx);11 thread.name.set("Fix login", cx);12 let body = cx.create_buffer("Check Safari");13 let resolved = cx.create_register(false);14 thread.comments.insert(1, Comment { body, resolved }, cx);15 thread.name.set("Fix redirect", cx);16 resolved.set(true, cx);17});[0][1][2][3]Δ3Δ4

Rename the threadAdd a comment

Replay

6 / 6

## Putting it all together in Delta’s native UI

You can use DeltaDB today via the [CLI](https://delta.dev/cli) or using [plugins](/docs/cli/claude-code) that integrate with third-party agentic harnesses. The CLI already runs in its own process, so interacting with SQLite directly is totally fine there.

But our company’s history is one of vertical integration and fixing problems end-to-end. Controlling the whole stack lets us design the UX we want rather than working around someone else’s idea of how you’re supposed to interact with an agent. That’s why we built [Delta](https://delta.dev/download), our first-party agentic harness. I believe it’s the state of the art for agentic coding, and it shares many of the underpinnings that make Zed as fast as it is.

In a native UI, the situation is very different: you need to render at the display’s refresh rate, so performing expensive operations on the main thread (such as blocking I/O) would blow right past our tight 8 ms frame budget.

The good news is that the API showcased above is actually designed to be interacted with from the main thread. We achieved this by maintaining an `Arc`-allocated version of the `Snapshot` that gets created during the initial load, which is a moment where we can afford an asynchronous pause to load from the database. From then on, the main thread mints deltas synchronously as it manipulates the replicated primitives. Those deltas are then shipped asynchronously to a background thread alongside a clone of the snapshot. Remember that cloning a snapshot is cheap because we only use copy-on-write data structures.

In the background, we keep one dedicated writer thread and between two and four reader threads, depending on the machine’s cores. Once the foreground publishes the deltas it just produced, the writer thread saves them durably in SQLite and applies them to a SQLite-allocated version of the snapshot, whose nodes live in the `nodes` table we described earlier. As soon as that transaction commits, a reader thread loads the new deltas and uploads them to the server, without waiting for the writer.

If remote deltas have arrived in the meantime, we apply them to both the background snapshot and the last foreground snapshot we received from the main thread. The updated `Arc`-allocated snapshot is then shipped back to the main thread.

Note how the foreground never blocks on any of this and is free to keep minting new deltas and mutating its own version of the snapshot. When that happens, we can take the snapshot the background just produced and reapply the concurrent foreground deltas on top.

This system only works because the whole state is represented as one big CRDT, so we can apply deltas in any valid causal order, essentially treating the foreground and background threads as two concurrent clients manipulating the same CRDT.

Four bands show a client's main thread, its writer thread next to SQLite, two reader threads, and the server. The state is a card with a title, which you type, and a note, which your teammate types. You type "Fix". The main thread applies each key to its snapshot at once and queues the delta, and a frame meter under the main thread stays green. The main thread then sends the queued deltas down to the writer with a clone of the snapshot, which is cheap because the clone shares every node with it. The writer saves the deltas to SQLite. As soon as that transaction commits, a reader thread loads them and uploads them to the server, while the writer compares the clone with storage. The clone comes back and takes the snapshot's place, and the old snapshot is freed later, off the main thread. Your teammate types "iOS only". Their deltas reach the server and then your writer, which saves them, but your snapshot doesn't have them yet. The next clone goes down, and the writer finds the eight deltas it lacks and applies them to the clone. Meanwhile you type " login" on the main thread, and the frame meter stays green. The clone comes back with the note but without " login", so the main thread replays those keys onto it and swaps it in. Under the main thread, a list shows the reads the app asks for. Each request leaves its row for a reader thread, and its result comes back to the row, while the main thread carries on. Every save overlaps a read, and a reader serves each at once from the last commit, without the save in progress: an agent reads src/auth.rs while "Fix" is saved, so both readers work at once; a review diffs the thread while your teammate's deltas are saved; and you open an older version while " login" is saved. Afterwards a reader uploads " login". On the main thread replays the same story with no other threads: the clone never leaves, the snapshot freezes while it saves, compares, and applies, typed keys pile up waiting, and the frame meter fills with dropped frames. Flood it has your teammate and an agent type in bursts while you type " redirect", and the agent and a review keep reading: deltas queue up waiting for the writer, both readers stay busy, yet the snapshot keeps up with your typing and no frames drop, unless the work runs on the main thread, where every read runs on the main thread too and the snapshot stalls while it does.Main thread0 frames droppedfreed laterWriter threadidleReader threadsidlereader 1reader 2ServerDurable Objectunsaved deltasSQLitehistoryFixiOS·only·loginFixiOS·only·loginyour teammate's clientnoteiOSonlyold snapshottitlenoteFixloginiOSonlysnapshottitlenoteFixloginiOSonlyreads the app asked foryou open an older version✓ rebuilt

Off the main threadOn the main thread

Flood it

Replay

6 / 6

## Conclusion

This post mostly covered how DeltaDB works on the client. There are many more things we’ve been working on, and some of them are about to ship.

For example, we can leverage the fine-grained history we capture to track a piece of code across files and versions, all the way back to the conversation that produced it. If you’re familiar with [anchors in Zed](https://zed.dev/blog/crdts), the same idea lets us assign an identity to every character and follow it as code evolves. Think of it like a supercharged, per-character `git blame`.

A lot of work also went into the server side to ensure clients can load a thread without replaying millions of deltas before seeing pixels on screen. Similarly, we made sure a single Durable Object can handle bursts of concurrent writes. Both will be the subject of a future post.

On this page

  1. Architecture
  2. The mighty B+ tree
  3. Timelines and synchronization
  4. One Durable Object per thread
  5. #[derive(Replicate)]
  6. Putting it all together in Delta’s native UI
  7. Conclusion



## More from the blog

  * ### [DeltaDB: The Future of Version Control](/blog/future-of-version-control)

Oct 15, 2026

  * ### [Replace PRs with Delta – Now in Public Beta ](https://zed.dev/blog/delta-public-beta)

FeaturedSep 16, 2026

  * ### [Xanadu Was Waiting for Agents ](https://zed.dev/blog/agentic-xanadu)

Sep 1, 2026




[Download Delta Now`D`](/download)[Open Delta`D`](/threads)

Zed Industries (C) 2026

[Terms](https://zed.dev/terms)[Privacy](https://zed.dev/privacy)[Brand](/brand)Cookies
