package yass

import (
	"fmt"
	"os"
	"os/exec"
	"path"
	"path/filepath"
	"regexp"
	"sort"
	"strings"
)

// The optional pre-commit check: progress travels with code; intent changes get their own commit.
//
// It warns when one commit changes code and also edits existing intent in a yass folder: prd.md,
// design.md, plan.md beyond marking boxes, a yass folder's queue.md, or a change's title, Goal or
// Acceptance. Writing new intent files is fine. Marking boxes and appending to Log and Decisions
// alongside code is expected and never flagged. Editing the archive is always flagged: it's append-only.

var (
	hookBoxRE = regexp.MustCompile(`^(\s*[-*+]\s+)\[[ xX/]\]`) // starting, ticking or unticking is progress; dropping ([-]) is not
	h1RE      = regexp.MustCompile(`(?m)^# .*$`)
)

type entry struct{ status, path, oldPath string }

type place struct {
	kind   string // "changes", "archive", "queue", "meta", or "" for code
	fname  string
	folder string // for the archive: the archived change's folder
}

type hook struct {
	top      string   // the repo's top, where git's paths start
	prefixes []string // yass folders inside the repo that yass.yaml files point to, repo-relative
	ignored  []string // folders a yass.yaml ignores, repo-relative: their files are ordinary files
}

func newHook() *hook {
	h := &hook{}
	// Only the yass folders, seen from the top (git's paths are relative to it); no changes are read.
	r := &Repo{}
	r.Top = repoTop(getwd())
	r.Cwd = r.Top
	h.top = r.Top
	r.findRoots()
	for _, d := range r.Ignored {
		rel, _ := filepath.Rel(r.Top, d)
		h.ignored = append(h.ignored, filepath.ToSlash(rel))
	}
	for _, root := range r.Roots {
		if root.Config != "" && within(root.Dir, r.Top) && root.Dir != r.Top {
			rel, _ := filepath.Rel(r.Top, root.Dir)
			h.prefixes = append(h.prefixes, filepath.ToSlash(rel))
		}
	}
	return h
}

func (h *hook) where(p string) place {
	for _, d := range h.ignored {
		if strings.HasPrefix(p, d+"/") {
			return place{}
		}
	}
	parts := strings.Split(p, "/")
	if path.Base(p) == ConfigName {
		return place{kind: "meta"}
	}
	for _, pre := range h.prefixes {
		if !strings.HasPrefix(p, pre+"/") {
			continue
		}
		rest := strings.Split(strings.TrimPrefix(p, pre+"/"), "/")
		if len(rest) == 1 && rest[0] == QueueName {
			return place{kind: "queue", fname: QueueName}
		}
		if len(rest) >= 2 && (rest[0] == "changes" || rest[0] == "archive") {
			return place{rest[0], parts[len(parts)-1], pre + "/" + changeFolder(rest)}
		}
		return place{kind: "meta"}
	}
	for i := 0; i < len(parts)-1; i++ {
		if parts[i] == "yass" && (parts[i+1] == "changes" || parts[i+1] == "archive") {
			return place{parts[i+1], parts[len(parts)-1], strings.Join(parts[:i+1], "/") + "/" + changeFolder(parts[i+1:])}
		}
	}
	if len(parts) >= 2 && parts[len(parts)-2] == "yass" && parts[len(parts)-1] == QueueName {
		return place{kind: "queue", fname: QueueName}
	}
	if len(parts) >= 2 && parts[len(parts)-2] == "yass" && parts[len(parts)-1] == "README.md" {
		return place{kind: "meta"}
	}
	return place{}
}

var archivedLineRE = regexp.MustCompile(`(?m)^archived:.*\n`)

