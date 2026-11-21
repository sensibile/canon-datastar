//! Port of Astryx `Field`, `FieldLabel`, `FieldStatus` (packages/core/src/Field, FieldStatus),
//! MIT, Meta Platforms, Inc. Not ported: labelTooltip (needs InfoTip), FormLayout horizontal
//! labels, clicking the description to focus the control.

use crate::Attr;
use crate::html::{attrs, cls, decl, el, or, text};
use crate::icon::{IconProps, icon};
use crate::types::FieldStatus;
use maud::Markup;

/// Props for [`field_label`].
#[derive(Clone, Debug, Default)]
pub struct FieldLabelProps {
    pub label: String,
    /// The id of the control the label points at.
    pub input_id: String,
    pub label_id: String,
    /// Render a `<span>` instead of `<label for>` (for groups of controls).
    pub is_group_label: bool,
    pub is_label_hidden: bool,
    pub is_disabled: bool,
    pub is_optional: bool,
    pub is_required: bool,
    /// A built-in glyph name shown before the text.
    pub label_icon: String,
    pub description: String,
    pub description_id: String,
    pub class: String,
}

/// Renders a control's label and optional description.
pub fn field_label(p: &FieldLabelProps) -> Markup {
    let tag = if p.is_group_label { "span" } else { "label" };
    let a = attrs()
        .set("class", &cls("astryx-field-label", &p.class))
        .opt("id", &p.label_id)
        .when(!p.is_group_label, "for", &p.input_id)
        .flag("data-disabled", p.is_disabled)
        .flag("data-label-hidden", p.is_label_hidden);
    let icon_m = (!p.label_icon.is_empty()).then(|| icon(&IconProps::sized(&p.label_icon, "sm")));
    let status_text = if p.is_optional {
        "Optional"
    } else if p.is_required {
        "Required"
    } else {
        ""
    };
    let status = (!status_text.is_empty()).then(|| {
        el(
            "span",
            attrs().set("class", "astryx-field-optionality"),
            [
                Some(el(
                    "span",
                    attrs().set("aria-hidden", "true"),
                    [Some(text(" ∙ "))],
                )),
                Some(text(status_text)),
            ],
        )
    });
    let label = el(tag, a, [icon_m, Some(text(&p.label)), status]);
    if p.description.is_empty() {
        return label;
    }
    el(
        "div",
        attrs()
            .set("class", "astryx-field-label-group")
            .flag("data-label-hidden", p.is_label_hidden),
        [
            Some(label),
            Some(el(
                "span",
                attrs()
                    .set("class", "astryx-field-description")
                    .opt("id", &p.description_id)
                    .flag("data-label-hidden", p.is_label_hidden),
                [Some(text(&p.description))],
            )),
        ],
    )
}

/// Renders a status message. `variant`: "attached" (default) or "detached".
pub fn field_status(status: &FieldStatus, variant: &str, id: &str) -> Markup {
    let variant = or(variant, "attached");
    let a = attrs()
        .set("class", "astryx-field-status")
        .opt("id", id)
        .set("data-type", &status.kind)
        .set("data-variant", variant);
    if variant != "detached" {
        return el("div", a, [Some(text(&status.message))]);
    }
    el(
        "div",
        a,
        [Some(el(
            "span",
            attrs().set("class", "astryx-field-status-content"),
            [
                Some(el(
                    "span",
                    attrs().set("class", "astryx-field-status-icon"),
                    [Some(icon(&IconProps::sized(&status.kind, "sm")))],
                )),
                Some(el("span", attrs(), [Some(text(&status.message))])),
            ],
        ))],
    )
}

/// Props for [`field`].
#[derive(Clone, Debug, Default)]
pub struct FieldProps {
    pub label: String,
    pub is_label_hidden: bool,
    pub description: String,
    /// The id of the wrapped control.
    pub input_id: String,
    pub label_id: String,
    pub is_group_label: bool,
    pub description_id: String,
    pub is_optional: bool,
    pub is_required: bool,
    pub is_disabled: bool,
    pub label_icon: String,
    pub status: Option<FieldStatus>,
    /// "attached" (default, joined under the control) or "detached".
    pub status_variant: String,
    pub width: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Wraps a control (`children`) with label, description and status message.
pub fn field(p: &FieldProps, children: Markup) -> Markup {
    let desc_id = if p.description_id.is_empty() && !p.description.is_empty() {
        format!("{}-desc", p.input_id)
    } else {
        p.description_id.clone()
    };
    let status = p
        .status
        .as_ref()
        .filter(|s| !s.message.is_empty())
        .map(|s| {
            let id = if s.message_id.is_empty() {
                format!("{}-status", p.input_id)
            } else {
                s.message_id.clone()
            };
            field_status(s, &p.status_variant, &id)
        });
    let a = attrs()
        .set("class", &cls("astryx-field", &p.class))
        .flag("data-label-hidden", p.is_label_hidden)
        .opt("style", &decl("width", &p.width))
        .extra(&p.attrs);
    let label = field_label(&FieldLabelProps {
        label: p.label.clone(),
        input_id: p.input_id.clone(),
        label_id: p.label_id.clone(),
        is_group_label: p.is_group_label,
        is_label_hidden: p.is_label_hidden,
        is_disabled: p.is_disabled,
        is_optional: p.is_optional,
        is_required: p.is_required,
        label_icon: p.label_icon.clone(),
        description: p.description.clone(),
        description_id: desc_id,
        class: String::new(),
    });
    if or(&p.status_variant, "attached") == "attached" {
        let control = el(
            "div",
            attrs().set("class", "astryx-field-control"),
            [Some(children), status],
        );
        return el("div", a, [Some(label), Some(control)]);
    }
    el("div", a, [Some(label), Some(children), status])
}
