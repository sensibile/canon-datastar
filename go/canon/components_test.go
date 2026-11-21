package canon

import "testing"

func boolPtr(b bool) *bool { return &b }

func TestIconFixtures(t *testing.T) {
	checkFixtures(t, "icon", []fixture{
		{"outline", Icon(IconProps{Name: "check"})},
		{"solid-labelled", Icon(IconProps{Name: "success", Size: "lg", Color: "success", Label: "Done"})},
		{"unknown", Icon(IconProps{Name: "nope"})},
	})
}

func TestTextFixtures(t *testing.T) {
	checkFixtures(t, "text", []fixture{
		{"default", with(Text(TextProps{}), child)},
		{"supporting", with(Text(TextProps{Type: "supporting", Weight: "bold", Justify: "center", As: "p"}), child)},
		{"clamp", with(Text(TextProps{MaxLines: 3, HasTabularNumbers: true, Class: "x"}), child)},
		{"single", with(Text(TextProps{MaxLines: 1, HasCapsize: true, HasStrikethrough: true, TextWrap: "balance"}), child)},
	})
}

func TestHeadingFixtures(t *testing.T) {
	checkFixtures(t, "heading", []fixture{
		{"default", with(Heading(HeadingProps{Level: 1}), child)},
		{"display", with(Heading(HeadingProps{Level: 9, Type: "display-2", AccessibilityLevel: 1, Color: "secondary", MaxLines: 2}), child)},
	})
}

func TestButtonFixtures(t *testing.T) {
	checkFixtures(t, "button", []fixture{
		{"default", Button(ButtonProps{Label: "Save"})},
		{"primary-icon", Button(ButtonProps{Label: "Add", Variant: "primary", Size: "lg", Icon: "check", Elevation: "low",
			EndContent: Badge(BadgeProps{Label: "3"}), Attrs: []Attr{{"data-on:click", "@post('/add')"}}})},
		{"loading", Button(ButtonProps{Label: "Saving", IsLoading: true, Width: "100%"})},
		{"interruptible", Button(ButtonProps{Label: "Run", IsLoading: true, IsInterruptible: true})},
		{"disabled", Button(ButtonProps{Label: "Nope", IsDisabled: true, Href: "/x", Type: "submit"})},
		{"link", Button(ButtonProps{Label: "Docs", Href: "/docs", Target: "_blank", Rel: "noopener"})},
		{"icon-only", IconButton(ButtonProps{Label: "Close", Icon: "close", Variant: "ghost", Size: "sm"})},
		{"content", Button(ButtonProps{Label: "Bold", Content: raw("<b>Bold</b>")})},
	})
}

func TestLinkFixtures(t *testing.T) {
	checkFixtures(t, "link", []fixture{
		{"default", with(Link(LinkProps{Href: "/a"}), child)},
		{"external", with(Link(LinkProps{Href: "https://x.test", IsExternalLink: true, Rel: "me noopener", HasUnderline: true, Color: "primary"}), child)},
		{"button", with(Link(LinkProps{Label: "Do it", Attrs: []Attr{{"data-on:click", "$n++"}}}), child)},
		{"disabled", with(Link(LinkProps{Href: "/a", IsDisabled: true, IsStandalone: true, MaxLines: 1,
			Attrs: []Attr{{"data-on:click", "@post('/x')"}, {"title", "t"}}}), child)},
	})
}

func TestBadgeFixtures(t *testing.T) {
	checkFixtures(t, "badge", []fixture{
		{"default", Badge(BadgeProps{Label: "New"})},
		{"icon", Badge(BadgeProps{Label: `"A" & B`, Variant: "success", Icon: "check"})},
	})
}

func TestSpinnerFixtures(t *testing.T) {
	checkFixtures(t, "spinner", []fixture{
		{"default", Spinner(SpinnerProps{})},
		{"labelled", Spinner(SpinnerProps{Size: "xl", Shade: "subtle", Label: "Loading files"})},
		{"bad-size", Spinner(SpinnerProps{Size: "huge", AriaLabel: "Busy"})},
	})
}

