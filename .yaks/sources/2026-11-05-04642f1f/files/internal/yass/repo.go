package yass

import (
	"errors"
	"fmt"
	"io/fs"
	"os"
	"os/exec"
	"path"
	"path/filepath"
	"regexp"
	"sort"
	"strconv"
	"strings"
	"time"
)

// skipDirs are only used outside git; inside git, .gitignore decides.
var skipDirs = map[string]bool{".git": true, "node_modules": true, "__pycache__": true}

var yassDirRE = regexp.MustCompile(`^(?:(.*)/)?yass/(?:changes|archive)/`)

func git(dir string, args ...string) (string, bool) {
	cmd := exec.Command("git", args...)
	cmd.Dir = dir
	if dir != "" {
		// Asked about a particular folder, git should find that folder's repo. Hooks in a linked
		// worktree run with GIT_DIR set, which would point every git call at the code worktree,
		// even one meant for a plans repo elsewhere. Calls with no folder keep it (and the
		// hook's GIT_INDEX_FILE).
		cmd.Env = gitEnvFree()
	}
	out, err := cmd.Output()
	return string(out), err == nil
}

// gitEnvFree is the environment without the variables that pin git to a repo.
func gitEnvFree() []string {
	var out []string
	for _, kv := range os.Environ() {
		switch k, _, _ := strings.Cut(kv, "="); k {
		case "GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_COMMON_DIR", "GIT_PREFIX":
			continue
		}
		out = append(out, kv)
	}
	return out
}

// canon makes a path absolute and resolves symlinks, so paths compare reliably. For a path that
// doesn't exist yet, it resolves the deepest folder that does and keeps the rest: on macOS, a new
// folder under /var/... must still compare as inside a repo under /private/var/....
func canon(p string) string {
	if a, err := filepath.Abs(p); err == nil {
		p = a
	}
	p = filepath.Clean(p)
	for dir, rest := p, ""; ; {
		if r, err := filepath.EvalSymlinks(dir); err == nil {
			return filepath.Join(r, rest)
		}
		parent := filepath.Dir(dir)
		if parent == dir {
			return p
		}
		rest = filepath.Join(filepath.Base(dir), rest)
		dir = parent
	}
}

func within(p, dir string) bool {
	return p == dir || strings.HasPrefix(p, dir+string(filepath.Separator))
}

func getwd() string {
	wd, _ := os.Getwd()
	return canon(wd)
}

// repoTop is the git repository root; outside git, the highest folder above here with a yass folder.
func repoTop(cwd string) string {
	if out, ok := git(cwd, "rev-parse", "--show-toplevel"); ok {
		return canon(filepath.FromSlash(strings.TrimSpace(out)))
	}
	here, best := cwd, cwd
	for {
		if isDir(filepath.Join(here, "yass", "changes")) || isDir(filepath.Join(here, "yass", "archive")) ||
			isFile(filepath.Join(here, ConfigName)) {
			best = here
		}
		parent := filepath.Dir(here)
		if parent == here {
			return best
		}
		here = parent
	}
}

// Root is one yass folder and the part of the repo it serves.
type Root struct {
	Dir    string // the yass folder: changes/, archive/, README.md
	Owner  string // the folder it serves: where yass/ or the yass.yaml sits
	Config string // the yass.yaml that points to Dir, if any
	Branch string // the code branch that counts as merged, from yass.yaml
	// Included: a team folder git never sees, named in the clone's git config (yass.include)
	Included bool
	Queue    []string // the changes queue.md ranks, top first, as written
}

// keep is how many archived changes the yass folder should keep in its working tree (a soft limit).
func (root *Root) keep() int {
	if root.Config != "" {
		if c, _, err := loadConfig(root.Config); err == nil && c.Archive.Keep > 0 {
			return c.Archive.Keep
		}
	}
	return DefaultKeep
}

// rank is a change's place in its yass folder's queue.md; changes it doesn't list come after all that it does.
func (root *Root) rank(name string) int {
	for i, n := range root.Queue {
		if n == name {
			return i
		}
	}
	return len(root.Queue)
}

// created is when a change was made, for ordering: its created: stamp, else its folder's date,
// which sorts before any stamp from the same day.
func (c *Change) created() string {
	if t, err := time.Parse(time.RFC3339, c.Meta["created"]); err == nil {
		return t.UTC().Format(time.RFC3339)
	}
	if nameRE.MatchString(c.Name) {
		return c.Name[:10]
	}
	return ""
}

// byCreated orders changes oldest first; ties keep their order (by name, as loaded).
func byCreated(cs []*Change) {
	sort.SliceStable(cs, func(i, j int) bool { return cs[i].created() < cs[j].created() })
}

type fileBox struct{ file, mark, text string }

// Change is one change folder (or a piece inside one).
type Change struct {
	Path, Name, Head, Title string
	Root                    *Root
	Parent                  *Change
	Meta                    map[string]string
	Pieces                  []*Change
	Large                   bool
	Archived                bool
	Evicted                 bool              // archived, then evicted: only its month's .evicted file names it
	Deps                    []*Change         // the changes blocked: names, when it names changes rather than a reason
	files                   map[string]string // an evicted change read back from git: its .md files by name
}

