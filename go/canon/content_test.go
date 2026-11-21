package canon

import (
	"testing"
	"time"
)

func TestContainerFixtures(t *testing.T) {
	checkFixtures(t, "card", []fixture{
		{"default", with(Card(CardProps{}), child)},
		{"sized", with(Card(CardProps{Variant: "blue", Elevation: "med", Width: "320px", Height: "200px", Padding: "6", Class: "x"}), child)},
	})
	checkFixtures(t, "section", []fixture{
		{"default", with(Section(SectionProps{}), child)},
		{"full", with(Section(SectionProps{Variant: "muted", Dividers: []string{"top", "end"}, MaxWidth: "40rem",
			Padding: "2", PaddingInline: "4", PaddingBlockEnd: "0.5", Attrs: []Attr{{"data-x", "1"}}}), child)},
	})
	checkFixtures(t, "center", []fixture{
		{"default", with(Center(CenterProps{}), child)},
		{"inline", with(Center(CenterProps{Axis: "vertical", IsInline: true, MinHeight: "10rem", Padding: "2", PaddingBlockStart: "8"}), child)},
	})
	checkFixtures(t, "grid", []fixture{
		{"default", with(Grid(GridProps{}), child)},
		{"columns", with(Grid(GridProps{Columns: 3, Gap: "2", RowHeight: 80, Align: "center", Width: "100%"}), child)},
		{"auto", with(Grid(GridProps{Auto: &GridAuto{MinWidth: 200, Max: 4, Repeat: "fit"}, ColumnGap: "1.5", Justify: "end"}), child)},
		{"auto-plain", with(Grid(GridProps{Auto: &GridAuto{MinWidth: 160}}), child)},
		{"span", with(GridSpan(GridSpanProps{Columns: 2, Rows: 3}), child)},
		{"span-full", with(GridSpan(GridSpanProps{IsFull: true}), child)},
	})
	checkFixtures(t, "aspect-ratio", []fixture{
		{"default", with(AspectRatio(AspectRatioProps{Ratio: 16.0 / 9}), child)},
		{"ellipse", with(AspectRatio(AspectRatioProps{Ratio: 1, Shape: "ellipse", Fit: "cover", Class: "x"}), child)},
	})
}

func TestContentFixtures(t *testing.T) {
	checkFixtures(t, "blockquote", []fixture{
		{"default", with(Blockquote(BlockquoteProps{}), child)},
		{"cite", with(Blockquote(BlockquoteProps{Cite: "Ada <Lovelace>"}), child)},
	})
	checkFixtures(t, "code", []fixture{
		{"default", with(Code(CodeProps{}), child)},
		{"inherit", with(Code(CodeProps{Color: "secondary", Size: "inherit"}), child)},
	})
	checkFixtures(t, "kbd", []fixture{
		{"mod", Kbd(KbdProps{Keys: "mod+shift+P"})},
		{"special", Kbd(KbdProps{Keys: "Esc + return+up+plus+f1", Class: "x"})},
	})
	checkFixtures(t, "empty-state", []fixture{
		{"default", EmptyState(EmptyStateProps{Title: "No results"})},
		{"full", EmptyState(EmptyStateProps{Title: "Inbox zero", Description: "Nothing new.", Icon: child, Actions: child,
			HeadingLevel: 2, IsCompact: true})},
	})
	checkFixtures(t, "skeleton", []fixture{
		{"default", Skeleton(SkeletonProps{})},
		{"sized", Skeleton(SkeletonProps{Width: "120px", Height: "1em", Radius: "rounded", Index: 3})},
	})
	checkFixtures(t, "status-dot", []fixture{
		{"default", StatusDot(StatusDotProps{Label: "Idle"})},
		{"tooltip", StatusDot(StatusDotProps{Variant: "success", Label: "Online", IsPulsing: true, Tooltip: "Online now", Icon: "check"})},
	})
}

func TestProgressBarFixtures(t *testing.T) {
	checkFixtures(t, "progress-bar", []fixture{
		{"default", ProgressBar(ProgressBarProps{Label: "Upload", Value: 40})},
		{"value", ProgressBar(ProgressBarProps{Label: "Disk", Value: 2, Max: 3, HasValueLabel: true, Variant: "warning",
			Marks: []ProgressBarMark{{1, "Quota"}, {2.5, "Limit"}}})},
		{"hidden", ProgressBar(ProgressBarProps{ID: "p", Label: "Sync", IsLabelHidden: true, IsIndeterminate: true, IsDisabled: true})},
		{"custom", ProgressBar(ProgressBarProps{Label: "Steps", Value: 150, HasValueLabel: true, IsLabelHidden: true, ValueLabel: "3 of 3"})},
	})
}

