//! Port of Astryx `Dialog` and `DialogHeader` (packages/core/src/Dialog), MIT, Meta Platforms, Inc.
//! A modal native `<dialog>`. Open it with `canon.dialog(id)` or a Datastar signal (`signal`).
//! `purpose` decides how it can be dismissed: "info" closes on Escape and backdrop press, "form"
//! on Escape only, "required" only by its own controls. Focus moves to the DialogHeader title on
//! open and returns to the opener on close.
//! Not ported: isInline, custom position, directional animation.

use crate::Attr;
use crate::attr;
use crate::button::{ButtonProps, button};
use crate::html::{attrs, cls, decl, el, or, styles, text as text_node};
use crate::misc::{DividerProps, divider};
use crate::text::{HeadingProps, TextProps, heading, text};
use crate::types::space_css;
use maud::Markup;

/// Props for [`dialog`].
#[derive(Clone, Debug, Default)]
pub struct DialogProps {
    pub id: String,
    /// A CSS length, default "400px".
    pub width: String,
    /// A CSS length, default "75dvh".
    pub max_height: String,
    /// "standard" (default) or "fullscreen".
    pub variant: String,
    /// "info" (default), "form", "required".
    pub purpose: String,
    /// A spacing step, default "4".
    pub padding: String,
    /// Names the dialog when there is no DialogHeader.
    pub label: String,
    /// A boolean Datastar signal that opens and closes the dialog; closing it in the browser
    /// sets the signal back to false.
    pub signal: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Renders a `<dialog>` around `children`.
pub fn dialog(p: &DialogProps, children: Markup) -> Markup {
    let variant = or(&p.variant, "standard");
    let purpose = or(&p.purpose, "info");
    let closed_by = match purpose {
        "info" => "any",
        "form" => "closerequest",
        "required" => "none",
        _ => "",
    };
    let size = if variant == "fullscreen" {
        String::new()
    } else {
        styles(&[
            decl("width", or(&p.width, "400px")),
            decl("max-height", or(&p.max_height, "75dvh")),
        ])
    };
    let mut a = attrs()
        .opt("id", &p.id)
        .set("class", &cls("astryx-dialog", &p.class))
        .set("data-variant", variant)
        .set("data-purpose", purpose)
        .opt("closedby", closed_by)
        .opt("aria-label", &p.label)
        .set(
            "style",
            &styles(&[
                size,
                decl("--_dialog-padding", &space_css(or(&p.padding, "4"))),
            ]),
        );
    if !p.signal.is_empty() {
        a = a
            .set("data-effect", &format!("canon.dialog(el, ${})", p.signal))
            .set("data-on:close", &format!("${} = false", p.signal));
    }
    el(
        "dialog",
        a.extra(&p.attrs),
        [Some(el(
            "div",
            attrs().set("class", "astryx-dialog-inner"),
            [Some(children)],
        ))],
    )
}

/// Props for [`dialog_header`].
#[derive(Clone, Debug, Default)]
pub struct DialogHeaderProps {
    pub title: String,
    /// Rich title content; replaces `title`.
    pub title_markup: Option<Markup>,
    pub subtitle: String,
    /// The title's id; canon.js otherwise derives one to label the dialog.
    pub title_id: String,
    /// Adds a close button that closes the enclosing dialog.
    pub has_close_button: bool,
    /// The close button's accessible name, default "Close".
    pub close_label: String,
    pub start_content: Option<Markup>,
    pub end_content: Option<Markup>,
    pub has_divider: bool,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Renders the title row of a Dialog.
pub fn dialog_header(p: &DialogHeaderProps) -> Markup {
    let title = p
        .title_markup
        .clone()
        .unwrap_or_else(|| text_node(&p.title));
    let start = p.start_content.clone().map(|c| {
        el(
            "div",
            attrs().set("class", "astryx-dialog-header-actions"),
            [Some(c)],
        )
    });
    let end = (p.end_content.is_some() || p.has_close_button).then(|| {
        let close = p.has_close_button.then(|| {
            button(&ButtonProps {
                variant: "ghost".into(),
                label: or(&p.close_label, "Close").into(),
                icon: "close".into(),
                is_icon_only: true,
                attrs: vec![attr("data-dialog-close", "")],
                ..Default::default()
            })
        });
        el(
            "div",
            attrs().set(
                "class",
                "astryx-dialog-header-actions astryx-dialog-header-end",
            ),
            [p.end_content.clone(), close],
        )
    });
    let subtitle = (!p.subtitle.is_empty()).then(|| {
        text(
            &TextProps {
                kind: "body".into(),
                size: "sm".into(),
                color: "secondary".into(),
                ..Default::default()
            },
            text_node(&p.subtitle),
        )
    });
    let divider = p.has_divider.then(|| divider(&DividerProps::default()));
    let mut heading_attrs = Vec::new();
    if !p.title_id.is_empty() {
        heading_attrs.push(attr("id", &p.title_id));
    }
    heading_attrs.push(attr("tabindex", "-1"));
    heading_attrs.push(attr("autofocus", ""));
    let heading = heading(
        &HeadingProps {
            level: 2,
            class: "astryx-dialog-title".into(),
            attrs: heading_attrs,
            ..Default::default()
        },
        title,
    );
    let row = el(
        "div",
        attrs()
            .set("class", "astryx-dialog-header-row")
            .flag("data-has-close", p.has_close_button),
        [
            start,
            Some(el(
                "div",
                attrs().set("class", "astryx-dialog-header-titles"),
                [Some(heading), subtitle],
            )),
            end,
        ],
    );
    el(
        "div",
        attrs()
            .set("class", &cls("astryx-dialog-header", &p.class))
            .extra(&p.attrs),
        [Some(row), divider],
    )
}
