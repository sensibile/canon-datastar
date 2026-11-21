package canon

import "github.com/a-h/templ"

// Port of Astryx `DropdownMenu` (packages/core/src/DropdownMenu), MIT, Meta Platforms, Inc.
// A Button that opens a native popover menu (popovertarget). static/canon.js adds the menu
// keyboard model: Enter/Space/ArrowDown opens on the first item, ArrowUp on the last; arrows,
// Home/End and typeahead move between items; Escape closes and returns focus; Tab closes.
// Wire item actions with Datastar, e.g. Attrs: []Attr{{"data-on:click", "@post('/archive')"}}.
// Not ported: submenus, checkbox/radio items, bottom-sheet presentation on touch.

// DropdownMenuProps configures DropdownMenu.
type DropdownMenuProps struct {
	// ID of the menu; the trigger is ID+"-trigger" and the surface ID+"-surface". Default "menu".
	ID string
	// Button configures the trigger Button.
	Button ButtonProps
	// NoChevron drops the chevron after the trigger label.
	NoChevron bool
	// Placement: "below" (default), "above", "start", "end". Alignment: "start" (default), "center", "end".
	Placement string
	Alignment string
	// MenuWidth is a CSS length; by default the menu is at least as wide as the trigger.
	MenuWidth string
	// MenuMaxHeight is a CSS length.
	MenuMaxHeight string
	Header        templ.Component
	Footer        templ.Component
	Class         string
}

// DropdownMenu renders the trigger and the menu; its children are the items.
func DropdownMenu(p DropdownMenuProps) templ.Component {
	return withChildren(func(children templ.Component) templ.Component {
		id := or(p.ID, "menu")
		surfaceID := id + "-surface"
		btn := p.Button
		if !p.NoChevron && btn.EndContent == nil {
			btn.EndContent = Icon(IconProps{Name: "chevronDown", Size: "sm"})
		}
		btn.Attrs = append([]Attr{
			{"id", id + "-trigger"},
			{"popovertarget", surfaceID},
			{"aria-haspopup", "menu"},
			{"aria-expanded", "false"},
			{"aria-controls", id},
		}, btn.Attrs...)
		surface := newAttrs().set("popover", "auto").
			set("id", surfaceID).
			set("class", cls("astryx-layer astryx-dropdown-menu-surface", p.Class)).
			set("data-placement", or(p.Placement, "below")).
			set("data-alignment", or(p.Alignment, "start")).
			flag("data-match-width", p.MenuWidth == "").
			opt("style", decl("min-width", p.MenuWidth))
		maxHeight := ""
		if p.MenuMaxHeight != "" {
			maxHeight = "max-height: min(" + p.MenuMaxHeight + ", calc(100vh - 16px))"
		}
		menu := newAttrs().set("id", id).
			set("role", "menu").
			set("tabindex", "-1").
			set("aria-labelledby", id+"-trigger").
			set("class", "astryx-dropdown-menu").
			opt("style", maxHeight)
		return group(
			Button(btn),
			el("div", surface, region(p.Header), el("div", menu, children), region(p.Footer)),
		)
	})
}

func region(c templ.Component) templ.Component {
	if c == nil {
		return nil
	}
	return el("div", newAttrs().set("class", "astryx-dropdown-menu-region"), c)
}

// DropdownMenuItemProps configures DropdownMenuItem.
type DropdownMenuItemProps struct {
	Label       string
	Description string
	// Icon is a leading built-in glyph name.
	Icon string
	// Href renders a link item.
	Href, Target, Rel string
	IsDisabled        bool
	// Variant: "default" or "destructive".
	Variant string
	// KeepOpen leaves the menu open after this item is chosen.
	KeepOpen   bool
	EndContent templ.Component
	Class      string
	Attrs      []Attr
}

// DropdownMenuItem renders one menu row: a <button>, or an <a> when Href is set.
func DropdownMenuItem(p DropdownMenuItemProps) templ.Component {
	variant := or(p.Variant, "default")
	isLink := p.Href != "" && !p.IsDisabled
	a := newAttrs()
	tag := "button"
	if isLink {
		tag = "a"
		rel := p.Rel
		if p.Target == "_blank" && rel == "" {
			rel = "noopener noreferrer"
		}
		a.set("href", p.Href).opt("target", p.Target).opt("rel", rel)
	} else {
		a.set("type", "button")
	}
	a.set("role", "menuitem").
		set("tabindex", "-1").
		when(isLink, "draggable", "false").
		set("class", cls("astryx-dropdown-menu-item", p.Class)).
		set("data-variant", variant).
		when(p.IsDisabled, "aria-disabled", "true").
		flag("data-keep-open", p.KeepOpen)
	if p.IsDisabled {
		a.extra(withoutPrefix(p.Attrs, "data-on:"))
	} else {
		a.extra(p.Attrs)
	}
	var icon, desc, end templ.Component
	if p.Icon != "" {
		color := "secondary"
		if variant == "destructive" {
			color = "error"
		}
		icon = Icon(IconProps{Name: p.Icon, Size: "sm", Color: color})
	}
	if p.Description != "" {
		desc = el("span", newAttrs().set("class", "astryx-dropdown-menu-item-description"), text(p.Description))
	}
	if p.EndContent != nil {
		end = el("span", newAttrs().set("class", "astryx-dropdown-menu-item-end"), p.EndContent)
	}
	return el(tag, a, icon,
		el("span", newAttrs().set("class", "astryx-dropdown-menu-item-text"),
			el("span", newAttrs().set("class", "astryx-dropdown-menu-item-label"), text(p.Label)), desc),
		end)
}

// DropdownMenuGroupProps configures DropdownMenuGroup.
type DropdownMenuGroupProps struct {
	Title string
	// ID of the group; the title is ID+"-title". Defaults to "menu-group-" + a slug of Title.
	ID string
}

// DropdownMenuGroup renders a titled group of items (its children).
func DropdownMenuGroup(p DropdownMenuGroupProps) templ.Component {
	return withChildren(func(children templ.Component) templ.Component {
		titleID := ""
		if p.Title != "" {
			titleID = or(p.ID, "menu-group-"+slug(p.Title)) + "-title"
		}
		var title templ.Component
		if p.Title != "" {
			title = el("div", newAttrs().set("class", "astryx-dropdown-menu-group-title").set("id", titleID), text(p.Title))
		}
		return el("div", newAttrs().set("class", "astryx-dropdown-menu-group").set("role", "group").opt("aria-labelledby", titleID),
			title, children)
	})
}

// DropdownMenuDivider renders a separator between items.
func DropdownMenuDivider() templ.Component {
	return el("div", newAttrs().set("class", "astryx-dropdown-menu-divider").set("role", "separator").
		set("aria-orientation", "horizontal"), Divider(DividerProps{}))
}
