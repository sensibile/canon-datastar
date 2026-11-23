package canon

import (
	"strconv"

	"github.com/a-h/templ"
)

// Port of Astryx `Selector` (packages/core/src/Selector), MIT, Meta Platforms, Inc.
// A select-one control: a combobox button that opens a listbox of options in a native popover.
// The value lives in a hidden input (bind it with ValueAttrs, e.g. {"data-bind:plan", ""});
// static/canon.js keeps the trigger text and the selected row in step with it.
// Keyboard: ArrowDown/ArrowUp/Enter/Space open; arrows, Home/End and Page keys move; Enter or
// Space picks; Escape and Tab close. Typing a label's first letters picks it while closed and
// highlights it while open. With HasSearch, a search field heads the list and filters it.
// Not ported: the bottom-sheet presentation on touch, positioning the list over the selected
// option, changeAction, renderOption/renderValue, disabledMessage and labelTooltip.

// SelectorOption is one entry of a Selector's Options: an option, a divider (SelectorDivider)
// or a titled section of options (SelectorSection).
type SelectorOption struct {
	Value string
	// Label is the shown text; it defaults to Value. For a section it is the title.
	Label       string
	Description string
	// Icon is a built-in glyph name shown before the label (and in the trigger when picked).
	Icon       string
	IsDisabled bool
	IsDivider  bool
	IsSection  bool
	// Options are a section's options.
	Options []SelectorOption
}

// SelectorDivider is a line between options; it is hidden while searching.
func SelectorDivider() SelectorOption { return SelectorOption{IsDivider: true} }

// SelectorSection groups options under an optional title.
func SelectorSection(title string, options ...SelectorOption) SelectorOption {
	return SelectorOption{IsSection: true, Label: title, Options: options}
}

// SelectorProps configures Selector.
type SelectorProps struct {
	// ID of the trigger button; defaults to "astryx-" + a slug of Label. Derived ids: ID+"-listbox",
	// ID+"-popup", ID+"-listbox-option-N" (N counts options in order, sections included).
	ID      string
	Label   string
	Options []SelectorOption
	// Value is the selected option's value; "" selects nothing.
	Value string
	// HasClear shows a clear (×) button while there's a value.
	HasClear      bool
	IsLabelHidden bool
	Description   string
	IsOptional    bool
	IsRequired    bool
	// IsDisabled keeps the trigger focusable (aria-disabled) and drops the hidden input's name.
	IsDisabled bool
	IsReadOnly bool
	IsLoading  bool
	// Placeholder shows while nothing is selected. Default "Select…".
	Placeholder string
	// Size: "sm", "md" (default), "lg". Variant: "input" (default), "secondary", "ghost".
	Size, Variant string
	Status        *FieldStatus
	// StatusVariant: "attached" (default) or "detached"; non-input variants are always detached.
	StatusVariant string
	Width         string
	// StartIcon is a built-in glyph name.
	StartIcon string
	// IndicatorPosition: "end" (default) or "start" for the check mark on the selected row.
	IndicatorPosition string
	HasSearch         bool
	// SearchPlaceholder default "Search…"; EmptyText "No options"; EmptySearchText "No results found".
	SearchPlaceholder, EmptyText, EmptySearchText string
	// Placement: "below" (default) or "above".
	Placement     string
	IsDefaultOpen bool
	// Name and Form go on the hidden value input.
	Name, Form string
	// Class goes on the Field container, ControlClass on the bordered row.
	Class, ControlClass string
	// Attrs go on the Field container, TriggerAttrs on the button, ValueAttrs on the hidden input.
	Attrs, TriggerAttrs, ValueAttrs []Attr
	// Group places the selector in an InputGroup row (InputGroupProps.Member).
	Group *InputGroupMember
}

// selectorEntry is an option flattened out of Options, with its position for ids.
type selectorEntry struct {
	opt   SelectorOption
	index int
}

func optionLabel(o SelectorOption) string { return or(o.Label, o.Value) }

// selectableOptions lists every option (sections flattened) in display order.
func selectableOptions(options []SelectorOption) []SelectorOption {
	var out []SelectorOption
	for _, o := range options {
		switch {
		case o.IsDivider:
		case o.IsSection:
			out = append(out, o.Options...)
		default:
			out = append(out, o)
		}
	}
	return out
}

