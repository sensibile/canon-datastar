package canon

import "github.com/a-h/templ"

// TextInputProps configures TextInput.
type TextInputProps struct {
	// ID of the <input>; defaults to "astryx-" + a slug of Label. Set it when labels repeat.
	ID    string
	Label string
	Value string
	// Type: "text" (default), "password", "email", "search", "tel", "url".
	Type          string
	IsLabelHidden bool
	Description   string
	IsOptional    bool
	IsRequired    bool
	// IsDisabled keeps the input focusable (readonly + aria-disabled) and drops its name.
	IsDisabled bool
	IsReadOnly bool
	// StartIcon is a built-in glyph name.
	StartIcon     string
	Status        *FieldStatus
	StatusVariant string
	// Size: "sm", "md" (default), "lg".
	Size         string
	IsLoading    bool
	Placeholder  string
	Width        string
	HasAutoFocus bool
	// Name is the input's form field name.
	Name string
	// HasSpellCheck sets spellcheck="true"/"false" when non-nil.
	HasSpellCheck *bool
	// Class goes on the Field container, ControlClass on the bordered row.
	Class, ControlClass string
	// Attrs go on the Field container; InputAttrs on the <input>, e.g. {"data-bind:email", ""}.
	Attrs, InputAttrs []Attr
}

// TextInput renders a labelled single-line text field.
// Port of Astryx `TextInput` (packages/core/src/TextInput), MIT, Meta Platforms, Inc.
// Not ported: hasClear, disabledMessage (needs Tooltip), InputGroup.
func TextInput(p TextInputProps) templ.Component {
	id := controlID(p.ID, p.Label)
	statusID := id + "-status"
	descID := ""
	if p.Description != "" {
		descID = id + "-desc"
	}
	var statusType string
	hasMessage := false
	if p.Status != nil {
		statusType = p.Status.Type
		hasMessage = p.Status.Message != ""
	}
	messageless := statusType == "error" && !hasMessage
	describedBy := joinIDs(descID, map[bool]string{true: statusID}[hasMessage || messageless])
	spell := ""
	if p.HasSpellCheck != nil {
		spell = map[bool]string{true: "true", false: "false"}[*p.HasSpellCheck]
	}
	input := newAttrs().set("id", id).
		when(!p.IsDisabled, "name", p.Name).
		set("type", or(p.Type, "text")).
		opt("value", p.Value).
		opt("placeholder", p.Placeholder).
		opt("spellcheck", spell).
		flag("readonly", p.IsReadOnly || p.IsDisabled).
		when(p.IsDisabled, "aria-disabled", "true").
		opt("aria-describedby", describedBy).
		when(p.IsRequired && !p.IsOptional, "aria-required", "true").
		when(statusType == "error", "aria-invalid", "true").
		when(p.IsLoading, "aria-busy", "true").
		flag("autofocus", p.HasAutoFocus).
		extra(p.InputAttrs)
	var startIcon, spinner, statusIcon, hidden templ.Component
	if p.StartIcon != "" {
		startIcon = Icon(IconProps{Name: p.StartIcon, Size: "sm", Color: "secondary"})
	}
	if p.IsLoading {
		spinner = Spinner(SpinnerProps{Size: "sm", Attrs: []Attr{{"aria-hidden", "true"}}})
	}
	if statusType != "" {
		statusIcon = Icon(IconProps{Name: statusType, Size: "md", Color: statusType})
	}
	if messageless {
		hidden = visuallyHidden(VisuallyHiddenProps{ID: statusID}, text("This field has an error"))
	}
	control := el("div", newAttrs().set("class", cls("astryx-text-input", p.ControlClass)).
		set("data-size", or(p.Size, "md")).
		opt("data-status", statusType).
		flag("data-disabled", p.IsDisabled).
		flag("data-readonly", p.IsReadOnly),
		startIcon, void("input", input), spinner, statusIcon)
	var status *FieldStatus
	if p.Status != nil {
		status = &FieldStatus{Type: p.Status.Type, Message: p.Status.Message, MessageID: statusID}
	}
	return field(FieldProps{Label: p.Label, IsLabelHidden: p.IsLabelHidden, Description: p.Description,
		InputID: id, DescriptionID: descID, IsOptional: p.IsOptional, IsRequired: p.IsRequired,
		IsDisabled: p.IsDisabled, Status: status, StatusVariant: p.StatusVariant, Width: p.Width,
		Class: p.Class, Attrs: p.Attrs}, group(control, hidden))
}

