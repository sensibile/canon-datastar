//! Astryx components rendered with [maud] for Datastar apps.
//!
//! Markup (class names, `data-*` attributes) matches the Go `canon` package and the
//! golden files in `fixtures/`, and is styled by the shared `css/` stylesheets.
//! Ported from Astryx (<https://github.com/meta/astryx>), MIT, Meta Platforms, Inc.

mod tag;
mod theme;

pub use theme::{THEMES, ThemeMode, ThemeProps, theme};

/// One HTML attribute. A list keeps its order so Go and Rust render identical markup.
pub type Attr = (String, String);
