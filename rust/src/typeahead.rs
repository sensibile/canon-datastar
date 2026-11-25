//! Typeahead and Tokenizer. Ported from Astryx `Typeahead` and `Tokenizer`
//! (packages/core/src/Typeahead, Tokenizer), MIT, Meta Platforms, Inc. Both are a search field
//! over a results list in a popover.
//!
//! Results come from a static list (`items`, filtered in the browser by label and keywords) or,
//! with `is_remote`, from the server: after `debounce_ms` the control fires a bubbling
//! "astryx-search" event with detail {query}; answer it by patching [`typeahead_results`] (same
//! id) into the page. Answers for an older query are ignored.
//!
//! Typeahead picks one item: its id lives in a hidden input (bind a string signal with
//! `value_attrs`) and shows as a token. Tokenizer collects many: their ids live in a hidden
//! `<select multiple>` (bind an array signal), in the order they were added; `has_create` offers
//! the query as a new item and fires "astryx-create" with detail {query}.
//! Not ported: renderItem/renderToken, the custom menu, result grouping, token overflow,
//! disabledMessage and labelTooltip.

use crate::Attr;
use crate::button::{ButtonProps, button};
use crate::field::{FieldProps, field};
use crate::html::{attrs, cls, decl, el, group, join_ids, or, text, void};
use crate::icon::{IconProps, icon};
use crate::input_group::InputGroupMember;
use crate::misc::{SpinnerProps, VisuallyHiddenProps, spinner, visually_hidden};
use crate::selector::icon_or_none;
use crate::token::{TokenProps, token};
use crate::types::{FieldStatus, control_id};
use maud::{Markup, html};
use std::collections::HashSet;

/// One searchable result.
#[derive(Clone, Debug, Default)]
pub struct TypeaheadItem {
    pub id: String,
    pub label: String,
    /// A second line under the label.
    pub description: String,
    /// A built-in glyph name shown before the label.
    pub icon: String,
    /// Extra strings the item matches on (static lists). They can't contain "|".
    pub keywords: Vec<String>,
}

impl TypeaheadItem {
    pub fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            ..Default::default()
        }
    }
}

/// Props for [`typeahead`].
#[derive(Clone, Debug, Default)]
pub struct TypeaheadProps {
    /// Id of the search input; defaults to "astryx-" + a slug of the label. Derived ids:
    /// id+"-listbox", id+"-popup", id+"-listbox-item-N".
    pub id: String,
    pub label: String,
    /// The static list searched in the browser (with `is_remote`, the entries for an empty query).
    pub items: Vec<TypeaheadItem>,
    pub is_remote: bool,
    /// Delay before "astryx-search"; 0 means 150.
    pub debounce_ms: u32,
    /// The picked item's id.
    pub value: String,
    /// The picked item's label when it isn't among `items`.
    pub value_label: String,
    /// Default "Search…".
    pub placeholder: String,
    pub has_entries_on_focus: bool,
    /// 0 means 10.
    pub max_menu_items: u32,
    /// 0 means 1.
    pub min_query_length: u32,
    /// Default "No results found".
    pub empty_search_text: String,
    pub is_label_hidden: bool,
    pub description: String,
    pub is_optional: bool,
    pub is_required: bool,
    /// Makes the input read-only (aria-disabled) and drops the hidden input's name.
    pub is_disabled: bool,
    /// Hides the clear (×) button shown while an item is picked.
    pub no_clear: bool,
    pub has_auto_focus: bool,
    /// "sm", "md" (default), "lg".
    pub size: String,
    pub status: Option<FieldStatus>,
    /// "attached" (default) or "detached".
    pub status_variant: String,
    pub start_icon: String,
    pub width: String,
    /// On the hidden value input.
    pub name: String,
    pub form: String,
    pub class: String,
    pub control_class: String,
    /// On the Field container, or on the control in a group (put data-on:astryx-search here).
    pub attrs: Vec<Attr>,
    /// On the search input.
    pub input_attrs: Vec<Attr>,
    /// On the hidden value input.
    pub value_attrs: Vec<Attr>,
    pub group: Option<InputGroupMember>,
}

