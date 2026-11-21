package canon

import "github.com/a-h/templ"

// Port of Astryx `Icon` (packages/core/src/Icon/Icon.tsx), MIT, Meta Platforms, Inc.

type glyph struct {
	solid       bool
	strokeWidth string
	body        string
}

// IconProps configures Icon.
type IconProps struct {
	// Name is a built-in glyph (see icons/default-icons.json), e.g. "check", "close", "chevronDown".
	Name string
	// Size: "xsm", "sm", "md" (default), "lg".
	Size string
	// Color: "inherit" (default), "primary", "secondary", "accent", "error", "blue", ...
	Color string
	// Label names the icon for assistive tech; without it the icon is decorative.
	Label string
	Class string
	Attrs []Attr
}

// IconNames lists the built-in glyph names.
func IconNames() []string {
	names := make([]string, 0, len(defaultIcons))
	for n := range defaultIcons {
		names = append(names, n)
	}
	return names
}

// Icon renders a built-in glyph. An unknown name renders nothing.
func Icon(p IconProps) templ.Component {
	g, ok := defaultIcons[p.Name]
	if !ok {
		return raw("")
	}
	a := newAttrs().set("class", cls("astryx-icon", p.Class)).
		set("data-size", or(p.Size, "md")).
		set("data-color", or(p.Color, "inherit"))
	if p.Label != "" {
		a.set("role", "img").set("aria-label", p.Label)
	} else {
		a.set("aria-hidden", "true")
	}
	a.extra(p.Attrs)
	svg := newAttrs().set("xmlns", "http://www.w3.org/2000/svg").set("viewBox", "0 0 24 24").
		set("width", "1em").set("height", "1em").set("aria-hidden", "true")
	if g.solid {
		svg.set("fill", "currentColor")
	} else {
		svg.set("fill", "none").set("stroke", "currentColor").set("stroke-width", g.strokeWidth).
			set("stroke-linecap", "round").set("stroke-linejoin", "round")
	}
	return el("span", a, el("svg", svg, raw(g.body)))
}
