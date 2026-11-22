package canon

import (
	"strconv"
	"time"

	"github.com/a-h/templ"
)

// Port of Astryx `Timestamp` (packages/core/src/Timestamp), MIT, Meta Platforms, Inc.
// The server renders a <time datetime> with a UTC date-and-time fallback; static/canon.js formats
// it in the viewer's zone and locale with Intl (relative, absolute or "auto"), keeps live relative
// times ticking and fills the details tooltip. Not ported: the hover card with copy buttons,
// non-Gregorian calendars.

// TimestampTooltipEntry is one line of the details tooltip.
type TimestampTooltipEntry struct {
	// TimezoneID is an IANA zone, or "local" / empty for the viewer's zone.
	TimezoneID string
	// Format: "full" (default), "date", "date_long", "date_weekday", "date_time", "time",
	// "system_date", "system_date_time", "system_time", "unix_seconds".
	Format string
	Label  string
}

// TimestampProps configures Timestamp.
type TimestampProps struct {
	// ID of the details tooltip. Defaults to "timestamp-" + the Unix milliseconds.
	ID    string
	Value time.Time
	// Format: "auto" (default: relative within AutoThreshold, else date_time), "relative",
	// "relative_short", or one of the absolute formats listed on TimestampTooltipEntry.
	Format string
	// TimezoneID is the IANA zone for absolute formats (the viewer's zone by default).
	TimezoneID string
	// AutoThreshold is the seconds within which "auto" is relative (7 days when 0).
	AutoThreshold   int
	HasNoTooltip    bool
	TooltipEntries  []TimestampTooltipEntry
	IsTimezoneShown bool
	// IsLive keeps a relative time up to date.
	IsLive bool
	// Locale is a BCP 47 tag ("en-US" by default).
	Locale string
	// Type, Size, Color and Weight style the text ("supporting", secondary by default).
	Type, Size, Color, Weight string
	Class                     string
	Attrs                     []Attr
}

func isRelativeFormat(f string) bool { return f == "relative" || f == "relative_short" }

// Timestamp renders a date and time.
func Timestamp(p TimestampProps) templ.Component {
	format := or(p.Format, "auto")
	needsClock := isRelativeFormat(format) || format == "auto"
	effective := format
	if needsClock {
		effective = "date_time"
	}
	utc := p.Value.UTC()
	ms := p.Value.UnixMilli()
	threshold := p.AutoThreshold
	if threshold == 0 {
		threshold = 7 * 86400
	}
	entries := p.TooltipEntries
	hasTip := !p.HasNoTooltip && (needsClock || len(entries) > 0)

	ta := newAttrs().set("datetime", utc.Format("2006-01-02T15:04:05.000Z")).
		set("class", "astryx-timestamp-time").
		set("data-format", format).
		set("data-locale", or(p.Locale, "en-US")).
		opt("data-timezone", p.TimezoneID).
		when(format == "auto" && p.AutoThreshold != 0, "data-threshold", strconv.Itoa(threshold)).
		flag("data-timezone-shown", p.IsTimezoneShown).
		flag("data-live", p.IsLive).
		when(hasTip, "tabindex", "0")
	timeEl := el("time", ta, text(utc.Format("Jan 2, 2006, 3:04 PM")+" UTC"))

	var body templ.Component = timeEl
	if hasTip {
		if len(entries) == 0 {
			entries = []TimestampTooltipEntry{{}}
		}
		labelled := false
		for _, e := range entries {
			if e.Label != "" {
				labelled = true
			}
		}
		rows := make([]templ.Component, len(entries))
		for i, e := range entries {
			zone := visuallyHidden(VisuallyHiddenProps{Attrs: []Attr{{"data-zone-name", ""}}}, nil)
			var dt templ.Component
			switch {
			case !labelled:
				dt = el("dt", newAttrs().set("class", "astryx-timestamp-sr-dt"), zone)
			case e.Label != "":
				dt = el("dt", newAttrs().set("class", "astryx-timestamp-label"), text(e.Label))
			default:
				dt = el("dt", newAttrs().set("class", "astryx-timestamp-label"), zone)
			}
			rows[i] = el("div", newAttrs().set("class", "astryx-timestamp-row").
				opt("data-timezone", e.TimezoneID).opt("data-format", e.Format),
				dt, el("dd", newAttrs().set("class", "astryx-timestamp-value")))
		}
		details := el("dl", newAttrs().set("class", "astryx-timestamp-details").flag("data-labelled", labelled).
			flag("data-default", len(p.TooltipEntries) == 0), rows...)
		body = tooltip(TooltipProps{ID: or(p.ID, "timestamp-"+strconv.FormatInt(ms, 10)), ContentMarkup: details,
			FocusTrigger: "always", HoverIndication: "never"}, timeEl)
	}
	return textEl(TextProps{Type: or(p.Type, "supporting"), Size: p.Size, Color: or(p.Color, "secondary"), Weight: p.Weight,
		Class: cls("astryx-timestamp", p.Class), Attrs: append([]Attr{{"data-timestamp-format", effective}}, p.Attrs...)}, body)
}
