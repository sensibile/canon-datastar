//! Port of Astryx `Icon` (packages/core/src/Icon/Icon.tsx), MIT, Meta Platforms, Inc.

use crate::Attr;
use crate::html::{attrs, cls, el, or, raw};
use crate::icons_gen::DEFAULT_ICONS;
use maud::Markup;

pub(crate) struct Glyph {
    pub(crate) solid: bool,
    pub(crate) stroke_width: &'static str,
    pub(crate) body: &'static str,
}

/// Props for [`icon`].
#[derive(Clone, Debug, Default)]
pub struct IconProps {
    /// A built-in glyph (see icons/default-icons.json), e.g. "check", "close", "chevronDown".
    pub name: String,
    /// "xsm", "sm", "md" (default), "lg".
    pub size: String,
    /// "inherit" (default), "primary", "secondary", "accent", "error", "blue", ...
    pub color: String,
    /// Names the icon for assistive tech; without it the icon is decorative.
    pub label: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

impl IconProps {
    pub fn new(name: &str) -> Self {
        IconProps {
            name: name.into(),
            ..Default::default()
        }
    }
    pub(crate) fn sized(name: &str, size: &str) -> Self {
        IconProps {
            name: name.into(),
            size: size.into(),
            ..Default::default()
        }
    }
}

/// The built-in glyph names.
pub fn icon_names() -> impl Iterator<Item = &'static str> {
    DEFAULT_ICONS.iter().map(|(n, _)| *n)
}

/// Renders a built-in glyph. An unknown name renders nothing.
pub fn icon(p: &IconProps) -> Markup {
    let Ok(i) = DEFAULT_ICONS.binary_search_by(|(n, _)| (*n).cmp(p.name.as_str())) else {
        return raw("");
    };
    let g = &DEFAULT_ICONS[i].1;
    let mut a = attrs()
        .set("class", &cls("astryx-icon", &p.class))
        .set("data-size", or(&p.size, "md"))
        .set("data-color", or(&p.color, "inherit"));
    a = if p.label.is_empty() {
        a.set("aria-hidden", "true")
    } else {
        a.set("role", "img").set("aria-label", &p.label)
    };
    a = a.extra(&p.attrs);
    let mut svg = attrs()
        .set("xmlns", "http://www.w3.org/2000/svg")
        .set("viewBox", "0 0 24 24")
        .set("width", "1em")
        .set("height", "1em")
        .set("aria-hidden", "true");
    svg = if g.solid {
        svg.set("fill", "currentColor")
    } else {
        svg.set("fill", "none")
            .set("stroke", "currentColor")
            .set("stroke-width", g.stroke_width)
            .set("stroke-linecap", "round")
            .set("stroke-linejoin", "round")
    };
    el("span", a, [Some(el("svg", svg, [Some(raw(g.body))]))])
}
