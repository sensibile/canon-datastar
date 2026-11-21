//! The demo's Overlays page. Mirrors go/cmd/demo/overlays.templ.

use canon_datastar::*;
use maud::{Markup, html};

fn s(v: &str) -> String {
    v.to_string()
}

fn section(title: &str, body: Markup) -> Markup {
    html! {
        section.demo-section {
            h2 { (title) }
            div.demo-row { (body) }
        }
    }
}

fn btn(label: &str, variant: &str, attrs: Vec<Attr>) -> Markup {
    button(&ButtonProps {
        label: s(label),
        variant: s(variant),
        attrs,
        ..Default::default()
    })
}

fn filters_content() -> Markup {
    html! {
        div.demo-stack {
            (heading(&HeadingProps { level: 4, ..Default::default() }, html! { "Filters" }))
            (checkbox_input(&CheckboxInputProps { label: s("Only open issues"), id: s("f-open"), ..Default::default() }))
            (checkbox_input(&CheckboxInputProps { label: s("Assigned to me"), id: s("f-mine"), ..Default::default() }))
        }
    }
}

fn share_content() -> Markup {
    html! {
        div.demo-stack {
            (text_input(&TextInputProps {
                label: s("Invite by email"),
                id: s("share-email"),
                placeholder: s("name@example.com"),
                ..Default::default()
            }))
            (btn("Send invite", "primary", vec![attr("data-on:click", "$shareOpen = false; canon.toast({body: 'Invite sent'})")]))
        }
    }
}

fn item(p: DropdownMenuItemProps) -> Markup {
    dropdown_menu_item(&p)
}

fn menu_items() -> Markup {
    html! {
        (item(DropdownMenuItemProps {
            label: s("Edit"),
            icon: s("wrench"),
            attrs: vec![attr("data-on:click", "canon.toast({body: 'Edit chosen'})")],
            ..Default::default()
        }))
        (item(DropdownMenuItemProps {
            label: s("Duplicate"),
            icon: s("copy"),
            description: s("Makes a copy in this folder"),
            attrs: vec![attr("data-on:click", "@post('/demo/toast?body=Duplicated')")],
            ..Default::default()
        }))
        (item(DropdownMenuItemProps { label: s("Archive"), is_disabled: true, ..Default::default() }))
        (dropdown_menu_divider())
        (dropdown_menu_group(
            &DropdownMenuGroupProps { title: s("Links"), ..Default::default() },
            item(DropdownMenuItemProps {
                label: s("Astryx docs"),
                icon: s("externalLink"),
                href: s("https://astryx.atmeta.com"),
                target: s("_blank"),
                ..Default::default()
            }),
        ))
        (dropdown_menu_divider())
        (item(DropdownMenuItemProps {
            label: s("Delete"),
            icon: s("close"),
            variant: s("destructive"),
            attrs: vec![attr("data-on:click", "$confirmOpen = true")],
            ..Default::default()
        }))
    }
}

fn tab(value: &str, label: &str, icon: &str, disabled: bool) -> TabProps {
    let linked = !disabled;
    TabProps {
        value: s(value),
        label: s(label),
        icon: s(icon),
        id: if linked {
            format!("tab-{value}")
        } else {
            String::new()
        },
        panel_id: if linked {
            format!("panel-{value}")
        } else {
            String::new()
        },
        is_disabled: disabled,
        ..Default::default()
    }
}

fn panel(value: &str, selected: bool, body: Markup) -> Markup {
    tab_panel(
        &TabPanelProps {
            id: format!("panel-{value}"),
            tab_id: format!("tab-{value}"),
            value: s(value),
            signal: s("tab"),
            is_selected: selected,
            ..Default::default()
        },
        html! { p.demo-panel { (body) } },
    )
}