func newChange(p string, root *Root, parent *Change) *Change {
	c := nameOnly(p, root, parent)
	c.read()
	return c
}

// nameOnly is a change known by its folder alone, its files unread: how archived changes load, so
// commands that only need their names (follows:, blocked:, the queue) stay fast however big the
// archive gets. readArchived reads them when a command shows what's in them.
func nameOnly(p string, root *Root, parent *Change) *Change {
	name := filepath.Base(p)
	return &Change{Path: p, Root: root, Parent: parent, Name: name, Title: name, Meta: map[string]string{}}
}

func (c *Change) read() {
	p := c.Path
	c.setHead(readText(filepath.Join(p, "change.md")))
	c.Large = exists(filepath.Join(p, "prd.md")) || exists(filepath.Join(p, "plan.md"))
}

// setHead takes in a change's change.md: its frontmatter and title.
func (c *Change) setHead(text string) {
	c.Head = text
	var body string
	c.Meta, body = frontmatter(c.Head)
	c.Title = c.Name
	if m := titleRE.FindStringSubmatch(body); m != nil {
		c.Title = strings.TrimSpace(m[1])
	}
}

// readArchived reads the archived changes' files, for commands that show what's in them, and
// warns about any it can't read.
func (r *Repo) readArchived() {
	r.archivedPieces()
	for _, c := range flatten(r.Archived) {
		if c.Evicted {
			continue
		}
		head := filepath.Join(c.Path, "change.md")
		if _, err := os.ReadFile(head); err != nil && !os.IsNotExist(err) {
			r.warn("%s: can't read it: %v", r.disp(head), errors.Unwrap(err))
		}
		c.read()
		if t := c.Meta["archived"]; t != "" && c.Parent == nil {
			if _, err := time.Parse(time.RFC3339, t); err != nil {
				r.warn("%s: archived '%s' isn't a time like 2026-10-05T14:32:07Z", r.disp(c.Path), t)
			}
		}
	}
	for _, c := range r.Archived {
		byCreated(c.Pieces)
	}
	byCreated(r.Archived)
}

func mdFiles(dir string) []string {
	entries, _ := os.ReadDir(dir)
	var out []string
	for _, e := range entries {
		if !e.IsDir() && strings.HasSuffix(e.Name(), ".md") {
			out = append(out, e.Name())
		}
	}
	sort.Strings(out)
	return out
}

// ownBoxes are the boxes in this folder's own .md files (pieces count separately).
func (c *Change) ownBoxes() []fileBox {
	var out []fileBox
	if c.files != nil {
		names := make([]string, 0, len(c.files))
		for f := range c.files {
			names = append(names, f)
		}
		sort.Strings(names)
		for _, f := range names {
			for _, b := range boxes(c.files[f]) {
				out = append(out, fileBox{f, b.mark, b.text})
			}
		}
		return out
	}
	for _, f := range mdFiles(c.Path) {
		for _, b := range boxes(read(filepath.Join(c.Path, f))) {
			out = append(out, fileBox{f, b.mark, b.text})
		}
	}
	return out
}

// allOpen lists open and in-progress boxes in this folder's own files and its pieces': the same
// boxes tally and progress count, so status and archive always agree.
var (
	deliversRE  = regexp.MustCompile(`(?i)\bdelivers\b`)
	acRefRE     = regexp.MustCompile(`\bAC(\d+)(?:\s*[–-]\s*AC(\d+))?`)
	criterionRE = regexp.MustCompile(`^AC(\d+)\b`)
)

// deliveredIDs reads a piece's "Delivers AC1, AC3–AC5 in plan.md" box: the parent's criteria it
// names, as lists and ranges, ignoring the words around them ("and AC19's preview card" counts AC19).
func deliveredIDs(text string) []int {
	loc := deliversRE.FindStringIndex(text)
	if loc == nil {
		return nil
	}
	var out []int
	for _, m := range acRefRE.FindAllStringSubmatch(text[loc[1]:], -1) {
		from, _ := strconv.Atoi(m[1])
		to := from
		if m[2] != "" {
			to, _ = strconv.Atoi(m[2])
		}
		for n := from; n <= to && n-from < 1000; n++ {
			out = append(out, n)
		}
	}
	return out
}

// delivered is the parent's criteria (by number) that pieces deliver, and whether every piece
// delivering each one is done: a criterion is met once all the pieces that name it are.
func (c *Change) delivered() map[int]bool {
	if c.Parent != nil || len(c.Pieces) == 0 {
		return nil
	}
	by := map[int]map[*Change]bool{}
	for _, p := range c.Pieces {
		for _, b := range p.ownBoxes() {
			for _, n := range deliveredIDs(b.text) {
				if by[n] == nil {
					by[n] = map[*Change]bool{}
				}
				by[n][p] = true
			}
		}
	}
	out := map[int]bool{}
	for n, ps := range by {
		all := true
		for p := range ps {
			all = all && p.state() == "done"
		}
		out[n] = all
	}
	return out
}

