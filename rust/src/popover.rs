//! Port of Astryx `Popover` (packages/core/src/Popover), MIT, Meta Platforms, Inc.
//! A native popover anchored to the first button inside the trigger markup. static/canon.js points
//! that button at the surface (popovertarget), so clicks toggle it natively; the browser closes it
//! on an outside press or Escape and returns focus to the trigger.
//! Not ported: bottom-sheet presentation on touch, render-prop trigger.

use crate::Attr;
use crate::html::{attrs, cls, decl, el, group, or, styles, text};
use crate::types::space_css;
use maud::Markup;

/// Props for [`popover`].
#[derive(Clone, Debug, Default)]
pub struct PopoverProps {
    /// Id of the surface. Required when a page has more than one popover; default "popover".
    pub id: String,
    pub content: Option<Markup>,
    /// "below" (default), "above", "start", "end".
    pub placement: String,
    /// "start" (default), "center", "end".
    pub alignment: String,
    /// A CSS length.
    pub width: String,
    /// A spacing step, default "3".
    pub padding: String,
    /// Accessible name of the dialog surface.
    pub label: String,
    /// "dialog" (default) or "none".
    pub role: String,
    /// Traps focus inside while open, and moves focus in on open.
    pub is_modal: bool,
    /// Moves focus into the surface on open (always on for modal popovers).
    pub has_auto_focus: bool,
    /// Adds a close button that is visually hidden until focused.
    pub has_close_button: bool,
    pub close_button_label: String,
    /// Keeps it open on an outside press.
    pub no_light_dismiss: bool,
    /// Keeps it open on Escape.
    pub no_escape_dismiss: bool,
    /// A Datastar signal that mirrors the open state both ways.
    pub signal: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Wraps `trigger` and renders the surface after it.
pub fn popover(p: &PopoverProps, trigger: Markup) -> Markup {
    let id = or(&p.id, "popover");
    let mode = if p.no_light_dismiss || p.no_escape_dismiss {
        "manual"
    } else {
        "auto"
    };
    let is_dialog = p.role != "none";
    let mut a = attrs()
        .set("popover", mode)
        .set("id", id)
        .set("class", &cls("astryx-layer astryx-popover", &p.class))
        .when(is_dialog, "role", "dialog")
        .when(is_dialog && !p.label.is_empty(), "aria-label", &p.label)
        .when(is_dialog && p.is_modal, "aria-modal", "true")
        .set("data-placement", or(&p.placement, "below"))
        .set("data-alignment", or(&p.alignment, "start"))
        .flag("data-modal", p.is_modal)
        .flag("data-autofocus-open", p.is_modal || p.has_auto_focus)
        .flag("data-no-light-dismiss", p.no_light_dismiss)
        .flag("data-no-escape-dismiss", p.no_escape_dismiss)
        .set(
            "style",
            &styles(&[
                decl("width", &p.width),
                decl("--_popover-padding", &space_css(or(&p.padding, "3"))),
            ]),
        );
    if !p.signal.is_empty() {
        a = a
            .set("data-effect", &format!("canon.popover(el, ${})", p.signal))
            .set(
                "data-on:toggle",
                &format!("${} = evt.newState === 'open'", p.signal),
            );
    }
    let a = a.extra(&p.attrs);
    let close = p.has_close_button.then(|| {
        el(
            "button",
            attrs()
                .set("type", "button")
                .set("class", "astryx-popover-close")
                .flag("data-popover-close", true),
            [Some(text(or(&p.close_button_label, "Close")))],
        )
    });
    group([
        Some(el(
            "span",
            attrs()
                .set("class", "astryx-popover-anchor")
                .set("data-popover", id),
            [Some(trigger)],
        )),
        Some(el(
            "div",
            a,
            [Some(el(
                "div",
                attrs().set("class", "astryx-popover-content"),
                [p.content.clone(), close],
            ))],
        )),
    ])
}
