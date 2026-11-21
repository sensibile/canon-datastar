package canon

import "testing"

func TestTooltipFixtures(t *testing.T) {
	checkFixtures(t, "tooltip", []fixture{
		{"default", with(Tooltip(TooltipProps{Content: "Copy link"}), child)},
		{"full", with(Tooltip(TooltipProps{ID: "tip", ContentMarkup: raw("<b>Rich</b>"), Placement: "end", Alignment: "start",
			Delay: 500, HideDelay: 100, FocusTrigger: "always", HoverIndication: "always", Class: "x", Attrs: []Attr{{"data-x", "1"}}}), child)},
		{"no-indication", with(Tooltip(TooltipProps{Content: "Plain", FocusTrigger: "never", HoverIndication: "never"}), raw("Text"))},
		{"disabled", with(Tooltip(TooltipProps{Content: "Hidden", IsDisabled: true}), child)},
		{"info-tip", InfoTip(InfoTipProps{Content: "Billed monthly", Size: "md", Placement: "below"})},
	})
}

func TestPopoverFixtures(t *testing.T) {
	trigger := Button(ButtonProps{Label: "Filters"})
	checkFixtures(t, "popover", []fixture{
		{"default", with(Popover(PopoverProps{ID: "filters", Content: child}), trigger)},
		{"full", with(Popover(PopoverProps{ID: "share", Content: child, Placement: "above", Alignment: "end", Width: "320px",
			Padding: "4", Label: "Share", IsModal: true, HasCloseButton: true, CloseButtonLabel: "Done",
			NoLightDismiss: true, Signal: "shareOpen", Class: "x", Attrs: []Attr{{"data-x", "1"}}}), trigger)},
		{"no-role", with(Popover(PopoverProps{Content: child, Role: "none", HasAutoFocus: true, NoEscapeDismiss: true}), trigger)},
	})
}

func TestDialogFixtures(t *testing.T) {
	checkFixtures(t, "dialog", []fixture{
		{"default", with(Dialog(DialogProps{ID: "confirm"}), child)},
		{"full", with(Dialog(DialogProps{ID: "edit", Width: "600px", MaxHeight: "90dvh", Purpose: "required", Padding: "6",
			Label: "Edit", Signal: "editOpen", Class: "x", Attrs: []Attr{{"data-x", "1"}}}), child)},
		{"fullscreen", with(Dialog(DialogProps{Variant: "fullscreen", Purpose: "form"}), child)},
		{"header", DialogHeader(DialogHeaderProps{Title: "Delete file?"})},
		{"header-full", DialogHeader(DialogHeaderProps{Title: "Settings", Subtitle: "Workspace", TitleID: "settings-title",
			HasCloseButton: true, CloseLabel: "Close settings", StartContent: raw("<i>s</i>"), EndContent: raw("<i>e</i>"),
			HasDivider: true, Class: "x", Attrs: []Attr{{"data-x", "1"}}})},
	})
}

func TestDropdownMenuFixtures(t *testing.T) {
	items := group(
		DropdownMenuItem(DropdownMenuItemProps{Label: "Edit", Icon: "wrench", Attrs: []Attr{{"data-on:click", "@post('/edit')"}}}),
		DropdownMenuDivider(),
		with(DropdownMenuGroup(DropdownMenuGroupProps{Title: "More actions"}),
			DropdownMenuItem(DropdownMenuItemProps{Label: "Docs", Description: "Opens in a new tab", Href: "/docs", Target: "_blank"})),
	)
	checkFixtures(t, "dropdown-menu", []fixture{
		{"default", with(DropdownMenu(DropdownMenuProps{Button: ButtonProps{Label: "Actions"}}), items)},
		{"full", with(DropdownMenu(DropdownMenuProps{ID: "row", Button: ButtonProps{Label: "Row", Icon: "moreHorizontal", IsIconOnly: true,
			Attrs: []Attr{{"data-x", "1"}}}, NoChevron: true, Placement: "above", Alignment: "end", MenuWidth: "240px",
			MenuMaxHeight: "300px", Header: raw("<i>h</i>"), Footer: raw("<i>f</i>"), Class: "x"}), items)},
		{"item-full", DropdownMenuItem(DropdownMenuItemProps{Label: "Delete", Variant: "destructive", Icon: "close",
			KeepOpen: true, EndContent: raw("<kbd>D</kbd>"), Class: "x"})},
		{"item-disabled", DropdownMenuItem(DropdownMenuItemProps{Label: "Archive", Href: "/a", IsDisabled: true,
			Attrs: []Attr{{"data-on:click", "@post('/a')"}, {"title", "t"}}})},
	})
}

func TestTabListFixtures(t *testing.T) {
	tabs := []TabProps{
		{Value: "home", Label: "Home", Href: "/", Icon: "menu"},
		{Value: "docs", Label: "Docs", Href: "/docs", EndContent: raw("<i>3</i>")},
		{Value: "off", Label: "Off", IsDisabled: true, Attrs: []Attr{{"data-on:click", "x()"}}},
	}
	checkFixtures(t, "tab-list", []fixture{
		{"nav", TabList(TabListProps{Value: "home", Tabs: tabs})},
		{"tablist", TabList(TabListProps{Value: "b", Signal: "tab", Role: "tablist", Size: "lg", Layout: "fill", HasDivider: true,
			Label: "Sections", Class: "x", Attrs: []Attr{{"data-x", "1"}}, Tabs: []TabProps{
				{Value: "a", Label: "A", ID: "tab-a", PanelID: "panel-a", Icon: "clock", SelectedIcon: "check"},
				{Value: "b", Label: "It's B", ID: "tab-b", PanelID: "panel-b", IsLabelHidden: true, Icon: "search",
					Attrs: []Attr{{"data-on:click", "@get('/b')"}, {"title", "t"}}},
			}})},
		{"nav-signal", TabList(TabListProps{Value: "x", Signal: "view", LabelledBy: "h", Tabs: []TabProps{{Value: "x", Label: "X"}}})},
		{"panel", with(TabPanel(TabPanelProps{ID: "panel-a", TabID: "tab-a", Value: "a", Signal: "tab", Class: "x"}), child)},
		{"panel-selected", with(TabPanel(TabPanelProps{Value: "b", IsSelected: true}), child)},
	})
}

func TestToastFixtures(t *testing.T) {
	checkFixtures(t, "toast", []fixture{
		{"default", Toast(ToastProps{Body: "Saved"})},
		{"error", Toast(ToastProps{ID: "t1", Type: "error", Body: "Failed", EndContent: raw("<i>retry</i>"),
			DismissLabel: "Close", Class: "x", Attrs: []Attr{{"data-x", "1"}}})},
		{"error-timed", Toast(ToastProps{Type: "error", BodyMarkup: raw("<b>Oops</b>"), AutoHideDuration: 8000})},
		{"sticky", Toast(ToastProps{Body: "Pinned", IsSticky: true})},
		{"viewport", with(ToastViewport(ToastViewportProps{}), Toast(ToastProps{Body: "Hi"}))},
		{"viewport-top", ToastViewport(ToastViewportProps{ID: "toasts", Position: "topStart", MaxVisible: 3, Label: "Alerts", DismissLabel: "Close"})},
	})
}
