//! Item, the row primitive behind lists, menus and choice lists. Ported from Astryx
//! (packages/core/src/Item), MIT, Meta Platforms, Inc.
//! Not ported: swipe actions, marker columns, list-row link bands, touch press feedback.

use crate::Attr;
use crate::html::{attrs, cls, el, or, text, without_prefix};
use maud::Markup;

/// Props for [`item`].
#[derive(Clone, Debug, Default)]
pub struct ItemProps {
    /// "div" (default), "li" or "span".
    pub as_: String,
    pub label: String,
    /// Replaces `label` with markup (no truncation unless `label_lines` is set).
    pub label_content: Option<Markup>,
    pub description: String,
    pub description_content: Option<Markup>,
    pub start_content: Option<Markup>,
    pub end_content: Option<Markup>,
    /// "center" (default), "start" or "first-line".
    pub align: String,
    /// "balanced" (default), "compact" or "spacious".
    pub density: String,
    /// Clamps the label to this many lines; plain labels truncate to one line by default.
    pub label_lines: u32,
    pub description_lines: u32,
    /// "inline" puts the description on the label's line.
    pub layout: String,
    /// Makes the row a link.
    pub href: String,
    pub target: String,
    pub rel: String,
    /// Makes the row a button and goes on it, e.g. `("data-on:click", "@post('/x')")`.
    /// `Some(vec![])` still renders the button.
    pub action: Option<Vec<Attr>>,
    /// Row clicks are forwarded (by canon.js) to the control in `start_content`.
    pub is_delegate: bool,
    pub is_highlighted: bool,
    pub is_selected: bool,
    pub is_disabled: bool,
    pub label_id: String,
    pub description_id: String,
    /// Set when a parent widget (listbox, menu) owns the row's semantics.
    pub role: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

const ARIA_SELECTED_ROLES: [&str; 7] = [
    "option",
    "tab",
    "row",
    "gridcell",
    "columnheader",
    "rowheader",
    "treeitem",
];

fn truncation(lines: u32, plain: bool) -> &'static str {
    match lines {
        1 => "single",
        n if n > 1 => "multi",
        _ if plain => "single",
        _ => "",
    }
}

fn clamp(lines: u32) -> String {
    if lines > 1 {
        format!("-webkit-line-clamp: {lines}")
    } else {
        String::new()
    }
}

/// Renders a row: start content, label + description, end content. A button (`action`),
/// a link (`href`), or a row that forwards clicks to a control inside it (`is_delegate`).
pub fn item(p: &ItemProps) -> Markup {
    let has_role = !p.role.is_empty();
    let is_link = !p.href.is_empty() && !has_role && !p.is_delegate;
    let is_button = !is_link && p.action.is_some() && !has_role && !p.is_delegate;
    let interactive = !p.href.is_empty() || p.action.is_some() || p.is_delegate;
    let has_desc = !p.description.is_empty() || p.description_content.is_some();
    let inline = p.layout == "inline" && has_desc;
    let allows_selected = has_role && ARIA_SELECTED_ROLES.contains(&p.role.as_str());
    let current = if p.is_selected && !allows_selected {
        "true"
    } else {
        ""
    };
    let current_on_control = !has_role && (is_link || is_button);
    let align = or(&p.align, "center");

    let label = p.label_content.clone().unwrap_or_else(|| text(&p.label));
    let mut body = vec![Some(el(
        "span",
        attrs()
            .opt("id", &p.label_id)
            .set("class", "astryx-item-label")
            .flag("data-inline", inline)
            .opt(
                "data-truncate",
                truncation(p.label_lines, p.label_content.is_none()),
            )
            .opt("style", &clamp(p.label_lines)),
        [Some(label)],
    ))];
    if has_desc {
        let desc = p
            .description_content
            .clone()
            .unwrap_or_else(|| text(&p.description));
        body.push(Some(el(
            "span",
            attrs()
                .opt("id", &p.description_id)
                .set("class", "astryx-item-description")
                .flag("data-inline", inline)
                .opt(
                    "data-truncate",
                    truncation(
                        p.description_lines,
                        p.description_content.is_none() || inline,
                    ),
                )
                .opt("style", &clamp(p.description_lines)),
            [Some(desc)],
        )));
    }
    let content = if is_link {
        let mut a = attrs()
            .set("class", "astryx-item-action")
            .flag("data-inline", inline);
        if p.is_disabled {
            a = a
                .set("role", "link")
                .set("aria-disabled", "true")
                .set("tabindex", "0");
        } else {
            let rel = if p.rel.is_empty() && p.target == "_blank" {
                "noopener noreferrer"
            } else {
                &p.rel
            };
            a = a
                .set("href", &p.href)
                .opt("target", &p.target)
                .opt("rel", rel);
        }
        el("a", a.opt("aria-current", current), body)
    } else if is_button {
        let action = p.action.clone().unwrap_or_default();
        let action = if p.is_disabled {
            without_prefix(&action, "data-on:")
        } else {
            action
        };
        el(
            "button",
            attrs()
                .set("type", "button")
                .set("class", "astryx-item-action")
                .flag("data-inline", inline)
                .when(p.is_disabled, "aria-disabled", "true")
                .opt("aria-current", current)
                .extra(&action),
            body,
        )
    } else {
        el(
            "span",
            attrs()
                .set("class", "astryx-item-content")
                .flag("data-inline", inline),
            body,
        )
    };
    let side = |class: &str, m: &Option<Markup>| {
        m.clone().map(|m| {
            el(
                "span",
                attrs()
                    .set("class", class)
                    .flag("data-first-line", align == "first-line"),
                [Some(m)],
            )
        })
    };
    let a = attrs()
        .set("class", &cls("astryx-item", &p.class))
        .set("data-density", or(&p.density, "balanced"))
        .set("data-align", align)
        .flag("data-interactive", interactive)
        .flag("data-delegate", p.is_delegate)
        .flag("data-highlighted", p.is_highlighted)
        .flag("data-selected", p.is_selected)
        .flag("data-disabled", p.is_disabled)
        .flag("data-has-role", has_role)
        .opt("role", &p.role)
        .when(allows_selected && p.is_selected, "aria-selected", "true")
        .when(
            !current_on_control && !current.is_empty(),
            "aria-current",
            current,
        )
        .when(p.is_disabled, "aria-disabled", "true")
        .extra(&p.attrs);
    el(
        or(&p.as_, "div"),
        a,
        [
            side("astryx-item-start", &p.start_content),
            Some(content),
            side("astryx-item-end", &p.end_content),
        ],
    )
}