// intoMonths is the migration of a flat archived change into archive/<YYYY>/<MM>/: the same
// change, moved from archive/<name>/ to a month folder, with at most an archived: stamp added.
// Git sees the move as a rename, or, when the stamp changes a small file too much, as a deletion
// here and an addition there; both count.
func (h *hook) intoMonths(e entry, pl place, entries []entry, oldRef, newRef string) bool {
	oldPath, newPath := e.oldPath, e.path
	switch {
	case strings.HasPrefix(e.status, "R"):
	case e.status == "D":
		oldPath, newPath = e.path, ""
		for _, a := range entries {
			if a.status == "A" && h.monthOf(h.where(e.path), h.where(a.path)) && path.Base(a.path) == path.Base(e.path) &&
				strings.TrimPrefix(a.path, h.where(a.path).folder) == strings.TrimPrefix(e.path, pl.folder) {
				newPath = a.path
			}
		}
		if newPath == "" {
			return false
		}
	case e.status == "A":
		return false // the addition side is new to the archive, which is fine on its own
	default:
		return false
	}
	if !h.monthOf(h.where(oldPath), h.where(newPath)) {
		return false
	}
	if e.status == "R100" {
		return true
	}
	if oldRef == "" {
		return false
	}
	old, cur := show(oldRef, oldPath), archivedLineRE.ReplaceAllString(show(newRef, newPath), "")
	// The stamp may have come with frontmatter of its own, when the change had none.
	return old == cur || old == strings.TrimPrefix(cur, "---\n---\n")
}

// evicting is yass evict's commit: an archived change deleted while the same commit adds its month's
// .evicted file naming it (evicting), or that new .evicted file itself (manifest).
func (h *hook) evicting(e entry, pl place, entries []entry, newRef string) (evicting, manifest bool) {
	if strings.HasSuffix(e.path, evictedExt) {
		return false, e.status == "A" && monthRE.MatchString(strings.TrimSuffix(path.Base(e.path), evictedExt))
	}
	if e.status != "D" {
		return false, false
	}
	month := path.Dir(pl.folder) // <archive>/<YYYY>/<MM>
	if !monthRE.MatchString(path.Base(month)) || !yearRE.MatchString(path.Base(path.Dir(month))) {
		return false, false
	}
	for _, a := range entries {
		if a.status == "A" && a.path == month+evictedExt {
			for _, l := range lines(show(newRef, a.path)) {
				if strings.TrimSpace(strings.TrimPrefix(strings.TrimSpace(l), "- ")) == path.Base(pl.folder) {
					return true, false
				}
			}
		}
	}
	return false, false
}

// monthOf says whether to is from's flat archived change, under archive/<YYYY>/<MM>/.
func (h *hook) monthOf(from, to place) bool {
	month := path.Dir(to.folder) // <archive>/<YYYY>/<MM>
	return from.kind == "archive" && to.kind == "archive" && path.Base(to.folder) == path.Base(from.folder) &&
		path.Dir(from.folder) == path.Dir(path.Dir(month)) && monthRE.MatchString(path.Base(month)) &&
		yearRE.MatchString(path.Base(path.Dir(month)))
}

// changeFolder is the change's folder in a path that starts at changes/ or archive/: the folder
// right under it, or, in the archive, the one under <YYYY>/<MM>/.
func changeFolder(rest []string) string {
	n := 2
	if rest[0] == "archive" && len(rest) >= 4 && yearRE.MatchString(rest[1]) && monthRE.MatchString(rest[2]) {
		n = 4
	}
	if n > len(rest) {
		n = len(rest)
	}
	return strings.Join(rest[:n], "/")
}

func norm(text string) []string {
	var out []string
	for _, l := range lines(commentRE.ReplaceAllString(text, "")) {
		l = strings.TrimRight(hookBoxRE.ReplaceAllString(l, "${1}[ ]"), " \t")
		if strings.TrimSpace(l) != "" {
			out = append(out, l)
		}
	}
	return out
}

func intentOfChangeMD(text string) string {
	parts := []string{}
	if m := h1RE.FindString(text); m != "" {
		parts = append(parts, m)
	}
	parts = append(parts, norm(section(text, "Goal"))...)
	parts = append(parts, "--")
	parts = append(parts, norm(section(text, "Acceptance"))...)
	return strings.Join(parts, "\n")
}

// show is a file at a commit; ref "" is the index (what's staged).
func show(ref, p string) string {
	out, _ := git("", "show", ref+":"+p)
	return normalize(out)
}

