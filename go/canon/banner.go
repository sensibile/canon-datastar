package canon

import (
	"strings"

	"github.com/a-h/templ"
)

// Port of Astryx `Banner` (packages/core/src/Banner), MIT, Meta Platforms, Inc.
// static/canon.js expands and collapses the body (firing "canon-toggle" with detail.open) and
// dismisses the banner (hiding it, returning focus to where it came from and firing
// "canon-dismiss"); add DismissAttrs such as a data-on:click to tell the server too.
// With Role "alert", the alert role goes on the message so only it is announced.

// BannerProps configures Banner.
type BannerProps struct {
	// ID prefixes the content id. Defaults to "banner-" + a slug of Title.
	ID string
	// Status: "info" (default), "warning", "error", "success".
	Status      string
	Title       string
	TitleMarkup templ.Component
	Description string
	// DescriptionMarkup replaces Description.
	DescriptionMarkup templ.Component
	// Icon replaces the status icon.
	Icon          templ.Component
	IsDismissable bool
	// DismissLabel names the dismiss button ("Dismiss " + Title by default).
	DismissLabel string
	DismissAttrs []Attr
	EndContent   templ.Component
	// Container: "card" (default) or "section".
	Container string
	// Elevation: "none" (default), "low", "med", "high".
	Elevation string
	// Content is the body. It collapses behind a toggle (closed at first) unless IsNotCollapsible.
	Content          templ.Component
	IsNotCollapsible bool
	IsOpen           bool
	// Signal names a boolean Datastar signal that mirrors the body's open state.
	Signal string
	Role   string
	Class  string
	Attrs  []Attr
}

// Banner renders a status message.
func Banner(p BannerProps) templ.Component {
	status := or(p.Status, "info")
	container := or(p.Container, "card")
	id := or(p.ID, "banner-"+or(slug(p.Title), "message"))
	contentID := id + "-content"
	hasContent := p.Content != nil
	hasToggle := hasContent && !p.IsNotCollapsible
	open := p.IsOpen || !hasToggle
	hasActions := p.EndContent != nil || p.IsDismissable
	isAlert := p.Role == "alert"

	var icon templ.Component
	if p.Icon != nil {
		icon = el("div", newAttrs().set("class", "astryx-banner-icon").set("data-status", status).set("aria-hidden", "true"), p.Icon)
	} else {
		icon = Icon(IconProps{Name: status, Size: "md", Class: "astryx-banner-icon", Attrs: []Attr{{"data-status", status}}})
	}
	title := p.TitleMarkup
	if title == nil {
		title = text(p.Title)
	}
	var desc templ.Component
	if p.DescriptionMarkup != nil || p.Description != "" {
		d := p.DescriptionMarkup
		if d == nil {
			d = text(p.Description)
		}
		desc = group(text(" "), el("div", newAttrs().set("class", "astryx-banner-description"), d))
	}
	// With role="alert", aria-* attributes describe the alert message rather than the frame.
	var frameAttrs, msgAttrs []Attr
	for _, a := range p.Attrs {
		if isAlert && strings.HasPrefix(a.Name, "aria-") {
			msgAttrs = append(msgAttrs, a)
		} else {
			frameAttrs = append(frameAttrs, a)
		}
	}
	message := el("div", newAttrs().extra(msgAttrs).when(isAlert, "role", "alert").
		set("class", "astryx-banner-message").flag("data-with-end", p.EndContent != nil),
		el("div", newAttrs().set("class", "astryx-banner-title"), title), desc)

	var end templ.Component
	if hasActions || hasToggle {
		var toggle, dismiss templ.Component
		if hasToggle {
			label := map[bool]string{true: "Collapse", false: "Expand"}[open]
			toggle = IconButton(ButtonProps{Label: label, Variant: "ghost", Size: "sm", Icon: "chevronDown",
				Class: "astryx-banner-control astryx-banner-toggle",
				Attrs: []Attr{{"aria-expanded", map[bool]string{true: "true", false: "false"}[open]}, {"aria-controls", contentID}}})
		}
		if p.IsDismissable {
			name := p.DismissLabel
			if name == "" {
				name = "Dismiss"
				if p.TitleMarkup == nil && p.Title != "" {
					name = "Dismiss " + p.Title
				}
			}
			dismiss = IconButton(ButtonProps{Label: name, Variant: "ghost", Size: "sm", Icon: "close",
				Class: "astryx-banner-control astryx-banner-close", Attrs: append([]Attr{{"data-banner-dismiss", ""}}, p.DismissAttrs...)})
		}
		end = el("div", newAttrs().set("class", "astryx-banner-end"), p.EndContent, toggle, dismiss)
	}
	header := el("div", newAttrs().set("class", "astryx-banner").
		set("data-container", container).
		set("data-status", status).
		flag("data-centered", p.Description == "" && p.DescriptionMarkup == nil && hasActions),
		icon, message, end)

	var content templ.Component
	if hasContent {
		content = el("div", newAttrs().set("id", contentID).set("class", "astryx-banner-content").
			set("data-container", container).set("data-status", status).flag("hidden", !open), p.Content)
	}
	frame := newAttrs().set("id", id).
		set("class", cls("astryx-banner-frame", p.Class)).
		set("data-container", container).
		set("data-elevation", or(p.Elevation, "none")).
		when(!isAlert && p.Role != "", "role", p.Role)
	if p.Signal != "" && hasToggle {
		frame.set("data-effect", "canon.collapse(el, $"+p.Signal+")").
			set("data-on:canon-toggle", "evt.target === el && ($"+p.Signal+" = evt.detail.open)")
	}
	frame.extra(frameAttrs)
	return el("div", frame, header, content)
}
