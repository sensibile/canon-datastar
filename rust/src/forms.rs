//! TextArea, CheckboxList, RadioList, NumberInput, Slider, FileInput. Ported from Astryx
//! (packages/core/src/TextArea, CheckboxList, RadioList, NumberInput, Slider, FileInput),
//! MIT, Meta Platforms, Inc. static/canon.js adds the client behaviour.

use crate::Attr;
use crate::avatar::group_thousands;
use crate::button::{ButtonProps, button};
use crate::field::{FieldProps, field, field_status_message};
use crate::html::{attrs, cls, el, group, join_ids, or, raw, text, void};
use crate::icon::{IconProps, icon};
use crate::input_group::InputGroupMember;
use crate::inputs::CHECK_MARK;
use crate::item::{ItemProps, item};
use crate::misc::{SpinnerProps, VisuallyHiddenProps, spinner, visually_hidden};
use crate::types::{FieldStatus, control_id};
use maud::{Markup, html};

fn when_id(cond: bool, id: &str) -> &str {
    if cond { id } else { "" }
}

fn desc_id(id: &str, description: &str) -> String {
    if description.is_empty() {
        String::new()
    } else {
        format!("{id}-desc")
    }
}

fn status_parts(status: &Option<FieldStatus>) -> (&str, &str) {
    status
        .as_ref()
        .map(|s| (s.kind.as_str(), s.message.as_str()))
        .unwrap_or(("", ""))
}

fn with_message_id(status: &Option<FieldStatus>, id: &str) -> Option<FieldStatus> {
    status.as_ref().map(|s| FieldStatus {
        message_id: id.to_string(),
        ..s.clone()
    })
}

fn empty() -> Markup {
    html! {}
}

fn icon_with(name: &str, size: &str, color: &str, class: &str) -> Markup {
    icon(&IconProps {
        color: color.into(),
        class: class.into(),
        ..IconProps::sized(name, size)
    })
}

fn opt_num(f: Option<f64>) -> String {
    f.map(|v| v.to_string()).unwrap_or_default()
}

/// Props for [`text_area`].
#[derive(Clone, Debug, Default)]
pub struct TextAreaProps {
    /// Id of the `<textarea>`; defaults to "astryx-" + a slug of the label.
    pub id: String,
    pub label: String,
    pub value: String,
    pub is_label_hidden: bool,
    pub description: String,
    pub is_optional: bool,
    pub is_required: bool,
    /// Keeps the textarea focusable (readonly + aria-disabled) and drops its name.
    pub is_disabled: bool,
    pub is_read_only: bool,
    /// A built-in glyph name.
    pub start_icon: String,
    pub status: Option<FieldStatus>,
    pub status_variant: String,
    /// "sm", "md" (default), "lg".
    pub size: String,
    pub is_loading: bool,
    pub placeholder: String,
    /// Defaults to 3.
    pub rows: u32,
    pub width: String,
    /// A soft limit: shows a counter and marks the field invalid past it without blocking input.
    /// canon.js keeps the counter current as the user types.
    pub max_length: u32,
    /// The hard native maxlength.
    pub html_max_length: u32,
    pub has_auto_focus: bool,
    pub name: String,
    /// spellcheck="true"/"false" when set.
    pub has_spell_check: Option<bool>,
    /// On the Field container.
    pub class: String,
    /// On the bordered box.
    pub control_class: String,
    /// On the Field container.
    pub attrs: Vec<Attr>,
    /// On the `<textarea>`, e.g. `("data-bind:bio", "")`.
    pub input_attrs: Vec<Attr>,
}

