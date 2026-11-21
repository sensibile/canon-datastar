package canon

import (
	"strconv"

	"github.com/a-h/templ"
)

// Port of Astryx `Card`, `Section`, `Center`, `Grid`, `GridSpan` and `AspectRatio`
// (packages/core/src/Card, Section, Center, Grid, AspectRatio), MIT, Meta Platforms, Inc.
// Sizes (Width, Height, MaxWidth, MinHeight) are CSS lengths, e.g. "320px" or "100%".
// Not ported: Card's layout container scope (header/footer bleed).

// CardProps configures Card.
type CardProps struct {
	// Variant: "default", "transparent", "muted", or a color: "blue", "cyan", "gray", "green",
	// "orange", "pink", "purple", "red", "teal", "yellow".
	Variant string
	// Elevation: "none" (default), "low", "med", "high".
	Elevation                          string
	Width, Height, MaxWidth, MinHeight string
	// Padding overrides the theme's card padding.
	Padding Space
	Class   string
	Attrs   []Attr
}

func sizeStyles(width, height, maxWidth, minHeight string) []string {
	return []string{decl("width", width), decl("height", height), decl("max-width", maxWidth), decl("min-height", minHeight)}
}

// Card renders a surface around its children.
func Card(p CardProps) templ.Component {
	return withChildren(func(children templ.Component) templ.Component {
		style := styles(append(sizeStyles(p.Width, p.Height, p.MaxWidth, p.MinHeight),
			decl("--_card-padding", p.Padding.css()))...)
		a := newAttrs().set("class", cls("astryx-card", p.Class)).
			set("data-variant", or(p.Variant, "default")).
			set("data-elevation", or(p.Elevation, "none")).
			flag("data-scrollable", p.Height != "" && p.Height != "auto").
			opt("style", style).
			extra(p.Attrs)
		return el("div", a, children)
	})
}

// SectionProps configures Section.
type SectionProps struct {
	// Variant: "section" (default), "transparent", "muted".
	Variant                            string
	Width, Height, MaxWidth, MinHeight string
	// Dividers draws borders on these sides: "top", "bottom", "start", "end".
	Dividers                                            []string
	Padding                                             Space
	PaddingInline, PaddingInlineStart, PaddingInlineEnd Space
	PaddingBlock, PaddingBlockStart, PaddingBlockEnd    Space
	Class                                               string
	Attrs                                               []Attr
}

// Section renders a full-bleed band that cancels its container's padding.
func Section(p SectionProps) templ.Component {
	return withChildren(func(children templ.Component) templ.Component {
		has := func(side string) bool {
			for _, d := range p.Dividers {
				if d == side {
					return true
				}
			}
			return false
		}
		outer := newAttrs().set("class", cls("astryx-section-outer", p.Class)).
			opt("style", styles(sizeStyles(p.Width, p.Height, p.MaxWidth, p.MinHeight)...)).
			extra(p.Attrs)
		inner := newAttrs().set("class", "astryx-section").
			set("data-variant", or(p.Variant, "section")).
			flag("data-divider-top", has("top")).
			flag("data-divider-bottom", has("bottom")).
			flag("data-divider-start", has("start")).
			flag("data-divider-end", has("end")).
			opt("style", styles(
				decl("--_section-padding", p.Padding.css()),
				decl("--_section-pis", firstSpace(p.PaddingInlineStart, p.PaddingInline).css()),
				decl("--_section-pie", firstSpace(p.PaddingInlineEnd, p.PaddingInline).css()),
				decl("--_section-pbs", firstSpace(p.PaddingBlockStart, p.PaddingBlock).css()),
				decl("--_section-pbe", firstSpace(p.PaddingBlockEnd, p.PaddingBlock).css()),
			))
		return el("div", outer, el("div", inner, children))
	})
}

// CenterProps configures Center.
type CenterProps struct {
	// Axis: "both" (default), "horizontal", "vertical".
	Axis                                                string
	Width, Height, MaxWidth, MinHeight                  string
	Padding                                             Space
	PaddingInline, PaddingInlineStart, PaddingInlineEnd Space
	PaddingBlock, PaddingBlockStart, PaddingBlockEnd    Space
	IsInline                                            bool
	Class                                               string
	Attrs                                               []Attr
}

// Center centers its children.
func Center(p CenterProps) templ.Component {
	return withChildren(func(children templ.Component) templ.Component {
		style := styles(append(sizeStyles(p.Width, p.Height, p.MaxWidth, p.MinHeight),
			decl("padding-inline-start", firstSpace(p.PaddingInlineStart, p.PaddingInline, p.Padding).css()),
			decl("padding-inline-end", firstSpace(p.PaddingInlineEnd, p.PaddingInline, p.Padding).css()),
			decl("padding-block-start", firstSpace(p.PaddingBlockStart, p.PaddingBlock, p.Padding).css()),
			decl("padding-block-end", firstSpace(p.PaddingBlockEnd, p.PaddingBlock, p.Padding).css()),
		)...)
		a := newAttrs().set("class", cls("astryx-center", p.Class)).
			flag("data-inline", p.IsInline).
			set("data-axis", or(p.Axis, "both")).
			opt("style", style).
			extra(p.Attrs)
		return el("div", a, children)
	})
}