/// Props for [`tokenizer`]. Fields shared with [`TypeaheadProps`] mean the same.
#[derive(Clone, Debug, Default)]
pub struct TokenizerProps {
    pub id: String,
    pub label: String,
    pub items: Vec<TypeaheadItem>,
    pub is_remote: bool,
    pub debounce_ms: u32,
    /// The chosen items, in order.
    pub value: Vec<TypeaheadItem>,
    /// Caps the number of tokens (0: no cap).
    pub max_entries: u32,
    pub placeholder: String,
    pub has_entries_on_focus: bool,
    pub max_menu_items: u32,
    pub min_query_length: u32,
    /// The results list width (CSS length); default the field's width.
    pub menu_width: String,
    pub empty_search_text: String,
    /// Offer the query itself as a new item.
    pub has_create: bool,
    /// A "Clear all" button while there are tokens.
    pub has_clear: bool,
    pub is_label_hidden: bool,
    pub description: String,
    pub is_optional: bool,
    pub is_required: bool,
    pub is_disabled: bool,
    pub has_auto_focus: bool,
    pub size: String,
    pub status: Option<FieldStatus>,
    pub status_variant: String,
    pub start_icon: String,
    pub width: String,
    /// On the hidden select.
    pub name: String,
    pub form: String,
    pub class: String,
    pub control_class: String,
    pub attrs: Vec<Attr>,
    pub input_attrs: Vec<Attr>,
    pub value_attrs: Vec<Attr>,
    pub group: Option<InputGroupMember>,
}

/// Props for [`typeahead_results`].
#[derive(Clone, Debug, Default)]
pub struct TypeaheadResultsProps {
    /// The Typeahead's or Tokenizer's id.
    pub id: String,
    /// The query these results answer (as sent in the "astryx-search" event).
    pub query: String,
    pub items: Vec<TypeaheadItem>,
}

/// A results list for a remote Typeahead or Tokenizer; patch it into the page (it replaces the
/// element with id id+"-listbox").
pub fn typeahead_results(p: &TypeaheadResultsProps) -> Markup {
    typeahead_listbox(&format!("{}-listbox", p.id), &p.items, Some(&p.query))
}

fn typeahead_listbox(id: &str, items: &[TypeaheadItem], query: Option<&str>) -> Markup {
    let mut rows = vec![Some(el(
        "div",
        attrs()
            .set("role", "option")
            .set("aria-selected", "false")
            .set("aria-disabled", "true")
            .set("class", "astryx-typeahead-empty-state")
            .flag("hidden", true),
        [None],
    ))];
    for (i, it) in items.iter().enumerate() {
        let label = or(&it.label, &it.id);
        let desc = (!it.description.is_empty()).then(|| {
            el(
                "span",
                attrs().set("class", "astryx-typeahead-item-description"),
                [Some(text(&it.description))],
            )
        });
        rows.push(Some(el(
            "div",
            attrs()
                .set("id", &format!("{id}-item-{i}"))
                .set("role", "option")
                .set("aria-selected", "false")
                .set("tabindex", "-1")
                .set("class", "astryx-typeahead-option")
                .set("data-id", &it.id)
                .set("data-label", label)
                .opt("data-keywords", &it.keywords.join("|"))
                .flag("hidden", true),
            [
                Some(el(
                    "span",
                    attrs().set("class", "astryx-typeahead-option-content"),
                    [Some(el(
                        "div",
                        attrs().set("class", "astryx-typeahead-item"),
                        [
                            icon_or_none(&it.icon),
                            Some(el(
                                "div",
                                attrs().set("class", "astryx-typeahead-item-content"),
                                [
                                    Some(el(
                                        "span",
                                        attrs().set("class", "astryx-typeahead-item-label"),
                                        [Some(text(label))],
                                    )),
                                    desc,
                                ],
                            )),
                        ],
                    ))],
                )),
                Some(el(
                    "span",
                    attrs().set("class", "astryx-typeahead-check"),
                    [Some(icon(&IconProps {
                        color: "primary".into(),
                        ..IconProps::sized("check", "sm")
                    }))],
                )),
            ],
        )));
    }
    let mut a = attrs()
        .set("id", id)
        .set("role", "listbox")
        .set("aria-label", "Search results")
        .set("tabindex", "-1")
        .set("class", "astryx-typeahead-dropdown");
    if let Some(q) = query {
        a = a.set("data-query", q);
    }
    el("div", a, rows)
}