/// Renders a labelled multi-line text field with an optional character counter. The server
/// counts code points; canon.js recounts user-perceived characters on load and input.
/// Not ported: change_action, disabled_message, the "tooltip" status variant.
pub fn text_area(p: &TextAreaProps) -> Markup {
    let id = control_id(&p.id, &p.label);
    let status_id = format!("{id}-status");
    let counter_id = format!("{id}-counter");
    let desc = desc_id(&id, &p.description);
    let (kind, message) = status_parts(&p.status);
    let messageless = kind == "error" && message.is_empty();
    let has_counter = p.max_length > 0;
    let length = if has_counter {
        p.value.chars().count() as u32
    } else {
        0
    };
    let over = has_counter && length > p.max_length;
    let described_by = join_ids(&[
        &desc,
        when_id(!message.is_empty() || messageless, &status_id),
        when_id(has_counter, &counter_id),
    ]);
    let spell = match p.has_spell_check {
        Some(true) => "true",
        Some(false) => "false",
        None => "",
    };
    let rows = if p.rows == 0 { 3 } else { p.rows };
    let busy = p.is_loading && !kind.is_empty();
    let ta = attrs()
        .set("class", "astryx-text-area-control")
        .set("id", &id)
        .when(!p.is_disabled && !p.name.is_empty(), "name", &p.name)
        .opt("placeholder", &p.placeholder)
        .set("rows", &rows.to_string())
        .when(
            p.html_max_length > 0,
            "maxlength",
            &p.html_max_length.to_string(),
        )
        .when(has_counter, "data-max-length", &p.max_length.to_string())
        .opt("spellcheck", spell)
        .flag("readonly", p.is_read_only || p.is_disabled)
        .flag(
            "required",
            p.is_required && !p.is_optional && !p.is_disabled,
        )
        .when(p.is_disabled, "aria-disabled", "true")
        .opt("aria-describedby", &described_by)
        .when(p.is_required && !p.is_optional, "aria-required", "true")
        .when(kind == "error" || over, "aria-invalid", "true")
        .when(p.is_loading, "aria-busy", "true")
        .flag("autofocus", p.has_auto_focus)
        .flag("data-start-icon", !p.start_icon.is_empty())
        .flag("data-with-status", p.is_loading || !kind.is_empty())
        .flag("data-busy-status", busy)
        .flag("data-counter", has_counter)
        .extra(&p.input_attrs);
    // The HTML parser drops one newline right after <textarea>, so keep a leading one.
    let value = if p.value.starts_with('\n') {
        format!("\n{}", p.value)
    } else {
        p.value.clone()
    };
    let start_icon = (!p.start_icon.is_empty()).then(|| {
        el(
            "span",
            attrs().set("class", "astryx-text-area-start-icon"),
            [Some(icon_with(&p.start_icon, "sm", "secondary", ""))],
        )
    });
    let end = (p.is_loading || !kind.is_empty()).then(|| {
        el(
            "span",
            attrs().set("class", "astryx-text-area-end"),
            [
                p.is_loading.then(|| {
                    spinner(&SpinnerProps {
                        attrs: vec![("aria-hidden".into(), "true".into())],
                        ..SpinnerProps::of("sm", "")
                    })
                }),
                (!kind.is_empty()).then(|| icon_with(kind, "md", kind, "")),
            ],
        )
    });
    let (counter, live) = if has_counter {
        (
            Some(el(
                "div",
                attrs()
                    .set("id", &counter_id)
                    .set("class", "astryx-text-area-counter")
                    .flag("data-over", over),
                [
                    Some(icon(&IconProps::sized("warning", "sm"))),
                    Some(el(
                        "span",
                        attrs().set("class", "astryx-text-area-count"),
                        [Some(text(&format!(
                            "{}/{}",
                            group_thousands(length as i64),
                            group_thousands(p.max_length as i64)
                        )))],
                    )),
                ],
            )),
            Some(group([
                Some(live_region("astryx-text-area-polite", "polite")),
                Some(live_region("astryx-text-area-assertive", "assertive")),
            ])),
        )
    } else {
        (None, None)
    };
    let hidden = messageless.then(|| {
        visually_hidden(
            &VisuallyHiddenProps {
                id: status_id.clone(),
                ..Default::default()
            },
            text("Invalid value"),
        )
    });
    let control = el(
        "div",
        attrs()
            .set("class", &cls("astryx-text-area", &p.control_class))
            .set("data-size", or(&p.size, "md"))
            .opt("data-status", kind)
            .flag("data-disabled", p.is_disabled)
            .flag("data-readonly", p.is_read_only),
        [
            start_icon,
            Some(el("textarea", ta, [Some(text(&value))])),
            end,
            counter,
        ],
    );
    field(
        &FieldProps {
            label: p.label.clone(),
            is_label_hidden: p.is_label_hidden,
            description: p.description.clone(),
            input_id: id.clone(),
            description_id: desc,
            is_optional: p.is_optional,
            is_required: p.is_required,
            is_disabled: p.is_disabled,
            status: with_message_id(&p.status, &status_id),
            status_variant: p.status_variant.clone(),
            width: p.width.clone(),
            class: p.class.clone(),
            attrs: p.attrs.clone(),
            ..Default::default()
        },
        group([Some(control), live, hidden]),
    )
}

fn live_region(class: &str, politeness: &str) -> Markup {
    visually_hidden(
        &VisuallyHiddenProps {
            class: class.into(),
            attrs: vec![("aria-live".into(), politeness.into())],
            ..Default::default()
        },
        empty(),
    )
}

/// One row of a [`checkbox_list`].
#[derive(Clone, Debug, Default)]
pub struct CheckboxListItem {
    /// Id of the `<input>`; defaults to the list's id + "-" + its index.
    pub id: String,
    pub label: String,
    /// Replaces `label` with markup; the checkbox is then named by the visible label unless
    /// `aria_label` is set.
    pub label_content: Option<Markup>,
    pub aria_label: String,
    pub value: String,
    pub description: String,
    pub end_content: Option<Markup>,
    pub is_checked: bool,
    pub is_indeterminate: bool,
    pub is_disabled: bool,
    pub is_loading: bool,
    /// "center" (default), "start" or "first-line".
    pub align: String,
    pub class: String,
    /// On the row.
    pub attrs: Vec<Attr>,
    /// On the `<input>`.
    pub input_attrs: Vec<Attr>,
}

/// Props for [`checkbox_list`].
#[derive(Clone, Debug, Default)]
pub struct CheckboxListProps {
    /// Id of the group; defaults to "astryx-" + a slug of the label.
    pub id: String,
    pub label: String,
    pub is_label_hidden: bool,
    pub description: String,
    pub is_optional: bool,
    /// Marks the first enabled box required while none is checked (canon.js keeps it current).
    pub is_required: bool,
    pub status: Option<FieldStatus>,
    /// The form field name every box submits its value under.
    pub name: String,
    /// "balanced" (default), "compact" (small boxes) or "spacious".
    pub density: String,
    pub has_dividers: bool,
    pub is_disabled: bool,
    pub is_read_only: bool,
    pub width: String,
    pub items: Vec<CheckboxListItem>,
    /// On the Field container.
    pub class: String,
    /// On the Field container.
    pub attrs: Vec<Attr>,
    /// On every `<input>`.
    pub input_attrs: Vec<Attr>,
}

