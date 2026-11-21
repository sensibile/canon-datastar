package canon

import (
	"context"
	"io"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/a-h/templ"
)

// Each case renders to exactly fixtures/<component>/<case>.html, the same files the
// Rust crate tests against. Set UPDATE_FIXTURES=1 to rewrite them from Go output.
type fixture struct {
	name string
	c    templ.Component
}

func checkFixtures(t *testing.T, component string, cases []fixture) {
	t.Helper()
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			var b strings.Builder
			if err := tc.c.Render(context.Background(), &b); err != nil {
				t.Fatal(err)
			}
			got := normalize(b.String())
			path := filepath.Join("..", "..", "fixtures", component, tc.name+".html")
			if os.Getenv("UPDATE_FIXTURES") != "" {
				if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
					t.Fatal(err)
				}
				if err := os.WriteFile(path, []byte(got+"\n"), 0o644); err != nil {
					t.Fatal(err)
				}
				return
			}
			want, err := os.ReadFile(path)
			if err != nil {
				t.Fatal(err)
			}
			if got != strings.TrimSuffix(string(want), "\n") {
				t.Errorf("%s\n got: %s\nwant: %s", path, got, want)
			}
		})
	}
}

// normalize maps escaping that differs between templ and maud to one spelling:
// templ writes &#34; and &#39;, maud writes &quot; and leaves ' alone.
func normalize(s string) string {
	return strings.NewReplacer("&#34;", "&quot;", "&#39;", "'").Replace(s)
}

// child is the shared fixture body used wherever a component takes children.
var child = templ.Raw(`<p>Hello</p>`)

func with(c templ.Component, children templ.Component) templ.Component {
	return templ.ComponentFunc(func(ctx context.Context, w io.Writer) error {
		return c.Render(templ.WithChildren(ctx, children), w)
	})
}
