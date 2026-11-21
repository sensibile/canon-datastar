package canon

import (
	"strconv"
	"strings"

	"github.com/a-h/templ"
)

// Port of Astryx `Blockquote`, `Code`, `Kbd`, `EmptyState`, `Skeleton` and `StatusDot`
// (packages/core/src/…), MIT, Meta Platforms, Inc.

// BlockquoteProps configures Blockquote.
type BlockquoteProps struct {
	// Cite is the attribution shown under the quote; CiteMarkup replaces it with markup.
	Cite       string
	CiteMarkup templ.Component
	Class      string
	Attrs      []Attr
}

// Blockquote renders a quotation around its children.
func Blockquote(p BlockquoteProps) templ.Component {
	return withChildren(func(children templ.Component) templ.Component {
		var cite templ.Component
		if p.CiteMarkup != nil {
			cite = p.CiteMarkup
		} else if p.Cite != "" {
			cite = text(p.Cite)
		}
		if cite != nil {
			cite = el("cite", newAttrs().set("class", "astryx-blockquote-cite"), cite)
		}
		return el("blockquote", newAttrs().set("class", cls("astryx-blockquote", p.Class)).extra(p.Attrs), children, cite)
	})
}

// CodeProps configures Code.
type CodeProps struct {
	// Color: "primary" (default), "secondary", "inherit".
	Color string
	// Size: "" or "inherit" (takes the surrounding font size and line height).
	Size  string
	Class string
	Attrs []Attr
}

// Code renders inline code around its children.
func Code(p CodeProps) templ.Component {
	return withChildren(func(children templ.Component) templ.Component {
		a := newAttrs().set("class", cls("astryx-code", p.Class)).
			set("dir", "ltr").
			set("data-color", or(p.Color, "primary")).
			opt("data-size", p.Size).
			extra(p.Attrs)
		return el("code", a, children)
	})
}

// KbdProps configures Kbd.
type KbdProps struct {
	// Keys is a "+"-joined shortcut, e.g. "mod+shift+p". "mod" is ⌘ on Apple platforms and
	// Ctrl elsewhere: the server renders the non-Apple form and static/canon.js switches it.
	Keys  string
	Class string
	Attrs []Attr
}

var kbdDisplay = map[string]string{"enter": "↵", "backspace": "⌫", "tab": "⇥", "up": "↑", "down": "↓", "left": "←", "right": "→", "plus": "+"}
var kbdShort = map[string]string{"mod": "Ctrl", "ctrl": "Ctrl", "alt": "Alt", "shift": "Shift", "meta": "Win"}
var kbdLabel = map[string]string{"mod": "Control", "ctrl": "Control", "alt": "Alt", "shift": "Shift", "enter": "Enter", "backspace": "Backspace",
	"escape": "Escape", "tab": "Tab", "up": "Up arrow", "down": "Down arrow", "left": "Left arrow", "right": "Right arrow", "plus": "Plus"}

func kbdKeys(keys string) []string {
	parts := strings.Split(keys, "+")
	for i, k := range parts {
		k = strings.ToLower(strings.TrimSpace(k))
		switch k {
		case "esc":
			k = "escape"
		case "return":
			k = "enter"
		}
		parts[i] = k
	}
	return parts
}

// Kbd renders a keyboard shortcut.
func Kbd(p KbdProps) templ.Component {
	keys := kbdKeys(p.Keys)
	labels := make([]string, len(keys))
	var caps []templ.Component
	for i, k := range keys {
		labels[i] = or(kbdLabel[k], strings.ToUpper(k))
		shown := kbdShort[k]
		if shown == "" {
			if k == "escape" {
				shown = "Esc"
			} else {
				shown = or(kbdDisplay[k], strings.ToUpper(k))
			}
		}
		caps = append(caps, el("kbd", newAttrs().set("aria-hidden", "true").set("class", "astryx-kbd-key"), text(shown)))
	}
	a := newAttrs().set("role", "img").
		set("aria-label", strings.Join(labels, " + ")).
		set("class", cls("astryx-kbd", p.Class)).
		set("data-keys", strings.Join(keys, "+")).
		extra(p.Attrs)
	return el("span", a, caps...)
}

