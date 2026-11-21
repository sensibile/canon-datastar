//! Badge, Divider, Stack, Spinner, VisuallyHidden. Ported from Astryx
//! (packages/core/src/Badge, Divider, Stack, Spinner, VisuallyHidden), MIT, Meta Platforms, Inc.

use crate::Attr;
use crate::html::{attrs, cls, decl, el, or, styles, text};
use crate::icon::{IconProps, icon};
use crate::text::{TextProps, text as text_el};
use crate::types::space_css;
use maud::Markup;

/// Props for [`visually_hidden`].
#[derive(Clone, Debug, Default)]
pub struct VisuallyHiddenProps {
    /// The element, "span" by default.
    pub as_: String,
    pub id: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Renders content for assistive tech only.
pub fn visually_hidden(p: &VisuallyHiddenProps, children: Markup) -> Markup {
    let a = attrs()
        .set("class", &cls("astryx-visually-hidden", &p.class))
        .opt("id", &p.id)
        .extra(&p.attrs);
    el(or(&p.as_, "span"), a, [Some(children)])
}

/// Props for [`badge`].
#[derive(Clone, Debug, Default)]
pub struct BadgeProps {
    /// "neutral" (default), "info", "success", "warning", "error", or a color:
    /// "blue", "cyan", "gray", "green", "orange", "pink", "purple", "red", "teal", "yellow".
    pub variant: String,
    pub label: String,
    /// A leading built-in glyph name.
    pub icon: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Renders a small status label.
pub fn badge(p: &BadgeProps) -> Markup {
    let a = attrs()
        .set("class", &cls("astryx-badge", &p.class))
        .set("data-variant", or(&p.variant, "neutral"))
        .flag("data-has-label", !p.label.is_empty())
        .opt("title", &p.label)
        .extra(&p.attrs);
    let icon_m = (!p.icon.is_empty()).then(|| icon(&IconProps::sized(&p.icon, "xsm")));
    let label = el(
        "span",
        attrs()
            .set("class", "astryx-badge-label")
            .set("dir", "auto"),
        [Some(text(&p.label))],
    );
    el("span", a, [icon_m, Some(label)])
}

/// Props for [`divider`].
#[derive(Clone, Debug, Default)]
pub struct DividerProps {
    /// "horizontal" (default) or "vertical".
    pub orientation: String,
    /// Text shown in the middle of the line.
    pub label: String,
    /// "subtle" (default) or "strong".
    pub variant: String,
    /// Extend past the parent container's padding.
    pub is_full_bleed: bool,
    /// With a label, names the separator by the label element (`id` + "-label");
    /// otherwise the label text becomes aria-label.
    pub id: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Renders a separator line.
pub fn divider(p: &DividerProps) -> Markup {
    let orientation = or(&p.orientation, "horizontal");
    let label_id = if !p.label.is_empty() && !p.id.is_empty() {
        format!("{}-label", p.id)
    } else {
        String::new()
    };
    let a = attrs()
        .set("class", &cls("astryx-divider", &p.class))
        .opt("id", &p.id)
        .set("role", "separator")
        .set("aria-orientation", orientation)
        .when(
            !p.label.is_empty() && label_id.is_empty(),
            "aria-label",
            &p.label,
        )
        .opt("aria-labelledby", &label_id)
        .set("data-orientation", orientation)
        .set("data-variant", or(&p.variant, "subtle"))
        .flag("data-full-bleed", p.is_full_bleed)
        .extra(&p.attrs);
    let line = || el("div", attrs().set("class", "astryx-divider-line"), []);
    if p.label.is_empty() {
        return el("div", a, [Some(line())]);
    }
    let label = el(
        "div",
        attrs()
            .set("class", "astryx-divider-label")
            .opt("id", &label_id),
        [Some(text(&p.label))],
    );
    el("div", a, [Some(line()), Some(label), Some(line())])
}

/// Props for [`stack`], [`hstack`] and [`vstack`]. Spacing fields take steps such as "2" or "1.5".
#[derive(Clone, Debug, Default)]
pub struct StackProps {
    /// "vertical" (default) or "horizontal".
    pub direction: String,
    /// Align horizontally / vertically whatever the direction:
    /// "start", "center", "end", "stretch", "between", "around", "evenly".
    pub h_align: String,
    pub v_align: String,
    /// Main-axis (`justify`) and cross-axis (`align`) alignment, used when h/v_align are unset.
    pub justify: String,
    pub align: String,
    pub gap: String,
    pub padding: String,
    pub padding_inline: String,
    pub padding_inline_start: String,
    pub padding_inline_end: String,
    pub padding_block: String,
    pub padding_block_start: String,
    pub padding_block_end: String,
    pub is_scrollable: bool,
    /// CSS lengths, e.g. "320px".
    pub width: String,
    pub height: String,
    pub max_width: String,
    pub min_height: String,
    /// "nowrap", "wrap", "wrap-reverse".
    pub wrap: String,
    /// The element, "div" by default.
    pub as_: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

fn justify_css(v: &str) -> &'static str {
    match v {
        "start" => "flex-start",
        "center" => "center",
        "end" => "flex-end",
        "between" => "space-between",
        "around" => "space-around",
        "evenly" => "space-evenly",
        _ => "",
    }
}

fn align_css(v: &str) -> &'static str {
    match v {
        "start" => "flex-start",
        "center" => "center",
        "end" => "flex-end",
        "stretch" => "stretch",
        _ => "",
    }
}

fn first<'a>(vals: &[&'a str]) -> &'a str {
    vals.iter().copied().find(|v| !v.is_empty()).unwrap_or("")
}