// countedBoxes are a folder's own boxes as progress counts them: a parent's criterion in plan.md
// that its pieces have delivered counts as done, so no closing commit is needed to tick it.
func (c *Change) countedBoxes() []fileBox {
	bs := c.ownBoxes()
	d := c.delivered()
	if len(d) == 0 {
		return bs
	}
	for i, b := range bs {
		if b.file != "plan.md" || !isOpen(b.mark) {
			continue
		}
		if m := criterionRE.FindStringSubmatch(b.text); m != nil {
			if n, _ := strconv.Atoi(m[1]); d[n] {
				bs[i].mark = markDone
			}
		}
	}
	return bs
}

var boxLineRE = regexp.MustCompile(`^(\s*[-*+]\s+\[)([ /])(\]\s+)(AC(\d+)\b.*)$`)

// markDelivered ticks the open criteria in a plan.md that the pieces delivered, leaving every other
// line as it was, comments included.
func markDelivered(text string, delivered map[int]bool) string {
	ls := strings.SplitAfter(text, "\n")
	inComment := false
	for i, l := range ls {
		body := strings.TrimRight(l, "\r\n")
		wasComment := inComment
		if strings.Contains(body, "<!--") {
			inComment = true
		}
		if strings.Contains(body, "-->") {
			inComment = false
			wasComment = true
		}
		if wasComment || inComment {
			continue
		}
		if m := boxLineRE.FindStringSubmatch(body); m != nil {
			if n, _ := strconv.Atoi(m[5]); delivered[n] {
				ls[i] = m[1] + "x" + m[3] + m[4] + l[len(body):]
			}
		}
	}
	return strings.Join(ls, "")
}

func (c *Change) allOpen() []fileBox {
	var out []fileBox
	for _, b := range c.countedBoxes() {
		if isOpen(b.mark) {
			out = append(out, b)
		}
	}
	for _, p := range c.Pieces {
		for _, b := range p.allOpen() {
			out = append(out, fileBox{p.Name + "/" + b.file, b.mark, b.text})
		}
	}
	return out
}

// tally counts the boxes in this folder and its pieces: open (or in progress), done, and dropped.
func (c *Change) tally() (open, done, dropped int) {
	for _, b := range c.countedBoxes() {
		switch b.mark {
		case markDone:
			done++
		case markDropped:
			dropped++
		default:
			open++
		}
	}
	for _, p := range c.Pieces {
		o, d, x := p.tally()
		open, done, dropped = open+o, done+d, dropped+x
	}
	return
}

// progress is (done, total) over this folder and its pieces; dropped boxes don't count, in-progress ones aren't done.
func (c *Change) progress() (int, int) {
	open, done, _ := c.tally()
	return done, open + done
}

func (c *Change) blocked() string { return c.Meta["blocked"] }

// met: a change others wait on is done (every box checked) or archived.
func (c *Change) met() bool { return c.Archived || c.state() == "done" }

// blockedBy is why a change can't proceed: a reason for a human, or the changes it's still waiting on.
// A blocked: that names only changes that are all met blocks nothing.
func (c *Change) blockedBy() (reason string, waiting []*Change) {
	if c.Deps == nil {
		return c.blocked(), nil
	}
	for _, d := range c.Deps {
		if !d.met() {
			waiting = append(waiting, d)
		}
	}
	return "", waiting
}

func labels(cs []*Change) string {
	var out []string
	for _, c := range cs {
		out = append(out, c.label())
	}
	return strings.Join(out, ", ")
}

func (c *Change) log() string { return commentRE.ReplaceAllString(section(c.Head, "Log"), "") }

// started: a Log entry, or a box in progress or done, here or in a piece. Shaping and planning write none.
func (c *Change) started() bool {
	if logEntryRE.MatchString(c.log()) {
		return true
	}
	for _, b := range c.countedBoxes() {
		if b.mark == markDone || b.mark == markWorking {
			return true
		}
	}
	for _, p := range c.Pieces {
		if p.started() {
			return true
		}
	}
	return false
}

func (c *Change) nextStep() string {
	if m := nextRE.FindAllStringSubmatch(c.log(), -1); len(m) > 0 {
		return strings.TrimSpace(m[len(m)-1][1])
	}
	own := c.countedBoxes()
	for _, mark := range []string{markWorking, markOpen} {
		for _, b := range own {
			if b.mark == mark {
				return b.text
			}
		}
	}
	return ""
}

// state is "not started", "in progress", or "done" (every box done or dropped: ready to archive).
func (c *Change) state() string {
	if open, done, dropped := c.tally(); open == 0 && done+dropped > 0 {
		return "done"
	}
	if c.started() {
		return "in progress"
	}
	return "not started"
}

func (c *Change) label() string {
	if c.Parent == nil {
		return c.Name
	}
	return c.Parent.Name + "/" + c.Name
}