/// Renders a labelled group of checkbox rows. Clicking anywhere on a row toggles it.
/// Not ported: change_action, disabled_message.
pub fn checkbox_list(p: &CheckboxListProps) -> Markup {
    let id = control_id(&p.id, &p.label);
    let label_id = format!("{id}-label");
    let desc = desc_id(&id, &p.description);
    let status_id = if p.status.as_ref().is_some_and(|s| !s.message.is_empty()) {
        format!("{id}-status")
    } else {
        String::new()
    };
    let density = or(&p.density, "balanced");
    let size = if density == "compact" { "sm" } else { "md" };
    let required = p.is_required && !p.is_optional;
    let any_checked = p.items.iter().any(|it| it.is_checked);
    let required_idx = if required && !p.is_read_only && !any_checked && !p.is_disabled {
        p.items.iter().position(|it| !it.is_disabled)
    } else {
        None
    };
    let invalid = p.status.as_ref().is_some_and(|s| s.kind == "error");
    let rows = p.items.iter().enumerate().map(|(i, it)| {
        let input_id = if it.id.is_empty() {
            format!("{id}-{i}")
        } else {
            it.id.clone()
        };
        let item_label_id = format!("{input_id}-label");
        let item_desc_id = desc_id(&input_id, &it.description);
        let disabled = p.is_disabled || it.is_disabled;
        let from_visible = it.label_content.is_some() && it.aria_label.is_empty();
        let aria_label = or(&it.aria_label, or(&it.label, "Checkbox"));
        let input = attrs()
            .set("id", &input_id)
            .set("type", "checkbox")
            .when(!disabled && !p.name.is_empty(), "name", &p.name)
            .opt("value", &it.value)
            .flag("checked", it.is_checked)
            .flag("disabled", disabled)
            .flag("required", required_idx == Some(i))
            .when(p.is_read_only, "aria-readonly", "true")
            .when(!from_visible, "aria-label", aria_label)
            .when(from_visible, "aria-labelledby", &item_label_id)
            .opt("aria-describedby", &item_desc_id)
            .when(invalid, "aria-invalid", "true")
            .when(it.is_loading, "aria-busy", "true")
            .set("data-size", size)
            .when(it.is_indeterminate, "data-init", "el.indeterminate = true")
            .when(
                p.is_read_only || it.is_loading,
                "data-on:click",
                "evt.preventDefault()",
            )
            .extra(&p.input_attrs)
            .extra(&it.input_attrs);
        let mark = if it.is_loading {
            spinner(&SpinnerProps::of("sm", "inherit"))
        } else {
            raw(CHECK_MARK)
        };
        let media = el(
            "span",
            attrs().set("class", "astryx-checkbox-control"),
            [
                Some(void("input", input)),
                Some(el(
                    "span",
                    attrs()
                        .set("class", "astryx-checkbox-indicator")
                        .set("aria-hidden", "true")
                        .set("data-size", size),
                    [Some(mark)],
                )),
            ],
        );
        let mut row_attrs = Vec::new();
        if it.is_loading {
            row_attrs.push(("aria-busy".to_string(), "true".to_string()));
        }
        row_attrs.extend(it.attrs.iter().cloned());
        Some(item(&ItemProps {
            as_: "li".into(),
            label: it.label.clone(),
            label_content: it.label_content.clone(),
            description: it.description.clone(),
            start_content: Some(media),
            end_content: it.end_content.clone(),
            align: it.align.clone(),
            density: density.into(),
            is_delegate: !p.is_read_only,
            is_disabled: disabled,
            label_id: if from_visible {
                item_label_id
            } else {
                String::new()
            },
            description_id: item_desc_id,
            class: cls("astryx-checkbox-list-item", &it.class),
            attrs: row_attrs,
            ..Default::default()
        }))
    });
    let group_el = el(
        "div",
        attrs()
            .set("id", &id)
            .set("role", "group")
            .set("class", "astryx-checkbox-list-group")
            .flag("data-required", required)
            .when(p.is_disabled, "aria-disabled", "true")
            .set("aria-labelledby", &label_id)
            .opt("aria-describedby", &join_ids(&[&desc, &status_id])),
        [Some(el(
            "ul",
            attrs()
                .set("class", "astryx-list")
                .set("data-density", density)
                .flag("data-dividers", p.has_dividers),
            rows.collect::<Vec<_>>(),
        ))],
    );
    field(
        &FieldProps {
            label: p.label.clone(),
            is_label_hidden: p.is_label_hidden,
            description: p.description.clone(),
            input_id: id.clone(),
            label_id,
            is_group_label: true,
            description_id: desc,
            is_optional: p.is_optional,
            is_required: p.is_required,
            is_disabled: p.is_disabled,
            status: with_message_id(&p.status, &status_id),
            status_variant: "detached".into(),
            width: p.width.clone(),
            class: cls("astryx-checkbox-list", &p.class),
            attrs: p.attrs.clone(),
            ..Default::default()
        },
        group_el,
    )
}

/// One option of a [`radio_list`].
#[derive(Clone, Debug, Default)]
pub struct RadioListItem {
    /// Id of the `<input>`; defaults to the list's id + "-" + its index.
    pub id: String,
    pub label: String,
    pub label_content: Option<Markup>,
    pub aria_label: String,
    pub value: String,
    pub description: String,
    pub start_content: Option<Markup>,
    pub end_content: Option<Markup>,
    pub is_disabled: bool,
    /// "center" (default), "start" or "first-line".
    pub align: String,
    pub class: String,
    /// On the row.
    pub attrs: Vec<Attr>,
    /// On the `<input>`.
    pub input_attrs: Vec<Attr>,
}

/// Props for [`radio_list`].
#[derive(Clone, Debug, Default)]
pub struct RadioListProps {
    /// Id of the radiogroup; defaults to "astryx-" + a slug of the label.
    pub id: String,
    pub label: String,
    pub is_label_hidden: bool,
    pub description: String,
    /// The checked option's value.
    pub value: String,
    /// "vertical" (default) or "horizontal".
    pub orientation: String,
    pub is_disabled: bool,
    /// Groups the radios; defaults to the group id + "-radio".
    pub name: String,
    pub is_required: bool,
    pub is_optional: bool,
    pub status: Option<FieldStatus>,
    /// "sm" or "md" (default).
    pub size: String,
    pub width: String,
    pub items: Vec<RadioListItem>,
    /// On the radiogroup.
    pub class: String,
    /// On the Field container.
    pub container_class: String,
    /// On the radiogroup.
    pub attrs: Vec<Attr>,
    /// On every `<input>`, e.g. `("data-bind:plan", "")`.
    pub input_attrs: Vec<Attr>,
}

