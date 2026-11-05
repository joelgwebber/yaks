package yass

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
	"time"
)

func cmdInit(a *args) error {
	if semverOf(binVersion) == "" {
		return noVersion()
	}
	cwd := getwd()
	top := repoTop(cwd)
	base := top
	if len(a.pos) > 0 {
		base = canon(a.pos[0])
	}
	cfg := filepath.Join(base, ConfigName)
	var made []string
	if a.b["private"] {
		if err := makePrivate(top, base, a.v["path"]); err != nil {
			return err
		}
	}
	if p := a.v["path"]; p != "" {
		if isFile(cfg) {
			c, _, err := loadConfig(cfg)
			if err != nil {
				return fmt.Errorf("%s: %v", cfg, err)
			}
			if strings.TrimSpace(c.Path) != p {
				return fmt.Errorf("%s already points to '%s'; edit it by hand to change it", cfg, c.Path)
			}
		} else {
			if err := write(cfg, configText(p)); err != nil {
				return err
			}
			made = append(made, cfg)
		}
	}
	y, configured := filepath.Join(base, "yass"), false
	if isFile(cfg) {
		dir, c, _, err := configDir(cfg)
		if err != nil {
			return fmt.Errorf("%s: %v", cfg, err)
		}
		y, configured = dir, strings.TrimSpace(c.Path) != ""
		if configured && !isDir(y) {
			found, _, linked := fromOtherCheckouts(cfg, strings.TrimSpace(c.Path))
			if found != "" {
				y = found
			} else if linked {
				return fmt.Errorf("this is a linked worktree, and %s points to %s, which no checkout of this clone has;"+
					" run `yass init` from the main checkout, so the plans land where every checkout looks", cfg, y)
			}
		}
	}
	for _, f := range []struct{ path, text string }{
		{filepath.Join(y, "changes", ".gitkeep"), ""},
		{filepath.Join(y, "archive", ".gitkeep"), ""},
		{filepath.Join(y, "README.md"), render("readme.md", nil)},
	} {
		if !exists(f.path) {
			if err := write(f.path, f.text); err != nil {
				return err
			}
			made = append(made, f.path)
		}
	}
	// Where the agent files go: the repo root when init sets up the repo itself, or the folder with
	// --agents (a monorepo part with its own AGENTS.md and skills). A folder without --agents gets
	// only its planning. The hook belongs to the repo root alone.
	agentDir := ""
	if len(a.pos) == 0 {
		agentDir = top
	} else if a.b["agents"] {
		agentDir = base
	}
	if a.b["no-agents"] {
		agentDir = ""
	}
	kw, freshAgents := &kitWriter{}, false
	if agentDir != "" {
		ap := filepath.Join(agentDir, "AGENTS.md")
		cur := read(ap)
		if strings.Contains(cur, "<!-- yass:begin") {
			kw.kept = append(kw.kept, ap) // init keeps the section; yass upgrade updates it
		} else {
			next := ""
			if strings.TrimSpace(cur) != "" {
				next = strings.TrimRight(cur, " \t\r\n") + "\n\n"
			}
			next += stampAgents(strings.TrimSpace(render("agents.md", nil)), stampVersion()) + "\n"
			if err := write(ap, next); err != nil {
				return err
			}
			made, freshAgents = append(made, ap), true
		}
		if err := kw.agentFiles(agentDir, a.b["claude"], a.b["global"]); err != nil {
			return err
		}
	}
	if len(a.pos) == 0 && !a.b["no-agents"] {
		if err := kw.hook(top); err != nil {
			return err
		}
	}
	made = append(made, kw.made...)
	r := &Repo{Top: top}
	for _, p := range made {
		fmt.Printf("wrote %s%s\n", r.disp(canon(p)), outsideNote(!within(canon(p), canon(top))))
	}
	if a.b["hooks"] {
		if err := turnOnHook(top); err != nil {
			return err
		}
	}
	if configured && !within(canon(y), top) {
		lr := findRepo()
		lr.link()
		if ln := filepath.Join(base, "yass"); isSymlinkTo(ln, canon(y)) {
			fmt.Printf("linked %s/ to %s (ignored through .git/info/exclude)\n", r.disp(ln), canon(y))
		}
		for _, n := range lr.Notes {
			fmt.Printf("note: %s\n", n)
		}
		fmt.Printf("note: %s is outside this repo, so give your agent access to it too"+
			" (Claude Code: claude --add-dir %s, or permissions.additionalDirectories in .claude/settings.local.json)\n", canon(y), canon(y))
	}
	if !configured {
		if _, ignored := git(top, "check-ignore", "-q", filepath.Join(y, "changes", ".gitkeep")); ignored {
			fmt.Printf("warning: %s is ignored by .gitignore, so yass won't see it; pick another folder or unignore it"+
				" (or keep it out of the repo on purpose with %s)\n", r.disp(y), ConfigName)
		}
	}
	if len(made) == 0 {
		fmt.Println("already set up")
	}
	if freshAgents && len(a.pos) == 0 {
		fmt.Print(`
YASS is set up. Next:
  1. Commit it:  git add -A && git commit -m "chore: adopt YASS"
  2. Ask your agent to start something:
       small:  "use yass-work to fix <bug>"
       large:  "use yass-shape to plan <feature>"
  3. ` + "`yass status`" + ` shows what's in flight.
`)
	}
	for _, p := range kw.kept {
		if olderThanBinary(readStamp(p)) {
			fmt.Printf("note: YASS's files here are older than this yass (%s); `yass upgrade` updates them\n", binVersion)
			break
		}
	}
	return nil
}

