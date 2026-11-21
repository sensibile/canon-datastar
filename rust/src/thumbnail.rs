//! Port of Astryx `Thumbnail` (packages/core/src/Thumbnail), MIT, Meta Platforms, Inc.
//! A 64px image tile. The placeholder is rendered hidden next to an image and static/canon.js
//! shows it if the image fails to load.

use crate::Attr;
use crate::attr;
use crate::button::{ButtonProps, icon_button};
use crate::content::{SkeletonProps, skeleton};
use crate::html::{attrs, cls, el, group, text, void, without_prefix};
use crate::misc::{SpinnerProps, VisuallyHiddenProps, spinner, visually_hidden};
use crate::tooltip::{TooltipProps, tooltip};
use maud::Markup;

/// Props for [`thumbnail`].
#[derive(Clone, Debug, Default)]
pub struct ThumbnailProps {
    pub src: String,
    /// Describes the image; without it the image is decorative.
    pub alt: String,
    /// A file name or caption, shown as a tooltip.
    pub label: String,
    pub tooltip_id: String,
    /// `Some` (e.g. a data-on:click) makes the image an "open" button.
    pub action: Option<Vec<Attr>>,
    /// Adds a remove button (so does any `remove_attrs`); `remove_attrs` are its attributes.
    pub is_removable: bool,
    pub remove_attrs: Vec<Attr>,
    /// Shows a skeleton, or a spinner over an existing image.
    pub is_loading: bool,
    pub is_disabled: bool,
    /// "hover" (default) or "always".
    pub show_remove_on: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

const PLACEHOLDER: &str = "M21 19V5a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2M8.5 13.5l2.5 3 3.5-4.5 4.5 6H5l3.5-5.5z";

/// Renders an image tile.
pub fn thumbnail(p: &ThumbnailProps) -> Markup {
    let has_src = !p.src.is_empty();
    let show_skeleton = p.is_loading && !has_src;
    let show_image = has_src;
    let is_interactive = p.action.is_some() && !p.is_disabled && !p.is_loading;
    let removable = p.is_removable || !p.remove_attrs.is_empty();
    let name = match (p.label.is_empty(), p.alt.is_empty()) {
        (false, false) => format!("{} — {}", p.label, p.alt),
        (false, true) => p.label.clone(),
        (true, false) => p.alt.clone(),
        (true, true) => "Thumbnail".into(),
    };

    let image = show_image.then(|| {
        void(
            "img",
            attrs()
                .set("src", &p.src)
                .set("alt", &p.alt)
                .when(p.alt.is_empty(), "role", "presentation")
                .when(p.alt.is_empty(), "aria-hidden", "true")
                .set("class", "astryx-thumbnail-image"),
        )
    });
    let skel = show_skeleton.then(|| {
        skeleton(&SkeletonProps {
            radius: "2".into(),
            ..Default::default()
        })
    });
    let placeholder = (!show_skeleton).then(|| {
        el(
            "div",
            attrs()
                .set("class", "astryx-thumbnail-placeholder")
                .flag("hidden", show_image),
            [Some(el(
                "svg",
                attrs()
                    .set("width", "24")
                    .set("height", "24")
                    .set("viewBox", "0 0 24 24")
                    .set("fill", "currentColor")
                    .set("aria-hidden", "true"),
                [Some(el("path", attrs().set("d", PLACEHOLDER), []))],
            ))],
        )
    });
    let mut visual = group([image, skel, placeholder]);
    if let Some(action) = &p.action {
        let handlers = if is_interactive {
            action.clone()
        } else {
            without_prefix(action, "data-on:")
        };
        let a = attrs()
            .set("type", "button")
            .set("class", "astryx-thumbnail-open")
            .set("aria-label", &format!("Open {name}"))
            .when(!is_interactive, "aria-disabled", "true")
            .extra(&handlers);
        visual = el("button", a, [Some(visual)]);
    }
    let border = show_image.then(|| {
        el(
            "div",
            attrs().set("class", "astryx-thumbnail-inset-border"),
            [],
        )
    });
    let overlay = (p.is_loading && has_src).then(|| {
        el(
            "div",
            attrs().set("class", "astryx-thumbnail-upload-overlay"),
            [
                Some(spinner(&SpinnerProps {
                    size: "sm".into(),
                    shade: "onMedia".into(),
                    attrs: vec![attr("aria-hidden", "true")],
                    ..Default::default()
                })),
                Some(visually_hidden(
                    &VisuallyHiddenProps {
                        attrs: vec![attr("role", "status")],
                        ..Default::default()
                    },
                    text("Loading"),
                )),
            ],
        )
    });
    let remove = removable.then(|| {
        el(
            "div",
            attrs().set("class", "astryx-thumbnail-remove-slot"),
            [Some(icon_button(&ButtonProps {
                label: format!("Remove {name}"),
                icon: "close".into(),
                variant: "secondary".into(),
                size: "sm".into(),
                is_disabled: p.is_disabled,
                class: "astryx-thumbnail-remove".into(),
                attrs: p.remove_attrs.clone(),
                ..Default::default()
            }))],
        )
    });
    let frame = attrs()
        .set("class", "astryx-thumbnail-frame")
        .flag("data-interactive", is_interactive)
        .flag("data-reveal", removable && p.show_remove_on != "always");
    let a = attrs()
        .set("role", "group")
        .set("aria-label", &name)
        .set("class", &cls("astryx-thumbnail", &p.class))
        .flag("data-disabled", p.is_disabled)
        .extra(&p.attrs);
    let tile = el(
        "div",
        a,
        [Some(el(
            "div",
            frame,
            [Some(visual), border, overlay, remove],
        ))],
    );
    if p.label.is_empty() {
        return tile;
    }
    tooltip(
        &TooltipProps {
            id: p.tooltip_id.clone(),
            content: p.label.clone(),
            ..Default::default()
        },
        tile,
    )
}
