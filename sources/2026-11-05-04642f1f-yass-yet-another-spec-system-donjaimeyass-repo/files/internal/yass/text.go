package yass

import (
	"embed"
	"os"
	"path/filepath"
	"regexp"
	"strings"
	"unicode/utf8"
)

//go:embed templates/*.md
var templates embed.FS

var (
	nameRE      = regexp.MustCompile(`^\d{4}-\d{2}-\d{2}-[a-z0-9][a-z0-9-]*$`)
	boxRE       = regexp.MustCompile(`^\s*[-*+]\s+\[([ xX/-])\]\s+(.*\S)\s*$`)
	commentRE   = regexp.MustCompile(`(?s)<!--.*?-->`)
	fmRE        = regexp.MustCompile(`(?s)^---\n(.*?)\n---\n?`)
	fmCommentRE = regexp.MustCompile(`\s+#(?:\s.*)?$`) // "# a comment", but not "#41" in "gh issue #41"
	titleRE     = regexp.MustCompile(`(?m)^# (.+)$`)
	h2RE        = regexp.MustCompile(`(?m)^## `)
	logEntryRE  = regexp.MustCompile(`(?m)^###\s`)
	nextRE      = regexp.MustCompile(`(?m)^\s*[-*]\s+\**Next:\**\s*(.+)$`)
	slugRE      = regexp.MustCompile(`[^a-z0-9]+`)
	agentsRE    = regexp.MustCompile(`(?s)<!-- yass:begin.*?<!-- yass:end -->`)
	codeRefRE   = regexp.MustCompile(`\s*(?:—|-{1,2})?\s*\bcode:\s*`)
	shaRE       = regexp.MustCompile(`^[0-9a-fA-F]{7,40}$`)
	queueRE     = regexp.MustCompile(`^\s*(?:[-*+]|\d+[.)])\s+(?:\[([^\]]+)\]\([^)]*\)|(\S+))`)
)

// QueueName is the optional file in a yass folder that ranks its changes, top first.
const QueueName = "queue.md"

// queueEntry is the change a queue.md line names: the first word of a list item ("- name",
// "1. name"), in backticks or as a link's text if it is one. Anything after it is a note.
func queueEntry(line string) (string, bool) {
	m := queueRE.FindStringSubmatch(line)
	if m == nil {
		return "", false
	}
	name := m[1] + m[2]
	name = strings.TrimRight(strings.Trim(name, "`"), "/")
	return name, name != ""
}

// parseQueue lists the changes a queue.md names, in order, duplicates included.
func parseQueue(text string) []string {
	var out []string
	for _, l := range lines(commentRE.ReplaceAllString(text, "")) {
		if name, ok := queueEntry(l); ok {
			out = append(out, name)
		}
	}
	return out
}

// Box marks: ' ' not started, '/' in progress, 'x' done, '-' dropped.
const (
	markOpen    = " "
	markWorking = "/"
	markDone    = "x"
	markDropped = "-"
)

func isOpen(mark string) bool { return mark == markOpen || mark == markWorking }

func read(path string) string {
	b, err := os.ReadFile(path)
	if err != nil {
		return ""
	}
	return string(b)
}

// normalize turns Windows line endings into \n, so the patterns below see one kind of line.
func normalize(text string) string { return strings.ReplaceAll(text, "\r\n", "\n") }

// readText reads a markdown file for parsing, with normalized line endings.
func readText(path string) string { return normalize(read(path)) }

func write(path, text string) error {
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		return err
	}
	return os.WriteFile(path, []byte(text), 0o644)
}

func exists(path string) bool { _, err := os.Stat(path); return err == nil }

func isDir(path string) bool { st, err := os.Stat(path); return err == nil && st.IsDir() }

func isFile(path string) bool { st, err := os.Stat(path); return err == nil && !st.IsDir() }

func render(name string, kw map[string]string) string {
	b, _ := templates.ReadFile("templates/" + name)
	text := string(b)
	for k, v := range kw {
		text = strings.ReplaceAll(text, "{{"+k+"}}", v)
	}
	return text
}

func lines(text string) []string {
	return strings.Split(normalize(text), "\n")
}