// Repo is every yass folder this repository uses, and the changes in them.
type Repo struct {
	Top, Cwd         string
	Roots            []*Root
	Ignored          []string // folders a yass.yaml ignores
	Configs          []string // the yass.yaml files that count, wherever they point
	Warnings         []string
	Notes            []string // worth knowing, but not a problem: they don't fail --strict
	Linked           bool     // a yass.yaml's folder wasn't found from this linked worktree
	Active, Archived []*Change
	piecesFound      bool              // whether archivedPieces has run
	absent           map[string]string // changes git has that aren't on disk (sparse checkouts), by name
	merged           map[*Root]string  // the merged branch per yass folder, once looked up
	codeDir          *string           // this code repo's git common dir, once looked up
}

// findRepo finds the repo and its yass folders, without reading any changes.
func findRepo() *Repo {
	r := &Repo{Cwd: getwd()}
	r.Top = repoTop(r.Cwd)
	r.findRoots()
	return r
}

func loadRepo() *Repo {
	r := findRepo()
	r.link()
	for _, root := range r.Roots {
		root.Queue = parseQueue(readText(filepath.Join(root.Dir, QueueName)))
		r.Active = append(r.Active, r.load(root, "changes", true)...)
		r.Archived = append(r.Archived, r.load(root, "archive", false)...)
	}
	r.check()
	return r
}

func (r *Repo) warn(format string, a ...any) {
	r.Warnings = append(r.Warnings, fmt.Sprintf(format, a...))
}

func (r *Repo) note(format string, a ...any) {
	r.Notes = append(r.Notes, fmt.Sprintf(format, a...))
}

// noRoot says there's no yass folder to use, and how to get one. From a linked worktree that's never
// `yass init`: it would make a stray, empty plans folder where only this worktree looks.
func (r *Repo) noRoot(msg string) string {
	if r.Linked {
		return "no yass folder found from this worktree; see the warning"
	}
	return msg
}

// disp shows a path relative to the repo when it's inside it, and in full when it isn't.
func (r *Repo) disp(p string) string {
	if within(p, r.Top) {
		rel, _ := filepath.Rel(r.Top, p)
		return rel
	}
	return p
}

// findRoots finds every folder named yass/ with changes/ or archive/ in it, and every yass.yaml.
// Inside git, .gitignore decides, except that a yass.yaml at the top or above the current folder
// always counts, so it can be kept out of the repo. Folders a yass.yaml ignores don't count, unless
// you're standing in one: then it's a project of its own, and its folder is the top.
func (r *Repo) findRoots() {
	warned := len(r.Warnings)
	dirs, configs := map[string]bool{}, map[string]bool{}
	out, ok := git(r.Top, "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--",
		":(glob)**/yass/changes/**", ":(glob)**/yass/archive/**", ":(glob)**/"+ConfigName)
	if ok {
		for _, f := range strings.Split(out, "\x00") { // -z: raw paths, even with non-ASCII names
			if f == "" {
				continue
			}
			if path.Base(f) == ConfigName {
				configs[filepath.Join(r.Top, filepath.FromSlash(f))] = true
			} else if m := yassDirRE.FindStringSubmatch(f); m != nil {
				dirs[filepath.Join(r.Top, filepath.FromSlash(m[1]), "yass")] = true
			}
		}
	} else {
		filepath.WalkDir(r.Top, func(p string, d fs.DirEntry, err error) error {
			if err != nil {
				return nil
			}
			if d.IsDir() {
				name := d.Name()
				if p != r.Top && (skipDirs[name] || strings.HasPrefix(name, ".")) {
					return filepath.SkipDir
				}
				if name == "yass" && (isDir(filepath.Join(p, "changes")) || isDir(filepath.Join(p, "archive"))) {
					dirs[p] = true
					return filepath.SkipDir
				}
			} else if d.Name() == ConfigName {
				configs[p] = true
			}
			return nil
		})
	}
	for d := r.Cwd; within(d, r.Top); d = filepath.Dir(d) {
		if f := filepath.Join(d, ConfigName); isFile(f) {
			configs[f] = true
		}
		if d == r.Top || filepath.Dir(d) == d {
			break
		}
	}

	// Settle ignores shallowest first: a yass.yaml in an ignored folder doesn't count, nor do its ignores.
	files := sortedKeys(configs)
	sort.SliceStable(files, func(i, j int) bool { return depth(files[i]) < depth(files[j]) })
	r.Ignored = nil
	for _, f := range files {
		if r.ignored(f) {
			delete(configs, f)
			continue
		}
		c, _, err := loadConfig(f)
		if err != nil {
			continue // reported below
		}
		ign, bad := ignoredDirs(f, c)
		for _, b := range bad {
			r.warn("%s: %s", r.disp(f), b)
		}
		r.Ignored = append(r.Ignored, ign...)
	}
	var inside string
	for _, d := range r.Ignored {
		if within(r.Cwd, d) && len(d) > len(inside) {
			inside = d
		}
	}
	if inside != "" {
		r.Top, r.Roots, r.Ignored, r.Warnings = inside, nil, nil, r.Warnings[:warned]
		r.findRoots()
		return
	}
	for d := range dirs {
		if r.ignored(d) {
			delete(dirs, d)
		}
	}

	included, borrowed := r.includes(configs)
	seen, claimed := map[string]bool{}, map[string]bool{}
	for _, f := range sortedKeys(configs) {
		if !included[f] {
			r.Configs = append(r.Configs, f)
		}
	}
	for _, f := range sortedKeys(configs) {
		owner := filepath.Dir(f)
		claimed[owner] = true
		dir, cfg, unknown, err := configDir(f)
		for _, k := range unknown {
			r.warn("%s: unknown setting '%s' (a newer yass may know it)", r.disp(f), k)
		}
		if err != nil {
			r.warn("%s: %v", r.disp(f), err)
			continue
		}
		if !isDir(dir) && strings.TrimSpace(cfg.Path) != "" {
			found, looked, linked := fromOtherCheckouts(f, strings.TrimSpace(cfg.Path))
			switch {
			case found != "":
				dir = found
			case linked:
				r.Linked = true
				places := r.disp(dir)
				for _, l := range looked {
					if l != dir {
						places += ", " + r.disp(l)
					}
				}
				r.warn("%s: points to a folder that isn't there from this worktree or the clone's other checkouts (looked in %s); check `path:`, or that the plans are where the main checkout expects them",
					r.disp(f), places)
				continue
			}
		}
		if !isDir(dir) {
			r.warn("%s: points to %s, which doesn't exist yet (`yass init` creates it)", r.disp(f), r.disp(dir))
			continue
		}
		dir = canon(dir)
		// On disk too, not just in git's list: info/exclude may hide a yass/ that was once a link.
		if local := filepath.Join(owner, "yass"); (dirs[local] || isRealYassDir(local)) && local != dir {
			r.warn("%s/: ignored, because %s points to %s", r.disp(local), r.disp(f), r.disp(dir))
		}
		if !seen[dir] {
			seen[dir] = true
			r.Roots = append(r.Roots, &Root{Dir: dir, Owner: owner, Config: f, Branch: strings.TrimSpace(cfg.Branch), Included: included[f]})
		}
	}
	for _, b := range borrowed {
		if !seen[b.Dir] {
			seen[b.Dir] = true
			r.Roots = append(r.Roots, b)
		}
	}
	for _, d := range sortedKeys(dirs) {
		owner := filepath.Dir(d)
		if claimed[owner] || seen[d] || !isDir(d) {
			continue
		}
		seen[d] = true
		r.Roots = append(r.Roots, &Root{Dir: d, Owner: owner})
	}
	if len(r.Roots) == 0 && len(r.Warnings) == warned {
		r.borrowConfig()
	}
	sort.SliceStable(r.Roots, func(i, j int) bool {
		a, b := r.Roots[i].Owner, r.Roots[j].Owner
		if da, db := depth(a), depth(b); da != db {
			return da < db
		}
		return a < b
	})
}

