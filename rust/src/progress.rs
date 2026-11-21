//! Port of Astryx `ProgressBar` (packages/core/src/ProgressBar), MIT, Meta Platforms, Inc.
//! To update a bar live, patch it from the server (Datastar morphs it in place, so the fill
//! animates). Not ported: the coarse-pointer minimum fill and touch-tuned motion.

use crate::Attr;
use crate::html::{attrs, cls, el, group, or, text};
use crate::misc::{VisuallyHiddenProps, visually_hidden};
use crate::tooltip::{TooltipProps, tooltip};
use crate::types::slug;
use maud::Markup;

/// A labelled tick on the track.
#[derive(Clone, Debug, Default)]
pub struct ProgressBarMark {
    pub value: f64,
    pub label: String,
}

/// Props for [`progress_bar`].
#[derive(Clone, Debug, Default)]
pub struct ProgressBarProps {
    /// Prefixes the label and mark tooltip ids. Defaults to "progress-" + a slug of `label`.
    pub id: String,
    pub label: String,
    pub value: f64,
    /// The value at full (100 when 0).
    pub max: f64,
    pub is_label_hidden: bool,
    pub has_value_label: bool,
    /// Replaces the default "N%" value text.
    pub value_label: String,
    /// "accent" (default), "success", "warning", "error", "neutral".
    pub variant: String,
    pub is_indeterminate: bool,
    pub marks: Vec<ProgressBarMark>,
    pub is_disabled: bool,
    pub class: String,
    pub attrs: Vec<Attr>,
}

fn clamp_to(v: f64, max: f64) -> f64 {
    let v = if v.is_finite() { v } else { 0.0 };
    v.max(0.0).min(max)
}

/// Renders a labelled progress bar.
pub fn progress_bar(p: &ProgressBarProps) -> Markup {
    let id = if p.id.is_empty() {
        format!("progress-{}", or(&slug(&p.label), "bar"))
    } else {
        p.id.clone()
    };
    let label_id = format!("{id}-label");
    let mut max = if p.max == 0.0 { 100.0 } else { p.max };
    if !max.is_finite() || max < 0.0 {
        max = 0.0;
    }
    let value = clamp_to(p.value, max);
    let pct = if max > 0.0 { value / max * 100.0 } else { 0.0 };
    let value_text = if p.value_label.is_empty() {
        format!("{}%", pct.round())
    } else {
        p.value_label.clone()
    };
    let variant = or(&p.variant, "accent");
    let fill_variant = if p.is_disabled { "disabled" } else { variant };
    let show_value = p.has_value_label && !p.is_indeterminate;

    let header = if !p.is_label_hidden || show_value {
        let val = show_value.then(|| {
            el(
                "span",
                attrs().set("class", "astryx-progress-bar-value"),
                [Some(text(&value_text))],
            )
        });
        el(
            "div",
            attrs().set("class", "astryx-progress-bar-header"),
            [
                Some(el(
                    "span",
                    attrs()
                        .set("id", &label_id)
                        .set("class", "astryx-progress-bar-label")
                        .flag("data-hidden", p.is_label_hidden),
                    [Some(text(&p.label))],
                )),
                val,
            ],
        )
    } else {
        visually_hidden(
            &VisuallyHiddenProps {
                id: label_id.clone(),
                ..Default::default()
            },
            text(&p.label),
        )
    };

    let mut track = attrs().set("role", "progressbar");
    let fill = if p.is_indeterminate {
        el(
            "div",
            attrs()
                .set("class", "astryx-progress-bar-fill")
                .set("data-mode", "indeterminate")
                .set("data-variant", fill_variant),
            [],
        )
    } else {
        track = track
            .set("aria-valuenow", &value.to_string())
            .set("aria-valuemin", "0")
            .set("aria-valuemax", &max.to_string())
            .set("aria-valuetext", &value_text);
        el(
            "div",
            attrs()
                .set("class", "astryx-progress-bar-fill")
                .set("data-mode", "determinate")
                .set("data-variant", fill_variant)
                .set("style", &format!("--_progress-bar-fill: {pct}%")),
            [],
        )
    };
    let track = track
        .set("aria-labelledby", &label_id)
        .set("class", "astryx-progress-bar-track");

    let marks = if p.is_indeterminate {
        vec![]
    } else {
        p.marks
            .iter()
            .enumerate()
            .filter(|(_, m)| m.value.is_finite())
            .map(|(i, m)| {
                let mark_pct = if max > 0.0 {
                    clamp_to(m.value, max) / max * 100.0
                } else {
                    0.0
                };
                let placement = if pct > 0.0 && mark_pct <= pct {
                    "fill"
                } else {
                    "track"
                };
                let mark = el(
                    "span",
                    attrs()
                        .set("role", "img")
                        .set("aria-label", &m.label)
                        .set("tabindex", "0")
                        .set("class", "astryx-progress-bar-mark")
                        .set("data-variant", fill_variant)
                        .set("data-placement", placement)
                        .set("style", &format!("inset-inline-start: {mark_pct}%")),
                    [],
                );
                Some(tooltip(
                    &TooltipProps {
                        id: format!("{id}-mark-{i}"),
                        content: m.label.clone(),
                        focus_trigger: "always".into(),
                        ..Default::default()
                    },
                    mark,
                ))
            })
            .collect()
    };
    let a = attrs()
        .set("class", &cls("astryx-progress-bar", &p.class))
        .set("data-variant", variant)
        .flag("data-disabled", p.is_disabled)
        .extra(&p.attrs);
    el(
        "div",
        a,
        [
            Some(header),
            Some(el(
                "div",
                attrs().set("class", "astryx-progress-bar-track-container"),
                [Some(el("div", track, [Some(fill)])), Some(group(marks))],
            )),
        ],
    )
}