// makePrivate keeps a team folder out of git for this clone only: its own line in the clone's
// .git/info/exclude, and a yass.include entry in the clone's git config so YASS still finds it,
// from every worktree. Nothing is written to the repo.
func makePrivate(top, base, path string) error {
	rel, err := filepath.Rel(top, base)
	if err != nil || rel == "." || !within(base, top) {
		return fmt.Errorf("--private needs a folder inside the repo: yass init <folder> --path <plans> --private")
	}
	if path == "" && !isFile(filepath.Join(base, ConfigName)) {
		return fmt.Errorf("--private needs --path: the plans live outside the repo, and %s points to them", ConfigName)
	}
	if _, ok := git(top, "rev-parse", "--git-dir"); !ok {
		return fmt.Errorf("--private needs a git repo")
	}
	rel = filepath.ToSlash(rel)
	r := &Repo{Top: top}
	r.exclude(base)
	if out, _ := git(top, "config", "--get-all", "yass.include"); !contains(lines(strings.TrimSpace(out)), rel) {
		if _, ok := git(top, "config", "--add", "yass.include", rel); !ok {
			return fmt.Errorf("couldn't add %s to git config yass.include", rel)
		}
	}
	for _, n := range r.Notes {
		fmt.Printf("note: %s\n", n)
	}
	fmt.Printf("private: %s/ is kept out of git (.git/info/exclude) and found through git config yass.include, for this clone only\n", rel)
	return nil
}

