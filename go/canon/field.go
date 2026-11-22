package canon

import "github.com/a-h/templ"

// Port of Astryx `Field`, `FieldLabel`, `FieldStatus` (packages/core/src/Field, FieldStatus),
// MIT, Meta Platforms, Inc. Not ported: labelTooltip (needs InfoTip), FormLayout horizontal labels,
// clicking the description to focus the control.

// FieldLabelProps configures FieldLabel.
type FieldLabelProps struct {
	Label string
	// InputID is the id of the control the label points at.
	InputID string
	LabelID string
	// IsGroupLabel renders a <span> instead of <label for> (for groups of controls).
	IsGroupLabel  bool
	IsLabelHidden bool
	IsDisabled    bool
	IsOptional    bool
	IsRequired    bool
	// LabelIcon is a built-in glyph name shown before the text.
	LabelIcon     string
	Description   string
	DescriptionID string
	Class         string
}

// FieldLabel renders a control's label and optional description.
func FieldLabel(p FieldLabelProps) templ.Component {
	tag := "label"
	if p.IsGroupLabel {
		tag = "span"
	}
	a := newAttrs().set("class", cls("astryx-field-label", p.Class)).opt("id", p.LabelID)
	if !p.IsGroupLabel {
		a.set("for", p.InputID)
	}
	a.flag("data-disabled", p.IsDisabled).flag("data-label-hidden", p.IsLabelHidden)
	var icon, status templ.Component
	if p.LabelIcon != "" {
		icon = Icon(IconProps{Name: p.LabelIcon, Size: "sm"})
	}
	statusText := ""
	if p.IsOptional {
		statusText = "Optional"
	} else if p.IsRequired {
		statusText = "Required"
	}
	if statusText != "" {
		status = el("span", newAttrs().set("class", "astryx-field-optionality"),
			el("span", newAttrs().set("aria-hidden", "true"), text(" ∙ ")), text(statusText))
	}
	label := el(tag, a, icon, text(p.Label), status)
	if p.Description == "" {
		return label
	}
	return el("div", newAttrs().set("class", "astryx-field-label-group").flag("data-label-hidden", p.IsLabelHidden),
		label,
		el("span", newAttrs().set("class", "astryx-field-description").opt("id", p.DescriptionID).
			flag("data-label-hidden", p.IsLabelHidden), text(p.Description)))
}

// FieldStatusMessage renders a status message. Variant: "attached" (default) or "detached".
func FieldStatusMessage(status FieldStatus, variant, id string) templ.Component {
	return fieldStatusMessage(status, variant, id, "", false)
}

// fieldStatusMessage adds an extra class and an initially hidden state (for client-shown messages).
func fieldStatusMessage(status FieldStatus, variant, id, class string, hidden bool) templ.Component {
	variant = or(variant, "attached")
	a := newAttrs().set("class", cls("astryx-field-status", class)).opt("id", id).flag("hidden", hidden).
		set("data-type", status.Type).set("data-variant", variant)
	if variant != "detached" {
		return el("div", a, text(status.Message))
	}
	return el("div", a, el("span", newAttrs().set("class", "astryx-field-status-content"),
		el("span", newAttrs().set("class", "astryx-field-status-icon"), Icon(IconProps{Name: status.Type, Size: "sm"})),
		el("span", newAttrs(), text(status.Message))))
}

// FieldProps configures Field.
type FieldProps struct {
	Label         string
	IsLabelHidden bool
	Description   string
	// InputID is the id of the wrapped control.
	InputID       string
	LabelID       string
	IsGroupLabel  bool
	DescriptionID string
	IsOptional    bool
	IsRequired    bool
	IsDisabled    bool
	LabelIcon     string
	Status        *FieldStatus
	// StatusVariant: "attached" (default, joined under the control) or "detached".
	StatusVariant string
	Width         string
	Class         string
	Attrs         []Attr
}

// Field wraps a control (its children) with label, description and status message.
func Field(p FieldProps) templ.Component {
	return withChildren(func(children templ.Component) templ.Component {
		return field(p, children)
	})
}

func field(p FieldProps, children templ.Component) templ.Component {
	descID := p.DescriptionID
	if descID == "" && p.Description != "" {
		descID = p.InputID + "-desc"
	}
	var status templ.Component
	if p.Status != nil && p.Status.Message != "" {
		status = FieldStatusMessage(*p.Status, p.StatusVariant, or(p.Status.MessageID, p.InputID+"-status"))
	}
	a := newAttrs().set("class", cls("astryx-field", p.Class)).
		flag("data-label-hidden", p.IsLabelHidden).
		opt("style", decl("width", p.Width)).
		extra(p.Attrs)
	label := FieldLabel(FieldLabelProps{Label: p.Label, InputID: p.InputID, LabelID: p.LabelID,
		IsGroupLabel: p.IsGroupLabel, IsLabelHidden: p.IsLabelHidden, IsDisabled: p.IsDisabled,
		IsOptional: p.IsOptional, IsRequired: p.IsRequired, LabelIcon: p.LabelIcon,
		Description: p.Description, DescriptionID: descID})
	if or(p.StatusVariant, "attached") == "attached" {
		return el("div", a, label, el("div", newAttrs().set("class", "astryx-field-control"), children, status))
	}
	return el("div", a, label, children, status)
}
