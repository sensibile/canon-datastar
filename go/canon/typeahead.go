package canon

import (
	"strconv"
	"strings"

	"github.com/a-h/templ"
)

// Ports of Astryx `Typeahead` and `Tokenizer` (packages/core/src/Typeahead, Tokenizer), MIT,
// Meta Platforms, Inc. Both are a search field over a results list in a popover.
//
// Results come from a static list (Items, filtered in the browser by label and keywords) or,
// with IsRemote, from the server: after DebounceMs the control fires a bubbling
// "astryx-search" event with detail {query}; answer it by patching TypeaheadResults (same ID)
// into the page, e.g. Attrs: {"data-on:astryx-search", "$q = evt.detail.query; @get('/search')"}.
// Answers for an older query are ignored.
//
// Typeahead picks one item: its id lives in a hidden input (bind a string signal with
// ValueAttrs) and shows as a token; click it (or press Backspace on it with NoClear) to edit.
// Tokenizer collects many: their ids live in a hidden <select multiple> (bind an array signal),
// in the order they were added; HasCreate offers the query as a new item and fires
// "astryx-create" with detail {query}.
// Keyboard: type to search; ArrowDown/ArrowUp wrap through results; Home/End; Enter picks;
// Escape closes; Backspace in an empty Tokenizer removes the last token.
// Not ported: renderItem/renderToken, the custom menu, result grouping, token overflow,
// disabledMessage and labelTooltip.

// TypeaheadItem is one searchable result.
type TypeaheadItem struct {
	ID    string
	Label string
	// Description is a second line under the label.
	Description string
	// Icon is a built-in glyph name shown before the label.
	Icon string
	// Keywords are extra strings the item matches on (static lists). They can't contain "|".
	Keywords []string
}

// searchProps holds what Typeahead and Tokenizer share.
type searchProps struct {
	ID, Label              string
	Items                  []TypeaheadItem
	IsRemote               bool
	DebounceMs             int
	Placeholder            string
	HasEntriesOnFocus      bool
	MaxMenuItems           int
	MinQueryLength         int
	EmptySearchText        string
	IsLabelHidden          bool
	Description            string
	IsOptional, IsRequired bool
	IsDisabled             bool
	HasAutoFocus           bool
	Size                   string
	Status                 *FieldStatus
	StatusVariant          string
	StartIcon              string
	Width                  string
	Name, Form             string
	Class, ControlClass    string
	Attrs, InputAttrs      []Attr
	Group                  *InputGroupMember
}

// TypeaheadProps configures Typeahead.
type TypeaheadProps struct {
	// ID of the search input; defaults to "astryx-" + a slug of Label. Derived ids: ID+"-listbox",
	// ID+"-popup", ID+"-listbox-item-N".
	ID    string
	Label string
	// Items is the static list searched in the browser. With IsRemote, Items are the entries
	// shown for an empty query (with HasEntriesOnFocus).
	Items    []TypeaheadItem
	IsRemote bool
	// DebounceMs delays the "astryx-search" event; default 150.
	DebounceMs int
	// Value is the picked item's id; ValueLabel its label when it isn't among Items.
	Value, ValueLabel string
	// Placeholder default "Search…".
	Placeholder string
	// HasEntriesOnFocus shows the entries for an empty query when the field gets focus.
	HasEntriesOnFocus bool
	// MaxMenuItems caps the results shown; default 10. MinQueryLength default 1.
	MaxMenuItems, MinQueryLength int
	// EmptySearchText default "No results found".
	EmptySearchText        string
	IsLabelHidden          bool
	Description            string
	IsOptional, IsRequired bool
	// IsDisabled makes the input read-only (aria-disabled) and drops the hidden input's name.
	IsDisabled bool
	// NoClear hides the clear (×) button shown while an item is picked.
	NoClear      bool
	HasAutoFocus bool
	// Size: "sm", "md" (default), "lg".
	Size   string
	Status *FieldStatus
	// StatusVariant: "attached" (default) or "detached".
	StatusVariant string
	StartIcon     string
	Width         string
	// Name and Form go on the hidden value input.
	Name, Form string
	// Class goes on the Field container, ControlClass on the bordered row.
	Class, ControlClass string
	// Attrs go on the Field container (put data-on:astryx-search here), InputAttrs on the search
	// input, ValueAttrs on the hidden value input.
	Attrs, InputAttrs, ValueAttrs []Attr
	Group                         *InputGroupMember
}

