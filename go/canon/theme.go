// Package canon renders Astryx components as server-side HTML with templ.
// Markup (class names, data-* attributes) matches the Rust crate and the golden
// files in fixtures/, and is styled by the shared css/ stylesheets.
package canon

import "github.com/a-h/templ"

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

// Attr is one HTML attribute, e.g. a Datastar binding {"data-bind:email", ""}. Order is kept so Go and Rust render identical markup.
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

// Theme scopes an Astryx theme and color mode to its children.
// Port of Astryx `Theme` (packages/core/src/theme/Theme.tsx), MIT, Meta Platforms, Inc.
// For the page root, put the same data-astryx-theme / data-theme attributes on <html>
// so browser chrome and top-layer overlays follow it.
func Theme(p ThemeProps) templ.Component {
	return withChildren(func(children templ.Component) templ.Component {
		return el("div", newAttrs().
			set("class", "astryx-theme").
			set("data-astryx-theme", p.name()).
			opt("data-theme", p.Mode.DataTheme()).
			extra(p.Attrs), children)
	})
}

// A builds an Attr, e.g. canon.A("data-bind:email", "").
func A(name, value string) Attr { return Attr{Name: name, Value: value} }