func TestLayoutFixtures(t *testing.T) {
	checkFixtures(t, "layout", []fixture{
		{"stack", with(Stack(StackProps{Gap: "2", Align: "center", Padding: "4", PaddingBlockStart: "1.5"}), child)},
		{"hstack", with(HStack(StackProps{Gap: "3", Justify: "between", VAlign: "center", Wrap: "wrap", Width: "100%", As: "section", IsScrollable: true}), child)},
		{"vstack-empty", with(VStack(StackProps{}), child)},
		{"divider", Divider(DividerProps{})},
		{"divider-label", Divider(DividerProps{Label: "or", Variant: "strong", IsFullBleed: true})},
		{"divider-labelledby", Divider(DividerProps{Label: "Section", ID: "sec", Orientation: "vertical"})},
		{"visually-hidden", with(VisuallyHidden(VisuallyHiddenProps{As: "div", ID: "vh"}), child)},
	})
}

func TestFieldFixtures(t *testing.T) {
	checkFixtures(t, "field", []fixture{
		{"label", FieldLabel(FieldLabelProps{Label: "Name", InputID: "name", IsRequired: true, LabelIcon: "info"})},
		{"label-description", FieldLabel(FieldLabelProps{Label: "Tags", InputID: "tags", IsGroupLabel: true, IsLabelHidden: true, IsOptional: true, Description: "Pick some", DescriptionID: "tags-desc", LabelID: "tags-label"})},
		{"status-detached", FieldStatusMessage(FieldStatus{Type: "warning", Message: "Careful"}, "detached", "w")},
		{"field-attached", with(Field(FieldProps{Label: "Email", InputID: "email", Description: "Work email", Status: &FieldStatus{Type: "error", Message: "Required"}, Width: "320px"}), child)},
		{"field-detached", with(Field(FieldProps{Label: "Email", InputID: "email", IsDisabled: true, Status: &FieldStatus{Type: "success", Message: "OK", MessageID: "m"}, StatusVariant: "detached"}), child)},
	})
}

func TestTextInputFixtures(t *testing.T) {
	checkFixtures(t, "text-input", []fixture{
		{"default", TextInput(TextInputProps{Label: "Full name"})},
		{"full", TextInput(TextInputProps{ID: "email", Label: "Email", Value: `a"b`, Type: "email", Description: "We never share it",
			IsRequired: true, StartIcon: "search", Status: &FieldStatus{Type: "error", Message: "Invalid"}, Size: "lg", IsLoading: true,
			Placeholder: "you@example.com", HasAutoFocus: true, Name: "email", HasSpellCheck: boolPtr(false),
			Class: "c", ControlClass: "cc", Attrs: []Attr{{"data-x", "1"}}, InputAttrs: []Attr{{"data-bind:email", ""}}})},
		{"disabled-messageless", TextInput(TextInputProps{ID: "q", Label: "Query", IsDisabled: true, Name: "q", IsLabelHidden: true, Status: &FieldStatus{Type: "error"}})},
		{"korean-label", TextInput(TextInputProps{Label: "이름", IsReadOnly: true, Value: "Tony", StatusVariant: "detached", Status: &FieldStatus{Type: "warning", Message: "Check"}})},
	})
}

func TestCheckboxFixtures(t *testing.T) {
	checkFixtures(t, "checkbox", []fixture{
		{"default", CheckboxInput(CheckboxInputProps{Label: "Accept terms"})},
		{"full", CheckboxInput(CheckboxInputProps{ID: "agree", Label: "Agree", Checked: true, Description: "Required to continue",
			IsRequired: true, Name: "agree", Value: "yes", Size: "sm", Width: "200px", LabelIcon: "info",
			Status: &FieldStatus{Type: "error", Message: "Please agree"}, InputAttrs: []Attr{{"data-bind:agree", ""}}})},
		{"indeterminate", CheckboxInput(CheckboxInputProps{ID: "all", Label: "All", IsIndeterminate: true, IsReadOnly: true, IsLabelHidden: true})},
		{"loading-disabled", CheckboxInput(CheckboxInputProps{ID: "l", Label: "L", IsLoading: true, IsDisabled: true, Name: "l", IsOptional: true})},
	})
}

func TestSwitchFixtures(t *testing.T) {
	checkFixtures(t, "switch", []fixture{
		{"default", Switch(SwitchProps{Label: "Notifications"})},
		{"full", Switch(SwitchProps{ID: "wifi", Label: "Wi-Fi", Checked: true, Description: "Connect automatically", Name: "wifi",
			Size: "sm", LabelPosition: "start", LabelSpacing: "spread", Width: "100%", IsRequired: true,
			Status: &FieldStatus{Type: "success", Message: "Connected"}, InputAttrs: []Attr{{"data-bind:wifi", ""}}})},
		{"loading-disabled", Switch(SwitchProps{ID: "s", Label: "S", IsLoading: true, IsDisabled: true, IsLabelHidden: true})},
	})
}
