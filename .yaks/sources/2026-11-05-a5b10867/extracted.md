# Juggle less Juggle less Ship more Agents that keep going Share with teammates Send prompt, close laptop Continue from your phone Continue while you sleep Forget worktrees Juggle less

When you give every agent its own orb everything changes. Run agents anywhere and they keep working after you close your laptop.

Working Day to Day in Amp

## Watch Thorsten Build in Amp

- 0:16 An agent that wakes itself to check production

- 4:07 Send a screen recording of a flicker

- 8:14 Ship Changes and watch the rollout

- 11:46 Grant an orb production access from the terminal

- 16:24 Ask for screenshot proof

- 22:52 Cut a release, email the user, close the bug

- 00:00 Introduction

- 00:15 Monitor a Performance Rollout

- 02:03 Review the amp clone Fix

- 03:51 Fix Puck Flicker from a Video

- 05:55 Dictate a Rollout Prompt

- 07:45 Test and Ship in a Portal

- 09:38 Read the Canary Results

- 11:45 Grant Production Access

- 12:47 Check In on Threads

- 16:14 Ask for Proof

- 17:00 Post to Slack with MCP

- 18:31 Verify a Runner Refactor

- 21:48 Close the amp clone Bug

- 23:20 Reply to Slack Feedback

- 24:27 Wrap Up

### 00:00 Introduction

00:00 Hey there, Thorsten from Amp here. I thought, I'm just going to record a quick video and show you how I work in Amp day-to-day. This is what I do. Like this is actual work. So, I was about to send this prompt and I thought, no, no, no, I'm going to show this.

### 00:15 Monitor a Performance Rollout

00:16 So here I'm working on performance improvements. I had a huge thread going that has been shipping production improvements, and then monitoring it. And, you know, you can see here, it uses automations to wake itself up. 35 minutes ago, did a wake up, then it worked. It said both fixes are now live. It compared the numbers. No fleet-wide fleet win visible yet, and I said, 'Hey, check now,' because it says, 'Hey, the next schedule fires.' So I said, like, 'Hey, check now.' It did, it checked. It has access to our production logs. You can see here Google Cloud Grants. We use OIDC, so each Orb has access to production, and we can grant them access or revoke it. And this found out that, well, we have a clear winner here, right? Confirmed fleet-wide win steady state window, boom, boom, boom. So, it used a bunch of canaries, which are things that I shipped yesterday, which are real agents or threads in production that we can use to debug production issues.

01:16 So, here, I said, 'Awesome. How often is that used?' And I can say, 'Does this run on every turn when the agent does a turn?' And then, what I wanted to show you, and I guess, okay, two things. One is, I want like a celebratory canary image. And then the other thing is, look, it says, 'Found along the way, worth a follow-up. A handful of active threads do this, which is an internal method.' And I noticed like, this is one of those that I wrote by hand back months ago, and it says they dominate this, and they're all unrelated. They likely trigger, 'Is it tool progress events that drive...' Want me to spin up an Orb? So I'm going to do this, hit R, and then say, 'Yes, do that.' And now I'm going to submit this.

### 02:03 Review the amp clone Fix

02:03 And now, as you can see, I have a bunch of other threads running here. So let's take a look at those, right?

02:10 Here, this is unread. This started from a bug report. We can see what happened here. So a user reported that amp clone, so basically in Amp, when you create a new project, let me show you this. You create a new project, you can say start from scratch, and then that repo is hosted by Amp. And then you can clone this project by running amp clone. And a user who's using Nix wanted to do this, and says, 'Blah, blah, blah, amp clone tried, Git exited, couldn't unlock config file.' And we basically threw a dumb error, and I asked Amp to fix it. And then it tried to update something with Home Manager in the documentation and whatnot.

02:51 So now I'm just going to look at what it did here. So here, apparently it added this little helper, credential helper, it added a test for those. And then we can look here, ensure Git, so here it uses this helper, the repository was cloned. So here it basically fixes the error message, which is, you know, we had like an internal—we didn't do a good error message. So now I'm going to say, okay, 'Summarize the fix for me, please.' I mean here it says like it treats it as best-effort, passes an empty helper followed by Amp's helper directly to the clone command. If the global setup fails, cloning continues, and Amp prints the original Git error. So I can say, 'So do we only show a better error message, or do we actually change behavior?'

