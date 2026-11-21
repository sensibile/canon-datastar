//! TextInput, CheckboxInput, Switch. Ported from Astryx (packages/core/src/TextInput,
//! CheckboxInput, Switch), MIT, Meta Platforms, Inc.

use crate::Attr;
use crate::field::{FieldLabelProps, FieldProps, field, field_label, field_status};
use crate::html::{Attrs, attrs, cls, decl, el, group, join_ids, or, raw, text, void};
use crate::icon::{IconProps, icon};
use crate::misc::{SpinnerProps, VisuallyHiddenProps, spinner, visually_hidden};
use crate::types::{FieldStatus, control_id};
use maud::Markup;

/// Props for [`text_input`].
#[derive(Clone, Debug, Default)]
pub struct TextInputProps {
    /// Id of the `<input>`; defaults to "astryx-" + a slug of the label. Set it when labels repeat.
    pub id: String,
    pub label: String,
    pub value: String,
    /// "text" (default), "password", "email", "search", "tel", "url".
    pub kind: String,
    pub is_label_hidden: bool,
    pub description: String,
    pub is_optional: bool,
    pub is_required: bool,
    /// Keeps the input focusable (readonly + aria-disabled) and drops its name.
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
    pub width: String,
    pub has_auto_focus: bool,
    /// The input's form field name.
    pub name: String,
    /// spellcheck="true"/"false" when set.
    pub has_spell_check: Option<bool>,
    /// On the Field container.
    pub class: String,
    /// On the bordered row.
    pub control_class: String,
    /// On the Field container.
    pub attrs: Vec<Attr>,
    /// On the `<input>`, e.g. `("data-bind:email", "")`.
    pub input_attrs: Vec<Attr>,
}

/// Renders a labelled single-line text field.
/// Not ported: has_clear, disabled_message (needs Tooltip), InputGroup.
pub fn text_input(p: &TextInputProps) -> Markup {
    let id = control_id(&p.id, &p.label);
    let status_id = format!("{id}-status");
    let desc_id = if p.description.is_empty() {
        String::new()
    } else {
        format!("{id}-desc")
    };
    let status_kind = p.status.as_ref().map(|s| s.kind.as_str()).unwrap_or("");
    let has_message = p.status.as_ref().is_some_and(|s| !s.message.is_empty());
    let messageless = status_kind == "error" && !has_message;
    let described_by = join_ids(&[
        &desc_id,
        if has_message || messageless {
            &status_id
        } else {
            ""
        },
    ]);
    let spell = match p.has_spell_check {
        Some(true) => "true",
        Some(false) => "false",
        None => "",
    };
    let input = attrs()
        .set("id", &id)
        .when(!p.is_disabled, "name", &p.name)
        .set("type", or(&p.kind, "text"))
        .opt("value", &p.value)
        .opt("placeholder", &p.placeholder)
        .opt("spellcheck", spell)
        .flag("readonly", p.is_read_only || p.is_disabled)
        .flag(
            "required",
            p.is_required && !p.is_optional && !p.is_disabled,
        )
        .when(p.is_disabled, "aria-disabled", "true")
        .opt("aria-describedby", &described_by)
        .when(p.is_required && !p.is_optional, "aria-required", "true")
        .when(status_kind == "error", "aria-invalid", "true")
        .when(p.is_loading, "aria-busy", "true")
        .flag("autofocus", p.has_auto_focus)
        .extra(&p.input_attrs);
    let start_icon = (!p.start_icon.is_empty()).then(|| {
        icon(&IconProps {
            color: "secondary".into(),
            ..IconProps::sized(&p.start_icon, "sm")
        })
    });
    let spin = p.is_loading.then(|| {
        spinner(&SpinnerProps {
            attrs: vec![("aria-hidden".into(), "true".into())],
            ..SpinnerProps::of("sm", "")
        })
    });
    let status_icon = (!status_kind.is_empty()).then(|| {
        icon(&IconProps {
            color: status_kind.into(),
            ..IconProps::sized(status_kind, "md")
        })
    });
    let hidden = messageless.then(|| {
        visually_hidden(
            &VisuallyHiddenProps {
                id: status_id.clone(),
                ..Default::default()
            },
            text("This field has an error"),
        )
    });
    let control = el(
        "div",
        attrs()
            .set("class", &cls("astryx-text-input", &p.control_class))
            .set("data-size", or(&p.size, "md"))
            .opt("data-status", status_kind)
            .flag("data-disabled", p.is_disabled)
            .flag("data-readonly", p.is_read_only),
        [start_icon, Some(void("input", input)), spin, status_icon],
    );
    let status = p.status.as_ref().map(|s| FieldStatus {
        message_id: status_id.clone(),
        ..s.clone()
    });
    field(
        &FieldProps {
            label: p.label.clone(),
            is_label_hidden: p.is_label_hidden,
            description: p.description.clone(),
            input_id: id.clone(),
            description_id: desc_id,
            is_optional: p.is_optional,
            is_required: p.is_required,
            is_disabled: p.is_disabled,
            status,
            status_variant: p.status_variant.clone(),
            width: p.width.clone(),
            class: p.class.clone(),
            attrs: p.attrs.clone(),
            ..Default::default()
        },
        group([Some(control), hidden]),
    )
}

