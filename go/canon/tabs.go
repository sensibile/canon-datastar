package canon

import (
	"strconv"
	"strings"

	"github.com/a-h/templ"
)

// Port of Astryx `TabList` and `Tab` (packages/core/src/TabList), MIT, Meta Platforms, Inc.
// Without a Role it is a <nav> of links or buttons marked with aria-current; with
// Role "tablist" it follows the tabs pattern (role=tab, aria-selected, PanelID). With Signal the
// selection is a Datastar signal: clicking a tab sets it and every tab and TabPanel follows it.
// static/canon.js moves focus with the arrow keys, Home and End (roving tabindex).
// Not ported: overflow scroll arrows and edge fades, TabMenu.

// TabListProps configures TabList.
type TabListProps struct {
	// Value is the selected tab's value.
	Value string
	// Signal names a Datastar signal holding the selected value (initialise it to Value).
	Signal string
	Tabs   []TabProps
	// Size: "sm", "md" (default), "lg". Layout: "hug" (default) or "fill".
	Size       string
	Layout     string
	HasDivider bool
	// Role: "" (navigation) or "tablist".
	Role string
	// Label is the accessible name, default "Tabs"; LabelledBy names it by another element.
	Label      string
	LabelledBy string
	Class      string
	Attrs      []Attr
}

// TabProps configures one tab of a TabList.
type TabProps struct {
	Value string
	Label string
	// IsLabelHidden shows only the icon; Label becomes the accessible name.
	IsLabelHidden bool
	// Href makes a link tab (navigation TabLists only).
	Href string
	ID   string
	// PanelID is the id of the panel this tab controls (Role "tablist" only).
	PanelID string
	// Icon and SelectedIcon are built-in glyph names; SelectedIcon shows while selected.
	Icon         string
	SelectedIcon string
	EndContent   templ.Component
	IsDisabled   bool
	Class        string
	Attrs        []Attr
}

// TabList renders a row of tabs.
func TabList(p TabListProps) templ.Component {
	size := or(p.Size, "md")
	layout := or(p.Layout, "hug")
	isTabList := p.Role == "tablist"
	label := or(p.Label, "Tabs")
	if p.LabelledBy != "" && p.Label == "" {
		label = ""
	}
	root := newAttrs().set("class", cls("astryx-tab-list", p.Class)).
		set("data-size", size).
		set("data-layout", layout).
		flag("data-divider", p.HasDivider)
	strip := newAttrs().set("class", "astryx-tab-strip")
	tag := "nav"
	if isTabList {
		tag = "div"
		strip.set("role", "tablist").opt("aria-label", label).opt("aria-labelledby", p.LabelledBy)
	} else {
		root.opt("aria-label", label).opt("aria-labelledby", p.LabelledBy)
	}
	root.extra(p.Attrs)
	stop := tabStop(p)
	tabs := make([]templ.Component, len(p.Tabs))
	for i, t := range p.Tabs {
		tabs[i] = tab(t, p, size, layout, isTabList, i == stop)
	}
	return el(tag, root, el("div", strip, tabs...))
}

// tabStop picks the tab that is in the Tab order: the selected one, or the first enabled tab
// when no enabled tab matches Value, so keyboard users can always reach the list.
func tabStop(p TabListProps) int {
	first := -1
	for i, t := range p.Tabs {
		if t.Value == p.Value && !t.IsDisabled {
			return i
		}
		if first < 0 && !t.IsDisabled {
			first = i
		}
	}
	return first
}

