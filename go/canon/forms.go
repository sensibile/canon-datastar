package canon

import (
	"math"
	"strconv"
	"strings"
	"unicode/utf8"

	"github.com/a-h/templ"
)

// TextAreaProps configures TextArea.
type TextAreaProps struct {
	// ID of the <textarea>; defaults to "astryx-" + a slug of Label.
	ID            string
	Label         string
	Value         string
	IsLabelHidden bool
	Description   string
	IsOptional    bool
	IsRequired    bool
	// IsDisabled keeps the textarea focusable (readonly + aria-disabled) and drops its name.
	IsDisabled bool
	IsReadOnly bool
	// StartIcon is a built-in glyph name.
	StartIcon     string
	Status        *FieldStatus
	StatusVariant string
	// Size: "sm", "md" (default), "lg".
	Size        string
	IsLoading   bool
	Placeholder string
	// Rows defaults to 3.
	Rows  int
	Width string
	// MaxLength is a soft limit: it shows a counter and marks the field invalid past it
	// without blocking input. canon.js keeps the counter current as the user types.
	MaxLength int
	// HTMLMaxLength is the hard native maxlength.
	HTMLMaxLength int
	HasAutoFocus  bool
	Name          string
	// HasSpellCheck sets spellcheck="true"/"false" when non-nil (Astryx default true).
	HasSpellCheck *bool
	// Class goes on the Field container, ControlClass on the bordered box.
	Class, ControlClass string
	// Attrs go on the Field container; InputAttrs on the <textarea>, e.g. {"data-bind:bio", ""}.
	Attrs, InputAttrs []Attr
}

// TextArea renders a labelled multi-line text field with an optional character counter.
// Port of Astryx `TextArea` (packages/core/src/TextArea), MIT, Meta Platforms, Inc.
// The server counts code points; canon.js recounts user-perceived characters on load and input.
// Not ported: changeAction, disabledMessage, the "tooltip" status variant.
func TextArea(p TextAreaProps) templ.Component {
	id := controlID(p.ID, p.Label)
	statusID, counterID := id+"-status", id+"-counter"
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
	hasCounter := p.MaxLength > 0
	length := 0
	if hasCounter {
		length = utf8.RuneCountInString(p.Value)
	}
	over := hasCounter && length > p.MaxLength
	describedBy := joinIDs(descID, map[bool]string{true: statusID}[hasMessage || messageless],
		map[bool]string{true: counterID}[hasCounter])
	spell := ""
	if p.HasSpellCheck != nil {
		spell = strconv.FormatBool(*p.HasSpellCheck)
	}
	rows := p.Rows
	if rows <= 0 {
		rows = 3
	}
	busy := p.IsLoading && statusType != ""
	ta := newAttrs().set("class", "astryx-text-area-control").set("id", id).
		when(!p.IsDisabled && p.Name != "", "name", p.Name).
		opt("placeholder", p.Placeholder).
		set("rows", strconv.Itoa(rows)).
		when(p.HTMLMaxLength > 0, "maxlength", strconv.Itoa(p.HTMLMaxLength)).
		when(hasCounter, "data-max-length", strconv.Itoa(p.MaxLength)).
		opt("spellcheck", spell).
		flag("readonly", p.IsReadOnly || p.IsDisabled).
		flag("required", p.IsRequired && !p.IsOptional && !p.IsDisabled).
		when(p.IsDisabled, "aria-disabled", "true").
		opt("aria-describedby", describedBy).
		when(p.IsRequired && !p.IsOptional, "aria-required", "true").
		when(statusType == "error" || over, "aria-invalid", "true").
		when(p.IsLoading, "aria-busy", "true").
		flag("autofocus", p.HasAutoFocus).
		flag("data-start-icon", p.StartIcon != "").
		flag("data-with-status", p.IsLoading || statusType != "").
		flag("data-busy-status", busy).
		flag("data-counter", hasCounter).
		extra(p.InputAttrs)
	// The HTML parser drops one newline right after <textarea>, so keep a leading one.
	value := p.Value
	if strings.HasPrefix(value, "\n") {
		value = "\n" + value
	}
	var startIcon, end, counter, live, hidden templ.Component
	if p.StartIcon != "" {
		startIcon = el("span", newAttrs().set("class", "astryx-text-area-start-icon"),
			Icon(IconProps{Name: p.StartIcon, Size: "sm", Color: "secondary"}))
	}
	if p.IsLoading || statusType != "" {
		var spinner, icon templ.Component
		if p.IsLoading {
			spinner = Spinner(SpinnerProps{Size: "sm", Attrs: []Attr{{"aria-hidden", "true"}}})
		}
		if statusType != "" {
			icon = Icon(IconProps{Name: statusType, Size: "md", Color: statusType})
		}
		end = el("span", newAttrs().set("class", "astryx-text-area-end"), spinner, icon)
	}
	if hasCounter {
		// The warning icon shows only while over the limit (CSS), so canon.js only edits the count.
		counter = el("div", newAttrs().set("id", counterID).set("class", "astryx-text-area-counter").flag("data-over", over),
			Icon(IconProps{Name: "warning", Size: "sm"}),
			el("span", newAttrs().set("class", "astryx-text-area-count"),
				text(groupThousands(length)+"/"+groupThousands(p.MaxLength))))
		live = group(
			visuallyHidden(VisuallyHiddenProps{Class: "astryx-text-area-polite", Attrs: []Attr{{"aria-live", "polite"}}}, nil),
			visuallyHidden(VisuallyHiddenProps{Class: "astryx-text-area-assertive", Attrs: []Attr{{"aria-live", "assertive"}}}, nil))
	}
	if messageless {
		hidden = visuallyHidden(VisuallyHiddenProps{ID: statusID}, text("Invalid value"))
	}
	control := el("div", newAttrs().set("class", cls("astryx-text-area", p.ControlClass)).
		set("data-size", or(p.Size, "md")).
		opt("data-status", statusType).
		flag("data-disabled", p.IsDisabled).
		flag("data-readonly", p.IsReadOnly),
		startIcon, el("textarea", ta, text(value)), end, counter)
	var status *FieldStatus
	if p.Status != nil {
		status = &FieldStatus{Type: p.Status.Type, Message: p.Status.Message, MessageID: statusID}
	}
	return field(FieldProps{Label: p.Label, IsLabelHidden: p.IsLabelHidden, Description: p.Description,
		InputID: id, DescriptionID: descID, IsOptional: p.IsOptional, IsRequired: p.IsRequired,
		IsDisabled: p.IsDisabled, Status: status, StatusVariant: p.StatusVariant, Width: p.Width,
		Class: p.Class, Attrs: p.Attrs}, group(control, live, hidden))
}