### 03:51 Fix Puck Flicker from a Video

03:50 Let's check what happened here. So I have an experimental version of Puck, which is this, where you have multiple Puck conversations. And if you look at this, that seems flickery. Like look at this. That sucks. So what I did was I recorded a video of this. Here, look at this. And I send it to Amp. And I said look, like this is basically what I just showed you. And I sent this video to Amp, and I said look, this is flickery, moving the cursor feels a bit glitchy. Yes, blah, blah, blah. And then it says, 'It's also the tooltips that make it seem glitchy.' That's what I wrote, right, because what it says was, well, the archive button was removing the hover highlight, which is a different thing that was going on, right? Like if you look at this, like this, you know, this is the row highlight, and if you hover this, it removes this, so that feels glitchy. But that's not really it yet, right? So I said, 'It's also the tooltips that make it seem glitchy. You hover first over archive.' And then it says, 'I'll check, blah, blah, blah.' 'I agree. I added a 500.' And now I'm going to say—oh, by the way, this here is a little experimental feature, Recap. People like it, so maybe we should—we can start a thread and maybe promote it as like a thing that people can activate in the experimental features. Let's do this after this. Okay, so, 'Can I try this out? Can you create a Storybook and start it in a Portal so that I can test what it feels like?'

05:27 I think that whole feature, that sidebar in Puck, that's still behind a feature flag, so I could ship this to production and just ask it like, 'Hey, let me know once it's live in production.' And then I could check it in production, but I think, in this case, it's a frontend-only change. I'll just ask it to whip up a Storybook thing. Okay, apparently we already have one. That's great. And it's going to start it up in a Portal, and then in the Portal, I can try it out.

05:52 So let's do this other thing.

### 05:55 Dictate a Rollout Prompt

05:55 So let's start this. So while this is working here, we using High, which is GPT-6. No big science behind this decision. The only thing is, I've been kind of falling in love with Fable 5.1, but I don't need a lot of creativity here. I don't need like that little sparkle of something else, so I'm just going to set it to High. And then, because you know, what we want to do is a mechanical change. We have this Recap feature and we want to offer it to users in like an experimental setting, right? So I can now write, or let's use dictation, right?

06:33 'Hey, I recently added a feature called Recap. It is behind a feature rollout flag, and a bunch of users already have that flag, and I think the Amp Frontier Corporation workspace also has that flag. I think we can promote this feature to an experimental feature, meaning the user can enable it for themselves in the user advanced experimental feature settings. Please figure out where that feature flag is, figure out how experimental features work in the user settings, and promote this feature to one that a user can enable or disable. And we should add like a little explanatory text to it where we say, hey, this is an experimental feature that adds like a recap of what happened in a thread if you are away from a thread for a few minutes. Additional costs might accrue, I don't know if that's the word, something to that effect.'

07:38 All right, I don't always use dictation, but instead of you watching me type, obviously that's nice, right?

### 07:45 Test and Ship in a Portal

07:45 Now let's take a look at what this did. So now the Storybook is running, we're opening in a Portal pane. Our Storybook is using the Vite dev server, and most dev servers aren't optimized to be run over a network and that is not a local network, so this takes a while. Here, slow to load, 10 seconds. So, by the way, this is also a feature that we might be able to promote. Let's save this screenshot. So let's take a look. Here.

08:17 That feels better, no? That is good. So now we have the delay. You can see, I hover, we have the delay here. There's a delay. Nice. Doesn't feel glitchy. I think that feels good. Compare this to here. Like this, oh my god, this is—but this, this does feel good. All right, so now we say, 'That looks great. Let's ship.' You know what, I'm going to show you something. In the Changes tab, we have this button here, and you can press this to ship stuff. So look.

09:00 So what this does now, it gives the agent instructions to basically run all of the tests, make sure everything works, it should rebase onto origin main, we ship to main, and then here, you see all of these instructions, right? And then it goes live. And now what I can say is, 'Monitor the rollout, and check every 15 minutes, and let me know once this is live.'

