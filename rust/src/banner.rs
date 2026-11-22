//! Port of Astryx `Banner` (packages/core/src/Banner), MIT, Meta Platforms, Inc.
//! static/canon.js expands and collapses the body (firing "canon-toggle" with detail.open) and
//! dismisses the banner (hiding it, returning focus to where it came from and firing
//! "canon-dismiss"); add `dismiss_attrs` such as a data-on:click to tell the server too.
//! With role "alert", the alert role goes on the message so only it is announced.

use crate::button::{ButtonProps, icon_button};
use crate::html::{attrs, cls, el, group, or, text};
use crate::icon::{IconProps, icon};
use crate::types::slug;
use crate::{Attr, attr};
use maud::Markup;

/// Props for [`banner`].
#[derive(Clone, Debug, Default)]
pub struct BannerProps {
    /// Prefixes the content id. Defaults to "banner-" + a slug of `title`.
    pub id: String,
    /// "info" (default), "warning", "error", "success".
    pub status: String,
    pub title: String,
    pub title_markup: Option<Markup>,
    pub description: String,
    /// Replaces `description`.
    pub description_markup: Option<Markup>,
    /// Replaces the status icon.
    pub icon: Option<Markup>,
    pub is_dismissable: bool,
    /// Names the dismiss button ("Dismiss " + title by default).
    pub dismiss_label: String,
    pub dismiss_attrs: Vec<Attr>,
    pub end_content: Option<Markup>,
    /// "card" (default) or "section".
    pub container: String,
    /// "none" (default), "low", "med", "high".
    pub elevation: String,
    /// The body. It collapses behind a toggle (closed at first) unless `is_not_collapsible`.
    pub content: Option<Markup>,
    pub is_not_collapsible: bool,
    pub is_open: bool,
    /// Names a boolean Datastar signal that mirrors the body's open state.
    pub signal: String,
    pub role: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

fn bool_str(b: bool) -> &'static str {
    if b { "true" } else { "false" }
}

/// Renders a status message.
pub fn banner(p: &BannerProps) -> Markup {
    let status = or(&p.status, "info");
    let container = or(&p.container, "card");
    let id = if p.id.is_empty() {
        format!("banner-{}", or(&slug(&p.title), "message"))
    } else {
        p.id.clone()
    };
    let content_id = format!("{id}-content");
    let has_content = p.content.is_some();
    let has_toggle = has_content && !p.is_not_collapsible;
    let open = p.is_open || !has_toggle;
    let has_actions = p.end_content.is_some() || p.is_dismissable;
    let is_alert = p.role == "alert";
    let has_description = p.description_markup.is_some() || !p.description.is_empty();

    let icon_m = match &p.icon {
        Some(i) => el(
            "div",
            attrs()
                .set("class", "astryx-banner-icon")
                .set("data-status", status)
                .set("aria-hidden", "true"),
            [Some(i.clone())],
        ),
        None => icon(&IconProps {
            name: status.into(),
            size: "md".into(),
            class: "astryx-banner-icon".into(),
            attrs: vec![attr("data-status", status)],
            ..Default::default()
        }),
    };
    let title = p.title_markup.clone().unwrap_or_else(|| text(&p.title));
    let desc = has_description.then(|| {
        let d = p
            .description_markup
            .clone()
            .unwrap_or_else(|| text(&p.description));
        group([
            Some(text(" ")),
            Some(el(
                "div",
                attrs().set("class", "astryx-banner-description"),
                [Some(d)],
            )),
        ])
    });
    // With role="alert", aria-* attributes describe the alert message rather than the frame.
    let (msg_attrs, frame_attrs): (Vec<Attr>, Vec<Attr>) = p
        .attrs
        .iter()
        .cloned()
        .partition(|(n, _)| is_alert && n.starts_with("aria-"));
    let message = el(
        "div",
        attrs()
            .extra(&msg_attrs)
            .when(is_alert, "role", "alert")
            .set("class", "astryx-banner-message")
            .flag("data-with-end", p.end_content.is_some()),
        [
            Some(el(
                "div",
                attrs().set("class", "astryx-banner-title"),
                [Some(title)],
            )),
            desc,
        ],
    );

    let end = (has_actions || has_toggle).then(|| {
        let toggle = has_toggle.then(|| {
            icon_button(&ButtonProps {
                label: if open { "Collapse" } else { "Expand" }.into(),
                variant: "ghost".into(),
                size: "sm".into(),
                icon: "chevronDown".into(),
                class: "astryx-banner-control astryx-banner-toggle".into(),
                attrs: vec![
                    attr("aria-expanded", bool_str(open)),
                    attr("aria-controls", &content_id),
                ],
                ..Default::default()
            })
        });
        let dismiss = p.is_dismissable.then(|| {
            let name = if !p.dismiss_label.is_empty() {
                p.dismiss_label.clone()
            } else if p.title_markup.is_none() && !p.title.is_empty() {
                format!("Dismiss {}", p.title)
            } else {
                "Dismiss".into()
            };
            let mut a = vec![attr("data-banner-dismiss", "")];
            a.extend(p.dismiss_attrs.iter().cloned());
            icon_button(&ButtonProps {
                label: name,
                variant: "ghost".into(),
                size: "sm".into(),
                icon: "close".into(),
                class: "astryx-banner-control astryx-banner-close".into(),
                attrs: a,
                ..Default::default()
            })
        });
        el(
            "div",
            attrs().set("class", "astryx-banner-end"),
            [p.end_content.clone(), toggle, dismiss],
        )
    });
    let header = el(
        "div",
        attrs()
            .set("class", "astryx-banner")
            .set("data-container", container)
            .set("data-status", status)
            .flag("data-centered", !has_description && has_actions),
        [Some(icon_m), Some(message), end],
    );
    let content = p.content.clone().map(|c| {
        el(
            "div",
            attrs()
                .set("id", &content_id)
                .set("class", "astryx-banner-content")
                .set("data-container", container)
                .set("data-status", status)
                .flag("hidden", !open),
            [Some(c)],
        )
    });
    let mut frame = attrs()
        .set("id", &id)
        .set("class", &cls("astryx-banner-frame", &p.class))
        .set("data-container", container)
        .set("data-elevation", or(&p.elevation, "none"))
        .when(!is_alert && !p.role.is_empty(), "role", &p.role);
    if !p.signal.is_empty() && has_toggle {
        frame = frame
            .set("data-effect", &format!("canon.collapse(el, ${})", p.signal))
            .set(
                "data-on:canon-toggle",
                &format!("evt.target === el && (${} = evt.detail.open)", p.signal),
            );
    }
    el("div", frame.extra(&frame_attrs), [Some(header), content])
}