/// Props for [`checkbox_input`].
#[derive(Clone, Debug, Default)]
pub struct CheckboxInputProps {
    /// Id of the `<input>`; defaults to "astryx-" + a slug of the label.
    pub id: String,
    pub label: String,
    pub checked: bool,
    pub is_indeterminate: bool,
    pub is_label_hidden: bool,
    pub description: String,
    pub is_loading: bool,
    pub is_disabled: bool,
    pub is_read_only: bool,
    pub is_optional: bool,
    pub is_required: bool,
    pub name: String,
    /// The submitted form value (browser default "on").
    pub value: String,
    /// "sm" or "md" (default).
    pub size: String,
    pub width: String,
    pub label_icon: String,
    pub status: Option<FieldStatus>,
    pub class: String,
    /// On the root.
    pub attrs: Vec<Attr>,
    /// On the `<input>`, e.g. `("data-bind:agree", "")`.
    pub input_attrs: Vec<Attr>,
}

/// The attributes checkbox and switch inputs share, in the shared order.
#[allow(clippy::too_many_arguments)]
fn check_input(
    id: &str,
    role: &str,
    name: &str,
    value: &str,
    checked: bool,
    required: bool,
    disabled: bool,
    desc_id: &str,
    status_id: &str,
    status: Option<&FieldStatus>,
    loading: bool,
) -> Attrs {
    let invalid = if status.is_some_and(|s| s.kind == "error") {
        "true"
    } else if required {
        "false"
    } else {
        ""
    };
    attrs()
        .set("id", id)
        .set("type", "checkbox")
        .opt("role", role)
        .when(!disabled, "name", name)
        .opt("value", value)
        .flag("checked", checked)
        .flag("required", required)
        .flag("disabled", disabled)
        .when(required, "aria-required", "true")
        .opt("aria-describedby", &join_ids(&[desc_id, status_id]))
        .opt("aria-invalid", invalid)
        .when(loading, "aria-busy", "true")
}

fn ids(id: &str, description: &str, status: Option<&FieldStatus>) -> (String, String) {
    let desc = if description.is_empty() {
        String::new()
    } else {
        format!("{id}-desc")
    };
    let status = if status.is_some_and(|s| !s.message.is_empty()) {
        format!("{id}-status")
    } else {
        String::new()
    };
    (desc, status)
}

const CHECK_MARK: &str = r#"<svg viewBox="0 0 10 10" class="astryx-checkbox-indicator-check"><path d="M8.5 2.5L4 7.5L1.5 5" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round" stroke-linejoin="round"></path></svg><span class="astryx-checkbox-indicator-dash"></span>"#;

/// Renders a labelled checkbox. The indicator follows the native input (:checked,
/// :indeterminate), so data-bind on the input drives it. `is_indeterminate` and the
/// read-only/loading click blocking use Datastar (data-init / data-on:click).
pub fn checkbox_input(p: &CheckboxInputProps) -> Markup {
    let id = control_id(&p.id, &p.label);
    let size = or(&p.size, "md");
    let (desc_id, status_id) = ids(&id, &p.description, p.status.as_ref());
    let input = check_input(
        &id,
        "",
        &p.name,
        &p.value,
        p.checked,
        p.is_required,
        p.is_disabled,
        &desc_id,
        &status_id,
        p.status.as_ref(),
        p.is_loading,
    )
    .when(p.is_read_only, "aria-readonly", "true")
    .set("data-size", size)
    .when(p.is_indeterminate, "data-init", "el.indeterminate = true")
    .when(
        p.is_read_only || p.is_loading,
        "data-on:click",
        "evt.preventDefault()",
    )
    .extra(&p.input_attrs);
    let mark = if p.is_loading {
        spinner(&SpinnerProps::of("sm", "inherit"))
    } else {
        raw(CHECK_MARK)
    };
    let status = (!status_id.is_empty())
        .then(|| field_status(p.status.as_ref().unwrap(), "detached", &status_id));
    let label = field_label(&FieldLabelProps {
        class: "astryx-checkbox-label".into(),
        label: p.label.clone(),
        input_id: id.clone(),
        is_label_hidden: p.is_label_hidden,
        is_disabled: p.is_disabled,
        is_optional: p.is_optional,
        is_required: p.is_required,
        label_icon: p.label_icon.clone(),
        description: p.description.clone(),
        description_id: desc_id.clone(),
        ..Default::default()
    });
    el(
        "div",
        attrs()
            .set("class", &cls("astryx-checkbox-input", &p.class))
            .set("data-checkbox-size", size)
            .opt("style", &decl("width", &p.width))
            .extra(&p.attrs),
        [
            Some(el(
                "div",
                attrs()
                    .set("class", "astryx-checkbox-row")
                    .flag("data-label-hidden", p.is_label_hidden)
                    .flag("data-disabled", p.is_disabled),
                [
                    Some(el(
                        "div",
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
                    )),
                    Some(el(
                        "div",
                        attrs().set("class", "astryx-checkbox-label-wrapper"),
                        [Some(label)],
                    )),
                ],
            )),
            status,
        ],
    )
}