09:30 So now, it will do this, like it will ship it, and then it will let me know once it's live, and I do this a lot.

### 09:38 Read the Canary Results

09:36 So, now let's go back to our production performance canaries. Here's the image. Oh my god. So, this—all right. All right. So, these are the canaries merged with Puck, which is quite the—I think there is a law against this. Okay, so what I asked it was, 'Hey, we made a performance improvement, but how often does it run, right? So now it says how often the lookup runs on every tick of the loop. So basically, if you have a coordinator, that's like the agent loop. Every time the agent has to run tool, or has to run inference, that's one tick of the loop. So within one tick, we run it twice, at least twice.

10:21 So per agent, it's several calls. That's why the fleet-wide slow tick dropped 44%. Boom, that's a celebration. Suggested captions now runs on this. And now the hot loop runs here. Sweet. And here's our Recap where it says, 'Goal: improve the thread actor SQLite performance. Latest outcome, unresolved this.' So what I will now do, or would do, is I will say, 'Give me a neat little table with before and after numbers I can share in Slack,' right?

11:01 But hey, everybody loves performance improvements. So underneath it, you can see here this thing already found something. Monospace, okay. Before, look at this, 27 seconds, after 1 second. Before 1.5 seconds, after 0.6. Before, 0.9, okay. Fleet, so that's the fleet, so that's all of them. So these are the slow ticks per minute, and we basically halved it, and then really like p99. That's cool. Fleet numbers exclude a handful of hot looping threads that queue behind themselves, separate fix. All right. So now let's take—what did he write? Like we have two Orbs, right? So what did they do? Let's take a look here.

### 11:45 Grant Production Access

11:46 Fix concurrent this, so this is actually how we give access to these Orbs in production. So they ask us like, 'Hey, give me access to production.'

11:58 I'll try and do this now. I don't know if this is going to work. We might have to cut this out of the video. Let's see. So I copy this, and then here, if I go to Ghostty...

12:18 It worked. I just ran that command in my Ghostty terminal, and as you can see here at the bottom of this command, it says, you know, 'Approve this Orb's access to production.' And then you run amp threads continue and it sends approved. Which is what this is, right? So you can use amp threads continue -ox, ox for the Orb execution, to send an approved. So now it's going to look into the hot threads and going to investigate this, right?

### 12:47 Check In on Threads

12:46 So here, this the other thing we had going, right? Recap is available. So this would promote it, there you go. Preview, get a recap of what happened in thread when you've been away for a few minutes. This experiment may incur additional costs. That's a bad label, right? Can we improve the label? Additional costs sounds scary. Can we somehow get some numbers here, and maybe frame it better? Can we, for example, say something like this feature makes additional inference requests which incur additional costs, but in a neater way? Something like this.

13:39 Yeah, so let's check back in on this one here. amp clone, we actually change the behavior. So this is the amp clone thing that we were looking at. Previously amp clone stopped before cloning if Amp could not write its credential helper. This happens with Home Manager, which is Nix. So it tries to update the global gitconfig, oof, regardless of whether that succeed—oh, I see. So this was the piece I wasn't clear about. So basically, if you run amp clone for the first time, it tries to update the global gitconfig. And regardless of whether that succeeds, Amp passes its credentials to the Git. The clone therefore succeeds with Home Manager. If the global update failed, Amp shows the Git error and explain that—so I think this is good, right? So we don't crash, and instead we show warning here, right? So now I'm going to comment here, 'Don't we have a better way to print warnings than just standard out?' It doesn't matter, I think, with like subcommands, but yeah, maybe there is a better way. Who knows?

14:48 So here, let's take a look.

14:50 So this was the fix for the hover flickering, right? It finished pushing, then check the rollout every 15 minutes, and let you know when it's live. I'll keep this thread open for those updates. So it pushed it, and now it's going to check the rollout every 15 minutes. So next run in 12 minutes, that's when it's going to wake up. We can ask, because this one I want to show you: How are you going to check that it's live, right? This is something we encode in the codebase or in the AGENTS.md file so that the agent knows how to check whether something's live.

15:24 Here, there you go. So we have like an endpoint /_version, which is the server commit. And then it's also going to check CI, and then it's going to check that the code is deployed. So it's going to do this in 12 minutes.