// Selector renders the labelled control, its hidden value input and the popover listbox.
func Selector(p SelectorProps) templ.Component {
	var selected *SelectorOption
	all := selectableOptions(p.Options)
	for i := range all {
		if p.Value != "" && all[i].Value == p.Value {
			selected = &all[i]
			break
		}
	}
	placeholder := or(p.Placeholder, "Select…")
	return selectorShell(p, shellParts{
		class:    "astryx-selector astryx-selector-control",
		hasValue: selected != nil,
		clear:    "Clear " + p.Label,
		triggerBody: func(ids selectorIDSet) templ.Component {
			var valueIcon templ.Component
			valueText := placeholder
			if selected != nil {
				valueText = optionLabel(*selected)
			}
			if p.StartIcon == "" {
				icon := ""
				if selected != nil {
					icon = selected.Icon
				}
				// Kept (hidden while empty) so canon.js can show a picked option's icon.
				valueIcon = el("span", newAttrs().set("class", "astryx-selector-value-icon").flag("hidden", icon == ""),
					iconOrNil(icon))
			}
			return group(valueIcon, el("span", newAttrs().set("id", ids.value).set("class", "astryx-selector-value-text").
				set("data-placeholder-text", placeholder), text(valueText)))
		},
		value: void("input", newAttrs().set("type", "hidden").
			set("class", "astryx-selector-value").
			when(!p.IsDisabled && p.Name != "", "name", p.Name).
			when(!p.IsDisabled && p.Form != "", "form", p.Form).
			set("value", p.Value).
			extra(p.ValueAttrs)),
		listbox: func(ids selectorIDSet, size string) templ.Component {
			return selectorListbox(ids, p.Label, nil, p.Options, size, p.IndicatorPosition,
				func(o SelectorOption) bool { return p.Value != "" && o.Value == p.Value },
				or(p.EmptyText, "No options"), or(p.EmptySearchText, "No results found"), "check")
		},
	})
}

// shellParts is what Selector and MultiSelector put into the shared control.
type shellParts struct {
	class       string
	multiple    bool
	hasValue    bool
	clear       string
	triggerBody func(ids selectorIDSet) templ.Component
	value       templ.Component
	listbox     func(ids selectorIDSet, size string) templ.Component
}

