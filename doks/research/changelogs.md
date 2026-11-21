---
type: topic
title: 'Changelogs: how existing tools do it, and what yaks can take'
created: '2026-11-21T21:16:40Z'
tags:
- research
- changelog
- release
stale_after: P1Y
sources:
- sources/2026-11-21-9a1c601c-keep-a-changelog-110.md
- sources/2026-11-21-435c0b83-changesets-a-detailed-explanation-and-adding-a.md
- sources/2026-11-21-82ac39ee-towncrier-tutorial-and-check-command-2690.md
- sources/2026-11-21-2cb3322d-reno-openstack-design-constraints-and-usage.md
- sources/2026-11-21-2fffc71b-git-cliff-docs-and-conventional-commits-100.md
- sources/2026-11-21-f66f0491-github-automatically-generated-release-notes.md
---

# Changelogs: how existing tools do it, and what yaks can take

Read on 2026-11-21 for [yaks-a4d8](/yaks/yaks/yaks-a4d8.md). Joel asked whether yaks should adopt an existing changelog tool, fed from yak history, or build one in, and how a changelog could lead a reader into the farm's deeper context. The decision this research led to is in [the changelog design](/doks/design/changelog.md); this dok keeps the survey and the evidence.

## In one paragraph

Changelog tools fall into four families: a hand-kept file, a file derived from commit messages, small per-change "fragment" files rolled up at release, and notes generated from the tracker's pull requests. Each family gets one thing right. The hand-kept file is written for readers; derivation from git is automatic; fragments are captured while the change is fresh and a check makes them hard to forget; tracker notes link back to the discussion. None of them goes more than one level deep: an entry is a line, and at most it links a PR. In yaks a shorn yak already is a fragment, with the depth attached (notes, doks, sources, evidence, commits), so what yaks needs from the tools is their conventions, not their machinery.

## The families

### A hand-kept file: Keep a Changelog

[Keep a Changelog](/sources/2026-11-21-9a1c601c-keep-a-changelog-110.md) is a convention, not a tool. Its rules are the ones every generated changelog ends up imitating: changelogs are for humans, every version gets an entry, latest first, dated in ISO 8601, changes grouped under `Added`, `Changed`, `Deprecated`, `Removed`, `Fixed`, `Security`, and versions linkable. An `Unreleased` section at the top collects changes until release. It argues against commit-log diffs ("full of noise"; a commit documents a step in the code, an entry documents a noteworthy difference to users), against inconsistent changelogs ("as dangerous as not having a changelog"), and calls GitHub Releases "non-portable" and less discoverable than a `CHANGELOG.md`. Rewriting a changelog after the fact is fine. Yanked releases stay listed, marked `[YANKED]`.

Its weakness is the one Joel named first: nothing makes anyone write it, and everyone edits the same `Unreleased` section, so parallel branches conflict.

### Derived from commits: Conventional Commits and git-cliff

[Conventional Commits](/sources/2026-11-21-2fffc71b-git-cliff-docs-and-conventional-commits-100.md) puts a type in every commit subject (`feat:`, `fix:`, `!` for breaking), and tools such as git-cliff, release-please and semantic-release turn the history into a changelog and a version bump. It is fully automatic, which is its appeal, but:

- An entry is a commit subject, written for developers at the moment of the commit, not for readers at release.
- It depends on history discipline: git-cliff recommends squash merges so a PR is one commit, and the fix for a mistyped commit is `git rebase -i`. This is the "complicated git workflow machinery" Joel wants to avoid.
- It is one level deep. A commit can link an issue, but nothing structures what lies behind it.

### Fragment files: Changesets, towncrier, reno

Each change adds a small file beside the code; the release rolls them up.

- **[Changesets](/sources/2026-11-21-435c0b83-changesets-a-detailed-explanation-and-adding-a.md)** (JavaScript, monorepo-first) frames a changeset as an "intent to change": a bump type plus a markdown summary in `.changeset/<id>.md`, written "when submitting a PR (when it is fresh in your mind), not when you eventually go to batch and release". It asks for WHAT, WHY and HOW-to-update. Its three tools are creation, consumption at version time, and "detection + surfacing of changesets in PRs". Its explanation also says git "is a bad place to store this information, as it discourages writing detailed change descriptions"; the farm is the answer to that complaint.
- **[towncrier](/sources/2026-11-21-82ac39ee-towncrier-tutorial-and-check-command-2690.md)** (Python) names fragments by issue id and type (`1234.bugfix`; `+name` for one with no issue). `towncrier check` fails a branch that adds no fragment, and is skipped when the branch edits the news file itself (a release branch). The build deletes the fragments with `git rm` unless `--keep`.
- **[reno](/sources/2026-11-21-2cb3322d-reno-openstack-design-constraints-and-usage.md)** (OpenStack) keeps its notes forever and **computes which release each note belongs to** by scanning the branch's history: a note belongs to the first release tag that contains it. A note can be corrected later and keeps its release ("pulls the updated contents from the newest commit while retaining the note's association with the release where it was first introduced"). It is built for trunk-based development with fixes cherry-picked to stable branches, and each branch reports its own releases. Its design inputs include "notes may need to change over time" and "must not slow down to a crawl over years".