/// Renders a labelled radiogroup. Tab enters on the checked radio and leaves the group in one
/// step (canon.js); arrow keys move between radios natively. Not ported: disabled_message.
pub fn radio_list(p: &RadioListProps) -> Markup {
    let id = control_id(&p.id, &p.label);
    let label_id = format!("{id}-label");
    let desc = desc_id(&id, &p.description);
    let status_id = if p.status.as_ref().is_some_and(|s| !s.message.is_empty()) {
        format!("{id}-status")
    } else {
        String::new()
    };
    let size = or(&p.size, "md");
    let name = if p.name.is_empty() {
        format!("{id}-radio")
    } else {
        p.name.clone()
    };
    let required = p.is_required && !p.is_optional;
    let rows: Vec<_> = p
        .items
        .iter()
        .enumerate()
        .map(|(i, it)| {
            let input_id = if it.id.is_empty() {
                format!("{id}-{i}")
            } else {
                it.id.clone()
            };
            let item_label_id = format!("{input_id}-label");
            let item_desc_id = desc_id(&input_id, &it.description);
            let disabled = p.is_disabled || it.is_disabled;
            let input = attrs()
                .set("id", &input_id)
                .set("type", "radio")
                .set("name", &name)
                .set("value", &it.value)
                .flag("checked", !p.value.is_empty() && p.value == it.value)
                .flag("disabled", disabled)
                .flag("required", required)
                .opt("aria-label", &it.aria_label)
                .when(it.aria_label.is_empty(), "aria-labelledby", &item_label_id)
                .opt("aria-describedby", &item_desc_id)
                .extra(&p.input_attrs)
                .extra(&it.input_attrs);
            let media = group([
                Some(el(
                    "span",
                    attrs().set("class", "astryx-radio-control"),
                    [
                        Some(void("input", input)),
                        Some(el(
                            "span",
                            attrs()
                                .set("class", "astryx-radio-indicator")
                                .set("aria-hidden", "true")
                                .set("data-size", size),
                            [Some(el(
                                "span",
                                attrs().set("class", "astryx-radio-indicator-dot"),
                                [None],
                            ))],
                        )),
                    ],
                )),
                it.start_content.clone(),
            ]);
            let mut row_attrs = vec![("data-size".to_string(), size.to_string())];
            row_attrs.extend(it.attrs.iter().cloned());
            Some(item(&ItemProps {
                label: it.label.clone(),
                label_content: it.label_content.clone(),
                description: it.description.clone(),
                start_content: Some(media),
                end_content: it.end_content.clone(),
                align: it.align.clone(),
                is_delegate: true,
                is_disabled: disabled,
                label_id: item_label_id,
                description_id: item_desc_id,
                class: cls("astryx-radio-list-item", &it.class),
                attrs: row_attrs,
                ..Default::default()
            }))
        })
        .collect();
    let group_el = el(
        "div",
        attrs()
            .set("id", &id)
            .set("role", "radiogroup")
            .set("class", &cls("astryx-radio-list", &p.class))
            .set("data-orientation", or(&p.orientation, "vertical"))
            .set("data-size", size)
            .when(p.is_disabled, "aria-disabled", "true")
            .set("aria-labelledby", &label_id)
            .opt("aria-describedby", &join_ids(&[&desc, &status_id]))
            .when(
                p.status.as_ref().is_some_and(|s| s.kind == "error"),
                "aria-invalid",
                "true",
            )
            .when(required, "aria-required", "true")
            .extra(&p.attrs),
        rows,
    );
    field(
        &FieldProps {
            label: p.label.clone(),
            is_label_hidden: p.is_label_hidden,
            description: p.description.clone(),
            input_id: id.clone(),
            label_id,
            is_group_label: true,
            description_id: desc,
            is_optional: p.is_optional,
            is_required: p.is_required,
            is_disabled: p.is_disabled,
            status: with_message_id(&p.status, &status_id),
            status_variant: "detached".into(),
            width: p.width.clone(),
            class: p.container_class.clone(),
            ..Default::default()
        },
        group_el,
    )
}

/// Props for [`number_input`].
#[derive(Clone, Debug, Default)]
pub struct NumberInputProps {
    /// Id of the visible `<input>`; defaults to "astryx-" + a slug of the label.
    pub id: String,
    pub label: String,
    /// The committed number; `None` is empty.
    pub value: Option<f64>,
    pub is_label_hidden: bool,
    pub description: String,
    pub is_optional: bool,
    pub is_required: bool,
    /// Keeps the field focusable (readonly + aria-disabled) and drops its name.
    pub is_disabled: bool,
    pub is_read_only: bool,
    pub start_icon: String,
    pub label_icon: String,
    pub status: Option<FieldStatus>,
    pub status_variant: String,
    /// "sm", "md" (default), "lg".
    pub size: String,
    pub placeholder: String,
    pub width: String,
    pub has_auto_focus: bool,
    /// The form field name; the committed number is submitted from a hidden input.
    pub name: String,
    pub min: Option<f64>,
    pub max: Option<f64>,
    /// The arrow-key, wheel and stepper increment (default 1), on a grid anchored at `min`.
    pub step: f64,
    /// JSON Intl.NumberFormat options for the unfocused display, e.g.
    /// `{"style":"currency","currency":"USD"}`. canon.js formats it.
    pub format: String,
    /// Stops the wheel from stepping while the field is focused.
    pub is_wheel_disabled: bool,
    pub has_number_steppers: bool,
    pub units: String,
    pub is_integer_only: bool,
    /// Lets emptying the field commit no value, and shows a clear button.
    pub has_clear: bool,
    /// BCP 47 locale for parsing and formatting; the browser's by default.
    pub locale: String,
    /// On the Field container.
    pub class: String,
    /// On the bordered row.
    pub control_class: String,
    /// On the Field container.
    pub attrs: Vec<Attr>,
    /// On the visible `<input>`.
    pub input_attrs: Vec<Attr>,
    /// On the hidden `<input>` holding the committed number: bind it with `("data-bind:qty", "")`.
    pub value_attrs: Vec<Attr>,
    /// Places the field in an InputGroup row.
    pub group: Option<InputGroupMember>,
}

