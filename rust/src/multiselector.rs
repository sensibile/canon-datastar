//! MultiSelector. Ported from Astryx `MultiSelector` (packages/core/src/MultiSelector), MIT,
//! Meta Platforms, Inc. A select-many control: a combobox button that opens a listbox of checkbox
//! rows in a native popover. The values live in a hidden `<select multiple>` (bind an array signal
//! to it with `value_attrs`, e.g. `("data-bind:tags", "")`); static/canon.js keeps the trigger and
//! rows in step. The list stays open while you toggle; selected options move to the top of their
//! group when it opens. `has_select_all` adds a "Select all" row; `has_create` (with `has_search`)
//! offers the typed query as a new value: it's added to the select and the control fires a
//! bubbling "astryx-create" event with detail {query}.
//! The value is in option order, not the order of picking.
//! Not ported: option actions (grid mode), renderTrigger/formatValue, the bottom-sheet
//! presentation, changeAction, disabledMessage and labelTooltip.

use crate::Attr;
use crate::html::{attrs, el, or, text};
use crate::input_group::InputGroupMember;
use crate::misc::{BadgeProps, badge};
use crate::selector::{
    SelectorIds, SelectorOption, SelectorProps, ShellParts, option_label, selectable_options,
    selector_listbox, selector_row, selector_shell,
};
use crate::types::FieldStatus;
use maud::Markup;
use std::collections::HashSet;

/// Props for [`multi_selector`]. Fields shared with [`SelectorProps`] mean the same.
#[derive(Clone, Debug, Default)]
pub struct MultiSelectorProps {
    pub id: String,
    pub label: String,
    pub options: Vec<SelectorOption>,
    /// The selected values; values not among `options` are kept (shown by value).
    pub value: Vec<String>,
    /// A clear (×) button while anything is selected.
    pub has_clear: bool,
    /// A row that selects (or clears) every enabled option that's shown.
    pub has_select_all: bool,
    /// Default "Select all".
    pub select_all_label: String,
    pub has_search: bool,
    /// Offer the search query as a new value when no option has that label.
    pub has_create: bool,
    /// "count" (default, "3 selected"), "labels" ("A, B, C, +2") or "badges".
    pub trigger_display: String,
    /// Badges shown before "+N"; 0 means 3.
    pub max_badges: usize,
    pub search_placeholder: String,
    pub empty_text: String,
    pub empty_search_text: String,
    pub is_label_hidden: bool,
    pub description: String,
    pub is_optional: bool,
    pub is_required: bool,
    pub is_disabled: bool,
    pub is_read_only: bool,
    pub is_loading: bool,
    /// Shown while nothing is selected. Default "Select…".
    pub placeholder: String,
    pub size: String,
    pub variant: String,
    pub status: Option<FieldStatus>,
    pub status_variant: String,
    pub width: String,
    pub start_icon: String,
    /// "start" (default) or "end" for the checkbox.
    pub indicator_position: String,
    pub placement: String,
    pub is_default_open: bool,
    /// On the hidden select.
    pub name: String,
    pub form: String,
    pub class: String,
    pub control_class: String,
    pub attrs: Vec<Attr>,
    pub trigger_attrs: Vec<Attr>,
    /// On the hidden select, e.g. `("data-bind:tags", "")`.
    pub value_attrs: Vec<Attr>,
    pub group: Option<InputGroupMember>,
}

/// The trigger text for the "count" and "labels" displays.
fn multi_selector_text(display: &str, labels: &[String], placeholder: &str) -> String {
    if labels.is_empty() {
        return placeholder.into();
    }
    if display == "labels" {
        let shown = &labels[..labels.len().min(3)];
        let rest = labels.len() - shown.len();
        return if rest > 0 {
            format!("{}, +{rest}", shown.join(", "))
        } else {
            shown.join(", ")
        };
    }
    format!("{} selected", labels.len())
}

fn a(name: &str, value: &str) -> Attr {
    (name.into(), value.into())
}