func (h *hook) check(entries []entry, oldRef, newRef string) []string {
	places := map[string]place{}
	hasCode := false
	for _, e := range entries {
		places[e.path] = h.where(e.path)
		if places[e.path].kind == "" {
			hasCode = true
		}
	}
	var out []string
	for _, e := range entries {
		pl := places[e.path]
		if pl.kind == "" || pl.kind == "meta" || pl.fname == ".gitkeep" {
			continue
		}
		fromChanges := strings.HasPrefix(e.status, "R") && h.where(e.oldPath).kind == "changes"
		if pl.kind == "queue" {
			if hasCode && e.status[0] != 'A' {
				out = append(out, e.path+": queue.md is intent; reorder in a commit without code")
			}
			continue
		}
		if pl.kind == "archive" {
			if h.intoMonths(e, pl, entries, oldRef, newRef) {
				if hasCode {
					out = append(out, e.path+": move the archive into months in its own commit, not alongside code")
				}
				continue
			}
			if evicting, manifest := h.evicting(e, pl, entries, newRef); evicting || manifest {
				if hasCode && manifest {
					out = append(out, e.path+": evict archived months in their own commit, not alongside code")
				}
				continue
			}
			addedToOld := false
			if e.status[0] == 'A' && oldRef != "" {
				_, addedToOld = git("", "cat-file", "-e", oldRef+":"+pl.folder)
			}
			if strings.HasPrefix(e.status, "M") || strings.HasPrefix(e.status, "D") || addedToOld ||
				(strings.HasPrefix(e.status, "R") && !fromChanges) {
				out = append(out, e.path+": the archive is append-only; follow-up work is a new change with `follows:`")
			} else if fromChanges && hasCode {
				out = append(out, e.path+": archive a change in its own commit, not alongside code")
			}
			continue
		}
		if !hasCode || e.status[0] == 'A' {
			continue // new intent isn't a moved goalpost
		}
		oldP := e.oldPath
		if oldP == "" {
			oldP = e.path
		}
		old, cur := "", ""
		if oldRef != "" {
			old = show(oldRef, oldP)
		}
		if e.status[0] != 'D' {
			cur = show(newRef, e.path)
		}
		switch pl.fname {
		case "prd.md", "design.md":
			out = append(out, fmt.Sprintf("%s: %s is intent; change it in a commit without code", e.path, pl.fname))
		case "plan.md":
			if e.status[0] == 'D' || strings.Join(norm(old), "\n") != strings.Join(norm(cur), "\n") {
				out = append(out, e.path+": with code, only mark boxes in plan.md; revise the plan in its own commit")
			}
		case "change.md":
			if e.status[0] == 'D' {
				out = append(out, e.path+": removing a change alongside code")
			} else if intentOfChangeMD(old) != intentOfChangeMD(cur) {
				out = append(out, e.path+": the title, Goal or Acceptance changed; that's intent, so give it its own commit")
			}
		}
	}
	if hasCode {
		out = append(out, h.outsideThePiece(entries, places)...)
	}
	return out
}

// outsideThePiece flags a commit with code that reaches past one piece of a large change: it edits
// the parent's own files, or files in two of its pieces. A piece's branch changes only its own
// folder, so parallel branches on different pieces never touch the same file. New files are fine.
func (h *hook) outsideThePiece(entries []entry, places map[string]place) []string {
	var out []string
	pieces := map[string]map[string]bool{} // parent folder → pieces touched
	for _, e := range entries {
		pl := places[e.path]
		if pl.kind != "changes" || e.status[0] == 'A' || !h.hasPieces(pl.folder) {
			continue
		}
		rel := strings.TrimPrefix(e.path, pl.folder+"/")
		piece, _, inside := strings.Cut(rel, "/")
		if !inside || !isFile(filepath.Join(h.top, filepath.FromSlash(pl.folder), piece, "change.md")) {
			// the parent's own files, its assets/ included: anything not in a piece
			out = append(out, e.path+": a large change's own files change in commits without code; a piece's branch changes only its own folder")
			continue
		}
		if pieces[pl.folder] == nil {
			pieces[pl.folder] = map[string]bool{}
		}
		pieces[pl.folder][piece] = true
	}
	for folder, ps := range pieces {
		if len(ps) > 1 {
			var names []string
			for p := range ps {
				names = append(names, p)
			}
			sort.Strings(names)
			out = append(out, fmt.Sprintf("%s: this commit with code changes pieces %s; each piece's branch changes only its own folder",
				folder, strings.Join(names, " and ")))
		}
	}
	return out
}

// hasPieces says whether a change folder holds pieces: subfolders with a change.md.
func (h *hook) hasPieces(folder string) bool {
	m, _ := filepath.Glob(filepath.Join(h.top, filepath.FromSlash(folder), "*", "change.md"))
	return len(m) > 0
}

