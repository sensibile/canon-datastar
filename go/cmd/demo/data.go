package main

// Token lists shown on the demo's Tokens page. Mirrors rust/examples/demo.
type group struct {
	Name string
	Vars []string
}

func prefixed(prefix string, names ...string) []string {
	out := make([]string, len(names))
	for i, n := range names {
		out[i] = prefix + n
	}
	return out
}

var colorGroups = []group{
	{"Background", prefixed("--color-background-", "body", "surface", "card", "muted", "popover", "inverted")},
	{"Text", prefixed("--color-text-", "primary", "secondary", "disabled", "accent", "error", "success", "warning", "info")},
	{"Accent", []string{"--color-accent", "--color-accent-muted", "--color-on-accent"}},
	{"Status", []string{"--color-error", "--color-error-muted", "--color-success", "--color-success-muted", "--color-warning", "--color-warning-muted", "--color-info", "--color-info-muted"}},
	{"Border", prefixed("--color-border", "", "-emphasized", "-error", "-success", "-warning", "-info")},
}

var (
	typeStyles = []string{"display-1", "display-2", "display-3", "heading-1", "heading-2", "heading-3", "heading-4", "heading-5", "heading-6", "large", "body", "label", "supporting", "code"}
	spacing    = []string{"0-5", "1", "1-5", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12"}
	radii      = []string{"inner", "element", "container", "page", "full"}
	shadows    = []string{"low", "med", "high"}
)
