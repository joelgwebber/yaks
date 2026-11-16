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



  1. How Delta Works
  2. Delta & Git



# Delta & Git

How Delta records your work, and how that record connects to git.

On this page

  1. What is DeltaDB?
  2. What is a delta?
  3. Open a project
  4. Jujutsu repositories
  5. The two remotes
  6. How work comes back out
  7. What Delta records that git doesn’t
  8. What’s next



Delta records your work continuously. Changes to files included in DeltaDB, whether made by you or the agent, are saved as they happen rather than only when someone commits. Each connected Delta client holds a copy of the database, and Delta keeps the copies in sync, so everyone sees the same history.

## What is DeltaDB?

DeltaDB is the shared database behind a Delta thread. It records thread messages, [comments](/docs/agents/comments), Delta worktree changes, and file edits as deltas. Agents can read and edit a Delta worktree’s shared file representation in DeltaDB. On desktop machines, Delta can also materialize those files in folders called checkouts, where people and filesystem-based tools can edit them. Browser participants and browser-run agents use the replicated file representation directly, without a checkout. DeltaDB records the edits and synchronizes every mounted checkout.

## What is a delta?

A delta is a recorded change to a thread or Delta worktree. It can represent a file edit, a change to the file tree, a message, a comment, or another update to shared state. Each delta records who made the change and where it belongs in the thread’s history. This allows DeltaDB to combine changes from different participants and keep every copy synchronized.

Unlike a git commit, a delta is created continuously as work happens. You do not need to stage or commit it.

## Open a project

The folder you open as a project must be the top-level folder of a Git checkout. Delta does not currently accept an arbitrary folder inside one.

Delta imports every file Git tracks, including tracked files whose paths now match an ignore rule. It also imports uncommitted changes to tracked files and untracked files that Git does not ignore. After the import, DeltaDB records changes to those files as they happen. Commits stay in Git. Delta moves them between machines as needed, so a collaborator’s machine can get your current commit without a round trip through origin.

Delta preserves each file’s line endings when it writes a checkout and honors Git’s `core.autocrlf`, `core.eol`, and `.gitattributes` conversion rules. This keeps CRLF repositories and Windows checkouts from showing every file as modified in `git status`.

The shared representation and its checkouts are two ways to work with the same Delta worktree. The agent’s direct file operations can use the representation in DeltaDB; terminal commands and other filesystem-based tools run in a checkout. Delta keeps both synchronized. [Delta Worktrees](/docs/concepts/worktrees) explains where checkouts live.

## Jujutsu repositories

Delta has early support for Jujutsu (`jj`) repositories that use colocated mode, with `.git` and `.jj` in the same project folder. When the project you open is colocated, Delta runs `jj git init --colocate` in each managed checkout it creates and tracks the default remote branch and your checkout’s upstream, so the agent’s checkouts are colocated too. Jujutsu must be installed for this to succeed; a checkout adopted in place is left as it is.

Delta preserves a managed checkout’s Jujutsu operation log when the checkout is archived and restores it when the checkout is mounted again. On mount, Delta fetches bookmarks from your source checkout through the `local` remote and copies selected repository settings for identity, signing, and `trunk()` behavior. Arbitrary aliases and executable configuration remain isolated from the managed checkout.

Jujutsu support remains incomplete. If you see unexpected behavior, email [delta-beta@zed.dev](mailto:delta-beta@zed.dev).

## The two remotes

A checkout that Delta creates from your own project folder normally has two Git remotes with different roles. Your repository must have a configured remote before you add or attach it to a thread, even if you don’t plan to share the thread. Delta uses the remote to identify the repository.

With one source remote, its name can be anything. With several, one must be named `origin`; Delta uses that one. The `local` backlink Delta creates is separate from this source-remote selection.

If your repository has no remote, add one and push your branch before opening it in Delta. See [Git remote troubleshooting](/docs/troubleshooting#delta-says-my-project-needs-a-git-remote) for the steps.

  * **Source remote** , often named `origin`: the shared upstream your repository already points at, such as GitHub. Delta preserves the selected remote’s original name. Use this remote to publish a branch or open a pull request; check its name before pushing.
  * **local** : your own repository on your machine. The agent can push a branch to local to hand you work without a round trip through GitHub, including the branch you have checked out: a clean checkout updates in place, and git rejects a push that would overwrite local edits.



On a teammate’s machine (they joined the thread without having the project), the source remote points at the same shared upstream, but `local` points at Delta’s own clone of the repository rather than at a checkout of theirs.

## How work comes back out

The agent’s changes reach files you use in four ways:

  1. **In place.** A thread can adopt your own project folder as its checkout. Your uncommitted changes come along with it, and the agent’s edits appear in your project folder directly.
  2. **Through local.** The agent pushes a branch to the `local` remote, straight into your repository. Pushing the branch you have checked out updates your working tree in place when it is clean.
  3. **Through the source remote.** The agent pushes a branch to your shared upstream. You fetch that branch into your own checkout, or review it as a pull request.
  4. **From the checkout.** A Delta-managed checkout is a real folder. You can open it in Zed, build in it, or copy files out of it.



See [Review & Sync Changes](/docs/agents/review-and-sync) for step-by-step commands.

## What Delta records that git doesn’t

Git records a snapshot of the repository each time someone commits. DeltaDB records each edit, from each person and the agent, in the order it is made. You can review and comment on anyone’s work, agent or collaborator, in the middle of a turn, and revert the conversation to an earlier point, restoring the thread’s Delta worktrees along with it. None of it involves a commit.

For untracked files, the record follows your repository’s `.gitignore`. Untracked ignored files are not recorded or synced, so build artifacts and untracked `.env` files stay on the machine where they were created.

WARNING

Adding a path to `.gitignore` does not protect a file Git already tracks. Delta still imports tracked files that match an ignore rule.

## What’s next

  * [Delta Worktrees](/docs/concepts/worktrees) explains where the agent works and how checkouts sync.
  * [Review & Sync Changes](/docs/agents/review-and-sync) shows how to review edits and bring them back to your repository.



Zed Industries (C) 2026

[Terms](https://zed.dev/terms)[Privacy](https://zed.dev/privacy)Cookies

  1. What is DeltaDB?
  2. What is a delta?
  3. Open a project
  4. Jujutsu repositories
  5. The two remotes
  6. How work comes back out
  7. What Delta records that git doesn’t
  8. What’s next


