//! Shared prop vocabulary, mirroring Astryx (packages/core/src/utils/types.ts).
//! Enum-like props are strings; an empty string means "use the default".

/// A validation message for a form control.
#[derive(Clone, Debug, Default)]
pub struct FieldStatus {
    /// "warning", "error" or "success".
    pub kind: String,
    pub message: String,
    /// Overrides the message element's id.
    pub message_id: String,
}

impl FieldStatus {
    pub fn new(kind: &str, message: &str) -> Self {
        FieldStatus {
            kind: kind.into(),
            message: message.into(),
            message_id: String::new(),
        }
    }
}

/// A spacing step ("0", "0.5", "1", "1.5", "2", "3", "4", "5", "6", "8", "10") as
/// var(--spacing-N); "" stays "".
pub(crate) fn space_css(step: &str) -> String {
    if step.is_empty() {
        String::new()
    } else {
        format!("var(--spacing-{})", step.replace('.', "-"))
    }
}

/// Derives a default element id from a label: ASCII letters and digits, others become "-".
pub(crate) fn slug(s: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in s.to_lowercase().chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            out.push(c);
            dash = false;
        } else if !dash && !out.is_empty() {
            out.push('-');
            dash = true;
        }
    }
    out.trim_end_matches('-').to_string()
}

/// A form control's id: the explicit one, else "astryx-" + slug(label).
pub(crate) fn control_id(id: &str, label: &str) -> String {
    if !id.is_empty() {
        return id.to_string();
    }
    let s = slug(label);
    if s.is_empty() {
        "astryx-input".into()
    } else {
        format!("astryx-{s}")
    }
}
