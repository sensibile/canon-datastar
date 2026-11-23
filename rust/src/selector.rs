//! Selector. Ported from Astryx `Selector` (packages/core/src/Selector), MIT, Meta Platforms, Inc.
//! A select-one control: a combobox button that opens a listbox of options in a native popover.
//! The value lives in a hidden input (bind it with `value_attrs`, e.g. `("data-bind:plan", "")`);
//! static/canon.js keeps the trigger text and the selected row in step with it.
//! Not ported: the bottom-sheet presentation on touch, positioning the list over the selected
//! option, changeAction, renderOption/renderValue, disabledMessage and labelTooltip.

use crate::Attr;
use crate::button::{ButtonProps, button};
use crate::field::{FieldProps, field};
use crate::html::{attrs, cls, el, join_ids, or, raw, text, void};
use crate::icon::{IconProps, icon};
use crate::input_group::InputGroupMember;
use crate::inputs::CHECK_MARK;
use crate::misc::{
    DividerProps, SpinnerProps, VisuallyHiddenProps, divider, spinner, visually_hidden,
};
use crate::types::{FieldStatus, control_id};
use maud::{Markup, html};

/// One entry of a selector's options: an option, a divider ([`selector_divider`]) or a titled
/// section of options ([`selector_section`]).
#[derive(Clone, Debug, Default)]
pub struct SelectorOption {
    pub value: String,
    /// The shown text; defaults to `value`. For a section it is the title.
    pub label: String,
    pub description: String,
    /// A built-in glyph name shown before the label (and in the trigger when picked).
    pub icon: String,
    pub is_disabled: bool,
    pub is_divider: bool,
    pub is_section: bool,
    /// A section's options.
    pub options: Vec<SelectorOption>,
}

impl SelectorOption {
    /// An option whose label is its value.
    pub fn new(value: &str, label: &str) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            ..Default::default()
        }
    }
}

/// A line between options; it is hidden while searching.
pub fn selector_divider() -> SelectorOption {
    SelectorOption {
        is_divider: true,
        ..Default::default()
    }
}

/// Groups options under an optional title.
pub fn selector_section(title: &str, options: Vec<SelectorOption>) -> SelectorOption {
    SelectorOption {
        is_section: true,
        label: title.into(),
        options,
        ..Default::default()
    }
}

/// Props for [`selector`].
#[derive(Clone, Debug, Default)]
pub struct SelectorProps {
    /// Id of the trigger button; defaults to "astryx-" + a slug of the label. Derived ids:
    /// id+"-listbox", id+"-popup", id+"-listbox-option-N".
    pub id: String,
    pub label: String,
    pub options: Vec<SelectorOption>,
    /// The selected option's value; "" selects nothing.
    pub value: String,
    /// A clear (×) button while there's a value.
    pub has_clear: bool,
    pub is_label_hidden: bool,
    pub description: String,
    pub is_optional: bool,
    pub is_required: bool,
    /// Keeps the trigger focusable (aria-disabled) and drops the hidden input's name.
    pub is_disabled: bool,
    pub is_read_only: bool,
    pub is_loading: bool,
    /// Shown while nothing is selected. Default "Select…".
    pub placeholder: String,
    /// "sm", "md" (default), "lg".
    pub size: String,
    /// "input" (default), "secondary", "ghost".
    pub variant: String,
    pub status: Option<FieldStatus>,
    /// "attached" (default) or "detached"; non-input variants are always detached.
    pub status_variant: String,
    pub width: String,
    /// A built-in glyph name.
    pub start_icon: String,
    /// "end" (default) or "start" for the check mark on the selected row.
    pub indicator_position: String,
    pub has_search: bool,
    /// Default "Search…".
    pub search_placeholder: String,
    /// Default "No options".
    pub empty_text: String,
    /// Default "No results found".
    pub empty_search_text: String,
    /// "below" (default) or "above".
    pub placement: String,
    pub is_default_open: bool,
    /// On the hidden value input.
    pub name: String,
    pub form: String,
    /// On the Field container.
    pub class: String,
    /// On the bordered row.
    pub control_class: String,
    /// On the Field container.
    pub attrs: Vec<Attr>,
    /// On the trigger button.
    pub trigger_attrs: Vec<Attr>,
    /// On the hidden value input, e.g. `("data-bind:plan", "")`.
    pub value_attrs: Vec<Attr>,
    /// Places the selector in an InputGroup row.
    pub group: Option<InputGroupMember>,
}

