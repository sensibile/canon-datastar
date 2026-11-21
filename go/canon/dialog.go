package canon

import "github.com/a-h/templ"

// Port of Astryx `Dialog` and `DialogHeader` (packages/core/src/Dialog), MIT, Meta Platforms, Inc.
// A modal native <dialog>. Open it with canon.dialog(id) or a Datastar signal (Signal). Purpose
// decides how it can be dismissed: "info" closes on Escape and backdrop press, "form" on Escape
// only, "required" only by its own controls. Focus moves to the DialogHeader title on open and
// returns to the opener on close. Not ported: isInline, custom position, directional animation.

// DialogProps configures Dialog.
type DialogProps struct {
	ID string
	// Width is a CSS length, default "400px". MaxHeight defaults to "75dvh".
	Width     string
	MaxHeight string
	// Variant: "standard" (default) or "fullscreen".
	Variant string
	// Purpose: "info" (default), "form", "required".
	Purpose string
	// Padding is a spacing step, default "4".
	Padding Space
	// Label names the dialog when there is no DialogHeader.
	Label string
	// Signal names a boolean Datastar signal that opens and closes the dialog; closing it in
	// the browser sets the signal back to false.
	Signal string
	Class  string
	Attrs  []Attr
}

var closedBy = map[string]string{"info": "any", "form": "closerequest", "required": "none"}

// Dialog renders a <dialog> around its children.
func Dialog(p DialogProps) templ.Component {
	return withChildren(func(children templ.Component) templ.Component {
		variant := or(p.Variant, "standard")
		purpose := or(p.Purpose, "info")
		size := ""
		if variant != "fullscreen" {
			size = styles(decl("width", or(p.Width, "400px")), decl("max-height", or(p.MaxHeight, "75dvh")))
		}
		a := newAttrs().opt("id", p.ID).
			set("class", cls("astryx-dialog", p.Class)).
			set("data-variant", variant).
			set("data-purpose", purpose).
			opt("closedby", closedBy[purpose]).
			opt("aria-label", p.Label).
			set("style", styles(size, decl("--_dialog-padding", Space(or(string(p.Padding), "4")).css())))
		if p.Signal != "" {
			a.set("data-effect", "canon.dialog(el, $"+p.Signal+")").
				set("data-on:close", "$"+p.Signal+" = false")
		}
		a.extra(p.Attrs)
		return el("dialog", a, el("div", newAttrs().set("class", "astryx-dialog-inner"), children))
	})
}

// DialogHeaderProps configures DialogHeader.
type DialogHeaderProps struct {
	Title string
	// TitleMarkup replaces Title with rich content.
	TitleMarkup templ.Component
	Subtitle    string
	// TitleID sets the title's id; canon.js otherwise derives one to label the dialog.
	TitleID string
	// HasCloseButton adds a close button that closes the enclosing dialog.
	HasCloseButton bool
	// CloseLabel is the close button's accessible name, default "Close".
	CloseLabel   string
	StartContent templ.Component
	EndContent   templ.Component
	HasDivider   bool
	Class        string
	Attrs        []Attr
}

// DialogHeader renders the title row of a Dialog.
func DialogHeader(p DialogHeaderProps) templ.Component {
	title := p.TitleMarkup
	if title == nil {
		title = text(p.Title)
	}
	var start, end, subtitle, divider templ.Component
	if p.StartContent != nil {
		start = el("div", newAttrs().set("class", "astryx-dialog-header-actions"), p.StartContent)
	}
	if p.EndContent != nil || p.HasCloseButton {
		var closeBtn templ.Component
		if p.HasCloseButton {
			closeBtn = Button(ButtonProps{Variant: "ghost", Label: or(p.CloseLabel, "Close"), Icon: "close",
				IsIconOnly: true, Attrs: []Attr{{"data-dialog-close", ""}}})
		}
		end = el("div", newAttrs().set("class", "astryx-dialog-header-actions astryx-dialog-header-end"),
			p.EndContent, closeBtn)
	}
	if p.Subtitle != "" {
		subtitle = textEl(TextProps{Type: "body", Size: "sm", Color: "secondary"}, text(p.Subtitle))
	}
	if p.HasDivider {
		divider = Divider(DividerProps{})
	}
	heading := headingEl(HeadingProps{Level: 2, Class: "astryx-dialog-title",
		Attrs: append(optAttr("id", p.TitleID), Attr{"tabindex", "-1"}, Attr{"autofocus", ""})}, title)
	row := el("div", newAttrs().set("class", "astryx-dialog-header-row").flag("data-has-close", p.HasCloseButton),
		start,
		el("div", newAttrs().set("class", "astryx-dialog-header-titles"), heading, subtitle),
		end)
	return el("div", newAttrs().set("class", cls("astryx-dialog-header", p.Class)).extra(p.Attrs), row, divider)
}

func optAttr(name, value string) []Attr {
	if value == "" {
		return nil
	}
	return []Attr{{name, value}}
}
