---
id: yaks-b32e
title: Consider optimizations of the merged yak directory structure
status: shaving
type: task
priority: 1
created: '2026-10-18T19:31:08Z'
updated: '2026-10-18T20:00:00Z'
parent: yaks-73f4
---

Given that we've recently merged all yaks into a single directory, and that we're also in the process of reconsidering the way we handle branches and worktrees (ie, yaks-6a34), it seems like a good time to consider whether this is the right final directory structure state.

A lot of systems (git included, IIRC) go to some lengths to put large numbers of files into a predictably nested directory hierarchy. For yaks, this might look like (just to throw out an idea:
```
.yaks/
  thingone/
    3/
      5/
        a/
          thingone-35a5/
            artifact-zero.png
          thingone-35a5.md
      e/
        0/
          thingone-3e00.md
          thingone-3e05.md
        7/
          thingone-3e72.md
          thingone-3e7f.md
     c/
       1/
         b/
          thingone-c1ba.md
	thingtwo/
    4/
      8/
        f/
          thingtwo-48f9.md
...
```

or something like that, at least. I expect there are plenty of technical arguments to be had about whether such a thing is important, unnecessary, just a convenience, or even anachronistic cargo-culting. For all I know, this habit grew out of the constraints of 90s-era win16 filesystem constraints, like so many things.

Let's research this carefully, and if it proves still worthwhile, make a concrete plan to design & implement. A few considerations that come to mind:
- Do we separate herds at the top-level, even if just for clarity?
- Is it best to keep the `foo-1234/` and `foo-1234.md` structure we currently have at the leaves?
  Or would it be better to colocate a yak and its attachments? Does it matter beyond aesthetics?
- Should individual yak dirs/files get their full name repeated, even though it's visible in the dir hierarchy? Argument for: Quickly opening a yak or attachment by find-file in an IDE.
- What does the resulting tree balancing distribution look like? Do our random allocations keep that reasonable? I don't love the idea of having to perform inscrutable hashes, but are there reasons to do this that go beyond our random id allocation?
- What should the tools do with misplaced files? Eg, it's a little hacky, but could be convenient if `yaks` automagically finds them and puts them where they belong, so a user that _does_ need to create one manually (or just feels like it that day) doesn't have to do file-system surgery.

That last point gets at a larger principle: We started this project with the idea that it would be _simple_ and _filesystem-native_. We've grown some modestly complex features, somewhat in tension with this. Which is fine, but we shouldn't do such things without a defensible purpose.

The most obvious reason to do so would be performance at scale. But that's a question that can only be answered in light of an understanding of modern filesystem performance characteristics, and perhaps also those of modern editor/IDE indexing systems.

---
▸ 2026-10-18T20:00:00Z [delta-farm]

moved: hairy -> shaving

---
▸ 2026-10-18T20:00:00Z [delta-farm]

Claim (delta-farm, yakherd). Owner: `scale-1` (scout: measures, recommends, does not decide or shear). Decision belongs to Joel, before the orphan-farm migration (yaks-73f4).

Question: does a flat `.yaks/` stay fine at O(1000s-10000s) yaks, or does a nested layout earn its keep? Measure on synthetic farms of 1k, 10k, 50k yaks (realistic files: copy the shape of real yak files, a few % with attachment dirs), in /tmp, with the real binary: yaks `list`, `show`, `next`, `search`, the TUI startup (headless, first frame), `yaks sheds`; plain `ls`, `ls -l`, `find`, `grep -r`, shell globbing (`ls .yaks/*.md` arg limits); git `status`, `add -A`, `commit`, `log -- .yaks`; and a web IDE file tree: if openvscode-server/code-server can run headless here (a release tarball unpacked in /tmp is fine; no system install, no sudo), open each farm and time the explorer listing `.yaks/` (headless Chrome is on this machine; see docs/tui.md for the recipe). Compare against a nested layout only if flat shows a real problem. Report numbers in a table, attach the scripts and raw output, and give a recommendation with the threshold where flat stops being fine, if any.
