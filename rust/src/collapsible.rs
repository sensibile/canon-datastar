//! Collapsible and CollapsibleGroup. Ported from Astryx (packages/core/src/Collapsible), MIT,
//! Meta Platforms, Inc. static/canon.js toggles a collapsible on click (closing its siblings in a
//! "single" group) and fires a bubbling "canon-toggle" event with detail.open. `signal` mirrors
//! the open state in a Datastar signal both ways. Not ported: the animatedLayout height animation.

use crate::Attr;
use crate::html::{attrs, cls, el, or, text};
use crate::icon::{IconProps, icon};
use crate::types::slug;
use maud::Markup;
use std::collections::HashSet;

/// Props for [`collapsible`].
#[derive(Clone, Debug, Default)]
pub struct CollapsibleProps {
    /// Id of the root; the content is id + "-content". Defaults to "collapsible-" + a slug of `trigger`.
    pub id: String,
    /// The button text; `trigger_markup` replaces it.
    pub trigger: String,
    pub trigger_markup: Option<Markup>,
    /// Starts closed (Astryx opens by default).
    pub is_closed: bool,
    pub is_disabled: bool,
    /// "end" (default) or "start".
    pub chevron_position: String,
    /// Identifies it inside a [`collapsible_group`].
    pub value: String,
    /// Names a boolean Datastar signal that mirrors the open state.
    pub signal: String,
    /// The body inside a [`collapsible_group`] (standalone, pass it as `children`).
    pub content: Option<Markup>,
    pub class: String,
    pub attrs: Vec<Attr>,
}

struct GroupCtx<'a> {
    divided: bool,
    density: &'a str,
    chevron: &'a str,
    open: HashSet<&'a str>,
}

/// Renders a disclosure: a full-width trigger and `children` that show when open.
pub fn collapsible(p: &CollapsibleProps, children: Markup) -> Markup {
    render(p, children, None)
}

fn render(p: &CollapsibleProps, children: Markup, g: Option<&GroupCtx>) -> Markup {
    let id = if p.id.is_empty() {
        format!("collapsible-{}", or(&slug(&p.trigger), "item"))
    } else {
        p.id.clone()
    };
    let content_id = format!("{id}-content");
    let mut open = !p.is_closed;
    let mut position = p.chevron_position.as_str();
    let (mut divided, mut density) = (false, "");
    if let Some(g) = g {
        divided = g.divided;
        density = g.density;
        position = or(position, g.chevron);
        if !p.value.is_empty() {
            open = g.open.contains(p.value.as_str());
        }
    }
    let position = or(position, "end");
    let chevron = el(
        "span",
        attrs().set("class", "astryx-collapsible-chevron"),
        [Some(icon(&IconProps {
            name: if position == "start" {
                "chevronRight"
            } else {
                "chevronDown"
            }
            .into(),
            size: "sm".into(),
            color: "secondary".into(),
            ..Default::default()
        }))],
    );
    let (before, after) = if position == "start" {
        (Some(chevron), None)
    } else {
        (None, Some(chevron))
    };
    let label = p.trigger_markup.clone().unwrap_or_else(|| text(&p.trigger));
    let btn = attrs()
        .set("type", "button")
        .set("class", "astryx-collapsible-trigger")
        .set("data-chevron-position", position)
        .flag("data-disabled", p.is_disabled)
        .when(p.is_disabled, "aria-disabled", "true")
        .set("aria-expanded", if open { "true" } else { "false" })
        .set("aria-controls", &content_id)
        .when(p.is_disabled, "tabindex", "-1");
    let mut root = attrs()
        .set("id", &id)
        .set("class", &cls("astryx-collapsible", &p.class))
        .flag("data-divided", divided)
        .opt("data-density", density)
        .opt("data-value", &p.value);
    if !p.signal.is_empty() {
        root = root
            .set("data-effect", &format!("canon.collapse(el, ${})", p.signal))
            .set(
                "data-on:canon-toggle",
                &format!("evt.target === el && (${} = evt.detail.open)", p.signal),
            );
    }
    el(
        "div",
        root.extra(&p.attrs),
        [
            Some(el(
                "button",
                btn,
                [
                    before,
                    Some(el(
                        "span",
                        attrs().set("class", "astryx-collapsible-label"),
                        [Some(label)],
                    )),
                    after,
                ],
            )),
            Some(el(
                "div",
                attrs()
                    .set("id", &content_id)
                    .set("class", "astryx-collapsible-content")
                    .flag("hidden", !open),
                [Some(el("div", attrs(), [Some(children)]))],
            )),
        ],
    )
}

/// Props for [`collapsible_group`].
#[derive(Clone, Debug, Default)]
pub struct CollapsibleGroupProps {
    /// "single" (default, at most one open, an accordion) or "multiple".
    pub kind: String,
    /// The items open at first (by their `value`).
    pub value: Vec<String>,
    pub has_dividers: bool,
    /// "compact", "balanced" (default with dividers), "spacious".
    pub density: String,
    pub chevron_position: String,
    /// Items, each with its body in `content`.
    pub items: Vec<CollapsibleProps>,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Coordinates collapsibles by value.
pub fn collapsible_group(p: &CollapsibleGroupProps) -> Markup {
    let density = if p.density.is_empty() && p.has_dividers {
        "balanced"
    } else {
        p.density.as_str()
    };
    let kind = or(&p.kind, "single");
    let take = if kind == "single" { 1 } else { p.value.len() };
    let g = GroupCtx {
        divided: p.has_dividers,
        density,
        chevron: &p.chevron_position,
        open: p.value.iter().take(take).map(String::as_str).collect(),
    };
    let items = p
        .items
        .iter()
        .map(|it| Some(render(it, it.content.clone().unwrap_or_default(), Some(&g))));
    let a = attrs()
        .set("class", &cls("astryx-collapsible-group", &p.class))
        .set("data-type", kind)
        .flag("data-dividers", p.has_dividers)
        .opt("data-density", density)
        .extra(&p.attrs);
    el("div", a, items)
}
