//! Port of Astryx `DropdownMenu` (packages/core/src/DropdownMenu), MIT, Meta Platforms, Inc.
//! A Button that opens a native popover menu (popovertarget). static/canon.js adds the menu
//! keyboard model: Enter/Space/ArrowDown opens on the first item, ArrowUp on the last; arrows,
//! Home/End and typeahead move between items; Escape closes and returns focus; Tab closes.
//! Wire item actions with Datastar, e.g. `attrs: vec![attr("data-on:click", "@post('/archive')")]`.
//! Not ported: submenus, checkbox/radio items, bottom-sheet presentation on touch.

use crate::Attr;
use crate::attr;
use crate::button::{ButtonProps, button};
use crate::html::{attrs, cls, decl, el, group, or, text, without_prefix};
use crate::icon::{IconProps, icon};
use crate::misc::{DividerProps, divider};
use crate::types::slug;
use maud::Markup;

/// Props for [`dropdown_menu`].
#[derive(Clone, Debug, Default)]
pub struct DropdownMenuProps {
    /// Id of the menu; the trigger is id + "-trigger" and the surface id + "-surface".
    /// Default "menu".
    pub id: String,
    /// The trigger Button.
    pub button: ButtonProps,
    /// Drops the chevron after the trigger label.
    pub no_chevron: bool,
    /// "below" (default), "above", "start", "end".
    pub placement: String,
    /// "start" (default), "center", "end".
    pub alignment: String,
    /// A CSS length; by default the menu is at least as wide as the trigger.
    pub menu_width: String,
    /// A CSS length.
    pub menu_max_height: String,
    pub header: Option<Markup>,
    pub footer: Option<Markup>,
    pub class: String,
}

/// Renders the trigger and the menu around `items`.
pub fn dropdown_menu(p: &DropdownMenuProps, items: Markup) -> Markup {
    let id = or(&p.id, "menu");
    let surface_id = format!("{id}-surface");
    let trigger_id = format!("{id}-trigger");
    let mut btn = p.button.clone();
    if !p.no_chevron && btn.end_content.is_none() {
        btn.end_content = Some(icon(&IconProps::sized("chevronDown", "sm")));
    }
    let mut btn_attrs = vec![
        attr("id", &trigger_id),
        attr("popovertarget", &surface_id),
        attr("aria-haspopup", "menu"),
        attr("aria-expanded", "false"),
        attr("aria-controls", id),
    ];
    btn_attrs.extend(btn.attrs);
    btn.attrs = btn_attrs;
    let surface = attrs()
        .set("popover", "auto")
        .set("id", &surface_id)
        .set(
            "class",
            &cls("astryx-layer astryx-dropdown-menu-surface", &p.class),
        )
        .set("data-placement", or(&p.placement, "below"))
        .set("data-alignment", or(&p.alignment, "start"))
        .flag("data-match-width", p.menu_width.is_empty())
        .opt("style", &decl("min-width", &p.menu_width));
    let max_height = if p.menu_max_height.is_empty() {
        String::new()
    } else {
        format!("max-height: min({}, calc(100vh - 16px))", p.menu_max_height)
    };
    let menu = attrs()
        .set("id", id)
        .set("role", "menu")
        .set("tabindex", "-1")
        .set("aria-labelledby", &trigger_id)
        .set("class", "astryx-dropdown-menu")
        .opt("style", &max_height);
    group([
        Some(button(&btn)),
        Some(el(
            "div",
            surface,
            [
                region(&p.header),
                Some(el("div", menu, [Some(items)])),
                region(&p.footer),
            ],
        )),
    ])
}

fn region(c: &Option<Markup>) -> Option<Markup> {
    c.clone().map(|c| {
        el(
            "div",
            attrs().set("class", "astryx-dropdown-menu-region"),
            [Some(c)],
        )
    })
}