/// Renders a text field with spinbutton semantics. canon.js treats typing as a draft that
/// commits on blur or Enter, reads pasted local formats, clamps to the range, shows
/// "Invalid number" for unreadable text, and steps with the arrow keys, the wheel and the
/// steppers. Each commit updates the hidden value input and fires input and change on it.
/// Not ported: format_value callbacks (use `format`), disabled_message, the "tooltip" status
/// variant, on_entry_error_change.
pub fn number_input(p: &NumberInputProps) -> Markup {
    let g = p.group.as_ref();
    let mut p = p.clone();
    if let Some(g) = g {
        p.is_disabled = p.is_disabled || g.is_disabled;
        p.size = or(&p.size, &g.size).to_string();
        p.description = String::new();
    }
    let p = &p;
    let id = control_id(&p.id, &p.label);
    let status_id = format!("{id}-status");
    let units_id = format!("{id}-units");
    let entry_id = format!("{id}-entry");
    let desc = desc_id(&id, &p.description);
    let (kind, message) = status_parts(&p.status);
    let error_message = kind == "error" && !message.is_empty();
    let (labelled_by, group_desc, message) = match g {
        Some(g) => (
            join_ids(&[&g.label_id, &format!("{id}-label")]),
            g.described_by.clone(),
            "",
        ),
        None => (String::new(), String::new(), message),
    };
    let described_by = join_ids(&[
        &group_desc,
        &desc,
        when_id(!message.is_empty(), &status_id),
        when_id(!p.units.is_empty(), &units_id),
    ]);
    let value = opt_num(p.value);
    let mode = if p.is_integer_only {
        "numeric"
    } else {
        "decimal"
    };
    let input = attrs()
        .set("class", "astryx-number-input-field")
        .set("id", &id)
        .set("type", "text")
        .set("inputmode", mode)
        .set("role", "spinbutton")
        .opt("value", &value)
        .opt("placeholder", &p.placeholder)
        .flag("readonly", p.is_read_only || p.is_disabled)
        .when(p.is_disabled, "aria-disabled", "true")
        .opt("aria-valuemin", &opt_num(p.min))
        .opt("aria-valuemax", &opt_num(p.max))
        .opt("aria-valuenow", &value)
        .opt("aria-labelledby", &labelled_by)
        .opt("aria-describedby", &described_by)
        .when(p.is_required && !p.is_optional, "aria-required", "true")
        .when(kind == "error", "aria-invalid", "true")
        .flag("autofocus", p.has_auto_focus)
        .extra(&p.input_attrs);
    let hidden = attrs()
        .set("type", "hidden")
        .set("class", "astryx-number-input-value")
        .when(!p.is_disabled && !p.name.is_empty(), "name", &p.name)
        .set("value", &value)
        .extra(&p.value_attrs);
    let start_icon =
        (!p.start_icon.is_empty()).then(|| icon_with(&p.start_icon, "sm", "secondary", ""));
    let group_label = g.map(|_| {
        visually_hidden(
            &VisuallyHiddenProps {
                id: format!("{id}-label"),
                ..Default::default()
            },
            text(&p.label),
        )
    });
    let units = (!p.units.is_empty()).then(|| {
        el(
            "span",
            attrs()
                .set("id", &units_id)
                .set("class", "astryx-number-input-units"),
            [Some(text(&p.units))],
        )
    });
    let alert = visually_hidden(
        &VisuallyHiddenProps {
            class: "astryx-number-input-alert".into(),
            attrs: vec![
                ("role".into(), "alert".into()),
                ("aria-live".into(), "assertive".into()),
            ],
            ..Default::default()
        },
        empty(),
    );
    let clear = (p.has_clear && !p.is_disabled && !p.is_read_only).then(|| {
        let mut a = vec![("data-number-input-clear".to_string(), String::new())];
        if p.value.is_none() {
            a.push(("hidden".into(), "hidden".into()));
        }
        button(&ButtonProps {
            variant: "ghost".into(),
            size: "sm".into(),
            label: format!("Clear {}", p.label),
            icon: "close".into(),
            is_icon_only: true,
            class: "astryx-input-clear-button".into(),
            attrs: a,
            ..Default::default()
        })
    });
    let status_icon =
        (!kind.is_empty()).then(|| icon_with(kind, "md", kind, "astryx-number-input-status-icon"));
    let entry_icon = (!error_message)
        .then(|| icon_with("error", "md", "error", "astryx-number-input-entry-icon"));
    let steppers = p.has_number_steppers.then(|| {
        let inactive = p.is_disabled || p.is_read_only;
        let btn = |dir: &str, name: &str| {
            Some(el(
                "button",
                attrs()
                    .set("type", "button")
                    .set("tabindex", "-1")
                    .set("class", "astryx-number-input-stepper")
                    .set("data-direction", dir)
                    .when(inactive, "aria-disabled", "true")
                    .set("aria-label", &format!("{name} {}", p.label)),
                [Some(icon(&IconProps::sized(
                    "numberInput:stepperDown",
                    "xsm",
                )))],
            ))
        };
        el(
            "div",
            attrs().set("class", "astryx-number-input-steppers"),
            [btn("increment", "Increment"), btn("decrement", "Decrement")],
        )
    });
    let step = if p.step > 0.0 {
        p.step.to_string()
    } else {
        String::new()
    };
    let control = el(
        "div",
        attrs()
            .set("class", &cls("astryx-number-input", &p.control_class))
            .set("data-size", or(&p.size, "md"))
            .opt("data-status", kind)
            .flag("data-disabled", p.is_disabled)
            .flag("data-readonly", p.is_read_only)
            .flag("data-steppers", p.has_number_steppers)
            .flag("data-error-message", error_message)
            .when(g.is_some(), "data-astryx-input-group-item", "")
            .opt("data-min", &opt_num(p.min))
            .opt("data-max", &opt_num(p.max))
            .opt("data-step", &step)
            .flag("data-integer", p.is_integer_only)
            .flag("data-clear", p.has_clear)
            .flag("data-no-wheel", p.is_wheel_disabled)
            .opt("data-locale", &p.locale)
            .opt("data-format", &p.format),
        [
            start_icon,
            group_label,
            Some(void("input", input)),
            Some(void("input", hidden)),
            units,
            Some(alert),
            clear,
            status_icon,
            entry_icon,
            steppers,
        ],
    );
    if g.is_some() {
        return control;
    }
    // "Invalid number" under the field while a rejected draft is on screen (canon.js shows it).
    let entry = (!error_message).then(|| {
        field_status_message(
            &FieldStatus {
                kind: "error".into(),
                message: "Invalid number".into(),
                ..Default::default()
            },
            &p.status_variant,
            &entry_id,
            "astryx-number-input-entry-status",
            true,
        )
    });
    field(
        &FieldProps {
            label: p.label.clone(),
            is_label_hidden: p.is_label_hidden,
            description: p.description.clone(),
            input_id: id.clone(),
            description_id: desc,
            is_optional: p.is_optional,
            is_required: p.is_required,
            is_disabled: p.is_disabled,
            label_icon: p.label_icon.clone(),
            status: with_message_id(&p.status, &status_id),
            status_variant: p.status_variant.clone(),
            width: p.width.clone(),
            class: p.class.clone(),
            attrs: p.attrs.clone(),
            ..Default::default()
        },
        group([Some(control), entry]),
    )
}