// nameStatus lists what a diff changed. With -z, paths come raw (no quoting of non-ASCII names) and
// NUL-separated: "M\0path\0", or "R100\0old\0new\0" for renames and copies.
func nameStatus(args ...string) []entry {
	out, _ := git("", append([]string{"diff", "--name-status", "-M", "-z"}, args...)...)
	f := strings.Split(strings.TrimSuffix(out, "\x00"), "\x00")
	var rows []entry
	for i := 0; i+1 < len(f); {
		status := f[i]
		if (status[0] == 'R' || status[0] == 'C') && i+2 < len(f) {
			rows = append(rows, entry{status, f[i+2], f[i+1]})
			i += 3
		} else {
			rows = append(rows, entry{status, f[i+1], ""})
			i += 2
		}
	}
	return rows
}

func emptyTree() string {
	cmd := exec.Command("git", "hash-object", "-t", "tree", "--stdin")
	cmd.Stdin = strings.NewReader("")
	out, _ := cmd.Output()
	return strings.TrimSpace(string(out))
}

func cmdHook(a *args) int {
	switch strings.ToLower(os.Getenv("YASS_HOOK")) {
	case "off", "0", "false":
		return 0
	}
	strict := a.b["strict"] || (os.Getenv("YASS_STRICT") != "" && os.Getenv("YASS_STRICT") != "0")
	switch {
	case a.b["prepare-msg"]:
		if len(a.pos) > 0 {
			source := ""
			if len(a.pos) > 1 {
				source = a.pos[1]
			}
			prepareMsg(a.pos[0], source)
		}
		return 0
	case a.b["commit-msg"]:
		if len(a.pos) == 0 {
			return 0
		}
		return commitMsg(a.pos[0])
	}
	h := newHook()
	var found, trailers []string
	trailerFail, namesChecked := false, true
	if rng, ok := a.v["range"]; ok {
		shas, ok := git("", "rev-list", "--reverse", "--no-merges", rng)
		if rng == "" || !ok {
			fmt.Fprintf(os.Stderr, "yass: can't read the range '%s'. In CI, fetch history first (e.g. actions/checkout with fetch-depth: 0).\n", rng)
			return 2
		}
		for _, sha := range strings.Fields(shas) {
			parent, _ := git("", "rev-parse", "-q", "--verify", sha+"^")
			parent = strings.TrimSpace(parent)
			base := parent
			if base == "" {
				base = emptyTree()
			}
			if w := h.check(nameStatus(base, sha), parent, sha); len(w) > 0 {
				subj, _ := git("", "log", "-1", "--format=%h %s", sha)
				for _, x := range w {
					found = append(found, strings.TrimSpace(subj)+": "+x)
				}
			}
		}
		if r := loadRepo(); r.usesSeparatePlans() {
			trailers, namesChecked = r.rangeTrailers(strings.Fields(shas), a.b["require-trailers"])
			trailerFail = len(trailers) > 0 && strict
		}
	} else {
		if _, merging := git("", "rev-parse", "-q", "--verify", "MERGE_HEAD"); merging {
			return 0
		}
		head := ""
		if _, ok := git("", "rev-parse", "-q", "--verify", "HEAD"); ok {
			head = "HEAD"
		}
		base := head
		if base == "" {
			base = emptyTree()
		}
		found = h.check(nameStatus("--cached", base), head, "")
	}
	if len(trailers) > 0 {
		verb := "heads-up"
		if trailerFail {
			verb = "failing"
		}
		fmt.Fprintf(os.Stderr, "yass: %s: commits whose `yass:` trailers need a look (the plans are kept apart from the code)\n", verb)
		for _, t := range trailers {
			fmt.Fprintf(os.Stderr, "  - %s\n", t)
		}
	}
	if !namesChecked {
		fmt.Fprintln(os.Stderr, "yass: the plans folder isn't here, so trailers were checked for form only; change names weren't checked")
	}
	if len(found) == 0 {
		if trailerFail {
			return 1
		}
		return 0
	}
	verb := "heads-up"
	if strict {
		verb = "refusing"
	}
	fmt.Fprintf(os.Stderr, "yass: %s: code and intent are mixed (progress travels with code; intent gets its own commit)\n", verb)
	for _, f := range found {
		fmt.Fprintf(os.Stderr, "  - %s\n", f)
	}
	if !strict {
		fmt.Fprintln(os.Stderr, "  (a warning only; YASS_STRICT=1 makes it an error, YASS_HOOK=off skips it)")
		if trailerFail {
			return 1
		}
		return 0
	}
	return 1
}
