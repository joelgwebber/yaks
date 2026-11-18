---
type: source
title: 'Delta CLI product page: bring Delta to the agent you already use'
description: Saved copy of https://delta.dev/cli
media_type: text/markdown
size: 7244
sha256: 0df8e0eb8f4f5adf1df94b86acc8d31999c27a1f814a6374a444a96fbd11004e
sources:
  - id: web
    resource: https://delta.dev/cli
    host: joel
    last_modified: 2026-11-16T21:32:55Z
    retrieved: 2026-11-16T21:33:21Z
generated:
  by: delta:ksQQM-tlBqyPR_q2SQ147vmMIZLOAGl2rZIBxBCA
  at: 2026-11-16T21:33:21Z
---

Skip to content

[](/)

[](/)

Now in Beta[Home](/)[Download](/download)[CLI](/cli)[DeltaDB](/deltadb)[Docs](/docs/getting-started)[Roadmap](/roadmap)[Pricing](/pricing)[Blog](/blog)

Delta CLIBeta

# Bring Delta to the agent you already use

The Delta CLI syncs your agent sessions to DeltaDB. Share one from Claude Code, and your teammate joins from their own session with the conversation and your uncommitted changes in front of them.

[Get started`G`](/docs/cli/claude-code)[Learn more`L`](/blog/claude-code-multiplayer)

macOSLinuxWindowsDelta app
    
    
    curl -fsSL https://delta.dev/install.sh -o delta-install.sh
    sh delta-install.sh

❯ /delta-setup

/delta-setupchoose which sessions sync to DeltaDB

❯ /delta-setup

☐ Delta sync

Delta isn't set up for this repository yet. Delta stores the conversation and a copy of this checkout, including uncommitted changes, in DeltaDB. Which sessions should Delta sync?

  1. 1\. Private sync (Recommended)
  2. 2\. Organization sharing
  3. 3\. Manual (only sessions you /delta-share)



Enter to select · ↑/↓ to navigate · Esc to cancel

○ Delta ·link appears when readySyncing…

❯ Try "write a test for register.tsx"

▸▸ auto mode on (shift+tab to cycle)

● Delta ·delta.dev/join/thread_01s5e5he446jdgzyk4✓ Synced

❯ i wanna understand how we handle context windows in this plugin when reading a really long delta thread from a new /delta join

▸▸ auto mode on (shift+tab to cycle)

❯ i wanna understand how we handle context windows in this plugin when reading a really long delta thread from a new /delta join

● Bash(git ls-files | grep -rniE "join" -l .)

⎿ .claude-plugin/plugin.json

hooks/register.tsx

… +98 lines (ctrl+o to expand)

✻ Gallivanting… (6s · ↓ 244 tokens)

❯ i wanna understand how we handle context windows in this plugin when reading a really long delta thread from a new /delta join

● When you /delta join a long thread, Claude gets about 100k tokens of the newest history, and the rest is left out behind a pointer row. Nothing is summarized.

How it works

1\. Join only sets things up. It runs /clear, mounts the thread's worktrees, and never puts history into the prompt.

2\. The watcher reads the thread. Every item already in it arrives as one snapshot.

✻ Cooked for 1m 2s

● Delta ·delta.dev/join/thread_01s5e5he446jdgzyk4

DeltaDB, now in the CLI

↻ Replay

## Connect your preferred harness

Starting with Claude Code. More coming soon.

[Read the docs](/docs/cli/get-started)[Set up Claude Code](/docs/cli/claude-code)

  1. 1

### Share from Claude Code

Install the plugin and send a teammate the link, or join the thread yourself on another machine.

You
         
         # Install the plugin
         claude plugin marketplace add zed-industries/delta-claude
         claude plugin install delta@zed-industries
         
         # Share the session you're in
         /delta share

  2. 2

### Your teammate joins

Your teammate's agent gets what you asked, what Claude tried, and your uncommitted changes. No summary required.

Your teammate
         
         # Pick up where you left off
         /delta join <link>

  3. 3

### Ask about any change, any time

Search past threads, then get the reasoning from the conversation that produced it.

Anyone on the team
         
         # Search messages, summaries, and the files an agent edited
         delta thread search sign-in
         
         # Get an answer from past transcripts
         delta thread query --keyword sign-in "What did we decide?"

  4. 4

### Build on it

Search, query, and fork threads from your shell. Scripts and agents can run the same commands.

You or your agent
         
         # Try another approach without touching the original
         delta thread fork $THREAD_ID
         
         # Get a local checkout you can edit with any tool
         delta worktree mount $REFERENCE ~/checkout
         
         # Publish a thread to your organization
         delta thread share $THREAD_ID --visibility organization
         
         # JSON on stdout, diagnostics on stderr
         delta --json worktree list

  5. +

### Bring your own harness

Pi support is next. The plugin is open source, so you can build your own.

Claude CodePi (Soon)Your Harness
         
         # Start from the Claude Code plugin
         git clone https://github.com/zed-industries/delta-claude

[Build a plugin](/docs/cli/build-a-plugin)[View the source](https://github.com/zed-industries/delta-claude)




## Learn More

### [Get started with the Delta CLI Install it, sign in, and run your first commands.](/docs/cli/get-started)### [Claude Code plugin Share a Claude Code session and join a teammate's.](/docs/cli/claude-code)### [Build a coding agent plugin Connect another harness to Delta.](/docs/cli/build-a-plugin)### [Scripts and automation JSON output, exit codes, and background mode.](/docs/cli/get-started#use-the-cli-in-scripts)

## Built on DeltaDB

Git records a snapshot each time you commit. DeltaDB, the version control underneath Delta, records every step in between, so a teammate can join work you haven't committed yet.

[Learn about DeltaDB](/deltadb)

Git stops at the last commit. DeltaDB keeps every step after it, so a teammate can run /delta join and pick up work that isn't committed yet.GitA snapshot at each commitNot committed yetDeltaDBEvery step in between4a7c2e19d03b5f/delta joinYour teammate picks up here

## Frequently asked questions

Do I need the Delta app?

The CLI runs on Delta, so the installer downloads it into your home folder: ~/Applications on macOS and ~/.local/delta.app on Linux. You never have to open it. The CLI starts Delta in the background when a command needs it. If you already use Delta on a Mac, choose Delta > Install CLI… from the menu bar instead.

Which agents does it work with?

Claude Code, in a terminal or in the Code tab of the Claude desktop app. Support for Pi is in progress. Claude Code cloud sessions, print mode, and SDK sessions aren't supported yet.

Can teammates join if they don't use Claude Code?

No. Teammates who use Delta can join the same thread from the app or a browser, and work with the same conversation and files.

Who can see what I share?

Sharing publishes the thread and its files to your whole Delta organization, including people you didn't send the link to. You can also set up a repository so sessions upload privately and stay visible only to you until you share them.

What gets shared?

Your messages and the agent's replies, tool calls with their inputs and outputs, images, and your checkout's files, including staged, unstaged, and untracked changes. Delta doesn't share extra `--add-dir` folders, Git-ignored files, local branches, unsent prompts, or messages from earlier or compacted sessions.

How does Delta work with Git?

Sharing starts from a Git checkout with a remote, and you keep committing and opening pull requests as you do today. Delta covers the work that happens before a change is ready to commit.

[Download Delta Now`D`](/download)[Open Delta`D`](/threads)

Zed Industries (C) 2026

[Terms](https://zed.dev/terms)[Privacy](https://zed.dev/privacy)[Brand](/brand)Cookies
