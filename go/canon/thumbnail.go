package canon

import "github.com/a-h/templ"

// Port of Astryx `Thumbnail` (packages/core/src/Thumbnail), MIT, Meta Platforms, Inc.
// A 64px image tile. The placeholder is rendered hidden next to an image and static/canon.js
// shows it if the image fails to load.

// ThumbnailProps configures Thumbnail.
type ThumbnailProps struct {
	Src string
	// Alt describes the image; without it the image is decorative.
	Alt string
	// Label is a file name or caption, shown as a tooltip.
	Label     string
	TooltipID string
	// Action (any non-nil list, e.g. a data-on:click) makes the image an "open" button.
	Action []Attr
	// IsRemovable (or any RemoveAttrs) adds a remove button; RemoveAttrs are its attributes.
	IsRemovable bool
	RemoveAttrs []Attr
	// IsLoading shows a skeleton, or a spinner over an existing image.
	IsLoading  bool
	IsDisabled bool
	// ShowRemoveOn: "hover" (default) or "always".
	ShowRemoveOn string
	Class        string
	Attrs        []Attr
}

const thumbnailPlaceholder = "M21 19V5a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2M8.5 13.5l2.5 3 3.5-4.5 4.5 6H5l3.5-5.5z"

// Thumbnail renders an image tile.
func Thumbnail(p ThumbnailProps) templ.Component {
	hasSrc := p.Src != ""
	showSkeleton := p.IsLoading && !hasSrc
	showImage := hasSrc
	isOpenable := p.Action != nil
	isInteractive := isOpenable && !p.IsDisabled && !p.IsLoading
	removable := p.IsRemovable || len(p.RemoveAttrs) > 0
	name := "Thumbnail"
	switch {
	case p.Label != "" && p.Alt != "":
		name = p.Label + " — " + p.Alt
	case p.Label != "":
		name = p.Label
	case p.Alt != "":
		name = p.Alt
	}

	var image, skeleton templ.Component
	if showImage {
		image = void("img", newAttrs().set("src", p.Src).set("alt", p.Alt).
			when(p.Alt == "", "role", "presentation").when(p.Alt == "", "aria-hidden", "true").
			set("class", "astryx-thumbnail-image"))
	}
	if showSkeleton {
		skeleton = Skeleton(SkeletonProps{Radius: "2"})
	}
	var placeholder templ.Component
	if !showSkeleton {
		placeholder = el("div", newAttrs().set("class", "astryx-thumbnail-placeholder").flag("hidden", showImage),
			el("svg", newAttrs().set("width", "24").set("height", "24").set("viewBox", "0 0 24 24").
				set("fill", "currentColor").set("aria-hidden", "true"),
				el("path", newAttrs().set("d", thumbnailPlaceholder))))
	}
	visual := group(image, skeleton, placeholder)
	if isOpenable {
		a := newAttrs().set("type", "button").set("class", "astryx-thumbnail-open").
			set("aria-label", "Open "+name).when(!isInteractive, "aria-disabled", "true")
		if isInteractive {
			a.extra(p.Action)
		} else {
			a.extra(withoutPrefix(p.Action, "data-on:"))
		}
		visual = el("button", a, visual)
	}
	var border, overlay, remove templ.Component
	if showImage {
		border = el("div", newAttrs().set("class", "astryx-thumbnail-inset-border"))
	}
	if p.IsLoading && hasSrc {
		overlay = el("div", newAttrs().set("class", "astryx-thumbnail-upload-overlay"),
			Spinner(SpinnerProps{Size: "sm", Shade: "onMedia", Attrs: []Attr{{"aria-hidden", "true"}}}),
			visuallyHidden(VisuallyHiddenProps{Attrs: []Attr{{"role", "status"}}}, text("Loading")))
	}
	if removable {
		remove = el("div", newAttrs().set("class", "astryx-thumbnail-remove-slot"),
			IconButton(ButtonProps{Label: "Remove " + name, Icon: "close", Variant: "secondary", Size: "sm",
				IsDisabled: p.IsDisabled, Class: "astryx-thumbnail-remove", Attrs: p.RemoveAttrs}))
	}
	frame := newAttrs().set("class", "astryx-thumbnail-frame").
		flag("data-interactive", isInteractive).
		flag("data-reveal", removable && p.ShowRemoveOn != "always")
	a := newAttrs().set("role", "group").
		set("aria-label", name).
		set("class", cls("astryx-thumbnail", p.Class)).
		flag("data-disabled", p.IsDisabled).
		extra(p.Attrs)
	tile := el("div", a, el("div", frame, visual, border, overlay, remove))
	if p.Label == "" {
		return tile
	}
	return tooltip(TooltipProps{ID: p.TooltipID, Content: p.Label}, tile)
}