15:35 What else did we do? The Recap experimental settings, we just looked at this, it's still working. This one here is the fixing concurrent build tool plan context, so this is what we just approved. So it's now pulling the logs, right? And it says blah, blah, blah, lifecycle concurrent path, tool—I'm checking the logs. The logs confirm it. Share span with the tool progress. So basically, whenever a tool has progress, like say, in this case like you run a command called gcloud logging, this is communicated back and forth, and I think the problem here is that every time we send a tool progress, we run an expensive query.

### 16:14 Ask for Proof

16:14 So yeah, it's probably going to fix it. So here, amp clone Nix, this is done. Updated it. There you go, see? I was right. We do have chalk, which is, you know, makes the text yellow.

16:24 Here's something cool, because look, like, as you can see, I'm jumping around. I'm not blocked on anything. I'm now going to say, 'Okay, I want you to show me what that looks like. Give me irrefutable proof that this works, and show me a screenshot of that.' So now I want it to basically show me what the error looks like. So I want it to exercise this, right? So this is what I would ask somebody that opens a pull request. I'm like, 'Hey, show me proof that you actually ran through this,' okay?

16:54 So in this case, this is deploying the Puck sidebar, we're still waiting on it.

### 17:00 Post to Slack with MCP

17:00 The Recap experimental setting, get a recap of what happened in a thread when you've been away for a few minutes. In this case, let's do this: 'Hey, send a Slack message to our #wip channel, and write the following: ‘Hey, Thorsten here. I'm recording a screencast where I show how I work in Amp. I'm wondering, what do people think about the Recap feature so far? Should we promote it to be an experimental feature that users can turn on in their settings? Any big concerns, any meh reactions? Please reply here. Sent from my Orb.’'

17:57 So now, we do have the Slack MCP configured, which not everybody can use it. Hopefully, by the time you see this video, you can use it. We're still waiting approval. But now it's going to use the Slack MCP, it's going to send the message to our working progress channel. And then, I don't know, I think it can monitor that thread. I'm pretty sure it can. And now, you know, we sent a message to Slack. Let's see what the other people say. I'm like, the Recap feature, I like it, but I'm not sure whether we should actually make it live or not, or whether we should give it to users.

### 18:31 Verify a Runner Refactor

18:31 Let me show you something else here. So this is maybe interesting. I'm working on the Amp runners, which are, you know, the amp --no-tui. And if I collapse everything, you will see this is a big, big old change. And basically, the gist of it is the amp --no-tui, so this is the runner, this is what I mean, amp --no-tui. That's a thing we already have, you can already use. But right now, it only allows you to spawn threads in a single directory. And the reason for that is that's how the Amp CLI works, or has historically worked. And one of the issues was that I wanted to allow it to spawn threads in different directories, but there's a lot of places, or there were a lot of places, which are removed in that diff here, which we're going to look at in a second, where we call process—you know, process.cwd(), current working directory.

19:26 And this is what this fixed. It's no user-facing change yet. I can show you something maybe here. There you go, look, like this is exactly what I was talking about. Like, getThreadEnv, we do process.cwd() and here we say, give it for the current working directory, blah, blah, blah. And then this should be called somewhere else and it's passed into this, or this function is exported so it's probably used somewhere else, right? So if we look at some other places, headless executor, here working directory, include linked working—like as you can see here, like it's a mechanical change like it's a mechanical change and not just ship some old feature. I basically said, 'Let's refactor. Let's add the capability before we ship any user-facing feature.'

20:15 So, if I scroll up here to my previous message, let's see if we can find it here. 'Can you confirm…' So, here I ask: 'Can you confirm that it didn't break anything?' So, then it checked everything, blah blah blah. It ran through our local dev server, it tested everything. And what did it say? Yes, 'TUI, execute modes, headless all work in the refactored build. I rebuilt this and ran each entry point from against the dev stack with the same prompt. So, with MCP servers, with skills, with AGENTS.md, all three returned this, so this works. Execute mode. So, this works. The TUI works, the headless executor works, static review against the new this. One gotcha worth noting, unrelated to refactor, the first failed with access token expired because Orb…' Okay. So, this is like a little bit moot, except you don't have this when you don't work in Orbs.

