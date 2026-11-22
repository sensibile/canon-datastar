package canon

import (
	"strconv"

	"github.com/a-h/templ"
)

// ItemProps configures Item.
type ItemProps struct {
	// As is the element: "div" (default), "li" or "span".
	As    string
	Label string
	// LabelContent replaces Label with markup (no truncation unless LabelLines is set).
	LabelContent       templ.Component
	Description        string
	DescriptionContent templ.Component
	StartContent       templ.Component
	EndContent         templ.Component
	// Align: "center" (default), "start" or "first-line".
	Align string
	// Density: "balanced" (default), "compact" or "spacious".
	Density string
	// LabelLines clamps the label to this many lines; plain labels truncate to one line by default.
	LabelLines       int
	DescriptionLines int
	// Layout "inline" puts the description on the label's line.
	Layout string
	// Href makes the row a link.
	Href, Target, Rel string
	// Action makes the row a button and goes on it, e.g. {{"data-on:click", "@post('/x')"}}.
	// A non-nil empty slice still renders the button.
	Action []Attr
	// IsDelegate marks a row whose clicks canon.js forwards to the control in StartContent
	// (a checkbox or radio).
	IsDelegate    bool
	IsHighlighted bool
	IsSelected    bool
	IsDisabled    bool
	LabelID       string
	DescriptionID string
	// Role is set when a parent widget (listbox, menu) owns the row's semantics.
	Role  string
	Class string
	Attrs []Attr
}

var ariaSelectedRoles = map[string]bool{"option": true, "tab": true, "row": true, "gridcell": true,
	"columnheader": true, "rowheader": true, "treeitem": true}

// Item renders the row primitive behind lists, menus and choice lists: start content,
// label + description, end content. It is a button (Action), a link (Href), or forwards row
// clicks to a control inside it (IsDelegate).
// Port of Astryx `Item` (packages/core/src/Item), MIT, Meta Platforms, Inc.
// Not ported: swipe actions, marker columns, list-row link bands, touch press feedback.
func Item(p ItemProps) templ.Component {
	hasRole := p.Role != ""
	isLink := p.Href != "" && !hasRole && !p.IsDelegate
	isButton := !isLink && p.Action != nil && !hasRole && !p.IsDelegate
	interactive := p.Href != "" || p.Action != nil || p.IsDelegate
	hasDesc := p.Description != "" || p.DescriptionContent != nil
	inline := p.Layout == "inline" && hasDesc
	allowsSelected := hasRole && ariaSelectedRoles[p.Role]
	current := ""
	if p.IsSelected && !allowsSelected {
		current = "true"
	}
	currentOnControl := !hasRole && (isLink || isButton)
	align := or(p.Align, "center")

	labelTrunc := truncation(p.LabelLines, p.LabelContent == nil)
	descTrunc := truncation(p.DescriptionLines, p.DescriptionContent == nil || inline)
	var label templ.Component = text(p.Label)
	if p.LabelContent != nil {
		label = p.LabelContent
	}
	body := []templ.Component{el("span", newAttrs().opt("id", p.LabelID).set("class", "astryx-item-label").
		flag("data-inline", inline).opt("data-truncate", labelTrunc).
		opt("style", clamp(p.LabelLines)), label)}
	if hasDesc {
		var desc templ.Component = text(p.Description)
		if p.DescriptionContent != nil {
			desc = p.DescriptionContent
		}
		body = append(body, el("span", newAttrs().opt("id", p.DescriptionID).set("class", "astryx-item-description").
			flag("data-inline", inline).opt("data-truncate", descTrunc).
			opt("style", clamp(p.DescriptionLines)), desc))
	}

	var content templ.Component
	switch {
	case isLink:
		a := newAttrs().set("class", "astryx-item-action").flag("data-inline", inline)
		if p.IsDisabled {
			a.set("role", "link").set("aria-disabled", "true").set("tabindex", "0")
		} else {
			rel := p.Rel
			if rel == "" && p.Target == "_blank" {
				rel = "noopener noreferrer"
			}
			a.set("href", p.Href).opt("target", p.Target).opt("rel", rel)
		}
		content = el("a", a.opt("aria-current", current), body...)
	case isButton:
		action := p.Action
		if p.IsDisabled {
			action = withoutPrefix(action, "data-on:")
		}
		content = el("button", newAttrs().set("type", "button").set("class", "astryx-item-action").
			flag("data-inline", inline).
			when(p.IsDisabled, "aria-disabled", "true").
			opt("aria-current", current).extra(action), body...)
	default:
		content = el("span", newAttrs().set("class", "astryx-item-content").flag("data-inline", inline), body...)
	}
	var start, end templ.Component
	if p.StartContent != nil {
		start = el("span", newAttrs().set("class", "astryx-item-start").flag("data-first-line", align == "first-line"), p.StartContent)
	}
	if p.EndContent != nil {
		end = el("span", newAttrs().set("class", "astryx-item-end").flag("data-first-line", align == "first-line"), p.EndContent)
	}
	a := newAttrs().set("class", cls("astryx-item", p.Class)).
		set("data-density", or(p.Density, "balanced")).
		set("data-align", align).
		flag("data-interactive", interactive).
		flag("data-delegate", p.IsDelegate).
		flag("data-highlighted", p.IsHighlighted).
		flag("data-selected", p.IsSelected).
		flag("data-disabled", p.IsDisabled).
		flag("data-has-role", hasRole).
		opt("role", p.Role).
		when(allowsSelected && p.IsSelected, "aria-selected", "true").
		when(!currentOnControl && current != "", "aria-current", current).
		when(p.IsDisabled, "aria-disabled", "true").
		extra(p.Attrs)
	return el(or(p.As, "div"), a, start, content, end)
}

func truncation(lines int, plain bool) string {
	switch {
	case lines == 1:
		return "single"
	case lines > 1:
		return "multi"
	case plain:
		return "single"
	}
	return ""
}

func clamp(lines int) string {
	if lines > 1 {
		return "-webkit-line-clamp: " + strconv.Itoa(lines)
	}
	return ""
}