// CheckboxListItem is one row of a CheckboxList.
type CheckboxListItem struct {
	// ID of the <input>; defaults to the list's id + "-" + its index.
	ID    string
	Label string
	// LabelContent replaces Label with markup; the checkbox is then named by the visible
	// label unless AriaLabel is set.
	LabelContent    templ.Component
	AriaLabel       string
	Value           string
	Description     string
	EndContent      templ.Component
	IsChecked       bool
	IsIndeterminate bool
	IsDisabled      bool
	IsLoading       bool
	// Align: "center" (default), "start" or "first-line".
	Align string
	Class string
	// Attrs go on the row; InputAttrs on the <input>.
	Attrs, InputAttrs []Attr
}

// CheckboxListProps configures CheckboxList.
type CheckboxListProps struct {
	// ID of the group; defaults to "astryx-" + a slug of Label.
	ID            string
	Label         string
	IsLabelHidden bool
	Description   string
	IsOptional    bool
	// IsRequired marks the first enabled box required while none is checked, so native form
	// validation asks for at least one (canon.js keeps it current).
	IsRequired bool
	Status     *FieldStatus
	// Name is the form field name every box submits its Value under.
	Name string
	// Density: "balanced" (default), "compact" (small boxes) or "spacious".
	Density     string
	HasDividers bool
	IsDisabled  bool
	IsReadOnly  bool
	Width       string
	Items       []CheckboxListItem
	// Class and Attrs go on the Field container; InputAttrs on every <input>, e.g.
	// {{"data-bind:toppings", ""}} with an array signal.
	Class             string
	Attrs, InputAttrs []Attr
}

const checkMark = `<svg viewBox="0 0 10 10" class="astryx-checkbox-indicator-check"><path d="M8.5 2.5L4 7.5L1.5 5" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round" stroke-linejoin="round"></path></svg><span class="astryx-checkbox-indicator-dash"></span>`

// CheckboxList renders a labelled group of checkbox rows. Clicking anywhere on a row toggles it.
// Port of Astryx `CheckboxList` (packages/core/src/CheckboxList), MIT, Meta Platforms, Inc.
// Not ported: changeAction, disabledMessage.
func CheckboxList(p CheckboxListProps) templ.Component {
	id := controlID(p.ID, p.Label)
	labelID, descID, statusID := id+"-label", "", ""
	if p.Description != "" {
		descID = id + "-desc"
	}
	if p.Status != nil && p.Status.Message != "" {
		statusID = id + "-status"
	}
	density := or(p.Density, "balanced")
	size := "md"
	if density == "compact" {
		size = "sm"
	}
	required := p.IsRequired && !p.IsOptional
	// The first enabled box carries `required` while none is checked.
	requiredIdx := -1
	if required && !p.IsReadOnly {
		checked := false
		for _, it := range p.Items {
			checked = checked || it.IsChecked
		}
		for i, it := range p.Items {
			if !checked && !p.IsDisabled && !it.IsDisabled {
				requiredIdx = i
				break
			}
		}
	}
	invalid := p.Status != nil && p.Status.Type == "error"
	rows := make([]templ.Component, len(p.Items))
	for i, it := range p.Items {
		inputID := or(it.ID, id+"-"+strconv.Itoa(i))
		itemLabelID, itemDescID := inputID+"-label", ""
		if it.Description != "" {
			itemDescID = inputID + "-desc"
		}
		disabled := p.IsDisabled || it.IsDisabled
		fromVisible := it.LabelContent != nil && it.AriaLabel == ""
		ariaLabel := or(it.AriaLabel, or(it.Label, "Checkbox"))
		input := newAttrs().set("id", inputID).set("type", "checkbox").
			when(!disabled && p.Name != "", "name", p.Name).
			opt("value", it.Value).
			flag("checked", it.IsChecked).
			flag("disabled", disabled).
			flag("required", i == requiredIdx).
			when(p.IsReadOnly, "aria-readonly", "true").
			when(!fromVisible, "aria-label", ariaLabel).
			when(fromVisible, "aria-labelledby", itemLabelID).
			opt("aria-describedby", itemDescID).
			when(invalid, "aria-invalid", "true").
			when(it.IsLoading, "aria-busy", "true").
			set("data-size", size).
			when(it.IsIndeterminate, "data-init", "el.indeterminate = true").
			when(p.IsReadOnly || it.IsLoading, "data-on:click", "evt.preventDefault()").
			extra(p.InputAttrs).extra(it.InputAttrs)
		var mark templ.Component = raw(checkMark)
		if it.IsLoading {
			mark = Spinner(SpinnerProps{Size: "sm", Shade: "inherit"})
		}
		media := el("span", newAttrs().set("class", "astryx-checkbox-control"),
			void("input", input),
			el("span", newAttrs().set("class", "astryx-checkbox-indicator").set("aria-hidden", "true").set("data-size", size), mark))
		rowLabelID := ""
		if fromVisible {
			rowLabelID = itemLabelID
		}
		rows[i] = Item(ItemProps{As: "li", Label: it.Label, LabelContent: it.LabelContent,
			Description: it.Description, StartContent: media, EndContent: it.EndContent,
			Align: it.Align, Density: density, IsDelegate: !p.IsReadOnly, IsDisabled: disabled,
			LabelID: rowLabelID, DescriptionID: itemDescID,
			Class: cls("astryx-checkbox-list-item", it.Class),
			Attrs: append(boolAttr(it.IsLoading, "aria-busy", "true"), it.Attrs...)})
	}
	groupEl := el("div", newAttrs().set("id", id).set("role", "group").set("class", "astryx-checkbox-list-group").
		flag("data-required", required).
		when(p.IsDisabled, "aria-disabled", "true").
		set("aria-labelledby", labelID).
		opt("aria-describedby", joinIDs(descID, statusID)),
		el("ul", newAttrs().set("class", "astryx-list").set("data-density", density).flag("data-dividers", p.HasDividers), rows...))
	var status *FieldStatus
	if p.Status != nil {
		status = &FieldStatus{Type: p.Status.Type, Message: p.Status.Message, MessageID: statusID}
	}
	return field(FieldProps{Label: p.Label, IsLabelHidden: p.IsLabelHidden, Description: p.Description,
		InputID: id, LabelID: labelID, IsGroupLabel: true, DescriptionID: descID,
		IsOptional: p.IsOptional, IsRequired: p.IsRequired, IsDisabled: p.IsDisabled, Status: status,
		StatusVariant: "detached", Width: p.Width, Class: cls("astryx-checkbox-list", p.Class), Attrs: p.Attrs}, groupEl)
}

