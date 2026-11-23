package canon

import (
	"strconv"
	"strings"

	"github.com/a-h/templ"
)

// Port of Astryx `MultiSelector` (packages/core/src/MultiSelector), MIT, Meta Platforms, Inc.
// A select-many control: a combobox button that opens a listbox of checkbox rows in a native
// popover. The values live in a hidden <select multiple> (bind an array signal to it with
// ValueAttrs, e.g. {"data-bind:tags", ""}); static/canon.js keeps the trigger and rows in step.
// The list stays open while you toggle. Selected options move to the top of their group when the
// list opens. HasSelectAll adds a "Select all" row; HasCreate (with HasSearch) offers the typed
// query as a new value: it's added to the select and the control fires a bubbling
// "astryx-create" event with detail {query}.
// Keyboard: ArrowDown/ArrowUp/Enter/Space open; arrows, Home/End and Page keys move; Enter or
// Space toggles; Escape and Tab close; typing jumps to a matching option.
// The value is in option order (the order of the select's options), not the order of picking.
// Not ported: option actions (grid mode), renderTrigger/formatValue, the bottom-sheet
// presentation, changeAction, disabledMessage and labelTooltip.

// MultiSelectorProps configures MultiSelector. Fields shared with SelectorProps mean the same.
type MultiSelectorProps struct {
	// ID of the trigger button; defaults to "astryx-" + a slug of Label.
	ID      string
	Label   string
	Options []SelectorOption
	// Value lists the selected values; values not among Options are kept (shown by value).
	Value []string
	// HasClear shows a clear (×) button while anything is selected.
	HasClear bool
	// HasSelectAll adds a row that selects (or clears) every enabled option that's shown.
	HasSelectAll bool
	// SelectAllLabel default "Select all".
	SelectAllLabel string
	HasSearch      bool
	// HasCreate offers the search query as a new value when no option has that label.
	HasCreate bool
	// TriggerDisplay: "count" (default, "3 selected"), "labels" ("A, B, C, +2") or "badges".
	TriggerDisplay string
	// MaxBadges caps the badges shown before "+N"; default 3.
	MaxBadges                                     int
	SearchPlaceholder, EmptyText, EmptySearchText string
	IsLabelHidden                                 bool
	Description                                   string
	IsOptional, IsRequired                        bool
	IsDisabled, IsReadOnly, IsLoading             bool
	// Placeholder shows while nothing is selected. Default "Select…".
	Placeholder   string
	Size, Variant string
	Status        *FieldStatus
	StatusVariant string
	Width         string
	StartIcon     string
	// IndicatorPosition: "start" (default) or "end" for the checkbox.
	IndicatorPosition string
	Placement         string
	IsDefaultOpen     bool
	// Name and Form go on the hidden select.
	Name, Form                      string
	Class, ControlClass             string
	Attrs, TriggerAttrs, ValueAttrs []Attr
	Group                           *InputGroupMember
}

// multiSelectorText is the trigger text for the "count" and "labels" displays.
func multiSelectorText(display string, labels []string, placeholder string) string {
	if len(labels) == 0 {
		return placeholder
	}
	if display == "labels" {
		shown := labels[:min(3, len(labels))]
		if rest := len(labels) - len(shown); rest > 0 {
			return strings.Join(shown, ", ") + ", +" + strconv.Itoa(rest)
		}
		return strings.Join(shown, ", ")
	}
	return strconv.Itoa(len(labels)) + " selected"
}

