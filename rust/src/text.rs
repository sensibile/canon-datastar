//! Port of Astryx `Text` and `Heading` (packages/core/src/Text, Heading), MIT, Meta Platforms, Inc.
//! Not ported: the truncation tooltip.

use crate::Attr;
use crate::html::{Attrs, attrs, cls, el, or};
use maud::Markup;

/// Props for [`text`].
#[derive(Clone, Debug, Default)]
pub struct TextProps {
    /// "body" (default), "large", "label", "supporting", "code", "display-1..3", "inherit".
    pub kind: String,
    /// "4xs" … "4xl"; overrides the type's font size.
    pub size: String,
    /// "primary", "secondary", "disabled", "placeholder", "accent", "inherit".
    /// Defaults by type (supporting is secondary, inherit is inherit, others primary).
    pub color: String,
    /// "normal", "medium", "semibold", "bold".
    pub weight: String,
    /// "inline" (default) or "block".
    pub display: String,
    /// Clamp to this many lines with an ellipsis. 0 = no clamp.
    pub max_lines: u32,
    /// "break-word" or "break-all" (applies with max_lines).
    pub word_break: String,
    /// "wrap", "nowrap", "balance", "pretty".
    pub text_wrap: String,
    /// "start" (default), "center", "end".
    pub justify: String,
    pub has_capsize: bool,
    pub has_strikethrough: bool,
    pub has_tabular_numbers: bool,
    /// The element: "span" (default), "p", "div", "label", "h1", "h2", "h3".
    pub as_: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

#[allow(clippy::too_many_arguments)]
fn text_layout(
    mut a: Attrs,
    display: &str,
    max_lines: u32,
    word_break: &str,
    justify: &str,
    text_wrap: &str,
    capsize: bool,
    strike: bool,
) -> Attrs {
    if max_lines == 0 {
        let d = if capsize {
            "block"
        } else {
            or(display, "inline")
        };
        a = a.set("data-display", d);
    } else {
        let trunc = if max_lines == 1 { "single" } else { "multi" };
        let wb = or(
            word_break,
            if max_lines == 1 {
                "break-all"
            } else {
                "break-word"
            },
        );
        a = a.set("data-truncate", trunc).set("data-word-break", wb);
    }
    if justify != "start" {
        a = a.opt("data-justify", justify);
    }
    a.opt("data-wrap", text_wrap)
        .flag("data-capsize", capsize)
        .flag("data-strikethrough", strike)
}

fn clamp_style(max_lines: u32) -> String {
    if max_lines > 1 {
        format!("-webkit-line-clamp: {max_lines}")
    } else {
        String::new()
    }
}

fn default_text_color(kind: &str) -> &'static str {
    match kind {
        "supporting" => "secondary",
        "inherit" => "inherit",
        _ => "primary",
    }
}

/// Renders styled text around `children`.
pub fn text(p: &TextProps, children: Markup) -> Markup {
    let kind = or(&p.kind, "body");
    let a = attrs()
        .set("class", &cls("astryx-text", &p.class))
        .set("data-type", kind)
        .opt("data-size", &p.size)
        .set("data-color", or(&p.color, default_text_color(kind)))
        .opt("data-weight", &p.weight);
    let a = text_layout(
        a,
        &p.display,
        p.max_lines,
        &p.word_break,
        &p.justify,
        &p.text_wrap,
        p.has_capsize,
        p.has_strikethrough,
    )
    .flag("data-tabular", p.has_tabular_numbers)
    .opt("style", &clamp_style(p.max_lines))
    .extra(&p.attrs);
    el(or(&p.as_, "span"), a, [Some(children)])
}

/// Props for [`heading`].
#[derive(Clone, Debug, Default)]
pub struct HeadingProps {
    /// 1–6 picks the h1–h6 tag and the type scale (invalid values become 2).
    pub level: u8,
    /// A display scale instead: "display-1", "display-2", "display-3".
    pub kind: String,
    pub weight: String,
    /// aria-level when the outline level differs from `level`.
    pub accessibility_level: u8,
    /// Defaults to "primary".
    pub color: String,
    /// Defaults to "block".
    pub display: String,
    pub max_lines: u32,
    pub word_break: String,
    pub text_wrap: String,
    pub justify: String,
    pub has_capsize: bool,
    pub has_strikethrough: bool,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Renders an h1–h6 around `children`.
pub fn heading(p: &HeadingProps, children: Markup) -> Markup {
    let level = if (1..=6).contains(&p.level) {
        p.level
    } else {
        2
    };
    let lv = level.to_string();
    let mut a = attrs().set("class", &cls("astryx-heading", &p.class));
    if p.accessibility_level != 0 && p.accessibility_level != level {
        a = a.set("aria-level", &p.accessibility_level.to_string());
    }
    let a = a
        .set("data-level", &lv)
        .opt("data-type", &p.kind)
        .set("data-color", or(&p.color, "primary"))
        .opt("data-weight", &p.weight);
    let a = text_layout(
        a,
        or(&p.display, "block"),
        p.max_lines,
        &p.word_break,
        &p.justify,
        &p.text_wrap,
        p.has_capsize,
        p.has_strikethrough,
    )
    .opt("style", &clamp_style(p.max_lines))
    .extra(&p.attrs);
    el(&format!("h{lv}"), a, [Some(children)])
}
