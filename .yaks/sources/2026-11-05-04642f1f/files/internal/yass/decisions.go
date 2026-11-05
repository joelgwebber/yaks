package yass

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"strconv"
	"strings"
	"time"
)

// yass decisions: the ## Decisions entries of every change, active and archived, in every yass
// folder, so an agent asking "why did we…" gets a bounded answer instead of grepping the archive.

var (
	entryRE  = regexp.MustCompile(`^[-*+]\s+(.*\S)\s*$`)
	creditRE = regexp.MustCompile(`\s*\(([^()]*)\)\s*$`)
	dayRE    = regexp.MustCompile(`^\d{4}-\d{2}-\d{2}$`)
)

type decision struct {
	Change   string  `json:"change"`
	Title    string  `json:"title"`
	Who      string  `json:"who"`
	Decision string  `json:"decision"`
	Date     *string `json:"date"`
	Created  *string `json:"created"`
	Archived *string `json:"archived"`
	when     string  // what filters and sorting go by: its own date, else archived, else created
	order    int
}

// parseEntry splits a Decisions line into the decision and its credit: the last parenthesis, whose
// text after the last comma is the date when it's a YYYY-MM-DD (design §6).
func parseEntry(line string) (text, who, date string) {
	text = line
	m := creditRE.FindStringSubmatchIndex(line)
	if m == nil {
		return text, "", ""
	}
	text, who = strings.TrimSpace(line[:m[0]]), strings.TrimSpace(line[m[2]:m[3]])
	if i := strings.LastIndex(who, ","); i >= 0 && dayRE.MatchString(strings.TrimSpace(who[i+1:])) {
		who, date = strings.TrimSpace(who[:i]), strings.TrimSpace(who[i+1:])
	}
	return text, who, date
}

func day(stamp string) string {
	if t, err := time.Parse(time.RFC3339, stamp); err == nil {
		return t.UTC().Format("2006-01-02")
	}
	if len(stamp) >= 10 && dayRE.MatchString(stamp[:10]) {
		return stamp[:10]
	}
	return ""
}

func ptr(s string) *string {
	if s == "" {
		return nil
	}
	return &s
}

// decisionsOf lists a change's own entries (its pieces have their own).
func decisionsOf(c *Change) []decision {
	top := c
	if c.Parent != nil {
		top = c.Parent
	}
	created, archived := day(c.created()), day(top.Meta["archived"])
	var out []decision
	for _, l := range lines(commentRE.ReplaceAllString(section(c.Head, "Decisions"), "")) {
		m := entryRE.FindStringSubmatch(l)
		if m == nil {
			continue
		}
		text, who, date := parseEntry(m[1])
		d := decision{Change: c.label(), Title: c.Title, Who: who, Decision: text,
			Date: ptr(date), Created: ptr(created), Archived: ptr(archived), order: len(out)}
		for _, w := range []string{date, archived, created} {
			if w != "" {
				d.when = w
				break
			}
		}
		out = append(out, d)
	}
	return out
}

// related is the change named and every change linked to it through follows:, both ways, with
// their pieces.
func related(all []*Change, start *Change) map[*Change]bool {
	top := func(c *Change) *Change {
		if c.Parent != nil {
			return c.Parent
		}
		return c
	}
	byName := map[string]*Change{}
	for _, c := range all {
		byName[c.Name] = c
	}
	links := map[*Change][]*Change{}
	for _, c := range all {
		if f := strings.TrimRight(c.Meta["follows"], "/"); f != "" {
			if o := byName[f[strings.LastIndex(f, "/")+1:]]; o != nil {
				a, b := top(c), top(o)
				links[a], links[b] = append(links[a], b), append(links[b], a)
			}
		}
	}
	seen := map[*Change]bool{}
	var walk func(*Change)
	walk = func(c *Change) {
		if seen[c] {
			return
		}
		seen[c] = true
		for _, p := range c.Pieces {
			seen[p] = true
		}
		for _, o := range links[c] {
			walk(o)
		}
	}
	if start.Parent != nil {
		seen[start] = true // a piece alone: itself, and what its change is linked to
		for _, o := range links[start.Parent] {
			walk(o)
		}
		return seen
	}
	walk(start)
	return seen
}