21:02 But, so yeah, what I would do now, and we don't have to do this now, is I would look through like the most important files in here—and there are some important files—and do spot-checks of what this does. And then, you know, like I'll think about: is there another way I need to test this? Is there some other danger? Is there something else? But then I would basically ship this. And, you know, it tested it thoroughly. Or maybe I have it ship it to a branch, I'm thinking out loud here, and I test it locally again. But I'm going to go through the code and, you know, try and see what exactly it did. So, here. There you go. 'Recap experimental setting.' Let's say, did someone reply yet or leave an emoji reaction?

### 21:48 Close the amp clone Bug

21:46 amp clone. There you go. Look at this. It produced a nice screenshot. 'I ran real CLI against…' Oh, I see what this is. This is the user who reported this, which is their project. 'So, the evidence show…' 'Git cannot update permission denied.' So, it trying to reproduce the original issue, right? And then the origin is this, head resolves to this. 'The screenshot was rendered from the captured terminal output with the warning styled exactly like this.' So, here basically, look, a bun run build and then, you know, this builds it and runs the CLI. Up here, Git global config. So, this says, 'Warning: the repository is… could not update this. Command failed. Error: configure this.' And so, you know, basically it showed me like how it set up the stuff in Nix. And I think that's pretty good. Like if it can confirm that it actually tested it, we can now say,

22:52 'That sounds great. Let's commit, push this to main, and cut a CLI release. Then monitor the CLI release, wait until it's out, and then let's send the user an email and close the bug report.'

23:10 We have tools in our Amp repo that allow us to send the user an email and close the bug report. And then once this is done, you know, it'll do it.

### 23:20 Reply to Slack Feedback

23:20 So, let's go back to this. Let's see what it did. Oh, 'I love it.' Tim Culverhouse said, 'I love it. I think one change I would make is so that it auto-disappears if I send a new message.' Lewis says, 'I don't really get it. What does it tell me that the last agent message doesn't tell me?'

23:35 Send a message in that thread and say:

23:40 'To Tim: It does auto-disappear if you send a new message. At least it does for me. Show me a video where it doesn't.

23:53 Lewis, it's a shorter recap than usually of a long message. I'm also still on the fence whether it's that,

24:05 but I do like the short bullet points.

24:11 Also, both of you say hi to the camera.'

24:17 I think it will mess this, but hey, I like formatting, so I'm going to do this.

24:22 So yeah, it's going to send this to Slack. Let's see. This is fixing this.

### 24:27 Wrap Up

24:27 This is rolled out. Is it live? 'Next run in 1 minute.' Okay. amp clone, it's rolling this out. It used get_amp_bug_report. It's reading the Slack threads already where this bug was reported. It uses the get_amp_bug_report function, which, again, is an Amp plugin. You can write this yourself for how whatever you want. So, it's now going to cut, you know, here: 'Recent releases take about 7 to 15 minutes.' So, it's going to do this: commit, rerun full checks, push. Once it's pushed, it's going to cut a release. Let's see here. And then what are we doing here? This is the thing. This is ship. Next run in 1 minute. So, what I was about to do was I'm going to right-click and say snooze until the next message, so it disappears from like the sidebar here.

25:13 Yeah, then fix confirmed this. This is still working. Let's check in how it's going. It's still fetching logs, right? It's still doing stuff. It's still confirming this. That's going well. This is the amp clone.

25:26 Yeah, it's all going well. This is how I use Amp all day. I'm in this, and this is how I hop around, how I manage my threads. This is how I start, you know, new threads, develop features, ship stuff, monitor rollouts.

25:41 One last look. Hover tooltips are live. Okay, so now if I hit reload, it should reload. It just did. If I open Puck now, there you go. I hover. No more flicker. Beautiful.

25:54 So, as you can see, this is the life cycle of how I fix and ship and develop new things in here. I hope you got something out of this. Loved to have you here.

26:03 All right. Bye-bye.

## Remote Agents, Controllable From Everywhere

Orbs are remote machines where agents run without supervision. Every thread gets a fresh orb with your code, plugins, and tools, so there are no worktrees to juggle, no port conflicts, and nothing to clean up when it’s done.