// ignored says whether p is inside a folder a yass.yaml ignores.
func (r *Repo) ignored(p string) bool {
	for _, d := range r.Ignored {
		if within(p, d) {
			return true
		}
	}
	return false
}

func depth(p string) int { return strings.Count(p, string(filepath.Separator)) }

func sortedKeys(m map[string]bool) []string {
	out := make([]string, 0, len(m))
	for k := range m {
		out = append(out, k)
	}
	sort.Strings(out)
	return out
}

func subdirs(dir string) []string {
	entries, _ := os.ReadDir(dir)
	var out []string
	for _, e := range entries {
		if e.IsDir() {
			out = append(out, e.Name())
		}
	}
	sort.Strings(out)
	return out
}

var (
	yearRE  = regexp.MustCompile(`^\d{4}$`)
	monthRE = regexp.MustCompile(`^\d{2}$`)
)

// changeDirs lists the change folders in changes/ or archive/. The archive keeps them in
// <YYYY>/<MM>/ folders by the month they were archived, and older archives keep them flat;
// both count, so a half-migrated archive still works.
func changeDirs(base string, archive bool) []string {
	var out []string
	for _, name := range subdirs(base) {
		if !archive || !yearRE.MatchString(name) {
			out = append(out, filepath.Join(base, name))
			continue
		}
		for _, month := range subdirs(filepath.Join(base, name)) {
			if !monthRE.MatchString(month) {
				continue
			}
			for _, c := range subdirs(filepath.Join(base, name, month)) {
				out = append(out, filepath.Join(base, name, month, c))
			}
		}
	}
	return out
}

// archiveDir is where a change archived at t goes: archive/<YYYY>/<MM>/<name>, by UTC month.
func archiveDir(root *Root, name string, t time.Time) string {
	t = t.UTC()
	return filepath.Join(root.Dir, "archive", t.Format("2006"), t.Format("01"), name)
}