func cmdDecisions(a *args) (int, error) {
	r := loadRepo()
	if len(r.Roots) == 0 {
		fmt.Println(r.noRoot("no yass folder here; `yass init` sets one up"))
		return 0, nil
	}
	r.readArchived()
	all := append(r.everything(), flatten(r.Archived)...)
	for _, k := range []string{"since", "until"} {
		if v := a.v[k]; v != "" && !dayRE.MatchString(v) {
			return 0, fmt.Errorf("--%s takes a date like 2026-10-07", k)
		}
	}
	limit := 50
	if v := a.v["limit"]; v != "" {
		n, err := strconv.Atoi(v)
		if err != nil || n < 1 {
			return 0, fmt.Errorf("--limit takes a number above 0")
		}
		limit = n
	}

	// Evicted months are read from git only when the query reaches them.
	stubs := r.evictedStubs()
	done := map[*Root]map[string]bool{}
	var missing []string
	readMonths := func(pick func(root *Root, m string) bool) {
		want := map[*Root][]string{}
		for root, months := range stubs {
			for m := range months {
				if !done[root][m] && pick(root, m) {
					if done[root] == nil {
						done[root] = map[string]bool{}
					}
					done[root][m] = true
					want[root] = append(want[root], m)
				}
			}
		}
		for root := range want {
			sort.Strings(want[root]) // oldest month first, whatever order the map gave
		}
		if len(want) > 0 {
			missing = append(missing, r.readEvicted(want)...)
			all = append(r.everything(), flatten(r.Archived)...)
		}
	}
	monthOfStub := func(c *Change) string {
		if c.Parent != nil {
			c = c.Parent
		}
		m := filepath.ToSlash(filepath.Dir(c.Path))
		return m[len(m)-7:]
	}

	if sha := strings.ToLower(a.v["cites"]); sha != "" {
		if !shaRE.MatchString(sha) {
			return 0, fmt.Errorf("--cites takes a commit hash (7 to 40 hex digits)")
		}
		readMonths(func(*Root, string) bool { return true })
		return citations(a, all, sha, limit, missing)
	}

	var only map[*Change]bool
	if name := a.v["change"]; name != "" {
		c, err := r.resolve(name, all, false)
		if err != nil {
			return 0, err
		}
		// Follow the chain into evicted months, reading each one the chain reaches.
		for {
			only = related(all, c)
			reach := map[*Root]map[string]bool{}
			for x := range only {
				if x.Evicted {
					if reach[x.Root] == nil {
						reach[x.Root] = map[string]bool{}
					}
					reach[x.Root][monthOfStub(x)] = true
				}
			}
			before := len(missing) + countDone(done)
			readMonths(func(root *Root, m string) bool { return reach[root][m] })
			if len(missing)+countDone(done) == before {
				break
			}
			for _, x := range all { // the change named may have been a stub, now read
				if x.Name == c.Name && x.Root == c.Root {
					c = x
				}
			}
		}
	} else if since := a.v["since"]; since != "" {
		readMonths(func(_ *Root, m string) bool { return monthEnd(m) >= since })
	}

	words := strings.Fields(strings.ToLower(a.v["about"]))
	collect := func() []decision {
		var found []decision
		for _, c := range all {
			if only != nil && !only[c] {
				continue
			}
		entries:
			for _, d := range decisionsOf(c) {
				if (a.v["since"] != "" && d.when < a.v["since"]) || (a.v["until"] != "" && (d.when == "" || d.when > a.v["until"])) {
					continue
				}
				hay := strings.ToLower(d.Decision + " " + d.Title)
				for _, w := range words {
					if !strings.Contains(hay, w) {
						continue entries
					}
				}
				found = append(found, d)
			}
		}
		return found
	}
	found := collect()
	// Without dates or a change to go by, evicted months are older than everything in the tree:
	// read them only when what's here doesn't fill the answer.
	if only == nil && a.v["since"] == "" && len(found) < limit && len(stubs) > 0 {
		readMonths(func(*Root, string) bool { return true })
		found = collect()
	}
	sort.SliceStable(found, func(i, j int) bool {
		if found[i].when != found[j].when {
			return found[i].when > found[j].when
		}
		if found[i].Change != found[j].Change {
			return found[i].Change < found[j].Change
		}
		return found[i].order < found[j].order
	})
	left := 0
	if len(found) > limit {
		left, found = len(found)-limit, found[:limit]
	}
	if len(missing) > 0 && a.b["json"] {
		fmt.Fprintf(os.Stderr, "yass: %s\n", missingNote(missing))
	}
	for _, w := range r.Warnings { // only what bears on the answer; yass status shows the rest
		if strings.Contains(w, "can't read it") {
			fmt.Fprintf(os.Stderr, "warning: %s\n", w)
		}
	}
	if a.b["json"] {
		if found == nil {
			found = []decision{}
		}
		out, _ := json.MarshalIndent(found, "", "  ")
		fmt.Println(string(out))
		if left > 0 {
			fmt.Fprintf(os.Stderr, "yass: %d more not shown; narrow the query or raise --limit\n", left)
		}
		return 0, nil
	}
	if len(found) == 0 {
		fmt.Println("no decisions match")
		return 0, nil
	}
	for _, d := range found {
		date := "          "
		if d.Date != nil {
			date = *d.Date
		}
		line := fmt.Sprintf("%s  %s  %s", date, d.Change, d.Decision)
		if d.Who != "" {
			line += " (" + d.Who + ")"
		}
		var also []string
		if d.Created != nil {
			also = append(also, "created "+*d.Created)
		}
		if d.Archived != nil {
			also = append(also, "archived "+*d.Archived)
		}
		if len(also) > 0 {
			line += "  [" + strings.Join(also, ", ") + "]"
		}
		fmt.Println(line)
	}
	if left > 0 {
		fmt.Printf("… %d more not shown; narrow the query or raise --limit\n", left)
	}
	if len(missing) > 0 {
		fmt.Printf("note: %s\n", missingNote(missing))
	}
	return 0, nil
}