pub(crate) fn option_label(o: &SelectorOption) -> &str {
    or(&o.label, &o.value)
}

/// Every option (sections flattened) in display order.
pub(crate) fn selectable_options(options: &[SelectorOption]) -> Vec<&SelectorOption> {
    let mut out = Vec::new();
    for o in options {
        if o.is_divider {
        } else if o.is_section {
            out.extend(o.options.iter());
        } else {
            out.push(o);
        }
    }
    out
}

pub(crate) struct SelectorIds {
    pub listbox: String,
    pub popup: String,
    pub label: String,
    pub input_label: String,
    pub value: String,
    pub desc: String,
    pub status: String,
    pub read_only: String,
    pub search: String,
}

pub(crate) fn selector_ids(id: &str) -> SelectorIds {
    SelectorIds {
        listbox: format!("{id}-listbox"),
        popup: format!("{id}-popup"),
        label: format!("{id}-label"),
        input_label: format!("{id}-input-label"),
        value: format!("{id}-value"),
        desc: format!("{id}-desc"),
        status: format!("{id}-status"),
        read_only: format!("{id}-readonly"),
        search: format!("{id}-search"),
    }
}

pub(crate) fn icon_or_none(name: &str) -> Option<Markup> {
    (!name.is_empty()).then(|| {
        icon(&IconProps {
            color: "secondary".into(),
            ..IconProps::sized(name, "sm")
        })
    })
}

/// Renders the labelled control, its hidden value input and the popover listbox.
pub fn selector(p: &SelectorProps) -> Markup {
    let all = selectable_options(&p.options);
    let selected = all
        .iter()
        .find(|o| !p.value.is_empty() && o.value == p.value)
        .copied();
    let placeholder = or(&p.placeholder, "Select…");
    selector_shell(
        p,
        ShellParts {
            class: "astryx-selector astryx-selector-control",
            multiple: false,
            has_value: selected.is_some(),
            clear: format!("Clear {}", p.label),
            trigger_body: &|ids: &SelectorIds| {
                let value_text = selected.map(option_label).unwrap_or(placeholder);
                let value_icon = p.start_icon.is_empty().then(|| {
                    let name = selected.map(|o| o.icon.as_str()).unwrap_or("");
                    // Kept (hidden while empty) so canon.js can show a picked option's icon.
                    el(
                        "span",
                        attrs()
                            .set("class", "astryx-selector-value-icon")
                            .flag("hidden", name.is_empty()),
                        [icon_or_none(name)],
                    )
                });
                html! {
                    @if let Some(i) = value_icon { (i) }
                    (el(
                        "span",
                        attrs()
                            .set("id", &ids.value)
                            .set("class", "astryx-selector-value-text")
                            .set("data-placeholder-text", placeholder),
                        [Some(text(value_text))],
                    ))
                }
            },
            value: void(
                "input",
                attrs()
                    .set("type", "hidden")
                    .set("class", "astryx-selector-value")
                    .when(!p.is_disabled && !p.name.is_empty(), "name", &p.name)
                    .when(!p.is_disabled && !p.form.is_empty(), "form", &p.form)
                    .set("value", &p.value)
                    .extra(&p.value_attrs),
            ),
            listbox: &|ids: &SelectorIds, size: &str| {
                selector_listbox(
                    ids,
                    &p.label,
                    Vec::new(),
                    &p.options,
                    size,
                    &p.indicator_position,
                    &|o: &SelectorOption| !p.value.is_empty() && o.value == p.value,
                    or(&p.empty_text, "No options"),
                    or(&p.empty_search_text, "No results found"),
                    "check",
                )
            },
        },
    )
}

/// What Selector and MultiSelector put into the shared control.
pub(crate) struct ShellParts<'a> {
    pub class: &'a str,
    pub multiple: bool,
    pub has_value: bool,
    pub clear: String,
    pub trigger_body: &'a dyn Fn(&SelectorIds) -> Markup,
    pub value: Markup,
    pub listbox: &'a dyn Fn(&SelectorIds, &str) -> Markup,
}