// TokenizerProps configures Tokenizer. Fields shared with TypeaheadProps mean the same.
type TokenizerProps struct {
	ID         string
	Label      string
	Items      []TypeaheadItem
	IsRemote   bool
	DebounceMs int
	// Value lists the chosen items, in order.
	Value []TypeaheadItem
	// MaxEntries caps the number of tokens (0: no cap).
	MaxEntries                   int
	Placeholder                  string
	HasEntriesOnFocus            bool
	MaxMenuItems, MinQueryLength int
	// MenuWidth sets the results list width (CSS length); default the field's width.
	MenuWidth       string
	EmptySearchText string
	// HasCreate offers the query itself as a new item.
	HasCreate bool
	// HasClear shows a "Clear all" button while there are tokens.
	HasClear               bool
	IsLabelHidden          bool
	Description            string
	IsOptional, IsRequired bool
	IsDisabled             bool
	HasAutoFocus           bool
	Size                   string
	Status                 *FieldStatus
	StatusVariant          string
	StartIcon              string
	Width                  string
	// Name and Form go on the hidden select.
	Name, Form                    string
	Class, ControlClass           string
	Attrs, InputAttrs, ValueAttrs []Attr
	Group                         *InputGroupMember
}

// TypeaheadResultsProps configures TypeaheadResults.
type TypeaheadResultsProps struct {
	// ID is the Typeahead's or Tokenizer's ID.
	ID string
	// Query is the query these results answer (as sent in the "astryx-search" event).
	Query string
	Items []TypeaheadItem
}

// TypeaheadResults renders a results list for a remote Typeahead or Tokenizer; patch it into
// the page (it replaces the element with id ID+"-listbox").
func TypeaheadResults(p TypeaheadResultsProps) templ.Component {
	return typeaheadListbox(p.ID+"-listbox", p.Items, &p.Query)
}

func typeaheadListbox(id string, items []TypeaheadItem, query *string) templ.Component {
	rows := []templ.Component{el("div", newAttrs().set("role", "option").set("aria-selected", "false").
		set("aria-disabled", "true").set("class", "astryx-typeahead-empty-state").flag("hidden", true), nil)}
	for i, it := range items {
		var icon, desc templ.Component
		if it.Icon != "" {
			icon = iconOrNil(it.Icon)
		}
		if it.Description != "" {
			desc = el("span", newAttrs().set("class", "astryx-typeahead-item-description"), text(it.Description))
		}
		label := or(it.Label, it.ID)
		rows = append(rows, el("div", newAttrs().set("id", id+"-item-"+strconv.Itoa(i)).
			set("role", "option").
			set("aria-selected", "false").
			set("tabindex", "-1").
			set("class", "astryx-typeahead-option").
			set("data-id", it.ID).
			set("data-label", label).
			opt("data-keywords", strings.Join(it.Keywords, "|")).
			flag("hidden", true),
			el("span", newAttrs().set("class", "astryx-typeahead-option-content"),
				el("div", newAttrs().set("class", "astryx-typeahead-item"), icon,
					el("div", newAttrs().set("class", "astryx-typeahead-item-content"),
						el("span", newAttrs().set("class", "astryx-typeahead-item-label"), text(label)), desc))),
			el("span", newAttrs().set("class", "astryx-typeahead-check"),
				Icon(IconProps{Name: "check", Size: "sm", Color: "primary"}))))
	}
	a := newAttrs().set("id", id).
		set("role", "listbox").
		set("aria-label", "Search results").
		set("tabindex", "-1").
		set("class", "astryx-typeahead-dropdown")
	if query != nil {
		a.set("data-query", *query)
	}
	return el("div", a, rows...)
}

