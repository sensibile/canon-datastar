package canon

import (
	"strings"

	"github.com/a-h/templ"
)

// Port of Astryx `Link` (packages/core/src/Link), MIT, Meta Platforms, Inc.
// Renders <a href> normally, a <button type="button"> when there is no Href (wire it up with
// data-on:click), and an inert role="link" when disabled. Not ported: LinkProvider, tooltip.

// LinkProps configures Link.
type LinkProps struct {
	Href string
	// Label is the accessible name, when the visible text isn't enough.
	Label        string
	Target, Rel  string
	HasUnderline bool
	IsDisabled   bool
	// IsExternalLink opens in a new tab and shows an external-link icon.
	IsExternalLink bool
	// NewTabLabel is the hidden text for external links ("(opens in a new tab)").
	NewTabLabel string
	// IsStandalone uses body-sized text outside running prose.
	IsStandalone bool
	// Type, Size, Weight, Display, MaxLines style the label like Text.
	Type, Size, Weight, Display string
	MaxLines                    int
	// Color: "accent" (default), "primary", "secondary", "disabled", "inherit".
	Color string
	Class string
	Attrs []Attr
}

// Link renders a text link around its children.
func Link(p LinkProps) templ.Component {
	return withChildren(func(children templ.Component) templ.Component {
		target := p.Target
		if p.IsExternalLink {
			target = "_blank"
		}
		rel := p.Rel
		if target == "_blank" {
			tokens := strings.Fields(p.Rel)
			for _, t := range []string{"noopener", "noreferrer"} {
				found := false
				for _, x := range tokens {
					found = found || x == t
				}
				if !found {
					tokens = append(tokens, t)
				}
			}
			rel = strings.Join(tokens, " ")
		}
		asButton := p.Href == "" && !p.IsDisabled
		a := newAttrs().set("class", cls("astryx-link", p.Class)).
			set("data-color", or(p.Color, "accent")).
			flag("data-underline", p.HasUnderline).
			flag("data-standalone", p.IsStandalone).
			flag("data-truncating", p.MaxLines > 0).
			opt("aria-label", p.Label)
		tag := "a"
		switch {
		case asButton:
			tag = "button"
			a.set("type", "button")
		case p.IsDisabled:
			a.set("role", "link").set("tabindex", "0").set("aria-disabled", "true")
		default:
			a.set("href", p.Href).opt("target", target).opt("rel", rel)
		}
		if p.IsDisabled {
			// A disabled link must not run its Datastar actions.
			a.extra(withoutPrefix(p.Attrs, "data-on:"))
		} else {
			a.extra(p.Attrs)
		}
		label := textEl(TextProps{Type: or(p.Type, "body"), Size: p.Size, Weight: p.Weight, Color: "inherit",
			Display: p.Display, MaxLines: p.MaxLines, Class: "astryx-link-label"}, children)
		var external templ.Component
		if p.IsExternalLink && !asButton {
			external = group(Icon(IconProps{Name: "externalLink", Size: "xsm"}),
				visuallyHidden(VisuallyHiddenProps{}, text(or(p.NewTabLabel, "(opens in a new tab)"))))
		}
		return el(tag, a, label, external)
	})
}
