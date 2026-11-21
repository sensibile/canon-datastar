package canon

import (
	"strconv"

	"github.com/a-h/templ"
)

// Port of Astryx `Text` and `Heading` (packages/core/src/Text, Heading), MIT, Meta Platforms, Inc.
// Not ported: the truncation tooltip.

// TextProps configures Text.
type TextProps struct {
	// Type: "body" (default), "large", "label", "supporting", "code", "display-1..3", "inherit".
	Type string
	// Size: "4xs" … "4xl"; overrides the type's font size.
	Size string
	// Color: "primary", "secondary", "disabled", "placeholder", "accent", "inherit".
	// Defaults by type (supporting is secondary, inherit is inherit, others primary).
	Color string
	// Weight: "normal", "medium", "semibold", "bold".
	Weight string
	// Display: "inline" (default) or "block".
	Display string
	// MaxLines clamps to this many lines with an ellipsis. 0 = no clamp.
	MaxLines int
	// WordBreak: "break-word" or "break-all" (applies with MaxLines).
	WordBreak string
	// TextWrap: "wrap", "nowrap", "balance", "pretty".
	TextWrap string
	// Justify: "start" (default), "center", "end".
	Justify           string
	HasCapsize        bool
	HasStrikethrough  bool
	HasTabularNumbers bool
	// As is the element: "span" (default), "p", "div", "label", "h1", "h2", "h3".
	As    string
	Class string
	Attrs []Attr
}

// textLayout writes the attributes Text and Heading share, in the shared order.
func textLayout(a *attrs, display string, maxLines int, wordBreak, justify, textWrap string, capsize, strike bool) {
	if maxLines == 0 {
		d := or(display, "inline")
		if capsize {
			d = "block"
		}
		a.set("data-display", d)
	} else {
		trunc := "multi"
		if maxLines == 1 {
			trunc = "single"
		}
		wb := wordBreak
		if wb == "" {
			wb = "break-word"
			if maxLines == 1 {
				wb = "break-all"
			}
		}
		a.set("data-truncate", trunc).set("data-word-break", wb)
	}
	if justify != "start" {
		a.opt("data-justify", justify)
	}
	a.opt("data-wrap", textWrap).flag("data-capsize", capsize).flag("data-strikethrough", strike)
}

func clampStyle(maxLines int) string {
	if maxLines > 1 {
		return "-webkit-line-clamp: " + strconv.Itoa(maxLines)
	}
	return ""
}

func defaultTextColor(t string) string {
	switch t {
	case "supporting":
		return "secondary"
	case "inherit":
		return "inherit"
	}
	return "primary"
}

// Text renders styled text around its children.
func Text(p TextProps) templ.Component {
	return withChildren(func(children templ.Component) templ.Component {
		return textEl(p, children)
	})
}

func textEl(p TextProps, children templ.Component) templ.Component {
	typ := or(p.Type, "body")
	a := newAttrs().set("class", cls("astryx-text", p.Class)).
		set("data-type", typ).
		opt("data-size", p.Size).
		set("data-color", or(p.Color, defaultTextColor(typ))).
		opt("data-weight", p.Weight)
	textLayout(a, p.Display, p.MaxLines, p.WordBreak, p.Justify, p.TextWrap, p.HasCapsize, p.HasStrikethrough)
	a.flag("data-tabular", p.HasTabularNumbers).
		opt("style", clampStyle(p.MaxLines)).
		extra(p.Attrs)
	return el(or(p.As, "span"), a, children)
}

// HeadingProps configures Heading.
type HeadingProps struct {
	// Level 1–6 picks the h1–h6 tag and the type scale (invalid values become 2).
	Level int
	// Type uses a display scale instead: "display-1", "display-2", "display-3".
	Type   string
	Weight string
	// AccessibilityLevel sets aria-level when the outline level differs from Level.
	AccessibilityLevel int
	// Color defaults to "primary".
	Color string
	// Display defaults to "block".
	Display          string
	MaxLines         int
	WordBreak        string
	TextWrap         string
	Justify          string
	HasCapsize       bool
	HasStrikethrough bool
	Class            string
	Attrs            []Attr
}

// Heading renders an h1–h6 around its children.
func Heading(p HeadingProps) templ.Component {
	return withChildren(func(children templ.Component) templ.Component {
		level := p.Level
		if level < 1 || level > 6 {
			level = 2
		}
		lv := strconv.Itoa(level)
		a := newAttrs().set("class", cls("astryx-heading", p.Class))
		if p.AccessibilityLevel != 0 && p.AccessibilityLevel != level {
			a.set("aria-level", strconv.Itoa(p.AccessibilityLevel))
		}
		a.set("data-level", lv).
			opt("data-type", p.Type).
			set("data-color", or(p.Color, "primary")).
			opt("data-weight", p.Weight)
		textLayout(a, or(p.Display, "block"), p.MaxLines, p.WordBreak, p.Justify, p.TextWrap, p.HasCapsize, p.HasStrikethrough)
		a.opt("style", clampStyle(p.MaxLines)).extra(p.Attrs)
		return el("h"+lv, a, children)
	})
}