func boolAttr(cond bool, name, value string) []Attr {
	if cond {
		return []Attr{{name, value}}
	}
	return nil
}

// RadioListItem is one option of a RadioList.
type RadioListItem struct {
	// ID of the <input>; defaults to the list's id + "-" + its index.
	ID           string
	Label        string
	LabelContent templ.Component
	AriaLabel    string
	Value        string
	Description  string
	StartContent templ.Component
	EndContent   templ.Component
	IsDisabled   bool
	// Align: "center" (default), "start" or "first-line".
	Align string
	Class string
	// Attrs go on the row; InputAttrs on the <input>.
	Attrs, InputAttrs []Attr
}

// RadioListProps configures RadioList.
type RadioListProps struct {
	// ID of the radiogroup; defaults to "astryx-" + a slug of Label.
	ID            string
	Label         string
	IsLabelHidden bool
	Description   string
	// Value is the checked option's value.
	Value string
	// Orientation: "vertical" (default) or "horizontal".
	Orientation string
	IsDisabled  bool
	// Name groups the radios; defaults to the group id + "-radio".
	Name       string
	IsRequired bool
	IsOptional bool
	Status     *FieldStatus
	// Size: "sm" or "md" (default).
	Size  string
	Width string
	Items []RadioListItem
	// Class goes on the radiogroup, ContainerClass on the Field container. InputAttrs go on every
	// <input>, e.g. {{"data-bind:plan", ""}}.
	Class, ContainerClass string
	Attrs, InputAttrs     []Attr
}

