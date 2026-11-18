---
type: source
title: 'Syntax #1029 transcript: The Workflow of the Future With Zed (Nathan Sobo, 2026-08-12)'
description: Saved copy of https://syntax.fm/show/1029/the-workflow-of-the-future-with-zed/transcript
media_type: text/markdown
size: 63174
sha256: 34edbef83641bcadda49bf64de12921f5355d2aa817328cf5786b8d5b0292ae3
sources:
  - id: web
    resource: https://syntax.fm/show/1029/the-workflow-of-the-future-with-zed/transcript
    host: joel
    last_modified: 2026-11-16T21:33:00Z
    retrieved: 2026-11-16T21:33:26Z
generated:
  by: delta:ksQQM-tlBqyPR_q2SQ147vmMIZLOAGl2rZIBxBCA
  at: 2026-11-16T21:33:26Z
---

Skip to main content

[Syntax](/ "Syntax Podcast Home")

[Shows](/shows) [Video](/videos) [Newsletter](/snackpack) [About](/about) [Potluck Qs](/potluck) [Swag](https://sentry.shop) Search

`⌘K`

Menu

August 12th, 2026 × #Zed#AIAgents#Rust#VersionControl

# The Workflow of the Future With Zed

Zed cofounder Nathan Sobo unveils Delta and DeltaDB, a collaborative agent editor where conversations are the source and code is a downstream artifact. Plus Atom's death, GPUI, Rust compiled to Wasm, and some seriously cultured jazz picks.

[Nathan Sobo](/guest/nathan-sobo) Guest

[Nathan Sobo on X](https://x.com/nathansobo) [Nathan Sobo on GitHub](https://github.com/nathansobo)

Wes Bos Host

[Wes Bos on X](https://x.com/wesbos) [Wes Bos on GitHub](https://github.com/wesbos)

Scott Tolinski Host

[Scott Tolinski on X](https://x.com/stolinski) [Scott Tolinski on GitHub](https://github.com/stolinski)

#1029

Play Play Episode 1029 or [](https://open.spotify.com/search/syntax.fm The%20Workflow%20of%20the%20Future%20With%20Zed/episodes "Watch or Listen on Spotify") [](https://www.youtube.com/watch?v=GsLyhrxaMIo "Watch on Youtube") [](https://podcasts.apple.com/ca/podcast/syntax-tasty-web-development-treats/id1253186678 "Listen on Apple Podcasts")

[](https://traffic.megaphone.fm/FSI5646050512.mp3 "Download Episode") [](https://github.com/syntaxfm/website/tree/main/shows/1029 - The Workflow of the Future With Zed.md "Edit Show Notes")

[Show Notes](/show/1029/the-workflow-of-the-future-with-zed) [Transcript](/show/1029/the-workflow-of-the-future-with-zed/transcript)

  * 00:00 Conversations are the new source code
  * 00:36 Meet Nathan Sobo, the Zed cofounder and CEO
  * 01:13 Obsessed with the ultimate dev tool since 2006, plus pair programming at Pivotal Labs
  * 03:37 From Atom and Electron's devil's bargain to going closer to the metal with Rust
  * 06:38 Sentry sponsor read: catch every error in your app
  * 07:18 Building a UI framework from scratch with shaders and zero lag keystrokes
  * 10:43 GPUI is open but Zed moves too fast for an ecosystem
  * 11:44 Atom nostalgia, Sublime loyalty, and why VS Code won thanks to TypeScript and LSP
  * 14:16 Would he pick Rust again? Yes, except the compiler is slow
  * 15:58 How the Zed team uses AI, reviews PRs, and closes slop
  * 18:36 Do we even need a text editor anymore? Text is more powerful than ever
  * 21:52 Enter Delta and DeltaDB: version control below the commit
  * 26:45 Conversations become the source and code becomes a downstream artifact
  * 27:31 Shared agent threads with conflict-free synced work trees and model blame
  * 29:06 Does Delta replace GitHub or Claude Code? Not yet
  * 34:35 Pair prompting and fighting decision fatigue
  * 36:58 How virtualized work trees and a custom sync engine make branches free
  * 40:51 Centralized hosting at delta.dev and Rust UI compiled to Wasm in the browser
  * 44:38 Will UI just be language? Why graphics still matter
  * 49:08 Conversations as editable documents you can annotate anywhere
  * 51:45 Early access, beta launch, and Wes gets invited
  * 52:54 Sick picks: Keychron Q11 split keyboard and Blue Note jazz with Grant Green
  * 58:44 Shameless plug for delta.dev and a Colorado meetup



**Topic 0** 00:00

#### Conversations are the new source code

0:00

Guest 0

Our our vision is really it was always conversations in the Node, linked to the code rather than snapshots of the code. But what's changed in the past two, three years and really acceleratingly, like, since last fall, at least for people that write Rust, is that now conversations are becoming sort of the source of our code. Like, we were thinking we'd write all the code and have conversations in it. Now conversations are the beginning, and code comes out of that. It's a downstream artifact.

0:35

Scott Tolinski

Welcome to Syntax. Today, we have Nathan Sobo from Zed on the show.

**Topic 1** 00:36

#### Meet Nathan Sobo, the Zed cofounder and CEO

0:42

Scott Tolinski

Nathan, previously from Warp and then, Wes also at GitHub for nine Yarn, is now the, cofounder and CEO of Zed, the text editor that I use every single day, one of my favorite, applications on my computer.

0:59

Scott Tolinski

So, Nathan, welcome to Syntax.

1:01

Guest 0

Thank you very much.

1:03

Scott Tolinski

Would you like to, get into I know I gave a little bit of a backstory on who you are, but maybe, like, you could dive in just a little bit more of how you ended up at Zed.

**Topic 2** 01:13

#### Obsessed with the ultimate dev tool since 2006, plus pair programming at Pivotal Labs

1:13

Guest 0

Yeah. I mean, I have been obsessed with the idea of building the ultimate tool for software development, basically, for myself for a very long time. I think it was right when I graduated college, the year after 2006, that, you know, Node been using TextMate, which was hot and new at the time. I'd used Eclipse. I'd used eMax.

1:38

Guest 0

Hadn't used Vim at that point, although that did come along, later.

1:43

Guest 0

And I just wanted to build a tool that combined all the things I loved about these various editors, into one product. And I completely underestimated how difficult that would be to do.

1:58

Guest 0

And it took kind of until now, I think, to really even begin to get close, to yeah.

2:07

Guest 0

Basically, bringing the speed of TextMate with the features of something like Visual Studio Node, and then also bringing this collaborative dimension that came along eventually. And that was based on my time at, this company, Pivotal Labs, which, was a pair programming shop.

2:25

Guest 0

You would show up, and there'd be nine to six with a one hour one hour lunch break just, pairing all day long. Wow. Really? And back then, we would plug two keyboards into the same machine, two keyboards, two mice, and just, like, work through problems together, code together.

2:44

Guest 0

And that taught me so much. I got I became such a better software developer so rapidly.

2:50

Guest 0

But I always wanted a tool that could replicate that experience that we had in this shared office, like, over the Internet and turn the Internet into that whole experience.

2:59

Guest 0

And Zed is as close as I've ever gotten,

3:04

Scott Tolinski

until what comes next. Yes. Yeah. Yes. So I can't wait to get into what comes next because I I got a a preview of it, and and it it definitely, like, takes it makes sense to hear you say that about pair programming given, what I have seen in this. So that that's really super, super cool. So you you picked that, and was Rust, because that is built on Rust, right, from the ground up.

3:30

Scott Tolinski

Was Rust the choice initially, or did you land on Rust somehow?

**Topic 3** 03:37

#### From Atom and Electron's devil's bargain to going closer to the metal with Rust

3:37

Guest 0

Well, once upon a time, I, talked the founders of GitHub into hiring me to build an editor called Adam. Yes. Some people may remember. Oh, yeah. Process of creating Atom, we, also created this framework called Electron, which is still around and probably running on all of our machines in some shape or form, eating way more RAM than the apps that it's running really justify.

4:04

Guest 0

So, yeah, that Adam the big idea with Adam was we wanted to make it hackable, and we wanted to wanted it to run on every major platform.

4:14

Guest 0

And so it really seemed like if we could build on web technology, which was already something people were familiar with, was really a flexible technology. You can load CSS, and everybody knows JavaScript.

4:26

Guest 0

If we could just figure out a way to masquerade a web browser as a desktop app, we would sort of achieve our goal of this flexible, hackable system that ran on all the three platforms.

4:40

Guest 0

And we did. But, unfortunately, the trade off that we made, the devil's bargain that we made was that, the overhead of the browser and sort of the abstraction that it presents, which is this kind of lowest common denominator, common standard that needs to run-in a backward compatible way on all these different browsers. Well, although that is reducing down, but still, it's like it it really put a a limit on the performance of and the user experience that we could yield. And so after Adam sort of ran its course and Microsoft acquired GitHub and, I was sitting there using Visual Studio Node, I just wasn't happy. Like, I just couldn't see a future where I continued writing all my software in this web browser masquerading as a desktop app.

5:36

Guest 0

Just what the performance wasn't there. The aesthetics weren't there for me. This collaborative dimension, even though they did introduce Live Share, like, they it never really worked as well as I wanted it to.

5:50

Guest 0

And so that led me to one inexorable conclusion that we had to get closer to the metal.

5:56

Guest 0

And, luckily, around that time, Rust, you know yeah. It was there were we were working on it for quite a while before we started Zed. And, you know, as we worked on it, Rust hit one point o, async await, got introduced, like, Rust really matured as a a viable option.

6:16

Guest 0

And, you know, after being covered in blood from learning the language, we were finally traversed the learning curve and and figured out how to build a UI framework, which really didn't exist at the time.

6:29

Scott Tolinski

And if you want to see all of the errors in your application, you'll want to check out Sentry at sentry.io/syntax.

6:37

Scott Tolinski

You can sign up today and get two months for free. Sentry is just a really incredible tool for not only tracking your performance, making sure application has no bugs, but even just seeing what goes wrong when something goes wrong because things go wrong all the time when we're coding. And you don't want a production application out there that, well, you have no visibility into in case something is blowing up, and you might not even know it. So head on to dosentry.io/syntax.

**Topic 4** 06:38

#### Sentry sponsor read: catch every error in your app

7:02

Scott Tolinski

Again, we've been using this tool for a long time, and it totally rules. Alright.

7:07

Guest 0

I didn't even know how to begin building a UI framework because if you've tried Rust, you know the the ownership rules, the borrower checker.

7:15

Guest 0

There's quite a few constraints.

7:17

Guest 0

It was really relearning how to do desktop app development, from from the beginning and kinda reinventing it. Yeah.

**Topic 5** 07:18

#### Building a UI framework from scratch with shaders and zero lag keystrokes

7:26

Wes Bos

Like, can we talk about that? The the UI thing just blows my mind. Because you think if you're gonna build a text editor, something as simple as, like, line wrapping is is a pain in the and pain in the butt. Right? And then, like, like, a text editor has, like, underlines and things that are overlapping. And if you look at at Versus Code, they throw a lot of that out of the window as well. A lot of it is just absolutely positioned over top of each other in in many layers.

7:53

Wes Bos

That seems really hard. So, like, why would you think that's a good idea to to build it from scratch? Because just it's not like there's some there was something already there where it's like, here JS a text Bos. Here is a background. Here is, like, a text Node that you can use. Right?

8:09

Guest 0

Yeah. I mean, we started with well, I tried a a couple different approaches. Like, at first, I tried, oh, maybe we could use Rust in the core of this thing, and then we'll wrap it in Electron. And so all the Yeah. All the really heavy parts will be in native code, and then I won't have to solve the UI problem. But it was just, like, performance went out the window as soon as we had to deserialize JSON coming out of this Mhmm. Native core or whatever. So finally, I got to the point where I'm like, okay.

8:36

Guest 0

It's a metal graphic surface.

8:39

Guest 0

Mhmm. And, you know, back then, we didn't have agents to just tell us how to do it. It was like digging up tutorials from the guy that wrote shader toy Yeah. On YouTube and stuff, learning how to write, you know, very primitive shaders, learning how to build a render a renderer for a two d app.

9:01

Guest 0

And then, of course, all the CPU bound stuff that, was designed to deliver the data to the graphics processor.

9:09

Guest 0

But, like, for me, the key insight was just, like, all those layers and all that all that abstraction that the web developed to give people something to program against that, you know, wasn't a shader, comes at a real cost. Right? Like and, ultimately, all those things that I felt like warp getting in the way.

9:32

Guest 0

So by bypassing all of that and going straight from our program to the graphics processor, we really reduced the problem down, from this sort of I'm staring at the dev tools in Chrome, like, looking at the profiler, looking at all these little slices of time that I have no idea what's going on inside of and watching the garbage collector run, and all of this nonsense style recalculation and reflows and just what you know, having no control to having complete control.

10:03

Guest 0

And, ultimately, I I decided that, like, even though it was really hard and we had a lot to do, if we had control, we would be able to deliver.

10:13

Guest 0

And the goal was you receive a keystroke, and you have pixels on the next v sync of the display, so there's zero perceptible lag. Wow. And I knew if we had the control, we'd be able to achieve that goal.

10:28

Guest 0

And what is it? Six years later?

10:31

Scott Tolinski

Here we are. And and this is available, folks, for for you to use, g p u I .rs, if you'd like to use the UI framework.

10:42

Scott Tolinski

This is this the exact framework in just made available

**Topic 6** 10:43

#### GPUI is open but Zed moves too fast for an ecosystem

10:47

Guest 0

from Zed? Yeah. It is. Right now, it it's available under a permissive license.

10:53

Guest 0

It's still in the Zed repo. It's not exactly like, you know, what thirty seven signals did with Rails, where they really made it this big project that everybody could use. Like, it's open. It's available. But we're not exactly bending over backwards to worry about building this big ecosystem around it because Mhmm. Zed just moves too fast.

11:14

Guest 0

Like so up until now, we've just sort of kept it as a a crate inside of Zed, but it's there. You can use it.

11:23

Guest 0

People have even spun up forks. People built all sorts of apps on top of it. I saw a, like, quant trading app, high performance quant trading app, you know, doing Wow. Crazy trading graphs and stuff like that built in it, and there's a bunch of things out there. Yeah. The you know, one thing I've always appreciated about Zed is

11:43

Scott Tolinski

the speed at which it it operates in general, and I know that that seemed to be a big component of building this editor.

**Topic 7** 11:44

#### Atom nostalgia, Sublime loyalty, and why VS Code won thanks to TypeScript and LSP

11:53

Scott Tolinski

But but Zed has always felt so lightweight while being so full featured to me. So it it does make sense that that would be born out of the death of Adam and, the arrival of Electron. Wes, were you an Adam user?

12:10

Wes Bos

Yeah. Well, kinda. Because, like, I was the sublime sublime tech guy. Like, that was my whole thing. Yeah. And I use sublime forever.

12:20

Wes Bos

And I dipped into Adam here and there, and but I never went whole hog on it until, like, Versus Code came around. And, it was like like, Versus Code started to get that big momentum. That was probably, like, eleven years ago if I were to guess. Yeah. I I really liked Adam because,

12:41

Scott Tolinski

I always liked designed apps. So for me, it was the Adam was the first text editor where I felt like everything was designed, where Yeah. When I went to Versus Code, it felt like a massive step backwards in terms of

12:55

Guest 0

the user experience of it all, but then it just became, like, the thing to use. So it was like, okay. I guess I'm using Versus Code. Yeah. It just they had so many more resources they were pouring into it, and the reality was they were a much a more mature team with a more mature code base. Like, they were doing Visual Studio online.

13:15

Guest 0

Eric Gamma had already created Eclipse before that.

13:18

Guest 0

And we were just, like, noob web developers making Adam. Like, we didn't really stand a chance in that matchup.

13:28

Guest 0

And I think also a really critical thing that we just missed was TypeScript.

13:33

Guest 0

And I think Versus Node really rode to success on the back of TypeScript and the language server protocol that they built around it and just that whole evolution.

13:44

Guest 0

And we were still trying to build an editor in, like, dynamically typed at the beginning, it was TypeScript. Don't even get me started. That was not my choice.

13:54

Guest 0

You know, it it took by the time we realized that the language server protocol, which grew up around Visual Studio Node, was gonna be a big thing, it was just kind of too late for Adam, I think.

14:07

Guest 0

But no worries. Like, just that all served as the training ground for what we ultimately applied to Zed. Yeah. Would would you,

**Topic 8** 14:16

#### Would he pick Rust again? Yes, except the compiler is slow

14:16

Scott Tolinski

like, given what you've gone through in terms of having to build a whole UI layer and everything in Rust, If you were starting over today, July well, I'm not gonna date this because it's if you were starting over today, would you still pick Rust? Of twenty twenty six. Yeah. Yeah. Summer of twenty twenty six.

14:38

Guest 0

Would I use Rust again? Yes. Yeah. A 100%. Absolutely.

14:42

Guest 0

Yeah.

14:43

Guest 0

I briefly looked at Zig, and, you know, I don't know enough about Zig, and it wouldn't have been a choice at the time when we started Zed. I don't think it wasn't far enough along.

14:56

Guest 0

But I think for the nature of the complexity of what we're building with Zed, and it being an open source project where we're merging pull requests in from strangers on the Internet.

15:08

Guest 0

Having the memory Sanity, you know, having really strict type safe security around everything we're doing is critical.

15:20

Guest 0

And I don't really know any a lot of other options that give us that kind of on the metal axis.

15:25

Guest 0

My one gripe about Rust continues to be that the compiler is slower than I would like.

15:31

Scott Tolinski

I'm just gonna say that. I have said that. People in the comments are like, that's not slow. I'm like, oh, come on.

15:38

Guest 0

It's slower than I would like, but, the language is incredibly well designed. I I love it.

15:47

Guest 0

So, yeah, I haven't really seen anything emerge that would be a better alternative for the problem we're trying to solve. Sick. So you mentioned you're you're taking in, contributions

**Topic 9** 15:58

#### How the Zed team uses AI, reviews PRs, and closes slop

15:58

Scott Tolinski

from outside into zed.

16:00

Scott Tolinski

I'm I'm curious about, like we always, in this day and age, need to ask about, like, AI usage in building these things. Like, what's AI usage for the zed team look like? And and what what are some of your processes look like in building?

16:17

Guest 0

We use AI a Scott.

16:20

Guest 0

At least I do. And I think it it varies across the team. And, obviously, most of the Wes, everybody on the team is an engineer back from the days when you didn't have an agent to write any code for you. You were typing it out, and, you know, you were lucky if you could get an autocomplete.

16:39

Guest 0

So I'm really glad that I have that pretraining of my neurons in place to really understand software. Like, I I think it would be a different story if I were getting my start now. Some things would be easier, and then certain kinds of lessons might be harder.

16:54

Guest 0

But I I write a ton of code agentically these days.

16:59

Guest 0

And I use it in the pull request review process. When I'm reviewing pull requests, it's amazing to get some help interpreting what the heck is going on in this diff.

17:10

Guest 0

One thing we don't have a ton of is, like, this dark factory thing, where there's, like, loops and things going on super autonomously.

17:23

Guest 0

We have a little of that happening, like, you know, just getting a head start on stack traces and things of that nature.

17:31

Guest 0

But for the most part, definitely, my usage of AI is extremely engaged and reading the majority of the output coming out of the thing and, yeah, being very engaged with it. So, but it's a big part of my workflow. It's changed the calculus a bit. Like, there's a the the volume of contributions. We're having to get much more clear about what we're gonna accept and Scott. And, you know, in review the review load is, quite a lot to keep up with, especially with a bunch of other priorities going on. Yeah.

18:07

Guest 0

But I don't know. We get, like, 40 something bugs a week fixed via external contributions.

18:15

Guest 0

I'm sure a lot of those were facilitated by agents.

18:19

Guest 0

I don't begrudge people for getting leverage, especially in a code Bos complex as zed. You do have to insta close the slop once that someone's not putting care into what they're doing.

18:31

Guest 0

But, yeah, it's a big part of it, and I don't have anything against it.

**Topic 10** 18:36

#### Do we even need a text editor anymore? Text is more powerful than ever

18:36

Wes Bos

Can we ask about you guys building a text editor in the age of maybe we don't even need a text editor? And and I I don't wanna say that, like, snarkily, but, like like No.

18:50

Wes Bos

Like, what do you think about that? What do you guys are you moving to something? Or or, like, what do you think there? Yeah. I used to call

18:57

Guest 0

what we were doing building a text editor.

19:00

Guest 0

Yeah.

19:01

Guest 0

And now I really think about what we're doing more JS building a user interface for interacting with software.

19:09

Guest 0

And text is a big part of every dimension of that, even even more so now that Mhmm.

19:17

Guest 0

Agents are involved. Right? Like, if you think about all the problems of interfacing to text, like, text has become even more powerful than it has ever been now that language models are this big part of our lives. Language is made out of text.

19:33

Guest 0

And I think people underestimate how difficult it can be to build a good user interface around text.

19:43

Guest 0

And I also still think that reading code and navigating through code with the facilitation of agents is gonna remain a pretty important part of building real software that's not just tossed out vibe coded or Yeah. We're just gonna run a loop and torch tokens on this until maybe it's approximating correct.

20:05

Guest 0

Yeah.

20:06

Guest 0

I think the craft of software is going to involve humans, I think, multiple humans engaging with firehoses of text, and those firehoses of text interleaving through source code.

20:24

Guest 0

So yeah, I think a lot of the yeah. I don't know that necessarily the, latency of typing a key inside of a source file and getting a response and a autocomplete popping up and choosing from your algorithmically determined list of options Yeah. Is as relevant anymore. But a lot of the skills and lessons that we learn solving that problem, I think, apply

20:51

Wes Bos

even more so to this new world that we're Yeah. Entering. Whatever that tool looks like. I agree as well. Like and I'm so glad that you guys are are building this because, like, we need wicked awesome I I would say I would I'm not gonna say primitive, but, like, we we talked to the PR computer guys, and and they're building, sidebar tree structure of files, like arrows, like drop down, and diffs. And they built, like, the fastest one ever, and everybody everybody needs it. In in this age of being able to just type in the box whatever you want, we need these amazing, like, well done, low level, like like you said, on the metal primitives, to build this type of stuff. So I'm glad that, like, not everybody has, like, thrown it out the window. And it's like, alright. We're just gonna Vibe Node absolutely everything, and it's all gonna be in the browser. Like, you need people who care about this type of stuff. So I'm stoked that Zed is still here for that. Well and

21:51

Guest 0

I think it's worth mentioning that we're building something new. Yes. I was gonna say this Wes, almost kind of unknowingly

**Topic 11** 21:52

#### Enter Delta and DeltaDB: version control below the commit

21:58

Scott Tolinski

because Wes hasn't seen Delta.

22:00

Scott Tolinski

And No.

22:02

Scott Tolinski

I wanna applaud you, Wes, for that lead in because it's a great lead in to talking about Delta, which JS the the zed.dev describes on the early access that it's a version control system that records the work as it unfolds and keeps every change connected to the conversation that shaped it. I think that sentence really undersells what I saw from Delta. So, Nathan, I'm gonna give the floor to you to maybe, tell us and describe to Wes a little bit about Delta. Yeah. So, I mean,

22:31

Guest 0

what you're reading on our site as of the recording of this podcast, which I think by the time of publication will have said a lot more, but, it's really about Delta DB.

22:44

Guest 0

And Delta DB was about a year ago where our head was out about the problems we wanted to solve, in general and with respect to the emergence of agents, which is really about filling in the gaps between commits and enabling this new kind of collaboration that we've been thinking about since the very beginning of Zed. The plan was Zed from the get go was build an environment worthy of the world's best developers where, you know, the most discerning, high skilled developers would wanna spend their time writing software, and then give those developers an environment to connect to one another Scott of in the process of writing the software itself.

23:29

Guest 0

Because I always viewed this diff review.

23:32

Guest 0

I worked at GitHub. I I I loved my time there, but it never felt back in the day, like, it was called social Node, It never felt that social to me. I mentioned earlier that time of plugging two keyboards into a computer and, like, working directly with somebody in the code as we were writing software.

23:51

Guest 0

That always felt like social coding to me. And it, to me, felt like a tools problem that we weren't able to have higher fidelity conversations sort of directly connected to the software we were writing, and we were instead just tying our conversations to these comments that were posted on this web page that rendered kinda slow.

24:15

Guest 0

And they were tied to these snapshots.

24:17

Guest 0

And, really, like, it's all because of Git. It's all because Linus Torvalds built this tool for, like, helping people email patches to one another on the kernel mailing ESLint, and everything was sort of designed around snapshots and ran at the pace of email.

24:34

Guest 0

And always just felt like a higher fidelity conversation would be possible if we could drop below that floor of commits. And so Delta DB has been our vision for, yeah, giving basically every version of the software as it evolves an address, that you can recall it, talk about it, and you can drape over that level of tracking.

25:01

Guest 0

The ability to comment on any part of the code at any time and thus be able to have a conversation sort of in the code Bos as you're working on it. Over the past year, the vision of Delta DB, you know, as we built it, as we started building it, of course, we weren't gonna shove it into our million line plus IDE with hundreds of thousands of users Node day one.

25:29

Guest 0

We built this little reference client for it.

25:33

Guest 0

And, yeah, over the past year, that little reference client has become essentially a a brand new product. So we're gonna become a two product company.

25:45

Guest 0

And yeah, so we have Delta DB, and the we named the the new agent post focused editor that we built around it, Delta.

25:57

Guest 0

And so in some ways, Delta looks at first glance like a lot of the other agentic development environments that you see out there.

26:06

Guest 0

But when you peek below the surface a bit and start using it, I think you're gonna discover a lot of the power that is enabled by the it being built around this more sophisticated version control system.

26:19

Guest 0

That drapes on top of Git, it's still designed to interoperate with Git, and it you really don't think about Delta DB as you're using Delta, but what it enables is a new way to collaborate.

26:33

Guest 0

And, yeah, our our vision is really it was always conversations in the Node, linked to the Node, rather than snapshots of the code.

26:44

Guest 0

But what's changed in the past two, three years and really accelerate acceleratingly, like, since last fall, at least for people that write Rust, because before then, agents couldn't really write Rust, is that now conversations are becoming sort of the source of our code.

**Topic 12** 26:45

#### Conversations become the source and code becomes a downstream artifact

27:03

Guest 0

Like, we were thinking we'd write all the code and have conversations in it. Now conversations are the beginning, and code comes out of that. It's a downstream artifact.

27:11

Wes Bos

Yeah. So you're you're sharing, like, the entire agentic conversation? Like, that is stored in your history of

27:19

Guest 0

of delta? Yeah. We have a delta level.

27:23

Guest 0

So every single token that flows in every prompt Yeah. Everything that happens is sort of captured continuously as it's flowing in.

**Topic 13** 27:31

#### Shared agent threads with conflict-free synced work trees and model blame

27:31

Guest 0

And what it enables is you can start this conversation, attach, clones of your repository to the conversation, share a link with a a member of your team or, you know, if you set up a GitHub app or whatever we eventually evolve into, you could sort of broadcast it to your team. Hey. I have this thread that I started with an agent, and anybody else can pop into that thread with you.

28:02

Guest 0

And they'll have the same work trees cloned down to their disk. And we do a conflict free sync of every single edit. So you can have a copy of the exact work like, JS in in Git, like, a work tree is your own personal thing. Yeah. If you wanna collaborate, you gotta commit and push.

28:21

Guest 0

Delta DB backing this whole thing means, like, every participant in that conversation with the agent or multiple agents, because you can switch between them, JS a a clone of the code that they can touch and manipulate, and it will all be synced across all the machines in in real time.

28:39

Scott Tolinski

Oh, man. That's cool. So it's It rips. It it rips. And and, some of the things that you might not know from this is that, like, every single line of Node, you can determine, like, which agent wrote that code specifically.

28:55

Scott Tolinski

Like, which Node. It is like a model blame is really what it is. It. Opus five.

29:00

Scott Tolinski

And you can annotate responses.

29:04

Scott Tolinski

It's really, really sick.

**Topic 14** 29:06

#### Does Delta replace GitHub or Claude Code? Not yet

29:06

Wes Bos

The people listening right now are probably trying to place where this will live in their thing. Like like, this is like, does this replace GitHub? Does this replace Cloud Code or Open Node? Or, like, is this the this is the chat app, but also the, like, the history?

29:26

Guest 0

We we're not aiming necessarily I mean, long term, I would love to replace GitHub. And I think there are men a number of contenders to that prize. But I think Scott term, I think it will be a really good complement to whatever Git hosting provider you have in the very short term.

29:43

Guest 0

Because what it enables is something that a snapshot based collaboration can't really provide, which is just the ability to share a thread with somebody.

29:53

Guest 0

And, you know, like, if you share a link in Figma, you can just dive right in and see what the designer JS doing and comment on it and just interact right in the corner or follow. Yeah. You can come along a day later, and maybe they're gone.

30:08

Guest 0

And you can still sort of pick up. And that's this continuous experience that we're imagining Wes you rather than kind of chatting with an agent all by yourself, having it stamp out these snapshots downstream artifacts, pushing them up to a server that's hopefully online that day, and then having someone try to load those massive diffs in a web page that can't even render them, because it's using web technology that can't Scott, and having a conversation with your team now on this inscrutable diff that, like, maybe you didn't even understand as it vomited out of this agent at, you know, fire hose speed.

30:48

Guest 0

All of that seems like kind of just not an efficient way to work anymore, especially in this paradigm.

30:55

Guest 0

Our vision is more and more, maybe you get something started, then you pull your teammates into that conversation where they can interrogate the agent with you. Mhmm.

31:05

Guest 0

Have a conversation with you.

31:08

Guest 0

You both have access to the agent and the entire context window leading up to that moment and the code that's, you know, cross referenced with all those moments.

31:17

Guest 0

Yeah. And it's just a continuous experience where the collaboration takes place sort of in the process of writing the code and we're continuously with it. At the end of that, you can still, like, open a pull request and have CI run and merge and do all those things, but a lot of the conversation that you might have had in comments on the diff if you could get it to render, you would have already had, and you'd had with the benefit of the very agent and all the context and planning

31:48

Wes Bos

that went into that code existing to begin with. Yeah. I I often think about how much is lost in the conversation about how you got up to something. Right? Like like Stack Overflow JS trying this, like, Stack Overflow for agents right now where when the agent learns something or when it cracks something, like, write it down somewhere because somebody else is gonna have to go through that process in the future. And wouldn't it be nice if if that history of you getting up to that point would be would be visible. Or even just like you're you're looking through some Node. Why is this built like this? Well, hopefully, they leave a comment. But if not, you can there you can blame and look at the history of how you got there.

32:31

Guest 0

Yeah. Yeah. I completely agree. And there are two big pieces of Delta DB that are, like, underlying the experience I'm describing. One is conflict free replicated work trees, and that is sort of starting from some particular GitHub base and building forward from there.

32:51

Guest 0

And because we're tracking that all in such a fine grained way, we have a very nice fluid open comment on anything anywhere in this real time or asynchronously. You don't have to work in real time, but it's like the system doesn't impose any can be synchronous or asynchronous. The system doesn't impose it on you. It's whatever makes sense for you and your team. So there's that conflict free replicated work trees. And then this other big dimension that we're working on is this delta level understanding of the repository as a whole.

33:25

Guest 0

And what that's gonna enable is the conversations taking place in each of these threads that have their own work trees on them to be aggregated together into this global index Wes we're essentially turning your code Bos into a, like, a scaffold for all the conversations that you might wanna have or like a canvas.

33:45

Guest 0

And so it adds this, like, metadata layer to the code where you can sort of in any state of the code that you can see, whether it's committed or not, you can attach an annotation and then be confident that in any other version of that code base that that character or range of characters that you marked is visible, it will be able to be retrieved.

34:11

Guest 0

I don't know that we're gonna we're that we might ship the work tree stuff slightly before that, the repo stuff, but, it'll be following soon after. Or if I can really crank between the recording of this podcast and, and, when when it comes out, it'll be out there.

34:33

Scott Tolinski

Sick. Yeah.

**Topic 15** 34:35

#### Pair prompting and fighting decision fatigue

34:35

Scott Tolinski

It it it all makes so much sense to me. And when we were at JS Nation and Amsterdam React Summit, I was a part of, like, a panel on, like, the future of UI and everything like that. And one thing that we landed on in that panel was, like, pair programming and pair prompting needs to become more of a thing. There's too many decisions you're making. There's too much decision fatigue. Right? Before Wes used to, make, you know, ten, twenty decisions a day in regards to maybe even much less than that, like, large decisions.

35:09

Scott Tolinski

And now we're making tons of it's constant barrage of answer this Wes, answer this question.

35:15

Scott Tolinski

And just the amount of value that you'd have from bringing someone else into that process cannot go understated. It's like something that, like, I'm I'm so missing. And I think about so many instances on, like, projects that I'm working on or JS Wes like, it'll be so much better to instill that layer of not just pair prompting, but this pair workflow. I'm, like, I'm really excited for this to exist in the world.

35:41

Guest 0

Me too. Yeah. I mean and, again, I think sometimes people hear pairing, and they freak out because it's like, you mean I'm gonna have to talk to somebody? And, I mean, the answer is maybe because I think you're right that, like, it used to be that you could look at a diff, and the diff of Scott of Wes a reasonable proxy for the conversation that needed to happen and the alignment on the decisions.

36:07

Guest 0

But now, like, the deciding is all that we do, like like you're saying. Right? And so by the time you're looking at the 10,000 line diff, and you could try to break those down into smaller diffs, etcetera, but, like, in the limit of that, the smallest diffs you can have are deltas. Right? Like, bring somebody in on the conversation itself. Like, that actually is the true source of our software. It's not the source code anymore.

36:35

Guest 0

And so how can you really claim to have an effective review if you're not pulling somebody into that? And, yes, you can, like, staple a transcript and, you know, attach it to the pull request or something for someone to look at later.

36:49

Guest 0

But why not just make that conversation itself collaborative? That's what that's what we're after. Yeah.

36:57

Wes Bos

The Worktree stuff is is kinda interesting. Like, Delta DB, it says on the website, virtualizes the work tree, so spinning up a new agent branch is effectively free.

**Topic 16** 36:58

#### How virtualized work trees and a custom sync engine make branches free

37:07

Wes Bos

Before I have my second question, can you talk about how you how that is technically possible?

37:12

Guest 0

Yeah.

37:13

Guest 0

So we have our own it's a database, Delta DB. Yeah. We have our own b tree implementations and hash array map trees, and then, you know, those ultimately write to some underlying backing store, depending on, you know and the ESLint, we are currently using SQLite, but that's fungible.

37:33

Guest 0

But it's sort of our own abstract representation of the entire file system underlying your tree. Like, we're not critically dependent on the file system. We model the file system. And we do that for a couple different reasons. One is it lets us have it lets us make it a lot more lightweight. Like, we use copy on write, persistent data structures.

37:58

Guest 0

So clones are order one operations for us.

38:03

Guest 0

You can get that on some exotic file systems, but it's not really something that's, like, universally available and and good. So we just decided kinda like the same decision we made around the web browser, it's like, let's just own this. Let's take total control.

38:21

Guest 0

Mhmm. And so and then another reason is we're able to make it conflict free.

38:26

Guest 0

So, you know, we have a surface representation of this tree of files and all their contents.

38:33

Guest 0

But underneath that surface representation is a bunch of distributed systems, foo, that lets us, you know, have multiple replicas of this thing and people manipulating it on their own schedules concurrently.

38:49

Wes Bos

So my, my Scott sense is tinkling here. Did did you build a sync engine then? Is that what you did? Yeah. That's a big Yeah. That's a part of it. Yeah. Scott, they built a sync engine. Get back here. I don't know where he went, but Scott loves sink engines. Okay. So that means that any any change ever that anyone has ever made

39:10

Scott Tolinski

is done Sorry. Wes, my dogs are going nuts, so I gotta bring them inside. I'll be right back. Okay.

39:16

Wes Bos

We'll talk about sick engines in a second. But but that means that because every single delta I I I understand the name now.

39:25

Wes Bos

You can just replay every single one, and then there would be no

39:29

Guest 0

conflicts. Is that how it works? Yeah. No. Of course, if you're collaborating asynchronously Yeah. You know, we could we could, if we wanted to, like, collate together all of the deltas of people collaborating asynchronously, and you would get some convergent result. Everybody would see the same mishmash of stuff slammed together. So, obviously, like, on an on a semantic level, there's still the notion of a conflict. But if you're working together synchronously or multiple agents, more importantly, are working together, on tighter time scales, the conflict free nature of it is really helpful. Essentially, it's giving the illusion.

40:11

Guest 0

And we also do, like, mirror it down to the file system.

40:15

Guest 0

System is just kind of like a user interface. Like, we have the source of truth, like, in our database where we fully control things and have the efficiency guarantees and the abstraction and the, eventual consistent semantics that we want, then we mirror that down to the file system to give if you wanna run your compiler or if an agent that doesn't have access to Delta DB wants to interact with it.

40:41

Guest 0

Ultimately, the illusion that we're creating is that this work tree exists on n machines at once.

40:50

Guest 0

Okay.

**Topic 17** 40:51

#### Centralized hosting at delta.dev and Rust UI compiled to Wasm in the browser

40:51

Wes Bos

And is that like a like a mesh of computers, or is there will there be, like, a Delta DB server that's, like, the the main boy? So we're launching delta.dev,

41:01

Guest 0

which is going to be our collaborative platform that hosts everything.

41:06

Guest 0

But the algorithms are designed to be, you know, very, I don't know how you describe it. Like, they don't depend on a central synchronization coordinator, in a critical way. Yeah. But it just turns out that, it's the best way for us to deliver a really reliable, solid, good, clear user experience.

41:28

Guest 0

It's also the best way for us to monetize this because it's always been the plan of Zed to build a service that lets people collaborate.

41:36

Guest 0

Mhmm.

41:37

Guest 0

So for now, at launch, it's a centralized architecture for both of those reasons that just make it stupid simple to go to delta.dev.

41:47

Guest 0

Yeah. So you we have a desktop.

41:49

Guest 0

Like, delta is a desktop app that you can download, and, you know, can read from your file system and do all the same things that you can do in Zed, with respect to, you know, native privileged access and, you know, the the fastest that we can possibly give you.

42:07

Guest 0

We also have a web component of it, which is basically very the same code Bos compiled the Wes assembly targeting, you know, WebGL, like, graphic surface.

42:19

Guest 0

And that means you can share a link and give somebody a URL on delta.dev that they could drop in on and without even downloading anything, you know, off with their GitHub and do collaborating with you. Woah. Wait. So Wow. The entire hold on.

42:37

Wes Bos

The entire u the the interface is built in, like, Rust in the the UI that you built. GPU UI. Yeah. GPU UI. But then you compiled that to Wasm or and have a web component that you can drop into the web, but the UI was written in Rust?

42:56

Guest 0

Right. So it won't be terribly slow.

43:00

Scott Tolinski

Yeah. That's crazy.

43:01

Wes Bos

So it's back to the browser. Just no HTML.

43:05

Wes Bos

Is that right?

43:06

Guest 0

That's right. Yeah. And the browser this time yeah. Before, we brought the browser to the desktop.

43:14

Guest 0

AM Electron era. We're going full circle by bringing the desktop to the browser kind of, like That's so bad. Fast tech to the distribution channel Mhmm. And just making it convenient. Really, the browser JS about convenience.

43:27

Guest 0

And we are gonna wanna encourage people, download the desktop app, you know, like every dual like Linear. Right? You know? Like, they encourage you to download the desktop app, and there are benefits to that. But if you're willing to run-in our hosted cloud provider, you know, and the cool thing about our like, I was talking about the work trees.

43:51

Guest 0

And, you know, so if you're using the web, then you're obviously gonna have to use our hosted runners.

43:59

Guest 0

But if you're using the desktop app, you could use your own computer, and we can actually mix and match in the same thread.

44:05

Guest 0

So you could have, somebody running stuff on their own computer, and then they could share a link and someone joins from the browser, and they're running their turn when they submit on a hosted runner. And these and they could go back and forth, actually. Like, yeah. I don't quite you know, it's gonna start simple, but, like, kind of the sky's the limit in terms of the flexibility we have ESLint of moving the code around, to be worked on in the appropriate spot. Mhmm.

44:37

Wes Bos

That's really cool. Man, I just I I've been thinking about this. Like, this is not related to this, but, I've been thinking about, like like, feature of UI.

**Topic 18** 44:38

#### Will UI just be language? Why graphics still matter

44:48

Wes Bos

And everybody's talking about, like like, I did a video on just, like, generated UI or, like, some people are talking about it will just be streamed in. Do you have do you have any opinions of, like would that ever be a future? I Node games kind of already do that, but, like, just some UI generated and simply just streamed to a thin client. I mean, the first thing I'm gonna say about UI is I think we're gonna have a lot less

45:10

Guest 0

UI, because Yeah. The ultimate user interface is basically language.

45:17

Guest 0

Yeah.

45:19

Guest 0

And, you know, for the longest time, for the last, whatever, fifty years, forty years, like, the only people that could really use language to talk to a computer were people like us, software developers.

45:29

Guest 0

And, even then, it was, like, pretty grueling, brutal process of memorizing a menagerie of commands and flags

45:39

Scott Tolinski

and programming words and tags and all of it. Right? I was trying to explain to my nine year old the usage of brackets and semicolons, and I was just like, it's really wild. Yeah.

45:51

Guest 0

So I think now that language natural language is, like, opened up as a way of interacting, like, it's such a superior means of expression over essentially, like, these gestural user interfaces that we've been confined to use in the previous era. Like, it's really just yeah. It's sort of like the, like, I like, I if you if you couldn't talk, I guess you'd have sign language. So I don't know. Maybe I'm but, like, you know, I think the most sophisticated user interfaces, the problem with them is they started to approach sign language, which is really hard to learn. And so we were kind of in this common denominator of, yeah, fairly crude gestural interfaces. And I think JS language becomes cheaper and we get better at building user interfaces, utilizing text and language, it's gonna take over more and more of the user interface. But then I still think there is a place for graphics. Like, it's why Delta's a graphical app and not a terminal agent.

46:58

Guest 0

Because I do think that, yeah, graphics and, you know, subtle use of images and icons and progress bars and things that are not confined to a, you know A terminal line. Monospace terminal grid Yeah.

47:17

Guest 0

Still have tremendous value.

47:20

Guest 0

And so I think in that context, absolutely, are we gonna have yeah. Even, like, Mermaid is in Delta, like, a very early example of that, and I think we're gonna be adding more capability of that sort over time of, like, yeah. Mermaid is this markdownesque.

47:39

Guest 0

It's like the markdown of diagrams or something. Right? Like, the syntax the agent can token efficiently generate to build a little flowchart or whatever it is you need in line.

47:50

Guest 0

But, yeah, why wouldn't that extend to where gestural interfaces are appropriate? Like, why wouldn't you let the agent sort of assemble that on the fly? But I also just think you can accomplish so much with language alone.

48:03

Wes Bos

Totally. I agree. I agree. That's good. I'm I'm I'm glad you said that because I've been on this, like, what does the future of UI kick go for in the in the next little last, like, six months. You Node? I've been in talks. I've been talking about MCP UI, and some people think the browser is gonna go away, and the it's somewhere in between. So I Wes, like, asking people that whenever I, get to chat with them.

48:26

Guest 0

It's funny. Like, when I I forgot what website I was trying to navigate.

48:31

Guest 0

It was I'd be, like, from, like, a a summer camp for my kids or something like that I was trying to figure out, and, like, their website was just, like, awful and really hard to navigate and understand. And I just finally was like or it didn't actually take me long. I just fed it to my agent in Demosha.

48:50

Guest 0

So can you read this and tell me just answer my questions? Like, I don't want to browse. I wanna ask want this interface at all. I just want to do the thing that I wanna do. Yeah. Yeah. So I think a lot of UI is converging, and everybody's kind of going after it from a different angle.

**Topic 19** 49:08

#### Conversations as editable documents you can annotate anywhere

49:08

Guest 0

But I think we're converging on these you see the same app over and over again. I mean, I think one unique thing about Delta is we know a lot about how to build efficient interfaces for text. And so, like, we don't model the conversation as a chat. Yeah. It is a chat, but it's a document. And that means you can move your cursor around inside of it. And when you put your cursor in, anything that came out of the agent, whether it's a command that it ran or an edit that it made to your code or, you know, nested markdown ESLint with 17 different options lettered a through z with numbers and letters and, like right? I got sick of remembering the the letter of the option that the agent wanted me to follow. And so what we let you do is you just move your cursor around inside all that rich text freely.

49:59

Guest 0

And wherever you want to annotate, you just start typing, and it immediately, like, lets you annotate right there.

50:08

Guest 0

So

50:09

Scott Tolinski

that's the important part of it. Amazed by I mean, the fact that when when you all first showed me Delta, I was immediately impressed with the fact that it wasn't another Node slash any of these apps that, like you said, they all look the same. Here's your projects. Here's your chats. Here's your chat window. And to some extent, there is that, but, like, it is different. It is fundamentally different.

50:34

Scott Tolinski

And I think that takes such a, I think that takes such a different brain to to like, what does this actually look like? Surely, we can't just be having a a chat g p t chat interface for every single code Bos where you're just chatting at it. And I think this like, in my experience with the tool, I was really impressed by the capabilities there.

50:55

Guest 0

Thank you. Yeah. I mean, it was just born out of the experience of spending a ton of time trying to write code with agents and building what we wanted or, trying to reduce the friction we ourselves were feeling.

51:11

Scott Tolinski

Yeah. Yeah. And it feels like, you know, where you know, you wanted to create Zed, so you had to create GPUI, to solve those problems. You know, wanted to solve these problems of working with agents, and you have a whole version control system.

51:30

Scott Tolinski

All kinds of new collaborative tools and and interfaces. So, it takes people like that to think outside the box, and I I'm really stoked for the audience to give Delta a try. I'm stoked for Wes to give it a try, honestly.

51:44

Wes Bos

West, you wanna pair program with me on some stuff? I'm in. I'm Say Yeah. I I feel I I remember this now. I I tried to sign up for it when you sent me the link, and I hit some Google Forms issue. And I did I forgot to go back to it, but I just put request early access on there, and it worked this time. So I'll give you the binary. And the Sanity, you know

**Topic 20** 51:45

#### Early access, beta launch, and Wes gets invited

52:07

Guest 0

obviously, we're recording this before the launch date. It's getting ever less alpha and ever more beta.

52:15

Guest 0

And, yeah, we're, you know, we're we'll be launching the beta and starting to let the first people just from the Internet in, sort of around the publication date. Yeah. So today. Sick.

52:29

Scott Tolinski

So, folks, head on over to, you said this will be at a new URL? It will be at delta.dev.

52:37

Scott Tolinski

Cool. Head on over to delta.dev.

52:40

Scott Tolinski

Nathan, is there anything you wanted to

52:43

Guest 0

hit that we didn't touch upon here? Well, you had you told me that we had the you wanted me to share about what I loved or something that I loved unrelated to Wes.

**Topic 21** 52:54

#### Sick picks: Keychron Q11 split keyboard and Blue Note jazz with Grant Green

52:54

Scott Tolinski

Well, that that's what I was going to get into next show. Okay. Good. No. No. Not shoot. Perfect lead in. This is the part of the show where we talk about sick picks and shame shameless plugs. So a sick pick is anything you're enjoying in life right now. What is your sick pick?

53:09

Guest 0

Can I have two?

53:10

Wes Bos

Yes. As many as you want. Yep.

53:14

Guest 0

So, yeah, the the two thoughts that came to mind, one is my keyboard.

53:20

Guest 0

It's the Keychron q 11 here. I have two of them on my desk because, one, when it's not raining, I take out all my deck. But what I love about it is it's a mechanical keyboard. Oh, shit.

53:35

Guest 0

Properly.

53:38

Guest 0

But it's not weird. Right? Like, if you were to put this thing together, it looks normal, and it doesn't have, like, an ortho linear layout or any of that, like, probably really ergonomic stuff that I cannot get my hands to learn.

53:53

Guest 0

But it lets me separate my arms.

53:57

Guest 0

Yeah. And it has mechanical key switches, and it has programmable firmware.

54:02

Guest 0

And so I'm able to map the, the right or the left space bar to command. So I'm not curling my thumb under.

54:13

Guest 0

You know, just getting some control and then, like, using the yeah. Using agents to program the QMK firmware Yeah. Yeah. To, like, make it light up. You You know, I light up the f keys in a certain way so that when I'm coding at night, I, like, know exactly where they are.

54:33

Guest 0

Yeah. I love it. It's a great product. Keychron seen a split that is normal. Like, all the black keyboards are all crazy, and they're all really hard to get used to. Yeah. Exactly. And my problem is, like, you know, inevitably, as a founder, I'm always, like, on an airplane flying somewhere, and then I'm on my laptop again. And so I I find, like, as soon as I build the muscle memory for, like, the ortho linear domed crazy thing, I go back to my laptop, and then I couldn't use it there. I don't know. I just don't have some people are great at that, like, switching UI.

55:06

Guest 0

I'm very bad at it. I I like, my muscle memory is very one path.

55:12

Guest 0

Yeah. So that's the technical recommendation. And then the the nontechnical recommendation, something recently in my life that I love JS, yeah, growing up, my dad would listen to, like, the worst jazz.

55:27

Guest 0

Like, it was, like, you know, Node smooth jazz, like, the kind of stuff that you would listen to, like, if you're in the elevator or, like, at the dentist office or something like that.

55:36

Guest 0

But a little over a year ago now, I, like, flew into you know, I actually was going to Mississippi, for Christmas. My girlfriend's mom visiting my girlfriend's mom for Christmas, but we flew through Louis Armstrong International Airport or whatever and I in New Orleans.

55:57

Guest 0

I'm like, oh, Louis Armstrong. Like, I wanna learn more about jazz.

56:01

Guest 0

And I've kind of with the help of AI, which knows so much about, like, this catalog of of jazz, like, yeah, really honed in on, like, kinda 1955 through '64, really starting with Blue Note Records, which Wes, you know, they were one of the first labels to put, African Americans, like, on the album covers and not be racist jerks.

56:31

Guest 0

And they gave the musicians time to practice, which was, like, a rare thing at the time.

56:38

Guest 0

And they really valued artistry in the musicians. They weren't after just, like, what would sell.

56:46

Guest 0

They wanted to find kind of the artistic essence of the music.

56:51

Guest 0

And yeah. So it's it was it just served JS, like, a great, entry point into a new understanding of jazz that I never had. Takes more patience because I just think the music is a lot more sophisticated than, like, ACDC or whatever. Mhmm. But it's something that I've really come to to love. So I I figured I'd give you two, like, a a nerd one and a cultured

57:14

Scott Tolinski

one. That's good. Yeah. I like that. So any album in particular you want us to check out? Because, we you would not you would not believe that you are the second guest in a row to Sick Pick Jazz.

57:26

Scott Tolinski

Oh, wow. No one has ever sick picked jazz.

57:29

Scott Tolinski

Dan Abramov, who we just had on the show, sick picked the Thelonious Monk album. So Nice.

57:35

Guest 0

I love this, well, the first track of it, especially, Grant Green, Idol Moments.

57:44

Guest 0

It's, like, slow, bluesy, just, like, got so much mood and atmosphere. I don't I'm not a musician, so I don't I don't know that I'm, you know, well positioned to, like, give you all of the musical detail of it, but I just love it. It's got vibraphone in it.

58:02

Guest 0

Oh, I love vibraphone.

58:04

Guest 0

Love vibraphone. I remember that from, like, mister Rogers as a kid or whatever, but, like, hearing Vibraphone played by somebody that's got this just, like they're so cool. It's like blues sophistication going on. I don't know. I just feel, like,

58:18

Scott Tolinski

so cool when I listen to that track. Yeah. I don't know. I haven't heard this before. I I don't I don't know any Grant Green, so this is going to, this is gonna go on immediately following this. Can't wait. I love hyphen. Yes. Big fan. Yes.

58:34

Scott Tolinski

Sick. And shameless plugs, we already talked about, Delta. Is there anything else you'd like to plug? No. That's really it. Check out delta.dev.

58:43

Guest 0

It's a new way to collaborate with agents and your team written on all the same tech stack as Zed, and and a lot more.

**Topic 22** 58:44

#### Shameless plug for delta.dev and a Colorado meetup

58:56

Scott Tolinski

Sick. And it's awesome. So check it out.

58:59

Scott Tolinski

Thank you so much, Nathan. This has been totally totally cool. Yeah. Really appreciate that opportunity.

59:06

Guest 0

Yeah. I'm talking over you again.

59:08

Scott Tolinski

No. Talk over.

59:11

Scott Tolinski

Yes. And, fellow Colorado, folk, we gotta get together sometime. So,

59:17

Guest 0

agreed. That'd be great.

59:19

Guest 0

Yeah. For sure. Me up.

59:21

Guest 0

Sick.

59:23

Guest 0

Okay. Well, until then, it's been good.

## Share

×

Start at timestamp:

Embed Link [X](https://twitter.com/intent/tweet?url=https%3A//syntax.fm/1029&text=The Workflow of the Future With Zed&via=syntaxfm) [ Facebook](https://facebook.com/sharer/sharer.php?u=https%3A//syntax.fm/1029&quote=The Workflow of the Future With Zed) [ LinkedIn](https://www.linkedin.com/sharing/share-offsite/?url=https%3A//syntax.fm/1029)

[Podcast](/shows) [RSS Feed](https://feed.syntax.fm) [About](/about) [Sick Picks](/sickpicks) [Guest List](/guests) [Sentry.io](https://sentry.io/welcome/?utm_medium=site&utm_source=syntax&utm_campaign=syntax-sentry-evergreen&utm_content=footer)

[Source Code](https://github.com/syntaxfm/website) [Colors](/system/colors) [Layout](/system/layout) [Typography](/system/typography) [Theme](/system/theme) [Privacy Policy](/pages/privacy) [Terms of Service](/pages/terms-of-service)

[X](https://x.com/syntaxfm) [Github](https://github.com/syntaxfm) [Discord](https://discord.gg/W5y68HMfZV) [](https://www.youtube.com/@syntaxfm) [Tiktok](https://www.tiktok.com/@syntaxfm) [](https://www.instagram.com/syntax_fm/)

Syntax × Syntax is Produced by Sentry

Nearly 4M developers and 90K organizations rely on Sentry to see what actually matters, solve what's urgent faster, and learn continuously about their code.

[Try Sentry for Free](https://sentry.io/welcome/?utm_medium=site&utm_source=syntax&utm_campaign=syntax-sentry-evergreen&utm_content=footer)

©️ 2026 - Sentry.io