func cmdNew(a *args) error {
	r := loadRepo()
	title := a.pos[0]
	var root *Root
	var base string
	if into := a.v["in"]; into != "" {
		if a.b["large"] {
			return fmt.Errorf("a piece can't be large; pieces nest one level deep")
		}
		parent, err := r.resolve(into, r.Active, false)
		if err != nil {
			return err
		}
		root, base = parent.Root, parent.Path
	} else {
		if root = r.rootForCwd(); root == nil {
			msg := r.noRoot("no yass/ folder here; run `yass init` first")
			for _, w := range r.Warnings {
				msg += "\n  " + w
			}
			return fmt.Errorf("%s", msg)
		}
		base = filepath.Join(root.Dir, "changes")
	}
	stem := time.Now().Format("2006-01-02") + "-" + slug(title, 40)
	name := stem
	for n := 2; exists(filepath.Join(base, name)); n++ {
		name = fmt.Sprintf("%s-%d", stem, n)
	}
	p := filepath.Join(base, name)
	platforms := a.v["platforms"]
	if platforms == "" {
		platforms = "all"
	}
	kw := map[string]string{"title": title, "platforms": platforms, "source": a.v["source"],
		"follows": a.v["follows"], "goal": a.v["goal"], "created": time.Now().UTC().Format(time.RFC3339)}
	head := "change.md"
	if a.b["large"] {
		head = "change-large.md"
	}
	files := map[string]string{"change.md": render(head, kw)}
	if a.b["large"] {
		files["prd.md"] = render("prd.md", map[string]string{"title": title})
		files["plan.md"] = render("plan.md", map[string]string{"title": title})
	}
	if a.b["design"] {
		files["design.md"] = render("design.md", map[string]string{"title": title})
	}
	for f, text := range files {
		if err := write(filepath.Join(p, f), text); err != nil {
			return err
		}
	}
	fmt.Println(r.disp(p))
	return nil
}

func statusLine(c *Change) string {
	done, total := c.progress()
	first := "-"
	if total > 0 {
		first = fmt.Sprintf("%d/%d", done, total)
	}
	var bits []string
	if s := c.state(); s != "in progress" {
		bits = append(bits, s)
	}
	if reason, waiting := c.blockedBy(); reason != "" {
		bits = append(bits, "BLOCKED: "+reason)
	} else if len(waiting) > 0 {
		bits = append(bits, "waiting on: "+labels(waiting))
	} else if n := c.nextStep(); n != "" {
		bits = append(bits, "next: "+trunc(n, 70))
	}
	return fmt.Sprintf("%5s  ", first) + strings.Join(bits, "  ")
}

// brief is a change's progress in one line, for commit messages and pull requests: a piece also
// gets its parent's, so a reader sees where the whole change stands.
func brief(c *Change) string {
	short := func(c *Change) string {
		if nameRE.MatchString(c.Name) {
			return c.Name[11:] // without the date
		}
		return c.Name
	}
	one := func(c *Change) string {
		done, total := c.progress()
		var bits []string
		if len(c.Pieces) > 0 { // a piece has none, so its part is just boxes
			n := 0
			for _, p := range c.Pieces {
				if p.state() == "done" {
					n++
				}
			}
			bits = append(bits, fmt.Sprintf("%d/%d pieces done", n, len(c.Pieces)))
		}
		bits = append(bits, fmt.Sprintf("%d/%d boxes", done, total))
		if c.state() == "done" {
			bits = append(bits, "done")
		}
		return short(c) + ": " + strings.Join(bits, ", ")
	}
	if c.Parent != nil {
		return one(c.Parent) + " · " + one(c)
	}
	return one(c)
}

