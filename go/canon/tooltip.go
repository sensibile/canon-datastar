package canon

import (
	"strconv"

	"github.com/a-h/templ"
)

// Port of Astryx `Tooltip` and `InfoTip` (packages/core/src/Tooltip, InfoTip), MIT, Meta Platforms, Inc.
// The trigger is the first element inside (or the text itself); static/canon.js shows the tooltip
// on hover after Delay, on keyboard focus, and hides it on leave, blur, press or Escape, and adds
// the tooltip to the trigger's aria-describedby. Not ported: touch long-press, controlled open.

// TooltipProps configures Tooltip.
type TooltipProps struct {
	// ID of the tooltip element. Defaults to "tooltip-" + a slug of Content.
	ID string
	// Content is the tooltip text; ContentMarkup replaces it with rich content.
	Content       string
	ContentMarkup templ.Component
	// Placement: "above" (default), "below", "start", "end".
	Placement string
	// Alignment: "center" (default), "start", "end".
	Alignment string
	// Delay is the ms before showing on hover; 0 means the default 200.
	Delay int
	// HideDelay is the ms before hiding after the pointer leaves.
	HideDelay int
	// FocusTrigger: "auto" (default, keyboard focus only), "always", "never".
	FocusTrigger string
	// IsDisabled renders the trigger without a tooltip.
	IsDisabled bool
	// HoverIndication: "auto" (default, dashed underline on plain-text triggers), "always", "never".
	HoverIndication string
	Class           string
	Attrs           []Attr
}

func (p TooltipProps) id() string {
	if p.ID != "" {
		return p.ID
	}
	return "tooltip-" + or(slug(p.Content), "content")
}

// Tooltip wraps its children (the trigger) and renders the tooltip layer after them.
func Tooltip(p TooltipProps) templ.Component {
	return withChildren(func(children templ.Component) templ.Component { return tooltip(p, children) })
}

func tooltip(p TooltipProps, children templ.Component) templ.Component {
	if p.IsDisabled {
		return children
	}
	id := p.id()
	indication := or(p.HoverIndication, "auto")
	trigger := newAttrs().set("class", "astryx-tooltip-trigger").
		set("data-tooltip", id).
		when(p.Delay > 0, "data-delay", strconv.Itoa(p.Delay)).
		when(p.HideDelay > 0, "data-hide-delay", strconv.Itoa(p.HideDelay)).
		when(p.FocusTrigger != "" && p.FocusTrigger != "auto", "data-focus-trigger", p.FocusTrigger).
		when(indication != "never", "data-indication", indication)
	layer := newAttrs().set("popover", "manual").
		set("id", id).
		set("role", "tooltip").
		set("class", cls("astryx-layer astryx-tooltip", p.Class)).
		set("data-placement", or(p.Placement, "above")).
		set("data-alignment", or(p.Alignment, "center")).
		extra(p.Attrs)
	content := p.ContentMarkup
	if content == nil {
		content = text(p.Content)
	}
	return group(
		el("span", trigger, children),
		el("div", layer, el("div", newAttrs().set("class", "astryx-tooltip-content"), content)),
	)
}

// InfoTipProps configures InfoTip.
type InfoTipProps struct {
	// ID of the tooltip element. Defaults to "infotip-" + a slug of Content.
	ID            string
	Content       string
	ContentMarkup templ.Component
	// Label is the button's accessible name. Default "More information".
	Label string
	// Size of the icon: "xsm", "sm" (default), "md", "lg".
	Size      string
	Placement string
	Class     string
	Attrs     []Attr
}

// InfoTip renders a small info button that shows a tooltip on hover or focus and pins it open
// on click. Port of Astryx `InfoTip` (packages/core/src/InfoTip), MIT, Meta Platforms, Inc.
func InfoTip(p InfoTipProps) templ.Component {
	id := p.ID
	if id == "" {
		id = "infotip-" + or(slug(p.Content), "content")
	}
	btn := newAttrs().set("type", "button").
		set("class", cls("astryx-info-tip", p.Class)).
		set("aria-label", or(p.Label, "More information")).
		set("aria-expanded", "false").
		extra(p.Attrs)
	return tooltip(TooltipProps{
		ID: id, Content: p.Content, ContentMarkup: p.ContentMarkup, Placement: p.Placement,
		HoverIndication: "never",
	}, el("button", btn, Icon(IconProps{Name: "info", Size: or(p.Size, "sm")})))
}
