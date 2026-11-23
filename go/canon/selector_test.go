package canon

import "testing"

func TestSelectorFixtures(t *testing.T) {
	opts := []SelectorOption{
		{Value: "a", Label: "Apple", Description: "Crisp", Icon: "check"},
		{Value: "b"},
		{Value: "c", Label: "Cherry", IsDisabled: true},
	}
	sections := []SelectorOption{
		SelectorSection("Fruit", SelectorOption{Value: "a", Label: "Apple"}, SelectorOption{Value: "b", Label: "Banana"}),
		SelectorDivider(),
		SelectorSection("", SelectorOption{Value: "x", Label: "Other"}),
		{Value: "z", Label: "Zed"},
	}
	g := InputGroupProps{ID: "grp", Label: "Group", Size: "sm"}
	checkFixtures(t, "selector", []fixture{
		{"default", Selector(SelectorProps{Label: "Fruit", Options: opts})},
		{"full", Selector(SelectorProps{ID: "f", Label: "Fruit", Options: opts, Value: "a", HasClear: true,
			Description: "Pick one", IsRequired: true, Status: &FieldStatus{Type: "error", Message: "Bad"}, Size: "lg",
			Placeholder: "Choose", Width: "300px", IndicatorPosition: "start", Placement: "above", IsDefaultOpen: true,
			IsLoading: true, Name: "fruit", Form: "frm", Class: "c", ControlClass: "cc", Attrs: []Attr{{"data-x", "1"}},
			TriggerAttrs: []Attr{{"data-t", "1"}}, ValueAttrs: []Attr{{"data-bind:fruit", ""}}})},
		{"search-sections", Selector(SelectorProps{ID: "s", Label: "Region", Options: sections, HasSearch: true,
			StartIcon: "search", SearchPlaceholder: "Find", EmptyText: "Nothing", EmptySearchText: "No match",
			Variant: "secondary", Status: &FieldStatus{Type: "warning", Message: "Slow"}, IsOptional: true, IsRequired: true})},
		{"read-only", Selector(SelectorProps{ID: "r", Label: "Locked", Options: opts, Value: "b", IsReadOnly: true, HasClear: true})},
		{"disabled-empty", Selector(SelectorProps{ID: "d", Label: "Off", IsDisabled: true, Name: "off", Variant: "ghost",
			Status: &FieldStatus{Type: "error"}})},
		{"group", inputGroup(g, Selector(SelectorProps{ID: "gs", Label: "Unit", Options: opts, Group: g.Member()}))},
	})
}