Agents in orbs keep working after you close your laptop. Spawn as many as you need, for anything from a tiny fix to a complete feature, and pick them up from the web, your terminal, or your phone.

### Change, Preview and Ship

Run your app inside the orb and try the agent’s changes live through portals, with live reloading. No local checkout, no local dev server.

I'll add both to the hero and check them in the portal as I go.

### Review from Anywhere

Review the agent’s changes and browse the orb’s files from any device, and edit like you’re developing locally.

### Fix Bugs the Moment They’re Filed

Orbs wake on outside events: a new Linear issue, CI failing on main, or a schedule the agent sets itself.

### Multiplayer

Invite your team into the orb to share control of the same agent, code, and running app.

### A Terminal into the Orb

Open a shell inside the orb, shared with the agent. Inspect files, run tests, and debug in the same tmux session.

The parallel case races the temp dir cleanup. I'll await it in upload.ts and rerun.

orbservations

## A Completely New Experience

Something is happening out there. People send one prompt, close their laptop, and come back changed.

> “Building software in anything other than @AmpCode orbs feels like eating broken glass in comparison. But it’s frustratingly one of those ‘you can’t understand why it’s better until you just use it’ tools, a lot like Tailwind was. Use it, and be ruined.”

> “Local was getting cumbersome with workspaces and multiple agents. … Stuck with Amp because it was able to help me get my workspace scripted and working well … Orbs seems a bit smarter and less locked down. Feels more like having an actual computer in the cloud vs. a restricted sandbox.”

> “it is indeed Orb'in time. the @AmpCode folks have cooked. Making new orbs is so easy and really unlocks a lot of the multi-agent behaviors that had just a tad too much friction before.”

> “Amp with Orbs feels significantly above the current state of agents in the cloud, much like the jump from Opus 4.1 to 4.5 from late last year.”

> “never imagined I'm gonna love using cloud agents that much until I used @AmpCode's Orbs. … Orbs was the only concept that felt intuitive from minute one. shipped more side work in one session than I usually do in weeks.”

> “I was at a meeting yesterday approving pr's, looking [at] screenshots and videos showing me implementation worked, fixing ci, improving test coverage and so much more. And my pc was off. All in orbs.”

> “Orbs gave me same goosebumps and feeling when I first used ChatGPT. … One of my orbs is working for over 4 days now, coordinating with other orbs, sleeps when it offloads the work and picks it up when needed.”

> “we spent longer than we should’ve setting up Cursor cloud agents earlier this year … and then Amp comes out with orbs and they Just Work™”

> “Going to sleep. Laptop asleep. Agents still running. @AmpCode Orbs are the future.”

> “It’s not the harness, it’s the workspace and the VMs. It’s cloud agents designed and exposed in a way that is an absolute frictionless delight to work with.”

> “@AmpCode orbs feel like the equivalent of early claude code, where a paradigm shift is clearly on the way. imo it's not long before you don't have to worry whether you're developing on a local machine or not”

> “I’m at the “do I even need a computer?” stage of using @AmpCode”

> “It's orbin' time. I think I'm ready to move most of my project work to @AmpCode. I'm even writing some custom plugins and custom infra around it to better manage my orbs.”

> “Cloud coding agents often make their infrastructure impossible to ignore. Amp Orbs did the opposite. They made the infrastructure disappear so I could focus on shipping the change.”

> “orbs have completely changed the way I work. I can't imagine going back to working locally at this point.”

> “Orbs have been a game-changer for how I work. I can fire off multiple agents walking down the street and check on them when I get to the pub!”

> “I was also pretty shocked at how fast it ran everything. Idk how they’re doing that”

> “Tried @AmpCode's Orbs. Super good shit. Not unlike what [Ona] does but with the usual Amp team polish and a few additional concepts on top.”

> “First time to do a straight push to main via @AmpCode orbs. Review the code change, test the app via the portal, review the code with the Oracle, ask screenshots for proof of work. All these from my phone. Still in the loop.”

> “been using [Amp] as my main for a week now, it's really nice. first harness that made agents+vms click for me”

> “Orb unifies local and remote environment controls, allowing you to control anything from any device. It made my iPad useful again.”

