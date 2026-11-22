package canon

import (
	"strconv"

	"github.com/a-h/templ"
)

// Port of Astryx `MetadataList` and `MetadataListItem` (packages/core/src/MetadataList), MIT,
// Meta Platforms, Inc. Items past MaxNumOfItems render hidden behind a "Show more" button that
// static/canon.js toggles.

// MetadataListItem is one label/value pair.
type MetadataListItem struct {
	Label string
	Icon  templ.Component
	// Value is the value markup; Text is used when Value is nil.
	Value templ.Component
	Text  string
	Class string
	Attrs []Attr
}

// MetadataListProps configures MetadataList.
type MetadataListProps struct {
	// ID of the list (for the toggle's aria-controls). Defaults to "metadata-" + a slug of Title.
	ID string
	// Columns: "single" (default), "multi", or a count such as "3".
	Columns string
	// LabelPosition: "start" (default for single column) or "top" (default for multi).
	LabelPosition string
	// LabelWidth is a CSS length for inline labels in a single column.
	LabelWidth    string
	MaxNumOfItems int
	// Orientation: "vertical" (default) or "horizontal".
	Orientation string
	Title       string
	TitleMarkup templ.Component
	Items       []MetadataListItem
	Class       string
	Attrs       []Attr
}

// MetadataList renders a description list.
func MetadataList(p MetadataListProps) templ.Component {
	columns := or(p.Columns, "single")
	n, _ := strconv.Atoi(columns)
	isMulti := columns == "multi" || n > 1
	isSingle := columns == "single" || n == 1
	orientation := or(p.Orientation, "vertical")
	horizontal := orientation == "horizontal"
	position := p.LabelPosition
	if position == "" {
		position = map[bool]string{true: "top", false: "start"}[isMulti]
	}
	stacked := position == "top" || horizontal

	layout := "horizontal"
	if !horizontal {
		switch {
		case position == "top" && isSingle:
			layout = "stacked-single"
		case position == "top":
			layout = "stacked-multi"
		case isSingle:
			layout = "single"
		default:
			layout = "multi"
		}
	}
	template := ""
	if !horizontal {
		if n > 1 {
			if position == "top" {
				template = "repeat(" + columns + ", 1fr)"
			} else {
				template = "repeat(" + columns + ", auto minmax(0, 1fr))"
			}
		} else if position != "top" && p.LabelWidth != "" {
			template = p.LabelWidth + " minmax(0, 1fr)"
		}
	}
	max := p.MaxNumOfItems
	if horizontal {
		max = 0
	}
	exceeds := max > 0 && len(p.Items) > max
	id := or(p.ID, "metadata-"+or(slug(p.Title), "list"))

	var items []templ.Component
	for i, it := range p.Items {
		hidden := exceeds && i >= max
		var icon templ.Component
		if it.Icon != nil {
			icon = el("span", newAttrs().set("class", "astryx-metadata-list-icon"), it.Icon)
		}
		value := it.Value
		if value == nil {
			value = text(it.Text)
		}
		if stacked {
			a := newAttrs().set("class", cls("astryx-metadata-list-item", it.Class)).
				set("data-metadata-item", "").set("data-stacked", "").flag("hidden", hidden).extra(it.Attrs)
			items = append(items, el("div", a,
				el("dt", newAttrs().set("class", "astryx-metadata-list-label"), icon, text(it.Label)),
				el("dd", newAttrs().set("class", "astryx-metadata-list-value"), value)))
		} else {
			a := newAttrs().set("class", cls("astryx-metadata-list-item astryx-metadata-list-label", it.Class)).
				set("data-metadata-item", "").set("data-inline", "").flag("hidden", hidden).extra(it.Attrs)
			items = append(items, el("dt", a, icon, text(it.Label)),
				el("dd", newAttrs().set("class", "astryx-metadata-list-value").set("data-inline", "").flag("hidden", hidden), value))
		}
	}
	var title, toggle templ.Component
	if p.TitleMarkup != nil || p.Title != "" {
		t := p.TitleMarkup
		if t == nil {
			t = text(p.Title)
		}
		title = el("div", newAttrs().set("class", "astryx-metadata-list-title"), t)
	}
	if exceeds {
		toggle = el("button", newAttrs().set("type", "button").set("class", "astryx-metadata-list-toggle").
			set("aria-controls", id).set("aria-expanded", "false"), text("Show more"))
	}
	a := newAttrs().set("class", cls("astryx-metadata-list", p.Class)).
		set("data-columns", columns).
		set("data-orientation", orientation).
		extra(p.Attrs)
	return el("div", a, title,
		el("dl", newAttrs().set("id", id).set("class", "astryx-metadata-list-list").set("data-layout", layout).
			when(exceeds, "data-max", strconv.Itoa(max)).opt("style", decl("grid-template-columns", template)), items...),
		toggle)
}
