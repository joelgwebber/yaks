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



  1. Delta CLI
  2. Get Started with Delta CLI



# Get Started with Delta CLI

Install Delta CLI from your terminal, then find, share, and fork conversations and manage worktrees.

On this page

  1. Install Delta CLI
  2. Update Delta
  3. Open Delta
  4. Find conversations
  5. Read a conversation
  6. Ask about previous work
  7. Share a conversation and its files
     1. Invite people to a thread
     2. Make a thread private
     3. Start a thread from a checkout
  8. Manage organizations
  9. Fork a thread
  10. Send a message between agents
  11. Delete a thread
  12. Work with worktrees
  13. Search connected MCP tools
  14. Use the CLI in scripts
  15. Next steps



> **Beta:** The Delta CLI is in beta. Commands, flags, and output can change between releases.

Use Delta CLI to search and read your conversations, share threads with your organization or invite people by email, and work with Delta worktrees from your terminal. To share Claude Code sessions, use the [Claude Code Plugin](/docs/cli/claude-code), which runs the CLI for you.

## Install Delta CLI

On macOS or Linux, run:
    
    
    curl -fsSL https://delta.dev/install.sh -o delta-install.sh
    sh delta-install.sh

The installer downloads the latest stable Delta release, because the CLI runs on the Delta app. You don’t need to open the app: the CLI starts it in the background when a command needs it, and it exits when nothing needs it. Everything goes in your home folder:

  * **macOS:** `~/Applications/Delta.app`, the Applications folder in your home folder rather than `/Applications`.
  * **Linux:** `~/.local/delta.app`.



The `delta` command is linked at `~/.local/bin/delta`. If Delta is already installed, including in `/Applications` on macOS, the installer uses that copy without updating it. The CLI needs Delta 0.22.0 or later, so update an older Delta first with **Delta > Check for Updates** in the app. Later updates can use `delta update`.

Open a new terminal, then sign in and check your installation:
    
    
    delta auth login
    delta --version

`delta auth login` provides a device sign-in URL. Open the URL on a device with a browser to finish signing in; the machine running the CLI does not need a browser.

