package canon

import "github.com/a-h/templ"

// Port of Astryx `Button` / `IconButton` (packages/core/src/Button), MIT, Meta Platforms, Inc.
// Differences: a disabled button uses the native disabled attribute; the async clickAction
// is left to Datastar (bind loading state with data-attr:aria-busy="$saving", or
// data-indicator). Not ported: tooltip (comes with overlays), ButtonGroup integration.

// ButtonProps configures Button and IconButton.
type ButtonProps struct {
	// Label is the visible text, and the accessible name when icon-only or loading.
	Label string
	// Variant: "secondary" (default), "primary", "ghost", "destructive".
	Variant string
	// Size: "sm", "md" (default), "lg".
	Size string
	// Elevation: "none" (default), "low", "med", "high".
	Elevation string
	// Type: "button" (default), "submit", "reset".
	Type       string
	IsDisabled bool
	// IsLoading shows the spinner and sets aria-busy. Datastar can toggle it client-side
	// through aria-busy, since the spinner is always rendered.
	IsLoading bool
	// IsInterruptible keeps the button clickable while loading; the spinner fades in late.
	IsInterruptible bool
	// Icon is a leading built-in glyph name.
	Icon       string
	IsIconOnly bool
	// Width is a CSS length, e.g. "100%".
	Width string
	// Content replaces the visible label; Label stays the accessible name.
	Content templ.Component
	// EndContent goes after the label, e.g. a Badge.
	EndContent templ.Component
	// Href renders an <a> (unless disabled).
	Href, Target, Rel string
	Class             string
	Attrs             []Attr
}

// busyGuard blocks activation (click, Enter, Space) while a non-interruptible button is busy,
// including when Datastar sets aria-busy client-side. Capture listeners on the target run
// before the caller's data-on:click.
const busyGuard = "el.ariaBusy === 'true' && !el.hasAttribute('data-interruptible') && (evt.preventDefault(), evt.stopImmediatePropagation())"

var buttonIconSize = map[string]string{"sm": "sm", "md": "sm", "lg": "md"}

// Button renders an Astryx button.
func Button(p ButtonProps) templ.Component {
	size := or(p.Size, "md")
	asLink := p.Href != "" && !p.IsDisabled
	a := newAttrs().set("class", cls("astryx-button", p.Class)).
		set("data-variant", or(p.Variant, "secondary")).
		set("data-size", size).
		set("data-elevation", or(p.Elevation, "none")).
		flag("data-icon-only", p.IsIconOnly).
		flag("data-disabled", p.IsDisabled).
		flag("data-interruptible", p.IsInterruptible).
		// Always named by Label: the visible content is hidden while busy, and Datastar may
		// set aria-busy after render.
		opt("aria-label", p.Label).
		when(p.IsLoading, "aria-busy", "true")
	tag := "button"
	if asLink {
		tag = "a"
		a.set("href", p.Href).opt("target", p.Target).opt("rel", p.Rel)
	} else {
		a.set("type", or(p.Type, "button")).flag("disabled", p.IsDisabled)
	}
	a.opt("style", decl("width", p.Width)).
		set("data-on:click__capture", busyGuard).
		extra(p.Attrs)

	var icon, label, end templ.Component
	if p.Icon != "" {
		iconSize := buttonIconSize[size]
		icon = el("span", newAttrs().set("class", "astryx-button-icon").set("data-icon-size", iconSize),
			Icon(IconProps{Name: p.Icon, Size: iconSize}))
	}
	if !p.IsIconOnly {
		content := p.Content
		if content == nil {
			content = text(p.Label)
		}
		label = el("span", newAttrs().set("class", "astryx-button-label"), content)
		if p.EndContent != nil {
			end = el("span", newAttrs().set("class", "astryx-button-end"), p.EndContent)
		}
	}
	return el(tag, a,
		el("span", newAttrs().set("class", "astryx-button-spinner").set("aria-hidden", "true"),
			Spinner(SpinnerProps{Size: "sm", Shade: "inherit"})),
		el("span", newAttrs().set("class", "astryx-button-content"), icon, label, end))
}

// IconButton is a Button that shows only its icon; Label is the accessible name.
func IconButton(p ButtonProps) templ.Component {
	p.IsIconOnly = true
	return Button(p)
}
