//! Blockquote, Code, Kbd, EmptyState, Skeleton, StatusDot. Ported from Astryx
//! (packages/core/src/…), MIT, Meta Platforms, Inc.

use crate::Attr;
use crate::html::{attrs, cls, decl, el, or, styles, text};
use crate::icon::{IconProps, icon};
use crate::tooltip::{TooltipProps, tooltip};
use maud::Markup;

/// Props for [`blockquote`].
#[derive(Clone, Debug, Default)]
pub struct BlockquoteProps {
    /// Attribution shown under the quote; `cite_markup` replaces it.
    pub cite: String,
    pub cite_markup: Option<Markup>,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Renders a quotation around `children`.
pub fn blockquote(p: &BlockquoteProps, children: Markup) -> Markup {
    let cite = p
        .cite_markup
        .clone()
        .or_else(|| (!p.cite.is_empty()).then(|| text(&p.cite)))
        .map(|c| {
            el(
                "cite",
                attrs().set("class", "astryx-blockquote-cite"),
                [Some(c)],
            )
        });
    el(
        "blockquote",
        attrs()
            .set("class", &cls("astryx-blockquote", &p.class))
            .extra(&p.attrs),
        [Some(children), cite],
    )
}

/// Props for [`code`].
#[derive(Clone, Debug, Default)]
pub struct CodeProps {
    /// "primary" (default), "secondary", "inherit".
    pub color: String,
    /// "" or "inherit" (takes the surrounding font size and line height).
    pub size: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Renders inline code around `children`.
pub fn code(p: &CodeProps, children: Markup) -> Markup {
    let a = attrs()
        .set("class", &cls("astryx-code", &p.class))
        .set("dir", "ltr")
        .set("data-color", or(&p.color, "primary"))
        .opt("data-size", &p.size)
        .extra(&p.attrs);
    el("code", a, [Some(children)])
}

/// Props for [`kbd`].
#[derive(Clone, Debug, Default)]
pub struct KbdProps {
    /// A "+"-joined shortcut, e.g. "mod+shift+p". "mod" is ⌘ on Apple platforms and Ctrl
    /// elsewhere: the server renders the non-Apple form and static/canon.js switches it.
    pub keys: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

fn kbd_display(k: &str) -> Option<&'static str> {
    Some(match k {
        "enter" => "↵",
        "backspace" => "⌫",
        "tab" => "⇥",
        "up" => "↑",
        "down" => "↓",
        "left" => "←",
        "right" => "→",
        "plus" => "+",
        _ => return None,
    })
}

fn kbd_short(k: &str) -> Option<&'static str> {
    Some(match k {
        "mod" | "ctrl" => "Ctrl",
        "alt" => "Alt",
        "shift" => "Shift",
        "meta" => "Win",
        _ => return None,
    })
}

fn kbd_label(k: &str) -> Option<&'static str> {
    Some(match k {
        "mod" | "ctrl" => "Control",
        "alt" => "Alt",
        "shift" => "Shift",
        "enter" => "Enter",
        "backspace" => "Backspace",
        "escape" => "Escape",
        "tab" => "Tab",
        "up" => "Up arrow",
        "down" => "Down arrow",
        "left" => "Left arrow",
        "right" => "Right arrow",
        "plus" => "Plus",
        _ => return None,
    })
}

/// Renders a keyboard shortcut.
pub fn kbd(p: &KbdProps) -> Markup {
    let keys: Vec<String> = p
        .keys
        .split('+')
        .map(|k| match k.trim().to_lowercase().as_str() {
            "esc" => "escape".to_string(),
            "return" => "enter".to_string(),
            k => k.to_string(),
        })
        .collect();
    let labels: Vec<String> = keys
        .iter()
        .map(|k| kbd_label(k).map_or_else(|| k.to_uppercase(), str::to_string))
        .collect();
    let caps = keys.iter().map(|k| {
        let shown = match kbd_short(k) {
            Some(s) => s.to_string(),
            None if k == "escape" => "Esc".to_string(),
            None => kbd_display(k).map_or_else(|| k.to_uppercase(), str::to_string),
        };
        Some(el(
            "kbd",
            attrs()
                .set("aria-hidden", "true")
                .set("class", "astryx-kbd-key"),
            [Some(text(&shown))],
        ))
    });
    let a = attrs()
        .set("role", "img")
        .set("aria-label", &labels.join(" + "))
        .set("class", &cls("astryx-kbd", &p.class))
        .set("data-keys", &keys.join("+"))
        .extra(&p.attrs);
    el("span", a, caps)
}

