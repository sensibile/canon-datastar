//! Port of Astryx `TabList` and `Tab` (packages/core/src/TabList), MIT, Meta Platforms, Inc.
//! Without a role it is a `<nav>` of links or buttons marked with aria-current; with role
//! "tablist" it follows the tabs pattern (role=tab, aria-selected, panel_id). With `signal` the
//! selection is a Datastar signal: clicking a tab sets it and every tab and TabPanel follows it.
//! static/canon.js moves focus with the arrow keys, Home and End (roving tabindex).
//! Not ported: overflow scroll arrows and edge fades, TabMenu.

use crate::Attr;
use crate::html::{attrs, cls, el, group, js_string, or, text, without_prefix};
use crate::icon::{IconProps, icon};
use maud::Markup;

/// Props for [`tab_list`].
#[derive(Clone, Debug, Default)]
pub struct TabListProps {
    /// The selected tab's value.
    pub value: String,
    /// A Datastar signal holding the selected value (initialise it to `value`).
    pub signal: String,
    pub tabs: Vec<TabProps>,
    /// "sm", "md" (default), "lg".
    pub size: String,
    /// "hug" (default) or "fill".
    pub layout: String,
    pub has_divider: bool,
    /// "" (navigation) or "tablist".
    pub role: String,
    /// Accessible name, default "Tabs".
    pub label: String,
    /// Names the list by another element.
    pub labelled_by: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// One tab of a [`tab_list`].
#[derive(Clone, Debug, Default)]
pub struct TabProps {
    pub value: String,
    pub label: String,
    /// Shows only the icon; `label` becomes the accessible name.
    pub is_label_hidden: bool,
    /// Makes a link tab (navigation lists only).
    pub href: String,
    pub id: String,
    /// Id of the panel this tab controls (role "tablist" only).
    pub panel_id: String,
    /// Built-in glyph names; `selected_icon` shows while selected.
    pub icon: String,
    pub selected_icon: String,
    pub end_content: Option<Markup>,
    pub is_disabled: bool,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Renders a row of tabs.
pub fn tab_list(p: &TabListProps) -> Markup {
    let size = or(&p.size, "md");
    let layout = or(&p.layout, "hug");
    let is_tab_list = p.role == "tablist";
    let label = if !p.labelled_by.is_empty() && p.label.is_empty() {
        ""
    } else {
        or(&p.label, "Tabs")
    };
    let mut root = attrs()
        .set("class", &cls("astryx-tab-list", &p.class))
        .set("data-size", size)
        .set("data-layout", layout)
        .flag("data-divider", p.has_divider);
    let mut strip = attrs().set("class", "astryx-tab-strip");
    let tag = if is_tab_list {
        strip = strip
            .set("role", "tablist")
            .opt("aria-label", label)
            .opt("aria-labelledby", &p.labelled_by);
        "div"
    } else {
        root = root
            .opt("aria-label", label)
            .opt("aria-labelledby", &p.labelled_by);
        "nav"
    };
    let tabs = p
        .tabs
        .iter()
        .map(|t| Some(tab(t, p, size, layout, is_tab_list)));
    el(tag, root.extra(&p.attrs), [Some(el("div", strip, tabs))])
}

fn tab(t: &TabProps, p: &TabListProps, size: &str, layout: &str, is_tab_list: bool) -> Markup {
    let selected = t.value == p.value;
    let is_link = !t.href.is_empty() && !is_tab_list;
    let mut a = attrs()
        .set("class", &cls("astryx-tab", &t.class))
        .opt("id", &t.id)
        .set("data-tab-value", &t.value)
        .set("data-size", size)
        .set("data-layout", layout)
        .when(selected, "data-selected", "selected")
        .when(t.is_label_hidden, "aria-label", &t.label)
        .when(t.is_disabled, "aria-disabled", "true");
    a = if is_tab_list {
        a.set("role", "tab")
            .set("aria-selected", if selected { "true" } else { "false" })
            .opt("aria-controls", &t.panel_id)
    } else {
        a.when(selected, "aria-current", "true")
    };
    a = a.set("tabindex", if selected { "0" } else { "-1" });
    let tag = if is_link {
        a = a.when(!t.is_disabled, "href", &t.href);
        "a"
    } else {
        a = a.set("type", "button");
        "button"
    };
    if !p.signal.is_empty() && !t.is_disabled {
        let is = format!("${} === {}", p.signal, js_string(&t.value));
        a = a.set("data-attr:data-selected", &format!("{is} && 'selected'"));
        a = if is_tab_list {
            a.set("data-attr:aria-selected", &format!("String({is})"))
        } else {
            a.set("data-attr:aria-current", &format!("{is} && 'true'"))
        };
        a = a.set("data-attr:tabindex", &format!("{is} ? 0 : -1")).set(
            "data-on:click",
            &format!("${} = {}", p.signal, js_string(&t.value)),
        );
    }
    a = if t.is_disabled {
        a.extra(&without_prefix(&t.attrs, "data-on:"))
    } else {
        a.extra(&t.attrs)
    };

    let tab_icon = |name: &str, state: &str| {
        el(
            "span",
            attrs()
                .set("class", "astryx-tab-icon")
                .opt("data-icon-state", state),
            [Some(icon(&IconProps::sized(name, "sm")))],
        )
    };
    let icons = match (t.icon.is_empty(), t.selected_icon.is_empty()) {
        (false, false) => Some(group([
            Some(tab_icon(&t.icon, "idle")),
            Some(tab_icon(&t.selected_icon, "selected")),
        ])),
        (false, true) => Some(tab_icon(&t.icon, "")),
        (true, false) => Some(tab_icon(&t.selected_icon, "selected")),
        (true, true) => None,
    };
    let label = (!t.is_label_hidden && !t.label.is_empty()).then(|| {
        el(
            "span",
            attrs().set("class", "astryx-tab-label"),
            [
                Some(el(
                    "span",
                    attrs().set("class", "astryx-tab-label-text"),
                    [Some(text(&t.label))],
                )),
                Some(el(
                    "span",
                    attrs()
                        .set("class", "astryx-tab-label-sizer")
                        .set("aria-hidden", "true"),
                    [Some(text(&t.label))],
                )),
            ],
        )
    });
    let end = t
        .end_content
        .clone()
        .map(|e| el("span", attrs().set("class", "astryx-tab-end"), [Some(e)]));
    el(
        tag,
        a,
        [
            Some(el(
                "span",
                attrs()
                    .set("class", "astryx-tab-hover")
                    .set("aria-hidden", "true"),
                [],
            )),
            icons,
            label,
            end,
            Some(el("span", attrs().set("class", "astryx-tab-indicator"), [])),
        ],
    )
}

/// Props for [`tab_panel`].
#[derive(Clone, Debug, Default)]
pub struct TabPanelProps {
    pub id: String,
    /// Id of the tab that labels this panel.
    pub tab_id: String,
    /// The tab value this panel belongs to.
    pub value: String,
    /// Shows the panel on first render.
    pub is_selected: bool,
    /// The TabList's signal; the panel shows while it equals `value`.
    pub signal: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// The content for one tab (role="tabpanel").
pub fn tab_panel(p: &TabPanelProps, children: Markup) -> Markup {
    let mut a = attrs()
        .set("class", &cls("astryx-tab-panel", &p.class))
        .opt("id", &p.id)
        .set("role", "tabpanel")
        .opt("aria-labelledby", &p.tab_id)
        .set("tabindex", "0")
        .when(!p.is_selected, "style", "display: none");
    if !p.signal.is_empty() {
        a = a.set(
            "data-show",
            &format!("${} === {}", p.signal, js_string(&p.value)),
        );
    }
    el("div", a.extra(&p.attrs), [Some(children)])
}
