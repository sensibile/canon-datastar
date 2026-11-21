package canon

import "github.com/a-h/templ"

// Port of Astryx `Collapsible` and `CollapsibleGroup` (packages/core/src/Collapsible), MIT,
// Meta Platforms, Inc. static/canon.js toggles a Collapsible on click (closing its siblings in a
// "single" group) and fires a bubbling "canon-toggle" event with detail.open. Signal mirrors the
// open state in a Datastar signal both ways. Not ported: the animatedLayout height animation.

// CollapsibleProps configures Collapsible.
type CollapsibleProps struct {
	// ID of the root; the content is ID + "-content". Defaults to "collapsible-" + a slug of Trigger.
	ID string
	// Trigger is the button text; TriggerMarkup replaces it.
	Trigger       string
	TriggerMarkup templ.Component
	// IsClosed starts it closed (Astryx opens by default).
	IsClosed   bool
	IsDisabled bool
	// ChevronPosition: "end" (default) or "start".
	ChevronPosition string
	// Value identifies it inside a CollapsibleGroup.
	Value string
	// Signal names a boolean Datastar signal that mirrors the open state.
	Signal string
	// Content is the body; Collapsible also takes it as templ children.
	Content templ.Component
	Class   string
	Attrs   []Attr
}

type collapsibleGroupCtx struct {
	divided   bool
	density   string
	chevron   string
	openValue map[string]bool
}

// Collapsible renders a disclosure: a full-width trigger and content that shows when open.
func Collapsible(p CollapsibleProps) templ.Component {
	return withChildren(func(children templ.Component) templ.Component {
		if p.Content == nil {
			p.Content = children
		}
		return collapsible(p, nil)
	})
}

func collapsible(p CollapsibleProps, g *collapsibleGroupCtx) templ.Component {
	id := or(p.ID, "collapsible-"+or(slug(p.Trigger), "item"))
	contentID := id + "-content"
	open := !p.IsClosed
	position := p.ChevronPosition
	var divided bool
	var density string
	if g != nil {
		divided, density = g.divided, g.density
		position = or(position, g.chevron)
		if p.Value != "" {
			open = g.openValue[p.Value]
		}
	}
	position = or(position, "end")
	chevronIcon := "chevronDown"
	if position == "start" {
		chevronIcon = "chevronRight"
	}
	chevron := el("span", newAttrs().set("class", "astryx-collapsible-chevron"),
		Icon(IconProps{Name: chevronIcon, Size: "sm", Color: "secondary"}))
	var before, after templ.Component
	if position == "start" {
		before = chevron
	} else {
		after = chevron
	}
	label := p.TriggerMarkup
	if label == nil {
		label = text(p.Trigger)
	}
	btn := newAttrs().set("type", "button").
		set("class", "astryx-collapsible-trigger").
		set("data-chevron-position", position).
		flag("data-disabled", p.IsDisabled).
		when(p.IsDisabled, "aria-disabled", "true").
		set("aria-expanded", map[bool]string{true: "true", false: "false"}[open]).
		set("aria-controls", contentID).
		when(p.IsDisabled, "tabindex", "-1")
	root := newAttrs().set("id", id).
		set("class", cls("astryx-collapsible", p.Class)).
		flag("data-divided", divided).
		opt("data-density", density).
		opt("data-value", p.Value)
	if p.Signal != "" {
		root.set("data-effect", "canon.collapse(el, $"+p.Signal+")").
			set("data-on:canon-toggle", "evt.target === el && ($"+p.Signal+" = evt.detail.open)")
	}
	root.extra(p.Attrs)
	return el("div", root,
		el("button", btn, before, el("span", newAttrs().set("class", "astryx-collapsible-label"), label), after),
		el("div", newAttrs().set("id", contentID).set("class", "astryx-collapsible-content").flag("hidden", !open),
			el("div", newAttrs(), p.Content)))
}

// CollapsibleGroupProps configures CollapsibleGroup.
type CollapsibleGroupProps struct {
	// Type: "single" (default, at most one open, an accordion) or "multiple".
	Type string
	// Value lists the items open at first (by their Value).
	Value       []string
	HasDividers bool
	// Density: "compact", "balanced" (default with dividers), "spacious".
	Density         string
	ChevronPosition string
	Items           []CollapsibleProps
	Class           string
	Attrs           []Attr
}

// CollapsibleGroup coordinates Collapsibles by Value.
func CollapsibleGroup(p CollapsibleGroupProps) templ.Component {
	density := p.Density
	if density == "" && p.HasDividers {
		density = "balanced"
	}
	g := &collapsibleGroupCtx{divided: p.HasDividers, density: density, chevron: p.ChevronPosition, openValue: map[string]bool{}}
	typ := or(p.Type, "single")
	for _, v := range p.Value {
		g.openValue[v] = true
		if typ == "single" {
			break
		}
	}
	items := make([]templ.Component, len(p.Items))
	for i, it := range p.Items {
		items[i] = collapsible(it, g)
	}
	a := newAttrs().set("class", cls("astryx-collapsible-group", p.Class)).
		set("data-type", typ).
		flag("data-dividers", p.HasDividers).
		opt("data-density", density).
		extra(p.Attrs)
	return el("div", a, items...)
}