func cmdStatus(a *args) (int, error) {
	r := loadRepo()
	if a.b["archived"] {
		r.readArchived()
	}
	r.pastKeep()
	if a.b["brief"] && len(a.pos) == 0 {
		return 0, fmt.Errorf("--brief needs a change: yass status <change> --brief")
	}
	if len(a.pos) > 0 {
		c, err := r.resolve(a.pos[0], nil, a.b["archived"])
		if err != nil {
			return 0, err
		}
		if a.b["brief"] {
			fmt.Println(brief(c))
			return 0, nil
		}
		showChange(r, c)
		return 0, nil
	}
	archived := a.b["archived"]
	since, err := parseSince(a.v["since"], time.Now())
	if err != nil {
		return 0, err
	}
	var view *branchView
	if !archived && !a.b["no-branches"] {
		view = r.branches(since)
	}
	multi := len(r.Roots) > 1
	pad := ""
	if multi {
		pad = "  "
	}
	for _, root := range r.Roots {
		pool := r.Active
		if archived {
			pool = r.Archived
		}
		var items []*Change
		var evicted []*Change
		for _, c := range pool {
			switch {
			case c.Root != root:
			case c.Evicted:
				evicted = append(evicted, c)
			default:
				items = append(items, c)
			}
		}
		if !archived {
			sort.SliceStable(items, func(i, j int) bool { return root.rank(items[i].Name) < root.rank(items[j].Name) })
		}
		if multi {
			fmt.Printf("%s/\n", r.disp(root.Dir))
		}
		if len(items) == 0 {
			if archived {
				fmt.Println(pad + "nothing archived yet")
			} else {
				fmt.Println(pad + "no active changes")
			}
		}
		var rows [][2]string
		for _, c := range items {
			if archived {
				t := ""
				if !strings.Contains(c.Name, slug(c.Title, 40)) {
					t = c.Title
				}
				rows = append(rows, [2]string{c.Name, t})
				continue
			}
			name := c.Name
			if c.Large {
				name += "  (large)"
			}
			rows = append(rows, [2]string{name, statusLine(c)})
			rows = append(rows, view.linesFor(r, c, pad+"      ")...)
			for i, p := range c.Pieces {
				branch := "├"
				if i == len(c.Pieces)-1 {
					branch = "└"
				}
				rows = append(rows, [2]string{"  " + branch + " " + p.Name, statusLine(p)})
				rows = append(rows, view.linesFor(r, p, pad+"        ")...)
			}
		}
		w := 0
		for _, row := range rows {
			if n := len([]rune(row[0])); n > w && row[0] != branchRowMark {
				w = n
			}
		}
		for _, row := range rows {
			if row[0] == branchRowMark {
				fmt.Println(row[1])
				continue
			}
			fmt.Println(strings.TrimRight(pad+padRight(row[0], w)+"  "+row[1], " "))
		}
		if len(evicted) > 0 {
			seen := map[string]bool{}
			for _, c := range evicted {
				m := filepath.ToSlash(filepath.Dir(c.Path))
				seen[m[len(m)-7:]] = true // <YYYY>/<MM>
			}
			months := sortedKeys(seen)
			fmt.Printf("%s… and %d evicted, from %s (in git history; see their month's .evicted file)\n", pad, len(evicted), strings.Join(months, ", "))
		}
	}
	if len(r.Roots) == 0 {
		fmt.Println(r.noRoot("no yass/ folder yet; run `yass init`"))
	}
	if view != nil && !archived {
		fmt.Printf("branches: unmerged work since %s; %s\n", since.Format("2006-01-02"), view.fresh)
	}
	if n := versionNote(r); n != "" {
		r.note("%s", n)
	}
	plans := r.plansNotes()
	for _, w := range r.Warnings {
		fmt.Printf("warning: %s\n", w)
	}
	for _, n := range r.Notes {
		fmt.Printf("note: %s\n", n)
	}
	for _, l := range plans {
		fmt.Println(l)
	}
	if a.b["strict"] && len(r.Warnings) > 0 {
		return 1, nil
	}
	return 0, nil
}