pub(crate) fn page() -> Markup {
    html! {
        h1 { "Overlays" }
        p.demo-lead { "Native popover and dialog elements, positioned and wired by static/canon.js, driven by Datastar signals and SSE." }
        div data-signals="{confirmOpen: false, shareOpen: false, tab: 'overview', deleted: 0}" {
            (section("Tooltip", html! {
                (tooltip(&TooltipProps { content: s("Copies the link to your clipboard"), ..Default::default() },
                    button(&ButtonProps { label: s("Copy link"), icon: s("copy"), ..Default::default() })))
                (tooltip(&TooltipProps { content: s("Shows below, after 600ms"), id: s("tip-below"), placement: s("below"), delay: 600, ..Default::default() },
                    btn("Slow tooltip", "ghost", vec![])))
                span {
                    "Plain text with a "
                    (tooltip(&TooltipProps { content: s("A word with a tooltip"), id: s("tip-text"), ..Default::default() }, html! { "tooltip" }))
                }
                span {
                    "Billing cycle"
                    (info_tip(&InfoTipProps { content: s("You are billed on the first day of each month."), ..Default::default() }))
                }
            }))
            (section("Popover", html! {
                (popover(&PopoverProps { id: s("filters"), label: s("Filters"), content: Some(filters_content()), width: s("260px"), ..Default::default() },
                    button(&ButtonProps { label: s("Filters"), icon: s("funnel"), ..Default::default() })))
                (popover(&PopoverProps {
                    id: s("share"), label: s("Share"), content: Some(share_content()), width: s("300px"),
                    is_modal: true, has_close_button: true, signal: s("shareOpen"), ..Default::default()
                }, btn("Share (modal, signal)", "primary", vec![])))
                span.demo-note data-text="'shareOpen: ' + $shareOpen" {}
            }))
            (section("DropdownMenu", html! {
                (dropdown_menu(&DropdownMenuProps { id: s("actions"), button: ButtonProps { label: s("Actions"), ..Default::default() }, ..Default::default() },
                    menu_items()))
                (dropdown_menu(&DropdownMenuProps {
                    id: s("more"),
                    button: ButtonProps { label: s("More"), icon: s("moreHorizontal"), is_icon_only: true, variant: s("ghost"), ..Default::default() },
                    no_chevron: true, menu_width: s("220px"), alignment: s("end"), ..Default::default()
                }, menu_items()))
            }))
            (section("Dialog", html! {
                (btn("Delete project", "destructive", vec![attr("data-on:click", "$confirmOpen = true")]))
                (btn("Open settings", "", vec![attr("data-on:click", "canon.dialog('settings')")]))
                span.demo-note data-text="'Deleted ' + $deleted + ' times'" {}
                (dialog(&DialogProps { id: s("confirm"), purpose: s("form"), signal: s("confirmOpen"), ..Default::default() }, html! {
                    (dialog_header(&DialogHeaderProps { title: s("Delete project?"), has_close_button: true, ..Default::default() }))
                    (text(&TextProps { as_: s("p"), ..Default::default() },
                        html! { "This removes the project and its history. The server confirms over SSE." }))
                    div.demo-row.demo-end {
                        (btn("Cancel", "", vec![attr("data-dialog-close", "")]))
                        (btn("Delete", "destructive", vec![
                            attr("data-on:click", "@post('/demo/delete')"),
                            attr("data-indicator", "deleting"),
                            attr("data-attr:aria-busy", "$deleting && 'true'"),
                        ]))
                    }
                }))
                (dialog(&DialogProps { id: s("settings"), width: s("520px"), ..Default::default() }, html! {
                    (dialog_header(&DialogHeaderProps {
                        title: s("Settings"), subtitle: s("Closes on Escape or a backdrop press"),
                        has_close_button: true, has_divider: true, ..Default::default()
                    }))
                    (text_input(&TextInputProps { label: s("Project name"), id: s("settings-name"), value: s("canon"), ..Default::default() }))
                    (switch(&SwitchProps { label: s("Email notifications"), id: s("settings-notify"), checked: true, ..Default::default() }))
                }))
            }))
            section.demo-section {
                h2 { "TabList" }
                (tab_list(&TabListProps {
                    value: s("overview"), signal: s("tab"), role: s("tablist"), label: s("Project"), has_divider: true,
                    tabs: vec![
                        tab("overview", "Overview", "info", false),
                        tab("activity", "Activity", "clock", false),
                        tab("settings", "Settings", "wrench", false),
                        tab("billing", "Billing", "", true),
                    ],
                    ..Default::default()
                }))
                (panel("overview", true, html! { "Overview panel. The selected tab is the Datastar signal " code { "$tab" } "." }))
                (panel("activity", false, html! { "Activity panel." }))
                (panel("settings", false, html! { "Settings panel." }))
                h3 { "Navigation" }
                (tab_list(&TabListProps {
                    value: s("/overlays"), size: s("sm"), label: s("Demo pages"),
                    tabs: [("/", "Tokens"), ("/components", "Components"), ("/content", "Content"), ("/overlays", "Overlays")]
                        .iter()
                        .map(|(v, l)| TabProps { value: s(v), label: s(l), href: s(v), ..Default::default() })
                        .collect(),
                    ..Default::default()
                }))
            }
            (section("Toast", html! {
                (btn("Toast from the server", "", vec![attr("data-on:click", "@post('/demo/toast?body=Saved on the server')")]))
                (btn("Error toast", "destructive", vec![attr("data-on:click", "@post('/demo/toast?type=error&body=Could not reach the server')")]))
                (btn("Toast in the browser", "ghost", vec![attr("data-on:click", "canon.toast({body: 'Copied to clipboard'})")]))
            }))
        }
    }
}
