package canon

import "testing"

func TestCalendarFixtures(t *testing.T) {
	checkFixtures(t, "calendar", []fixture{
		{"default", Calendar(CalendarProps{})},
		{"single", Calendar(CalendarProps{ID: "cal", Value: "2026-03-14", Min: "2026-01-01", Max: "2026-12-31",
			DisabledDates: []string{"2026-03-17", "2026-03-18"}, DisabledDaysOfWeek: []int{0, 6}, WeekStartsOn: 1,
			Locale: "de-DE", HasWeekNumbers: true, NoOutsideDays: true, Name: "day", Form: "frm", Class: "c",
			Attrs: []Attr{{"data-x", "1"}}, ValueAttrs: []Attr{{"data-bind:day", ""}}})},
		{"range", Calendar(CalendarProps{Mode: "range", Start: "2026-03-10", End: "2026-03-20", NumberOfMonths: 2,
			MaxRangeSpan: 14, MinRangeSpan: 3, FocusDate: "2026-03-01", HasVariableRowCount: true, StartName: "from",
			EndName: "to", StartValueAttrs: []Attr{{"data-bind:from", ""}}, EndValueAttrs: []Attr{{"data-bind:to", ""}}})},
		{"range-partial", Calendar(CalendarProps{Mode: "range", Start: "2026-03-10"})},
	})
}