// searchParts is what Typeahead and Tokenizer put into the shared control.
type searchParts struct {
	kind       string // "typeahead" or "tokenizer"
	hasValue   bool
	clearLabel string
	clear      bool
	// lead renders inside the control before the input (token lane / tokens).
	lead      func(ids searchIDSet, labelledBy, describedBy string) []templ.Component
	value     templ.Component
	extraCtl  func(a *attrs)
	extraIn   func(a *attrs)
	describe  []string
	afterIn   []templ.Component
	menuWidth string
	popupTail templ.Component
	// laneInput wraps the input with lead in a lane (Typeahead).
	laneInput bool
}

type searchIDSet struct {
	input, listbox, popup, label, inputLabel, desc, status, live string
}

func searchShell(p searchProps, parts searchParts) templ.Component {
	g := p.Group
	if g != nil {
		p.IsDisabled = p.IsDisabled || g.IsDisabled
		p.Size = or(p.Size, g.Size)
		p.Description = ""
	}
	size := or(p.Size, "md")
	id := controlID(p.ID, p.Label)
	ids := searchIDSet{input: id, listbox: id + "-listbox", popup: id + "-popup", label: id + "-label",
		inputLabel: id + "-input-label", desc: id + "-desc", status: id + "-status", live: id + "-live"}
	statusType, hasMessage := "", false
	if p.Status != nil {
		statusType, hasMessage = p.Status.Type, p.Status.Message != ""
	}
	descID := ""
	if p.Description != "" && g == nil {
		descID = ids.desc
	}
	labelledBy, groupDesc := "", ""
	if g != nil {
		labelledBy, groupDesc = joinIDs(g.LabelID, ids.inputLabel), g.DescribedBy
		hasMessage = false
	}
	describedBy := joinIDs(append([]string{groupDesc, descID, map[bool]string{true: ids.status}[hasMessage]}, parts.describe...)...)
	required := p.IsRequired && !p.IsOptional
	ia := newAttrs().set("id", id).
		set("type", "text").
		set("role", "combobox").
		set("class", "astryx-search-input").
		set("aria-expanded", "false").
		set("aria-controls", ids.listbox).
		set("aria-autocomplete", "list").
		opt("aria-labelledby", labelledBy).
		opt("aria-describedby", describedBy).
		when(required, "aria-required", "true").
		when(statusType == "error", "aria-invalid", "true").
		when(p.IsDisabled, "aria-disabled", "true").
		flag("readonly", p.IsDisabled).
		set("autocomplete", "off").
		set("autocorrect", "off").
		set("enterkeyhint", "search").
		set("spellcheck", "false").
		flag("autofocus", p.HasAutoFocus && !p.IsDisabled)
	if parts.extraIn != nil {
		parts.extraIn(ia)
	}
	ia.extra(p.InputAttrs)
	input := void("input", ia)
	var startIcon, groupLabel, clear, statusIcon templ.Component
	if p.StartIcon != "" {
		startIcon = el("span", newAttrs().set("class", "astryx-search-start"), iconOrNil(p.StartIcon))
	}
	if g != nil {
		groupLabel = visuallyHidden(VisuallyHiddenProps{ID: ids.inputLabel}, text(p.Label))
	}
	if parts.clear && !p.IsDisabled {
		a := []Attr{{"data-search-clear", ""}}
		if !parts.hasValue {
			a = append(a, Attr{"hidden", "hidden"})
		}
		clear = Button(ButtonProps{Variant: "ghost", Size: "sm", Label: parts.clearLabel, Icon: "close",
			IsIconOnly: true, Class: "astryx-input-clear-button", Attrs: a})
	}
	if statusType != "" && or(p.StatusVariant, "attached") != "detached" {
		statusIcon = Icon(IconProps{Name: statusType, Size: "md", Color: statusType})
	}
	spinner := el("span", newAttrs().set("class", "astryx-search-spinner").flag("hidden", true),
		Spinner(SpinnerProps{Size: "sm"}))
	end := el("span", newAttrs().set("class", "astryx-search-end"), spinner, clear, statusIcon)
	var body []templ.Component
	body = append(body, startIcon, groupLabel)
	lead := parts.lead(ids, joinIDs(or(labelledBy, ids.label)), describedBy)
	if parts.laneInput {
		body = append(body, el("div", newAttrs().set("class", "astryx-typeahead-lane"), append(append(lead, parts.value), input)...))
	} else {
		body = append(body, lead...)
		body = append(body, input)
		body = append(body, parts.afterIn...)
		body = append(body, parts.value)
	}
	live := visuallyHidden(VisuallyHiddenProps{ID: ids.live, Class: "astryx-search-live",
		Attrs: []Attr{{"aria-live", "polite"}, {"aria-atomic", "true"}}}, nil)
	var listQuery *string
	if p.IsRemote {
		q := ""
		listQuery = &q
	}
	popup := el("div", newAttrs().set("popover", "manual").
		set("id", ids.popup).
		set("class", "astryx-layer astryx-search-popup").
		set("data-placement", "below").
		set("data-alignment", "start").
		when(parts.menuWidth == "", "data-match-width", "").
		opt("style", decl("width", parts.menuWidth)).
		set("data-size", size),
		typeaheadListbox(ids.listbox, p.Items, listQuery), parts.popupTail)
	body = append(body, end, live, popup)
	ca := newAttrs().set("class", cls("astryx-"+parts.kind+" astryx-search-control", p.ControlClass)).
		set("data-size", size).
		opt("data-status", statusType).
		flag("data-disabled", p.IsDisabled).
		flag("data-remote", p.IsRemote).
		when(p.IsRemote, "data-debounce", strconv.Itoa(orInt(p.DebounceMs, 150))).
		set("data-max-items", strconv.Itoa(orInt(p.MaxMenuItems, 10))).
		set("data-min-query", strconv.Itoa(orInt(p.MinQueryLength, 1))).
		flag("data-entries-on-focus", p.HasEntriesOnFocus).
		set("data-empty-search-text", or(p.EmptySearchText, "No results found"))
	if parts.extraCtl != nil {
		parts.extraCtl(ca)
	}
	ca.when(g != nil, "data-astryx-input-group-item", "")
	control := el("div", ca, body...)
	if g != nil {
		return control
	}
	var status *FieldStatus
	if p.Status != nil {
		status = &FieldStatus{Type: p.Status.Type, Message: p.Status.Message, MessageID: ids.status}
	}
	return field(FieldProps{Label: p.Label, IsLabelHidden: p.IsLabelHidden, Description: p.Description,
		InputID: id, LabelID: ids.label, DescriptionID: descID, IsOptional: p.IsOptional, IsRequired: p.IsRequired,
		IsDisabled: p.IsDisabled, Status: status, StatusVariant: or(p.StatusVariant, "attached"), Width: p.Width,
		Class: p.Class, Attrs: p.Attrs}, control)
}