// GridAuto makes a responsive grid: as many columns of at least MinWidth px as fit.
type GridAuto struct {
	MinWidth int
	// Max caps the number of columns (0 = no cap).
	Max int
	// Repeat: "fill" (default, keeps empty tracks) or "fit" (collapses them).
	Repeat string
}

// GridProps configures Grid.
type GridProps struct {
	// Columns is a fixed column count (0 = one column); Auto, when set, wins.
	Columns                            int
	Auto                               *GridAuto
	Width, Height, MaxWidth, MinHeight string
	Gap, RowGap, ColumnGap             Space
	// RowHeight is a fixed row height in px (0 = auto).
	RowHeight int
	// Align and Justify: "start", "center", "end", "stretch".
	Align, Justify string
	Class          string
	Attrs          []Attr
}

func gridTemplate(p GridProps) string {
	if p.Auto != nil {
		mode := "auto-fill"
		if p.Auto.Repeat == "fit" {
			mode = "auto-fit"
		}
		min := strconv.Itoa(p.Auto.MinWidth) + "px"
		if p.Auto.Max > 0 {
			n := p.Auto.Max
			per := "calc(100% / " + strconv.Itoa(n) + ")"
			if g := firstSpace(p.ColumnGap, p.Gap); g != "" {
				per = "calc((100% - " + strconv.Itoa(n-1) + " * " + g.css() + ") / " + strconv.Itoa(n) + ")"
			}
			return "repeat(" + mode + ", minmax(min(100%, max(" + min + ", " + per + ")), 1fr))"
		}
		return "repeat(" + mode + ", minmax(min(100%, " + min + "), 1fr))"
	}
	if p.Columns > 0 {
		return "repeat(" + strconv.Itoa(p.Columns) + ", 1fr)"
	}
	return "1fr"
}

// Grid lays out its children in a CSS grid.
func Grid(p GridProps) templ.Component {
	return withChildren(func(children templ.Component) templ.Component {
		rowHeight := ""
		if p.RowHeight > 0 {
			rowHeight = strconv.Itoa(p.RowHeight) + "px"
		}
		style := styles(append([]string{
			decl("grid-template-columns", gridTemplate(p)),
			decl("grid-auto-rows", rowHeight),
			decl("gap", p.Gap.css()),
			decl("row-gap", p.RowGap.css()),
			decl("column-gap", p.ColumnGap.css()),
			decl("align-items", p.Align),
			decl("justify-items", p.Justify),
		}, sizeStyles(p.Width, p.Height, p.MaxWidth, p.MinHeight)...)...)
		a := newAttrs().set("class", cls("astryx-grid", p.Class)).
			when(p.Auto == nil && p.Columns > 0, "data-columns", strconv.Itoa(p.Columns)).
			opt("data-gap", string(p.Gap)).
			opt("data-align", p.Align).
			opt("data-justify", p.Justify).
			set("style", style).
			extra(p.Attrs)
		return el("div", a, children)
	})
}

// GridSpanProps configures GridSpan.
type GridSpanProps struct {
	// Columns to span (0 = one); IsFull spans the whole row.
	Columns int
	IsFull  bool
	// Rows to span (0 = one).
	Rows  int
	Class string
	Attrs []Attr
}

// GridSpan makes its children span several grid tracks.
func GridSpan(p GridSpanProps) templ.Component {
	return withChildren(func(children templ.Component) templ.Component {
		column, row := "", ""
		if p.IsFull {
			column = "1 / -1"
		} else if p.Columns > 0 {
			column = "span " + strconv.Itoa(p.Columns)
		}
		if p.Rows > 0 {
			row = "span " + strconv.Itoa(p.Rows)
		}
		a := newAttrs().set("class", cls("astryx-grid-span", p.Class)).
			opt("style", styles(decl("grid-column", column), decl("grid-row", row))).
			extra(p.Attrs)
		return el("div", a, children)
	})
}

// AspectRatioProps configures AspectRatio.
type AspectRatioProps struct {
	// Ratio is width divided by height, e.g. 16.0 / 9.
	Ratio float64
	// Shape: "rectangle" (default) or "ellipse".
	Shape string
	// Fit: how an img/video child fills the box: "cover", "contain", or "center" (any child).
	Fit   string
	Class string
	Attrs []Attr
}

// AspectRatio keeps its child at a fixed width-to-height ratio.
func AspectRatio(p AspectRatioProps) templ.Component {
	return withChildren(func(children templ.Component) templ.Component {
		ratio := p.Ratio
		if ratio <= 0 {
			ratio = 1
		}
		a := newAttrs().set("class", cls("astryx-aspect-ratio", p.Class)).
			set("data-shape", or(p.Shape, "rectangle")).
			set("style", "aspect-ratio: "+strconv.FormatFloat(ratio, 'f', -1, 64)).
			extra(p.Attrs)
		return el("div", a, el("div", newAttrs().set("class", "astryx-aspect-ratio-child").opt("data-fit", p.Fit), children))
	})
}
