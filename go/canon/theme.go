// Package canon renders Astryx components as server-side HTML with templ.
// Markup (class names, data-* attributes) matches the Rust crate and the golden
// files in fixtures/, and is styled by the shared css/ stylesheets.
package canon

// ThemeMode is a Theme's color mode.
type ThemeMode string

const (
	// ModeSystem follows the reader's OS color scheme (no data-theme attribute).
	ModeSystem ThemeMode = "system"
	ModeLight  ThemeMode = "light"
	ModeDark   ThemeMode = "dark"
)

// Themes lists the theme names shipped in css/themes.
var Themes = []string{"neutral", "butter", "chocolate", "matcha", "stone", "gothic", "y2k"}

// ThemeProps configures Theme. Zero values mean theme "neutral", mode "system".
type ThemeProps struct {
	// Name matches a theme stylesheet linked on the page (css/themes/<name>.css).
	Name string
	Mode ThemeMode
	// Attrs adds extra attributes, e.g. Datastar bindings such as
	// {"data-attr:data-astryx-theme", "$theme"}. Rendered in order, after the theme attributes.
	Attrs []Attr
}

// Attr is one HTML attribute. Order is kept so Go and Rust render identical markup.
type Attr struct{ Name, Value string }

func (p ThemeProps) name() string {
	if p.Name == "" {
		return "neutral"
	}
	return p.Name
}

// DataTheme returns the data-theme value for a mode, or "" for system.
func (m ThemeMode) DataTheme() string {
	switch m {
	case ModeLight, ModeDark:
		return string(m)
	}
	return ""
}