// RadioList renders a labelled radiogroup. Tab enters on the checked radio and leaves the group
// in one step (canon.js); arrow keys move between radios natively.
// Port of Astryx `RadioList` (packages/core/src/RadioList), MIT, Meta Platforms, Inc.
// Not ported: disabledMessage.
func RadioList(p RadioListProps) templ.Component {
	id := controlID(p.ID, p.Label)
	labelID, descID, statusID := id+"-label", "", ""
	if p.Description != "" {
		descID = id + "-desc"
	}
	if p.Status != nil && p.Status.Message != "" {
		statusID = id + "-status"
	}
	size := or(p.Size, "md")
	name := or(p.Name, id+"-radio")
	required := p.IsRequired && !p.IsOptional
	rows := make([]templ.Component, len(p.Items))
	for i, it := range p.Items {
		inputID := or(it.ID, id+"-"+strconv.Itoa(i))
		itemLabelID, itemDescID := inputID+"-label", ""
		if it.Description != "" {
			itemDescID = inputID + "-desc"
		}
		disabled := p.IsDisabled || it.IsDisabled
		input := newAttrs().set("id", inputID).set("type", "radio").
			set("name", name).
			set("value", it.Value).
			flag("checked", p.Value != "" && p.Value == it.Value).
			flag("disabled", disabled).
			flag("required", required).
			opt("aria-label", it.AriaLabel).
			when(it.AriaLabel == "", "aria-labelledby", itemLabelID).
			opt("aria-describedby", itemDescID).
			extra(p.InputAttrs).extra(it.InputAttrs)
		media := group(el("span", newAttrs().set("class", "astryx-radio-control"),
			void("input", input),
			el("span", newAttrs().set("class", "astryx-radio-indicator").set("aria-hidden", "true").set("data-size", size),
				el("span", newAttrs().set("class", "astryx-radio-indicator-dot")))), it.StartContent)
		rows[i] = Item(ItemProps{Label: it.Label, LabelContent: it.LabelContent, Description: it.Description,
			StartContent: media, EndContent: it.EndContent, Align: it.Align, IsDelegate: true,
			IsDisabled: disabled, LabelID: itemLabelID, DescriptionID: itemDescID,
			Class: cls("astryx-radio-list-item", it.Class),
			Attrs: append([]Attr{{"data-size", size}}, it.Attrs...)})
	}
	groupEl := el("div", newAttrs().set("id", id).set("role", "radiogroup").
		set("class", cls("astryx-radio-list", p.Class)).
		set("data-orientation", or(p.Orientation, "vertical")).
		set("data-size", size).
		when(p.IsDisabled, "aria-disabled", "true").
		set("aria-labelledby", labelID).
		opt("aria-describedby", joinIDs(descID, statusID)).
		when(p.Status != nil && p.Status.Type == "error", "aria-invalid", "true").
		when(required, "aria-required", "true").
		extra(p.Attrs), rows...)
	var status *FieldStatus
	if p.Status != nil {
		status = &FieldStatus{Type: p.Status.Type, Message: p.Status.Message, MessageID: statusID}
	}
	return field(FieldProps{Label: p.Label, IsLabelHidden: p.IsLabelHidden, Description: p.Description,
		InputID: id, LabelID: labelID, IsGroupLabel: true, DescriptionID: descID,
		IsOptional: p.IsOptional, IsRequired: p.IsRequired, IsDisabled: p.IsDisabled, Status: status,
		StatusVariant: "detached", Width: p.Width, Class: p.ContainerClass}, groupEl)
}

// NumberInputProps configures NumberInput.
type NumberInputProps struct {
	// ID of the visible <input>; defaults to "astryx-" + a slug of Label.
	ID    string
	Label string
	// Value is the committed number; nil is empty.
	Value         *float64
	IsLabelHidden bool
	Description   string
	IsOptional    bool
	IsRequired    bool
	// IsDisabled keeps the field focusable (readonly + aria-disabled) and drops its name.
	IsDisabled    bool
	IsReadOnly    bool
	StartIcon     string
	LabelIcon     string
	Status        *FieldStatus
	StatusVariant string
	// Size: "sm", "md" (default), "lg".
	Size         string
	Placeholder  string
	Width        string
	HasAutoFocus bool
	// Name is the form field name; the committed number is submitted from a hidden input.
	Name     string
	Min, Max *float64
	// Step is the arrow-key, wheel and stepper increment (default 1), on a grid anchored at Min.
	Step float64
	// Format is JSON Intl.NumberFormat options for the unfocused display, e.g.
	// `{"style":"currency","currency":"USD"}`. canon.js formats it; editing shows the plain number.
	Format string
	// IsWheelDisabled stops the wheel from stepping while the field is focused.
	IsWheelDisabled   bool
	HasNumberSteppers bool
	Units             string
	IsIntegerOnly     bool
	// HasClear lets emptying the field commit no value, and shows a clear button.
	HasClear bool
	// Locale (BCP 47) for parsing and formatting; the browser's by default.
	Locale string
	// Class goes on the Field container, ControlClass on the bordered row.
	Class, ControlClass string
	// Attrs go on the Field container, InputAttrs on the visible <input>, ValueAttrs on the
	// hidden <input> holding the committed number: bind it with {{"data-bind:qty", ""}}.
	Attrs, InputAttrs, ValueAttrs []Attr
	// Group places the field in an InputGroup row (InputGroupProps.Member).
	Group *InputGroupMember
}

func optNum(f *float64) string {
	if f == nil {
		return ""
	}
	return num(*f)
}