/// Lays out `children` in a flex row or column.
pub fn stack(p: &StackProps, children: Markup) -> Markup {
    let direction = or(&p.direction, "vertical");
    let horizontal = direction == "horizontal";
    let h = if p.h_align.is_empty() {
        if horizontal { &p.justify } else { &p.align }
    } else {
        &p.h_align
    };
    let v = if p.v_align.is_empty() {
        if horizontal { &p.align } else { &p.justify }
    } else {
        &p.v_align
    };
    let (main, cross) = if horizontal { (h, v) } else { (v, h) };
    let style = styles(&[
        decl("gap", &space_css(&p.gap)),
        decl("justify-content", justify_css(main)),
        decl("align-items", align_css(cross)),
        decl("flex-wrap", &p.wrap),
        decl(
            "padding-inline-start",
            &space_css(first(&[
                &p.padding_inline_start,
                &p.padding_inline,
                &p.padding,
            ])),
        ),
        decl(
            "padding-inline-end",
            &space_css(first(&[
                &p.padding_inline_end,
                &p.padding_inline,
                &p.padding,
            ])),
        ),
        decl(
            "padding-block-start",
            &space_css(first(&[
                &p.padding_block_start,
                &p.padding_block,
                &p.padding,
            ])),
        ),
        decl(
            "padding-block-end",
            &space_css(first(&[&p.padding_block_end, &p.padding_block, &p.padding])),
        ),
        decl("width", &p.width),
        decl("height", &p.height),
        decl("max-width", &p.max_width),
        decl("min-height", &p.min_height),
    ]);
    let a = attrs()
        .set("class", &cls("astryx-stack", &p.class))
        .set("data-direction", direction)
        .flag("data-scrollable", p.is_scrollable)
        .opt("style", &style)
        .extra(&p.attrs);
    el(or(&p.as_, "div"), a, [Some(children)])
}

/// A horizontal [`stack`].
pub fn hstack(p: &StackProps, children: Markup) -> Markup {
    stack(
        &StackProps {
            direction: "horizontal".into(),
            ..p.clone()
        },
        children,
    )
}

/// A vertical [`stack`].
pub fn vstack(p: &StackProps, children: Markup) -> Markup {
    stack(
        &StackProps {
            direction: "vertical".into(),
            ..p.clone()
        },
        children,
    )
}

/// Props for [`spinner`].
#[derive(Clone, Debug, Default)]
pub struct SpinnerProps {
    /// "sm", "md" (default), "lg", "xl".
    pub size: String,
    /// "default", "subtle", "onMedia", "inherit".
    pub shade: String,
    /// Visible text under the spinner; also names the progressbar.
    pub label: String,
    /// Names the progressbar when there is no label ("Loading" by default).
    pub aria_label: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

impl SpinnerProps {
    pub(crate) fn of(size: &str, shade: &str) -> Self {
        SpinnerProps {
            size: size.into(),
            shade: shade.into(),
            ..Default::default()
        }
    }
}

/// (frame, r, border, arc, gap), precomputed so Go and Rust print identical numbers.
fn spinner_geom(size: &str) -> Option<[&'static str; 5]> {
    Some(match size {
        "sm" => ["14", "5", "2", "11.780972450961723", "19.634954084936208"],
        "md" => ["20", "7", "3", "16.493361431346415", "27.48893571891069"],
        "lg" => ["24", "9", "3", "21.205750411731103", "35.34291735288517"],
        "xl" => ["36", "14", "4", "32.98672286269283", "54.97787143782138"],
        _ => return None,
    })
}

/// Renders an indeterminate progress ring.
/// Not ported: theme spinner indicators, rotation phase sync across spinners.
pub fn spinner(p: &SpinnerProps) -> Markup {
    let (size, g) = match spinner_geom(or(&p.size, "md")) {
        Some(g) => (or(&p.size, "md"), g),
        None => ("md", spinner_geom("md").unwrap()),
    };
    let [frame, r, border, arc, gap] = g;
    let has_label = !p.label.is_empty();
    let a = attrs()
        .set("class", &cls("astryx-spinner", &p.class))
        .set("data-size", size)
        .set("data-shade", or(&p.shade, "default"))
        .flag("data-has-label", has_label)
        .extra(&p.attrs);
    let name = or(&p.aria_label, or(&p.label, "Loading"));
    let circle = |class: &str, dash: Option<String>| {
        let mut c = attrs()
            .set("class", class)
            .set("cx", "50%")
            .set("cy", "50%")
            .set("r", r)
            .set("stroke-width", border);
        if let Some(d) = dash {
            c = c.set("stroke-dasharray", &d);
        }
        el("circle", c, [])
    };
    let ring = el(
        "span",
        attrs()
            .set("class", "astryx-spinner-ring")
            .set("role", "progressbar")
            .set("aria-label", name)
            .set("style", &format!("width: {frame}px; height: {frame}px")),
        [
            Some(el(
                "svg",
                attrs()
                    .set("width", frame)
                    .set("height", frame)
                    .set("aria-hidden", "true"),
                [Some(circle("astryx-spinner-track", None))],
            )),
            Some(el(
                "span",
                attrs()
                    .set("class", "astryx-spinner-rotor")
                    .set("aria-hidden", "true"),
                [Some(el(
                    "svg",
                    attrs().set("width", frame).set("height", frame),
                    [Some(circle(
                        "astryx-spinner-arc",
                        Some(format!("{arc} {gap}")),
                    ))],
                ))],
            )),
        ],
    );
    if !has_label {
        return el("span", a, [Some(ring)]);
    }
    let label = text_el(
        &TextProps {
            kind: "body".into(),
            weight: "bold".into(),
            ..Default::default()
        },
        text(&p.label),
    );
    el("div", a, [Some(ring), Some(label)])
}
