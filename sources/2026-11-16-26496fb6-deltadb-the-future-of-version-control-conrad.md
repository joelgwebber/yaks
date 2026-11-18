---
type: source
title: 'DeltaDB: The Future of Version Control (Conrad Irwin, Delta blog, 2026-10-15)'
description: Saved copy of https://delta.dev/blog/future-of-version-control
media_type: text/markdown
size: 11029
sha256: 26496fb6723022d0ec8d26aef6d265e209984ab82f7ab4eb52f3cc9900103472
sources:
  - id: web
    resource: https://delta.dev/blog/future-of-version-control
    host: joel
    last_modified: 2026-11-16T21:32:55Z
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

# DeltaDB: The Future of Version Control

Conrad IrwinOctober 15, 2026

[ This post is based on a talk by Conrad IrwinBuilding a Distributed CRDT Layer for GitWatch it on YouTube•22 min](https://www.youtube.com/watch?v=yif2rday2bw)

Think back to the last time you lost work or wasted time.

Maybe you were debugging a test, and you added logging but it wasn’t showing up, then you realize, “Oh, I didn’t save the file…” Oops.

Maybe you reach the end of the day, you build everything, you push, and then you’re sitting at home and get a Slack message: “Your branch doesn’t compile. Did you forget to add a file?” It’s embarrassing, particularly if you do it in front of your colleagues. It’s irritating, _particularly_ when you realize you’ve been using these tools for decades.

We’re software engineers; we see people with a problem, and we fix it. We saw people writing documents, having to save them, then losing work because they didn’t save.

So we built Google Docs, and Notion. Everything is synced to the server as you type, and synced to your collaborators’ machines. You can edit things together, and you won’t lose work because there’s no manual action to forget; it’s already saved.

The same is true for design: we used to have to save all the time in Photoshop or it would crash, and now we have Figma. _It just backs it up_ and saves your work as you do it. If you want to work with people, all the changes show up in real time. You can’t lose work. You can’t waste time.

But somehow, in the last 15 years of building amazing tools for everyone else, we never stopped and looked at our own stack.

We never said, “This is stupid, right? It’s painful to lose work. It’s really painful to mess up.”

We fixed it for everyone else, and never took the time to fix it for ourselves, the software engineers. Until now.

## Enter DeltaDB

We’ve been working on [DeltaDB](https://zed.dev/blog/introducing-deltadb) to solve this problem. Though Zed built an editor first, our mission was never to build a text editor. It’s to fundamentally improve the way we build software.

I’m not saying we want to replace Git, which has some nice properties. It’s convenient to be able to say _this commit is this version of the code_. You can tag it, ship it, deploy it, and that snapshot-based approach works.

Where Git falls short is that it requires extensive manual work to do two core things: share your work and back it up. Git is a _very_ fancy save button.

To avoid that you need a different kind of system: one that breaks changes up and sends them out as you type, keystroke by keystroke, to the server and to everyone you’re working with. In that new system, if something goes wrong, there’s no Git command you could have messed up, because there’s no Git command to run. Your work is already sent. Want to go back in time? Just undo. You can’t lose work, you can’t waste time. That’s how it should be.

DeltaDB is the collaboration layer for software development: as you type, changes sync to your collaborators and to the server. If you want to go back in time, you look at the history.

It’s based on CRDTs.

## How DeltaDB’s CRDTs work

CRDTs are a well-studied structure, and there are many different solutions. We used one scheme based on character IDs in [Zed](https://zed.dev/blog/crdts), but for DeltaDB we use a more modern approach. It’s loosely based on the excellent [diamond-types](https://github.com/josephg/diamond-types) approach (though the implementation is quite different to allow us to interoperate with Git).

The key observation of a CRDT is that when we want to send edits over the wire, we can’t just send the change, we need to anchor where it happened both in space and time. In hypothetical Rust:
    
    
    struct EditDelta {
        insert: String,
        at: Range<usize>,
        based_on: Version,
    }
    
    enum Version {
        Git(Sha1),
        Delta(Vec<DeltaId>),
    }

To avoid relying on a distributed clock, we identify each change by its parent in the causal graph. A delta is either based on something in Git, like a commit or a specific blob, or on earlier deltas. It’s a `Vec` because changes can happen simultaneously.

Let’s go through a contrived example. We both start with the string `TIVITY` and I insert `REA` at the same time as you insert a `C`. There are two possible end results: REACTIVITY, or CREATIVITY. The invariant we need to maintain is that regardless of which order these insertions were observed, they end up resolving the same way.

over the networkinsert REA at 0..0 of git-aaainsert C at 0..0 of git-aaainsert C at 0..0 of git-aaainsert REA at 0..0 of git-aaalook up fragment indexlook up fragment indexVersion git-aaaTIVITYgit-aaaYou (a) · version a3REAaTIVITYgit-aaaMe (b) · version b1CbTIVITYgit-aaaBoth · version [a3, b1]REAaCbTIVITYgit-aaaTIVITY at version git-aaa. You insert REA and I insert C, both at 0..0 of git-aaa. We exchange the edits, each look up the fragment index, and both end up with REACTIVITY at version [a3, b1].

We both start from `TIVITY` and we know it’s version `git-aaa`: the file as it is in the git commit we’re working from. I insert my `C` and tell you: insert `C` at `0..0` of `git-aaa`. You insert your `REA` and tell me the same thing: insert `REA` at `0..0` of `git-aaa`.

When my `C` arrives, your document isn’t at `git-aaa` anymore. You’ve inserted three characters, so you’re at version `a3`. You can tell there’s a conflict, because my edit is based on a version that doesn’t include yours. But position zero in `git-aaa` doesn’t tell you where my `C` goes in `a3`.

Now in practice this doesn’t come up that often. When you’re collaborating there’s usually very little concurrency: keystrokes arrive in order, and almost every delta is based on the version you’re already looking at. This makes the common case cheap; you can just insert my edits into your document with no extra work.

But with network latency, or when someone comes back online after hours away, a delta can reference a version that’s well out of date. So everyone needs to be able to answer one question: given that this was position X in version V, where is it now?

We do this with one more data-structure.

### The fragment index

Internally, each string is stored as two things: the text itself, and a fragment index. The fragment index is a B-tree that mirrors the text and records, for every position, which delta created that character.

Version [a3, b1]root10 chars0..3REAa3..4Cb4..10TIVITYgit-aaaThe fragment index for REACTIVITY at version [a3, b1]: a root over three fragments. REA, created by a, covers positions 0..3. C, created by b, covers 3..4. TIVITY, from git-aaa, covers 4..10.

Back to the example. In your version `a3`, the fragment index has two fragments: the `REA` you inserted, and the `TIVITY` from `git-aaa`. (The figure shows the index after both edits have merged, with my `C` as a third fragment.) When my `C` arrives, you look up position zero of `git-aaa`: it’s the start of the `TIVITY` fragment, and your `REA` fragment was inserted at that same spot, concurrently with my `C`. So we use the same actor ID tiebreak as before. `a` comes before `b`, and you get `REACTIVITY`.

The same thing happens on my side. I look up position zero of `git-aaa` in my fragment index, find my `C` there, resolve the conflict the same way, and also get `REACTIVITY`, at version `[a3, b1]`.

### Structural sharing

Naively, this would mean that we need a fragment index for every version of the document. To keep the one-to-one correspondence between characters and fragments, every insertion means rebuilding the part of the tree that covers it.

Version [a3, b1]Version b2root a10 charsroot a′11 charsREAaCbTIVITYgit-aaaREAaCbTIVITYgit-aaa!bnew in version b2from version [a3, b1]9 nodes storedTwo separate fragment indexes. Version [a3, b1] has root a over three leaves: REA from a, C from b, and TIVITY from git-aaa. Version b2 appends ! as a new fragment from b and has its own root a′ over copies of all three leaves plus a new leaf for !. Nine nodes are stored.

A B-tree roughly the size of the string, for every version of the string, is an n-squared-shaped problem.

But we have one more trick up our sleeves. Because these trees are immutable, we can copy an optimization from Haskell: instead of building a completely new tree for each version, the new version only creates the nodes on one path from the root to the changed leaf, and shares everything else with the previous version.

Version [a3, b1]Version b2root a10 charsroot a′11 charsREAaCbTIVITYgit-aaa!bnew in version b2from version [a3, b1]6 nodes storedThe same two versions with structural sharing. Root a points to the REA, C, and TIVITY leaves. The new root a′ points to those same three leaves and to a new leaf for !. Only a′ and the ! leaf are new, so six nodes are stored.

So the tree grows only linearithmicly with the size of the edits, instead of with the number of edits squared. We end up with a kind of multi-headed Medusa tree: each version has its own head, pointing into mostly shared nodes.

And with that, we have the basics working.

## What this unlocks

There’s still more to think about, like how to represent the file system. If you rename a directory and I rename another, it’s easy to end up with a directory inside itself. But text is mostly the hard part.

Instead of baking time into every character’s ID, we bake it into the changes. That lets us take changes made on one Git commit and rebase them onto another.

**And because every character knows which change created it,** we can attach more to it: which person wrote it, which agent, even which prompt. An agent can work on its own copy and merge back in. A colleague can check out your work without you remembering to `git add`, `git commit`, and `git push`.

So think back: when was the last time that your tooling let you down, that you lost work or you lost time?

I hope that will be the last time that ever happens.

On this page

  1. Enter DeltaDB
  2. How DeltaDB’s CRDTs work
     1. The fragment index
     2. Structural sharing
  3. What this unlocks



## More from the blog

  * ### [Building fine-grained version control for agents with Rust and Durable Objects](/blog/fine-grained-version-control-for-agents)

Oct 29, 2026

  * ### [Replace PRs with Delta – Now in Public Beta ](https://zed.dev/blog/delta-public-beta)

FeaturedSep 16, 2026

  * ### [Xanadu Was Waiting for Agents ](https://zed.dev/blog/agentic-xanadu)

Sep 1, 2026




[Download Delta Now`D`](/download)[Open Delta`D`](/threads)

Zed Industries (C) 2026

[Terms](https://zed.dev/terms)[Privacy](https://zed.dev/privacy)[Brand](/brand)Cookies