/// What Typeahead and Tokenizer share.
struct SearchProps<'a> {
    id: &'a str,
    label: &'a str,
    items: &'a [TypeaheadItem],
    is_remote: bool,
    debounce_ms: u32,
    has_entries_on_focus: bool,
    max_menu_items: u32,
    min_query_length: u32,
    empty_search_text: &'a str,
    is_label_hidden: bool,
    description: &'a str,
    is_optional: bool,
    is_required: bool,
    is_disabled: bool,
    has_auto_focus: bool,
    size: &'a str,
    status: &'a Option<FieldStatus>,
    status_variant: &'a str,
    start_icon: &'a str,
    width: &'a str,
    class: &'a str,
    control_class: &'a str,
    attrs: &'a [Attr],
    input_attrs: &'a [Attr],
    group: Option<&'a InputGroupMember>,
}

/// Renders what goes before the input: (ids, token labelledby, describedby).
type LeadFn<'a> = &'a dyn Fn(&SearchIds, &str, &str) -> Vec<Option<Markup>>;

struct SearchIds {
    input: String,
    listbox: String,
    popup: String,
    label: String,
    input_label: String,
    desc: String,
    status: String,
    live: String,
}

/// What Typeahead and Tokenizer put into the shared control.
struct SearchParts<'a> {
    kind: &'a str,
    has_value: bool,
    clear: bool,
    clear_label: &'a str,
    lane_input: bool,
    lead: LeadFn<'a>,
    value: Markup,
    extra_ctl: &'a dyn Fn(crate::html::Attrs) -> crate::html::Attrs,
    extra_in: &'a dyn Fn(crate::html::Attrs) -> crate::html::Attrs,
    describe: Vec<String>,
    after_in: Vec<Option<Markup>>,
    menu_width: &'a str,
    popup_tail: Option<Markup>,
}