> “Ephemeral sandboxes are a big deal because they change the failure mode. The agent can do real work without turning your main environment into the test surface.”

> “Orbs are a fantastic addition. The execution and integration of local and cloud development are truly great.”

> “Having the agent finding the bug, then record a video of a bug, this is a new addition to my toolbox. Thanks @AmpCode for removing the friction of such scenarios with orbs.”

> “I was really hesitant to try out @AmpCode's Orb feature ever since I had an unsatisfactory experience with Cursor cloud agents. But damn, it was so easy to setup, cuz there's literally nothing to setup lol. … I've basically made up my mind lol, not going to be running agents locally anymore.”

> “Our team is loving amp right now, specifically how their orbs work. We're seeing a ton of movement bringing things off your local and into the cloud.”

> “love @AmpCode, i have shipped more code in the last 24 hours than I did all last week. never go back to local dev.”

> “currently sitting in a chair rocking my youngest to sleep for a nap watching 3 orbs work from my phone”

> “I am using amp and orbs just a few weeks and I feel like: Amp & Orbs are built by humans… I can feel human touch and craft from using it, less features but better quality.”

09 · Bring ChatGPT

## Make More of Your ChatGPT Subscription

Bring ChatGPT to Amp

### Take Your Tokens Further

Link your ChatGPT subscription and the low, medium, and high settings of the Dial run on it instead of Amp credits.

Used For

- Main Agent

- Subagents

- Thread Reader

- Code Review

Your Amp tier still includes agent usage and orb time. Ultra continues to use Amp credits.

## Orbs from Zero

Thorsten takes a real project from a GitHub repository to a codebase agents can set up, run, review and test in orbs.

- 14:38 Get Started in Orbs Create a project from a GitHub repository, let Amp prepare it for orbs, and watch the first orb set up, run and snapshot the app.

14:38

### Get Started in Orbs

Create a project from a GitHub repository, let Amp prepare it for orbs, and watch the first orb set up, run and snapshot the app.

- 6:08 Review Your App Through Portals Open the dev server in a portal, comment on the page itself to send the agent changes, and check the result live.

6:08

### Review Your App Through Portals

Open the dev server in a portal, comment on the page itself to send the agent changes, and check the result live.

- 4:39 Add a Dev Sign-In for Agents Give agents and portal viewers a development sign-in route so nobody types a password in an orb again.

4:39

### Add a Dev Sign-In for Agents

Give agents and portal viewers a development sign-in route so nobody types a password in an orb again.

- 7:59 Using Orbs to Eval Agent Friendliness Fan out fresh orbs with no hints to find out whether your AGENTS.md and setup are enough for an agent to get to work.

7:59

### Using Orbs to Eval Agent Friendliness

Fan out fresh orbs with no hints to find out whether your AGENTS.md and setup are enough for an agent to get to work.

Install Amp CLI

## Also: Agents on Your Machines

### On Your Machine

Run amp in your terminal with your local files and tools, or amp sync to mirror an orb’s changes into your local checkout while the agent keeps working remotely.

### On All the Other Machines

Run amp --no-tui on servers, CI runners, and anything else with a shell, then watch and steer those agents from the web or your phone, right next to your orbs.

News

## Announcements of Amp

### Service Accounts

Service accounts for threads that belong to the team, not to one person.

### Artifacts

Documents that belong to a thread, which you, your teammates, and the agent edit together.

### macOS Satellites

Your orb can now start a Mac to build, test, and run your iOS and Mac apps.

### Visualize This

Amp can now show you charts, timelines, diagrams, and mockups right in the thread.

### Your Own Self-Hosted Orbs

Add your own self-hosted orb provider, no workspace admin needed.

### Amp on Android

Threads, Orbs and Puck in your pocket. Now on Android.

- All Systems Operational

- Security

- Privacy Policy

- Terms of Service

- Your Privacy Choices

### Product

- Download App

- Start

- Sign In

- Docs

- What Are Orbs?

- Models

### Resources

- About

- Chronicle

- Notes

- Pricing

- Podcast

- Press Kit

### Guides

- How to Build an Agent

- Context Management

### Community

- 𝕏 @ampcode

- YouTube