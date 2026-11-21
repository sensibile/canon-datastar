package canon

import "github.com/a-h/templ"

// VisuallyHiddenProps configures VisuallyHidden.
type VisuallyHiddenProps struct {
	// As is the element, "span" by default.
	As    string
	ID    string
	Class string
	Attrs []Attr
}

// VisuallyHidden renders content for assistive tech only.
// Port of Astryx `VisuallyHidden`, MIT, Meta Platforms, Inc.
func VisuallyHidden(p VisuallyHiddenProps) templ.Component {
	return withChildren(func(children templ.Component) templ.Component {
		return visuallyHidden(p, children)
	})
}

func visuallyHidden(p VisuallyHiddenProps, children templ.Component) templ.Component {
	return el(or(p.As, "span"), newAttrs().set("class", cls("astryx-visually-hidden", p.Class)).
		opt("id", p.ID).extra(p.Attrs), children)
}

// BadgeProps configures Badge.
type BadgeProps struct {
	// Variant: "neutral" (default), "info", "success", "warning", "error", or a color:
	// "blue", "cyan", "gray", "green", "orange", "pink", "purple", "red", "teal", "yellow".
	Variant string
	Label   string
	// Icon is a leading built-in glyph name.
	Icon  string
	Class string
	Attrs []Attr
}

// Badge renders a small status label.
// Port of Astryx `Badge` (packages/core/src/Badge), MIT, Meta Platforms, Inc.
func Badge(p BadgeProps) templ.Component {
	a := newAttrs().set("class", cls("astryx-badge", p.Class)).
		set("data-variant", or(p.Variant, "neutral")).
		flag("data-has-label", p.Label != "").
		opt("title", p.Label).
		extra(p.Attrs)
	var icon templ.Component
	if p.Icon != "" {
		icon = Icon(IconProps{Name: p.Icon, Size: "xsm"})
	}
	return el("span", a, icon,
		el("span", newAttrs().set("class", "astryx-badge-label").set("dir", "auto"), text(p.Label)))
}

// DividerProps configures Divider.
type DividerProps struct {
	// Orientation: "horizontal" (default) or "vertical".
	Orientation string
	// Label is text shown in the middle of the line.
	Label string
	// Variant: "subtle" (default) or "strong".
	Variant string
	// IsFullBleed extends the line past the parent container's padding.
	IsFullBleed bool
	// ID, when set with Label, names the separator by the label element (ID + "-label");
	// otherwise the label text becomes aria-label.
	ID    string
	Class string
	Attrs []Attr
}

// Divider renders a separator line.
// Port of Astryx `Divider` (packages/core/src/Divider), MIT, Meta Platforms, Inc.
func Divider(p DividerProps) templ.Component {
	orientation := or(p.Orientation, "horizontal")
	labelID := ""
	if p.Label != "" && p.ID != "" {
		labelID = p.ID + "-label"
	}
	a := newAttrs().set("class", cls("astryx-divider", p.Class)).
		opt("id", p.ID).
		set("role", "separator").
		set("aria-orientation", orientation).
		when(p.Label != "" && labelID == "", "aria-label", p.Label).
		opt("aria-labelledby", labelID).
		set("data-orientation", orientation).
		set("data-variant", or(p.Variant, "subtle")).
		flag("data-full-bleed", p.IsFullBleed).
		extra(p.Attrs)
	line := el("div", newAttrs().set("class", "astryx-divider-line"))
	if p.Label == "" {
		return el("div", a, line)
	}
	return el("div", a, line,
		el("div", newAttrs().set("class", "astryx-divider-label").opt("id", labelID), text(p.Label)),
		line)
}

// StackProps configures Stack, HStack and VStack.
type StackProps struct {
	// Direction: "vertical" (default) or "horizontal".
	Direction string
	// HAlign / VAlign align horizontally / vertically whatever the direction:
	// "start", "center", "end", "stretch", "between", "around", "evenly".
	HAlign, VAlign string
	// Justify (main axis) and Align (cross axis) apply when HAlign/VAlign are unset.
	Justify, Align                                      string
	Gap                                                 Space
	Padding                                             Space
	PaddingInline, PaddingInlineStart, PaddingInlineEnd Space
	PaddingBlock, PaddingBlockStart, PaddingBlockEnd    Space
	IsScrollable                                        bool
	// Width, Height, MaxWidth, MinHeight are CSS lengths, e.g. "320px".
	Width, Height, MaxWidth, MinHeight string
	// Wrap: "nowrap", "wrap", "wrap-reverse".
	Wrap string
	// As is the element, "div" by default.
	As    string
	Class string
	Attrs []Attr
}

var justifyMap = map[string]string{"start": "flex-start", "center": "center", "end": "flex-end", "between": "space-between", "around": "space-around", "evenly": "space-evenly"}
var alignMap = map[string]string{"start": "flex-start", "center": "center", "end": "flex-end", "stretch": "stretch"}

func firstSpace(vals ...Space) Space {
	for _, v := range vals {
		if v != "" {
			return v
		}
	}
	return ""
}