// CheckboxInputProps configures CheckboxInput.
type CheckboxInputProps struct {
	// ID of the <input>; defaults to "astryx-" + a slug of Label.
	ID              string
	Label           string
	Checked         bool
	IsIndeterminate bool
	IsLabelHidden   bool
	Description     string
	IsLoading       bool
	IsDisabled      bool
	IsReadOnly      bool
	IsOptional      bool
	IsRequired      bool
	Name            string
	// Value is the submitted form value (browser default "on").
	Value string
	// Size: "sm" or "md" (default).
	Size      string
	Width     string
	LabelIcon string
	Status    *FieldStatus
	Class     string
	// Attrs go on the root; InputAttrs on the <input>, e.g. {"data-bind:agree", ""}.
	Attrs, InputAttrs []Attr
}

// CheckboxInput renders a labelled checkbox. The indicator follows the native input
// (:checked, :indeterminate), so data-bind on the input drives it.
// Port of Astryx `CheckboxInput` (packages/core/src/CheckboxInput), MIT, Meta Platforms, Inc.
// IsIndeterminate and IsReadOnly/IsLoading click-blocking use Datastar (data-init / data-on:click).
func CheckboxInput(p CheckboxInputProps) templ.Component {
	id := controlID(p.ID, p.Label)
	size := or(p.Size, "md")
	descID, statusID := "", ""
	if p.Description != "" {
		descID = id + "-desc"
	}
	if p.Status != nil && p.Status.Message != "" {
		statusID = id + "-status"
	}
	input := checkInput(id, "", p.Name, p.Value, p.Checked, p.IsRequired, p.IsDisabled, descID, statusID, p.Status, p.IsLoading).
		when(p.IsReadOnly, "aria-readonly", "true").
		set("data-size", size).
		when(p.IsIndeterminate, "data-init", "el.indeterminate = true").
		when(p.IsReadOnly || p.IsLoading, "data-on:click", "evt.preventDefault()").
		extra(p.InputAttrs)
	var mark templ.Component
	if p.IsLoading {
		mark = Spinner(SpinnerProps{Size: "sm", Shade: "inherit"})
	} else {
		mark = raw(`<svg viewBox="0 0 10 10" class="astryx-checkbox-indicator-check"><path d="M8.5 2.5L4 7.5L1.5 5" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round" stroke-linejoin="round"></path></svg><span class="astryx-checkbox-indicator-dash"></span>`)
	}
	var status templ.Component
	if statusID != "" {
		status = FieldStatusMessage(*p.Status, "detached", statusID)
	}
	return el("div", newAttrs().set("class", cls("astryx-checkbox-input", p.Class)).
		set("data-checkbox-size", size).
		opt("style", decl("width", p.Width)).
		extra(p.Attrs),
		el("div", newAttrs().set("class", "astryx-checkbox-row").
			flag("data-label-hidden", p.IsLabelHidden).flag("data-disabled", p.IsDisabled),
			el("div", newAttrs().set("class", "astryx-checkbox-control"),
				void("input", input),
				el("span", newAttrs().set("class", "astryx-checkbox-indicator").set("aria-hidden", "true").set("data-size", size), mark)),
			el("div", newAttrs().set("class", "astryx-checkbox-label-wrapper"),
				FieldLabel(FieldLabelProps{Class: "astryx-checkbox-label", Label: p.Label, InputID: id,
					IsLabelHidden: p.IsLabelHidden, IsDisabled: p.IsDisabled, IsOptional: p.IsOptional,
					IsRequired: p.IsRequired, LabelIcon: p.LabelIcon, Description: p.Description, DescriptionID: descID}))),
		status)
}