// EmptyStateProps configures EmptyState.
type EmptyStateProps struct {
	Title       string
	Description string
	// Icon goes above the title; Actions below the text.
	Icon, Actions templ.Component
	// HeadingLevel 1–6 (default 3).
	HeadingLevel int
	IsCompact    bool
	Class        string
	Attrs        []Attr
}

// EmptyState renders a placeholder for an empty view.
func EmptyState(p EmptyStateProps) templ.Component {
	variant := ""
	if p.IsCompact {
		variant = "compact"
	}
	level := p.HeadingLevel
	if level < 1 || level > 6 {
		level = 3
	}
	var icon, desc, actions templ.Component
	if p.Icon != nil {
		icon = el("div", newAttrs().set("aria-hidden", "true"), p.Icon)
	}
	if p.Description != "" {
		desc = el("div", newAttrs().set("class", "astryx-empty-state-description").opt("data-variant", variant), text(p.Description))
	}
	if p.Actions != nil {
		actions = el("div", newAttrs().set("class", "astryx-empty-state-actions"), p.Actions)
	}
	a := newAttrs().set("class", cls("astryx-empty-state", p.Class)).opt("data-variant", variant).extra(p.Attrs)
	// Changes to the text are announced politely, like Astryx AnnounceRenderedText.
	return el("div", a, icon,
		el("div", newAttrs().set("class", "astryx-empty-state-text").set("aria-live", "polite"),
			el("h"+strconv.Itoa(level), newAttrs().set("class", "astryx-empty-state-title").opt("data-variant", variant), text(p.Title)),
			desc),
		actions)
}

// SkeletonProps configures Skeleton.
type SkeletonProps struct {
	// Width and Height are CSS lengths ("100%" by default).
	Width, Height string
	// Radius: "none", "0", "1", "2", "3" (default), "4", "rounded".
	Radius string
	// Index staggers the fade-in start in a list of skeletons.
	Index int
	Class string
	Attrs []Attr
}

// Skeleton renders a loading placeholder.
func Skeleton(p SkeletonProps) templ.Component {
	a := newAttrs().set("aria-hidden", "true").
		set("class", cls("astryx-skeleton", p.Class)).
		set("data-radius", or(p.Radius, "3")).
		set("style", styles(decl("width", or(p.Width, "100%")), decl("height", or(p.Height, "100%")),
			decl("animation-delay", strconv.Itoa(1000+100*p.Index)+"ms"))).
		extra(p.Attrs)
	return el("div", a)
}

// StatusDotProps configures StatusDot.
type StatusDotProps struct {
	// Variant: "success", "warning", "error", "accent", "neutral" (default).
	Variant string
	// Label is the accessible name.
	Label     string
	IsPulsing bool
	// Tooltip shows on hover and focus (the dot becomes focusable).
	Tooltip   string
	TooltipID string
	// Icon is a built-in glyph name drawn inside the dot.
	Icon  string
	Class string
	Attrs []Attr
}

// StatusDot renders a small colored status indicator.
func StatusDot(p StatusDotProps) templ.Component {
	a := newAttrs().when(p.Tooltip != "", "tabindex", "0").
		set("role", "img").
		set("aria-label", p.Label).
		set("class", cls("astryx-status-dot", p.Class)).
		set("data-variant", or(p.Variant, "neutral")).
		flag("data-pulsing", p.IsPulsing).
		extra(p.Attrs)
	var icon templ.Component
	if p.Icon != "" {
		icon = el("span", newAttrs().set("class", "astryx-status-dot-icon").set("aria-hidden", "true"),
			Icon(IconProps{Name: p.Icon, Size: "xsm"}))
	}
	dot := el("span", a, icon)
	if p.Tooltip == "" {
		return dot
	}
	return tooltip(TooltipProps{ID: p.TooltipID, Content: p.Tooltip}, dot)
}
