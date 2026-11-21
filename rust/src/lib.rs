//! Astryx components as server-rendered HTML ([maud] `Markup`) for Datastar apps.
//!
//! Markup (class names, `data-*` attributes) matches the Go `canon` package and the golden
//! files in `fixtures/`, and is styled by the shared `css/` stylesheets.
//! Ported from Astryx (<https://github.com/meta/astryx>), MIT, Meta Platforms, Inc.
//!
//! Props structs implement `Default`; enum-like props are strings where "" means the default.
//! Every component takes `attrs` (extra attributes, e.g. Datastar bindings), rendered in order.

mod button;
mod field;
mod html;
mod icon;
mod icons_gen;
mod inputs;
mod link;
mod misc;
mod text;
mod theme;
mod types;

pub use button::{ButtonProps, button, icon_button};
pub use field::{FieldLabelProps, FieldProps, field, field_label, field_status};
pub use icon::{IconProps, icon, icon_names};
pub use inputs::{
    CheckboxInputProps, SwitchProps, TextInputProps, checkbox_input, switch, text_input,
};
pub use link::{LinkProps, link};
pub use misc::{
    BadgeProps, DividerProps, SpinnerProps, StackProps, VisuallyHiddenProps, badge, divider,
    hstack, spinner, stack, visually_hidden, vstack,
};
pub use text::{HeadingProps, TextProps, heading, text};
pub use theme::{THEMES, ThemeMode, ThemeProps, theme};
pub use types::FieldStatus;

/// One HTML attribute, e.g. a Datastar binding `("data-bind:email".into(), "".into())`.
/// A list keeps its order so Go and Rust render identical markup.
pub type Attr = (String, String);

/// Builds an [`Attr`] from string slices.
pub fn attr(name: &str, value: &str) -> Attr {
    (name.to_string(), value.to_string())
}
