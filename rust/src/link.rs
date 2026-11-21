//! Port of Astryx `Link` (packages/core/src/Link), MIT, Meta Platforms, Inc.
//! Renders `<a href>` normally, a `<button type="button">` when there is no href (wire it up with
//! data-on:click), and an inert role="link" when disabled. Not ported: LinkProvider, tooltip.

use crate::Attr;
use crate::html::{attrs, cls, el, group, or, text};
use crate::icon::{IconProps, icon};
use crate::misc::{VisuallyHiddenProps, visually_hidden};
use crate::text::{TextProps, text as text_el};
use maud::Markup;

/// Props for [`link`].
#[derive(Clone, Debug, Default)]
pub struct LinkProps {
    pub href: String,
    /// Accessible name, when the visible text isn't enough.
    pub label: String,
    pub target: String,
    pub rel: String,
    pub has_underline: bool,
    pub is_disabled: bool,
    /// Opens in a new tab and shows an external-link icon.
    pub is_external_link: bool,
    /// Hidden text for external links ("(opens in a new tab)").
    pub new_tab_label: String,
    /// Body-sized text outside running prose.
    pub is_standalone: bool,
    /// kind, size, weight, display, max_lines style the label like text.
    pub kind: String,
    pub size: String,
    pub weight: String,
    pub display: String,
    pub max_lines: u32,
    /// "accent" (default), "primary", "secondary", "disabled", "inherit".
    pub color: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Renders a text link around `children`.
pub fn link(p: &LinkProps, children: Markup) -> Markup {
    let target = if p.is_external_link {
        "_blank"
    } else {
        p.target.as_str()
    };
    let rel = if target == "_blank" {
        let mut tokens: Vec<&str> = p.rel.split_whitespace().collect();
        for t in ["noopener", "noreferrer"] {
            if !tokens.contains(&t) {
                tokens.push(t);
            }
        }
        tokens.join(" ")
    } else {
        p.rel.clone()
    };
    let as_button = p.href.is_empty() && !p.is_disabled;
    let mut a = attrs()
        .set("class", &cls("astryx-link", &p.class))
        .set("data-color", or(&p.color, "accent"))
        .flag("data-underline", p.has_underline)
        .flag("data-standalone", p.is_standalone)
        .flag("data-truncating", p.max_lines > 0)
        .opt("aria-label", &p.label);
    let tag = if as_button {
        a = a.set("type", "button");
        "button"
    } else if p.is_disabled {
        a = a
            .set("role", "link")
            .set("tabindex", "0")
            .set("aria-disabled", "true");
        "a"
    } else {
        a = a
            .set("href", &p.href)
            .opt("target", target)
            .opt("rel", &rel);
        "a"
    };
    let a = a.extra(&p.attrs);
    let label = text_el(
        &TextProps {
            kind: or(&p.kind, "body").into(),
            size: p.size.clone(),
            weight: p.weight.clone(),
            color: "inherit".into(),
            display: p.display.clone(),
            max_lines: p.max_lines,
            class: "astryx-link-label".into(),
            ..Default::default()
        },
        children,
    );
    let external = (p.is_external_link && !as_button).then(|| {
        group([
            Some(icon(&IconProps::sized("externalLink", "xsm"))),
            Some(visually_hidden(
                &VisuallyHiddenProps::default(),
                text(or(&p.new_tab_label, "(opens in a new tab)")),
            )),
        ])
    });
    el(tag, a, [Some(label), external])
}