/// A tick on a [`slider`]'s track, optionally labelled.
#[derive(Clone, Debug, Default)]
pub struct SliderMark {
    pub value: f64,
    pub label: String,
}

/// Props for [`slider`].
#[derive(Clone, Debug, Default)]
pub struct SliderProps {
    /// Id of the thumb (single value); defaults to "astryx-" + a slug of the label.
    pub id: String,
    pub label: String,
    /// One value, or two for a range.
    pub values: Vec<f64>,
    pub is_label_hidden: bool,
    pub description: String,
    pub is_disabled: bool,
    pub is_optional: bool,
    pub is_required: bool,
    pub status: Option<FieldStatus>,
    pub width: String,
    /// Defaults to 0.
    pub min: Option<f64>,
    /// Defaults to 100.
    pub max: Option<f64>,
    /// Defaults to 1.
    pub step: f64,
    /// "horizontal" (default) or "vertical".
    pub orientation: String,
    /// JSON Intl.NumberFormat options for the shown value (canon.js formats it).
    pub format: String,
    pub locale: String,
    /// "tooltip" (default), "text" or "none".
    pub value_display: String,
    pub marks: Vec<SliderMark>,
    /// The form field name each value is submitted under.
    pub name: String,
    /// The id of a form outside this one that the values belong to.
    pub form: String,
    /// A range's smallest gap, in steps.
    pub min_steps_between_thumbs: u32,
    /// On the track container.
    pub class: String,
    /// On the Field container.
    pub container_class: String,
    /// On the thumb (single) or the group (range).
    pub attrs: Vec<Attr>,
    /// On the hidden input holding the first value, e.g. `("data-bind:volume", "")`.
    pub value_attrs: Vec<Attr>,
    /// On the hidden input holding the second value.
    pub end_value_attrs: Vec<Attr>,
}

const SLIDER_THUMB: f64 = 20.0;

fn round3(f: f64) -> f64 {
    (f * 1000.0).round() / 1000.0
}