On Windows, run the installer from the [Delta download page](https://delta.dev/download). It adds `delta.exe` to your `PATH`. See [Install Delta](/docs/installation) for other options, including unattended installs.

> **Tip:** Already using the Delta app on a Mac? Choose **Delta > Install CLI…** in the menu bar to add the `delta` command, instead of running the installer. Update to Delta 0.22.0 or later first with **Delta > Check for Updates**.

The CLI uses the same account as the app. If you add your own `delta` command, link to the installed executable instead of copying it, because the CLI finds the app next to itself.

Like the app, the CLI sends usage telemetry tied to your account, without your conversations, code, or file paths. See [AI Privacy & Telemetry](/docs/privacy-and-security/privacy).

## Update Delta
    
    
    delta update --check
    delta update

`delta update --check` reports whether a newer release is available without downloading anything. `delta update` downloads and installs it, for both the app and the CLI, on the same release channel, and shows its progress.

If Delta is running, `delta update` quits it, installs the update, and reopens the app if it had a window. Commands connected to Delta, such as Claude Code sessions with the Delta plugin, disconnect briefly. If an agent turn or a background terminal is still running, the update stops before changing anything and tells you. Try again once that work finishes.

On Windows, the installer runs after `delta update` exits, and Windows may ask for permission. When it finishes, run `delta update --check` to confirm. If a package manager installed Delta, update it there.

## Open Delta
    
    
    delta
    delta open .

`delta` opens the desktop app. `delta open .` opens it with the current folder. `delta open` also accepts a Delta thread link and opens it in the desktop app, not in a browser. Paths and links need `open`: `delta .` doesn’t work.

## Find conversations
    
    
    delta thread list
    delta thread search --scope messages authentication
    delta thread search --scope messages 'deadlock|panic' sqlite
    delta thread search --edited-path 'src/**' --limit 10

  * Search ignores case unless you add `--case-sensitive`.
  * Every term must match. Use `|` for alternatives within one term.
  * Without `--scope`, search checks generated titles, summaries, and keywords first, then searches messages if nothing matched.
  * `--edited-path` matches files the agent edited in the conversation, not files on disk.



## Read a conversation

This example finds the best match for a search and reads its first 20 messages. It requires `jq`:
    
    
    THREAD_ID="$(delta thread search --scope messages authentication --limit 1 | jq -er '.[0].id')"
    delta thread show "$THREAD_ID" --format messages --messages 1-20

Ranges are inclusive and contain at most 20 messages. Continue with `21-40`. The range must fit inside the conversation. If it doesn’t, the error tells you how many messages the conversation has. To read the full transcript, or the summarized version Delta uses for search:
    
    
    delta thread show "$THREAD_ID"
    delta thread show "$THREAD_ID" --format summarize

You can also pass an accessible shared thread’s link or `thread_...` share ID to `thread show`, even if the thread is not on this device yet. Its subthreads do not need to be shared separately.

## Ask about previous work
    
    
    delta thread query --keyword authentication "What did we decide about sign-in?"

Query sends the selected transcripts to a model, so you must be signed in. Select conversations with at least one `--thread`, `--keyword`, or `--edited-path`. Query answers questions about past work. It doesn’t run coding tasks.

## Share a conversation and its files

With `THREAD_ID` set as above:
    
    
    delta --json thread share "$THREAD_ID" --visibility organization

This publishes the thread and its attached files to your whole organization, not only to the people you send the link to. The response includes a `url` for [Delta on the web](/docs/collaboration/delta-on-the-web).

### Invite people to a thread

To share a thread with specific people instead of your whole organization:
    
    
    delta thread invite "$THREAD_ID" ada@example.com sam@example.com

If the thread isn’t uploaded yet, Delta uploads it privately first, so only you and the people you invite can open it. A thread that’s already shared keeps its current audience, and the invited people get access too. This works in any organization, including your personal one.

The response includes the thread’s `url` and a result for each address. One failed invitation doesn’t stop the others, so check each result.

### Make a thread private

If you own the thread, run:
    
    
    delta thread unshare "$THREAD_ID"

This removes organization-wide or link-based access while keeping explicitly invited people. To change a subthread’s access, change its parent instead.

### Start a thread from a checkout

Run this example from the root of a Git checkout. It requires `jq` and a selected organization:
    
    
    set -eu
    THREAD_ID="$(delta --json thread create | jq -er '.thread_id')"
    REFERENCE="$(delta --json worktree create "$PWD" | jq -er '.reference')"
    delta --json worktree attach "$REFERENCE" --thread "$THREAD_ID"
    delta --json thread share "$THREAD_ID" --visibility organization
    delta --json worktree mount "$REFERENCE" "$PWD" --existing-checkout

  * `worktree create` records a worktree for the checkout on this computer.
  * `worktree attach` adds the worktree to the thread. It doesn’t upload or sync any files.
  * `worktree mount --existing-checkout` uploads the checkout’s files, including uncommitted changes, and keeps syncing them. It keeps running until you stop it with `Ctrl+C`. See Work with worktrees.



To upload the thread privately instead, create it with `--organization ORGANIZATION_ID --visibility private`, using an ID from `delta org list`, and skip `thread share`. Delta uploads it in the background, even if you’re offline, and only you can see it until you share it.

To start the thread with an unsent draft and a chosen model, add `--prompt "Fix the flaky login test" --model anthropic/claude-sonnet-4-6` to `thread create`. Nothing runs until someone sends the draft.

Sharing a checkout doesn’t capture conversations from agents running in it. To share an agent conversation, use a plugin such as the [Claude Code Plugin](/docs/cli/claude-code).

## Manage organizations

Threads belong to an organization. Sharing a thread makes it visible to the organization’s members, so set up an organization with your team before you share:
    
    
    delta org create "Example Team"
    delta org list
    delta org switch ORGANIZATION_ID
    delta org invite ada@example.com sam@example.com
    delta org current

  * Replace `ORGANIZATION_ID` with an ID from `delta org list`. Creating an organization doesn’t switch to it.
  * `org invite` invites people to the selected organization. Add `--org ORGANIZATION_ID` to invite to a different one without switching. Members can open every thread shared with the organization. To give someone access to one thread only, use `thread invite`.
  * An invitation to several people can succeed for some and fail for others, so check the result for each address.



You can also create organizations and invite members in the Delta app. See [Create an organization](/docs/account/authenticate#create-an-organization).

## Fork a thread
    
    
    delta thread fork "$THREAD_ID" --name "Alternative approach"

The fork is a new local thread with a copy of the conversation and its own worktrees. Changes in the fork don’t affect the original. Forking doesn’t start an agent response.

## Send a message between agents

Agents can use the CLI to send a message to another thread’s agent:
    
    
    delta thread send DESTINATION_THREAD --from SOURCE_THREAD "The checks passed."

Replace both thread references with IDs or Delta links. The recipient receives an agent message attributed to the source thread, not a new user message. When run inside Delta, `--from` defaults to `DELTA_CURRENT_THREAD_ID`. If the message argument is omitted, the command reads it from stdin.

## Delete a thread
    
    
    delta thread delete "$THREAD_ID"

This prints a preview and exits with an error. Add `--force` to delete the thread, the local history only it uses, and your Cloud Runner for it. Forced deletion is permanent, so read the preview first.

To archive a thread instead, or restore an archived one:
    
    
    delta thread archive "$THREAD_ID"
    delta thread unarchive "$THREAD_ID"

## Work with worktrees

A [Delta worktree](/docs/concepts/worktrees) stores a project’s files. Mounting it creates a local checkout you can edit with any editor or tool. A worktree doesn’t need a thread.

Run this example from the root of a Git checkout. It requires `jq`:
    
    
    set -eu
    REFERENCE="$(delta --json worktree create "$PWD" | jq -er '.reference')"
    delta --json worktree mount "$REFERENCE" "$PWD" --existing-checkout

`worktree mount` keeps running and syncs changes in both directions until you stop it. Press `Ctrl+C` to stop it. Delta then spends a short time uploading changes it hasn’t sent yet, and stops syncing. The folder and its files stay where they are. To resume syncing, run the same `worktree mount` command again.

  * `worktree create` records a new worktree on this computer, starting from a commit: `HEAD` by default, or a revision you pass after the folder. It doesn’t upload anything.
  * `--existing-checkout` treats the files on disk, including uncommitted changes, as the starting point. It needs a checkout of the same repository with a Git remote.
  * Without `--existing-checkout`, the second argument must be a folder that doesn’t exist yet. Delta creates it from the worktree’s files.
  * `worktree list` shows worktrees on this computer, not every worktree in your organization. Add `--thread THREAD` to list a shared thread’s worktrees.
  * `worktree attach REFERENCE --thread THREAD` adds a worktree to a thread, and `worktree detach REFERENCE --thread THREAD` removes it. Neither changes your files or starts or stops syncing.



Run `delta worktree --help` for more commands, such as `worktree merge`.

## Search connected MCP tools
    
    
    delta mcp search github

This searches the tools of your [connected MCP servers](/docs/agents/mcp) and prints their schemas as JSON. It doesn’t call the tools. You must be signed in.

## Use the CLI in scripts

Results go to stdout and diagnostics go to stderr.

  * Add `--json` to `auth`, `org`, `update`, and `worktree` commands for JSON output.
  * `thread list`, `search`, `show`, and `status` print JSON without `--json`.
  * `thread append` and `thread watch` use JSON Lines.
  * `worktree mount` and `thread mount` print one JSON line when the mount is ready, then keep running until you stop them.
  * `delta` and `delta open` reject `--json`.



Exit codes vary by command. For example:

  * `auth status` exits `0` when you’re signed in, `1` when you aren’t, and `2` when Delta can’t determine your status.
  * `org invite` exits `1` when any invitation fails, even if others succeeded.



Run `delta --help`, or add `--help` to any command, for its full syntax.

## Next steps

  * [Claude Code Plugin](/docs/cli/claude-code): share or join a Claude Code session.
  * [Build a Coding Agent Plugin](/docs/cli/build-a-plugin): connect another coding agent to Delta.



Zed Industries (C) 2026

[Terms](https://zed.dev/terms)[Privacy](https://zed.dev/privacy)Cookies

  1. Install Delta CLI
  2. Update Delta
  3. Open Delta
  4. Find conversations
  5. Read a conversation
  6. Ask about previous work
  7. Share a conversation and its files
     1. Invite people to a thread
     2. Make a thread private
     3. Start a thread from a checkout
  8. Manage organizations
  9. Fork a thread
  10. Send a message between agents
  11. Delete a thread
  12. Work with worktrees
  13. Search connected MCP tools
  14. Use the CLI in scripts
  15. Next steps