// NumberInput renders a text field with spinbutton semantics. canon.js treats typing as a
// draft that commits on blur or Enter, reads pasted local formats ("1,234.5", "(12)", "€5"),
// clamps to the range, shows "Invalid number" for unreadable text, and steps with the arrow
// keys, the wheel and the steppers. Each commit updates the hidden value input and fires
// input and change on it, so data-bind there tracks the number.
// Port of Astryx `NumberInput` (packages/core/src/NumberInput), MIT, Meta Platforms, Inc.
// Not ported: formatValue callbacks (use Format), disabledMessage, the "tooltip" status
// variant, onEntryErrorChange.
func NumberInput(p NumberInputProps) templ.Component {
	g := p.Group
	if g != nil {
		p.IsDisabled = p.IsDisabled || g.IsDisabled
		p.Size = or(p.Size, g.Size)
		p.Description = ""
	}
	id := controlID(p.ID, p.Label)
	statusID, unitsID, entryID := id+"-status", id+"-units", id+"-entry"
	descID := ""
	if p.Description != "" {
		descID = id + "-desc"
	}
	statusType, message := "", ""
	if p.Status != nil {
		statusType, message = p.Status.Type, p.Status.Message
	}
	errorMessage := statusType == "error" && message != ""
	labelledBy, groupDesc := "", ""
	if g != nil {
		labelledBy, groupDesc = joinIDs(g.LabelID, id+"-label"), g.DescribedBy
		message = ""
	}
	describedBy := joinIDs(groupDesc, descID, map[bool]string{true: statusID}[message != ""],
		map[bool]string{true: unitsID}[p.Units != ""])
	value := optNum(p.Value)
	mode := "decimal"
	if p.IsIntegerOnly {
		mode = "numeric"
	}
	box := newAttrs().set("class", "astryx-number-input-field").set("id", id).
		set("type", "text").set("inputmode", mode).set("role", "spinbutton").
		opt("value", value).
		opt("placeholder", p.Placeholder).
		flag("readonly", p.IsReadOnly || p.IsDisabled).
		when(p.IsDisabled, "aria-disabled", "true").
		opt("aria-valuemin", optNum(p.Min)).
		opt("aria-valuemax", optNum(p.Max)).
		opt("aria-valuenow", value).
		opt("aria-labelledby", labelledBy).
		opt("aria-describedby", describedBy).
		when(p.IsRequired && !p.IsOptional, "aria-required", "true").
		when(statusType == "error", "aria-invalid", "true").
		flag("autofocus", p.HasAutoFocus).
		extra(p.InputAttrs)
	hidden := newAttrs().set("type", "hidden").set("class", "astryx-number-input-value").
		when(!p.IsDisabled && p.Name != "", "name", p.Name).
		set("value", value).
		extra(p.ValueAttrs)
	var startIcon, groupLabel, units, clear, statusIcon, entryIcon, steppers templ.Component
	if p.StartIcon != "" {
		startIcon = Icon(IconProps{Name: p.StartIcon, Size: "sm", Color: "secondary"})
	}
	if g != nil {
		groupLabel = visuallyHidden(VisuallyHiddenProps{ID: id + "-label"}, text(p.Label))
	}
	if p.Units != "" {
		units = el("span", newAttrs().set("id", unitsID).set("class", "astryx-number-input-units"), text(p.Units))
	}
	alert := visuallyHidden(VisuallyHiddenProps{Class: "astryx-number-input-alert",
		Attrs: []Attr{{"role", "alert"}, {"aria-live", "assertive"}}}, nil)
	if p.HasClear && !p.IsDisabled && !p.IsReadOnly {
		a := []Attr{{"data-number-input-clear", ""}}
		if p.Value == nil {
			a = append(a, Attr{"hidden", "hidden"})
		}
		clear = Button(ButtonProps{Variant: "ghost", Size: "sm", Label: "Clear " + p.Label, Icon: "close",
			IsIconOnly: true, Class: "astryx-input-clear-button", Attrs: a})
	}
	if statusType != "" {
		statusIcon = Icon(IconProps{Name: statusType, Size: "md", Color: statusType, Class: "astryx-number-input-status-icon"})
	}
	if !errorMessage {
		entryIcon = Icon(IconProps{Name: "error", Size: "md", Color: "error", Class: "astryx-number-input-entry-icon"})
	}
	if p.HasNumberSteppers {
		inactive := p.IsDisabled || p.IsReadOnly
		btn := func(dir, name string) templ.Component {
			return el("button", newAttrs().set("type", "button").set("tabindex", "-1").
				set("class", "astryx-number-input-stepper").set("data-direction", dir).
				when(inactive, "aria-disabled", "true").
				set("aria-label", name+" "+p.Label),
				Icon(IconProps{Name: "numberInput:stepperDown", Size: "xsm"}))
		}
		steppers = el("div", newAttrs().set("class", "astryx-number-input-steppers"),
			btn("increment", "Increment"), btn("decrement", "Decrement"))
	}
	step := ""
	if p.Step > 0 {
		step = num(p.Step)
	}
	control := el("div", newAttrs().set("class", cls("astryx-number-input", p.ControlClass)).
		set("data-size", or(p.Size, "md")).
		opt("data-status", statusType).
		flag("data-disabled", p.IsDisabled).
		flag("data-readonly", p.IsReadOnly).
		flag("data-steppers", p.HasNumberSteppers).
		flag("data-error-message", errorMessage).
		when(g != nil, "data-astryx-input-group-item", "").
		opt("data-min", optNum(p.Min)).
		opt("data-max", optNum(p.Max)).
		opt("data-step", step).
		flag("data-integer", p.IsIntegerOnly).
		flag("data-clear", p.HasClear).
		flag("data-no-wheel", p.IsWheelDisabled).
		opt("data-locale", p.Locale).
		opt("data-format", p.Format),
		startIcon, groupLabel, void("input", box), void("input", hidden), units, alert, clear, statusIcon, entryIcon, steppers)
	if g != nil {
		return control
	}
	// "Invalid number" under the field while a rejected draft is on screen (canon.js shows it).
	var entry templ.Component
	if !errorMessage {
		entry = fieldStatusMessage(FieldStatus{Type: "error", Message: "Invalid number"}, p.StatusVariant, entryID,
			"astryx-number-input-entry-status", true)
	}
	var status *FieldStatus
	if p.Status != nil {
		status = &FieldStatus{Type: p.Status.Type, Message: p.Status.Message, MessageID: statusID}
	}
	return field(FieldProps{Label: p.Label, IsLabelHidden: p.IsLabelHidden, Description: p.Description,
		InputID: id, DescriptionID: descID, IsOptional: p.IsOptional, IsRequired: p.IsRequired,
		IsDisabled: p.IsDisabled, LabelIcon: p.LabelIcon, Status: status, StatusVariant: p.StatusVariant,
		Width: p.Width, Class: p.Class, Attrs: p.Attrs}, group(control, entry))
}

// SliderMark is a tick on a Slider's track, optionally labelled.
type SliderMark struct {
	Value float64
	Label string
}