/// Renders a single-value or range slider. canon.js drags the thumbs and jumps them to clicks
/// on the track; arrow keys step, Page keys step by ten, Home and End jump to the bounds. It
/// fires input on the hidden value inputs while moving and change when a move ends.
/// Not ported: format_value callbacks (use `format`), disabled_message, the touch controller.
pub fn slider(p: &SliderProps) -> Markup {
    let id = control_id(&p.id, &p.label);
    let label_id = format!("{id}-label");
    let required_id = format!("{id}-required");
    let desc = desc_id(&id, &p.description);
    let status_id = if p.status.as_ref().is_some_and(|s| !s.message.is_empty()) {
        format!("{id}-status")
    } else {
        String::new()
    };
    let lo = p.min.unwrap_or(0.0);
    let hi = p.max.unwrap_or(100.0);
    let step = if p.step == 0.0 { 1.0 } else { p.step };
    let orientation = or(&p.orientation, "horizontal");
    let horizontal = orientation == "horizontal";
    let display = or(&p.value_display, "tooltip");
    let clamp_v = |v: f64| v.max(lo).min(hi);
    let is_range = p.values.len() >= 2;
    let mut values = vec![p.values.first().map(|v| clamp_v(*v)).unwrap_or(lo)];
    if is_range {
        values.push(clamp_v(p.values[1]));
    }
    let percent = |v: f64| {
        if hi == lo {
            0.0
        } else {
            (v - lo) / (hi - lo) * 100.0
        }
    };
    let inset = |pc: f64| {
        let px = SLIDER_THUMB / 2.0 - pc / 100.0 * SLIDER_THUMB;
        let sign = if px < 0.0 { "-" } else { "+" };
        format!("calc({}% {sign} {}px)", round3(pc), round3(px).abs())
    };
    let conveys_required = p.is_required && !p.is_optional;
    let described_by = join_ids(&[&desc, &status_id, when_id(conveys_required, &required_id)]);
    let invalid = p.status.as_ref().is_some_and(|s| s.kind == "error");
    let gap = p.min_steps_between_thumbs as f64 * step;

    let hidden_inputs: Vec<_> = values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let extra = if i == 1 {
                &p.end_value_attrs
            } else {
                &p.value_attrs
            };
            Some(void(
                "input",
                attrs()
                    .set("type", "hidden")
                    .set("class", "astryx-slider-value")
                    .when(!p.is_disabled && !p.name.is_empty(), "name", &p.name)
                    .opt("form", &p.form)
                    .set("value", &v.to_string())
                    .extra(extra),
            ))
        })
        .collect();
    let fill_start = if is_range {
        percent(values[0]) / 100.0
    } else {
        0.0
    };
    let fill_end = percent(values[values.len() - 1]) / 100.0;
    let mut travel = vec![
        Some(el(
            "div",
            attrs()
                .set("class", "astryx-slider-track")
                .set("aria-hidden", "true"),
            [None],
        )),
        Some(el(
            "div",
            attrs()
                .set("class", "astryx-slider-fill-clip")
                .set("aria-hidden", "true"),
            [Some(el(
                "div",
                attrs()
                    .set("class", "astryx-slider-fill-mover")
                    .flag("data-range", is_range)
                    .set("style", &format!("--_slider-fill-start: {fill_start}")),
                [Some(el(
                    "div",
                    attrs()
                        .set("class", "astryx-slider-fill")
                        .flag("data-range", is_range)
                        .set(
                            "style",
                            &format!(
                                "--_slider-fill-start: {fill_start}; --_slider-fill-end: {fill_end}"
                            ),
                        ),
                    [None],
                ))],
            ))],
        )),
    ];
    let mut has_mark_labels = false;
    if !p.marks.is_empty() {
        let mut marks = Vec::new();
        for m in &p.marks {
            let pos = inset(percent(m.value));
            let style = if horizontal {
                format!("inset-inline-start: {pos}")
            } else {
                format!("bottom: {pos}")
            };
            let filled = if is_range {
                m.value >= values[0] && m.value <= values[1]
            } else {
                m.value <= values[0]
            };
            let mv = m.value.to_string();
            marks.push(Some(el(
                "div",
                attrs()
                    .set("class", "astryx-slider-mark")
                    .flag("data-filled", filled)
                    .set("data-mark-value", &mv)
                    .set("style", &style),
                [None],
            )));
            if !m.label.is_empty() {
                has_mark_labels = has_mark_labels || horizontal;
                marks.push(Some(el(
                    "span",
                    attrs()
                        .set("class", "astryx-slider-mark-label")
                        .set("data-mark-value", &mv)
                        .set("style", &style),
                    [Some(text(&m.label))],
                )));
            }
        }
        travel.push(Some(el(
            "div",
            attrs()
                .set("class", "astryx-slider-marks")
                .set("aria-hidden", "true"),
            marks,
        )));
    }
    for (i, v) in values.iter().enumerate() {
        let progress = (percent(*v) / 100.0).to_string();
        let vmin = if is_range && i == 1 {
            clamp_v(values[0] + gap)
        } else {
            lo
        };
        let vmax = if is_range && i == 0 {
            clamp_v(values[1] - gap)
        } else {
            hi
        };
        let mut thumb = attrs();
        if !is_range {
            thumb = thumb.extra(&p.attrs).set("id", &id);
        }
        thumb = thumb
            .set("role", "slider")
            .set("tabindex", "0")
            .set("class", "astryx-slider-thumb")
            .set("data-orientation", orientation)
            .set("style", &format!("--_slider-progress: {progress}"))
            .set("aria-valuemin", &vmin.to_string())
            .set("aria-valuemax", &vmax.to_string())
            .set("aria-valuenow", &v.to_string())
            .set("aria-orientation", orientation)
            .when(p.is_disabled, "aria-disabled", "true")
            .when(invalid, "aria-invalid", "true");
        thumb = if is_range {
            thumb.set(
                "aria-label",
                if i == 0 {
                    "Minimum value"
                } else {
                    "Maximum value"
                },
            )
        } else {
            thumb.set("aria-labelledby", &label_id)
        };
        thumb = thumb.opt("aria-describedby", &described_by);
        if display == "tooltip" {
            travel.push(Some(el(
                "span",
                attrs()
                    .set("class", "astryx-slider-anchor")
                    .set("aria-hidden", "true")
                    .set("style", &format!("--_slider-progress: {progress}")),
                [Some(el(
                    "span",
                    attrs().set("class", "astryx-slider-tip"),
                    [Some(text(&v.to_string()))],
                ))],
            )));
        }
        travel.push(Some(el("div", thumb, [None])));
    }
    let mut control = attrs();
    if is_range {
        control = control.extra(&p.attrs);
    }
    control = control
        .set("class", &cls("astryx-slider-control", &p.class))
        .set("data-orientation", orientation)
        .flag("data-disabled", p.is_disabled)
        .set("data-min", &lo.to_string())
        .set("data-max", &hi.to_string())
        .set("data-step", &step.to_string())
        .when(
            p.min_steps_between_thumbs > 0,
            "data-min-gap",
            &p.min_steps_between_thumbs.to_string(),
        )
        .opt("data-format", &p.format)
        .opt("data-locale", &p.locale);
    if is_range {
        control = control
            .set("role", "group")
            .set("aria-labelledby", &label_id);
    }
    let text_value = (display == "text").then(|| {
        let mut s = values[0].to_string();
        if is_range {
            s = format!("{s} – {}", values[1]);
        }
        el(
            "span",
            attrs().set("class", "astryx-slider-text"),
            [Some(text(&s))],
        )
    });
    let row = el(
        "div",
        attrs()
            .set("class", "astryx-slider")
            .set("data-orientation", orientation)
            .flag("data-disabled", p.is_disabled)
            .flag("data-mark-labels", has_mark_labels),
        [
            Some(group(hidden_inputs)),
            Some(el(
                "div",
                control,
                [Some(el(
                    "div",
                    attrs().set("class", "astryx-slider-travel"),
                    travel,
                ))],
            )),
            text_value,
        ],
    );
    let required = conveys_required.then(|| {
        visually_hidden(
            &VisuallyHiddenProps {
                id: required_id.clone(),
                ..Default::default()
            },
            text("Required"),
        )
    });
    field(
        &FieldProps {
            label: p.label.clone(),
            is_label_hidden: p.is_label_hidden,
            description: p.description.clone(),
            input_id: id.clone(),
            label_id,
            is_group_label: true,
            description_id: desc,
            is_optional: p.is_optional,
            is_required: p.is_required,
            is_disabled: p.is_disabled,
            status: with_message_id(&p.status, &status_id),
            status_variant: "detached".into(),
            width: p.width.clone(),
            class: p.container_class.clone(),
            ..Default::default()
        },
        group([Some(row), required]),
    )
}

/// Props for [`file_input`].
#[derive(Clone, Debug, Default)]
pub struct FileInputProps {
    /// Id of the native file `<input>`; defaults to "astryx-" + a slug of the label.
    pub id: String,
    pub label: String,
    pub is_label_hidden: bool,
    /// Extensions (".pdf") and MIME types ("image/*"), comma-separated.
    pub accept: String,
    pub is_multiple: bool,
    /// The largest allowed file in bytes (0 = no limit).
    pub max_size: u64,
    /// Caps a multiple selection (0 = no limit).
    pub max_files: u32,
    pub is_disabled: bool,
    pub is_required: bool,
    pub is_optional: bool,
    pub is_loading: bool,
    pub status: Option<FieldStatus>,
    pub status_variant: String,
    pub description: String,
    pub placeholder: String,
    /// "input" (default, a compact row) or "dropzone".
    pub mode: String,
    pub width: String,
    /// The form field name; the native input submits the accepted files.
    pub name: String,
    pub form: String,
    /// On the Field container.
    pub class: String,
    /// On the box.
    pub control_class: String,
    /// On the Field container.
    pub attrs: Vec<Attr>,
    /// On the native file `<input>` (data-bind works there: canon.js validates first).
    pub input_attrs: Vec<Attr>,
}