/// Renders the Field, the bordered control with its trigger and the popup.
pub(crate) fn selector_shell(p: &SelectorProps, parts: ShellParts) -> Markup {
    let g = p.group.as_ref();
    let mut p = p.clone();
    if let Some(g) = g {
        p.is_disabled = p.is_disabled || g.is_disabled;
        p.size = or(&p.size, &g.size).to_string();
        p.description = String::new();
    }
    let p = &p;
    let variant = or(&p.variant, "input");
    let size = or(&p.size, "md");
    let status_variant = if variant != "input" {
        "detached"
    } else {
        or(&p.status_variant, "attached")
    };
    let read_only = p.is_read_only && !p.is_disabled;
    let required = p.is_required && !p.is_optional;
    let id = control_id(&p.id, &p.label);
    let ids = selector_ids(&id);
    let status_kind = p.status.as_ref().map(|s| s.kind.as_str()).unwrap_or("");
    let mut has_message = p.status.as_ref().is_some_and(|s| !s.message.is_empty());
    let desc_id = if !p.description.is_empty() && g.is_none() {
        ids.desc.clone()
    } else {
        String::new()
    };
    let (mut labelled_by, group_desc) = match g {
        Some(g) => {
            has_message = false;
            (
                join_ids(&[&g.label_id, &ids.input_label]),
                g.described_by.clone(),
            )
        }
        None => (ids.label.clone(), String::new()),
    };
    let combobox = read_only || !p.has_search;
    if p.has_search && !read_only {
        labelled_by = join_ids(&[&labelled_by, &ids.value]);
    }
    let trigger = attrs()
        .set("id", &id)
        .set("type", "button")
        .set("class", "astryx-selector-trigger")
        .when(combobox, "role", "combobox")
        .when(!read_only, "aria-haspopup", "listbox")
        .set("aria-expanded", "false")
        .when(!read_only, "aria-controls", &ids.listbox)
        .when(read_only, "aria-readonly", "true")
        .set("aria-labelledby", &labelled_by)
        .opt(
            "aria-describedby",
            &join_ids(&[
                &group_desc,
                &desc_id,
                if has_message { &ids.status } else { "" },
                if read_only { &ids.read_only } else { "" },
            ]),
        )
        .when(required && combobox, "aria-required", "true")
        .when(status_kind == "error", "aria-invalid", "true")
        .when(p.is_loading, "aria-busy", "true")
        .when(p.is_disabled, "aria-disabled", "true")
        .extra(&p.trigger_attrs);
    let trigger_btn = el("button", trigger, [Some((parts.trigger_body)(&ids))]);
    let start_icon = icon_or_none(&p.start_icon);
    let group_label = g.map(|_| {
        visually_hidden(
            &VisuallyHiddenProps {
                id: ids.input_label.clone(),
                ..Default::default()
            },
            text(&p.label),
        )
    });
    let read_only_text = read_only.then(|| {
        visually_hidden(
            &VisuallyHiddenProps {
                id: ids.read_only.clone(),
                ..Default::default()
            },
            text("Read only"),
        )
    });
    let spin = p.is_loading.then(|| {
        spinner(&SpinnerProps {
            attrs: vec![("aria-hidden".into(), "true".into())],
            ..SpinnerProps::of("sm", "")
        })
    });
    let clear = (p.has_clear && !p.is_disabled && !read_only).then(|| {
        let mut a = vec![
            ("data-selector-clear".to_string(), String::new()),
            ("tabindex".into(), "-1".into()),
        ];
        if !parts.has_value {
            a.push(("hidden".into(), "hidden".into()));
        }
        button(&ButtonProps {
            variant: "ghost".into(),
            size: "sm".into(),
            label: parts.clear.clone(),
            icon: "close".into(),
            is_icon_only: true,
            class: "astryx-input-clear-button".into(),
            attrs: a,
            ..Default::default()
        })
    });
    let end = if !status_kind.is_empty() && status_variant != "detached" {
        Some(icon(&IconProps {
            color: status_kind.into(),
            ..IconProps::sized(status_kind, "sm")
        }))
    } else if !read_only {
        Some(el(
            "span",
            attrs()
                .set("class", "astryx-selector-indicator-icon")
                .set("data-state", "collapsed"),
            [Some(icon(&IconProps {
                color: "secondary".into(),
                ..IconProps::sized("chevronDown", "sm")
            }))],
        ))
    } else {
        None
    };
    let live = visually_hidden(
        &VisuallyHiddenProps {
            id: format!("{id}-live"),
            class: "astryx-selector-live".into(),
            attrs: vec![
                ("aria-live".into(), "polite".into()),
                ("aria-atomic".into(), "true".into()),
            ],
            ..Default::default()
        },
        html! {},
    );
    // Rendered even when read-only: canon.js reads option labels from it for bound values.
    let popup = selector_popup(
        &ids,
        variant,
        size,
        or(&p.placement, "below"),
        p.has_search,
        required,
        or(&p.search_placeholder, "Search…"),
        (parts.listbox)(&ids, size),
    );
    let control = el(
        "div",
        attrs()
            .set("class", &cls(parts.class, &p.control_class))
            .set("data-variant", variant)
            .set("data-size", size)
            .opt("data-status", status_kind)
            .flag("data-disabled", p.is_disabled)
            .flag("data-readonly", read_only)
            .flag("data-placeholder", !parts.has_value)
            .flag("data-search", p.has_search)
            .flag("data-multiple", parts.multiple)
            .flag(
                "data-default-open",
                p.is_default_open && !p.is_disabled && !read_only,
            )
            .when(g.is_some(), "data-astryx-input-group-item", ""),
        [
            start_icon,
            group_label,
            read_only_text,
            Some(trigger_btn),
            Some(parts.value),
            spin,
            clear,
            end,
            Some(live),
            Some(popup),
        ],
    );
    if g.is_some() {
        return control;
    }
    let status = p.status.as_ref().map(|s| FieldStatus {
        message_id: ids.status.clone(),
        ..s.clone()
    });
    field(
        &FieldProps {
            label: p.label.clone(),
            is_label_hidden: p.is_label_hidden,
            description: p.description.clone(),
            input_id: id.clone(),
            label_id: ids.label.clone(),
            description_id: desc_id,
            is_optional: p.is_optional,
            is_required: p.is_required,
            is_disabled: p.is_disabled,
            status,
            status_variant: status_variant.into(),
            width: p.width.clone(),
            class: p.class.clone(),
            attrs: p.attrs.clone(),
            ..Default::default()
        },
        control,
    )
}

