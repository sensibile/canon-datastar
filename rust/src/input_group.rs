//! InputGroup and InputGroupText. Ported from Astryx (packages/core/src/InputGroup), MIT,
//! Meta Platforms, Inc.

use crate::Attr;
use crate::field::{FieldProps, field};
use crate::html::{attrs, cls, el, join_ids, or, text};
use crate::types::{FieldStatus, control_id};
use maud::Markup;

/// Marks an element (a button, say) as a joined member of an InputGroup row:
/// `ButtonProps { attrs: vec![input_group_item()], .. }`.
pub fn input_group_item() -> Attr {
    ("data-astryx-input-group-item".into(), String::new())
}

/// Tells a control it sits inside an InputGroup: it then renders only its control, named by
/// the group label plus its own. Get one from [`InputGroupProps::member`].
#[derive(Clone, Debug, Default)]
pub struct InputGroupMember {
    pub label_id: String,
    pub described_by: String,
    pub is_disabled: bool,
    pub size: String,
}

/// Props for [`input_group`].
#[derive(Clone, Debug, Default)]
pub struct InputGroupProps {
    /// Id of the group; defaults to "astryx-" + a slug of the label.
    pub id: String,
    pub label: String,
    pub is_label_hidden: bool,
    pub description: String,
    pub is_disabled: bool,
    pub is_optional: bool,
    pub is_required: bool,
    /// "sm", "md" (default), "lg"; members default to it.
    pub size: String,
    pub status: Option<FieldStatus>,
    pub class: String,
    pub attrs: Vec<Attr>,
}

impl InputGroupProps {
    fn ids(&self) -> (String, String, String, String) {
        let id = control_id(&self.id, &self.label);
        let label_id = format!("{id}-label");
        let desc_id = if self.description.is_empty() {
            String::new()
        } else {
            format!("{id}-desc")
        };
        let status_id = if self.status.as_ref().is_some_and(|s| !s.message.is_empty()) {
            format!("{id}-status")
        } else {
            String::new()
        };
        (id, label_id, desc_id, status_id)
    }

    /// What to pass as a member control's `group` prop.
    pub fn member(&self) -> Option<InputGroupMember> {
        let (_, label_id, desc_id, status_id) = self.ids();
        Some(InputGroupMember {
            label_id,
            described_by: join_ids(&[&desc_id, &status_id]),
            is_disabled: self.is_disabled,
            size: or(&self.size, "md").into(),
        })
    }
}

/// Joins inputs, buttons and [`input_group_text`] add-ons (`children`) into one row under one
/// label. Give member controls `group: p.member()`.
pub fn input_group(p: &InputGroupProps, children: Markup) -> Markup {
    let (id, label_id, desc_id, status_id) = p.ids();
    let status_kind = p.status.as_ref().map(|s| s.kind.as_str()).unwrap_or("");
    let row = el(
        "div",
        attrs()
            .set("id", &id)
            .set("role", "group")
            .set("class", &cls("astryx-input-group", &p.class))
            .set("data-size", or(&p.size, "md"))
            .opt("data-status", status_kind)
            .flag("data-disabled", p.is_disabled)
            .when(p.is_disabled, "aria-disabled", "true")
            .set("aria-labelledby", &label_id)
            .opt("aria-describedby", &join_ids(&[&desc_id, &status_id]))
            .extra(&p.attrs),
        [Some(children)],
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
            input_id: id,
            label_id,
            is_group_label: true,
            description_id: desc_id,
            is_optional: p.is_optional,
            is_required: p.is_required,
            is_disabled: p.is_disabled,
            status,
            status_variant: "detached".into(),
            ..Default::default()
        },
        row,
    )
}

/// A static add-on inside an InputGroup, such as a unit or a URL prefix.
pub fn input_group_text(label: &str) -> Markup {
    el(
        "div",
        attrs()
            .set("data-astryx-input-group-item", "")
            .set("class", "astryx-input-group-text"),
        [Some(el(
            "span",
            attrs().set("dir", "auto"),
            [Some(text(label))],
        ))],
    )
}
