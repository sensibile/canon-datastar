//! Port of Astryx `Token` (packages/core/src/Token, TokenLink), MIT, Meta Platforms, Inc.
//! `href` makes the chip a link and `action` (e.g. a data-on:click) a button. With a remove
//! button, or as a button, the label is its own `<a>`/`<button>` inside the chip and
//! static/canon.js forwards clicks on the chip body to it. A disabled token drops its data-on:
//! handlers.

use crate::Attr;
use crate::html::{Attrs, attrs, cls, el, or, text, without_prefix};
use crate::icon::{IconProps, icon};
use maud::Markup;

/// Props for [`token`].
#[derive(Clone, Debug, Default)]
pub struct TokenProps {
    pub label: String,
    /// "sm", "md" (default), "lg".
    pub size: String,
    /// "default", "red", "orange", "yellow", "green", "teal", "cyan", "blue", "purple", "pink", "gray".
    pub color: String,
    pub icon: Option<Markup>,
    pub end_content: Option<Markup>,
    pub is_disabled: bool,
    pub href: String,
    /// The label control's attributes when the token is a button, e.g.
    /// `attr("data-on:click", "@post('/filter')")`. `Some` (even empty) makes it a button.
    pub action: Option<Vec<Attr>>,
    /// Adds a remove button (so does any `remove_attrs`); `remove_attrs` are its attributes.
    pub is_removable: bool,
    pub remove_attrs: Vec<Attr>,
    /// Names the remove target when it differs from `label`.
    pub remove_name: String,
    /// Extra text for assistive tech (aria-description).
    pub description: String,
    pub is_label_hidden: bool,
    /// Overrides the accessible name.
    pub aria_label: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Renders a compact chip.
pub fn token(p: &TokenProps) -> Markup {
    let is_button = p.action.is_some();
    let is_link = !p.href.is_empty() && !is_button;
    let removable = p.is_removable || !p.remove_attrs.is_empty();
    let size = or(&p.size, "md");
    let color = or(&p.color, "default");

    // Naming goes on whichever element is the control.
    let shared = |a: Attrs| {
        let name = if p.aria_label.is_empty() && p.is_label_hidden {
            p.label.as_str()
        } else {
            p.aria_label.as_str()
        };
        a.opt("aria-label", name)
            .opt("aria-description", &p.description)
    };
    let base = |a: Attrs, interactive: bool, owner: bool| {
        a.set("class", &cls("astryx-token", &p.class))
            .flag("data-interactive", interactive)
            .flag("data-owner", owner)
            .set("data-color", color)
            .set("data-size", size)
            .flag("data-disabled", p.is_disabled)
    };
    let link = |a: Attrs| {
        if p.is_disabled {
            a.set("role", "link")
                .set("tabindex", "0")
                .set("aria-disabled", "true")
        } else {
            a.set("href", &p.href)
        }
    };
    let handlers = |list: &[Attr]| {
        if p.is_disabled {
            without_prefix(list, "data-on:")
        } else {
            list.to_vec()
        }
    };
    let label = el(
        "span",
        attrs()
            .set("dir", "auto")
            .set("class", "astryx-token-label")
            .flag("data-hidden", p.is_label_hidden),
        [Some(text(&p.label))],
    );
    let remove = removable.then(|| {
        let ra = attrs()
            .set("type", "button")
            .set("class", "astryx-token-remove")
            .set(
                "aria-label",
                &format!("Remove {}", or(&p.remove_name, &p.label)),
            )
            .when(p.is_disabled, "aria-disabled", "true")
            .extra(&handlers(&p.remove_attrs));
        el(
            "button",
            ra,
            [Some(icon(&IconProps::sized("close", "xsm")))],
        )
    });

    if is_link && !removable {
        let a = base(shared(link(attrs())), true, false).extra(&p.attrs);
        return el("a", a, [p.icon.clone(), Some(label), p.end_content.clone()]);
    }
    if is_link || is_button {
        let inner = if is_link {
            el(
                "a",
                shared(link(attrs())).set("class", "astryx-token-inner"),
                [Some(label)],
            )
        } else {
            let ia = shared(attrs().set("type", "button"))
                .when(p.is_disabled, "aria-disabled", "true")
                .set("class", "astryx-token-inner")
                .extra(&handlers(p.action.as_deref().unwrap_or_default()));
            el("button", ia, [Some(label)])
        };
        return el(
            "span",
            base(attrs(), true, true).extra(&p.attrs),
            [p.icon.clone(), Some(inner), p.end_content.clone(), remove],
        );
    }
    el(
        "span",
        base(shared(attrs()), false, false).extra(&p.attrs),
        [p.icon.clone(), Some(label), p.end_content.clone(), remove],
    )
}