/// The popover holding the optional search row and the listbox.
#[allow(clippy::too_many_arguments)]
pub(crate) fn selector_popup(
    ids: &SelectorIds,
    variant: &str,
    size: &str,
    placement: &str,
    has_search: bool,
    required: bool,
    search_placeholder: &str,
    listbox: Markup,
) -> Markup {
    let wrap = el(
        "div",
        attrs()
            .set("class", "astryx-selector-listbox-wrap")
            .flag("data-input", variant == "input"),
        [Some(listbox)],
    );
    let body = if has_search {
        let search = void(
            "input",
            attrs()
                .set("id", &ids.search)
                .set("class", "astryx-selector-search")
                .set("type", "text")
                .set("role", "combobox")
                .set("aria-label", "Search options")
                .when(required, "aria-required", "true")
                .set("aria-expanded", "true")
                .set("aria-controls", &ids.listbox)
                .set("aria-autocomplete", "list")
                .set("autocomplete", "off")
                .set("spellcheck", "false")
                .set("placeholder", search_placeholder),
        );
        let clear = button(&ButtonProps {
            variant: "ghost".into(),
            size: "sm".into(),
            label: "Clear search".into(),
            icon: "close".into(),
            is_icon_only: true,
            class: "astryx-selector-search-clear".into(),
            attrs: vec![("hidden".into(), "hidden".into())],
            ..Default::default()
        });
        el(
            "div",
            attrs().set("class", "astryx-selector-search-panel"),
            [
                Some(el(
                    "div",
                    attrs().set("class", "astryx-selector-search-row"),
                    [icon_or_none("search"), Some(search), Some(clear)],
                )),
                Some(divider(&DividerProps::default())),
                Some(wrap),
            ],
        )
    } else {
        wrap
    };
    el(
        "div",
        attrs()
            .set("popover", "auto")
            .set("id", &ids.popup)
            .set("class", "astryx-layer astryx-selector-popup")
            .set("data-placement", placement)
            .set("data-alignment", "start")
            .set("data-match-width", "")
            .set("data-size", size),
        [Some(body)],
    )
}

