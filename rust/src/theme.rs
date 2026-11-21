//! Port of Astryx `Theme` (packages/core/src/theme/Theme.tsx), MIT, Meta Platforms, Inc.

use crate::Attr;
use crate::html::{attrs, el};
use maud::Markup;

/// Theme names shipped in `css/themes`.
pub const THEMES: [&str; 7] = [
    "neutral",
    "butter",
    "chocolate",
    "matcha",
    "stone",
    "gothic",
    "y2k",
];

/// A theme's color mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThemeMode {
    /// Follow the reader's OS color scheme (no `data-theme` attribute).
    #[default]
    System,
    Light,
    Dark,
}

impl ThemeMode {
    /// The `data-theme` value, or `None` for system.
    pub fn data_theme(self) -> Option<&'static str> {
        match self {
            ThemeMode::System => None,
            ThemeMode::Light => Some("light"),
            ThemeMode::Dark => Some("dark"),
        }
    }

    /// Parses "light" / "dark"; anything else is system.
    pub fn parse(s: &str) -> Self {
        match s {
            "light" => ThemeMode::Light,
            "dark" => ThemeMode::Dark,
            _ => ThemeMode::System,
        }
    }
}

/// Props for [`theme`]. Defaults: theme "neutral", mode system.
#[derive(Clone, Debug, Default)]
pub struct ThemeProps {
    /// Matches a theme stylesheet linked on the page (`css/themes/<name>.css`).
    pub name: Option<String>,
    pub mode: ThemeMode,
    /// Extra attributes, e.g. Datastar bindings such as
    /// `("data-attr:data-astryx-theme", "$theme")`. Rendered in order, after the theme attributes.
    pub attrs: Vec<Attr>,
}

/// Scopes an Astryx theme and color mode to `children`.
/// For the page root, put the same `data-astryx-theme` / `data-theme` attributes on `<html>`
/// so browser chrome and top-layer overlays follow it.
pub fn theme(props: &ThemeProps, children: Markup) -> Markup {
    let a = attrs()
        .set("class", "astryx-theme")
        .set(
            "data-astryx-theme",
            props.name.as_deref().unwrap_or("neutral"),
        )
        .opt("data-theme", props.mode.data_theme().unwrap_or(""))
        .extra(&props.attrs);
    el("div", a, [Some(children)])
}