func TestTokenFixtures(t *testing.T) {
	checkFixtures(t, "token", []fixture{
		{"default", Token(TokenProps{Label: "Draft"})},
		{"link", Token(TokenProps{Label: "Docs", Href: "/docs", Color: "blue", Size: "sm", Description: "Opens docs"})},
		{"link-disabled", Token(TokenProps{Label: "Docs", Href: "/docs", IsDisabled: true})},
		{"button", Token(TokenProps{Label: "Filter", Action: []Attr{{"data-on:click", "$f = 1"}}, EndContent: child})},
		{"removable", Token(TokenProps{Label: "Tag", Href: "/t", RemoveAttrs: []Attr{{"data-on:click", "@delete('/t')"}}, Icon: child})},
		{"removable-plain", Token(TokenProps{Label: "Tag", IsRemovable: true, IsLabelHidden: true, RemoveName: "tag one"})},
		{"disabled", Token(TokenProps{Label: "Tag", Action: []Attr{{"data-on:click", "$f = 1"}}, IsDisabled: true,
			RemoveAttrs: []Attr{{"data-on:click", "@delete('/t')"}, {"title", "x"}}})},
	})
}

func TestAvatarFixtures(t *testing.T) {
	checkFixtures(t, "avatar", []fixture{
		{"initials", Avatar(AvatarProps{Name: "Ada  Byron Lovelace"})},
		{"icon", Avatar(AvatarProps{Size: "lg", Shape: "rounded", HasNoTooltip: true})},
		{"image", Avatar(AvatarProps{Name: "Ada", Src: "/a.png", FallbackSrc: "/b.png", Alt: "Ada L.", Size: "xl", TooltipID: "t",
			Status: &AvatarStatusDotProps{Label: "Online"}})},
		{"fallback-src", Avatar(AvatarProps{Name: "\"Q\"", FallbackSrc: "/b.png", Pixels: 60, Tooltip: "Custom",
			Status: &AvatarStatusDotProps{Variant: "neutral", Label: "Away"}})},
		{"link", Avatar(AvatarProps{Name: "Ada", Href: "/u/ada", Shape: "square", HasNoTooltip: true,
			Status: &AvatarStatusDotProps{Variant: "error", Label: "Busy", Icon: "close"}})},
		{"button", Avatar(AvatarProps{Alt: "Me", Action: []Attr{{"data-on:click", "$m = 1"}}, Size: "sm",
			Status: &AvatarStatusDotProps{Variant: "error"}})},
		{"group", AvatarGroup(AvatarGroupProps{ID: "team", Size: "sm", Shape: "rounded", Avatars: []AvatarProps{
			{Name: "Ada", TooltipID: "t1"}, {Name: "Bo", Href: "/bo", HasNoTooltip: true}, {Name: "Cy", Action: []Attr{}, HasNoTooltip: true}},
			Overflow: &AvatarGroupOverflowProps{Count: 1234, Action: []Attr{{"data-on:click", "$all = true"}}}})},
		{"group-static", AvatarGroup(AvatarGroupProps{Avatars: []AvatarProps{{Name: "Ada", HasNoTooltip: true}},
			Overflow: &AvatarGroupOverflowProps{Count: 3}})},
	})
}

func TestBannerFixtures(t *testing.T) {
	checkFixtures(t, "banner", []fixture{
		{"default", Banner(BannerProps{Title: "Saved"})},
		{"full", Banner(BannerProps{Status: "warning", Title: "Quota", Description: "Almost full.", IsDismissable: true,
			DismissAttrs: []Attr{{"data-on:click", "@post('/dismiss')"}}, EndContent: child, Elevation: "low", Content: child,
			Signal: "more", Role: "alert", Attrs: []Attr{{"aria-live", "polite"}, {"data-x", "1"}}})},
		{"open", Banner(BannerProps{ID: "b", Status: "error", Title: "Failed", Content: child, IsOpen: true, Container: "section"})},
		{"static", Banner(BannerProps{Status: "success", Title: "Done", Content: child, IsNotCollapsible: true, Icon: child,
			IsDismissable: true, DismissLabel: "Close", Role: "status"})},
	})
}