func countDone(done map[*Root]map[string]bool) int {
	n := 0
	for _, ms := range done {
		n += len(ms)
	}
	return n
}

type citation struct {
	Change string `json:"change"`
	Title  string `json:"title"`
	File   string `json:"file"`
	Mark   string `json:"mark"`
	Box    string `json:"box"`
}

// citations is yass decisions --cites: the boxes that cite a commit, by any prefix of its hash.
func citations(a *args, all []*Change, sha string, limit int, missing []string) (int, error) {
	var found []citation
	for _, c := range all {
		for _, b := range c.ownBoxes() {
			for _, ref := range codeRefs(b.text) {
				if strings.HasPrefix(ref, sha) || strings.HasPrefix(sha, ref) {
					found = append(found, citation{c.label(), c.Title, b.file, b.mark, b.text})
					break
				}
			}
		}
	}
	left := 0
	if len(found) > limit {
		left, found = len(found)-limit, found[:limit]
	}
	if a.b["json"] {
		if found == nil {
			found = []citation{}
		}
		out, _ := json.MarshalIndent(found, "", "  ")
		fmt.Println(string(out))
		if len(missing) > 0 {
			fmt.Fprintf(os.Stderr, "yass: %s\n", missingNote(missing))
		}
		return 0, nil
	}
	if len(found) == 0 {
		fmt.Printf("no box cites %s\n", sha)
	}
	for _, f := range found {
		fmt.Printf("%s  %s  [%s] %s\n", f.Change, f.File, f.Mark, f.Box)
	}
	if left > 0 {
		fmt.Printf("… %d more not shown; raise --limit\n", left)
	}
	if len(missing) > 0 {
		fmt.Printf("note: %s\n", missingNote(missing))
	}
	return 0, nil
}
