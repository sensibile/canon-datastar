package canon

import "testing"

func TestThemeFixtures(t *testing.T) {
	checkFixtures(t, "theme", []fixture{
		{"default", with(Theme(ThemeProps{}), child)},
		{"dark", with(Theme(ThemeProps{Name: "matcha", Mode: ModeDark}), child)},
		{"attrs", with(Theme(ThemeProps{Name: "y2k", Mode: ModeLight, Attrs: []Attr{
			{"data-attr:data-astryx-theme", "$theme"},
			{"title", `a "quoted" & <tagged> value`},
		}}), child)},
	})
}