func TestCollapsibleFixtures(t *testing.T) {
	checkFixtures(t, "collapsible", []fixture{
		{"default", with(Collapsible(CollapsibleProps{Trigger: "Details"}), child)},
		{"closed", Collapsible(CollapsibleProps{ID: "c", Trigger: "More", IsClosed: true, ChevronPosition: "start", Signal: "more",
			Content: child, Class: "x"})},
		{"disabled", with(Collapsible(CollapsibleProps{Trigger: "Nope", IsDisabled: true}), child)},
		{"group", CollapsibleGroup(CollapsibleGroupProps{Value: []string{"b", "a"}, HasDividers: true, Items: []CollapsibleProps{
			{Trigger: "A", Value: "a", Content: child}, {Trigger: "B", Value: "b", Content: child}, {Trigger: "C", Content: child}}})},
		{"group-multiple", CollapsibleGroup(CollapsibleGroupProps{Type: "multiple", Value: []string{"a", "b"}, Density: "compact",
			ChevronPosition: "start", Items: []CollapsibleProps{{Trigger: "A", Value: "a", Content: child}, {Trigger: "B", Value: "b",
				ChevronPosition: "end", Content: child}}})},
	})
}

func TestMetadataListFixtures(t *testing.T) {
	items := []MetadataListItem{{Label: "Owner", Text: "Ada"}, {Label: "Status", Value: child, Icon: child}, {Label: "Size", Text: "3 MB", Class: "x"}}
	checkFixtures(t, "metadata-list", []fixture{
		{"default", MetadataList(MetadataListProps{Items: items})},
		{"max", MetadataList(MetadataListProps{ID: "m", Title: "File", MaxNumOfItems: 2, LabelWidth: "8rem", Items: items})},
		{"multi", MetadataList(MetadataListProps{Columns: "multi", MaxNumOfItems: 1, Items: items})},
		{"columns", MetadataList(MetadataListProps{Columns: "3", LabelPosition: "start", Items: items})},
		{"columns-top", MetadataList(MetadataListProps{Columns: "2", Items: items[:1]})},
		{"horizontal", MetadataList(MetadataListProps{Orientation: "horizontal", MaxNumOfItems: 1, Items: items[:2]})},
	})
}

func TestThumbnailFixtures(t *testing.T) {
	checkFixtures(t, "thumbnail", []fixture{
		{"default", Thumbnail(ThumbnailProps{})},
		{"image", Thumbnail(ThumbnailProps{Src: "/a.png", Alt: "A cat", Label: "cat.png", TooltipID: "t",
			Action: []Attr{{"data-on:click", "$open = 1"}}, RemoveAttrs: []Attr{{"data-on:click", "@delete('/a')"}}})},
		{"loading", Thumbnail(ThumbnailProps{IsLoading: true})},
		{"uploading", Thumbnail(ThumbnailProps{Src: "/a.png", IsLoading: true, Action: []Attr{{"data-on:click", "$open = 1"}},
			IsRemovable: true, ShowRemoveOn: "always"})},
		{"disabled", Thumbnail(ThumbnailProps{Src: "/a.png", Alt: "A", IsDisabled: true, IsRemovable: true, Class: "x"})},
	})
}

func TestTimestampFixtures(t *testing.T) {
	at := time.Date(2026, 3, 4, 17, 5, 9, 120_000_000, time.FixedZone("X", 3600))
	checkFixtures(t, "timestamp", []fixture{
		{"default", Timestamp(TimestampProps{Value: at})},
		{"absolute", Timestamp(TimestampProps{Value: at, Format: "date", TimezoneID: "Asia/Seoul", IsTimezoneShown: true, Type: "body", Color: "primary"})},
		{"entries", Timestamp(TimestampProps{ID: "ts", Value: at, Format: "relative_short", IsLive: true, Locale: "ko-KR", AutoThreshold: 60,
			TooltipEntries: []TimestampTooltipEntry{{TimezoneID: "UTC", Format: "date_time", Label: "UTC"}, {}}})},
		{"no-tooltip", Timestamp(TimestampProps{Value: time.Date(1969, 12, 31, 0, 30, 0, 0, time.UTC), Format: "auto", AutoThreshold: 60, HasNoTooltip: true})},
		{"absolute-tip", Timestamp(TimestampProps{Value: at, Format: "system_date", TooltipEntries: []TimestampTooltipEntry{{TimezoneID: "local", Format: "full"}}})},
	})
}