fn search_shell(p: &SearchProps, parts: SearchParts) -> Markup {
    let g = p.group;
    let is_disabled = p.is_disabled || g.is_some_and(|g| g.is_disabled);
    let size_owned;
    let size = match g {
        Some(g) => {
            size_owned = or(p.size, &g.size).to_string();
            size_owned.as_str()
        }
        None => p.size,
    };
    let size = or(size, "md");
    let description = if g.is_some() { "" } else { p.description };
    let id = control_id(p.id, p.label);
    let ids = SearchIds {
        input: id.clone(),
        listbox: format!("{id}-listbox"),
        popup: format!("{id}-popup"),
        label: format!("{id}-label"),
        input_label: format!("{id}-input-label"),
        desc: format!("{id}-desc"),
        status: format!("{id}-status"),
        live: format!("{id}-live"),
    };
    let status_kind = p.status.as_ref().map(|s| s.kind.as_str()).unwrap_or("");
    let mut has_message = p.status.as_ref().is_some_and(|s| !s.message.is_empty());
    let desc_id = if !description.is_empty() {
        ids.desc.clone()
    } else {
        String::new()
    };
    let (labelled_by, group_desc) = match g {
        Some(g) => {
            has_message = false;
            (
                join_ids(&[&g.label_id, &ids.input_label]),
                g.described_by.clone(),
            )
        }
        None => (String::new(), String::new()),
    };
    let mut describe: Vec<&str> = vec![
        &group_desc,
        &desc_id,
        if has_message { &ids.status } else { "" },
    ];
    describe.extend(parts.describe.iter().map(String::as_str));
    let described_by = join_ids(&describe);
    let required = p.is_required && !p.is_optional;
    let ia = attrs()
        .set("id", &id)
        .set("type", "text")
        .set("role", "combobox")
        .set("class", "astryx-search-input")
        .set("aria-expanded", "false")
        .set("aria-controls", &ids.listbox)
        .set("aria-autocomplete", "list")
        .opt("aria-labelledby", &labelled_by)
        .opt("aria-describedby", &described_by)
        .when(required, "aria-required", "true")
        .when(status_kind == "error", "aria-invalid", "true")
        .when(is_disabled, "aria-disabled", "true")
        .flag("readonly", is_disabled)
        .set("autocomplete", "off")
        .set("autocorrect", "off")
        .set("enterkeyhint", "search")
        .set("spellcheck", "false")
        .flag("autofocus", p.has_auto_focus && !is_disabled);
    let ia = (parts.extra_in)(ia).extra(p.input_attrs);
    let input = void("input", ia);
    let start_icon = (!p.start_icon.is_empty()).then(|| {
        el(
            "span",
            attrs().set("class", "astryx-search-start"),
            [icon_or_none(p.start_icon)],
        )
    });
    let group_label = g.map(|_| {
        visually_hidden(
            &VisuallyHiddenProps {
                id: ids.input_label.clone(),
                ..Default::default()
            },
            text(p.label),
        )
    });
    let clear = (parts.clear && !is_disabled).then(|| {
        let mut a = vec![("data-search-clear".to_string(), String::new())];
        if !parts.has_value {
            a.push(("hidden".into(), "hidden".into()));
        }
        button(&ButtonProps {
            variant: "ghost".into(),
            size: "sm".into(),
            label: parts.clear_label.into(),
            icon: "close".into(),
            is_icon_only: true,
            class: "astryx-input-clear-button".into(),
            attrs: a,
            ..Default::default()
        })
    });
    let status_icon = (!status_kind.is_empty() && or(p.status_variant, "attached") != "detached")
        .then(|| {
            icon(&IconProps {
                color: status_kind.into(),
                ..IconProps::sized(status_kind, "md")
            })
        });
    let spin = el(
        "span",
        attrs()
            .set("class", "astryx-search-spinner")
            .flag("hidden", true),
        [Some(spinner(&SpinnerProps::of("sm", "")))],
    );
    let end = el(
        "span",
        attrs().set("class", "astryx-search-end"),
        [Some(spin), clear, status_icon],
    );
    let mut body: Vec<Option<Markup>> = vec![start_icon, group_label];
    let token_labelled_by = join_ids(&[if labelled_by.is_empty() {
        &ids.label
    } else {
        &labelled_by
    }]);
    let mut lead = (parts.lead)(&ids, &token_labelled_by, &described_by);
    if parts.lane_input {
        lead.push(Some(parts.value));
        lead.push(Some(input));
        body.push(Some(el(
            "div",
            attrs().set("class", "astryx-typeahead-lane"),
            lead,
        )));
    } else {
        body.extend(lead);
        body.push(Some(input));
        body.extend(parts.after_in);
        body.push(Some(parts.value));
    }
    let live = visually_hidden(
        &VisuallyHiddenProps {
            id: ids.live.clone(),
            class: "astryx-search-live".into(),
            attrs: vec![
                ("aria-live".into(), "polite".into()),
                ("aria-atomic".into(), "true".into()),
            ],
            ..Default::default()
        },
        html! {},
    );
    let popup = el(
        "div",
        attrs()
            .set("popover", "manual")
            .set("id", &ids.popup)
            .set("class", "astryx-layer astryx-search-popup")
            .set("data-placement", "below")
            .set("data-alignment", "start")
            .when(parts.menu_width.is_empty(), "data-match-width", "")
            .opt("style", &decl("width", parts.menu_width))
            .set("data-size", size),
        [
            Some(typeahead_listbox(
                &ids.listbox,
                p.items,
                p.is_remote.then_some(""),
            )),
            parts.popup_tail,
        ],
    );
    body.extend([Some(end), Some(live), Some(popup)]);
    let num = |v: u32, def: u32| (if v == 0 { def } else { v }).to_string();
    let ca = attrs()
        .set(
            "class",
            &cls(
                &format!("astryx-{} astryx-search-control", parts.kind),
                p.control_class,
            ),
        )
        .set("data-size", size)
        .opt("data-status", status_kind)
        .flag("data-disabled", is_disabled)
        .flag("data-remote", p.is_remote)
        .when(p.is_remote, "data-debounce", &num(p.debounce_ms, 150))
        .set("data-max-items", &num(p.max_menu_items, 10))
        .set("data-min-query", &num(p.min_query_length, 1))
        .flag("data-entries-on-focus", p.has_entries_on_focus)
        .set(
            "data-empty-search-text",
            or(p.empty_search_text, "No results found"),
        );
    let ca = (parts.extra_ctl)(ca).when(g.is_some(), "data-astryx-input-group-item", "");
    // In a group there's no Field: attrs (with the search handler) go on the control.
    let ca = if g.is_some() { ca.extra(p.attrs) } else { ca };
    let control = el("div", ca, body);
    if g.is_some() {
        return control;
    }
    let status = p.status.as_ref().map(|s| FieldStatus {
        message_id: ids.status.clone(),
        ..s.clone()
    });
    field(
        &FieldProps {
            label: p.label.into(),
            is_label_hidden: p.is_label_hidden,
            description: description.into(),
            input_id: id.clone(),
            label_id: ids.label.clone(),
            description_id: desc_id,
            is_optional: p.is_optional,
            is_required: p.is_required,
            is_disabled,
            status,
            status_variant: or(p.status_variant, "attached").into(),
            width: p.width.into(),
            class: p.class.into(),
            attrs: p.attrs.to_vec(),
            ..Default::default()
        },
        control,
    )
}

