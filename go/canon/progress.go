package canon

import (
	"math"
	"strconv"

	"github.com/a-h/templ"
)

// Port of Astryx `ProgressBar` (packages/core/src/ProgressBar), MIT, Meta Platforms, Inc.
// To update a bar live, patch it from the server (Datastar morphs it in place, so the fill
// animates). Not ported: the coarse-pointer minimum fill and touch-tuned motion.

// ProgressBarMark is a labelled tick on the track.
type ProgressBarMark struct {
	Value float64
	Label string
}

// ProgressBarProps configures ProgressBar.
type ProgressBarProps struct {
	// ID prefixes the label and mark tooltip ids. Defaults to "progress-" + a slug of Label.
	ID    string
	Label string
	Value float64
	// Max is the value at full (100 when 0).
	Max           float64
	IsLabelHidden bool
	HasValueLabel bool
	// ValueLabel replaces the default "N%" value text.
	ValueLabel string
	// Variant: "accent" (default), "success", "warning", "error", "neutral".
	Variant         string
	IsIndeterminate bool
	Marks           []ProgressBarMark
	IsDisabled      bool
	Class           string
	Attrs           []Attr
}

func num(f float64) string { return strconv.FormatFloat(f, 'f', -1, 64) }

func clampTo(v, max float64) float64 {
	if math.IsNaN(v) || math.IsInf(v, 0) {
		v = 0
	}
	return math.Min(math.Max(0, v), max)
}

// ProgressBar renders a labelled progress bar.
func ProgressBar(p ProgressBarProps) templ.Component {
	id := or(p.ID, "progress-"+or(slug(p.Label), "bar"))
	labelID := id + "-label"
	max := p.Max
	if max == 0 {
		max = 100
	}
	if math.IsNaN(max) || math.IsInf(max, 0) || max < 0 {
		max = 0
	}
	value := clampTo(p.Value, max)
	pct := 0.0
	if max > 0 {
		pct = value / max * 100
	}
	valueText := p.ValueLabel
	if valueText == "" {
		valueText = num(math.Round(pct)) + "%"
	}
	variant := or(p.Variant, "accent")
	fillVariant := variant
	if p.IsDisabled {
		fillVariant = "disabled"
	}
	showValue := p.HasValueLabel && !p.IsIndeterminate

	var header templ.Component
	if !p.IsLabelHidden || showValue {
		var val templ.Component
		if showValue {
			val = el("span", newAttrs().set("class", "astryx-progress-bar-value"), text(valueText))
		}
		header = el("div", newAttrs().set("class", "astryx-progress-bar-header"),
			el("span", newAttrs().set("id", labelID).set("class", "astryx-progress-bar-label").flag("data-hidden", p.IsLabelHidden), text(p.Label)),
			val)
	} else {
		header = visuallyHidden(VisuallyHiddenProps{ID: labelID}, text(p.Label))
	}

	track := newAttrs().set("role", "progressbar")
	var fill templ.Component
	if p.IsIndeterminate {
		fill = el("div", newAttrs().set("class", "astryx-progress-bar-fill").set("data-mode", "indeterminate").set("data-variant", fillVariant))
	} else {
		track.set("aria-valuenow", num(value)).set("aria-valuemin", "0").set("aria-valuemax", num(max)).set("aria-valuetext", valueText)
		fill = el("div", newAttrs().set("class", "astryx-progress-bar-fill").set("data-mode", "determinate").
			set("data-variant", fillVariant).set("style", "--_progress-bar-fill: "+num(pct)+"%"))
	}
	track.set("aria-labelledby", labelID).set("class", "astryx-progress-bar-track")

	var marks []templ.Component
	if !p.IsIndeterminate {
		for i, m := range p.Marks {
			if math.IsNaN(m.Value) || math.IsInf(m.Value, 0) {
				continue
			}
			markPct := 0.0
			if max > 0 {
				markPct = clampTo(m.Value, max) / max * 100
			}
			placement := "track"
			if pct > 0 && markPct <= pct {
				placement = "fill"
			}
			mark := el("span", newAttrs().set("role", "img").set("aria-label", m.Label).set("tabindex", "0").
				set("class", "astryx-progress-bar-mark").set("data-variant", fillVariant).
				set("data-placement", placement).set("style", "inset-inline-start: "+num(markPct)+"%"))
			marks = append(marks, tooltip(TooltipProps{ID: id + "-mark-" + strconv.Itoa(i), Content: m.Label,
				FocusTrigger: "always"}, mark))
		}
	}
	a := newAttrs().set("class", cls("astryx-progress-bar", p.Class)).
		set("data-variant", variant).
		flag("data-disabled", p.IsDisabled).
		extra(p.Attrs)
	return el("div", a, header,
		el("div", newAttrs().set("class", "astryx-progress-bar-track-container"),
			el("div", track, fill), group(marks...)))
}