func showChange(r *Repo, c *Change) {
	fmt.Println(c.Title)
	where := r.disp(c.Path)
	if c.Parent != nil {
		where += "  (piece of " + c.Parent.Name + ")"
	}
	fmt.Println(where)
	for _, k := range []string{"platforms", "source", "follows", "blocked"} {
		if v := c.Meta[k]; v != "" {
			fmt.Printf("%s: %s\n", k, v)
		}
	}
	if c.Deps != nil {
		if _, waiting := c.blockedBy(); len(waiting) > 0 {
			fmt.Printf("waiting on: %s\n", labels(waiting))
		} else {
			fmt.Printf("note: everything blocked: names is done or archived; clear `blocked:`\n")
		}
	}
	done, total := c.progress()
	fmt.Printf("files: %s   progress: %d/%d  (%s)\n", strings.Join(mdFiles(c.Path), ", "), done, total, c.state())
	for _, p := range c.Pieces {
		d, t := p.progress()
		line := fmt.Sprintf("piece: %s  %d/%d  (%s)", p.Name, d, t, p.state())
		if reason, waiting := p.blockedBy(); reason != "" {
			line += "  BLOCKED: " + reason
		} else if len(waiting) > 0 {
			line += "  waiting on: " + labels(waiting)
		}
		fmt.Println(line)
	}
	if opened := c.allOpen(); len(opened) > 0 {
		fmt.Println("open:")
		for i, b := range opened {
			if i == 25 {
				break
			}
			fmt.Printf("  [%s] %s  (%s)\n", b.mark, trunc(b.text, 90), b.file)
		}
	}
	if n := c.nextStep(); n != "" {
		fmt.Printf("next: %s\n", n)
	}
	if commits, byAC, branch, warns := r.changeCode(c); len(commits) > 0 || len(warns) > 0 {
		if len(commits) > 0 {
			fmt.Printf("code on %s (from yass: trailers):\n", branch)
			for i, tc := range commits {
				if i == 10 {
					fmt.Printf("  … and %d more (git log %s -i --grep='^yass: %s')\n", len(commits)-10, branch, strings.ToLower(c.label()))
					break
				}
				fmt.Printf("  %s %s\n", tc.sha, trunc(tc.subject, 80))
			}
		}
		if len(byAC) > 0 {
			ids := make([]string, 0, len(byAC))
			for id := range byAC {
				ids = append(ids, id)
			}
			sort.Slice(ids, func(i, j int) bool {
				a, _ := strconv.Atoi(ids[i][2:])
				b, _ := strconv.Atoi(ids[j][2:])
				return a < b
			})
			fmt.Println("criteria:")
			for _, id := range ids {
				var shas []string
				for i, tc := range byAC[id] {
					if i == 10 {
						shas = append(shas, fmt.Sprintf("… %d more", len(byAC[id])-10))
						break
					}
					shas = append(shas, tc.sha)
				}
				fmt.Printf("  %s: %s\n", id, strings.Join(shas, ", "))
			}
		}
		for _, w := range warns {
			fmt.Printf("warning: %s\n", w)
		}
	}
	for _, w := range r.Warnings {
		if strings.HasPrefix(w, r.disp(c.Path)+string(filepath.Separator)) || strings.HasPrefix(w, r.disp(c.Path)+":") {
			fmt.Printf("warning: %s\n", w)
		}
	}
}