// selectorShell renders the Field, the bordered control with its trigger and the popup.
func selectorShell(p SelectorProps, parts shellParts) templ.Component {
	g := p.Group
	if g != nil {
		p.IsDisabled = p.IsDisabled || g.IsDisabled
		p.Size = or(p.Size, g.Size)
		p.Description = ""
	}
	variant := or(p.Variant, "input")
	size := or(p.Size, "md")
	statusVariant := or(p.StatusVariant, "attached")
	if variant != "input" {
		statusVariant = "detached"
	}
	readOnly := p.IsReadOnly && !p.IsDisabled
	required := p.IsRequired && !p.IsOptional
	id := controlID(p.ID, p.Label)
	ids := selectorIDs(id)
	statusType, hasMessage := "", false
	if p.Status != nil {
		statusType, hasMessage = p.Status.Type, p.Status.Message != ""
	}
	descID := ""
	if p.Description != "" && g == nil {
		descID = ids.desc
	}
	labelledBy, groupDesc := ids.label, ""
	if g != nil {
		labelledBy, groupDesc = joinIDs(g.LabelID, ids.inputLabel), g.DescribedBy
		hasMessage = false
	}
	combobox := readOnly || !p.HasSearch
	if p.HasSearch && !readOnly {
		labelledBy = joinIDs(labelledBy, ids.value)
	}
	trigger := newAttrs().set("id", id).
		set("type", "button").
		set("class", "astryx-selector-trigger").
		when(combobox, "role", "combobox").
		when(!readOnly, "aria-haspopup", "listbox").
		set("aria-expanded", "false").
		when(!readOnly, "aria-controls", ids.listbox).
		when(readOnly, "aria-readonly", "true").
		set("aria-labelledby", labelledBy).
		opt("aria-describedby", joinIDs(groupDesc, descID, map[bool]string{true: ids.status}[hasMessage], map[bool]string{true: ids.readOnly}[readOnly])).
		when(required && combobox, "aria-required", "true").
		when(statusType == "error", "aria-invalid", "true").
		when(p.IsLoading, "aria-busy", "true").
		when(p.IsDisabled, "aria-disabled", "true").
		extra(p.TriggerAttrs)
	triggerBtn := el("button", trigger, parts.triggerBody(ids))
	var startIcon, groupLabel, readOnlyText, spinner, clear, end templ.Component
	if p.StartIcon != "" {
		startIcon = Icon(IconProps{Name: p.StartIcon, Size: "sm", Color: "secondary"})
	}
	if g != nil {
		groupLabel = visuallyHidden(VisuallyHiddenProps{ID: ids.inputLabel}, text(p.Label))
	}
	if readOnly {
		readOnlyText = visuallyHidden(VisuallyHiddenProps{ID: ids.readOnly}, text("Read only"))
	}
	if p.IsLoading {
		spinner = Spinner(SpinnerProps{Size: "sm", Attrs: []Attr{{"aria-hidden", "true"}}})
	}
	if p.HasClear && !p.IsDisabled && !readOnly {
		a := []Attr{{"data-selector-clear", ""}, {"tabindex", "-1"}}
		if !parts.hasValue {
			a = append(a, Attr{"hidden", "hidden"})
		}
		clear = Button(ButtonProps{Variant: "ghost", Size: "sm", Label: parts.clear, Icon: "close",
			IsIconOnly: true, Class: "astryx-input-clear-button", Attrs: a})
	}
	if statusType != "" && statusVariant != "detached" {
		end = Icon(IconProps{Name: statusType, Size: "sm", Color: statusType})
	} else if !readOnly {
		end = el("span", newAttrs().set("class", "astryx-selector-indicator-icon").set("data-state", "collapsed"),
			Icon(IconProps{Name: "chevronDown", Size: "sm", Color: "secondary"}))
	}
	live := visuallyHidden(VisuallyHiddenProps{ID: id + "-live", Class: "astryx-selector-live", Attrs: []Attr{{"aria-live", "polite"}, {"aria-atomic", "true"}}}, nil)
	// Rendered even when read-only: canon.js reads option labels from it for bound values.
	popup := selectorPopup(ids, p.Label, variant, size, or(p.Placement, "below"), p.HasSearch, required,
		or(p.SearchPlaceholder, "Search…"), parts.listbox(ids, size))
	control := el("div", newAttrs().set("class", cls(parts.class, p.ControlClass)).
		set("data-variant", variant).
		set("data-size", size).
		opt("data-status", statusType).
		flag("data-disabled", p.IsDisabled).
		flag("data-readonly", readOnly).
		flag("data-placeholder", !parts.hasValue).
		flag("data-search", p.HasSearch).
		flag("data-multiple", parts.multiple).
		flag("data-default-open", p.IsDefaultOpen && !p.IsDisabled && !readOnly).
		when(g != nil, "data-astryx-input-group-item", ""),
		startIcon, groupLabel, readOnlyText, triggerBtn, parts.value, spinner, clear, end, live, popup)
	if g != nil {
		return control
	}
	var status *FieldStatus
	if p.Status != nil {
		status = &FieldStatus{Type: p.Status.Type, Message: p.Status.Message, MessageID: ids.status}
	}
	return field(FieldProps{Label: p.Label, IsLabelHidden: p.IsLabelHidden, Description: p.Description,
		InputID: id, LabelID: ids.label, DescriptionID: descID, IsOptional: p.IsOptional, IsRequired: p.IsRequired,
		IsDisabled: p.IsDisabled, Status: status, StatusVariant: statusVariant, Width: p.Width,
		Class: p.Class, Attrs: p.Attrs}, control)
}

type selectorIDSet struct {
	listbox, popup, label, inputLabel, value, desc, status, readOnly, search string
}

func selectorIDs(id string) selectorIDSet {
	return selectorIDSet{listbox: id + "-listbox", popup: id + "-popup", label: id + "-label",
		inputLabel: id + "-input-label", value: id + "-value", desc: id + "-desc", status: id + "-status",
		readOnly: id + "-readonly", search: id + "-search"}
}

func iconOrNil(name string) templ.Component {
	if name == "" {
		return nil
	}
	return Icon(IconProps{Name: name, Size: "sm", Color: "secondary"})
}