// load reads a yass folder's active changes and their pieces, or, for the archive, only the names
// of its changes: their pieces are found when a name needs them (archivedPieces).
func (r *Repo) load(root *Root, kind string, warn bool) []*Change {
	base := filepath.Join(root.Dir, kind)
	var out []*Change
	for _, p := range changeDirs(base, kind == "archive") {
		name := filepath.Base(p)
		if kind == "archive" {
			c := nameOnly(p, root, nil)
			c.Archived = true
			out = append(out, c)
			continue
		}
		c := newChange(p, root, nil)
		out = append(out, c)
		if warn && c.Head == "" {
			r.warn("%s: no change.md", r.disp(p))
		}
		if warn && !nameRE.MatchString(name) {
			r.warn("%s: name it <YYYY-MM-DD>-<slug>", r.disp(p))
		}
		for _, sub := range subdirs(p) {
			sp := filepath.Join(p, sub)
			if !isFile(filepath.Join(sp, "change.md")) {
				continue
			}
			c.Pieces = append(c.Pieces, newChange(sp, root, c))
			if !warn {
				continue
			}
			filepath.WalkDir(sp, func(d string, e fs.DirEntry, err error) error {
				if err == nil && e.IsDir() && d != sp && isFile(filepath.Join(d, "change.md")) {
					r.warn("%s: pieces nest one level deep; move it up into %s", r.disp(d), c.Name)
				}
				return nil
			})
		}
		byCreated(c.Pieces)
	}
	if kind == "archive" {
		for _, m := range evictedMonths(root) {
			for _, name := range m.Changes {
				c := nameOnly(filepath.Join(root.Dir, "archive", filepath.FromSlash(m.Month), name), root, nil)
				c.Archived, c.Evicted = true, true
				out = append(out, c)
			}
		}
	}
	byCreated(out)
	return out
}

// archivedPieces finds the pieces of archived changes, by name, the first time a name isn't an
// archived change itself: it opens every archived change's folder, which plain status never needs.
func (r *Repo) archivedPieces() {
	if r.piecesFound {
		return
	}
	r.piecesFound = true
	for _, c := range r.Archived {
		if c.Evicted {
			continue
		}
		for _, sub := range subdirs(c.Path) {
			if sp := filepath.Join(c.Path, sub); isFile(filepath.Join(sp, "change.md")) {
				piece := nameOnly(sp, c.Root, c)
				piece.Archived = true
				c.Pieces = append(c.Pieces, piece)
			}
		}
	}
}

func (r *Repo) check() {
	known := map[string]bool{}
	addKnown := func() {
		for _, c := range append(append([]*Change{}, r.Active...), r.Archived...) {
			known[c.Name] = true
			for _, p := range c.Pieces {
				known[p.Name] = true
			}
		}
	}
	addKnown()
	isKnown := func(name string) bool {
		if !known[name] && !r.piecesFound {
			r.archivedPieces()
			addKnown()
		}
		return known[name]
	}
	for _, c := range r.everything() {
		if f := c.Meta["follows"]; f != "" && !isKnown(path.Base(strings.TrimRight(f, "/"))) {
			if where := r.notCheckedOut(path.Base(strings.TrimRight(f, "/"))); where != "" {
				r.note("%s: follows '%s', which is in %s, outside this sparse checkout, so it can't be checked", r.disp(c.Path), f, where)
			} else {
				r.warn("%s: follows '%s', which isn't in changes/ or archive/", r.disp(c.Path), f)
			}
		}
		if t := c.Meta["created"]; t != "" {
			if _, err := time.Parse(time.RFC3339, t); err != nil {
				r.warn("%s: created '%s' isn't a time like 2026-10-05T14:32:07Z", r.disp(c.Path), t)
			}
		}
	}
	r.checkDeps()
	r.checkQueues()
	r.checkCode()
}

// checkDeps reads each blocked: that names changes (comma-separated single words: a folder name, a
// path ending in one, or <change>/<piece>) into Deps, and warns about names that look like a change
// but aren't one, and about changes that end up waiting on themselves.
func (r *Repo) checkDeps() {
	all := append(flatten(r.Active), flatten(r.Archived)...)
	find := func(item string) *Change {
		if !r.piecesFound && !r.hasName(all, path.Base(item)) {
			r.archivedPieces()
			all = append(flatten(r.Active), flatten(r.Archived)...)
		}
		// By folder name, so a path keeps working after its change is archived; the path only
		// picks between changes that share a name.
		var hits, exact []*Change
		for _, c := range all {
			if c.Name == path.Base(item) {
				hits = append(hits, c)
				if strings.HasSuffix(filepath.ToSlash(c.Path), "/"+item) {
					exact = append(exact, c)
				}
			}
		}
		if len(hits) == 1 {
			return hits[0]
		}
		if len(exact) == 1 {
			return exact[0]
		}
		return nil
	}
	for _, c := range r.everything() {
		b := c.blocked()
		if b == "" {
			continue
		}
		var deps []*Change
		names := true
		for _, item := range strings.Split(b, ",") {
			item = strings.TrimRight(strings.TrimSpace(item), "/")
			if item == "" || strings.ContainsAny(item, " \t") {
				names = false
				continue
			}
			if d := find(item); d != nil {
				deps = append(deps, d)
				continue
			}
			names = false
			if nameRE.MatchString(path.Base(item)) {
				if where := r.notCheckedOut(path.Base(item)); where != "" {
					r.note("%s: blocked: '%s' is in %s, outside this sparse checkout, so it can't be checked", r.disp(c.Path), item, where)
				} else {
					r.warn("%s: blocked: '%s' looks like a change, but there's no such change (or more than one; give its path)", r.disp(c.Path), item)
				}
			}
		}
		if names && len(deps) > 0 {
			c.Deps = deps
		}
	}
	reported := map[string]bool{}
	for _, c := range r.everything() {
		if loop := depLoop(c, c, nil, map[*Change]bool{}); loop != nil {
			var names []string
			for _, x := range loop {
				names = append(names, x.label())
			}
			key := strings.Join(sortedCopy(names), " ")
			if !reported[key] {
				reported[key] = true
				r.warn("%s: blocked: waits on itself: %s", r.disp(c.Path), strings.Join(append([]string{c.label()}, names...), " → "))
			}
		}
	}
}