func cmdArchive(a *args) error {
	r := loadRepo()
	c, err := r.resolve(a.pos[0], nil, false)
	if err != nil {
		return err
	}
	if c.Parent != nil {
		return fmt.Errorf("%s is a piece of %s; pieces are archived with their change", c.Name, c.Parent.Name)
	}
	var problems []string
	if reason, waiting := c.blockedBy(); reason != "" {
		problems = append(problems, fmt.Sprintf("it's blocked: %s (clear `blocked:` when it isn't)", reason))
	} else if len(waiting) > 0 {
		problems = append(problems, fmt.Sprintf("it's waiting on %s, which isn't done yet", labels(waiting)))
	}
	if opened := c.allOpen(); len(opened) > 0 {
		msg := fmt.Sprintf("%d box(es) still open; finish them, or drop them with [-] and say why in Decisions:", len(opened))
		for i, b := range opened {
			if i == 10 {
				msg += "\n    …"
				break
			}
			msg += fmt.Sprintf("\n    [%s] %s  (%s)", b.mark, trunc(b.text, 80), b.file)
		}
		problems = append(problems, msg)
	} else if _, done, dropped := c.tally(); done+dropped == 0 {
		problems = append(problems, "it has no checkboxes at all, so nothing says it's done")
	}
	citeWarnings := r.archiveWarnings(c)
	if len(problems) > 0 && !a.b["force"] {
		return fmt.Errorf("%s isn't finished: %s\n  (--force archives it anyway)", c.Name, strings.Join(problems, "\n  "))
	}
	for _, o := range r.Archived {
		if o.Root == c.Root && o.Name == c.Name {
			return fmt.Errorf("%s is already archived, at %s; rename one of them", c.Name, r.disp(o.Path))
		}
	}
	delivered := c.delivered() // before the move, while the pieces are where they were read
	now := time.Now().UTC()
	dest := archiveDir(c.Root, c.Name, now)
	if exists(dest) {
		return fmt.Errorf("%s already exists", r.disp(dest))
	}
	if err := os.MkdirAll(filepath.Dir(dest), 0o755); err != nil {
		return err
	}
	if _, ok := git(c.Root.Dir, "mv", c.Path, dest); !ok {
		if err := os.Rename(c.Path, dest); err != nil {
			return err
		}
		git(c.Root.Dir, "add", "-A", "--", c.Path)
	}
	// The stamp is part of the move: an archived change is never edited after it lands.
	if head := filepath.Join(dest, "change.md"); isFile(head) {
		if err := write(head, setMeta(read(head), "archived", now.Format(time.RFC3339))); err != nil {
			return err
		}
	}
	// So is ticking the criteria the pieces delivered: the archive is the one closing commit.
	if plan := filepath.Join(dest, "plan.md"); isFile(plan) && len(delivered) > 0 {
		if err := write(plan, markDelivered(read(plan), delivered)); err != nil {
			return err
		}
	}
	git(c.Root.Dir, "add", "-A", "--", dest)
	fmt.Printf("archived %s\n", r.disp(dest))
	for _, w := range citeWarnings {
		fmt.Printf("warning: %s\n", w)
	}
	if n, k := r.kept(c.Root)+1, c.Root.keep(); n > k {
		fmt.Printf("note: %s holds %d archived changes, %d past its keep of %d; `yass evict` moves its oldest months to git history\n",
			r.disp(filepath.Join(c.Root.Dir, "archive")), n, n-k, k)
	}
	if q := filepath.Join(c.Root.Dir, QueueName); isFile(q) {
		cur := read(q)
		var keep []string
		for _, l := range strings.SplitAfter(cur, "\n") {
			if name, ok := queueEntry(strings.TrimRight(l, "\r\n")); !ok || name != c.Name {
				keep = append(keep, l)
			}
		}
		if next := strings.Join(keep, ""); next != cur {
			if err := write(q, next); err != nil {
				return err
			}
			git(c.Root.Dir, "add", "--", q)
			fmt.Printf("took it off %s\n", r.disp(q))
		}
	}
	commit := fmt.Sprintf("git commit -m \"yass: archive %s\"", c.Name)
	if out, ok := git(c.Root.Dir, "rev-parse", "--show-toplevel"); ok {
		if gitTop := canon(filepath.FromSlash(strings.TrimSpace(out))); gitTop != r.Top {
			commit = fmt.Sprintf("git -C %s commit -m \"yass: archive %s\"", gitTop, c.Name)
		}
		fmt.Printf("Commit it on its own: %s\n", commit)
	}
	return nil
}

func cmdRoot(a *args) error {
	r := loadRepo()
	if len(r.Roots) == 0 {
		msg := r.noRoot("no yass folder yet; run `yass init`")
		for _, w := range r.Warnings {
			msg += "\n  " + w
		}
		return fmt.Errorf("%s", msg)
	}
	for _, root := range r.Roots {
		if a.b["v"] {
			fmt.Printf("%s\t%s\n", root.Dir, r.kind(root))
		} else {
			fmt.Println(root.Dir)
		}
	}
	for _, n := range r.Notes { // stderr, so scripts reading the folders aren't disturbed
		fmt.Fprintf(os.Stderr, "note: %s\n", n)
	}
	return nil
}

func cmdTemplate(a *args) error {
	name := strings.TrimSuffix(a.pos[0], ".md") + ".md"
	if _, err := templates.ReadFile("templates/" + name); err != nil {
		return fmt.Errorf("no template '%s'; try change, change-large, prd, plan, design, readme or agents", a.pos[0])
	}
	fmt.Print(render(name, nil))
	return nil
}