func orInt(v, def int) int {
	if v <= 0 {
		return def
	}
	return v
}

// Typeahead renders the labelled search field, its picked-item token and the results popover.
func Typeahead(p TypeaheadProps) templ.Component {
	disabled := p.IsDisabled || (p.Group != nil && p.Group.IsDisabled)
	label := p.ValueLabel
	if label == "" {
		for _, it := range p.Items {
			if it.ID == p.Value && p.Value != "" {
				label = or(it.Label, it.ID)
				break
			}
		}
	}
	label = or(label, p.Value)
	has := p.Value != ""
	sp := searchProps{ID: p.ID, Label: p.Label, Items: p.Items, IsRemote: p.IsRemote, DebounceMs: p.DebounceMs,
		Placeholder: p.Placeholder, HasEntriesOnFocus: p.HasEntriesOnFocus, MaxMenuItems: p.MaxMenuItems,
		MinQueryLength: p.MinQueryLength, EmptySearchText: p.EmptySearchText, IsLabelHidden: p.IsLabelHidden,
		Description: p.Description, IsOptional: p.IsOptional, IsRequired: p.IsRequired, IsDisabled: p.IsDisabled,
		// With a value the token shows, so it takes the autofocus.
		HasAutoFocus: p.HasAutoFocus && !has, Size: p.Size, Status: p.Status, StatusVariant: p.StatusVariant,
		StartIcon: p.StartIcon, Width: p.Width, Class: p.Class, ControlClass: p.ControlClass, Attrs: p.Attrs,
		InputAttrs: p.InputAttrs, Group: p.Group}
	size := or(p.Size, "")
	if size == "" && p.Group != nil {
		size = p.Group.Size
	}
	size = or(size, "md")
	return searchShell(sp, searchParts{
		kind:       "typeahead",
		hasValue:   has,
		clear:      !p.NoClear,
		clearLabel: "Clear selection",
		laneInput:  true,
		extraCtl:   func(a *attrs) { a.flag("data-has-value", has) },
		extraIn: func(a *attrs) {
			a.when(has, "tabindex", "-1").
				set("data-placeholder-text", or(p.Placeholder, "Search…")).
				when(!has, "placeholder", or(p.Placeholder, "Search…"))
		},
		lead: func(ids searchIDSet, labelledBy, describedBy string) []templ.Component {
			tokenValue := el("span", newAttrs().set("id", ids.input+"-token-value").flag("hidden", true), text(label))
			// The token's button is named by the field label and the value.
			action := []Attr{{"aria-labelledby", labelledBy + " " + ids.input + "-token-value"}}
			if describedBy != "" {
				action = append(action, Attr{"aria-describedby", describedBy})
			}
			if p.HasAutoFocus && has && !disabled {
				action = append(action, Attr{"autofocus", ""})
			}
			token := Token(TokenProps{Label: label, Size: size, IsDisabled: disabled, Action: action})
			wrap := el("span", newAttrs().set("class", "astryx-typeahead-token").flag("hidden", !has), token)
			return []templ.Component{tokenValue, wrap}
		},
		value: void("input", newAttrs().set("type", "hidden").
			set("class", "astryx-typeahead-value").
			when(!disabled && p.Name != "", "name", p.Name).
			when(!disabled && p.Form != "", "form", p.Form).
			set("value", p.Value).
			set("data-label", label).
			extra(p.ValueAttrs)),
	})
}