// frontmatter returns the `key: value` pairs between the leading --- lines, and the rest of the text.
func frontmatter(text string) (map[string]string, string) {
	meta := map[string]string{}
	loc := fmRE.FindStringSubmatchIndex(text)
	if loc == nil {
		return meta, text
	}
	for _, line := range lines(text[loc[2]:loc[3]]) {
		k, v, ok := strings.Cut(line, ":")
		if ok && strings.TrimSpace(k) != "" {
			v = fmCommentRE.ReplaceAllString(v, "") // allow trailing "# comments"
			meta[strings.TrimSpace(k)] = strings.TrimSpace(strings.Trim(strings.TrimSpace(v), "[]"))
		}
	}
	return meta, text[loc[1]:]
}

// setMeta sets one frontmatter key, replacing its line if it has one, adding it at the end of the
// frontmatter if not, and adding frontmatter if there's none. The rest of the text is untouched.
func setMeta(text, key, value string) string {
	eol := "\n"
	if strings.Contains(text, "\r\n") {
		eol = "\r\n"
		text = strings.ReplaceAll(text, "\r\n", "\n")
	}
	line := key + ": " + value
	loc := fmRE.FindStringSubmatchIndex(text)
	switch {
	case loc == nil && strings.HasPrefix(text, "---\n---"): // empty frontmatter
		text = "---\n" + line + "\n" + text[len("---\n"):]
	case loc == nil:
		text = "---\n" + line + "\n---\n" + text
	default:
		fm := strings.Split(text[loc[2]:loc[3]], "\n")
		found := false
		for i, l := range fm {
			if k, _, ok := strings.Cut(l, ":"); ok && strings.TrimSpace(k) == key {
				fm[i], found = line, true
				break
			}
		}
		if !found {
			fm = append(fm, line)
		}
		text = text[:loc[2]] + strings.Join(fm, "\n") + text[loc[3]:]
	}
	return strings.ReplaceAll(text, "\n", eol)
}

// section returns the body of `## name`, up to the next `## ` heading.
func section(text, name string) string {
	re := regexp.MustCompile(`(?m)^## ` + regexp.QuoteMeta(name) + `[ \t]*\n`)
	loc := re.FindStringIndex(text)
	if loc == nil {
		return ""
	}
	rest := text[loc[1]:]
	if m := h2RE.FindStringIndex(rest); m != nil {
		return rest[:m[0]]
	}
	return rest
}

type box struct{ mark, text string }

// boxes returns every checkbox outside HTML comments.
func boxes(text string) []box {
	var out []box
	for _, line := range lines(commentRE.ReplaceAllString(text, "")) {
		if m := boxRE.FindStringSubmatch(line); m != nil {
			out = append(out, box{strings.ToLower(m[1]), m[2]})
		}
	}
	return out
}

// codeRefs are the commits a box cites at its end: "… — code: a1b2c3d" or "code: a1b2c3d, e4f5a6b".
// A token naming a repo (web@a1b2c3d) isn't a citation YASS reads.
func codeRefs(text string) []string {
	parts := codeRefRE.Split(text, -1)
	if len(parts) < 2 {
		return nil
	}
	var out []string
	for _, f := range strings.FieldsFunc(parts[len(parts)-1], func(r rune) bool { return r == ',' || r == ' ' }) {
		if shaRE.MatchString(f) {
			out = append(out, strings.ToLower(f))
		}
	}
	return out
}

func slug(text string, n int) string {
	s := strings.Trim(slugRE.ReplaceAllString(strings.ToLower(text), "-"), "-")
	if len(s) > n { // end on a whole word: cut at the last hyphen that fits (one right after n counts)
		if i := strings.LastIndex(s[:n+1], "-"); i > 0 {
			s = s[:i]
		} else {
			s = s[:n]
		}
	}
	if s = strings.TrimRight(s, "-"); s == "" {
		return "change"
	}
	return s
}

// trunc shortens s to at most n characters.
func trunc(s string, n int) string {
	if utf8.RuneCountInString(s) <= n {
		return s
	}
	return string([]rune(s)[:n])
}

// padRight pads s with spaces to n characters.
func padRight(s string, n int) string {
	if c := utf8.RuneCountInString(s); c < n {
		return s + strings.Repeat(" ", n-c)
	}
	return s
}