/// Renders a compact file row or a drag-and-drop zone. canon.js checks files against
/// `accept`, `max_size` and `max_files`, shows rejections as an error status, keeps the native
/// input's files equal to the accepted ones, and announces the selection.
/// Not ported: change_action, disabled_message, the "tooltip" status variant.
pub fn file_input(p: &FileInputProps) -> Markup {
    let id = control_id(&p.id, &p.label);
    let status_id = format!("{id}-status");
    let required_id = format!("{id}-required");
    let error_id = format!("{id}-error");
    let desc = desc_id(&id, &p.description);
    let mode = or(&p.mode, "input");
    let dropzone = mode == "dropzone";
    let (kind, message) = status_parts(&p.status);
    let conveys_required = p.is_required && !p.is_optional;
    let described_by = join_ids(&[
        &desc,
        when_id(!message.is_empty(), &status_id),
        when_id(conveys_required, &required_id),
    ]);
    let placeholder = if !p.placeholder.is_empty() {
        p.placeholder.as_str()
    } else if p.is_multiple {
        "Choose files"
    } else {
        "Choose file"
    };
    let busy = p.is_disabled || p.is_loading;
    let trigger = visually_hidden(
        &VisuallyHiddenProps::default(),
        el(
            "button",
            attrs()
                .set("type", "button")
                .set("class", "astryx-file-input-trigger")
                .when(busy, "aria-disabled", "true")
                .set("aria-label", &p.label)
                .when(p.is_loading, "aria-busy", "true")
                .opt("aria-describedby", &described_by),
            [None],
        ),
    );
    let native = void(
        "input",
        attrs()
            .set("id", &id)
            .set("type", "file")
            .set("class", "astryx-file-input-native")
            .when(!p.is_disabled && !p.name.is_empty(), "name", &p.name)
            .opt("accept", &p.accept)
            .flag("multiple", p.is_multiple)
            .set("aria-hidden", "true")
            .set("tabindex", "-1")
            .when(p.is_disabled, "form", "")
            .when(!p.is_disabled && !p.form.is_empty(), "form", &p.form)
            .extra(&p.input_attrs),
    );
    let size = if dropzone { "md" } else { "sm" };
    let spin = spinner(&SpinnerProps {
        class: "astryx-file-input-spinner".into(),
        attrs: vec![("aria-hidden".into(), "true".into())],
        ..SpinnerProps::of(size, "")
    });
    let name_el = el(
        if dropzone { "div" } else { "span" },
        attrs().set("class", "astryx-file-input-name"),
        [None],
    );
    let placeholder_el = el(
        "span",
        attrs().set("class", "astryx-file-input-placeholder"),
        [Some(text(placeholder))],
    );
    let clear = (!p.is_disabled).then(|| {
        button(&ButtonProps {
            variant: "ghost".into(),
            size: "sm".into(),
            label: format!("Clear {}", p.label),
            icon: "close".into(),
            is_icon_only: true,
            class: "astryx-input-clear-button".into(),
            attrs: vec![("data-file-input-clear".into(), String::new())],
            ..Default::default()
        })
    });
    let mut parts = vec![
        Some(trigger),
        Some(native),
        Some(icon_with(
            "upload",
            size,
            "secondary",
            "astryx-file-input-upload",
        )),
    ];
    if dropzone {
        parts.extend([Some(spin), Some(name_el), Some(placeholder_el)]);
    } else {
        parts.extend([
            Some(name_el),
            Some(placeholder_el),
            Some(spin),
            (!kind.is_empty())
                .then(|| icon_with(kind, "md", kind, "astryx-file-input-status-icon")),
            kind.is_empty()
                .then(|| icon_with("error", "md", "error", "astryx-file-input-error-icon")),
        ]);
    }
    parts.push(clear);
    let max_size = p.max_size.to_string();
    let max_files = p.max_files.to_string();
    let bx = el(
        "div",
        attrs()
            .set("class", &cls("astryx-file-input", &p.control_class))
            .set("data-mode", mode)
            .opt("data-status", kind)
            .flag("data-disabled", p.is_disabled)
            .flag("data-busy", p.is_loading)
            .when(p.max_size > 0, "data-max-size", &max_size)
            .when(
                p.is_multiple && p.max_files > 0,
                "data-max-files",
                &max_files,
            )
            .set("data-label", &p.label),
        parts,
    );
    let variant = if p.status_variant.is_empty() && dropzone {
        "detached"
    } else {
        p.status_variant.as_str()
    };
    // Validation errors appear here (canon.js fills and shows it) unless the caller set a status.
    let error_status = p.status.is_none().then(|| {
        field_status_message(
            &FieldStatus {
                kind: "error".into(),
                ..Default::default()
            },
            variant,
            &error_id,
            "astryx-file-input-error",
            true,
        )
    });
    let live = live_region("astryx-file-input-live", "polite");
    let required = conveys_required.then(|| {
        visually_hidden(
            &VisuallyHiddenProps {
                id: required_id.clone(),
                ..Default::default()
            },
            text("Required"),
        )
    });
    field(
        &FieldProps {
            label: p.label.clone(),
            is_label_hidden: p.is_label_hidden,
            description: p.description.clone(),
            input_id: id.clone(),
            description_id: desc,
            is_optional: p.is_optional,
            is_required: p.is_required,
            is_disabled: p.is_disabled,
            status: with_message_id(&p.status, &status_id),
            status_variant: variant.into(),
            width: p.width.clone(),
            class: p.class.clone(),
            attrs: p.attrs.clone(),
            ..Default::default()
        },
        group([Some(bx), error_status, Some(live), required]),
    )
}