// SliderProps configures Slider.
type SliderProps struct {
	// ID of the thumb (single value); defaults to "astryx-" + a slug of Label.
	ID    string
	Label string
	// Values holds one value, or two for a range.
	Values        []float64
	IsLabelHidden bool
	Description   string
	IsDisabled    bool
	IsOptional    bool
	IsRequired    bool
	Status        *FieldStatus
	Width         string
	// Min and Max default to 0 and 100; Step to 1.
	Min, Max *float64
	Step     float64
	// Orientation: "horizontal" (default) or "vertical".
	Orientation string
	// Format is JSON Intl.NumberFormat options for the shown value, e.g. `{"style":"percent"}`
	// (canon.js formats it; the server shows the plain number).
	Format string
	Locale string
	// ValueDisplay: "tooltip" (default, on hover, focus and drag), "text" (beside the track) or "none".
	ValueDisplay string
	Marks        []SliderMark
	// Name is the form field name each value is submitted under.
	Name string
	// Form is the id of a form outside this one that the values belong to.
	Form string
	// MinStepsBetweenThumbs is a range's smallest gap, in steps.
	MinStepsBetweenThumbs int
	// Class goes on the track container, ContainerClass on the Field container. Attrs go on the
	// thumb (single) or the group (range). ValueAttrs and EndValueAttrs go on the hidden inputs
	// holding the first and second value: bind them with {{"data-bind:volume", ""}}.
	// canon.js fires input on them while moving and change when a drag or key press ends.
	Class, ContainerClass            string
	Attrs, ValueAttrs, EndValueAttrs []Attr
}

const sliderThumb = 20.0

func round3(f float64) float64 { return math.Round(f*1000) / 1000 }