func tab(t TabProps, p TabListProps, size, layout string, isTabList, isStop bool) templ.Component {
	selected := t.Value == p.Value
	isLink := t.Href != "" && !isTabList
	tag := "button"
	a := newAttrs().set("class", cls("astryx-tab", t.Class)).
		opt("id", t.ID).
		set("data-tab-value", t.Value).
		set("data-size", size).
		set("data-layout", layout).
		when(selected, "data-selected", "selected").
		when(t.IsLabelHidden, "aria-label", t.Label).
		when(t.IsDisabled, "aria-disabled", "true")
	if isTabList {
		a.set("role", "tab").set("aria-selected", strconv.FormatBool(selected)).opt("aria-controls", t.PanelID)
	} else {
		a.when(selected, "aria-current", "true")
	}
	tabindex := "-1"
	if isStop {
		tabindex = "0"
	}
	a.set("tabindex", tabindex)
	if isLink {
		tag = "a"
		a.when(!t.IsDisabled, "href", t.Href)
	} else {
		a.set("type", "button")
	}
	if p.Signal != "" {
		is := "$" + p.Signal + " === " + jsString(t.Value)
		a.set("data-attr:data-selected", is+" && 'selected'")
		if isTabList {
			a.set("data-attr:aria-selected", "String("+is+")")
		} else {
			a.set("data-attr:aria-current", is+" && 'true'")
		}
	}
	switch {
	case t.IsDisabled:
		// Selection attributes (above) still follow the signal; activation stays off.
		a.extra(withoutPrefix(t.Attrs, "data-on:"))
	case p.Signal != "":
		is := "$" + p.Signal + " === " + jsString(t.Value)
		// Selecting runs first, then the caller's own click action (one attribute holds both).
		click := "$" + p.Signal + " = " + jsString(t.Value)
		var rest []Attr
		for _, at := range t.Attrs {
			if at.Name == "data-on:click" {
				click += "; " + at.Value
			} else {
				rest = append(rest, at)
			}
		}
		tabindex := is + " ? 0 : -1"
		if fallbackStop(p) == t.Value {
			// No enabled tab matches the signal: this first enabled tab stays reachable.
			tabindex = is + " || !" + tabValues(p) + ".includes($" + p.Signal + ") ? 0 : -1"
		}
		a.set("data-attr:tabindex", tabindex).set("data-on:click", click).extra(rest)
	default:
		a.extra(t.Attrs)
	}

	var icons, label, end templ.Component
	tabIcon := func(name, state string) templ.Component {
		return el("span", newAttrs().set("class", "astryx-tab-icon").opt("data-icon-state", state),
			Icon(IconProps{Name: name, Size: "sm"}))
	}
	switch {
	case t.Icon != "" && t.SelectedIcon != "":
		icons = group(tabIcon(t.Icon, "idle"), tabIcon(t.SelectedIcon, "selected"))
	case t.Icon != "":
		icons = tabIcon(t.Icon, "")
	case t.SelectedIcon != "":
		icons = tabIcon(t.SelectedIcon, "selected")
	}
	if !t.IsLabelHidden && t.Label != "" {
		label = el("span", newAttrs().set("class", "astryx-tab-label"),
			el("span", newAttrs().set("class", "astryx-tab-label-text"), text(t.Label)),
			el("span", newAttrs().set("class", "astryx-tab-label-sizer").set("aria-hidden", "true"), text(t.Label)))
	}
	if t.EndContent != nil {
		end = el("span", newAttrs().set("class", "astryx-tab-end"), t.EndContent)
	}
	return el(tag, a,
		el("span", newAttrs().set("class", "astryx-tab-hover").set("aria-hidden", "true")),
		icons, label, end,
		el("span", newAttrs().set("class", "astryx-tab-indicator")))
}

// TabPanelProps configures TabPanel.
type TabPanelProps struct {
	ID string
	// TabID is the id of the tab that labels this panel.
	TabID string
	// Value is the tab value this panel belongs to.
	Value string
	// IsSelected shows the panel on first render.
	IsSelected bool
	// Signal is the TabList's signal; the panel shows while it equals Value.
	Signal string
	Class  string
	Attrs  []Attr
}

// TabPanel renders the content for one tab (role="tabpanel").
func TabPanel(p TabPanelProps) templ.Component {
	return withChildren(func(children templ.Component) templ.Component {
		a := newAttrs().set("class", cls("astryx-tab-panel", p.Class)).
			opt("id", p.ID).
			set("role", "tabpanel").
			opt("aria-labelledby", p.TabID).
			set("tabindex", "0").
			when(!p.IsSelected, "style", "display: none")
		if p.Signal != "" {
			a.set("data-show", "$"+p.Signal+" === "+jsString(p.Value))
		}
		a.extra(p.Attrs)
		return el("div", a, children)
	})
}

// fallbackStop is the value of the first enabled tab, the Tab stop when nothing is selected.
func fallbackStop(p TabListProps) string {
	for _, t := range p.Tabs {
		if !t.IsDisabled {
			return t.Value
		}
	}
	return ""
}

// tabValues is a JS array literal of the enabled tabs' values.
func tabValues(p TabListProps) string {
	var vals []string
	for _, t := range p.Tabs {
		if !t.IsDisabled {
			vals = append(vals, jsString(t.Value))
		}
	}
	return "[" + strings.Join(vals, ",") + "]"
}
