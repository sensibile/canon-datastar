package canon

import "github.com/a-h/templ"

// Port of Astryx `Token` (packages/core/src/Token, TokenLink), MIT, Meta Platforms, Inc.
// Href makes the chip a link and Action (e.g. a data-on:click) a button. With a remove button,
// or as a button, the label is its own <a>/<button> inside the chip and static/canon.js forwards
// clicks on the chip body to it. A disabled token drops its data-on: handlers.

// TokenProps configures Token.
type TokenProps struct {
	Label string
	// Size: "sm", "md" (default), "lg".
	Size string
	// Color: "default", "red", "orange", "yellow", "green", "teal", "cyan", "blue", "purple",
	// "pink", "gray".
	Color      string
	Icon       templ.Component
	EndContent templ.Component
	IsDisabled bool
	Href       string
	// Action holds the label control's attributes when the token is a button, e.g.
	// {"data-on:click", "@post('/filter')"}. Any non-nil Action (even empty) makes it a button.
	Action []Attr
	// IsRemovable (or any RemoveAttrs) adds a remove button; RemoveAttrs are its attributes.
	IsRemovable bool
	RemoveAttrs []Attr
	// RemoveName names the remove target when it differs from Label.
	RemoveName string
	// Description is extra text for assistive tech (aria-description).
	Description   string
	IsLabelHidden bool
	// AriaLabel overrides the accessible name.
	AriaLabel string
	Class     string
	Attrs     []Attr
}

// Token renders a compact chip.
func Token(p TokenProps) templ.Component {
	isButton := p.Action != nil
	isLink := p.Href != "" && !isButton
	removable := p.IsRemovable || len(p.RemoveAttrs) > 0
	size := or(p.Size, "md")
	color := or(p.Color, "default")

	// Naming goes on whichever element is the control.
	shared := func(a *attrs) *attrs {
		name := p.AriaLabel
		if name == "" && p.IsLabelHidden {
			name = p.Label
		}
		return a.opt("aria-label", name).opt("aria-description", p.Description)
	}
	base := func(a *attrs, interactive, owner bool) *attrs {
		return a.set("class", cls("astryx-token", p.Class)).
			flag("data-interactive", interactive).
			flag("data-owner", owner).
			set("data-color", color).
			set("data-size", size).
			flag("data-disabled", p.IsDisabled)
	}
	link := func(a *attrs) *attrs {
		if p.IsDisabled {
			return a.set("role", "link").set("tabindex", "0").set("aria-disabled", "true")
		}
		return a.set("href", p.Href)
	}
	label := el("span", newAttrs().set("dir", "auto").set("class", "astryx-token-label").flag("data-hidden", p.IsLabelHidden), text(p.Label))
	var remove templ.Component
	if removable {
		ra := newAttrs().set("type", "button").
			set("class", "astryx-token-remove").
			set("aria-label", "Remove "+or(p.RemoveName, p.Label)).
			when(p.IsDisabled, "aria-disabled", "true")
		if p.IsDisabled {
			ra.extra(withoutPrefix(p.RemoveAttrs, "data-on:"))
		} else {
			ra.extra(p.RemoveAttrs)
		}
		remove = el("button", ra, Icon(IconProps{Name: "close", Size: "xsm"}))
	}

	switch {
	case isLink && !removable:
		a := shared(link(newAttrs()))
		return el("a", base(a, true, false).extra(p.Attrs), p.Icon, label, p.EndContent)
	case isLink || isButton:
		var inner templ.Component
		if isLink {
			inner = el("a", shared(link(newAttrs())).set("class", "astryx-token-inner"), label)
		} else {
			ia := shared(newAttrs().set("type", "button")).when(p.IsDisabled, "aria-disabled", "true").set("class", "astryx-token-inner")
			if p.IsDisabled {
				ia.extra(withoutPrefix(p.Action, "data-on:"))
			} else {
				ia.extra(p.Action)
			}
			inner = el("button", ia, label)
		}
		return el("span", base(newAttrs(), true, true).extra(p.Attrs), p.Icon, inner, p.EndContent, remove)
	default:
		return el("span", base(shared(newAttrs()), false, false).extra(p.Attrs), p.Icon, label, p.EndContent, remove)
	}
}