Fragments are the closest fit to yaks. They capture the entry while the change is fresh, a check enforces them, and parallel branches never conflict because each change is its own file. Their weakness is depth: a fragment is a line or a paragraph, and Changesets and towncrier delete it once it is rolled up, so the changelog becomes the only record.

### Generated from the tracker: GitHub release notes

[GitHub's generated release notes](/sources/2026-11-21-f66f0491-github-automatically-generated-release-notes.md) list the merged pull requests between two tags, grouped by label (`.github/release.yml`), with contributors and a compare link. They link back to the discussion, but an entry is a PR title, written for reviewers, and the result lives only on GitHub.

## What carries over to yaks

| Idea | From | In yaks |
|---|---|---|
| Write the entry when the change is fresh, not at release | Changesets, towncrier | a change line written when the yak is shorn |
| A check that a change has an entry or an explicit "none" | `towncrier check`, Changesets' PR bot | `yaks land` and a release check list shorn yaks without one |
| One file per change, so branches never conflict | all fragment tools | a yak is already one file, and not on any code branch |
| Keep the notes; compute release membership from history | reno | a yak shipped in the first tag whose range has a commit naming it |
| Correct an entry after release | reno, Keep a Changelog | a newer change note; the latest wins |
| Sections, ISO dates, latest first, `[YANKED]` | Keep a Changelog | the generated `CHANGELOG.md` follows it |
| Group by type | everyone | yak `type`, and families: an umbrella yak heads its children |

What none of them has, and yaks does: each entry links a yak, and the yak links the next layer down (design and research doks, sources, evidence attachments, then `yaks commits`). The changelog can stay a skim and still be the entry point for someone who needs to dig.

## This repository's own history

Counted on 2026-11-21 over the code branch's tags (`v0.0.1` 2026-08-23 to `v0.0.40`): no tag message says anything ("Bump version to 0.0.N"), and there is no `CHANGELOG.md`. What each release contained was written by hand into its release yak (yaks-bdec: "Contents since v0.0.30: yaks-be20, yaks-0370, …"). A scheduled "Write the release notes" yak, yaks-0d7c, was slaughtered undone.

Commits naming a yak in their subject, per release range:

- `v0.0.1`–`v0.0.4`: none (0 of 99): there is no yak to link, so a backfill has only commit subjects for these.
- `v0.0.5`–`v0.0.20`: most (418 of 480, about 87%).
- `v0.0.21`–`v0.0.40`: nearly all (242 of 256, about 95%).

So computing membership from commit messages works for almost all of the history, and the commits that name no yak are what a release check should list.

## Images at a commit, through LFS

Joel asked whether a changelog or dok can show an image as it was at a commit when the farm stores images in Git LFS. It can, locally and on GitHub. The probe is attached to yaks-a4d8 as [lfs-probe.md](/yaks/yaks/yaks-a4d8/lfs-probe.md):

- Locally, `git show <rev>:<path>` returns the 131-byte LFS pointer; `git cat-file -p <rev>:<path> | git lfs smudge -- <path>` returns the image. yaks' `@rev` reader runs `git show`, so it needs the smudge step.
- On GitHub, `github.com/<owner>/<repo>/raw/<sha>/<path>` and `media.githubusercontent.com/media/<owner>/<repo>/<sha>/<path>` serve the image at that commit (200, `image/png`, full size); `raw.githubusercontent.com/<owner>/<repo>/<sha>/<path>` serves the pointer.

An image pinned this way stays what it was; an unpinned link follows the current file. That gives the explicit choice Joel asked for, intentionally updated vs historical snapshot, with no file-layout convention.

## Limits

- Changesets' v3 docs were not available at the time of reading (the repository's `main` is v3 development); the saved explanation is the v2 one, which is what most projects run.
- release-please and semantic-release were not read separately; they follow the Conventional Commits model above.
- The GitHub LFS check covered `raw` URLs at a full sha. Not checked: a relative image link inside a markdown file viewed at `blob/<sha>/…`, and hosts other than GitHub.

## Open questions

- [ ] Does a relative LFS image link in a markdown file viewed on GitHub at `blob/<sha>/…` render the image, or only `raw/<sha>` URLs?
- [ ] Do GitLab and Gitea serve LFS content for a raw URL at a commit, as GitHub does?