// Stack lays out its children in a flex row or column.
// Port of Astryx `Stack` (packages/core/src/Stack), MIT, Meta Platforms, Inc.
func Stack(p StackProps) templ.Component {
	return withChildren(func(children templ.Component) templ.Component {
		direction := or(p.Direction, "vertical")
		horizontal := direction == "horizontal"
		h, v := p.HAlign, p.VAlign
		if h == "" {
			h = map[bool]string{true: p.Justify, false: p.Align}[horizontal]
		}
		if v == "" {
			v = map[bool]string{true: p.Align, false: p.Justify}[horizontal]
		}
		main, cross := v, h
		if horizontal {
			main, cross = h, v
		}
		style := styles(
			decl("gap", p.Gap.css()),
			decl("justify-content", justifyMap[main]),
			decl("align-items", alignMap[cross]),
			decl("flex-wrap", p.Wrap),
			decl("padding-inline-start", firstSpace(p.PaddingInlineStart, p.PaddingInline, p.Padding).css()),
			decl("padding-inline-end", firstSpace(p.PaddingInlineEnd, p.PaddingInline, p.Padding).css()),
			decl("padding-block-start", firstSpace(p.PaddingBlockStart, p.PaddingBlock, p.Padding).css()),
			decl("padding-block-end", firstSpace(p.PaddingBlockEnd, p.PaddingBlock, p.Padding).css()),
			decl("width", p.Width),
			decl("height", p.Height),
			decl("max-width", p.MaxWidth),
			decl("min-height", p.MinHeight),
		)
		a := newAttrs().set("class", cls("astryx-stack", p.Class)).
			set("data-direction", direction).
			flag("data-scrollable", p.IsScrollable).
			opt("style", style).
			extra(p.Attrs)
		return el(or(p.As, "div"), a, children)
	})
}

// HStack is a horizontal Stack.
func HStack(p StackProps) templ.Component {
	p.Direction = "horizontal"
	return Stack(p)
}

// VStack is a vertical Stack.
func VStack(p StackProps) templ.Component {
	p.Direction = "vertical"
	return Stack(p)
}

// SpinnerProps configures Spinner.
type SpinnerProps struct {
	// Size: "sm", "md" (default), "lg", "xl".
	Size string
	// Shade: "default", "subtle", "onMedia", "inherit".
	Shade string
	// Label is visible text under the spinner; it also names the progressbar.
	Label string
	// AriaLabel names the progressbar when there is no Label ("Loading" by default).
	AriaLabel string
	Class     string
	Attrs     []Attr
}

type spinnerGeom struct{ frame, r, border, arc, gap string }

// Precomputed so Go and Rust print identical numbers: frame = d + 2b, r = d/2,
// arc = πd·0.375, gap = πd − arc.
var spinnerSizes = map[string]spinnerGeom{
	"sm": {"14", "5", "2", "11.780972450961723", "19.634954084936208"},
	"md": {"20", "7", "3", "16.493361431346415", "27.48893571891069"},
	"lg": {"24", "9", "3", "21.205750411731103", "35.34291735288517"},
	"xl": {"36", "14", "4", "32.98672286269283", "54.97787143782138"},
}

// Spinner renders an indeterminate progress ring.
// Port of Astryx `Spinner` (packages/core/src/Spinner), MIT, Meta Platforms, Inc.
// Not ported: theme spinner indicators, rotation phase sync across spinners.
func Spinner(p SpinnerProps) templ.Component {
	size := or(p.Size, "md")
	g, ok := spinnerSizes[size]
	if !ok {
		size, g = "md", spinnerSizes["md"]
	}
	tag := "span"
	if p.Label != "" {
		tag = "div"
	}
	a := newAttrs().set("class", cls("astryx-spinner", p.Class)).
		set("data-size", size).
		set("data-shade", or(p.Shade, "default")).
		flag("data-has-label", p.Label != "").
		extra(p.Attrs)
	name := or(p.AriaLabel, or(p.Label, "Loading"))
	ring := el("span", newAttrs().set("class", "astryx-spinner-ring").set("role", "progressbar").
		set("aria-label", name).set("style", "width: "+g.frame+"px; height: "+g.frame+"px"),
		el("svg", newAttrs().set("width", g.frame).set("height", g.frame).set("aria-hidden", "true"),
			el("circle", newAttrs().set("class", "astryx-spinner-track").set("cx", "50%").set("cy", "50%").
				set("r", g.r).set("stroke-width", g.border))),
		el("span", newAttrs().set("class", "astryx-spinner-rotor").set("aria-hidden", "true"),
			el("svg", newAttrs().set("width", g.frame).set("height", g.frame),
				el("circle", newAttrs().set("class", "astryx-spinner-arc").set("cx", "50%").set("cy", "50%").
					set("r", g.r).set("stroke-width", g.border).set("stroke-dasharray", g.arc+" "+g.gap)))))
	if p.Label == "" {
		return el(tag, a, ring)
	}
	return el(tag, a, ring, textEl(TextProps{Type: "body", Weight: "bold"}, text(p.Label)))
}