fn effective_size(size: &str, g: Option<&InputGroupMember>) -> String {
    let s = match g {
        Some(g) if size.is_empty() => g.size.as_str(),
        _ => size,
    };
    or(s, "md").to_string()
}

/// Renders the labelled search field, its picked-item token and the results popover.
pub fn typeahead(p: &TypeaheadProps) -> Markup {
    let g = p.group.as_ref();
    let disabled = p.is_disabled || g.is_some_and(|g| g.is_disabled);
    let mut label = p.value_label.clone();
    if label.is_empty()
        && !p.value.is_empty()
        && let Some(it) = p.items.iter().find(|it| it.id == p.value)
    {
        label = or(&it.label, &it.id).to_string();
    }
    let label = or(&label, &p.value).to_string();
    let has = !p.value.is_empty();
    let size = effective_size(&p.size, g);
    let placeholder = or(&p.placeholder, "Search…");
    let sp = SearchProps {
        id: &p.id,
        label: &p.label,
        items: &p.items,
        is_remote: p.is_remote,
        debounce_ms: p.debounce_ms,
        has_entries_on_focus: p.has_entries_on_focus,
        max_menu_items: p.max_menu_items,
        min_query_length: p.min_query_length,
        empty_search_text: &p.empty_search_text,
        is_label_hidden: p.is_label_hidden,
        description: &p.description,
        is_optional: p.is_optional,
        is_required: p.is_required,
        is_disabled: p.is_disabled,
        // With a value the token shows, so it takes the autofocus.
        has_auto_focus: p.has_auto_focus && !has,
        size: &p.size,
        status: &p.status,
        status_variant: &p.status_variant,
        start_icon: &p.start_icon,
        width: &p.width,
        class: &p.class,
        control_class: &p.control_class,
        attrs: &p.attrs,
        input_attrs: &p.input_attrs,
        group: g,
    };
    let lead = |ids: &SearchIds, labelled_by: &str, described_by: &str| {
        let token_value = el(
            "span",
            attrs()
                .set("id", &format!("{}-token-value", ids.input))
                .flag("hidden", true),
            [Some(text(&label))],
        );
        // The token's button is named by the field label and the value.
        let mut action = vec![(
            "aria-labelledby".to_string(),
            format!("{labelled_by} {}-token-value", ids.input),
        )];
        if !described_by.is_empty() {
            action.push(("aria-describedby".into(), described_by.into()));
        }
        if p.has_auto_focus && has && !disabled {
            action.push(("autofocus".into(), String::new()));
        }
        let t = token(&TokenProps {
            label: label.clone(),
            size: size.clone(),
            is_disabled: disabled,
            action: Some(action),
            ..Default::default()
        });
        vec![
            Some(token_value),
            Some(el(
                "span",
                attrs()
                    .set("class", "astryx-typeahead-token")
                    .flag("hidden", !has),
                [Some(t)],
            )),
        ]
    };
    let value = void(
        "input",
        attrs()
            .set("type", "hidden")
            .set("class", "astryx-typeahead-value")
            .when(!disabled && !p.name.is_empty(), "name", &p.name)
            .when(!disabled && !p.form.is_empty(), "form", &p.form)
            .set("value", &p.value)
            .set("data-label", &label)
            .extra(&p.value_attrs),
    );
    search_shell(
        &sp,
        SearchParts {
            kind: "typeahead",
            has_value: has,
            clear: !p.no_clear,
            clear_label: "Clear selection",
            lane_input: true,
            lead: &lead,
            value,
            extra_ctl: &|a| a.flag("data-has-value", has),
            extra_in: &|a| {
                a.when(has, "tabindex", "-1")
                    .set("data-placeholder-text", placeholder)
                    .when(!has, "placeholder", placeholder)
            },
            describe: Vec::new(),
            after_in: Vec::new(),
            menu_width: "",
            popup_tail: None,
        },
    )
}

