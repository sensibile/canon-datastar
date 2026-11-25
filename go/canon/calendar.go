package canon

import (
	"strconv"
	"strings"

	"github.com/a-h/templ"
)

// Port of Astryx `Calendar` (packages/core/src/Calendar), MIT, Meta Platforms, Inc.
// A month grid for picking a date (Mode "single") or a range (Mode "range"). Dates are ISO
// YYYY-MM-DD strings. The server renders the frame, the settings and the value; static/canon.js
// draws the month grids, since today and the month and day names belong to the browser (its
// clock, its Intl). Until then the grid area keeps its space, hidden, the way the Svelte and
// React ports render before hydration.
// The value lives in hidden inputs: one for a single date, start and end for a range. Bind
// signals with ValueAttrs / StartValueAttrs / EndValueAttrs, e.g. {"data-bind:day", ""}.
// Moving the shown months fires a bubbling "astryx-focus-date" event with detail {focusDate}.
// Keyboard (APG grid): arrows move by day and week, Home/End to the row's ends (Ctrl/Cmd: the
// month's), PageUp/PageDown by month (Shift: year); Escape cancels a range in progress.
// Gregorian only. Not ported: the `calendar` prop, dateConstraints functions (use DisabledDates
// and DisabledDaysOfWeek), the locale provider (pass Locale).

// CalendarProps configures Calendar.
type CalendarProps struct {
	// ID goes on the root element.
	ID string
	// Mode: "single" (default) or "range".
	Mode string
	// Value is the selected date in single mode.
	Value string
	// Start and End are the selected range in range mode (both or neither).
	Start, End string
	// NumberOfMonths shown side by side: 1 (default) or 2.
	NumberOfMonths int
	// Min and Max bound the selectable dates.
	Min, Max string
	// DisabledDates can't be picked; DisabledDaysOfWeek (0 Sunday … 6 Saturday) neither.
	DisabledDates      []string
	DisabledDaysOfWeek []int
	// MaxRangeSpan and MinRangeSpan bound a range's length in days, counting both ends.
	MaxRangeSpan, MinRangeSpan int
	// FocusDate is a day in the first month shown. Default: the value, else today clamped into
	// Min/Max. A patch that changes it moves the calendar.
	FocusDate string
	// NoOutsideDays hides days of adjacent months (shown dimmed by default).
	NoOutsideDays bool
	// HasWeekNumbers adds ISO week numbers; HasVariableRowCount gives each month only the rows it
	// needs instead of six.
	HasWeekNumbers, HasVariableRowCount bool
	// WeekStartsOn: 0 (Sunday, default) to 6.
	WeekStartsOn int
	// Locale for month and day names, default "en-US".
	Locale string
	// Name (single), StartName and EndName (range) and Form go on the hidden inputs.
	Name, StartName, EndName, Form string
	Class                          string
	Attrs                          []Attr
	ValueAttrs, StartValueAttrs    []Attr
	EndValueAttrs                  []Attr
}

func joinInts(list []int) string {
	parts := make([]string, len(list))
	for i, n := range list {
		parts[i] = strconv.Itoa(n)
	}
	return strings.Join(parts, " ")
}

func calendarHidden(class, name, form, value string, extra []Attr) templ.Component {
	return void("input", newAttrs().set("type", "hidden").set("class", class).
		opt("name", name).opt("form", form).set("value", value).extra(extra))
}

// Calendar renders the calendar frame; canon.js draws the months.
func Calendar(p CalendarProps) templ.Component {
	mode := "single"
	if p.Mode == "range" {
		mode = "range"
	}
	months := 1
	if p.NumberOfMonths == 2 {
		months = 2
	}
	week := p.WeekStartsOn
	if week < 0 || week > 6 {
		week = 0
	}
	nav := func(dir, label, icon string) templ.Component {
		return Button(ButtonProps{Label: label, Variant: "ghost", Icon: icon, IsIconOnly: true, IsDisabled: true,
			Class: "astryx-calendar-nav", Attrs: []Attr{{"data-nav", dir}}})
	}
	header := el("div", newAttrs().set("class", "astryx-calendar-header"),
		nav("prev", "Previous month", "chevronLeft"),
		el("span", newAttrs().set("class", "astryx-calendar-label")),
		nav("next", "Next month", "chevronRight"))
	grids := el("div", newAttrs().set("class", "astryx-calendar-months").flag("data-pending", true).
		set("aria-hidden", "true").flag("inert", true))
	live := visuallyHidden(VisuallyHiddenProps{}, el("span", newAttrs().set("class", "astryx-calendar-live").
		set("aria-live", "polite").set("aria-atomic", "true")))
	var values []templ.Component
	if mode == "single" {
		values = append(values, calendarHidden("astryx-calendar-value", p.Name, p.Form, p.Value, p.ValueAttrs))
	} else {
		start, end := p.Start, p.End
		if start == "" || end == "" {
			start, end = "", ""
		}
		values = append(values, calendarHidden("astryx-calendar-start", p.StartName, p.Form, start, p.StartValueAttrs),
			calendarHidden("astryx-calendar-end", p.EndName, p.Form, end, p.EndValueAttrs))
	}
	a := newAttrs().opt("id", p.ID).
		set("class", cls("astryx-calendar", p.Class)).
		set("data-mode", mode).
		when(months == 2, "data-months", "2").
		set("data-locale", or(p.Locale, "en-US")).
		when(week != 0, "data-week-starts-on", strconv.Itoa(week)).
		opt("data-min", p.Min).
		opt("data-max", p.Max).
		opt("data-focus-date", p.FocusDate).
		opt("data-disabled-dates", strings.Join(p.DisabledDates, " ")).
		opt("data-disabled-days", joinInts(p.DisabledDaysOfWeek)).
		when(p.MaxRangeSpan > 0, "data-max-span", strconv.Itoa(p.MaxRangeSpan)).
		when(p.MinRangeSpan > 0, "data-min-span", strconv.Itoa(p.MinRangeSpan)).
		flag("data-no-outside-days", p.NoOutsideDays).
		flag("data-week-numbers", p.HasWeekNumbers).
		flag("data-variable-rows", p.HasVariableRowCount).
		extra(p.Attrs)
	return el("div", a, append([]templ.Component{header, grids, live}, values...)...)
}