/// The listbox rows; `mark` is "check" (Selector) or "checkbox" (MultiSelector).
#[allow(clippy::too_many_arguments)]
pub(crate) fn selector_listbox(
    ids: &SelectorIds,
    label: &str,
    lead: Vec<Markup>,
    options: &[SelectorOption],
    size: &str,
    indicator: &str,
    is_selected: &dyn Fn(&SelectorOption) -> bool,
    empty_text: &str,
    empty_search_text: &str,
    mark: &str,
) -> Markup {
    let mut index = 0usize;
    let mut row = |o: &SelectorOption| {
        let r = selector_row(
            &format!("{}-option-{index}", ids.listbox),
            o,
            size,
            indicator,
            is_selected(o),
            mark,
            &[],
        );
        index += 1;
        r
    };
    let mut rows: Vec<Option<Markup>> = Vec::new();
    for o in options {
        if o.is_divider {
            rows.push(Some(divider(&DividerProps {
                class: "astryx-selector-divider".into(),
                attrs: vec![("aria-hidden".into(), "true".into())],
                ..Default::default()
            })));
        } else if o.is_section {
            let heading = (!o.label.is_empty()).then(|| {
                el(
                    "div",
                    attrs()
                        .set("class", "astryx-selector-section-heading")
                        .set("aria-hidden", "true"),
                    [Some(text(&o.label))],
                )
            });
            let mut inner = vec![heading];
            for so in &o.options {
                inner.push(Some(row(so)));
            }
            rows.push(Some(el(
                "div",
                attrs()
                    .set("role", "group")
                    .opt("aria-label", &o.label)
                    .set("class", "astryx-selector-section"),
                inner,
            )));
        } else {
            rows.push(Some(row(o)));
        }
    }
    let empty = el(
        "div",
        attrs()
            .set("class", "astryx-selector-empty-state")
            .set("role", "presentation")
            .set("aria-live", "polite")
            .flag("hidden", index > 0),
        [Some(text(empty_text))],
    );
    let mut children: Vec<Option<Markup>> = lead.into_iter().map(Some).collect();
    children.push(Some(empty));
    children.extend(rows);
    el(
        "div",
        attrs()
            .set("id", &ids.listbox)
            .set("role", "listbox")
            .set("aria-label", label)
            .when(mark == "checkbox", "aria-multiselectable", "true")
            .set("tabindex", "-1")
            .set("class", "astryx-selector-listbox")
            .set("data-empty-text", empty_text)
            .set("data-empty-search-text", empty_search_text),
        children,
    )
}

pub(crate) fn selector_row(
    id: &str,
    o: &SelectorOption,
    size: &str,
    indicator: &str,
    selected: bool,
    mark: &str,
    extra: &[Attr],
) -> Markup {
    let mark_el = if mark == "none" {
        None
    } else if mark == "checkbox" {
        Some(el(
            "span",
            attrs().set("class", "astryx-selector-option-mark"),
            [Some(el(
                "span",
                attrs()
                    .set("class", "astryx-checkbox-indicator")
                    .set("data-size", "sm")
                    .set("aria-hidden", "true"),
                [Some(raw(CHECK_MARK))],
            ))],
        ))
    } else {
        Some(el(
            "span",
            attrs().set("class", "astryx-selector-option-mark"),
            [Some(el(
                "span",
                attrs()
                    .set("class", "astryx-check-indicator")
                    .set("aria-hidden", "true"),
                [Some(icon(&IconProps::sized("check", "sm")))],
            ))],
        ))
    };
    let desc = (!o.description.is_empty()).then(|| {
        el(
            "span",
            attrs().set("class", "astryx-selector-option-description"),
            [Some(text(&o.description))],
        )
    });
    let content = el(
        "span",
        attrs().set("class", "astryx-selector-option-content"),
        [
            icon_or_none(&o.icon),
            Some(el(
                "span",
                attrs().set("class", "astryx-selector-option"),
                [
                    Some(el(
                        "span",
                        attrs().set("class", "astryx-selector-option-label"),
                        [Some(text(option_label(o)))],
                    )),
                    desc,
                ],
            )),
        ],
    );
    let (start, end) = if indicator == "start" {
        (mark_el, None)
    } else {
        (None, mark_el)
    };
    el(
        "div",
        attrs()
            .set("id", id)
            .set("role", "option")
            .set("aria-selected", if selected { "true" } else { "false" })
            .when(o.is_disabled, "aria-disabled", "true")
            .set("class", "astryx-selector-option-row")
            .set("data-value", &o.value)
            .opt("data-icon", &o.icon)
            .set("data-mark", mark)
            .set("data-size", size)
            .extra(extra),
        [start, Some(content), end],
    )
}