// selectorPopup is the popover holding the optional search row and the listbox.
func selectorPopup(ids selectorIDSet, label, variant, size, placement string, hasSearch, required bool,
	searchPlaceholder string, listbox templ.Component) templ.Component {
	wrap := el("div", newAttrs().set("class", "astryx-selector-listbox-wrap").flag("data-input", variant == "input"), listbox)
	var body templ.Component = wrap
	if hasSearch {
		search := void("input", newAttrs().set("id", ids.search).
			set("class", "astryx-selector-search").
			set("type", "text").
			set("role", "combobox").
			set("aria-label", "Search options").
			when(required, "aria-required", "true").
			set("aria-expanded", "true").
			set("aria-controls", ids.listbox).
			set("aria-autocomplete", "list").
			set("autocomplete", "off").
			set("spellcheck", "false").
			set("placeholder", searchPlaceholder))
		clear := Button(ButtonProps{Variant: "ghost", Size: "sm", Label: "Clear search", Icon: "close",
			IsIconOnly: true, Class: "astryx-selector-search-clear", Attrs: []Attr{{"hidden", "hidden"}}})
		body = el("div", newAttrs().set("class", "astryx-selector-search-panel"),
			el("div", newAttrs().set("class", "astryx-selector-search-row"),
				Icon(IconProps{Name: "search", Size: "sm", Color: "secondary"}), search, clear),
			Divider(DividerProps{}), wrap)
	}
	return el("div", newAttrs().set("popover", "auto").
		set("id", ids.popup).
		set("class", "astryx-layer astryx-selector-popup").
		set("data-placement", placement).
		set("data-alignment", "start").
		set("data-match-width", "").
		set("data-size", size), body)
}

// selectorListbox renders the rows; mark is "check" (Selector) or "checkbox" (MultiSelector).
func selectorListbox(ids selectorIDSet, label string, lead []templ.Component, options []SelectorOption, size, indicator string,
	isSelected func(SelectorOption) bool, emptyText, emptySearchText, mark string) templ.Component {
	index := 0
	row := func(o SelectorOption) templ.Component {
		r := selectorRow(ids.listbox+"-option-"+strconv.Itoa(index), o, size, indicator, isSelected(o), mark)
		index++
		return r
	}
	var rows []templ.Component
	for _, o := range options {
		switch {
		case o.IsDivider:
			rows = append(rows, Divider(DividerProps{Class: "astryx-selector-divider", Attrs: []Attr{{"aria-hidden", "true"}}}))
		case o.IsSection:
			var heading templ.Component
			if o.Label != "" {
				heading = el("div", newAttrs().set("class", "astryx-selector-section-heading").set("aria-hidden", "true"), text(o.Label))
			}
			inner := []templ.Component{heading}
			for _, so := range o.Options {
				inner = append(inner, row(so))
			}
			rows = append(rows, el("div", newAttrs().set("role", "group").opt("aria-label", o.Label).
				set("class", "astryx-selector-section"), inner...))
		default:
			rows = append(rows, row(o))
		}
	}
	empty := el("div", newAttrs().set("class", "astryx-selector-empty-state").set("role", "presentation").
		set("aria-live", "polite").flag("hidden", index > 0), text(emptyText))
	return el("div", newAttrs().set("id", ids.listbox).
		set("role", "listbox").
		set("aria-label", label).
		when(mark == "checkbox", "aria-multiselectable", "true").
		set("tabindex", "-1").
		set("class", "astryx-selector-listbox").
		set("data-empty-text", emptyText).
		set("data-empty-search-text", emptySearchText),
		append(append(lead, empty), rows...)...)
}

func selectorRow(id string, o SelectorOption, size, indicator string, selected bool, mark string, extra ...Attr) templ.Component {
	var markEl templ.Component
	switch mark {
	case "none":
	case "checkbox":
		markEl = el("span", newAttrs().set("class", "astryx-selector-option-mark"),
			el("span", newAttrs().set("class", "astryx-checkbox-indicator").set("data-size", "sm").set("aria-hidden", "true"),
				raw(checkMark)))
	default:
		markEl = el("span", newAttrs().set("class", "astryx-selector-option-mark"),
			el("span", newAttrs().set("class", "astryx-check-indicator").set("aria-hidden", "true"),
				Icon(IconProps{Name: "check", Size: "sm"})))
	}
	var desc templ.Component
	if o.Description != "" {
		desc = el("span", newAttrs().set("class", "astryx-selector-option-description"), text(o.Description))
	}
	content := el("span", newAttrs().set("class", "astryx-selector-option-content"),
		iconOrNil(o.Icon),
		el("span", newAttrs().set("class", "astryx-selector-option"),
			el("span", newAttrs().set("class", "astryx-selector-option-label"), text(optionLabel(o))), desc))
	start, end := markEl, templ.Component(nil)
	if indicator != "start" {
		start, end = nil, markEl
	}
	return el("div", newAttrs().set("id", id).
		set("role", "option").
		set("aria-selected", strconv.FormatBool(selected)).
		when(o.IsDisabled, "aria-disabled", "true").
		set("class", "astryx-selector-option-row").
		set("data-value", o.Value).
		opt("data-icon", o.Icon).
		set("data-mark", mark).
		set("data-size", size).
		extra(extra), start, content, end)
}
