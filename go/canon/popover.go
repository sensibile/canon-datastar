package canon

import "github.com/a-h/templ"

// Port of Astryx `Popover` (packages/core/src/Popover), MIT, Meta Platforms, Inc.
// A native popover anchored to the first button inside the children. static/canon.js points
// that button at the surface (popovertarget), so clicks toggle it natively; the browser closes
// it on an outside press or Escape and returns focus to the trigger.
// Not ported: bottom-sheet presentation on touch, render-prop trigger.

// PopoverProps configures Popover.
type PopoverProps struct {
	// ID of the surface. Required when a page has more than one popover; default "popover".
	ID      string
	Content templ.Component
	// Placement: "below" (default), "above", "start", "end".
	Placement string
	// Alignment: "start" (default), "center", "end".
	Alignment string
	// Width is a CSS length.
	Width string
	// Padding is a spacing step, default "3".
	Padding Space
	// Label is the accessible name of the dialog surface.
	Label string
	// Role: "dialog" (default) or "none".
	Role string
	// IsModal traps focus inside while open, and moves focus in on open.
	IsModal bool
	// HasAutoFocus moves focus into the surface on open (always on for modal popovers).
	HasAutoFocus bool
	// HasCloseButton adds a close button that is visually hidden until focused.
	HasCloseButton   bool
	CloseButtonLabel string
	// NoLightDismiss keeps it open on an outside press; NoEscapeDismiss on Escape.
	NoLightDismiss  bool
	NoEscapeDismiss bool
	// Signal names a Datastar signal that mirrors the open state both ways.
	Signal string
	Class  string
	Attrs  []Attr
}

// Popover wraps its children (the trigger) and renders the surface after them.
func Popover(p PopoverProps) templ.Component {
	return withChildren(func(children templ.Component) templ.Component {
		id := or(p.ID, "popover")
		mode := "auto"
		if p.NoLightDismiss || p.NoEscapeDismiss {
			mode = "manual"
		}
		isDialog := p.Role != "none"
		a := newAttrs().set("popover", mode).
			set("id", id).
			set("class", cls("astryx-layer astryx-popover", p.Class)).
			when(isDialog, "role", "dialog").
			when(isDialog && p.Label != "", "aria-label", p.Label).
			when(isDialog && p.IsModal, "aria-modal", "true").
			set("data-placement", or(p.Placement, "below")).
			set("data-alignment", or(p.Alignment, "start")).
			flag("data-modal", p.IsModal).
			flag("data-autofocus-open", p.IsModal || p.HasAutoFocus).
			flag("data-no-light-dismiss", p.NoLightDismiss).
			flag("data-no-escape-dismiss", p.NoEscapeDismiss).
			set("style", styles(decl("width", p.Width), decl("--_popover-padding", Space(or(string(p.Padding), "3")).css())))
		if p.Signal != "" {
			a.set("data-effect", "canon.popover(el, $"+p.Signal+")").
				set("data-on:toggle", "$"+p.Signal+" = evt.newState === 'open'")
		}
		a.extra(p.Attrs)
		var closeBtn templ.Component
		if p.HasCloseButton {
			closeBtn = el("button", newAttrs().set("type", "button").set("class", "astryx-popover-close").
				flag("data-popover-close", true), text(or(p.CloseButtonLabel, "Close")))
		}
		return group(
			el("span", newAttrs().set("class", "astryx-popover-anchor").set("data-popover", id), children),
			el("div", a, el("div", newAttrs().set("class", "astryx-popover-content"), p.Content, closeBtn)),
		)
	})
}
