package canon

import "testing"

func fp(v float64) *float64 { return &v }

func TestFormsFixtures(t *testing.T) {
	no := false
	checkFixtures(t, "text-area", []fixture{
		{"default", TextArea(TextAreaProps{Label: "Bio"})},
		{"full", TextArea(TextAreaProps{ID: "bio", Label: "Bio", Value: "\nHi \"you\"", Description: "Short", IsRequired: true,
			StartIcon: "info", Status: &FieldStatus{Type: "warning", Message: "Long"}, Size: "lg", IsLoading: true,
			Placeholder: "Tell us", Rows: 5, MaxLength: 4, HTMLMaxLength: 2000, HasAutoFocus: true, Name: "bio",
			HasSpellCheck: &no, Class: "c", ControlClass: "cc", Attrs: []Attr{{"data-x", "1"}},
			InputAttrs: []Attr{{"data-bind:bio", ""}}})},
		{"disabled-messageless", TextArea(TextAreaProps{ID: "n", Label: "Notes", IsDisabled: true, Name: "n",
			Status: &FieldStatus{Type: "error"}, MaxLength: 1000, Value: "abc"})},
	})
	checkFixtures(t, "item", []fixture{
		{"default", Item(ItemProps{Label: "Row"})},
		{"button", Item(ItemProps{Label: "Row", Description: "More", Action: []Attr{{"data-on:click", "$n++"}},
			StartContent: child, EndContent: child, Align: "first-line", Density: "compact", IsSelected: true,
			LabelLines: 2, DescriptionLines: 1, Class: "x", Attrs: []Attr{{"data-y", "1"}}})},
		{"link-disabled", Item(ItemProps{As: "li", Label: "Docs", Href: "/docs", Target: "_blank", IsDisabled: true,
			Layout: "inline", Description: "inline"})},
		{"link", Item(ItemProps{Label: "Docs", Href: "/docs", Target: "_blank", IsHighlighted: true})},
		{"role", Item(ItemProps{Label: "Opt", Role: "option", IsSelected: true, IsDisabled: true, LabelContent: child,
			Density: "spacious", Align: "start"})},
		{"disabled-button", Item(ItemProps{Label: "Row", Action: []Attr{{"data-on:click", "$n++"}, {"data-x", "1"}}, IsDisabled: true})},
	})
	checkFixtures(t, "checkbox-list", []fixture{
		{"default", CheckboxList(CheckboxListProps{Label: "Toppings", Items: []CheckboxListItem{{Label: "Cheese"}, {Label: "Ham"}}})},
		{"full", CheckboxList(CheckboxListProps{ID: "t", Label: "Toppings", Description: "Pick", IsRequired: true,
			Status: &FieldStatus{Type: "error", Message: "Pick one"}, Name: "top", Density: "compact", HasDividers: true,
			Width: "300px", Class: "c", Attrs: []Attr{{"data-x", "1"}}, InputAttrs: []Attr{{"data-bind:top", ""}},
			Items: []CheckboxListItem{
				{Label: "Olives", Value: "olives", IsDisabled: true},
				{Label: "Cheese", Value: "cheese", Description: "Mozzarella", EndContent: child, Align: "start", Class: "i",
					Attrs: []Attr{{"data-r", "1"}}, InputAttrs: []Attr{{"data-i", "1"}}},
				{LabelContent: child, Value: "rich", IsIndeterminate: true, IsLoading: true, ID: "rich"},
			}})},
		{"read-only-checked", CheckboxList(CheckboxListProps{Label: "Read", IsReadOnly: true, IsRequired: true, IsDisabled: true,
			Items: []CheckboxListItem{{Label: "A", IsChecked: true, AriaLabel: "Alpha"}}})},
	})
	checkFixtures(t, "radio-list", []fixture{
		{"default", RadioList(RadioListProps{Label: "Plan", Items: []RadioListItem{{Label: "Free", Value: "free"}, {Label: "Pro", Value: "pro"}}})},
		{"full", RadioList(RadioListProps{ID: "p", Label: "Plan", Description: "Billing", Value: "pro", Orientation: "horizontal",
			Name: "plan", IsRequired: true, Status: &FieldStatus{Type: "error", Message: "Pick"}, Size: "sm", Width: "20rem",
			Class: "c", ContainerClass: "cc", Attrs: []Attr{{"data-x", "1"}}, InputAttrs: []Attr{{"data-bind:plan", ""}},
			Items: []RadioListItem{
				{Label: "Free", Value: "free", Description: "3 projects", StartContent: child, EndContent: child, Class: "i",
					Attrs: []Attr{{"data-r", "1"}}, InputAttrs: []Attr{{"data-i", "1"}}},
				{Label: "Pro", Value: "pro", AriaLabel: "Pro plan", ID: "pro"},
				{Label: "Enterprise", Value: "ent", IsDisabled: true},
			}})},
		{"disabled", RadioList(RadioListProps{Label: "Off", IsDisabled: true, IsRequired: true, IsOptional: true,
			Items: []RadioListItem{{Label: "A", Value: "a"}}})},
	})
	price := InputGroupProps{ID: "price", Label: "Price", Description: "Per seat", Size: "lg",
		Status: &FieldStatus{Type: "error", Message: "Too low"}, IsRequired: true, Class: "c", Attrs: []Attr{{"data-x", "1"}}}
	checkFixtures(t, "input-group", []fixture{
		{"default", with(InputGroup(InputGroupProps{Label: "Site"}), child)},
		{"members", inputGroup(price, group(
			InputGroupText("$"),
			NumberInput(NumberInputProps{ID: "amt", Label: "Amount", Value: fp(5), Group: price.Member(), Units: "USD"}),
			TextInput(TextInputProps{ID: "note", Label: "Note", Group: price.Member(), Status: &FieldStatus{Type: "error"}}),
			Button(ButtonProps{Label: "Go", Attrs: []Attr{InputGroupItem}}),
		))},
		{"disabled", inputGroup(InputGroupProps{Label: "Off", IsDisabled: true}, TextInput(TextInputProps{Label: "Inner",
			Group: InputGroupProps{Label: "Off", IsDisabled: true}.Member()}))},
	})
	checkFixtures(t, "number-input", []fixture{
		{"default", NumberInput(NumberInputProps{Label: "Quantity"})},
		{"full", NumberInput(NumberInputProps{ID: "qty", Label: "Quantity", Value: fp(1234.5), Description: "How many",
			IsRequired: true, StartIcon: "search", LabelIcon: "info", Status: &FieldStatus{Type: "warning", Message: "High"},
			Size: "lg", Placeholder: "0", Width: "200px", HasAutoFocus: true, Name: "qty", Min: fp(-5), Max: fp(1e6),
			Step: 0.5, Format: `{"style":"currency","currency":"USD"}`, IsWheelDisabled: true, HasNumberSteppers: true,
			Units: "pcs", IsIntegerOnly: true, HasClear: true, Locale: "de-DE", Class: "c", ControlClass: "cc",
			Attrs: []Attr{{"data-x", "1"}}, InputAttrs: []Attr{{"data-i", "1"}}, ValueAttrs: []Attr{{"data-bind:qty", ""}}})},
		{"error-message", NumberInput(NumberInputProps{Label: "Qty", Status: &FieldStatus{Type: "error", Message: "Bad"},
			StatusVariant: "detached", HasClear: true})},
		{"disabled", NumberInput(NumberInputProps{Label: "Qty", Value: fp(0), IsDisabled: true, Name: "q", HasClear: true,
			HasNumberSteppers: true, IsReadOnly: true})},
	})
	checkFixtures(t, "slider", []fixture{
		{"default", Slider(SliderProps{Label: "Volume"})},
		{"full", Slider(SliderProps{ID: "vol", Label: "Volume", Values: []float64{150}, Description: "Loud",
			IsRequired: true, Status: &FieldStatus{Type: "error", Message: "Too loud"}, Width: "300px", Min: fp(-10), Max: fp(120),
			Step: 2.5, Format: `{"style":"percent"}`, Locale: "ko-KR", Name: "v", Form: "f",
			Marks: []SliderMark{{Value: -10, Label: "Low"}, {Value: 55}, {Value: 120, Label: "High"}},
			Class: "c", ContainerClass: "cc", Attrs: []Attr{{"data-x", "1"}}, ValueAttrs: []Attr{{"data-bind:vol", ""}}})},
		{"range", Slider(SliderProps{Label: "Price", Values: []float64{20, 80}, MinStepsBetweenThumbs: 5, ValueDisplay: "text",
			Orientation: "vertical", Marks: []SliderMark{{Value: 50, Label: "Mid"}}, IsDisabled: true, Name: "p",
			Attrs: []Attr{{"data-x", "1"}}, ValueAttrs: []Attr{{"data-bind:lo", ""}}, EndValueAttrs: []Attr{{"data-bind:hi", ""}}})},
		{"none", Slider(SliderProps{Label: "Quiet", Values: []float64{1.25}, Min: fp(1), Max: fp(1), ValueDisplay: "none"})},
	})
	checkFixtures(t, "file-input", []fixture{
		{"default", FileInput(FileInputProps{Label: "Avatar"})},
		{"full", FileInput(FileInputProps{ID: "docs", Label: "Documents", Accept: ".pdf,image/*", IsMultiple: true,
			MaxSize: 1048576, MaxFiles: 3, IsRequired: true, Description: "PDFs", Placeholder: "Pick", Mode: "dropzone",
			Width: "400px", Name: "docs", Form: "f", Class: "c", ControlClass: "cc", Attrs: []Attr{{"data-x", "1"}},
			InputAttrs: []Attr{{"data-bind:docs", ""}}})},
		{"status", FileInput(FileInputProps{Label: "Avatar", Status: &FieldStatus{Type: "success", Message: "Uploaded"},
			IsLoading: true, StatusVariant: "detached"})},
		{"disabled", FileInput(FileInputProps{Label: "Avatar", IsDisabled: true, Name: "a", Form: "f", MaxFiles: 2})},
	})
}
