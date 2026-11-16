[](/)

  * Product
  * Resources
  * [Extensions](/extensions)
  * [Docs](/docs)
  * [Pricing](/pricing)
  *   * [Delta](https://delta.dev)



  * `P`
  *   * [Sign up`S`](/sign_up)
  * [Download`D`](/download)



# Software Is Made Between Commits

[Nathan Sobo](/team#nathan-sobo)

June 11th, 2026

## On This Page

  * Every operation, not just every commit
  * Source code is now source conversation
  * You shouldn't need to commit to collaborate
  * What comes next



## See what's next:

[Delta Roadmap →](https://delta.dev/roadmap)[Zed Roadmap →](/roadmap)

I have never been a big fan of pull requests.

Before agents, it was easier to believe that the ceremony of trading comments on snapshots was an effective way to collaborate on software, but it never really worked for the Zed team. We frequently work together in the same worktree, building trust and shared understanding by discussing the code _as we write it_. GitHub doesn't let you talk about code until after you commit and push, but by then our most important conversations are usually already over.

So in 2021, we founded Zed to move beyond the constraints of commits. Our plan was to [build an editor worthy of the world's best developers](/blog/zed-1-0), then offer a better way to work together inside it. We didn't foresee then how the problems we'd spent years thinking about in the context of human-to-human collaboration would become even more important when collaborating with agents.

Increasingly, the conversation that generates the code is becoming the true source of our software. That conversation unfolds continuously and must be cross-referenced to the code as it changes. Git, organized around discrete commits, was never designed to support this.

So we're building something that is. We call it DeltaDB, a new kind of version control built on a single coherent abstraction that transforms your conversations with agents and the worktrees they edit into shared artifacts. We've made a ton of progress [since I first spoke about it last fall](/blog/sequoia-backs-zed), and with a [beta version](/deltadb) ready in a few weeks, I'm excited to share more about what we're launching.

## Every operation, not just every commit

DeltaDB breaks your work into a stream of fine-grained _deltas_. Where Git captures a snapshot at each commit, DeltaDB captures every operation in between and gives each one a stable identity. Because every delta can be addressed on its own, you can point to the code at any moment in its evolution, even as it keeps changing. That lets us version a worktree as it evolves, together with the conversation driving it.

A message and the edit it produced are recorded side by side, so neither drifts away from the other. Because DeltaDB embeds conflict-free replicated worktrees, many people and agents can edit the same files at once across different machines. The files are real: agents work in them through a terminal, and you can mount the whole worktree to disk whenever you want your own tools on it.

## Source code is now source conversation

Because every reference is anchored to a delta instead of a line number, it survives as the code moves underneath it. From any line in a past conversation, you can jump to that code as it stands now or as it stood the moment the agent wrote it. From any line of code, you can find the conversation that produced it and every conversation that has touched it since.

Agents can draw on it too. They pick up the context behind the code they're touching or convene the prior agents that worked on it and ask why it's written the way it is.

## You shouldn't need to commit to collaborate

What we're really after is simple: the conversation with the agent becomes the only conversation you need to have. A teammate can join while the work is still happening, talk to the agent that did the work, and annotate as they go, without waiting for you to commit and push first.

Pull requests, review threads, and inline comments exist to reattach a discussion to code after the fact because the discussion and the code lived in separate places. Put them in the same place, and the ceremony disappears. Git and CI stay for what they're good at: running checks and connecting you to the rest of the world, rather than being the place collaboration is forced to happen.

## What comes next

Software now takes shape in the conversation, not the commit. DeltaDB is the version control built for that, and in a few weeks we'll start putting it in the hands of early users.

If you'd like to be among the first to try it, [join the waitlist](/deltadb).

* * *

* * *

### Related Posts

Check out similar blogs from the Zed team.

## [We're Not Building AI Features for the Money

* * *

|Featured|May 05, 2026

* * *](/blog/not-building-ai-for-the-money)## [Introducing Parallel Agents in Zed

* * *

|Featured|Apr 22, 2026

* * *](/blog/parallel-agents)## [Introducing Zed AI

* * *

|Featured|Aug 20, 2024

* * *](/blog/zed-ai)

* * *

### Looking for a better editor?

You can try Zed today on macOS, Windows, or Linux. [Download now](/download)!

* * *

### We are hiring!

If you're passionate about the topics we cover on our blog, please consider [joining our team](/jobs) to help us ship the future of software development.

### Zed Industries © 2026

* * *

[Sign Up](/sign_up)•Have an account? [Sign In](/sign_in)

[Terms of Service](/terms)•[Attributions](/attributions)

Manage Site Cookies

### Product

  * [Download](/download)
  * [Pricing](/pricing)
  * [Business](/business)
  * [Releases](/releases/stable)
  * [Extensions](/extensions)
  * [Roadmap](/roadmap)
  * [Docs](/docs)
  * [Compare](/compare)
  * [GitHub↗](https://github.com/zed-industries/zed)
  * [Status↗](https://status.zed.dev)



### Resources

  * [FAQ](/faq)
  * [CLA](/cla)
  * [Community](/community)
  * [Support](/support)
  * [Discussions↗](https://github.com/zed-industries/zed/discussions)
  * [Privacy](/privacy)
  * [Subscribe](/subscribe)
  * [Theme Builder](/theme-builder)
  * [Yearly Recaps](/2025)



### Company

  * [Blog](/blog)
  * [About](/about)
  * [Values](/values)
  * [Team](/team)
  * [Jobs](/jobs)
  * [Press](/press)
  * [Brand](/brand)
  * [Merch↗](https://merch.zed.dev)



### Social

  * [Twitter↗](https://twitter.com/zeddotdev)
  * [Bluesky↗](https://bsky.app/profile/zed.dev)
  * [Instagram↗](https://www.instagram.com/zed.industries/)
  * [YouTube↗](https://www.youtube.com/@zeddotdev)
  * [Discord↗](https://discord.com/invite/zedindustries)
  * [Reddit↗](https://www.reddit.com/r/ZedEditor/)


