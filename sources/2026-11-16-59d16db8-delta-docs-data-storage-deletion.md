---
type: source
title: 'Delta docs: Data Storage & Deletion'
description: Saved copy of https://delta.dev/docs/privacy-and-security/data-storage
media_type: text/markdown
size: 12115
sha256: 59d16db8a2e6c217245825cebe3a65bd5f5ecb7d67f2dc9cb508045a86e88690
sources:
  - id: web
    resource: https://delta.dev/docs/privacy-and-security/data-storage
    host: joel
    last_modified: 2026-11-16T21:32:57Z
    retrieved: 2026-11-16T21:33:23Z
generated:
  by: delta:ksQQM-tlBqyPR_q2SQ147vmMIZLOAGl2rZIBxBCA
  at: 2026-11-16T21:33:23Z
---

Skip to content

[](https://delta.dev)

Search…

Getting Started

  * [Welcome](/docs/getting-started)
  * [Install Delta](/docs/installation)
  * [Quick Start](/docs/getting-started/quick-start)
  * [Release notes](/docs/whats-in-the-latest)
  * [Troubleshooting](/docs/troubleshooting)



Delta CLI

Beta

  * [Get Started with Delta CLI](/docs/cli/get-started)
  * [Claude Code Plugin](/docs/cli/claude-code)
  * [Build a Coding Agent Plugin](/docs/cli/build-a-plugin)



How Delta Works

  * [Core Concepts](/docs/concepts/core-concepts)
  * [Delta & Git](/docs/concepts/delta-and-git)
  * [Delta Worktrees](/docs/concepts/worktrees)



Using Delta

  * [Threads](/docs/agents/threads)
  * [Subagents](/docs/agents/subagents)
  * [Terminals](/docs/agents/terminals)
  * [Cloud Runners](/docs/agents/cloud-runners)
  * [Review & Sync Changes](/docs/agents/review-and-sync)
  * [Language Support](/docs/agents/language-support)
  * [Comments](/docs/agents/comments)
  * [Skills](/docs/agents/skills)
  * [MCP Servers](/docs/agents/mcp)
  * [Models & Providers](/docs/agents/models-and-providers)
  * [Claude Agent](/docs/agents/claude-agent)



Collaboration

  * [Collaborate in a Thread](/docs/collaboration/collaborate-thread)
  * [Delta on the Web](/docs/collaboration/delta-on-the-web)



Configuration

  * [Prepare your project](/docs/configuration/project-setup)
  * [Settings](/docs/configuration/settings)
  * [Keybindings](/docs/configuration/keybindings)



Account & Billing

  * [Sign In](/docs/account/authenticate)
  * [Plans & Usage](/docs/account/plans-and-pricing)



Privacy & Security

  * [Data Storage & Deletion](/docs/privacy-and-security/data-storage)
  * [AI Privacy & Telemetry](/docs/privacy-and-security/privacy)
  * [Security](/docs/privacy-and-security/security)
  * [Agentic Safety](/docs/privacy-and-security/agentic-safety)



[](https://delta.dev)

[Back to site](https://delta.dev)

Getting Started

  * [Welcome](/docs/getting-started)
  * [Install Delta](/docs/installation)
  * [Quick Start](/docs/getting-started/quick-start)
  * [Release notes](/docs/whats-in-the-latest)
  * [Troubleshooting](/docs/troubleshooting)



Delta CLI

Beta

  * [Get Started with Delta CLI](/docs/cli/get-started)
  * [Claude Code Plugin](/docs/cli/claude-code)
  * [Build a Coding Agent Plugin](/docs/cli/build-a-plugin)



How Delta Works

  * [Core Concepts](/docs/concepts/core-concepts)
  * [Delta & Git](/docs/concepts/delta-and-git)
  * [Delta Worktrees](/docs/concepts/worktrees)



Using Delta

  * [Threads](/docs/agents/threads)
  * [Subagents](/docs/agents/subagents)
  * [Terminals](/docs/agents/terminals)
  * [Cloud Runners](/docs/agents/cloud-runners)
  * [Review & Sync Changes](/docs/agents/review-and-sync)
  * [Language Support](/docs/agents/language-support)
  * [Comments](/docs/agents/comments)
  * [Skills](/docs/agents/skills)
  * [MCP Servers](/docs/agents/mcp)
  * [Models & Providers](/docs/agents/models-and-providers)
  * [Claude Agent](/docs/agents/claude-agent)



Collaboration

  * [Collaborate in a Thread](/docs/collaboration/collaborate-thread)
  * [Delta on the Web](/docs/collaboration/delta-on-the-web)



Configuration

  * [Prepare your project](/docs/configuration/project-setup)
  * [Settings](/docs/configuration/settings)
  * [Keybindings](/docs/configuration/keybindings)



Account & Billing

  * [Sign In](/docs/account/authenticate)
  * [Plans & Usage](/docs/account/plans-and-pricing)



Privacy & Security

  * [Data Storage & Deletion](/docs/privacy-and-security/data-storage)
  * [AI Privacy & Telemetry](/docs/privacy-and-security/privacy)
  * [Security](/docs/privacy-and-security/security)
  * [Agentic Safety](/docs/privacy-and-security/agentic-safety)



  1. Privacy & Security
  2. Data Storage & Deletion



# Data Storage & Deletion

What DeltaDB stores on your behalf and where it lives.

On this page

  1. What Delta stores
  2. Where your data lives
     1. Encryption
     2. Subprocessors
  3. Data retention and deletion
     1. Delete your account and data
     2. Delete individual threads
     3. What deletion does not cover
     4. Retention



When you add a repository or create a thread, Delta uses [DeltaDB](/docs/concepts/delta-and-git#what-is-deltadb) to store its contents and history on your machine and on our servers. This keeps your work available across machines and makes it possible to share a thread with teammates. Storing a thread on our servers does not share it. Threads stay private to you unless you share them or set **Default Visibility** to **Your organization** in **Settings > Thread > Sharing**.

Using the agent also sends your conversation, including any code it quotes, through Zed Cloud to your chosen model provider. Learn more about [what is sent to model providers](/docs/privacy-and-security/privacy#ai-requests).

## What Delta stores

Delta stores these categories of data on its servers:

  * **Repository contents and history.** The Git objects for repositories attached to your threads, including commits and file contents.
  * **The thread’s deltas.** A delta is a recorded change to a thread or [Delta worktree](/docs/concepts/worktrees), such as a file edit, a change to the file tree, a message, or a comment. Delta stores these changes in sequence so it can reconstruct each thread and keep copies synchronized across machines.
  * **Metadata.** Delta stores: 
    * The shareable thread record, including the key needed to join the thread and its display name.
    * Your account’s thread catalog, including which uploaded or joined threads are pinned, archived, or active.
    * The Git remote URL you configured, the derived repository name, and a normalized lookup key. Avoid remotes that embed credentials.
    * Each collaborator’s GitHub login when a thread is shared.
    * Git commit author and committer identities and commit messages.
  * **Account settings.** Your application settings, including model provider API keys and subagent profiles, sync between devices. Each device keeps an encrypted local cache. Environment variables you add for Cloud Runners are stored here.
  * **Cloud Runner sandboxes.** If you use [Cloud Runners](/docs/agents/cloud-runners), each runner’s sandbox holds checkouts of the thread’s Delta worktrees and anything the agent or your setup script creates there. A paused sandbox keeps its files until you archive or delete the thread, which deletes your sandbox for it. Delta may also delete a sandbox and create a new one, for example after an incompatible update.
  * **Subscription credentials.** Zed Cloud stores encrypted OAuth grants for ChatGPT, GitHub Copilot, and Grok subscriptions. It refreshes these grants and provides your devices and Cloud Runners with short-lived access tokens.



## Where your data lives

Delta stores and synchronizes repository and thread data on Cloudflare. Cloud Runner sandboxes run on E2B.

Data| Storage  
---|---  
File contents, Git commits, history packs, and checkpoints| Cloudflare R2  
Thread and Delta worktree deltas used for active synchronization| Cloudflare Durable Objects backed by SQLite  
Account settings| Cloudflare Durable Objects backed by SQLite  
Thread, repository, collaborator, and account metadata| Cloudflare KV and D1  
Cloud Runner sandboxes| E2B, in `us-west-1` or `eu-west-1`  
  
Delta chooses a sandbox’s region when it creates your first runner for a thread. Threads whose runner is first handled by a Cloudflare data center in Europe use `eu-west-1`; all others use `us-west-1`. Later runners for that thread stay in the same region.

Each thread’s history is stored under its own unique ID. Delta uses the Git remote URL to identify a repository within your organization. Organization members working from the same remote use the same stored repository data. Access to stored repository data comes from direct repository membership, organization membership when the repository is visible to the organization, or access to a thread with an attached Delta worktree from the repository.

Sharing a thread grants access to its attached repositories. Delta worktree history access is repository-wide, so this also grants access to every Delta worktree history for each attached repository, including histories that are not attached to the shared thread. Removing every Delta worktree for a repository from the thread removes the access granted by that thread.

### Encryption

Data stored by Delta is encrypted at rest. See [Encryption](/docs/privacy-and-security/security#encryption) for details.

### Subprocessors

Delta uses Cloudflare as an additional subprocessor to store and synchronize repository and thread data. Cloudflare processes this data globally. Delta also uses the services on [Zed’s subprocessor list](https://zed.dev/subprocessors).

## Data retention and deletion

You can request deletion of your Delta account and personal data. Account deletion is not available directly in Delta; follow the email instructions below.

### Delete your account and data

To request deletion, email [privacy@zed.dev](mailto:privacy@zed.dev?subject=Privacy%20Request) from the email address associated with your account. We verify your identity, delete your data, and confirm when deletion is complete.

Account deletion removes:

  * Your account, credentials, and active sessions.
  * Your personal threads.
  * Delta server data controlled solely by you.



Deleting your Zed account also deletes your Delta account and associated data.

### Delete individual threads

You can delete a thread with **Delete Locally** in the thread list’s right-click menu, or with the `delta cli thread delete` and `delta cli thread purge` commands (`purge` only prints what would be lost until you pass `--force`; both require Delta to be closed). **Delete Locally** removes the thread’s local copy on this computer. It does not reliably remove copies on your other devices or delete the thread’s history from Delta’s servers. Account deletion removes all your personal threads together.

If you have a concern about the contents of a specific thread, email [privacy@zed.dev](mailto:privacy@zed.dev?subject=Privacy%20Request) and we will review it.

### What deletion does not cover

Delta controls its server storage, but it does not control every location your data may reach:

  * **Upstream repositories.** Deleting Delta data does not affect the canonical repository on GitHub or another host.
  * **Local copies.** Delta cannot delete clones, checkouts, or thread copies on other people’s machines. On your own machine, Delta removes the local history of an archived, synced thread three days after you archive or last open it; the server copy remains and downloads again when you open the thread.
  * **Already-shared content.** Deleting Delta’s server-side copy cannot retract content someone else has already downloaded or exported.
  * **Shared content.** Content you contributed to shared threads may be retained. We do not rewrite thread or git history, so your name may remain in retained shared content, such as commit history or thread state. If you have a privacy request about identifying information in retained content, email [privacy@zed.dev](mailto:privacy@zed.dev?subject=Privacy%20Request) and we will review it separately.



### Retention

We may retain some data where legally required. Deleted data may also remain temporarily in backups before it is fully purged.

Locally, Delta keeps the full history of your active threads. Archived threads that are synced to our servers drop their local history after three days to free disk space; opening one downloads its history again. Threads that were never synced keep their local history until you delete them.

Zed Industries (C) 2026

[Terms](https://zed.dev/terms)[Privacy](https://zed.dev/privacy)Cookies

  1. What Delta stores
  2. Where your data lives
     1. Encryption
     2. Subprocessors
  3. Data retention and deletion
     1. Delete your account and data
     2. Delete individual threads
     3. What deletion does not cover
     4. Retention