// depLoop returns the path of dependencies from cur back to start, if there is one.
func depLoop(start, cur *Change, trail []*Change, seen map[*Change]bool) []*Change {
	for _, d := range cur.Deps {
		next := append(append([]*Change{}, trail...), d)
		if d == start {
			return next
		}
		if !seen[d] {
			seen[d] = true
			if loop := depLoop(start, d, next, seen); loop != nil {
				return loop
			}
		}
	}
	return nil
}

func sortedCopy(s []string) []string {
	out := append([]string{}, s...)
	sort.Strings(out)
	return out
}

// checkQueues warns about queue.md entries that aren't active changes in that yass folder, and repeats.
func (r *Repo) checkQueues() {
	for _, root := range r.Roots {
		where := r.disp(filepath.Join(root.Dir, QueueName))
		active, archived, pieces := map[string]bool{}, map[string]bool{}, map[string]string{}
		for _, c := range r.Active {
			if c.Root == root {
				active[c.Name] = true
				for _, p := range c.Pieces {
					pieces[p.Name] = c.Name
				}
			}
		}
		for _, c := range r.Archived {
			if c.Root == root {
				archived[c.Name] = true
			}
		}
		seen := map[string]bool{}
		for _, name := range root.Queue {
			switch {
			case seen[name]:
				r.warn("%s: '%s' is listed more than once", where, name)
			case active[name]:
			case archived[name]:
				r.warn("%s: '%s' is archived; take it off the list", where, name)
			case pieces[name] != "":
				r.warn("%s: '%s' is a piece of %s; its plan.md orders its pieces", where, name, pieces[name])
			default:
				r.warn("%s: '%s' isn't a change in %s", where, name, r.disp(filepath.Join(root.Dir, "changes")))
			}
			seen[name] = true
		}
	}
}

// checkCode checks the code commits that boxes cite ("— code: a1b2c3d"): each must be a commit in
// this repo, and a box marked done must cite commits that are on the merged branch. Plans kept in
// another folder can't be committed with the code, so these links are how the two stay honest.
func (r *Repo) checkCode() {
	if _, ok := git(r.Top, "rev-parse", "--git-dir"); !ok {
		return
	}
	for _, c := range r.everything() {
		for _, b := range c.ownBoxes() {
			where, what := r.disp(filepath.Join(c.Path, b.file)), trunc(strings.TrimSpace(codeRefRE.Split(b.text, 2)[0]), 50)
			for _, sha := range codeRefs(b.text) {
				switch state, branch := r.citation(c.Root, sha); state {
				case citeMissing:
					r.warn("%s: '%s' cites %s, which isn't a commit here (fetch, or fix the ref)", where, what, sha)
				case citeUnmerged:
					if b.mark == markDone {
						r.warn("%s: '%s' is marked done, but %s isn't on %s yet (mark it [/] until it's merged, or cite the merged commit)",
							where, what, sha, branch)
					}
				}
			}
		}
	}
}

// archiveWarnings are a change's done boxes, its pieces' included, citing a commit that's missing
// here or not merged: yass archive says so as it archives.
func (r *Repo) archiveWarnings(c *Change) []string {
	if _, ok := git(r.Top, "rev-parse", "--git-dir"); !ok {
		return nil
	}
	var out []string
	for _, x := range flatten([]*Change{c}) {
		for _, b := range x.ownBoxes() {
			if b.mark != markDone {
				continue
			}
			what := trunc(strings.TrimSpace(codeRefRE.Split(b.text, 2)[0]), 50)
			where := filepath.ToSlash(filepath.Join(x.label(), b.file))
			for _, sha := range codeRefs(b.text) {
				switch state, branch := r.citation(c.Root, sha); state {
				case citeMissing:
					out = append(out, fmt.Sprintf("'%s' (%s) cites %s, which isn't a commit here", what, where, sha))
				case citeUnmerged:
					out = append(out, fmt.Sprintf("'%s' (%s) cites %s, which isn't on %s yet", what, where, sha, branch))
				}
			}
		}
	}
	return out
}