/// Renders the labelled control, its hidden select and the popover listbox.
pub fn multi_selector(p: &MultiSelectorProps) -> Markup {
    let all = selectable_options(&p.options);
    let chosen: HashSet<&str> = p.value.iter().map(String::as_str).collect();
    // The select's options: every option in order, then values not among them.
    let mut select_opts: Vec<Option<Markup>> = Vec::new();
    let mut labels: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut add_option = |value: &str, label: &str| {
        if !seen.insert(value.to_string()) {
            return;
        }
        let on = chosen.contains(value);
        if on {
            labels.push(label.to_string());
        }
        select_opts.push(Some(el(
            "option",
            attrs().set("value", value).flag("selected", on),
            [Some(text(label))],
        )));
    };
    for o in &all {
        add_option(&o.value, option_label(o));
    }
    for v in &p.value {
        add_option(v, v);
    }
    let display = or(&p.trigger_display, "count");
    let max_badges = if p.max_badges == 0 { 3 } else { p.max_badges };
    let placeholder = or(&p.placeholder, "Select…");
    let select_all_label = or(&p.select_all_label, "Select all");
    let enabled: Vec<_> = all.iter().filter(|o| !o.is_disabled).collect();
    let enabled_on = enabled
        .iter()
        .filter(|o| chosen.contains(o.value.as_str()))
        .count();
    let all_on = !enabled.is_empty() && enabled_on == enabled.len();
    let select_all_state = if all_on {
        "checked"
    } else if enabled_on > 0 {
        "indeterminate"
    } else {
        "unchecked"
    };
    let indicator = or(&p.indicator_position, "start");
    let shell = SelectorProps {
        id: p.id.clone(),
        label: p.label.clone(),
        is_label_hidden: p.is_label_hidden,
        description: p.description.clone(),
        is_optional: p.is_optional,
        is_required: p.is_required,
        is_disabled: p.is_disabled,
        is_read_only: p.is_read_only,
        is_loading: p.is_loading,
        size: p.size.clone(),
        variant: p.variant.clone(),
        status: p.status.clone(),
        status_variant: p.status_variant.clone(),
        width: p.width.clone(),
        start_icon: p.start_icon.clone(),
        has_search: p.has_search,
        search_placeholder: p.search_placeholder.clone(),
        placement: p.placement.clone(),
        is_default_open: p.is_default_open,
        class: p.class.clone(),
        control_class: p.control_class.clone(),
        attrs: p.attrs.clone(),
        trigger_attrs: p.trigger_attrs.clone(),
        group: p.group.clone(),
        has_clear: p.has_clear,
        ..Default::default()
    };
    let has_value = !labels.is_empty();
    let trigger_body = |ids: &SelectorIds| {
        let at = attrs()
            .set("id", &ids.value)
            .set("class", "astryx-selector-value-text")
            .set("data-placeholder-text", placeholder)
            .set("data-display", display)
            .when(
                display == "badges",
                "data-max-badges",
                &max_badges.to_string(),
            )
            .flag("data-badges", display == "badges" && has_value);
        if display != "badges" || !has_value {
            return el(
                "span",
                at,
                [Some(text(&multi_selector_text(
                    display,
                    &labels,
                    placeholder,
                )))],
            );
        }
        let mut badges: Vec<Option<Markup>> = labels[..labels.len().min(max_badges)]
            .iter()
            .map(|l| {
                Some(badge(&BadgeProps {
                    label: l.clone(),
                    ..Default::default()
                }))
            })
            .collect();
        if labels.len() > max_badges {
            badges.push(Some(el(
                "span",
                attrs().set("class", "astryx-multi-selector-overflow"),
                [Some(text(&format!("+{}", labels.len() - max_badges)))],
            )));
        }
        el("span", at, badges)
    };
    let value = el(
        "select",
        attrs()
            .flag("multiple", true)
            .set("class", "astryx-multi-selector-value")
            .flag("hidden", true)
            .set("tabindex", "-1")
            .set("aria-hidden", "true")
            .when(!p.is_disabled && !p.name.is_empty(), "name", &p.name)
            .when(!p.is_disabled && !p.form.is_empty(), "form", &p.form)
            .extra(&p.value_attrs),
        select_opts,
    );
    let listbox = |ids: &SelectorIds, size: &str| {
        let mut lead = Vec::new();
        if p.has_create && p.has_search {
            lead.push(selector_row(
                &format!("{}-create", ids.listbox),
                &SelectorOption::default(),
                size,
                indicator,
                false,
                "none",
                &[a("data-create", ""), a("hidden", "hidden")],
            ));
        }
        if p.has_select_all {
            let mut extra = vec![a("data-select-all", ""), a("data-state", select_all_state)];
            if select_all_state == "indeterminate" {
                extra.push(a(
                    "aria-label",
                    &format!("{select_all_label}, partially selected"),
                ));
            }
            if all.is_empty() {
                extra.push(a("hidden", "hidden"));
            }
            lead.push(selector_row(
                &format!("{}-select-all", ids.listbox),
                &SelectorOption {
                    label: select_all_label.into(),
                    ..Default::default()
                },
                size,
                indicator,
                all_on,
                "checkbox",
                &extra,
            ));
        }
        selector_listbox(
            ids,
            &p.label,
            lead,
            &p.options,
            size,
            indicator,
            &|o: &SelectorOption| chosen.contains(o.value.as_str()),
            or(&p.empty_text, "No options"),
            or(&p.empty_search_text, "No results found"),
            "checkbox",
        )
    };
    selector_shell(
        &shell,
        ShellParts {
            class: "astryx-multi-selector astryx-selector-control",
            multiple: true,
            has_value,
            clear: format!("Clear all {}", p.label),
            trigger_body: &trigger_body,
            value,
            listbox: &listbox,
        },
    )
}