// Tokenizer renders the labelled search field with its tokens, hidden select and results popover.
func Tokenizer(p TokenizerProps) templ.Component {
	disabled := p.IsDisabled || (p.Group != nil && p.Group.IsDisabled)
	size := or(p.Size, "")
	if size == "" && p.Group != nil {
		size = p.Group.Size
	}
	size = or(size, "md")
	atMax := p.MaxEntries > 0 && len(p.Value) >= p.MaxEntries
	sp := searchProps{ID: p.ID, Label: p.Label, Items: p.Items, IsRemote: p.IsRemote, DebounceMs: p.DebounceMs,
		Placeholder: p.Placeholder, HasEntriesOnFocus: p.HasEntriesOnFocus, MaxMenuItems: p.MaxMenuItems,
		MinQueryLength: p.MinQueryLength, EmptySearchText: p.EmptySearchText, IsLabelHidden: p.IsLabelHidden,
		Description: p.Description, IsOptional: p.IsOptional, IsRequired: p.IsRequired, IsDisabled: p.IsDisabled,
		HasAutoFocus: p.HasAutoFocus, Size: p.Size, Status: p.Status, StatusVariant: p.StatusVariant,
		StartIcon: p.StartIcon, Width: p.Width, Class: p.Class, ControlClass: p.ControlClass, Attrs: p.Attrs,
		InputAttrs: p.InputAttrs, Group: p.Group}
	id := controlID(p.ID, p.Label)
	limitID := id + "-limit"
	tokenFor := func(label string) templ.Component {
		return Token(TokenProps{Label: label, Size: size, IsDisabled: disabled, IsRemovable: true})
	}
	// The select's options: chosen items first (in order, selected), then the other static items.
	var opts []templ.Component
	seen := map[string]bool{}
	var tokens []templ.Component
	for _, it := range p.Value {
		if seen[it.ID] {
			continue
		}
		seen[it.ID] = true
		l := or(it.Label, it.ID)
		opts = append(opts, el("option", newAttrs().set("value", it.ID).flag("selected", true), text(l)))
		tokens = append(tokens, el("span", newAttrs().set("class", "astryx-tokenizer-token").set("data-id", it.ID), tokenFor(l)))
	}
	for _, it := range p.Items {
		if seen[it.ID] {
			continue
		}
		seen[it.ID] = true
		opts = append(opts, el("option", newAttrs().set("value", it.ID), text(or(it.Label, it.ID))))
	}
	var describe []string
	if atMax {
		describe = []string{limitID}
	}
	var limit templ.Component
	if p.MaxEntries > 0 {
		limit = visuallyHidden(VisuallyHiddenProps{ID: limitID, Class: "astryx-tokenizer-limit"},
			text("Limit of "+strconv.Itoa(p.MaxEntries)+" reached"))
	}
	var tail []templ.Component
	tail = append(tail, el("template", newAttrs().set("class", "astryx-tokenizer-token-template"),
		el("span", newAttrs().set("class", "astryx-tokenizer-token"), tokenFor(""))))
	if p.HasCreate {
		tail = append(tail, el("template", newAttrs().set("class", "astryx-tokenizer-create-template"),
			el("div", newAttrs().set("role", "option").set("aria-selected", "false").set("tabindex", "-1").
				set("class", "astryx-typeahead-option").set("data-create", ""),
				el("span", newAttrs().set("class", "astryx-typeahead-option-content"),
					el("div", newAttrs().set("class", "astryx-typeahead-item"),
						el("div", newAttrs().set("class", "astryx-typeahead-item-content"),
							el("span", newAttrs().set("class", "astryx-typeahead-item-label"), nil)))))))
	}
	placeholder := or(p.Placeholder, "Search…")
	return searchShell(sp, searchParts{
		kind:       "tokenizer",
		hasValue:   len(tokens) > 0,
		clear:      p.HasClear,
		clearLabel: "Clear all",
		describe:   describe,
		menuWidth:  p.MenuWidth,
		popupTail:  group(tail...),
		extraCtl: func(a *attrs) {
			a.flag("data-has-tokens", len(tokens) > 0).
				when(p.MaxEntries > 0, "data-max-entries", strconv.Itoa(p.MaxEntries)).
				flag("data-at-max", atMax).
				flag("data-create", p.HasCreate)
		},
		extraIn: func(a *attrs) {
			a.set("data-placeholder-text", placeholder).
				when(len(tokens) == 0, "placeholder", placeholder).
				flag("data-at-max", atMax)
			if atMax && !disabled {
				a.flag("readonly", true)
			}
		},
		lead:    func(searchIDSet, string, string) []templ.Component { return tokens },
		afterIn: []templ.Component{limit},
		value: el("select", newAttrs().flag("multiple", true).
			set("class", "astryx-tokenizer-value").
			flag("hidden", true).
			set("tabindex", "-1").
			set("aria-hidden", "true").
			when(!disabled && p.Name != "", "name", p.Name).
			when(!disabled && p.Form != "", "form", p.Form).
			extra(p.ValueAttrs), opts...),
	})
}
