package canon

import (
	"strconv"

	"github.com/a-h/templ"
)

// Port of Astryx `Toast` and `ToastViewport` (packages/core/src/Toast), MIT, Meta Platforms, Inc.
// Render one ToastViewport near the end of <body>. Show a toast from the server by patching a
// Toast into it over SSE (selector "#astryx-toasts", mode append), or from the browser with
// canon.toast({body: "Saved"}). static/canon.js runs the auto-hide timer (paused while hovered
// or focused), dismisses the oldest beyond MaxVisible, and keeps the viewport above open dialogs.
// Not ported: swipe to dismiss, renderContent.

// ToastProps configures Toast.
type ToastProps struct {
	// ID goes on the toast row; patch the same id again to replace a visible toast.
	ID string
	// Type: "info" (default) or "error".
	Type string
	Body string
	// BodyMarkup replaces Body with rich content.
	BodyMarkup templ.Component
	EndContent templ.Component
	// IsSticky turns auto-hide off. Error toasts are sticky unless AutoHideDuration is set.
	IsSticky bool
	// AutoHideDuration is in ms, default 5000.
	AutoHideDuration int
	// DismissLabel is the close button's accessible name, default "Dismiss notification".
	DismissLabel string
	Class        string
	Attrs        []Attr
}

// Toast renders one notification row for a ToastViewport.
func Toast(p ToastProps) templ.Component {
	typ := or(p.Type, "info")
	isError := typ == "error"
	autoHide := !p.IsSticky && (!isError || p.AutoHideDuration > 0)
	duration := p.AutoHideDuration
	if duration <= 0 {
		duration = 5000
	}
	role, live := "status", "polite"
	if isError {
		role, live = "alert", "assertive"
	}
	a := newAttrs().set("class", cls("astryx-toast", p.Class)).
		set("data-type", typ).
		set("role", role).
		set("aria-live", live).
		when(autoHide, "data-auto-hide", strconv.Itoa(duration)).
		extra(p.Attrs)
	body := p.BodyMarkup
	if body == nil {
		body = text(p.Body)
	}
	closeBtn := Button(ButtonProps{Variant: "ghost", Size: "sm", Icon: "close", IsIconOnly: true,
		Label: or(p.DismissLabel, "Dismiss notification"), Class: "astryx-toast-close",
		Attrs: []Attr{{"data-toast-dismiss", ""}}})
	return el("div", newAttrs().set("class", "astryx-toast-row").opt("id", p.ID),
		el("div", newAttrs().set("class", "astryx-toast-row-inner"),
			el("div", a,
				el("div", newAttrs().set("class", "astryx-toast-inner"),
					el("div", newAttrs().set("class", "astryx-toast-content"), body),
					el("div", newAttrs().set("class", "astryx-toast-end"), p.EndContent, closeBtn)))))
}

// ToastViewportProps configures ToastViewport.
type ToastViewportProps struct {
	// ID defaults to "astryx-toasts", the id canon.toast() looks for.
	ID string
	// Position: "bottomEnd" (default), "bottomStart", "topEnd", "topStart".
	Position string
	// MaxVisible is how many toasts show at once, default 5.
	MaxVisible int
	// Label names the region, default "Notifications".
	Label string
	// DismissLabel is used by toasts created with canon.toast().
	DismissLabel string
}

// ToastViewport renders the corner region toasts appear in; its children are initial toasts.
func ToastViewport(p ToastViewportProps) templ.Component {
	return withChildren(func(children templ.Component) templ.Component {
		max := p.MaxVisible
		if max <= 0 {
			max = 5
		}
		a := newAttrs().set("popover", "manual").
			set("id", or(p.ID, "astryx-toasts")).
			set("class", "astryx-toast-viewport").
			set("data-position", or(p.Position, "bottomEnd")).
			set("data-max-visible", strconv.Itoa(max)).
			set("role", "region").
			set("aria-label", or(p.Label, "Notifications"))
		tpl := el("template", newAttrs().set("class", "astryx-toast-template"), Toast(ToastProps{DismissLabel: p.DismissLabel}))
		return el("div", a, tpl, children)
	})
}