// checkInput writes the attributes CheckboxInput and Switch share, in the shared order.
func checkInput(id, role, name, value string, checked, required, disabled bool, descID, statusID string, status *FieldStatus, loading bool) *attrs {
	invalid := ""
	if status != nil && status.Type == "error" {
		invalid = "true"
	} else if required {
		invalid = "false"
	}
	return newAttrs().set("id", id).set("type", "checkbox").opt("role", role).
		when(!disabled, "name", name).
		opt("value", value).
		flag("checked", checked).
		flag("required", required).
		flag("disabled", disabled).
		when(required, "aria-required", "true").
		opt("aria-describedby", joinIDs(descID, statusID)).
		opt("aria-invalid", invalid).
		when(loading, "aria-busy", "true")
}

// SwitchProps configures Switch.
type SwitchProps struct {
	// ID of the <input>; defaults to "astryx-" + a slug of Label.
	ID            string
	Label         string
	Checked       bool
	IsLabelHidden bool
	Description   string
	IsLoading     bool
	IsDisabled    bool
	Name          string
	// Value is the submitted form value (browser default "on").
	Value      string
	IsOptional bool
	IsRequired bool
	LabelIcon  string
	Width      string
	// LabelPosition: "end" (default) or "start".
	LabelPosition string
	// LabelSpacing: "hug" (default) or "spread" (label and switch at opposite ends).
	LabelSpacing string
	Status       *FieldStatus
	// Size: "sm" or "md" (default).
	Size  string
	Class string
	// Attrs go on the root; InputAttrs on the <input>, e.g. {"data-bind:notify", ""}.
	Attrs, InputAttrs []Attr
}

// Switch renders a labelled on/off switch (a checkbox with role="switch"). The track follows
// the native input's :checked state.
// Port of Astryx `Switch` (packages/core/src/Switch), MIT, Meta Platforms, Inc.
// Not ported: drag-to-toggle, disabledMessage, labelTooltip.
func Switch(p SwitchProps) templ.Component {
	id := controlID(p.ID, p.Label)
	size := or(p.Size, "md")
	descID, statusID := "", ""
	if p.Description != "" {
		descID = id + "-desc"
	}
	if p.Status != nil && p.Status.Message != "" {
		statusID = id + "-status"
	}
	input := checkInput(id, "switch", p.Name, p.Value, p.Checked, p.IsRequired, p.IsDisabled, descID, statusID, p.Status, p.IsLoading).
		when(p.IsLoading, "data-on:click", "evt.preventDefault()").
		extra(p.InputAttrs)
	var spinner templ.Component
	if p.IsLoading {
		spinner = Spinner(SpinnerProps{Size: "sm"})
	}
	control := el("div", newAttrs().set("class", "astryx-switch-control"),
		void("input", input),
		el("div", newAttrs().set("class", "astryx-switch").set("aria-hidden", "true").set("data-size", size),
			el("div", newAttrs().set("class", "astryx-switch-thumb").set("data-size", size), spinner)))
	label := el("div", newAttrs().set("class", "astryx-switch-label-wrapper").set("data-size", size),
		FieldLabel(FieldLabelProps{Class: "astryx-switch-label", Label: p.Label, InputID: id,
			IsLabelHidden: p.IsLabelHidden, IsDisabled: p.IsDisabled, IsOptional: p.IsOptional,
			IsRequired: p.IsRequired, LabelIcon: p.LabelIcon, Description: p.Description, DescriptionID: descID}))
	first, second := control, label
	if p.LabelPosition == "start" {
		first, second = label, control
	}
	var status templ.Component
	if statusID != "" {
		status = FieldStatusMessage(*p.Status, "detached", statusID)
	}
	return el("div", newAttrs().set("class", cls("astryx-switch-field", p.Class)).
		when(p.LabelPosition == "start", "data-label-position", "start").
		when(p.LabelSpacing == "spread", "data-label-spacing", "spread").
		opt("style", decl("width", p.Width)).
		extra(p.Attrs),
		el("div", newAttrs().set("class", "astryx-switch-container").
			flag("data-label-hidden", p.IsLabelHidden).flag("data-disabled", p.IsDisabled),
			first, second),
		status)
}
