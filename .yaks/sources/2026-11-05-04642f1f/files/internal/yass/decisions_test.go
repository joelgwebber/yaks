package yass

import "testing"

func TestParseEntry(t *testing.T) {
	for _, tc := range []struct{ line, text, who, date string }{
		{"X - Y (Jaime, 2026-10-07)", "X - Y", "Jaime", "2026-10-07"},
		{"X - Y (claude, agreed by Jaime)", "X - Y", "claude, agreed by Jaime", ""},
		{"X - Y (Jaime, months from claude, 2026-10-07)", "X - Y", "Jaime, months from claude", "2026-10-07"},
		{"X (see #41) - Y (sam)", "X (see #41) - Y", "sam", ""},
		{"No credit at all", "No credit at all", "", ""},
		{"Odd (2026-10-07)", "Odd", "2026-10-07", ""},
	} {
		text, who, date := parseEntry(tc.line)
		if text != tc.text || who != tc.who || date != tc.date {
			t.Errorf("parseEntry(%q) = %q, %q, %q; want %q, %q, %q", tc.line, text, who, date, tc.text, tc.who, tc.date)
		}
	}
}