// Slider renders a single-value or range slider. canon.js drags the thumbs and jumps them to
// clicks on the track; arrow keys step, Page keys step by ten, Home and End jump to the bounds.
// Port of Astryx `Slider` (packages/core/src/Slider), MIT, Meta Platforms, Inc.
// Not ported: formatValue callbacks (use Format), disabledMessage, the touch controller.
func Slider(p SliderProps) templ.Component {
	id := controlID(p.ID, p.Label)
	labelID, descID, statusID, requiredID := id+"-label", "", "", id+"-required"
	if p.Description != "" {
		descID = id + "-desc"
	}
	if p.Status != nil && p.Status.Message != "" {
		statusID = id + "-status"
	}
	lo, hi := 0.0, 100.0
	if p.Min != nil {
		lo = *p.Min
	}
	if p.Max != nil {
		hi = *p.Max
	}
	step := p.Step
	if step == 0 {
		step = 1
	}
	orientation := or(p.Orientation, "horizontal")
	horizontal := orientation == "horizontal"
	display := or(p.ValueDisplay, "tooltip")
	clampV := func(v float64) float64 { return math.Min(math.Max(v, lo), hi) }
	values := []float64{lo}
	if len(p.Values) > 0 {
		values = []float64{clampV(p.Values[0])}
	}
	isRange := len(p.Values) >= 2
	if isRange {
		values = append(values, clampV(p.Values[1]))
	}
	percent := func(v float64) float64 {
		if hi == lo {
			return 0
		}
		return (v - lo) / (hi - lo) * 100
	}
	inset := func(pc float64) string {
		px := sliderThumb/2 - pc/100*sliderThumb
		sign := "+"
		if px < 0 {
			sign = "-"
		}
		return "calc(" + num(round3(pc)) + "% " + sign + " " + num(math.Abs(round3(px))) + "px)"
	}
	conveysRequired := p.IsRequired && !p.IsOptional
	describedBy := joinIDs(descID, statusID, map[bool]string{true: requiredID}[conveysRequired])
	invalid := p.Status != nil && p.Status.Type == "error"
	gap := float64(p.MinStepsBetweenThumbs) * step

	var hiddenInputs []templ.Component
	for i, v := range values {
		extra := p.ValueAttrs
		if i == 1 {
			extra = p.EndValueAttrs
		}
		hiddenInputs = append(hiddenInputs, void("input", newAttrs().set("type", "hidden").set("class", "astryx-slider-value").
			when(!p.IsDisabled && p.Name != "", "name", p.Name).opt("form", p.Form).set("value", num(v)).extra(extra)))
	}
	fillStart, fillEnd := 0.0, percent(values[len(values)-1])/100
	if isRange {
		fillStart = percent(values[0]) / 100
	}
	travel := []templ.Component{
		el("div", newAttrs().set("class", "astryx-slider-track").set("aria-hidden", "true")),
		el("div", newAttrs().set("class", "astryx-slider-fill-clip").set("aria-hidden", "true"),
			el("div", newAttrs().set("class", "astryx-slider-fill-mover").flag("data-range", isRange).
				set("style", "--_slider-fill-start: "+num(fillStart)),
				el("div", newAttrs().set("class", "astryx-slider-fill").flag("data-range", isRange).
					set("style", "--_slider-fill-start: "+num(fillStart)+"; --_slider-fill-end: "+num(fillEnd))))),
	}
	hasMarkLabels := false
	if len(p.Marks) > 0 {
		var marks []templ.Component
		for _, m := range p.Marks {
			pos := inset(percent(m.Value))
			style := "bottom: " + pos
			if horizontal {
				style = "inset-inline-start: " + pos
			}
			filled := m.Value <= values[0]
			if isRange {
				filled = m.Value >= values[0] && m.Value <= values[1]
			}
			marks = append(marks, el("div", newAttrs().set("class", "astryx-slider-mark").flag("data-filled", filled).
				set("data-mark-value", num(m.Value)).set("style", style)))
			if m.Label != "" {
				hasMarkLabels = hasMarkLabels || horizontal
				marks = append(marks, el("span", newAttrs().set("class", "astryx-slider-mark-label").
					set("data-mark-value", num(m.Value)).set("style", style), text(m.Label)))
			}
		}
		travel = append(travel, el("div", newAttrs().set("class", "astryx-slider-marks").set("aria-hidden", "true"), marks...))
	}
	for i, v := range values {
		progress := num(percent(v) / 100)
		vmin, vmax := lo, hi
		if isRange && i == 1 {
			vmin = clampV(values[0] + gap)
		}
		if isRange && i == 0 {
			vmax = clampV(values[1] - gap)
		}
		thumb := newAttrs()
		if !isRange {
			thumb.extra(p.Attrs).set("id", id)
		}
		thumb.set("role", "slider").set("tabindex", "0").set("class", "astryx-slider-thumb").
			set("data-orientation", orientation).
			set("style", "--_slider-progress: "+progress).
			set("aria-valuemin", num(vmin)).set("aria-valuemax", num(vmax)).set("aria-valuenow", num(v)).
			set("aria-orientation", orientation).
			when(p.IsDisabled, "aria-disabled", "true").
			when(invalid, "aria-invalid", "true")
		if isRange {
			thumb.set("aria-label", map[bool]string{true: "Minimum value", false: "Maximum value"}[i == 0])
		} else {
			thumb.set("aria-labelledby", labelID)
		}
		thumb.opt("aria-describedby", describedBy)
		if display == "tooltip" {
			travel = append(travel, el("span", newAttrs().set("class", "astryx-slider-anchor").set("aria-hidden", "true").
				set("style", "--_slider-progress: "+progress),
				el("span", newAttrs().set("class", "astryx-slider-tip"), text(num(v)))))
		}
		travel = append(travel, el("div", thumb))
	}
	control := newAttrs()
	if isRange {
		control.extra(p.Attrs)
	}
	control.set("class", cls("astryx-slider-control", p.Class)).
		set("data-orientation", orientation).
		flag("data-disabled", p.IsDisabled).
		set("data-min", num(lo)).set("data-max", num(hi)).set("data-step", num(step)).
		when(p.MinStepsBetweenThumbs > 0, "data-min-gap", strconv.Itoa(p.MinStepsBetweenThumbs)).
		opt("data-format", p.Format).
		opt("data-locale", p.Locale)
	if isRange {
		control.set("role", "group").set("aria-labelledby", labelID)
	}
	var textValue templ.Component
	if display == "text" {
		s := num(values[0])
		if isRange {
			s += " – " + num(values[1])
		}
		textValue = el("span", newAttrs().set("class", "astryx-slider-text"), text(s))
	}
	row := el("div", newAttrs().set("class", "astryx-slider").set("data-orientation", orientation).
		flag("data-disabled", p.IsDisabled).flag("data-mark-labels", hasMarkLabels),
		group(hiddenInputs...),
		el("div", control, el("div", newAttrs().set("class", "astryx-slider-travel"), travel...)),
		textValue)
	var required templ.Component
	if conveysRequired {
		required = visuallyHidden(VisuallyHiddenProps{ID: requiredID}, text("Required"))
	}
	var status *FieldStatus
	if p.Status != nil {
		status = &FieldStatus{Type: p.Status.Type, Message: p.Status.Message, MessageID: statusID}
	}
	return field(FieldProps{Label: p.Label, IsLabelHidden: p.IsLabelHidden, Description: p.Description,
		InputID: id, LabelID: labelID, IsGroupLabel: true, DescriptionID: descID,
		IsOptional: p.IsOptional, IsRequired: p.IsRequired, IsDisabled: p.IsDisabled, Status: status,
		StatusVariant: "detached", Width: p.Width, Class: p.ContainerClass}, group(row, required))
}

// FileInputProps configures FileInput.
type FileInputProps struct {
	// ID of the native file <input>; defaults to "astryx-" + a slug of Label.
	ID            string
	Label         string
	IsLabelHidden bool
	// Accept lists extensions (".pdf") and MIME types ("image/*"), comma-separated.
	Accept     string
	IsMultiple bool
	// MaxSize is the largest allowed file in bytes (0 = no limit); MaxFiles caps a multiple
	// selection (0 = no limit).
	MaxSize       int64
	MaxFiles      int
	IsDisabled    bool
	IsRequired    bool
	IsOptional    bool
	IsLoading     bool
	Status        *FieldStatus
	StatusVariant string
	Description   string
	Placeholder   string
	// Mode: "input" (default, a compact row) or "dropzone".
	Mode  string
	Width string
	// Name is the form field name; the native input submits the accepted files.
	Name string
	Form string
	// Class goes on the Field container, ControlClass on the box. InputAttrs go on the native
	// file <input> (data-bind:files works there: canon.js validates before Datastar reads it).
	Class, ControlClass string
	Attrs, InputAttrs   []Attr
}

