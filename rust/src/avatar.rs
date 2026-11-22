//! Avatar, AvatarStatusDot, AvatarGroup, AvatarGroupOverflow. Ported from Astryx
//! (packages/core/src/Avatar, AvatarGroup), MIT, Meta Platforms, Inc.
//! The initials or person glyph is always rendered (hidden behind an image) and static/canon.js
//! swaps to `fallback_src`, then to it, when an image fails to load. An avatar group's interactive
//! avatars form one tab stop moved with the arrow keys, Home and End (static/canon.js).
//! Differences: initials take letters and digits (not emoji), and CSS uppercases them.

use crate::Attr;
use crate::html::{attrs, cls, el, or, text, void};
use crate::icon::{IconProps, icon};
use crate::misc::{VisuallyHiddenProps, visually_hidden};
use crate::tooltip::{TooltipProps, tooltip};
use maud::Markup;

/// Props for an avatar's status dot ([`AvatarProps::status`]).
#[derive(Clone, Debug, Default)]
pub struct AvatarStatusDotProps {
    /// "success" (default), "neutral", "error".
    pub variant: String,
    /// Joins the avatar's accessible name.
    pub label: String,
    /// A built-in glyph shown on medium and large avatars instead of the default glyph.
    pub icon: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Props for [`avatar`].
#[derive(Clone, Debug, Default)]
pub struct AvatarProps {
    pub alt: String,
    pub name: String,
    pub src: String,
    pub fallback_src: String,
    /// "xsm", "sm", "md" (default), "lg", "xl"; `pixels` overrides it with a number.
    pub size: String,
    pub pixels: u32,
    /// "circle" (default), "rounded", "square".
    pub shape: String,
    pub status: Option<AvatarStatusDotProps>,
    /// Overrides the tooltip text (`name` by default); `has_no_tooltip` turns it off.
    pub tooltip: String,
    pub has_no_tooltip: bool,
    pub tooltip_id: String,
    /// Renders a link; `action` (`Some`, e.g. a data-on:click) renders a button.
    pub href: String,
    pub target: String,
    pub rel: String,
    pub action: Option<Vec<Attr>>,
    pub aria_label: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

fn avatar_size(size: &str, pixels: u32) -> (String, u32) {
    if pixels > 0 {
        return (pixels.to_string(), pixels);
    }
    match size {
        "xsm" => ("xsm".into(), 20),
        "sm" => ("sm".into(), 24),
        "md" => ("md".into(), 36),
        "lg" => ("lg".into(), 48),
        "xl" => ("xl".into(), 128),
        _ => ("md".into(), 36),
    }
}

/// The first letter or digit of the first and last words.
fn avatar_initials(name: &str) -> String {
    let initials: Vec<char> = name
        .split_whitespace()
        .filter_map(|w| w.chars().find(|c| c.is_alphanumeric()))
        .collect();
    match initials.as_slice() {
        [] => String::new(),
        [one] => one.to_string(),
        [first, .., last] => format!("{first}{last}"),
    }
}

/// `s` unless it is blank.
fn meaningful(s: &str) -> &str {
    if s.trim().is_empty() { "" } else { s }
}

fn css_string(s: &str) -> String {
    let s = s
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\a ");
    format!("\"{s}\"")
}

struct GroupCtx {
    size: String,
    shape: String,
    overlap: i64,
    px: u32,
    tab_stop: &'static str,
}

/// Renders a user image, initials or a person glyph.
pub fn avatar(p: &AvatarProps) -> Markup {
    render_avatar(p, None)
}

fn render_avatar(p: &AvatarProps, g: Option<&GroupCtx>) -> Markup {
    let (mut size, mut px) = avatar_size(&p.size, p.pixels);
    let mut shape = or(&p.shape, "circle").to_string();
    if let Some(g) = g {
        size = g.size.clone();
        px = g.px;
        shape = g.shape.clone();
    }
    let name = meaningful(&p.name);
    let initials = avatar_initials(name);
    let name_label = or(meaningful(&p.alt), name);
    let status_label = p.status.as_ref().map_or("", |s| meaningful(&s.label));
    let accessible_name = if !name_label.is_empty() && !status_label.is_empty() {
        format!("{name_label}, {status_label}")
    } else {
        or(name_label, status_label).to_string()
    };
    let tip = if p.has_no_tooltip {
        ""
    } else {
        or(&p.tooltip, name).trim()
    };

    let fpx = f64::from(px);
    let (src, fallback_src) = if p.src.is_empty() {
        (p.fallback_src.as_str(), "")
    } else {
        (p.src.as_str(), p.fallback_src.as_str())
    };
    let image = (!src.is_empty()).then(|| {
        void(
            "img",
            attrs()
                .set("src", src)
                .set("alt", "")
                .set("class", "astryx-avatar-image")
                .opt("data-fallback-src", fallback_src),
        )
    });
    let fallback = attrs()
        .set("class", "astryx-avatar-fallback")
        .flag("data-initials", !initials.is_empty())
        .flag("data-lone", g.is_none())
        .set("data-size", &size)
        .flag("hidden", image.is_some());
    let fallback_el = if initials.is_empty() {
        let glyph = (fpx * 0.6).to_string();
        el(
            "div",
            fallback,
            [Some(el(
                "svg",
                attrs()
                    .set("width", &glyph)
                    .set("height", &glyph)
                    .set("viewBox", "0 0 24 24")
                    .set("fill", "currentColor")
                    .set("aria-hidden", "true"),
                [Some(el("path", attrs().set("d", "M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"), []))],
            ))],
        )
    } else {
        el(
            "div",
            fallback.set(
                "style",
                &format!(
                    "font-size: {}px; --_initials: {}",
                    fpx * 0.4,
                    css_string(&initials)
                ),
            ),
            [],
        )
    };
    let status = p.status.as_ref().map(|s| {
        let mut sa = attrs()
            .set("class", "astryx-avatar-status")
            .flag("data-corner", shape != "circle");
        if shape == "circle" {
            let off = fpx * ((1.0 - 1.0 / 2f64.sqrt()) / 2.0);
            sa = sa.set(
                "style",
                &format!("bottom: {off}px; inset-inline-end: {off}px"),
            );
        }
        el("div", sa, [Some(status_dot(s, px))])
    });
    let content = el(
        "div",
        attrs().set("class", "astryx-avatar-content"),
        [image, Some(fallback_el)],
    );

    let is_link = !p.href.is_empty();
    let is_button = !is_link && p.action.is_some();
    let mut style = format!("width: {px}px; height: {px}px");
    if let Some(g) = g {
        style.push_str(&format!("; --_avatar-group-overlap: {}px", -g.overlap));
    }
    let mut a = attrs()
        .set("class", &cls("astryx-avatar", &p.class))
        .set("data-size", &size)
        .set("data-shape", &shape)
        .flag("data-interactive", is_link || is_button)
        .flag("data-in-group", g.is_some());
    let root = if is_link || is_button {
        a = if is_link {
            a.set("href", &p.href)
                .opt("target", &p.target)
                .opt("rel", &p.rel)
        } else {
            a.set("type", "button")
        };
        a = a
            .opt("aria-label", or(&p.aria_label, &accessible_name))
            .set("data-avatar-item", "");
        if let Some(g) = g {
            a = a.set("tabindex", g.tab_stop);
        }
        a = a
            .set("style", &style)
            .extra(p.action.as_deref().unwrap_or_default())
            .extra(&p.attrs);
        el(
            if is_link { "a" } else { "button" },
            a,
            [Some(content), status],
        )
    } else {
        let role = if accessible_name.is_empty() {
            "presentation"
        } else {
            "img"
        };
        let label = or(&p.aria_label, &accessible_name);
        a = a
            .set("role", role)
            .opt("aria-label", label)
            .when(label.is_empty(), "aria-hidden", "true")
            .when(!tip.is_empty() && g.is_none(), "tabindex", "0")
            .set("style", &style)
            .extra(&p.attrs);
        el("div", a, [Some(content), status])
    };
    if tip.is_empty() {
        return root;
    }
    tooltip(
        &TooltipProps {
            id: p.tooltip_id.clone(),
            content: tip.to_string(),
            ..Default::default()
        },
        root,
    )
}

fn status_dot(p: &AvatarStatusDotProps, avatar_px: u32) -> Markup {
    let (dot, border, icon_px, stroke) = match avatar_px {
        0..=36 => (10, 1, 0, 1.0),
        37..=72 => (20, 2, 12, 1.5),
        _ => (32, 4, 18, 2.0),
    };
    let variant = or(&p.variant, "success");
    let label = meaningful(&p.label);
    let a = attrs()
        .when(!label.is_empty(), "role", "img")
        .opt("aria-label", label)
        .set("class", &cls("astryx-avatar-status-dot", &p.class))
        .set("data-variant", variant)
        .set(
            "style",
            &format!("width: {dot}px; height: {dot}px; border-width: {border}px"),
        )
        .extra(&p.attrs);
    if !p.icon.is_empty() && icon_px > 0 {
        let inner = el(
            "span",
            attrs()
                .set("class", "astryx-avatar-status-dot-icon")
                .set("aria-hidden", "true")
                .set("style", &format!("width: {icon_px}px; height: {icon_px}px")),
            [Some(icon(&IconProps::sized(&p.icon, "xsm")))],
        );
        return el("div", a, [Some(inner)]);
    }
    if variant != "neutral" && variant != "error" {
        return el("div", a, []);
    }
    let field = f64::from(dot - border * 2);
    let f = field.to_string();
    let svg = attrs()
        .set("aria-hidden", "true")
        .set("viewBox", &format!("0 0 {f} {f}"))
        .set("width", &f)
        .set("height", &f)
        .set("fill", "none")
        .set("class", "astryx-avatar-status-dot-glyph");
    let (svg, shape) = if variant == "neutral" {
        let c = (field / 2.0).to_string();
        (
            svg.set("data-shape", "ring"),
            el(
                "circle",
                attrs()
                    .set("cx", &c)
                    .set("cy", &c)
                    .set("r", &((field - stroke) / 2.0).to_string())
                    .set("stroke", "currentColor")
                    .set("stroke-width", &stroke.to_string()),
                [],
            ),
        )
    } else {
        const SPAN: f64 = 0.75;
        let y = (field / 2.0).to_string();
        (
            svg.set("data-shape", "minus"),
            el(
                "line",
                attrs()
                    .set(
                        "x1",
                        &(field * (1.0 - SPAN) / 2.0 + stroke / 2.0).to_string(),
                    )
                    .set("y1", &y)
                    .set(
                        "x2",
                        &(field * (1.0 + SPAN) / 2.0 - stroke / 2.0).to_string(),
                    )
                    .set("y2", &y)
                    .set("stroke", "currentColor")
                    .set("stroke-width", &stroke.to_string())
                    .set("stroke-linecap", "round"),
                [],
            ),
        )
    };
    el("div", a, [Some(el("svg", svg, [Some(shape)]))])
}

/// Props for the "+N" chip at the end of an [`avatar_group`].
#[derive(Clone, Debug, Default)]
pub struct AvatarGroupOverflowProps {
    pub count: i64,
    /// `Some` makes it a button, e.g. `attr("data-on:click", "$showAll = true")`.
    pub action: Option<Vec<Attr>>,
    /// Replaces the "+N" text.
    pub content: Option<Markup>,
    pub aria_label: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// `n` with comma separators, like en-US toLocaleString.
fn group_thousands(n: i64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Props for [`avatar_group`].
#[derive(Clone, Debug, Default)]
pub struct AvatarGroupProps {
    /// Prefixes the keyboard hint's id ("avatar-group" by default).
    pub id: String,
    /// Size and shape apply to every avatar.
    pub size: String,
    pub pixels: u32,
    pub shape: String,
    pub avatars: Vec<AvatarProps>,
    pub overflow: Option<AvatarGroupOverflowProps>,
    /// Names the group ("Avatars" by default).
    pub aria_label: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Renders an overlapping row of avatars that share one size and shape.
pub fn avatar_group(p: &AvatarGroupProps) -> Markup {
    let (size, px) = avatar_size(&p.size, p.pixels);
    let shape = or(&p.shape, "circle").to_string();
    let overlap = (f64::from(px) * 0.25).round() as i64;
    let hint_id = format!("{}-hint", or(&p.id, "avatar-group"));
    let mut items = 0;
    let stop = |items: i32| if items == 0 { "0" } else { "-1" };
    let mut children: Vec<Option<Markup>> = Vec::new();
    for av in &p.avatars {
        let mut tab_stop = "-1";
        if !av.href.is_empty() || av.action.is_some() {
            tab_stop = stop(items);
            items += 1;
        }
        let ctx = GroupCtx {
            size: size.clone(),
            shape: shape.clone(),
            overlap,
            px,
            tab_stop,
        };
        children.push(Some(render_avatar(av, Some(&ctx))));
    }
    if let Some(o) = &p.overflow {
        let count = o.count.max(0);
        let content = o
            .content
            .clone()
            .unwrap_or_else(|| text(&format!("+{}", group_thousands(count))));
        let style = format!(
            "min-width: {w}px; height: {w}px; font-size: max(var(--text-supporting-size), {}px); --_avatar-group-overlap: {}px",
            f64::from(px) * 0.35,
            -overlap,
            w = px + 4
        );
        let label = if o.aria_label.is_empty() {
            format!("{} more", group_thousands(count))
        } else {
            o.aria_label.clone()
        };
        let (tag, a) = if o.action.is_some() {
            let a = attrs()
                .set("type", "button")
                .set("aria-label", &label)
                .set("data-avatar-item", "")
                .set("tabindex", stop(items));
            items += 1;
            ("button", a)
        } else {
            ("span", attrs().set("role", "img").set("aria-label", &label))
        };
        let a = a
            .set("class", &cls("astryx-avatar-group-overflow", &o.class))
            .set("data-size", &size)
            .set("data-shape", &shape)
            .set("style", &style)
            .extra(o.action.as_deref().unwrap_or_default())
            .extra(&o.attrs);
        children.push(Some(el(tag, a, [Some(content)])));
    }
    if items > 0 {
        children.push(Some(visually_hidden(
            &VisuallyHiddenProps {
                id: hint_id.clone(),
                ..Default::default()
            },
            text("Use arrow keys to move between avatars"),
        )));
    }
    let a = attrs()
        .set("role", "group")
        .set("aria-label", or(&p.aria_label, "Avatars"))
        .when(items > 0, "aria-describedby", &hint_id)
        .set("class", &cls("astryx-avatar-group", &p.class))
        .set("data-size", &size)
        .set("data-shape", &shape)
        .set("style", &format!("--_avatar-group-overlap: {}px", -overlap))
        .extra(&p.attrs);
    el("div", a, children)
}
