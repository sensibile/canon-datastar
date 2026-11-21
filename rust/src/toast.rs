//! Port of Astryx `Toast` and `ToastViewport` (packages/core/src/Toast), MIT, Meta Platforms, Inc.
//! Render one toast viewport near the end of `<body>`. Show a toast from the server by patching a
//! [`toast`] into it over SSE (selector "#astryx-toasts", mode append), or from the browser with
//! `canon.toast({body: "Saved"})`. static/canon.js runs the auto-hide timer (paused while hovered
//! or focused), dismisses the oldest beyond `max_visible`, and keeps the viewport above dialogs.
//! Not ported: swipe to dismiss, renderContent.

use crate::Attr;
use crate::attr;
use crate::button::{ButtonProps, button};
use crate::html::{attrs, cls, el, or, text};
use maud::Markup;

/// Props for [`toast`].
#[derive(Clone, Debug, Default)]
pub struct ToastProps {
    /// Goes on the toast row; patch the same id again to replace a visible toast.
    pub id: String,
    /// "info" (default) or "error".
    pub kind: String,
    pub body: String,
    /// Rich body content; replaces `body`.
    pub body_markup: Option<Markup>,
    pub end_content: Option<Markup>,
    /// Turns auto-hide off. Error toasts are sticky unless `auto_hide_duration` is set.
    pub is_sticky: bool,
    /// ms, default 5000.
    pub auto_hide_duration: u32,
    /// The close button's accessible name, default "Dismiss notification".
    pub dismiss_label: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// One notification row for a toast viewport.
pub fn toast(p: &ToastProps) -> Markup {
    let kind = or(&p.kind, "info");
    let is_error = kind == "error";
    let auto_hide = !p.is_sticky && (!is_error || p.auto_hide_duration > 0);
    let duration = if p.auto_hide_duration > 0 {
        p.auto_hide_duration
    } else {
        5000
    };
    let (role, live) = if is_error {
        ("alert", "assertive")
    } else {
        ("status", "polite")
    };
    let a = attrs()
        .set("class", &cls("astryx-toast", &p.class))
        .set("data-type", kind)
        .set("role", role)
        .set("aria-live", live)
        .when(auto_hide, "data-auto-hide", &duration.to_string())
        .extra(&p.attrs);
    let body = p.body_markup.clone().unwrap_or_else(|| text(&p.body));
    let close = button(&ButtonProps {
        variant: "ghost".into(),
        size: "sm".into(),
        icon: "close".into(),
        is_icon_only: true,
        label: or(&p.dismiss_label, "Dismiss notification").into(),
        class: "astryx-toast-close".into(),
        attrs: vec![attr("data-toast-dismiss", "")],
        ..Default::default()
    });
    let div = |class: &str| attrs().set("class", class);
    el(
        "div",
        attrs().set("class", "astryx-toast-row").opt("id", &p.id),
        [Some(el(
            "div",
            div("astryx-toast-row-inner"),
            [Some(el(
                "div",
                a,
                [Some(el(
                    "div",
                    div("astryx-toast-inner"),
                    [
                        Some(el("div", div("astryx-toast-content"), [Some(body)])),
                        Some(el(
                            "div",
                            div("astryx-toast-end"),
                            [p.end_content.clone(), Some(close)],
                        )),
                    ],
                ))],
            ))],
        ))],
    )
}

/// Props for [`toast_viewport`].
#[derive(Clone, Debug, Default)]
pub struct ToastViewportProps {
    /// Defaults to "astryx-toasts", the id `canon.toast()` looks for.
    pub id: String,
    /// "bottomEnd" (default), "bottomStart", "topEnd", "topStart".
    pub position: String,
    /// How many toasts show at once, default 5.
    pub max_visible: u32,
    /// Names the region, default "Notifications".
    pub label: String,
    /// Used by toasts created with `canon.toast()`.
    pub dismiss_label: String,
}

/// The corner region toasts appear in; `children` are initial toasts.
pub fn toast_viewport(p: &ToastViewportProps, children: Markup) -> Markup {
    let max = if p.max_visible > 0 { p.max_visible } else { 5 };
    let a = attrs()
        .set("popover", "manual")
        .set("id", or(&p.id, "astryx-toasts"))
        .set("class", "astryx-toast-viewport")
        .set("data-position", or(&p.position, "bottomEnd"))
        .set("data-max-visible", &max.to_string())
        .set("role", "region")
        .set("aria-label", or(&p.label, "Notifications"));
    let tpl = el(
        "template",
        attrs().set("class", "astryx-toast-template"),
        [Some(toast(&ToastProps {
            dismiss_label: p.dismiss_label.clone(),
            ..Default::default()
        }))],
    );
    el("div", a, [Some(tpl), Some(children)])
}
