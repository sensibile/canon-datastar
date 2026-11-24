package canon

import "testing"

func TestTypeaheadFixtures(t *testing.T) {
	items := []TypeaheadItem{
		{ID: "ada", Label: "Ada Lovelace", Description: "Analyst", Keywords: []string{"math", "engine"}},
		{ID: "alan", Label: "Alan Turing", Icon: "check"},
		{ID: "grace"},
	}
	g := InputGroupProps{ID: "grp", Label: "Group", Size: "sm"}
	gd := InputGroupProps{ID: "grpd", Label: "Group", IsDisabled: true}
	checkFixtures(t, "typeahead", []fixture{
		{"default", Typeahead(TypeaheadProps{Label: "Person", Items: items})},
		{"full", Typeahead(TypeaheadProps{ID: "p", Label: "Person", Items: items, Value: "alan", Placeholder: "Find",
			HasEntriesOnFocus: true, MaxMenuItems: 5, MinQueryLength: 2, EmptySearchText: "Nobody", Description: "Pick one",
			IsRequired: true, Status: &FieldStatus{Type: "error", Message: "Bad"}, Size: "lg", StartIcon: "search",
			Width: "300px", Name: "person", Form: "frm", Class: "c", ControlClass: "cc", Attrs: []Attr{{"data-x", "1"}},
			InputAttrs: []Attr{{"data-i", "1"}}, ValueAttrs: []Attr{{"data-bind:person", ""}}, HasAutoFocus: true})},
		{"remote", Typeahead(TypeaheadProps{ID: "r", Label: "Remote", IsRemote: true, DebounceMs: 300, Value: "x9",
			ValueLabel: "Xavier", NoClear: true, StatusVariant: "detached", Status: &FieldStatus{Type: "warning", Message: "Slow"}})},
		{"disabled", Typeahead(TypeaheadProps{ID: "d", Label: "Off", Items: items, Value: "ada", IsDisabled: true, Name: "off"})},
		{"group", inputGroup(g, Typeahead(TypeaheadProps{ID: "gt", Label: "Who", Items: items, Group: g.Member()}))},
		{"group-disabled", inputGroup(gd, Typeahead(TypeaheadProps{ID: "gtd", Label: "Who", Items: items, Name: "who",
			Value: "ada", Group: gd.Member()}))},
		{"results", TypeaheadResults(TypeaheadResultsProps{ID: "r", Query: "a\"b", Items: items})},
	})
	checkFixtures(t, "tokenizer", []fixture{
		{"default", Tokenizer(TokenizerProps{Label: "People", Items: items})},
		{"full", Tokenizer(TokenizerProps{ID: "t", Label: "People", Items: items,
			Value: []TypeaheadItem{{ID: "alan", Label: "Alan Turing"}, {ID: "zed", Label: "Zed"}}, MaxEntries: 3,
			HasCreate: true, HasClear: true, MenuWidth: "320px", HasEntriesOnFocus: true, Size: "sm", Description: "Add people",
			Status: &FieldStatus{Type: "success", Message: "Good"}, StartIcon: "search", Name: "people", Form: "frm",
			ValueAttrs: []Attr{{"data-bind:people", ""}}, InputAttrs: []Attr{{"data-i", "1"}}, IsOptional: true})},
		{"at-max", Tokenizer(TokenizerProps{ID: "m", Label: "Max", IsRemote: true, MaxEntries: 1,
			Value: []TypeaheadItem{{ID: "one"}}})},
		{"disabled", Tokenizer(TokenizerProps{ID: "d", Label: "Off", Items: items, IsDisabled: true, Name: "off",
			Value: []TypeaheadItem{{ID: "ada", Label: "Ada Lovelace"}}, HasClear: true})},
		{"group", inputGroup(g, Tokenizer(TokenizerProps{ID: "gk", Label: "Tags", Items: items, Group: g.Member()}))},
	})
}