/// Props for [`dropdown_menu_item`].
#[derive(Clone, Debug, Default)]
pub struct DropdownMenuItemProps {
    pub label: String,
    pub description: String,
    /// A leading built-in glyph name.
    pub icon: String,
    /// Renders a link item.
    pub href: String,
    pub target: String,
    pub rel: String,
    pub is_disabled: bool,
    /// "default" or "destructive".
    pub variant: String,
    /// Leaves the menu open after this item is chosen.
    pub keep_open: bool,
    pub end_content: Option<Markup>,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// One menu row: a `<button>`, or an `<a>` when `href` is set.
pub fn dropdown_menu_item(p: &DropdownMenuItemProps) -> Markup {
    let variant = or(&p.variant, "default");
    let is_link = !p.href.is_empty() && !p.is_disabled;
    let (tag, mut a) = if is_link {
        let rel = if p.target == "_blank" && p.rel.is_empty() {
            "noopener noreferrer"
        } else {
            p.rel.as_str()
        };
        (
            "a",
            attrs()
                .set("href", &p.href)
                .opt("target", &p.target)
                .opt("rel", rel),
        )
    } else {
        ("button", attrs().set("type", "button"))
    };
    a = a
        .set("role", "menuitem")
        .set("tabindex", "-1")
        .when(is_link, "draggable", "false")
        .set("class", &cls("astryx-dropdown-menu-item", &p.class))
        .set("data-variant", variant)
        .when(p.is_disabled, "aria-disabled", "true")
        .flag("data-keep-open", p.keep_open);
    a = if p.is_disabled {
        a.extra(&without_prefix(&p.attrs, "data-on:"))
    } else {
        a.extra(&p.attrs)
    };
    let icon_m = (!p.icon.is_empty()).then(|| {
        let color = if variant == "destructive" {
            "error"
        } else {
            "secondary"
        };
        icon(&IconProps {
            name: p.icon.clone(),
            size: "sm".into(),
            color: color.into(),
            ..Default::default()
        })
    });
    let desc = (!p.description.is_empty()).then(|| {
        el(
            "span",
            attrs().set("class", "astryx-dropdown-menu-item-description"),
            [Some(text(&p.description))],
        )
    });
    let end = p.end_content.clone().map(|e| {
        el(
            "span",
            attrs().set("class", "astryx-dropdown-menu-item-end"),
            [Some(e)],
        )
    });
    el(
        tag,
        a,
        [
            icon_m,
            Some(el(
                "span",
                attrs().set("class", "astryx-dropdown-menu-item-text"),
                [
                    Some(el(
                        "span",
                        attrs().set("class", "astryx-dropdown-menu-item-label"),
                        [Some(text(&p.label))],
                    )),
                    desc,
                ],
            )),
            end,
        ],
    )
}

/// Props for [`dropdown_menu_group`].
#[derive(Clone, Debug, Default)]
pub struct DropdownMenuGroupProps {
    pub title: String,
    /// Id of the group; the title is id + "-title". Defaults to "menu-group-" + a slug of `title`.
    pub id: String,
}

/// A titled group of menu items (`children`).
pub fn dropdown_menu_group(p: &DropdownMenuGroupProps, children: Markup) -> Markup {
    let title_id = if p.title.is_empty() {
        String::new()
    } else if p.id.is_empty() {
        format!("menu-group-{}-title", slug(&p.title))
    } else {
        format!("{}-title", p.id)
    };
    let title = (!p.title.is_empty()).then(|| {
        el(
            "div",
            attrs()
                .set("class", "astryx-dropdown-menu-group-title")
                .set("id", &title_id),
            [Some(text(&p.title))],
        )
    });
    el(
        "div",
        attrs()
            .set("class", "astryx-dropdown-menu-group")
            .set("role", "group")
            .opt("aria-labelledby", &title_id),
        [title, Some(children)],
    )
}

/// A separator between menu items.
pub fn dropdown_menu_divider() -> Markup {
    el(
        "div",
        attrs()
            .set("class", "astryx-dropdown-menu-divider")
            .set("role", "separator")
            .set("aria-orientation", "horizontal"),
        [Some(divider(&DividerProps::default()))],
    )
}
