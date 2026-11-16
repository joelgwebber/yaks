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
  2. Build a Coding Agent Plugin



# Build a Coding Agent Plugin

Connect a coding agent to Delta threads through the CLI, using the Claude Code plugin as a reference.

On this page

  1. Connect a session
  2. Send conversation updates
  3. Receive shared activity
  4. Prepare files for joining
  5. Record subagents
  6. Recover safely
  7. Record while offline
  8. Adapt the reference to your host
  9. Next steps



> **Beta:** Plugins use [Delta CLI](/docs/cli/get-started), which is in beta, so the commands and output your plugin relies on can change. The CLI reports which versions of that output it supports, so your plugin can stop and ask for an update instead of breaking. See Connect a session.

Build a plugin that connects a coding agent to Delta, so its users can share sessions and join teammates’ threads the way the [Claude Code Plugin](/docs/cli/claude-code) does. The plugin records the agent’s conversation and files in a Delta thread through the CLI. The open-source [Claude Code plugin](https://github.com/zed-industries/delta-claude) is a complete example.

Delta handles authentication, storage, synchronization, and checkouts. Your plugin is responsible for:

  * Translating your agent’s activity into records and sending them to Delta.
  * Saving records until Delta confirms them.
  * Giving your agent the activity it receives from teammates.
  * Stopping your agent’s turn when someone else takes over the thread.



Use the CLI for everything. Don’t read or write Delta’s database directly.

## Connect a session

Your plugin needs the [Delta CLI](/docs/cli/get-started) and a signed-in account. First, check that your plugin and the installed CLI are compatible:
    
    
    delta --json thread contract

The response reports the contract versions the CLI supports, such as `{"operation":"contract","min":8,"max":8}`. This page describes contract version 8, which Delta 0.22.0 introduced. If `max` is below 8, ask the user to update Delta. If `min` is above the version your plugin was built for, ask them to update your plugin. This version is separate from the `schemaVersion` in each record.

Each session needs two IDs: `--source`, a fixed name for your plugin, and `--external-id`, your agent’s ID for the session. Delta uses the pair to find the session’s thread, so save both with the thread ID.

Run this example from the root of a Git checkout. It requires `jq` and a selected organization:
    
    
    set -eu
    SOURCE=example-agent
    # Use your agent's own session ID, and reuse it when you retry.
    SESSION_ID=example-session-1
    THREAD_ID="$(delta --json thread create \
      --source "$SOURCE" --external-id "$SESSION_ID" | jq -er '.thread_id')"
    printf 'Save this thread ID with your session: %s\n' "$THREAD_ID"
    REFERENCE="$(delta --json worktree create "$PWD" | jq -er '.reference')"
    delta --json worktree attach "$REFERENCE" --thread "$THREAD_ID"
    delta --json worktree mount "$REFERENCE" "$PWD" --existing-checkout > mount.jsonl &
    MOUNT_PID=$!
    delta --json thread share "$THREAD_ID" --visibility organization

Ask before sharing, and tell the user which conversation content and files their organization will see. Save the thread ID and the worktree reference before you attach or share. Running `thread create` again with the same two IDs returns the same thread, so a retry is safe. To reuse a checkout’s worktree on a retry, find its `reference` in `delta --json worktree list` instead of creating another.

  * `worktree attach` only adds the worktree to the thread. It doesn’t upload or sync files.
  * `worktree mount --existing-checkout` uploads the checkout and keeps syncing it for as long as the command runs. It prints one JSON line when it’s ready. Keep it running for the whole session.
  * When the session ends, stop the mount with `kill -TERM "$MOUNT_PID"`. Delta spends a short time uploading changes it hasn’t sent yet, then stops syncing. A mount whose process is gone doesn’t keep Delta running.



To share with specific people instead of the whole organization, invite them in place of `thread share`:
    
    
    delta --json thread invite "$THREAD_ID" ada@example.com

If the thread isn’t uploaded yet, `thread invite` uploads it privately, so only the user and the invited people can open it. The response has a result for each address, and one failed address doesn’t fail the command.

To check access, attached worktrees, and the next sequence number to send:
    
    
    delta --json thread status "$THREAD_ID" \
      --source "$SOURCE" --external-id "$SESSION_ID"

Use this to check access rather than assuming a link grants it.

Reference: [sharing](https://github.com/zed-industries/delta-claude/blob/9ac24a11a82651559cf7c6bc30ccb024aadc96ea/hooks/sharing/share.ts), [CLI compatibility](https://github.com/zed-industries/delta-claude/blob/9ac24a11a82651559cf7c6bc30ccb024aadc96ea/hooks/cli/delta-cli.ts), and [output parsing](https://github.com/zed-industries/delta-claude/blob/9ac24a11a82651559cf7c6bc30ccb024aadc96ea/hooks/cli/thread-commands.ts).

## Send conversation updates

`thread append` reads records as UTF-8 JSON Lines on stdin. Each record has `schemaVersion`, `sequence`, an optional `turnSequence`, and an ACP v2 `update`. Appending records activity. It doesn’t run messages or tool calls. Share the thread before you append to it, or create it with an upload organization as described in Record while offline.

To record the first message in the thread created above:
    
    
    printf '%s\n' \
      '{"schemaVersion":1,"sequence":1,"update":{"sessionUpdate":"user_message","messageId":"message-1","content":[{"type":"text","text":"Please inspect this change."}]}}' \
      | delta --json thread append "$THREAD_ID" \
          --source "$SOURCE" --external-id "$SESSION_ID" \
          --producer-name "Example Agent"

`--producer-name` is the name Delta shows for your agent’s output. Without it, Delta shows `--source`.

Supported updates are `user_message`, `agent_message`, `agent_thought`, `tool_call_update`, `state_update`, and `session_info_update`, which sets the thread’s title.

Streaming chunk updates are rejected. To stream a message, send it again with the same `messageId` and its full content so far. Recording a tool call’s diff doesn’t change files in the checkout.

To record a live turn, start it with a `running` state update whose `turnSequence` equals its own `sequence`. Give the turn’s content and its final `idle` update the same `turnSequence`. Continuing after the record above:
    
    
    {"schemaVersion":1,"sequence":2,"turnSequence":2,"update":{"sessionUpdate":"state_update","state":"running"}}
    {"schemaVersion":1,"sequence":3,"turnSequence":2,"update":{"sessionUpdate":"agent_message","messageId":"answer-1","content":[{"type":"text","text":"The change adds validation."}]}}
    {"schemaVersion":1,"sequence":4,"turnSequence":2,"update":{"sessionUpdate":"state_update","state":"idle"}}

A `state_update` without `turnSequence` is recorded as history. It doesn’t start or finish a turn.

To end the thread’s current turn without starting another, as Delta’s Stop button does, run `delta --json thread stop "$THREAD_ID"`.

Reference: [conversation translation](https://github.com/zed-industries/delta-claude/blob/9ac24a11a82651559cf7c6bc30ccb024aadc96ea/hooks/recording/content.ts) and [initial export](https://github.com/zed-industries/delta-claude/blob/9ac24a11a82651559cf7c6bc30ccb024aadc96ea/hooks/recording/snapshot.ts).

## Receive shared activity
    
    
    delta --json thread watch "$THREAD_ID" \
      --source "$SOURCE" --external-id "$SESSION_ID" \
      --participate "Example Agent"

Passing `--source` and `--external-id` hides the items your plugin recorded. Omit both to see everything. Watching doesn’t create, share, mount, or run anything.

`--participate` shows the user to collaborators as taking part through your agent, under the name you pass, for as long as the watch runs. Without it, the watch is invisible to collaborators.

Keep this watch open while your agent’s turn runs. If your agent’s process exits while its turn is still running, Delta ends the turn when the watch ends, so a crash doesn’t leave the thread busy for everyone. The watch treats its parent process as your agent; pass `--owner-pid` to name another one.

The stream starts with `ready`, then the thread’s current items, then `live`, then changes as they happen. Keep your own copy of the items, keyed by ID:

  * `item` replaces an item; `item.append` extends its text; `remove` deletes it.
  * Order items by `after_id`, retaining unresolved predecessors until they arrive.
  * `turn`, `presence`, and `worktrees` replace their latest state.
  * `producer.turn` reports your turn’s state, including supersession.
  * `comment` and `comment.remove` update delivered comments.
  * `children` replaces the list of subagents and subthreads. It arrives once, after `live`, so don’t wait for it before treating the thread as ready. `child` adds or replaces one entry by `thread_id`, and `child.remove` deletes one.



To follow a child, pass its `reference` unchanged as the thread argument to `watch`, `status`, `append`, or `mount`. Don’t build a reference from a `thread_id`.

Allow new fields and skip unknown event types. Add remote activity to your agent’s context without automatically starting a turn.

Reference: [remote context and display](https://github.com/zed-industries/delta-claude/blob/9ac24a11a82651559cf7c6bc30ccb024aadc96ea/hooks/watch/conversation.ts).

## Prepare files for joining

When a user joins an existing thread, check their access, mount the thread’s files, and record only new activity from then on. Don’t export their previous session, attach its checkout, or share the thread again. Keep the share link for reconnecting, but use the `thread_id` that commands return as the thread’s ID.

Mount all of the thread’s worktrees for your agent session. `THREAD` accepts a share link, a share ID, or a thread ID:
    
    
    THREAD="https://delta.dev/join/thread_..."
    # A stable ID for this session's mounts. Reuse it to recover them later.
    OWNER="$SESSION_ID"
    delta --json thread mount "$THREAD" --owner "$OWNER" > thread-mount.jsonl &
    MOUNT_PID=$!

`thread mount` keeps running for as long as your session needs the files. When the checkouts are ready, it prints one JSON line with the thread’s `directory`, a folder containing a link to each worktree’s checkout, and the checkouts themselves. Move your agent into that `directory`, and keep the command running until the session ends. When worktrees are added to or removed from the thread, the links in that folder change to match.

A `worktrees` watch event means the thread’s attachments changed, not that new checkouts are ready. Start `thread mount` again with the same owner, wait for its ready line, then stop the previous mount process. A `live` frame or an `idle` turn doesn’t guarantee that a teammate’s latest file changes have reached your checkout.

To release the mounts, stop the command:
    
    
    kill -TERM "$MOUNT_PID"

Delta spends a short time uploading changes it hasn’t sent yet, then deletes the checkouts it created. The same happens if your agent’s process exits without stopping the mount. If the upload doesn’t finish, for example because the computer is offline, Delta keeps the checkouts. Run `thread mount` again with the same owner to recover them. Ignored files and local Git state, such as branches, aren’t shared or kept.

Reference: [joining](https://github.com/zed-industries/delta-claude/blob/9ac24a11a82651559cf7c6bc30ccb024aadc96ea/hooks/sharing/join.ts) and [lifecycle rules](https://github.com/zed-industries/delta-claude/blob/9ac24a11a82651559cf7c6bc30ccb024aadc96ea/hooks/sharing/lifecycle.ts).

## Record subagents

If your agent starts subagents, record each one as a child thread under the session’s thread, with the subagent’s own ID as `--external-id`:
    
    
    CHILD_REFERENCE="$(delta --json thread create \
      --source "$SOURCE" --external-id "$SUBAGENT_ID" \
      --parent "$THREAD_ID" --name "Inspect the parser" \
      --shared-worktrees | jq -er '.reference')"

Use the returned `reference` as the thread argument for the child’s `append`, `watch`, `status`, and `mount` commands. Running the command again with the same IDs returns the same child.

  * `--shared-worktrees` gives the child the parent’s checkouts, for a subagent that works in the same folder.
  * `--isolated-worktrees` gives the child private copies of the parent’s worktrees, including uncommitted changes. Mount them with `thread mount`, then merge the child’s changes back with `delta thread merge "$CHILD_REFERENCE" --into "$THREAD_ID"`.
  * With neither option, the child has no checkouts.



A subagent’s prompt usually comes from its parent agent. To show it as sent by the parent rather than the user, add the parent’s external ID to the `user_message` record’s `_meta`:
    
    
    printf '%s\n' \
      '{"schemaVersion":1,"sequence":1,"update":{"sessionUpdate":"user_message","messageId":"prompt-1","content":[{"type":"text","text":"Inspect the parser"}],"_meta":{"delta":{"sender":{"external_id":"example-session-1"}}}}}' \
      | delta --json thread append "$CHILD_REFERENCE" \
          --source "$SOURCE" --external-id "$SUBAGENT_ID" \
          --producer-name "Example Agent"

Recording a child doesn’t start a Delta turn. Your agent still runs the subagent.

Reference: [subagent recording](https://github.com/zed-industries/delta-claude/blob/9ac24a11a82651559cf7c6bc30ccb024aadc96ea/hooks/subagents/subagents.ts).

## Recover safely

  * **Save records before sending them.** Number them consecutively from 1, and never reuse a sequence number for different content.
  * **Retry with the same records.** A command can fail after Delta recorded its records. Resending them unchanged is safe and returns `duplicate`. Delete a saved record only after Delta confirms it.
  * **Expect partial batches.** If a batch fails partway, the records before the failure are still recorded. Check the receipt for each record: `recorded`, `duplicate`, or `fenced`. `fenced` means the record belongs to a turn that someone else took over, and nothing was written.
  * **Read errors from the JSON output.** With `--json`, retry only when `error.retryable` is `true`, waiting `retry_after_ms` when it’s present. Otherwise, pause and show the error. Log `reason`, but don’t make decisions based on it, because its values can change.
  * **Stop your agent when its turn is taken over.** Stop the local turn when a `producer.turn` event reports `superseded` or a current-turn record comes back `fenced`. Keep late output’s original `turnSequence`. Stopping can’t undo what tools already did.
  * **Rebuild your view on reconnect.** Each `thread watch` starts with a fresh snapshot, and there is no way to resume a previous stream. Track what you already gave your agent so a reconnect doesn’t repeat it.
  * **Keep your saved state.** If you lose your saved records or sequence numbers, you can’t restart the same session from sequence 1. Start a new session ID instead, which creates a new thread.



Reference: [outbox storage](https://github.com/zed-industries/delta-claude/blob/9ac24a11a82651559cf7c6bc30ccb024aadc96ea/hooks/recording/outbox.ts), [retry policy](https://github.com/zed-industries/delta-claude/blob/9ac24a11a82651559cf7c6bc30ccb024aadc96ea/hooks/recording/delivery-policy.ts), and [CLI fixtures](https://github.com/zed-industries/delta-claude/tree/9ac24a11a82651559cf7c6bc30ccb024aadc96ea/hooks/testing/fixtures/cli).

## Record while offline

Your plugin can keep recording without a connection to the server:
    
    
    THREAD_ID="$(delta --json thread create \
      --source "$SOURCE" --external-id "$SESSION_ID" \
      --organization ORGANIZATION_ID --visibility private | jq -er '.thread_id')"
    delta --json thread append "$THREAD_ID" \
      --source "$SOURCE" --external-id "$SESSION_ID" --acknowledge local

  * `--organization` and `--visibility` on `thread create` record where the new thread should upload, using an organization ID from `delta org list`. The command returns without waiting for the network, Delta uploads the thread in the background, and you can append to it right away. `--visibility` is `private` or `organization` and applies only to the first upload. Use `thread share` to share a private thread later.
  * `--acknowledge local` returns each receipt once the record is saved on this computer, instead of waiting for the server.



`thread status` also reports `visibility` and an `upload` object. `upload.state` is `queued`, `uploading`, `uploaded`, or `failed`, and `upload.error` explains the last failure.

If the server rejects changes recorded while offline, because someone else changed the thread, appends fail with `reload_required` until the user resolves it. Pause recording and tell them to open the thread in the Delta app and choose **Fork & Restore** or **Discard Changes**. From the CLI, they can run `delta thread fork` and then `delta thread discard --force`.

Reference: the Claude Code plugin’s automatic uploads, in [`activation.ts`](https://github.com/zed-industries/delta-claude/blob/9ac24a11a82651559cf7c6bc30ccb024aadc96ea/hooks/setup/activation.ts) and [`project-policy.ts`](https://github.com/zed-industries/delta-claude/blob/9ac24a11a82651559cf7c6bc30ccb024aadc96ea/hooks/setup/project-policy.ts).

## Adapt the reference to your host

Claude’s function hooks, `/clear`, `/cd`, trust prompt, and transcript APIs are host-specific. Replace them with your tool’s session, cancellation, folder-selection, context, and UI APIs.

Test interrupted sharing, lost receipts, reconnects, late superseded output, and resumed sessions. Test a handoff between two computers: one edits files and finishes a turn, then the other reads those files.

## Next steps

  * [Claude Code Plugin](/docs/cli/claude-code): see the reference workflow from a user’s point of view.
  * [Get Started with Delta CLI](/docs/cli/get-started): the commands your plugin builds on.



Zed Industries (C) 2026

[Terms](https://zed.dev/terms)[Privacy](https://zed.dev/privacy)Cookies

  1. Connect a session
  2. Send conversation updates
  3. Receive shared activity
  4. Prepare files for joining
  5. Record subagents
  6. Recover safely
  7. Record while offline
  8. Adapt the reference to your host
  9. Next steps