/// Renders the labelled search field with its tokens, hidden select and results popover.
pub fn tokenizer(p: &TokenizerProps) -> Markup {
    let g = p.group.as_ref();
    let disabled = p.is_disabled || g.is_some_and(|g| g.is_disabled);
    let size = effective_size(&p.size, g);
    let at_max = p.max_entries > 0 && p.value.len() >= p.max_entries as usize;
    let id = control_id(&p.id, &p.label);
    let limit_id = format!("{id}-limit");
    let token_for = |label: &str| {
        token(&TokenProps {
            label: label.into(),
            size: size.clone(),
            is_disabled: disabled,
            is_removable: true,
            ..Default::default()
        })
    };
    // The select's options: chosen items first (in order, selected), then the other static items.
    let mut opts: Vec<Option<Markup>> = Vec::new();
    let mut tokens: Vec<Option<Markup>> = Vec::new();
    let mut seen: HashSet<&str> = HashSet::new();
    for it in &p.value {
        if !seen.insert(&it.id) {
            continue;
        }
        let l = or(&it.label, &it.id);
        opts.push(Some(el(
            "option",
            attrs().set("value", &it.id).flag("selected", true),
            [Some(text(l))],
        )));
        tokens.push(Some(el(
            "span",
            attrs()
                .set("class", "astryx-tokenizer-token")
                .set("data-id", &it.id),
            [Some(token_for(l))],
        )));
    }
    for it in &p.items {
        if !seen.insert(&it.id) {
            continue;
        }
        opts.push(Some(el(
            "option",
            attrs().set("value", &it.id),
            [Some(text(or(&it.label, &it.id)))],
        )));
    }
    let has = !tokens.is_empty();
    let limit = (p.max_entries > 0).then(|| {
        visually_hidden(
            &VisuallyHiddenProps {
                id: limit_id.clone(),
                class: "astryx-tokenizer-limit".into(),
                ..Default::default()
            },
            text(&format!("Limit of {} reached", p.max_entries)),
        )
    });
    let mut tail = vec![Some(el(
        "template",
        attrs().set("class", "astryx-tokenizer-token-template"),
        [Some(el(
            "span",
            attrs().set("class", "astryx-tokenizer-token"),
            [Some(token_for(""))],
        ))],
    ))];
    if p.has_create {
        tail.push(Some(el(
            "template",
            attrs().set("class", "astryx-tokenizer-create-template"),
            [Some(el(
                "div",
                attrs()
                    .set("role", "option")
                    .set("aria-selected", "false")
                    .set("tabindex", "-1")
                    .set("class", "astryx-typeahead-option")
                    .set("data-create", ""),
                [Some(el(
                    "span",
                    attrs().set("class", "astryx-typeahead-option-content"),
                    [Some(el(
                        "div",
                        attrs().set("class", "astryx-typeahead-item"),
                        [Some(el(
                            "div",
                            attrs().set("class", "astryx-typeahead-item-content"),
                            [Some(el(
                                "span",
                                attrs().set("class", "astryx-typeahead-item-label"),
                                [None],
                            ))],
                        ))],
                    ))],
                ))],
            ))],
        )));
    }
    let placeholder = or(&p.placeholder, "Search…");
    let sp = SearchProps {
        id: &p.id,
        label: &p.label,
        items: &p.items,
        is_remote: p.is_remote,
        debounce_ms: p.debounce_ms,
        has_entries_on_focus: p.has_entries_on_focus,
        max_menu_items: p.max_menu_items,
        min_query_length: p.min_query_length,
        empty_search_text: &p.empty_search_text,
        is_label_hidden: p.is_label_hidden,
        description: &p.description,
        is_optional: p.is_optional,
        is_required: p.is_required,
        is_disabled: p.is_disabled,
        has_auto_focus: p.has_auto_focus,
        size: &p.size,
        status: &p.status,
        status_variant: &p.status_variant,
        start_icon: &p.start_icon,
        width: &p.width,
        class: &p.class,
        control_class: &p.control_class,
        attrs: &p.attrs,
        input_attrs: &p.input_attrs,
        group: g,
    };
    let value = el(
        "select",
        attrs()
            .flag("multiple", true)
            .set("class", "astryx-tokenizer-value")
            .flag("hidden", true)
            .set("tabindex", "-1")
            .set("aria-hidden", "true")
            .when(!disabled && !p.name.is_empty(), "name", &p.name)
            .when(!disabled && !p.form.is_empty(), "form", &p.form)
            .extra(&p.value_attrs),
        opts,
    );
    let max_entries = p.max_entries.to_string();
    search_shell(
        &sp,
        SearchParts {
            kind: "tokenizer",
            has_value: has,
            clear: p.has_clear,
            clear_label: "Clear all",
            lane_input: false,
            lead: &|_, _, _| tokens.clone(),
            value,
            extra_ctl: &|a| {
                a.flag("data-has-tokens", has)
                    .when(p.max_entries > 0, "data-max-entries", &max_entries)
                    .flag("data-at-max", at_max)
                    .flag("data-create", p.has_create)
            },
            extra_in: &|a| {
                a.set("data-placeholder-text", placeholder)
                    .when(!has, "placeholder", placeholder)
                    .flag("data-at-max", at_max)
                    .flag("readonly", at_max && !disabled)
            },
            describe: if at_max {
                vec![limit_id.clone()]
            } else {
                Vec::new()
            },
            after_in: vec![limit],
            menu_width: &p.menu_width,
            popup_tail: Some(group(tail)),
        },
    )
}