/// Props for [`empty_state`].
#[derive(Clone, Debug, Default)]
pub struct EmptyStateProps {
    pub title: String,
    pub description: String,
    /// Goes above the title.
    pub icon: Option<Markup>,
    /// Goes below the text.
    pub actions: Option<Markup>,
    /// 1–6 (default 3).
    pub heading_level: u8,
    pub is_compact: bool,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Renders a placeholder for an empty view.
pub fn empty_state(p: &EmptyStateProps) -> Markup {
    let variant = if p.is_compact { "compact" } else { "" };
    let level = if (1..=6).contains(&p.heading_level) {
        p.heading_level
    } else {
        3
    };
    let icon_m = p
        .icon
        .clone()
        .map(|i| el("div", attrs().set("aria-hidden", "true"), [Some(i)]));
    let desc = (!p.description.is_empty()).then(|| {
        el(
            "div",
            attrs()
                .set("class", "astryx-empty-state-description")
                .opt("data-variant", variant),
            [Some(text(&p.description))],
        )
    });
    let actions = p.actions.clone().map(|a| {
        el(
            "div",
            attrs().set("class", "astryx-empty-state-actions"),
            [Some(a)],
        )
    });
    let a = attrs()
        .set("class", &cls("astryx-empty-state", &p.class))
        .opt("data-variant", variant)
        .extra(&p.attrs);
    // Changes to the text are announced politely, like Astryx AnnounceRenderedText.
    let body = el(
        "div",
        attrs()
            .set("class", "astryx-empty-state-text")
            .set("aria-live", "polite"),
        [
            Some(el(
                &format!("h{level}"),
                attrs()
                    .set("class", "astryx-empty-state-title")
                    .opt("data-variant", variant),
                [Some(text(&p.title))],
            )),
            desc,
        ],
    );
    el("div", a, [icon_m, Some(body), actions])
}

/// Props for [`skeleton`].
#[derive(Clone, Debug, Default)]
pub struct SkeletonProps {
    /// CSS lengths ("100%" by default).
    pub width: String,
    pub height: String,
    /// "none", "0", "1", "2", "3" (default), "4", "rounded".
    pub radius: String,
    /// Staggers the fade-in start in a list of skeletons.
    pub index: u32,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Renders a loading placeholder.
pub fn skeleton(p: &SkeletonProps) -> Markup {
    let a = attrs()
        .set("aria-hidden", "true")
        .set("class", &cls("astryx-skeleton", &p.class))
        .set("data-radius", or(&p.radius, "3"))
        .set(
            "style",
            &styles(&[
                decl("width", or(&p.width, "100%")),
                decl("height", or(&p.height, "100%")),
                decl("animation-delay", &format!("{}ms", 1000 + 100 * p.index)),
            ]),
        )
        .extra(&p.attrs);
    el("div", a, [])
}

/// Props for [`status_dot`].
#[derive(Clone, Debug, Default)]
pub struct StatusDotProps {
    /// "success", "warning", "error", "accent", "neutral" (default).
    pub variant: String,
    /// The accessible name.
    pub label: String,
    pub is_pulsing: bool,
    /// Shows on hover and focus (the dot becomes focusable).
    pub tooltip: String,
    pub tooltip_id: String,
    /// A built-in glyph drawn inside the dot.
    pub icon: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Renders a small colored status indicator.
pub fn status_dot(p: &StatusDotProps) -> Markup {
    let a = attrs()
        .when(!p.tooltip.is_empty(), "tabindex", "0")
        .set("role", "img")
        .set("aria-label", &p.label)
        .set("class", &cls("astryx-status-dot", &p.class))
        .set("data-variant", or(&p.variant, "neutral"))
        .flag("data-pulsing", p.is_pulsing)
        .extra(&p.attrs);
    let icon_m = (!p.icon.is_empty()).then(|| {
        el(
            "span",
            attrs()
                .set("class", "astryx-status-dot-icon")
                .set("aria-hidden", "true"),
            [Some(icon(&IconProps::sized(&p.icon, "xsm")))],
        )
    });
    let dot = el("span", a, [icon_m]);
    if p.tooltip.is_empty() {
        return dot;
    }
    tooltip(
        &TooltipProps {
            id: p.tooltip_id.clone(),
            content: p.tooltip.clone(),
            ..Default::default()
        },
        dot,
    )
}
