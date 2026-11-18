---
type: topic
title: 'DeltaDB (Zed/Delta): what it is, what is open, and whether yaks can use it'
created: '2026-11-16T21:34:27Z'
tags:
- research
- delta
- git
- sync
- crdt
stale_after: P3M
sources:
- sources/2026-11-16-0df8e0eb-delta-cli-product-page-bring-delta-to-the-agent.md
- sources/2026-11-16-26496fb6-deltadb-the-future-of-version-control-conrad.md
- sources/2026-11-16-34edbef8-syntax-1029-transcript-the-workflow-of-the.md
- sources/2026-11-16-59d16db8-delta-docs-data-storage-deletion.md
- sources/2026-11-16-5b177191-zed-industriesdelta-claude-readme-agpl-30-claude.md
- sources/2026-11-16-662a0d4c-hacker-news-thread-zed-staff-on-self-hosting.md
- sources/2026-11-16-737a30ca-delta-public-roadmap-as-of-2026-11.md
- sources/2026-11-16-740faeed-delta-docs-delta-git.md
- sources/2026-11-16-985aa32d-delta-docs-build-a-coding-agent-plugin-cli.md
- sources/2026-11-16-a35b9190-software-is-made-between-commits-sobo-zed-blog.md
- sources/2026-11-16-b7f89c23-building-fine-grained-version-control-for-agents.md
- sources/2026-11-16-cea6b808-sequoia-backs-zeds-vision-for-collaborative.md
- sources/2026-11-16-e5d7f476-deltadb-product-page-deltadev-zed-industries.md
- sources/2026-11-16-f2ff0795-delta-docs-get-started-with-delta-cli.md
description: What DeltaDB (the CRDT version control under Zed's Delta) is, what infrastructure it runs on, what is open source (only the AGPL Claude plugin), and why it is not yet a store or sync layer for a farm; read before considering CRDT or live sync for yaks.
---

# DeltaDB: what it is, what is open, and whether yaks can use it

Research for yaks-d741. Question: could DeltaDB, the version-control layer under Zed Industries' Delta, store or sync a farm better than commits on the `yaks` branch, or run beside them? What wiring and server infrastructure would it take, and is any of it open source?

**Short answer.** Not today, and nothing to build against yet. DeltaDB is closed source, runs only on Zed's hosted service (Cloudflare), and is reachable outside the Delta app only through the beta `delta` CLI, which is the Delta app run headless and signed in to a Zed account. Zed has said it *plans* to open-source DeltaDB, but has published no code, crate or date. Its data model is a good fit for a farm (a last-writer-wins register for `status:`, an add-wins map for notes, a text CRDT for the body), which supports the direction the merge-driver yaks already take. Revisit when a public crate or a self-hostable server appears.

## What DeltaDB is

All of this comes from Zed's own posts and docs; none of it is independently checked.

- **A causal DAG of CRDT operations ("deltas"), stored locally in SQLite.** Every edit, message, comment and file-tree change is a delta naming the version it builds on: earlier deltas, or a Git blob for a file's starting text. A version is a *frontier* (the tips of the DAG), not a version vector, so short-lived writers (subagents, cloud sandboxes) don't grow it forever. The design follows Gentle and Kleppmann's Eg-walker and is "loosely based on diamond-types" ([Scandurra](/sources/2026-11-16-b7f89c23-building-fine-grained-version-control-for-agents.md), [Irwin](/sources/2026-11-16-26496fb6-deltadb-the-future-of-version-control-conrad.md), [product page](/sources/2026-11-16-e5d7f476-deltadb-product-page-deltadev-zed-industries.md)).
- **Text** is a rope plus a fragment index (which delta made each character); versions share structure through copy-on-write B+ trees, so any past version can be rebuilt cheaply.
- **Git stays.** A Delta worktree starts at a commit; each file is either pinned to its committed blob or carries the deltas written since, and making a commit is itself a delta. Commits, branches and remotes stay in Git; DeltaDB keeps "everything in between" ([Delta & Git](/sources/2026-11-16-740faeed-delta-docs-delta-git.md)).
- **Timelines** are named frontiers in one DAG: forking a subagent's worktree is a new timeline, merging is a merge of frontiers. No Git branch or commit is involved.
- **Client API** (Rust, internal): `Register<T>` (last writer wins), `Map<K, V>` (add wins), `Buffer` (collaborative text), `Worktree` (files over Git), composed with `#[derive(Replicate)]` into structs such as a thread with a transcript, a name and a map of comments. The post shows `deltadb::Client`, but no such crate is published (crates.io has nothing named deltadb).

## What the infrastructure is

- **Client:** SQLite per machine, one writer thread and several readers.
- **Server:** one Cloudflare Durable Object per thread (its own SQLite), WebSockets per thread, catch-up sync by exchanging logarithmically spaced ancestors to find a common one. File contents, Git objects, history packs and checkpoints go to Cloudflare R2; metadata to KV and D1 ([Data storage](/sources/2026-11-16-59d16db8-delta-docs-data-storage-deletion.md)).
- **Deployments:** Zed's multi-tenant service, plus single-tenant "bring your own Cloudflare" for customers. Zed says it is portable to any platform with an object store, a fault-tolerant Postgres for authentication, and addressable single-threaded processes with transactional storage. That describes their code, which only they can deploy.
- Sobo, on Syntax: the algorithms "don't depend on a central synchronization coordinator in a critical way", but launch is centralized at delta.dev, partly because that is how Zed makes money ([Syntax #1029](/sources/2026-11-16-34edbef8-syntax-1029-transcript-the-workflow-of-the.md)).

## Is it open source?

| Piece | Status |
|---|---|
| DeltaDB engine, server, `deltadb::Client` | **Closed.** No repository, no crate. |
| Delta app and `delta` CLI | **Closed** binaries; free Personal plan. |
| `zed-industries/delta-claude` (the Claude Code plugin) | **Open**, AGPL-3.0-only. It shells out to the closed `delta` CLI ([README](/sources/2026-11-16-5b177191-zed-industriesdelta-claude-readme-agpl-30-claude.md)). |
| Zed editor | Open (GPL-3.0, Apache-2.0 parts), but has no DeltaDB in it yet. |

**Stated intent.** In August 2025: "Zed is open-source with an optional paid offering, and we plan to do the same with DeltaDB: build it, open-source it, and offer an optional paid service" ([Sequoia post](/sources/2026-11-16-cea6b808-sequoia-backs-zeds-vision-for-collaborative.md)). Nothing I found since repeats or withdraws that; the 2026 posts don't mention licensing. On self-hosting, Zed staff said in November 2025 that self-hosted collab had been dropped and would be revisited "once we have a few more features settled", with collab to be rebuilt on DeltaDB ([HN](/sources/2026-11-16-662a0d4c-hacker-news-thread-zed-staff-on-self-hosting.md)). The public [roadmap](/sources/2026-11-16-737a30ca-delta-public-roadmap-as-of-2026-11.md) lists neither open-sourcing nor self-hosting; its nearest item is a Delta MCP server ("up next").

## Using it outside Delta today

The only door is the beta `delta` CLI ([get started](/sources/2026-11-16-f2ff0795-delta-docs-get-started-with-delta-cli.md), [plugin guide](/sources/2026-11-16-985aa32d-delta-docs-build-a-coding-agent-plugin-cli.md)):

- **Install:** a script installs the whole Delta app into your home folder; the CLI starts it in the background when needed. `delta auth login` signs in to a Zed account, and threads belong to a Delta organization.
- **Files:** `delta worktree create <checkout>` then `delta worktree mount <ref> <checkout> --existing-checkout` uploads the checkout (uncommitted and untracked-but-not-ignored files included) and syncs it both ways for as long as the command runs. The checkout must be a Git repository with a remote; Delta identifies the repository by that remote.
- **Conversation:** `delta thread create/append/watch` record and stream ACP-shaped messages; this is how the Claude Code plugin works.
- **The rule:** "Use the CLI for everything. Don't read or write Delta's database directly." There are no general primitives: no registers, maps or buffers of your own, only threads and worktrees.

So the wiring for any yaks use is: every participant installs Delta, signs in to Zed, joins one organization, and keeps a `delta` process running per mount. The data lives on Zed's Cloudflare account.

## What it could mean for yaks

Three shapes, from available now to hypothetical.

### 1. Mount the farm worktree with `delta worktree mount` (possible today)

The farm is a Git worktree of the `yaks` branch of a repository with a remote, so it meets the mount's requirements. Two machines mounting it would see each other's farm edits within seconds, with no `yaks sync` and no commits; the farm would still be committed and pushed to Git for anyone outside Delta.

Against it:

- **A text CRDT is not a field merge.** Two concurrent changes to `status:` in the same yak file merge character by character and keep both insertions, which can leave a value that is neither (`status: shornshaving`). That is exactly what the frontmatter-aware merge driver (yaks-4a53, yaks-4c8c) exists to prevent. A Delta-mounted farm loses it and gains nothing that catches it, except `yaks doctor` after the fact.
- **The farm lock doesn't span machines.** `.lock` is ignored, so it stays local, and it was only ever per machine anyway; DeltaDB has no lock to take.
- Everyone who writes needs Delta and a Zed account, and the farm's contents sit on a third party's servers.
- Commits are still needed for history outside Delta, and per-change commits (yaks-a838) would race with live sync from other machines.

Worth a one-afternoon experiment at most, as a Delta-only convenience, not a storage choice.

### 2. A farm as DeltaDB primitives (not possible: no public API)

A farm modeled as `Map<YakId, Yak { status: Register<Status>, title: Register<String>, body: Buffer, notes: Map<NoteId, Note> }>` would get the right merge per field by construction: status and fields last-writer-wins, notes add-wins (never lost), body merged as text. That is the op-log design yaks-4fe6 explored with git-bug, with someone else's engine, and it fails the same invariant: files stop being authoritative and become a cache. With no published client API, it is moot until DeltaDB is open.

### 3. Take the ideas, not the product (available)

- **Per-field merge rules are the right model.** DeltaDB uses an LWW register for scalars and an add-wins map for collections, which is what yaks' merge driver approximates in Git: last writer wins on `status:` and scalars, union of notes and tags. That is evidence for the yaks-4a53/yaks-4c8c direction, not a reason to change it.
- **Frontiers over version vectors** matter for systems with many short-lived writers. yaks has no vector clocks to replace, so this is only background.
- If yaks ever wants real CRDT sync, the open engines DeltaDB descends from (diamond-types, Eg-walker) and peers (Automerge, Loro) are the ones to evaluate. I have not reviewed their licenses or Rust APIs here, and nothing in yaks needs them today.

## Recommendation

Keep the farm as files on the `yaks` branch and keep the merge-driver work as it is. Don't build on DeltaDB until one of these is true, then reopen this dok:

- a public DeltaDB repository or crate with a license that allows embedding;
- a self-hostable server, or a documented sync protocol;
- a general-purpose primitives API through the CLI or the planned MCP server.

## Open questions

- [ ] Will DeltaDB be released open source, under what license, and does that include the server? The August 2025 pledge is the only statement found.
- [ ] How does Delta's file CRDT handle two concurrent edits to one frontmatter line in practice? Testable today with two `delta worktree mount`s of a scratch farm, if shape 1 is ever wanted.
- [ ] Will the roadmap's Delta MCP server expose anything beyond threads and repository context?