/// Props for [`switch`].
#[derive(Clone, Debug, Default)]
pub struct SwitchProps {
    /// Id of the `<input>`; defaults to "astryx-" + a slug of the label.
    pub id: String,
    pub label: String,
    pub checked: bool,
    pub is_label_hidden: bool,
    pub description: String,
    pub is_loading: bool,
    pub is_disabled: bool,
    pub name: String,
    /// The submitted form value (browser default "on").
    pub value: String,
    pub is_optional: bool,
    pub is_required: bool,
    pub label_icon: String,
    pub width: String,
    /// "end" (default) or "start".
    pub label_position: String,
    /// "hug" (default) or "spread" (label and switch at opposite ends).
    pub label_spacing: String,
    pub status: Option<FieldStatus>,
    /// "sm" or "md" (default).
    pub size: String,
    pub class: String,
    /// On the root.
    pub attrs: Vec<Attr>,
    /// On the `<input>`, e.g. `("data-bind:notify", "")`.
    pub input_attrs: Vec<Attr>,
}

/// Renders a labelled on/off switch (a checkbox with role="switch"). The track follows the
/// native input's :checked state. Not ported: drag-to-toggle, disabled_message, label_tooltip.
pub fn switch(p: &SwitchProps) -> Markup {
    let id = control_id(&p.id, &p.label);
    let size = or(&p.size, "md");
    let (desc_id, status_id) = ids(&id, &p.description, p.status.as_ref());
    let input = check_input(
        &id,
        "switch",
        &p.name,
        &p.value,
        p.checked,
        p.is_required,
        p.is_disabled,
        &desc_id,
        &status_id,
        p.status.as_ref(),
        p.is_loading,
    )
    .when(p.is_loading, "data-on:click", "evt.preventDefault()")
    .extra(&p.input_attrs);
    let spin = p.is_loading.then(|| spinner(&SpinnerProps::of("sm", "")));
    let control = el(
        "div",
        attrs().set("class", "astryx-switch-control"),
        [
            Some(void("input", input)),
            Some(el(
                "div",
                attrs()
                    .set("class", "astryx-switch")
                    .set("aria-hidden", "true")
                    .set("data-size", size),
                [Some(el(
                    "div",
                    attrs()
                        .set("class", "astryx-switch-thumb")
                        .set("data-size", size),
                    [spin],
                ))],
            )),
        ],
    );
    let label = el(
        "div",
        attrs()
            .set("class", "astryx-switch-label-wrapper")
            .set("data-size", size),
        [Some(field_label(&FieldLabelProps {
            class: "astryx-switch-label".into(),
            label: p.label.clone(),
            input_id: id.clone(),
            is_label_hidden: p.is_label_hidden,
            is_disabled: p.is_disabled,
            is_optional: p.is_optional,
            is_required: p.is_required,
            label_icon: p.label_icon.clone(),
            description: p.description.clone(),
            description_id: desc_id.clone(),
            ..Default::default()
        }))],
    );
    let start = p.label_position == "start";
    let (first, second) = if start {
        (label, control)
    } else {
        (control, label)
    };
    let status = (!status_id.is_empty())
        .then(|| field_status(p.status.as_ref().unwrap(), "detached", &status_id));
    el(
        "div",
        attrs()
            .set("class", &cls("astryx-switch-field", &p.class))
            .when(start, "data-label-position", "start")
            .when(p.label_spacing == "spread", "data-label-spacing", "spread")
            .opt("style", &decl("width", &p.width))
            .extra(&p.attrs),
        [
            Some(el(
                "div",
                attrs()
                    .set("class", "astryx-switch-container")
                    .flag("data-label-hidden", p.is_label_hidden)
                    .flag("data-disabled", p.is_disabled),
                [Some(first), Some(second)],
            )),
            status,
        ],
    )
}