// FileInput renders a compact file row or a drag-and-drop zone. canon.js checks files against
// Accept, MaxSize and MaxFiles, shows rejections as an error status, keeps the native input's
// files equal to the accepted ones, and announces the selection.
// Port of Astryx `FileInput` (packages/core/src/FileInput), MIT, Meta Platforms, Inc.
// Not ported: changeAction, disabledMessage, the "tooltip" status variant.
func FileInput(p FileInputProps) templ.Component {
	id := controlID(p.ID, p.Label)
	statusID, requiredID, errorID := id+"-status", id+"-required", id+"-error"
	descID := ""
	if p.Description != "" {
		descID = id + "-desc"
	}
	mode := or(p.Mode, "input")
	dropzone := mode == "dropzone"
	statusType, message := "", ""
	if p.Status != nil {
		statusType, message = p.Status.Type, p.Status.Message
	}
	conveysRequired := p.IsRequired && !p.IsOptional
	describedBy := joinIDs(descID, map[bool]string{true: statusID}[message != ""],
		map[bool]string{true: requiredID}[conveysRequired])
	placeholder := p.Placeholder
	if placeholder == "" {
		placeholder = "Choose file"
		if p.IsMultiple {
			placeholder = "Choose files"
		}
	}
	busy := p.IsDisabled || p.IsLoading
	trigger := visuallyHidden(VisuallyHiddenProps{}, el("button", newAttrs().set("type", "button").
		set("class", "astryx-file-input-trigger").
		when(busy, "aria-disabled", "true").
		set("aria-label", p.Label).
		when(p.IsLoading, "aria-busy", "true").
		opt("aria-describedby", describedBy)))
	native := void("input", newAttrs().set("id", id).set("type", "file").set("class", "astryx-file-input-native").
		when(!p.IsDisabled && p.Name != "", "name", p.Name).
		opt("accept", p.Accept).
		flag("multiple", p.IsMultiple).
		set("aria-hidden", "true").set("tabindex", "-1").
		when(p.IsDisabled, "form", "").
		when(!p.IsDisabled && p.Form != "", "form", p.Form).
		extra(p.InputAttrs))
	iconSize, spinSize := "sm", "sm"
	if dropzone {
		iconSize, spinSize = "md", "md"
	}
	var statusIcon, errorIcon, clear templ.Component
	if !dropzone && statusType != "" {
		statusIcon = Icon(IconProps{Name: statusType, Size: "md", Color: statusType, Class: "astryx-file-input-status-icon"})
	}
	if !dropzone && statusType == "" {
		errorIcon = Icon(IconProps{Name: "error", Size: "md", Color: "error", Class: "astryx-file-input-error-icon"})
	}
	if !p.IsDisabled {
		clear = Button(ButtonProps{Variant: "ghost", Size: "sm", Label: "Clear " + p.Label, Icon: "close",
			IsIconOnly: true, Class: "astryx-input-clear-button", Attrs: []Attr{{"data-file-input-clear", ""}}})
	}
	var parts []templ.Component
	parts = append(parts, trigger, native,
		Icon(IconProps{Name: "upload", Size: iconSize, Color: "secondary", Class: "astryx-file-input-upload"}))
	if dropzone {
		parts = append(parts,
			Spinner(SpinnerProps{Size: spinSize, Class: "astryx-file-input-spinner", Attrs: []Attr{{"aria-hidden", "true"}}}),
			el("div", newAttrs().set("class", "astryx-file-input-name")),
			el("span", newAttrs().set("class", "astryx-file-input-placeholder"), text(placeholder)))
	} else {
		parts = append(parts,
			el("span", newAttrs().set("class", "astryx-file-input-name")),
			el("span", newAttrs().set("class", "astryx-file-input-placeholder"), text(placeholder)),
			Spinner(SpinnerProps{Size: spinSize, Class: "astryx-file-input-spinner", Attrs: []Attr{{"aria-hidden", "true"}}}),
			statusIcon, errorIcon)
	}
	parts = append(parts, clear)
	box := el("div", newAttrs().set("class", cls("astryx-file-input", p.ControlClass)).
		set("data-mode", mode).
		opt("data-status", statusType).
		flag("data-disabled", p.IsDisabled).
		flag("data-busy", p.IsLoading).
		when(p.MaxSize > 0, "data-max-size", strconv.FormatInt(p.MaxSize, 10)).
		when(p.IsMultiple && p.MaxFiles > 0, "data-max-files", strconv.Itoa(p.MaxFiles)).
		set("data-label", p.Label), parts...)
	variant := p.StatusVariant
	if variant == "" && dropzone {
		variant = "detached"
	}
	// Validation errors appear here (canon.js fills and shows it) unless the caller set a status.
	var errorStatus, required templ.Component
	if p.Status == nil {
		errorStatus = fieldStatusMessage(FieldStatus{Type: "error"}, variant, errorID, "astryx-file-input-error", true)
	}
	live := visuallyHidden(VisuallyHiddenProps{Class: "astryx-file-input-live", Attrs: []Attr{{"aria-live", "polite"}}}, nil)
	if conveysRequired {
		required = visuallyHidden(VisuallyHiddenProps{ID: requiredID}, text("Required"))
	}
	var status *FieldStatus
	if p.Status != nil {
		status = &FieldStatus{Type: p.Status.Type, Message: p.Status.Message, MessageID: statusID}
	}
	return field(FieldProps{Label: p.Label, IsLabelHidden: p.IsLabelHidden, Description: p.Description,
		InputID: id, DescriptionID: descID, IsOptional: p.IsOptional, IsRequired: p.IsRequired,
		IsDisabled: p.IsDisabled, Status: status, StatusVariant: variant, Width: p.Width,
		Class: p.Class, Attrs: p.Attrs}, group(box, errorStatus, live, required))
}
