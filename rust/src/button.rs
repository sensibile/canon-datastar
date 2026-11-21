//! Port of Astryx `Button` / `IconButton` (packages/core/src/Button), MIT, Meta Platforms, Inc.
//! Differences: a disabled button uses the native disabled attribute; the async clickAction is
//! left to Datastar (bind loading state with `data-attr:aria-busy="$saving"`, or data-indicator).
//! Not ported: tooltip (comes with overlays), ButtonGroup integration.

use crate::Attr;
use crate::html::{attrs, cls, decl, el, or, text};
use crate::icon::{IconProps, icon};
use crate::misc::{SpinnerProps, spinner};
use maud::Markup;

/// Props for [`button`] and [`icon_button`].
#[derive(Clone, Debug, Default)]
pub struct ButtonProps {
    /// Visible text, and the accessible name when icon-only or loading.
    pub label: String,
    /// "secondary" (default), "primary", "ghost", "destructive".
    pub variant: String,
    /// "sm", "md" (default), "lg".
    pub size: String,
    /// "none" (default), "low", "med", "high".
    pub elevation: String,
    /// "button" (default), "submit", "reset".
    pub kind: String,
    pub is_disabled: bool,
    /// Shows the spinner and sets aria-busy. Datastar can toggle it client-side through
    /// aria-busy, since the spinner is always rendered.
    pub is_loading: bool,
    /// Keeps the button clickable while loading; the spinner fades in late.
    pub is_interruptible: bool,
    /// A leading built-in glyph name.
    pub icon: String,
    pub is_icon_only: bool,
    /// A CSS length, e.g. "100%".
    pub width: String,
    /// Replaces the visible label; `label` stays the accessible name.
    pub content: Option<Markup>,
    /// Goes after the label, e.g. a badge.
    pub end_content: Option<Markup>,
    /// Renders an `<a>` (unless disabled).
    pub href: String,
    pub target: String,
    pub rel: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Renders an Astryx button.
pub fn button(p: &ButtonProps) -> Markup {
    let size = or(&p.size, "md");
    let as_link = !p.href.is_empty() && !p.is_disabled;
    let busy_blocks = p.is_loading && !p.is_interruptible && !p.is_disabled;
    let mut a = attrs()
        .set("class", &cls("astryx-button", &p.class))
        .set("data-variant", or(&p.variant, "secondary"))
        .set("data-size", size)
        .set("data-elevation", or(&p.elevation, "none"))
        .flag("data-icon-only", p.is_icon_only)
        .flag("data-disabled", p.is_disabled)
        .flag("data-interruptible", p.is_interruptible)
        .when(
            p.is_icon_only || p.is_loading || p.content.is_some(),
            "aria-label",
            &p.label,
        )
        .when(p.is_loading, "aria-busy", "true")
        .when(busy_blocks, "aria-disabled", "true");
    let tag = if as_link {
        a = a
            .set("href", &p.href)
            .opt("target", &p.target)
            .opt("rel", &p.rel);
        "a"
    } else {
        a = a
            .set("type", or(&p.kind, "button"))
            .flag("disabled", p.is_disabled);
        "button"
    };
    let a = a.opt("style", &decl("width", &p.width)).extra(&p.attrs);

    let icon_m = (!p.icon.is_empty()).then(|| {
        let icon_size = if size == "lg" { "md" } else { "sm" };
        el(
            "span",
            attrs()
                .set("class", "astryx-button-icon")
                .set("data-icon-size", icon_size),
            [Some(icon(&IconProps::sized(&p.icon, icon_size)))],
        )
    });
    let (label, end) = if p.is_icon_only {
        (None, None)
    } else {
        let content = p.content.clone().unwrap_or_else(|| text(&p.label));
        (
            Some(el(
                "span",
                attrs().set("class", "astryx-button-label"),
                [Some(content)],
            )),
            p.end_content
                .clone()
                .map(|e| el("span", attrs().set("class", "astryx-button-end"), [Some(e)])),
        )
    };
    el(
        tag,
        a,
        [
            Some(el(
                "span",
                attrs()
                    .set("class", "astryx-button-spinner")
                    .set("aria-hidden", "true"),
                [Some(spinner(&SpinnerProps::of("sm", "inherit")))],
            )),
            Some(el(
                "span",
                attrs().set("class", "astryx-button-content"),
                [icon_m, label, end],
            )),
        ],
    )
}

/// A [`button`] that shows only its icon; `label` is the accessible name.
pub fn icon_button(p: &ButtonProps) -> Markup {
    button(&ButtonProps {
        is_icon_only: true,
        ..p.clone()
    })
}
