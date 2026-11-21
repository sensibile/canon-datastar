//! MetadataList and its items. Ported from Astryx (packages/core/src/MetadataList), MIT, Meta
//! Platforms, Inc. Items past `max_num_of_items` render hidden behind a "Show more" button that
//! static/canon.js toggles.

use crate::Attr;
use crate::html::{attrs, cls, decl, el, or, text};
use crate::types::slug;
use maud::Markup;

/// One label/value pair.
#[derive(Clone, Debug, Default)]
pub struct MetadataListItem {
    pub label: String,
    pub icon: Option<Markup>,
    /// The value markup; `text` is used when it's `None`.
    pub value: Option<Markup>,
    pub text: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Props for [`metadata_list`].
#[derive(Clone, Debug, Default)]
pub struct MetadataListProps {
    /// Id of the list (for the toggle's aria-controls). Defaults to "metadata-" + a slug of `title`.
    pub id: String,
    /// "single" (default), "multi", or a count such as "3".
    pub columns: String,
    /// "start" (default for a single column) or "top" (default for multi).
    pub label_position: String,
    /// A CSS length for inline labels in a single column.
    pub label_width: String,
    pub max_num_of_items: usize,
    /// "vertical" (default) or "horizontal".
    pub orientation: String,
    pub title: String,
    pub title_markup: Option<Markup>,
    pub items: Vec<MetadataListItem>,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Renders a description list.
pub fn metadata_list(p: &MetadataListProps) -> Markup {
    let columns = or(&p.columns, "single");
    let n: i64 = columns.parse().unwrap_or(0);
    let is_multi = columns == "multi" || n > 1;
    let is_single = columns == "single" || n == 1;
    let orientation = or(&p.orientation, "vertical");
    let horizontal = orientation == "horizontal";
    let position = or(&p.label_position, if is_multi { "top" } else { "start" });
    let stacked = position == "top" || horizontal;
    let layout = match () {
        _ if horizontal => "horizontal",
        _ if position == "top" && is_single => "stacked-single",
        _ if position == "top" => "stacked-multi",
        _ if is_single => "single",
        _ => "multi",
    };
    let template = if horizontal {
        String::new()
    } else if n > 1 {
        if position == "top" {
            format!("repeat({columns}, 1fr)")
        } else {
            format!("repeat({columns}, auto minmax(0, 1fr))")
        }
    } else if position != "top" && !p.label_width.is_empty() {
        format!("{} minmax(0, 1fr)", p.label_width)
    } else {
        String::new()
    };
    let max = if horizontal { 0 } else { p.max_num_of_items };
    let exceeds = max > 0 && p.items.len() > max;
    let id = if p.id.is_empty() {
        format!("metadata-{}", or(&slug(&p.title), "list"))
    } else {
        p.id.clone()
    };

    let mut items = Vec::new();
    for (i, it) in p.items.iter().enumerate() {
        let hidden = exceeds && i >= max;
        let icon = || {
            it.icon.clone().map(|ic| {
                el(
                    "span",
                    attrs().set("class", "astryx-metadata-list-icon"),
                    [Some(ic)],
                )
            })
        };
        let value = it.value.clone().unwrap_or_else(|| text(&it.text));
        if stacked {
            let a = attrs()
                .set("class", &cls("astryx-metadata-list-item", &it.class))
                .set("data-metadata-item", "")
                .set("data-stacked", "")
                .flag("hidden", hidden)
                .extra(&it.attrs);
            items.push(Some(el(
                "div",
                a,
                [
                    Some(el(
                        "dt",
                        attrs().set("class", "astryx-metadata-list-label"),
                        [icon(), Some(text(&it.label))],
                    )),
                    Some(el(
                        "dd",
                        attrs().set("class", "astryx-metadata-list-value"),
                        [Some(value)],
                    )),
                ],
            )));
        } else {
            let a = attrs()
                .set(
                    "class",
                    &cls(
                        "astryx-metadata-list-item astryx-metadata-list-label",
                        &it.class,
                    ),
                )
                .set("data-metadata-item", "")
                .set("data-inline", "")
                .flag("hidden", hidden)
                .extra(&it.attrs);
            items.push(Some(el("dt", a, [icon(), Some(text(&it.label))])));
            items.push(Some(el(
                "dd",
                attrs()
                    .set("class", "astryx-metadata-list-value")
                    .set("data-inline", "")
                    .flag("hidden", hidden),
                [Some(value)],
            )));
        }
    }
    let title = p
        .title_markup
        .clone()
        .or_else(|| (!p.title.is_empty()).then(|| text(&p.title)))
        .map(|t| {
            el(
                "div",
                attrs().set("class", "astryx-metadata-list-title"),
                [Some(t)],
            )
        });
    let toggle = exceeds.then(|| {
        el(
            "button",
            attrs()
                .set("type", "button")
                .set("class", "astryx-metadata-list-toggle")
                .set("aria-controls", &id)
                .set("aria-expanded", "false"),
            [Some(text("Show more"))],
        )
    });
    let a = attrs()
        .set("class", &cls("astryx-metadata-list", &p.class))
        .set("data-columns", columns)
        .set("data-orientation", orientation)
        .extra(&p.attrs);
    let list = el(
        "dl",
        attrs()
            .set("id", &id)
            .set("class", "astryx-metadata-list-list")
            .set("data-layout", layout)
            .when(exceeds, "data-max", &max.to_string())
            .opt("style", &decl("grid-template-columns", &template)),
        items,
    );
    el("div", a, [title, Some(list), toggle])
}
