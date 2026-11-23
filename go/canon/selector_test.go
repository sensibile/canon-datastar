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
	gd := InputGroupProps{ID: "grpd", Label: "Group", IsDisabled: true}
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
		{"group-disabled", inputGroup(gd, Selector(SelectorProps{ID: "gd", Label: "Unit", Options: opts, Value: "a", Name: "unit",
			Group: gd.Member()}))},
	})
}

func TestMultiSelectorFixtures(t *testing.T) {
	opts := []SelectorOption{
		{Value: "a", Label: "Apple", Description: "Crisp"},
		{Value: "b", Label: "Banana"},
		{Value: "c", Label: "Cherry", IsDisabled: true},
		SelectorDivider(),
		SelectorSection("More", SelectorOption{Value: "d", Label: "Date"}, SelectorOption{Value: "e", Label: "Elder"}),
	}
	g := InputGroupProps{ID: "grp", Label: "Group"}
	gd := InputGroupProps{ID: "grpd", Label: "Group", IsDisabled: true}
	checkFixtures(t, "multiselector", []fixture{
		{"default", MultiSelector(MultiSelectorProps{Label: "Fruit", Options: opts})},
		{"labels", MultiSelector(MultiSelectorProps{ID: "m", Label: "Fruit", Options: opts, Value: []string{"a", "b", "d", "zz"},
			TriggerDisplay: "labels", HasClear: true, HasSelectAll: true, SelectAllLabel: "All", Name: "fruit", Form: "frm",
			ValueAttrs: []Attr{{"data-bind:fruits", ""}}, IndicatorPosition: "end", Size: "sm", IsRequired: true,
			Description: "Pick some", Status: &FieldStatus{Type: "error", Message: "Bad"}})},
		{"badges", MultiSelector(MultiSelectorProps{ID: "b", Label: "Tags", Options: opts, Value: []string{"a", "b", "d", "e"},
			TriggerDisplay: "badges", MaxBadges: 2, HasSearch: true, HasCreate: true, HasSelectAll: true, StartIcon: "search",
			Variant: "secondary", Placeholder: "Any"})},
		{"count-all", MultiSelector(MultiSelectorProps{ID: "c", Label: "Count", Options: opts, Value: []string{"a", "b", "d", "e"},
			HasSelectAll: true, IsReadOnly: true, HasClear: true})},
		{"disabled", MultiSelector(MultiSelectorProps{ID: "d", Label: "Off", Options: opts, Value: []string{"a"}, IsDisabled: true,
			Name: "off", TriggerDisplay: "badges"})},
		{"group", inputGroup(g, MultiSelector(MultiSelectorProps{ID: "gm", Label: "Unit", Options: opts, Group: g.Member()}))},
		{"group-disabled", inputGroup(gd, MultiSelector(MultiSelectorProps{ID: "gmd", Label: "Unit", Options: opts, Value: []string{"a"},
			Name: "units", Group: gd.Member()}))},
	})
}
