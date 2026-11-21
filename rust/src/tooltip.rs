//! Port of Astryx `Tooltip` and `InfoTip` (packages/core/src/Tooltip, InfoTip), MIT, Meta
//! Platforms, Inc. The trigger is the first element inside (or the text itself); static/canon.js
//! shows the tooltip on hover after `delay`, on keyboard focus, and hides it on leave, blur, press
//! or Escape, and adds the tooltip to the trigger's aria-describedby.
//! Not ported: touch long-press, controlled open.

use crate::Attr;
use crate::html::{attrs, cls, el, group, or, text};
use crate::icon::{IconProps, icon};
use crate::types::slug;
use maud::Markup;

/// Props for [`tooltip`].
#[derive(Clone, Debug, Default)]
pub struct TooltipProps {
    /// Id of the tooltip element. Defaults to "tooltip-" + a slug of `content`.
    pub id: String,
    pub content: String,
    /// Rich content; replaces `content`.
    pub content_markup: Option<Markup>,
    /// "above" (default), "below", "start", "end".
    pub placement: String,
    /// "center" (default), "start", "end".
    pub alignment: String,
    /// ms before showing on hover; 0 means the default 200.
    pub delay: u32,
    /// ms before hiding after the pointer leaves.
    pub hide_delay: u32,
    /// "auto" (default, keyboard focus only), "always", "never".
    pub focus_trigger: String,
    /// Renders the trigger without a tooltip.
    pub is_disabled: bool,
    /// "auto" (default, dashed underline on plain-text triggers), "always", "never".
    pub hover_indication: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Wraps `children` (the trigger) and renders the tooltip layer after them.
pub fn tooltip(p: &TooltipProps, children: Markup) -> Markup {
    if p.is_disabled {
        return children;
    }
    let id = if p.id.is_empty() {
        format!("tooltip-{}", or(&slug(&p.content), "content"))
    } else {
        p.id.clone()
    };
    let indication = or(&p.hover_indication, "auto");
    let focus = p.focus_trigger.as_str();
    let trigger = attrs()
        .set("class", "astryx-tooltip-trigger")
        .set("data-tooltip", &id)
        .when(p.delay > 0, "data-delay", &p.delay.to_string())
        .when(
            p.hide_delay > 0,
            "data-hide-delay",
            &p.hide_delay.to_string(),
        )
        .when(
            !focus.is_empty() && focus != "auto",
            "data-focus-trigger",
            focus,
        )
        .when(indication != "never", "data-indication", indication);
    let layer = attrs()
        .set("popover", "manual")
        .set("id", &id)
        .set("role", "tooltip")
        .set("class", &cls("astryx-layer astryx-tooltip", &p.class))
        .set("data-placement", or(&p.placement, "above"))
        .set("data-alignment", or(&p.alignment, "center"))
        .extra(&p.attrs);
    let content = p.content_markup.clone().unwrap_or_else(|| text(&p.content));
    group([
        Some(el("span", trigger, [Some(children)])),
        Some(el(
            "div",
            layer,
            [Some(el(
                "div",
                attrs().set("class", "astryx-tooltip-content"),
                [Some(content)],
            ))],
        )),
    ])
}

/// Props for [`info_tip`].
#[derive(Clone, Debug, Default)]
pub struct InfoTipProps {
    /// Id of the tooltip element. Defaults to "infotip-" + a slug of `content`.
    pub id: String,
    pub content: String,
    pub content_markup: Option<Markup>,
    /// The button's accessible name. Default "More information".
    pub label: String,
    /// Icon size: "xsm", "sm" (default), "md", "lg".
    pub size: String,
    pub placement: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// A small info button that shows a tooltip on hover or focus and pins it open on click.
/// Port of Astryx `InfoTip` (packages/core/src/InfoTip), MIT, Meta Platforms, Inc.
pub fn info_tip(p: &InfoTipProps) -> Markup {
    let id = if p.id.is_empty() {
        format!("infotip-{}", or(&slug(&p.content), "content"))
    } else {
        p.id.clone()
    };
    let btn = attrs()
        .set("type", "button")
        .set("class", &cls("astryx-info-tip", &p.class))
        .set("aria-label", or(&p.label, "More information"))
        .set("aria-expanded", "false")
        .extra(&p.attrs);
    tooltip(
        &TooltipProps {
            id,
            content: p.content.clone(),
            content_markup: p.content_markup.clone(),
            placement: p.placement.clone(),
            hover_indication: "never".into(),
            ..Default::default()
        },
        el(
            "button",
            btn,
            [Some(icon(&IconProps::sized("info", or(&p.size, "sm"))))],
        ),
    )
}