// MultiSelector renders the labelled control, its hidden select and the popover listbox.
func MultiSelector(p MultiSelectorProps) templ.Component {
	all := selectableOptions(p.Options)
	chosen := map[string]bool{}
	for _, v := range p.Value {
		chosen[v] = true
	}
	labelOf := map[string]string{}
	for _, o := range all {
		if _, ok := labelOf[o.Value]; !ok {
			labelOf[o.Value] = optionLabel(o)
		}
	}
	// The select's options: every option in order, then values not among them.
	var selectOpts []templ.Component
	var labels []string
	seen := map[string]bool{}
	addOption := func(value, label string) {
		if seen[value] {
			return
		}
		seen[value] = true
		on := chosen[value]
		if on {
			labels = append(labels, label)
		}
		selectOpts = append(selectOpts, el("option", newAttrs().set("value", value).flag("selected", on), text(label)))
	}
	for _, o := range all {
		addOption(o.Value, optionLabel(o))
	}
	for _, v := range p.Value {
		addOption(v, v)
	}
	display := or(p.TriggerDisplay, "count")
	maxBadges := p.MaxBadges
	if maxBadges <= 0 {
		maxBadges = 3
	}
	placeholder := or(p.Placeholder, "Select…")
	selectAllLabel := or(p.SelectAllLabel, "Select all")
	enabled, enabledOn := 0, 0
	for _, o := range all {
		if !o.IsDisabled {
			enabled++
			if chosen[o.Value] {
				enabledOn++
			}
		}
	}
	allOn := enabled > 0 && enabledOn == enabled
	selectAllState := "unchecked"
	if allOn {
		selectAllState = "checked"
	} else if enabledOn > 0 {
		selectAllState = "indeterminate"
	}
	indicator := or(p.IndicatorPosition, "start")
	// A disabled group disables its members: no name, so nothing is submitted.
	disabled := p.IsDisabled || (p.Group != nil && p.Group.IsDisabled)
	// Values not among Options (e.g. created ones) get rows too, so they can be unselected.
	options := p.Options
	for _, v := range p.Value {
		if _, ok := labelOf[v]; !ok {
			labelOf[v] = v
			options = append(options[:len(options):len(options)], SelectorOption{Value: v})
		}
	}
	return selectorShell(SelectorProps{ID: p.ID, Label: p.Label, IsLabelHidden: p.IsLabelHidden,
		Description: p.Description, IsOptional: p.IsOptional, IsRequired: p.IsRequired, IsDisabled: p.IsDisabled,
		IsReadOnly: p.IsReadOnly, IsLoading: p.IsLoading, Size: p.Size, Variant: p.Variant, Status: p.Status,
		StatusVariant: p.StatusVariant, Width: p.Width, StartIcon: p.StartIcon, HasSearch: p.HasSearch,
		SearchPlaceholder: p.SearchPlaceholder, Placement: p.Placement, IsDefaultOpen: p.IsDefaultOpen,
		Class: p.Class, ControlClass: p.ControlClass, Attrs: p.Attrs, TriggerAttrs: p.TriggerAttrs, Group: p.Group,
		HasClear: p.HasClear}, shellParts{
		class:    "astryx-multi-selector astryx-selector-control",
		multiple: true,
		hasValue: len(labels) > 0,
		clear:    "Clear all " + p.Label,
		triggerBody: func(ids selectorIDSet) templ.Component {
			a := newAttrs().set("id", ids.value).set("class", "astryx-selector-value-text").
				set("data-placeholder-text", placeholder).
				set("data-display", display).
				when(display == "badges", "data-max-badges", strconv.Itoa(maxBadges)).
				flag("data-badges", display == "badges" && len(labels) > 0)
			if display != "badges" || len(labels) == 0 {
				return el("span", a, text(multiSelectorText(display, labels, placeholder)))
			}
			var badges []templ.Component
			for _, l := range labels[:min(maxBadges, len(labels))] {
				badges = append(badges, Badge(BadgeProps{Label: l}))
			}
			if rest := len(labels) - maxBadges; rest > 0 {
				badges = append(badges, el("span", newAttrs().set("class", "astryx-multi-selector-overflow"),
					text("+"+strconv.Itoa(rest))))
			}
			return el("span", a, badges...)
		},
		value: el("select", newAttrs().flag("multiple", true).
			set("class", "astryx-multi-selector-value").
			flag("hidden", true).
			set("tabindex", "-1").
			set("aria-hidden", "true").
			when(!disabled && p.Name != "", "name", p.Name).
			when(!disabled && p.Form != "", "form", p.Form).
			extra(p.ValueAttrs), selectOpts...),
		listbox: func(ids selectorIDSet, size string) templ.Component {
			var lead []templ.Component
			if p.HasCreate && p.HasSearch {
				lead = append(lead, selectorRow(ids.listbox+"-create", SelectorOption{}, size, indicator, false, "none",
					Attr{"data-create", ""}, Attr{"hidden", "hidden"}))
			}
			if p.HasSelectAll {
				var extra []Attr
				extra = append(extra, Attr{"data-select-all", ""}, Attr{"data-state", selectAllState})
				if selectAllState == "indeterminate" {
					extra = append(extra, Attr{"aria-label", selectAllLabel + ", partially selected"})
				}
				if len(all) == 0 {
					extra = append(extra, Attr{"hidden", "hidden"})
				}
				lead = append(lead, selectorRow(ids.listbox+"-select-all", SelectorOption{Label: selectAllLabel},
					size, indicator, allOn, "checkbox", extra...))
			}
			return selectorListbox(ids, p.Label, lead, options, size, indicator,
				func(o SelectorOption) bool { return chosen[o.Value] },
				or(p.EmptyText, "No options"), or(p.EmptySearchText, "No results found"), "checkbox")
		},
	})
}