// What checking one citation found.
const (
	citeMerged   = iota // a commit here, on the merged branch (or no merged branch to compare with)
	citeUnmerged        // a commit here, not merged yet
	citeMissing         // not a commit here
)

// citation checks one cited commit against this repo; branch is the merged branch it compared with.
func (r *Repo) citation(root *Root, sha string) (state int, branch string) {
	if _, ok := git(r.Top, "rev-parse", "-q", "--verify", sha+"^{commit}"); !ok {
		return citeMissing, ""
	}
	if r.merged == nil {
		r.merged = map[*Root]string{}
	}
	branch, seen := r.merged[root]
	if !seen {
		branch = r.mergedBranch(root)
		r.merged[root] = branch
	}
	if branch == "" {
		return citeMerged, ""
	}
	if _, ok := git(r.Top, "merge-base", "--is-ancestor", sha, branch); !ok {
		return citeUnmerged, branch
	}
	return citeMerged, branch
}

// mergedBranch is the code branch that counts as merged: yass.yaml's branch, else origin/HEAD, main or master.
func (r *Repo) mergedBranch(root *Root) string {
	if root.Branch != "" {
		if _, ok := git(r.Top, "rev-parse", "-q", "--verify", root.Branch+"^{commit}"); !ok {
			r.warn("%s: branch '%s' isn't in this repo (fetch it, or fix `branch`)", r.disp(root.Config), root.Branch)
			return ""
		}
		return root.Branch
	}
	if out, ok := git(r.Top, "rev-parse", "--abbrev-ref", "origin/HEAD"); ok && strings.TrimSpace(out) != "origin/HEAD" {
		return strings.TrimSpace(out)
	}
	for _, b := range []string{"main", "master"} {
		if _, ok := git(r.Top, "rev-parse", "-q", "--verify", "refs/heads/"+b); ok {
			return b
		}
	}
	return ""
}

// everything is the active changes and their pieces.
func (r *Repo) everything() []*Change { return flatten(r.Active) }

// notCheckedOut is the yass folder (repo-relative) holding a change by that name that git knows
// but this checkout doesn't have on disk, as in a sparse checkout; "" if there's none. It asks git
// only the first time a name doesn't resolve.
func (r *Repo) notCheckedOut(name string) string {
	if r.absent == nil {
		r.absent = map[string]string{}
		out, _ := git(r.Top, "ls-files", "--cached", "-z", "--",
			":(glob)**/yass/changes/**/change.md", ":(glob)**/yass/archive/**/change.md")
		for _, f := range strings.Split(out, "\x00") {
			if f == "" || exists(filepath.Join(r.Top, filepath.FromSlash(f))) {
				continue
			}
			if m := yassDirRE.FindStringSubmatch(f); m != nil {
				r.absent[path.Base(path.Dir(f))] = path.Join(m[1], "yass") + "/"
			}
		}
	}
	return r.absent[name]
}

func (r *Repo) hasName(cs []*Change, name string) bool {
	for _, c := range cs {
		if c.Name == name {
			return true
		}
	}
	return false
}

func flatten(changes []*Change) []*Change {
	var out []*Change
	for _, c := range changes {
		out = append(out, c)
		out = append(out, c.Pieces...)
	}
	return out
}

// resolve finds the one change a name (or a unique part of one) means, among active changes and
// their pieces, or among archived ones when archived is set.
func (r *Repo) resolve(token string, pool []*Change, archived bool) (*Change, error) {
	kind, hint := "active", "yass status"
	if pool == nil {
		pool = r.everything()
		if archived {
			r.archivedPieces()
			pool, kind, hint = flatten(r.Archived), "archived", "yass status --archived"
		}
	}
	t := strings.TrimRight(token, "/")
	ap := canon(t)
	var exact []*Change
	for _, c := range pool {
		if c.Path == ap || c.Name == filepath.Base(t) || c.label() == t {
			exact = append(exact, c)
		}
	}
	if len(exact) == 1 {
		return exact[0], nil
	}
	hits := exact
	if len(hits) == 0 {
		for _, c := range pool {
			if strings.Contains(strings.ToLower(c.Name), strings.ToLower(t)) {
				hits = append(hits, c)
			}
		}
	}
	switch len(hits) {
	case 1:
		return hits[0], nil
	case 0:
		return nil, fmt.Errorf("no %s change matches '%s' (see `%s`)", kind, token, hint)
	}
	var names []string
	for i, c := range hits {
		if i == 6 {
			break
		}
		names = append(names, c.label())
	}
	return nil, fmt.Errorf("'%s' matches %d changes: %s", token, len(hits), strings.Join(names, ", "))
}

// rootForCwd is the yass folder serving the current folder: the one whose owner is nearest.
func (r *Repo) rootForCwd() *Root {
	var best *Root
	for _, root := range r.Roots {
		if within(r.Cwd, root.Owner) && (best == nil || len(root.Owner) > len(best.Owner)) {
			best = root
		}
	}
	return best
}
