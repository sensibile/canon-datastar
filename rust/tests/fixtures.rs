//! Renders each case and compares it with fixtures/<component>/<case>.html, the same golden
//! files the Go package tests against (go/canon/*_test.go uses the same cases).

use canon_datastar::*;
use maud::{Markup, PreEscaped};
use std::path::PathBuf;

/// templ-era fixtures may spell quotes as &#34; / &#39;; compare one spelling.
fn normalize(s: &str) -> String {
    s.replace("&#34;", "&quot;").replace("&#39;", "'")
}

fn check(component: &str, cases: Vec<(&str, Markup)>) {
    let mut failures = Vec::new();
    for (case, got) in cases {
        let path: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "..",
            "fixtures",
            component,
            &format!("{case}.html"),
        ]
        .iter()
        .collect();
        let want =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let got = normalize(&got.into_string());
        if got != want.trim_end_matches('\n') {
            failures.push(format!("{}\n got: {got}\nwant: {want}", path.display()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

fn child() -> Markup {
    PreEscaped("<p>Hello</p>".into())
}

fn s(v: &str) -> String {
    v.to_string()
}

#[test]
fn theme_fixtures() {
    check(
        "theme",
        vec![
            ("default", theme(&ThemeProps::default(), child())),
            (
                "dark",
                theme(
                    &ThemeProps {
                        name: Some(s("matcha")),
                        mode: ThemeMode::Dark,
                        ..Default::default()
                    },
                    child(),
                ),
            ),
            (
                "attrs",
                theme(
                    &ThemeProps {
                        name: Some(s("y2k")),
                        mode: ThemeMode::Light,
                        attrs: vec![
                            attr("data-attr:data-astryx-theme", "$theme"),
                            attr("title", r#"a "quoted" & <tagged> value"#),
                        ],
                    },
                    child(),
                ),
            ),
        ],
    );
}

#[test]
fn icon_fixtures() {
    check(
        "icon",
        vec![
            ("outline", icon(&IconProps::new("check"))),
            (
                "solid-labelled",
                icon(&IconProps {
                    size: s("lg"),
                    color: s("success"),
                    label: s("Done"),
                    ..IconProps::new("success")
                }),
            ),
            ("unknown", icon(&IconProps::new("nope"))),
        ],
    );
}

#[test]
fn text_fixtures() {
    check(
        "text",
        vec![
            ("default", text(&TextProps::default(), child())),
            (
                "supporting",
                text(
                    &TextProps {
                        kind: s("supporting"),
                        weight: s("bold"),
                        justify: s("center"),
                        as_: s("p"),
                        ..Default::default()
                    },
                    child(),
                ),
            ),
            (
                "clamp",
                text(
                    &TextProps {
                        max_lines: 3,
                        has_tabular_numbers: true,
                        class: s("x"),
                        ..Default::default()
                    },
                    child(),
                ),
            ),
            (
                "single",
                text(
                    &TextProps {
                        max_lines: 1,
                        has_capsize: true,
                        has_strikethrough: true,
                        text_wrap: s("balance"),
                        ..Default::default()
                    },
                    child(),
                ),
            ),
        ],
    );
}

#[test]
fn heading_fixtures() {
    check(
        "heading",
        vec![
            (
                "default",
                heading(
                    &HeadingProps {
                        level: 1,
                        ..Default::default()
                    },
                    child(),
                ),
            ),
            (
                "display",
                heading(
                    &HeadingProps {
                        level: 9,
                        kind: s("display-2"),
                        accessibility_level: 1,
                        color: s("secondary"),
                        max_lines: 2,
                        ..Default::default()
                    },
                    child(),
                ),
            ),
        ],
    );
}

#[test]
fn button_fixtures() {
    let b = |p: ButtonProps| button(&p);
    check(
        "button",
        vec![
            (
                "default",
                b(ButtonProps {
                    label: s("Save"),
                    ..Default::default()
                }),
            ),
            (
                "primary-icon",
                b(ButtonProps {
                    label: s("Add"),
                    variant: s("primary"),
                    size: s("lg"),
                    icon: s("check"),
                    elevation: s("low"),
                    end_content: Some(badge(&BadgeProps {
                        label: s("3"),
                        ..Default::default()
                    })),
                    attrs: vec![attr("data-on:click", "@post('/add')")],
                    ..Default::default()
                }),
            ),
            (
                "loading",
                b(ButtonProps {
                    label: s("Saving"),
                    is_loading: true,
                    width: s("100%"),
                    ..Default::default()
                }),
            ),
            (
                "interruptible",
                b(ButtonProps {
                    label: s("Run"),
                    is_loading: true,
                    is_interruptible: true,
                    ..Default::default()
                }),
            ),
            (
                "disabled",
                b(ButtonProps {
                    label: s("Nope"),
                    is_disabled: true,
                    href: s("/x"),
                    kind: s("submit"),
                    ..Default::default()
                }),
            ),
            (
                "link",
                b(ButtonProps {
                    label: s("Docs"),
                    href: s("/docs"),
                    target: s("_blank"),
                    rel: s("noopener"),
                    ..Default::default()
                }),
            ),
            (
                "icon-only",
                icon_button(&ButtonProps {
                    label: s("Close"),
                    icon: s("close"),
                    variant: s("ghost"),
                    size: s("sm"),
                    ..Default::default()
                }),
            ),
            (
                "content",
                b(ButtonProps {
                    label: s("Bold"),
                    content: Some(PreEscaped(s("<b>Bold</b>"))),
                    ..Default::default()
                }),
            ),
        ],
    );
}

#[test]
fn link_fixtures() {
    check(
        "link",
        vec![
            (
                "default",
                link(
                    &LinkProps {
                        href: s("/a"),
                        ..Default::default()
                    },
                    child(),
                ),
            ),
            (
                "external",
                link(
                    &LinkProps {
                        href: s("https://x.test"),
                        is_external_link: true,
                        rel: s("me noopener"),
                        has_underline: true,
                        color: s("primary"),
                        ..Default::default()
                    },
                    child(),
                ),
            ),
            (
                "button",
                link(
                    &LinkProps {
                        label: s("Do it"),
                        attrs: vec![attr("data-on:click", "$n++")],
                        ..Default::default()
                    },
                    child(),
                ),
            ),
            (
                "disabled",
                link(
                    &LinkProps {
                        href: s("/a"),
                        is_disabled: true,
                        is_standalone: true,
                        max_lines: 1,
                        attrs: vec![attr("data-on:click", "@post('/x')"), attr("title", "t")],
                        ..Default::default()
                    },
                    child(),
                ),
            ),
        ],
    );
}

#[test]
fn badge_fixtures() {
    check(
        "badge",
        vec![
            (
                "default",
                badge(&BadgeProps {
                    label: s("New"),
                    ..Default::default()
                }),
            ),
            (
                "icon",
                badge(&BadgeProps {
                    label: s(r#""A" & B"#),
                    variant: s("success"),
                    icon: s("check"),
                    ..Default::default()
                }),
            ),
        ],
    );
}

#[test]
fn spinner_fixtures() {
    check(
        "spinner",
        vec![
            ("default", spinner(&SpinnerProps::default())),
            (
                "labelled",
                spinner(&SpinnerProps {
                    size: s("xl"),
                    shade: s("subtle"),
                    label: s("Loading files"),
                    ..Default::default()
                }),
            ),
            (
                "bad-size",
                spinner(&SpinnerProps {
                    size: s("huge"),
                    aria_label: s("Busy"),
                    ..Default::default()
                }),
            ),
        ],
    );
}

#[test]
fn layout_fixtures() {
    check(
        "layout",
        vec![
            (
                "stack",
                stack(
                    &StackProps {
                        gap: s("2"),
                        align: s("center"),
                        padding: s("4"),
                        padding_block_start: s("1.5"),
                        ..Default::default()
                    },
                    child(),
                ),
            ),
            (
                "hstack",
                hstack(
                    &StackProps {
                        gap: s("3"),
                        justify: s("between"),
                        v_align: s("center"),
                        wrap: s("wrap"),
                        width: s("100%"),
                        as_: s("section"),
                        is_scrollable: true,
                        ..Default::default()
                    },
                    child(),
                ),
            ),
            ("vstack-empty", vstack(&StackProps::default(), child())),
            ("divider", divider(&DividerProps::default())),
            (
                "divider-label",
                divider(&DividerProps {
                    label: s("or"),
                    variant: s("strong"),
                    is_full_bleed: true,
                    ..Default::default()
                }),
            ),
            (
                "divider-labelledby",
                divider(&DividerProps {
                    label: s("Section"),
                    id: s("sec"),
                    orientation: s("vertical"),
                    ..Default::default()
                }),
            ),
            (
                "visually-hidden",
                visually_hidden(
                    &VisuallyHiddenProps {
                        as_: s("div"),
                        id: s("vh"),
                        ..Default::default()
                    },
                    child(),
                ),
            ),
        ],
    );
}

#[test]
fn field_fixtures() {
    check(
        "field",
        vec![
            (
                "label",
                field_label(&FieldLabelProps {
                    label: s("Name"),
                    input_id: s("name"),
                    is_required: true,
                    label_icon: s("info"),
                    ..Default::default()
                }),
            ),
            (
                "label-description",
                field_label(&FieldLabelProps {
                    label: s("Tags"),
                    input_id: s("tags"),
                    is_group_label: true,
                    is_label_hidden: true,
                    is_optional: true,
                    description: s("Pick some"),
                    description_id: s("tags-desc"),
                    label_id: s("tags-label"),
                    ..Default::default()
                }),
            ),
            (
                "status-detached",
                field_status(&FieldStatus::new("warning", "Careful"), "detached", "w"),
            ),
            (
                "field-attached",
                field(
                    &FieldProps {
                        label: s("Email"),
                        input_id: s("email"),
                        description: s("Work email"),
                        status: Some(FieldStatus::new("error", "Required")),
                        width: s("320px"),
                        ..Default::default()
                    },
                    child(),
                ),
            ),
            (
                "field-detached",
                field(
                    &FieldProps {
                        label: s("Email"),
                        input_id: s("email"),
                        is_disabled: true,
                        status: Some(FieldStatus {
                            message_id: s("m"),
                            ..FieldStatus::new("success", "OK")
                        }),
                        status_variant: s("detached"),
                        ..Default::default()
                    },
                    child(),
                ),
            ),
        ],
    );
}

#[test]
fn text_input_fixtures() {
    check(
        "text-input",
        vec![
            (
                "default",
                text_input(&TextInputProps {
                    label: s("Full name"),
                    ..Default::default()
                }),
            ),
            (
                "full",
                text_input(&TextInputProps {
                    id: s("email"),
                    label: s("Email"),
                    value: s(r#"a"b"#),
                    kind: s("email"),
                    description: s("We never share it"),
                    is_required: true,
                    start_icon: s("search"),
                    status: Some(FieldStatus::new("error", "Invalid")),
                    size: s("lg"),
                    is_loading: true,
                    placeholder: s("you@example.com"),
                    has_auto_focus: true,
                    name: s("email"),
                    has_spell_check: Some(false),
                    class: s("c"),
                    control_class: s("cc"),
                    attrs: vec![attr("data-x", "1")],
                    input_attrs: vec![attr("data-bind:email", "")],
                    ..Default::default()
                }),
            ),
            (
                "disabled-messageless",
                text_input(&TextInputProps {
                    id: s("q"),
                    label: s("Query"),
                    is_disabled: true,
                    name: s("q"),
                    is_label_hidden: true,
                    status: Some(FieldStatus::new("error", "")),
                    ..Default::default()
                }),
            ),
            (
                "korean-label",
                text_input(&TextInputProps {
                    label: s("이름"),
                    is_read_only: true,
                    value: s("Tony"),
                    status_variant: s("detached"),
                    status: Some(FieldStatus::new("warning", "Check")),
                    ..Default::default()
                }),
            ),
        ],
    );
}

#[test]
fn checkbox_fixtures() {
    check(
        "checkbox",
        vec![
            (
                "default",
                checkbox_input(&CheckboxInputProps {
                    label: s("Accept terms"),
                    ..Default::default()
                }),
            ),
            (
                "full",
                checkbox_input(&CheckboxInputProps {
                    id: s("agree"),
                    label: s("Agree"),
                    checked: true,
                    description: s("Required to continue"),
                    is_required: true,
                    name: s("agree"),
                    value: s("yes"),
                    size: s("sm"),
                    width: s("200px"),
                    label_icon: s("info"),
                    status: Some(FieldStatus::new("error", "Please agree")),
                    input_attrs: vec![attr("data-bind:agree", "")],
                    ..Default::default()
                }),
            ),
            (
                "indeterminate",
                checkbox_input(&CheckboxInputProps {
                    id: s("all"),
                    label: s("All"),
                    is_indeterminate: true,
                    is_read_only: true,
                    is_label_hidden: true,
                    ..Default::default()
                }),
            ),
            (
                "loading-disabled",
                checkbox_input(&CheckboxInputProps {
                    id: s("l"),
                    label: s("L"),
                    is_loading: true,
                    is_disabled: true,
                    name: s("l"),
                    is_optional: true,
                    ..Default::default()
                }),
            ),
        ],
    );
}

#[test]
fn switch_fixtures() {
    check(
        "switch",
        vec![
            (
                "default",
                switch(&SwitchProps {
                    label: s("Notifications"),
                    ..Default::default()
                }),
            ),
            (
                "full",
                switch(&SwitchProps {
                    id: s("wifi"),
                    label: s("Wi-Fi"),
                    checked: true,
                    description: s("Connect automatically"),
                    name: s("wifi"),
                    size: s("sm"),
                    label_position: s("start"),
                    label_spacing: s("spread"),
                    width: s("100%"),
                    is_required: true,
                    status: Some(FieldStatus::new("success", "Connected")),
                    input_attrs: vec![attr("data-bind:wifi", "")],
                    ..Default::default()
                }),
            ),
            (
                "loading-disabled",
                switch(&SwitchProps {
                    id: s("s"),
                    label: s("S"),
                    is_loading: true,
                    is_disabled: true,
                    is_label_hidden: true,
                    ..Default::default()
                }),
            ),
        ],
    );
}

fn raw(v: &str) -> Markup {
    PreEscaped(v.into())
}

#[test]
fn tooltip_fixtures() {
    check(
        "tooltip",
        vec![
            (
                "default",
                tooltip(
                    &TooltipProps {
                        content: s("Copy link"),
                        ..Default::default()
                    },
                    child(),
                ),
            ),
            (
                "full",
                tooltip(
                    &TooltipProps {
                        id: s("tip"),
                        content_markup: Some(raw("<b>Rich</b>")),
                        placement: s("end"),
                        alignment: s("start"),
                        delay: 500,
                        hide_delay: 100,
                        focus_trigger: s("always"),
                        hover_indication: s("always"),
                        class: s("x"),
                        attrs: vec![attr("data-x", "1")],
                        ..Default::default()
                    },
                    child(),
                ),
            ),
            (
                "no-indication",
                tooltip(
                    &TooltipProps {
                        content: s("Plain"),
                        focus_trigger: s("never"),
                        hover_indication: s("never"),
                        ..Default::default()
                    },
                    raw("Text"),
                ),
            ),
            (
                "disabled",
                tooltip(
                    &TooltipProps {
                        content: s("Hidden"),
                        is_disabled: true,
                        ..Default::default()
                    },
                    child(),
                ),
            ),
            (
                "info-tip",
                info_tip(&InfoTipProps {
                    content: s("Billed monthly"),
                    size: s("md"),
                    placement: s("below"),
                    ..Default::default()
                }),
            ),
        ],
    );
}

#[test]
fn popover_fixtures() {
    let trigger = || {
        button(&ButtonProps {
            label: s("Filters"),
            ..Default::default()
        })
    };
    check(
        "popover",
        vec![
            (
                "default",
                popover(
                    &PopoverProps {
                        id: s("filters"),
                        content: Some(child()),
                        ..Default::default()
                    },
                    trigger(),
                ),
            ),
            (
                "full",
                popover(
                    &PopoverProps {
                        id: s("share"),
                        content: Some(child()),
                        placement: s("above"),
                        alignment: s("end"),
                        width: s("320px"),
                        padding: s("4"),
                        label: s("Share"),
                        is_modal: true,
                        has_close_button: true,
                        close_button_label: s("Done"),
                        no_light_dismiss: true,
                        signal: s("shareOpen"),
                        class: s("x"),
                        attrs: vec![attr("data-x", "1")],
                        ..Default::default()
                    },
                    trigger(),
                ),
            ),
            (
                "no-role",
                popover(
                    &PopoverProps {
                        content: Some(child()),
                        role: s("none"),
                        has_auto_focus: true,
                        no_escape_dismiss: true,
                        ..Default::default()
                    },
                    trigger(),
                ),
            ),
        ],
    );
}

#[test]
fn dialog_fixtures() {
    check(
        "dialog",
        vec![
            (
                "default",
                dialog(
                    &DialogProps {
                        id: s("confirm"),
                        ..Default::default()
                    },
                    child(),
                ),
            ),
            (
                "full",
                dialog(
                    &DialogProps {
                        id: s("edit"),
                        width: s("600px"),
                        max_height: s("90dvh"),
                        purpose: s("required"),
                        padding: s("6"),
                        label: s("Edit"),
                        signal: s("editOpen"),
                        class: s("x"),
                        attrs: vec![attr("data-x", "1")],
                        ..Default::default()
                    },
                    child(),
                ),
            ),
            (
                "fullscreen",
                dialog(
                    &DialogProps {
                        variant: s("fullscreen"),
                        purpose: s("form"),
                        ..Default::default()
                    },
                    child(),
                ),
            ),
            (
                "header",
                dialog_header(&DialogHeaderProps {
                    title: s("Delete file?"),
                    ..Default::default()
                }),
            ),
            (
                "header-full",
                dialog_header(&DialogHeaderProps {
                    title: s("Settings"),
                    subtitle: s("Workspace"),
                    title_id: s("settings-title"),
                    has_close_button: true,
                    close_label: s("Close settings"),
                    start_content: Some(raw("<i>s</i>")),
                    end_content: Some(raw("<i>e</i>")),
                    has_divider: true,
                    class: s("x"),
                    attrs: vec![attr("data-x", "1")],
                    ..Default::default()
                }),
            ),
        ],
    );
}

#[test]
fn dropdown_menu_fixtures() {
    let items = || {
        maud::html! {
            (dropdown_menu_item(&DropdownMenuItemProps {
                label: s("Edit"),
                icon: s("wrench"),
                attrs: vec![attr("data-on:click", "@post('/edit')")],
                ..Default::default()
            }))
            (dropdown_menu_divider())
            (dropdown_menu_group(
                &DropdownMenuGroupProps { title: s("More actions"), ..Default::default() },
                dropdown_menu_item(&DropdownMenuItemProps {
                    label: s("Docs"),
                    description: s("Opens in a new tab"),
                    href: s("/docs"),
                    target: s("_blank"),
                    ..Default::default()
                }),
            ))
        }
    };
    check(
        "dropdown-menu",
        vec![
            (
                "default",
                dropdown_menu(
                    &DropdownMenuProps {
                        button: ButtonProps {
                            label: s("Actions"),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    items(),
                ),
            ),
            (
                "full",
                dropdown_menu(
                    &DropdownMenuProps {
                        id: s("row"),
                        button: ButtonProps {
                            label: s("Row"),
                            icon: s("moreHorizontal"),
                            is_icon_only: true,
                            attrs: vec![attr("data-x", "1")],
                            ..Default::default()
                        },
                        no_chevron: true,
                        placement: s("above"),
                        alignment: s("end"),
                        menu_width: s("240px"),
                        menu_max_height: s("300px"),
                        header: Some(raw("<i>h</i>")),
                        footer: Some(raw("<i>f</i>")),
                        class: s("x"),
                    },
                    items(),
                ),
            ),
            (
                "item-full",
                dropdown_menu_item(&DropdownMenuItemProps {
                    label: s("Delete"),
                    variant: s("destructive"),
                    icon: s("close"),
                    keep_open: true,
                    end_content: Some(raw("<kbd>D</kbd>")),
                    class: s("x"),
                    ..Default::default()
                }),
            ),
            (
                "item-disabled",
                dropdown_menu_item(&DropdownMenuItemProps {
                    label: s("Archive"),
                    href: s("/a"),
                    is_disabled: true,
                    attrs: vec![attr("data-on:click", "@post('/a')"), attr("title", "t")],
                    ..Default::default()
                }),
            ),
        ],
    );
}

#[test]
fn tab_list_fixtures() {
    let nav_tabs = vec![
        TabProps {
            value: s("home"),
            label: s("Home"),
            href: s("/"),
            icon: s("menu"),
            ..Default::default()
        },
        TabProps {
            value: s("docs"),
            label: s("Docs"),
            href: s("/docs"),
            end_content: Some(raw("<i>3</i>")),
            ..Default::default()
        },
        TabProps {
            value: s("off"),
            label: s("Off"),
            is_disabled: true,
            attrs: vec![attr("data-on:click", "x()")],
            ..Default::default()
        },
    ];
    check(
        "tab-list",
        vec![
            (
                "nav-none",
                tab_list(&TabListProps {
                    value: s("none"),
                    tabs: vec![nav_tabs[2].clone(), nav_tabs[0].clone()],
                    ..Default::default()
                }),
            ),
            (
                "nav-disabled",
                tab_list(&TabListProps {
                    value: s("off"),
                    signal: s("nav"),
                    tabs: vec![nav_tabs[2].clone(), nav_tabs[0].clone()],
                    ..Default::default()
                }),
            ),
            (
                "nav",
                tab_list(&TabListProps {
                    value: s("home"),
                    tabs: nav_tabs,
                    ..Default::default()
                }),
            ),
            (
                "tablist",
                tab_list(&TabListProps {
                    value: s("b"),
                    signal: s("tab"),
                    role: s("tablist"),
                    size: s("lg"),
                    layout: s("fill"),
                    has_divider: true,
                    label: s("Sections"),
                    class: s("x"),
                    attrs: vec![attr("data-x", "1")],
                    tabs: vec![
                        TabProps {
                            value: s("a"),
                            label: s("A"),
                            id: s("tab-a"),
                            panel_id: s("panel-a"),
                            icon: s("clock"),
                            selected_icon: s("check"),
                            ..Default::default()
                        },
                        TabProps {
                            value: s("b"),
                            label: s("It's B"),
                            id: s("tab-b"),
                            panel_id: s("panel-b"),
                            is_label_hidden: true,
                            icon: s("search"),
                            attrs: vec![attr("data-on:click", "@get('/b')"), attr("title", "t")],
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }),
            ),
            (
                "nav-signal",
                tab_list(&TabListProps {
                    value: s("x"),
                    signal: s("view"),
                    labelled_by: s("h"),
                    tabs: vec![TabProps {
                        value: s("x"),
                        label: s("X"),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
            ),
            (
                "panel",
                tab_panel(
                    &TabPanelProps {
                        id: s("panel-a"),
                        tab_id: s("tab-a"),
                        value: s("a"),
                        signal: s("tab"),
                        class: s("x"),
                        ..Default::default()
                    },
                    child(),
                ),
            ),
            (
                "panel-selected",
                tab_panel(
                    &TabPanelProps {
                        value: s("b"),
                        is_selected: true,
                        ..Default::default()
                    },
                    child(),
                ),
            ),
        ],
    );
}

#[test]
fn toast_fixtures() {
    check(
        "toast",
        vec![
            (
                "default",
                toast(&ToastProps {
                    body: s("Saved"),
                    ..Default::default()
                }),
            ),
            (
                "error",
                toast(&ToastProps {
                    id: s("t1"),
                    kind: s("error"),
                    body: s("Failed"),
                    end_content: Some(raw("<i>retry</i>")),
                    dismiss_label: s("Close"),
                    class: s("x"),
                    attrs: vec![attr("data-x", "1")],
                    ..Default::default()
                }),
            ),
            (
                "error-timed",
                toast(&ToastProps {
                    kind: s("error"),
                    body_markup: Some(raw("<b>Oops</b>")),
                    auto_hide_duration: 8000,
                    ..Default::default()
                }),
            ),
            (
                "sticky",
                toast(&ToastProps {
                    body: s("Pinned"),
                    is_sticky: true,
                    ..Default::default()
                }),
            ),
            (
                "viewport",
                toast_viewport(
                    &ToastViewportProps::default(),
                    toast(&ToastProps {
                        body: s("Hi"),
                        ..Default::default()
                    }),
                ),
            ),
            (
                "viewport-top",
                toast_viewport(
                    &ToastViewportProps {
                        id: s("toasts"),
                        position: s("topStart"),
                        max_visible: 3,
                        label: s("Alerts"),
                        dismiss_label: s("Close"),
                    },
                    PreEscaped(String::new()),
                ),
            ),
        ],
    );
}

fn ss(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| s(x)).collect()
}

#[test]
fn container_fixtures() {
    check(
        "card",
        vec![
            ("default", card(&CardProps::default(), child())),
            (
                "sized",
                card(
                    &CardProps {
                        variant: s("blue"),
                        elevation: s("med"),
                        width: s("320px"),
                        height: s("200px"),
                        padding: s("6"),
                        class: s("x"),
                        ..Default::default()
                    },
                    child(),
                ),
            ),
        ],
    );
    check(
        "section",
        vec![
            ("default", section(&SectionProps::default(), child())),
            (
                "full",
                section(
                    &SectionProps {
                        variant: s("muted"),
                        dividers: ss(&["top", "end"]),
                        max_width: s("40rem"),
                        padding: s("2"),
                        padding_inline: s("4"),
                        padding_block_end: s("0.5"),
                        attrs: vec![attr("data-x", "1")],
                        ..Default::default()
                    },
                    child(),
                ),
            ),
        ],
    );
    check(
        "center",
        vec![
            ("default", center(&CenterProps::default(), child())),
            (
                "inline",
                center(
                    &CenterProps {
                        axis: s("vertical"),
                        is_inline: true,
                        min_height: s("10rem"),
                        padding: s("2"),
                        padding_block_start: s("8"),
                        ..Default::default()
                    },
                    child(),
                ),
            ),
        ],
    );
    check(
        "grid",
        vec![
            ("default", grid(&GridProps::default(), child())),
            (
                "columns",
                grid(
                    &GridProps {
                        columns: 3,
                        gap: s("2"),
                        row_height: 80,
                        align: s("center"),
                        width: s("100%"),
                        ..Default::default()
                    },
                    child(),
                ),
            ),
            (
                "auto",
                grid(
                    &GridProps {
                        auto: Some(GridAuto {
                            min_width: 200,
                            max: 4,
                            repeat: s("fit"),
                        }),
                        column_gap: s("1.5"),
                        justify: s("end"),
                        ..Default::default()
                    },
                    child(),
                ),
            ),
            (
                "auto-plain",
                grid(
                    &GridProps {
                        auto: Some(GridAuto {
                            min_width: 160,
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                    child(),
                ),
            ),
            (
                "span",
                grid_span(
                    &GridSpanProps {
                        columns: 2,
                        rows: 3,
                        ..Default::default()
                    },
                    child(),
                ),
            ),
            (
                "span-full",
                grid_span(
                    &GridSpanProps {
                        is_full: true,
                        ..Default::default()
                    },
                    child(),
                ),
            ),
        ],
    );
    check(
        "aspect-ratio",
        vec![
            (
                "default",
                aspect_ratio(
                    &AspectRatioProps {
                        ratio: 16.0 / 9.0,
                        ..Default::default()
                    },
                    child(),
                ),
            ),
            (
                "ellipse",
                aspect_ratio(
                    &AspectRatioProps {
                        ratio: 1.0,
                        shape: s("ellipse"),
                        fit: s("cover"),
                        class: s("x"),
                        ..Default::default()
                    },
                    child(),
                ),
            ),
        ],
    );
}

#[test]
fn content_fixtures() {
    check(
        "blockquote",
        vec![
            ("default", blockquote(&BlockquoteProps::default(), child())),
            (
                "cite",
                blockquote(
                    &BlockquoteProps {
                        cite: s("Ada <Lovelace>"),
                        ..Default::default()
                    },
                    child(),
                ),
            ),
        ],
    );
    check(
        "code",
        vec![
            ("default", code(&CodeProps::default(), child())),
            (
                "inherit",
                code(
                    &CodeProps {
                        color: s("secondary"),
                        size: s("inherit"),
                        ..Default::default()
                    },
                    child(),
                ),
            ),
        ],
    );
    check(
        "kbd",
        vec![
            (
                "mod",
                kbd(&KbdProps {
                    keys: s("mod+shift+P"),
                    ..Default::default()
                }),
            ),
            (
                "special",
                kbd(&KbdProps {
                    keys: s("Esc + return+up+plus+f1"),
                    class: s("x"),
                    ..Default::default()
                }),
            ),
        ],
    );
    check(
        "empty-state",
        vec![
            (
                "default",
                empty_state(&EmptyStateProps {
                    title: s("No results"),
                    ..Default::default()
                }),
            ),
            (
                "full",
                empty_state(&EmptyStateProps {
                    title: s("Inbox zero"),
                    description: s("Nothing new."),
                    icon: Some(child()),
                    actions: Some(child()),
                    heading_level: 2,
                    is_compact: true,
                    ..Default::default()
                }),
            ),
        ],
    );
    check(
        "skeleton",
        vec![
            ("default", skeleton(&SkeletonProps::default())),
            (
                "sized",
                skeleton(&SkeletonProps {
                    width: s("120px"),
                    height: s("1em"),
                    radius: s("rounded"),
                    index: 3,
                    ..Default::default()
                }),
            ),
        ],
    );
    check(
        "status-dot",
        vec![
            (
                "default",
                status_dot(&StatusDotProps {
                    label: s("Idle"),
                    ..Default::default()
                }),
            ),
            (
                "tooltip",
                status_dot(&StatusDotProps {
                    variant: s("success"),
                    label: s("Online"),
                    is_pulsing: true,
                    tooltip: s("Online now"),
                    icon: s("check"),
                    ..Default::default()
                }),
            ),
        ],
    );
}

#[test]
fn progress_bar_fixtures() {
    let mark = |value: f64, label: &str| ProgressBarMark {
        value,
        label: s(label),
    };
    check(
        "progress-bar",
        vec![
            (
                "default",
                progress_bar(&ProgressBarProps {
                    label: s("Upload"),
                    value: 40.0,
                    ..Default::default()
                }),
            ),
            (
                "value",
                progress_bar(&ProgressBarProps {
                    label: s("Disk"),
                    value: 2.0,
                    max: 3.0,
                    has_value_label: true,
                    variant: s("warning"),
                    marks: vec![mark(1.0, "Quota"), mark(2.5, "Limit")],
                    ..Default::default()
                }),
            ),
            (
                "hidden",
                progress_bar(&ProgressBarProps {
                    id: s("p"),
                    label: s("Sync"),
                    is_label_hidden: true,
                    is_indeterminate: true,
                    is_disabled: true,
                    ..Default::default()
                }),
            ),
            (
                "custom",
                progress_bar(&ProgressBarProps {
                    label: s("Steps"),
                    value: 150.0,
                    has_value_label: true,
                    is_label_hidden: true,
                    value_label: s("3 of 3"),
                    ..Default::default()
                }),
            ),
        ],
    );
}

#[test]
fn token_fixtures() {
    let click = || Some(vec![attr("data-on:click", "$f = 1")]);
    check(
        "token",
        vec![
            (
                "default",
                token(&TokenProps {
                    label: s("Draft"),
                    ..Default::default()
                }),
            ),
            (
                "link",
                token(&TokenProps {
                    label: s("Docs"),
                    href: s("/docs"),
                    color: s("blue"),
                    size: s("sm"),
                    description: s("Opens docs"),
                    ..Default::default()
                }),
            ),
            (
                "link-disabled",
                token(&TokenProps {
                    label: s("Docs"),
                    href: s("/docs"),
                    is_disabled: true,
                    ..Default::default()
                }),
            ),
            (
                "button",
                token(&TokenProps {
                    label: s("Filter"),
                    action: click(),
                    end_content: Some(child()),
                    ..Default::default()
                }),
            ),
            (
                "removable",
                token(&TokenProps {
                    label: s("Tag"),
                    href: s("/t"),
                    remove_attrs: vec![attr("data-on:click", "@delete('/t')")],
                    icon: Some(child()),
                    ..Default::default()
                }),
            ),
            (
                "removable-plain",
                token(&TokenProps {
                    label: s("Tag"),
                    is_removable: true,
                    is_label_hidden: true,
                    remove_name: s("tag one"),
                    ..Default::default()
                }),
            ),
            (
                "disabled",
                token(&TokenProps {
                    label: s("Tag"),
                    action: click(),
                    is_disabled: true,
                    remove_attrs: vec![attr("data-on:click", "@delete('/t')"), attr("title", "x")],
                    ..Default::default()
                }),
            ),
        ],
    );
}

#[test]
fn avatar_fixtures() {
    let status = |variant: &str, label: &str, icon: &str| {
        Some(AvatarStatusDotProps {
            variant: s(variant),
            label: s(label),
            icon: s(icon),
            ..Default::default()
        })
    };
    let named = |name: &str| AvatarProps {
        name: s(name),
        ..Default::default()
    };
    check(
        "avatar",
        vec![
            ("initials", avatar(&named("Ada  Byron Lovelace"))),
            (
                "icon",
                avatar(&AvatarProps {
                    size: s("lg"),
                    shape: s("rounded"),
                    has_no_tooltip: true,
                    ..Default::default()
                }),
            ),
            (
                "image",
                avatar(&AvatarProps {
                    src: s("/a.png"),
                    fallback_src: s("/b.png"),
                    alt: s("Ada L."),
                    size: s("xl"),
                    tooltip_id: s("t"),
                    status: status("", "Online", ""),
                    ..named("Ada")
                }),
            ),
            (
                "fallback-src",
                avatar(&AvatarProps {
                    fallback_src: s("/b.png"),
                    pixels: 60,
                    tooltip: s("Custom"),
                    status: status("neutral", "Away", ""),
                    ..named("\"Q\"")
                }),
            ),
            (
                "link",
                avatar(&AvatarProps {
                    href: s("/u/ada"),
                    shape: s("square"),
                    has_no_tooltip: true,
                    status: status("error", "Busy", "close"),
                    ..named("Ada")
                }),
            ),
            (
                "button",
                avatar(&AvatarProps {
                    alt: s("Me"),
                    action: Some(vec![attr("data-on:click", "$m = 1")]),
                    size: s("sm"),
                    status: status("error", "", ""),
                    ..Default::default()
                }),
            ),
            (
                "group",
                avatar_group(&AvatarGroupProps {
                    id: s("team"),
                    size: s("sm"),
                    shape: s("rounded"),
                    avatars: vec![
                        AvatarProps {
                            tooltip_id: s("t1"),
                            ..named("Ada")
                        },
                        AvatarProps {
                            href: s("/bo"),
                            has_no_tooltip: true,
                            ..named("Bo")
                        },
                        AvatarProps {
                            action: Some(vec![]),
                            has_no_tooltip: true,
                            ..named("Cy")
                        },
                    ],
                    overflow: Some(AvatarGroupOverflowProps {
                        count: 1234,
                        action: Some(vec![attr("data-on:click", "$all = true")]),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            ),
            (
                "group-static",
                avatar_group(&AvatarGroupProps {
                    avatars: vec![AvatarProps {
                        has_no_tooltip: true,
                        ..named("Ada")
                    }],
                    overflow: Some(AvatarGroupOverflowProps {
                        count: 3,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            ),
        ],
    );
}

#[test]
fn banner_fixtures() {
    check(
        "banner",
        vec![
            (
                "default",
                banner(&BannerProps {
                    title: s("Saved"),
                    ..Default::default()
                }),
            ),
            (
                "full",
                banner(&BannerProps {
                    status: s("warning"),
                    title: s("Quota"),
                    description: s("Almost full."),
                    is_dismissable: true,
                    dismiss_attrs: vec![attr("data-on:click", "@post('/dismiss')")],
                    end_content: Some(child()),
                    elevation: s("low"),
                    content: Some(child()),
                    signal: s("more"),
                    role: s("alert"),
                    attrs: vec![attr("aria-live", "polite"), attr("data-x", "1")],
                    ..Default::default()
                }),
            ),
            (
                "open",
                banner(&BannerProps {
                    id: s("b"),
                    status: s("error"),
                    title: s("Failed"),
                    content: Some(child()),
                    is_open: true,
                    container: s("section"),
                    ..Default::default()
                }),
            ),
            (
                "static",
                banner(&BannerProps {
                    status: s("success"),
                    title: s("Done"),
                    content: Some(child()),
                    is_not_collapsible: true,
                    icon: Some(child()),
                    is_dismissable: true,
                    dismiss_label: s("Close"),
                    role: s("status"),
                    ..Default::default()
                }),
            ),
        ],
    );
}

#[test]
fn collapsible_fixtures() {
    let item = |trigger: &str, value: &str| CollapsibleProps {
        trigger: s(trigger),
        value: s(value),
        content: Some(child()),
        ..Default::default()
    };
    check(
        "collapsible",
        vec![
            (
                "default",
                collapsible(
                    &CollapsibleProps {
                        trigger: s("Details"),
                        ..Default::default()
                    },
                    child(),
                ),
            ),
            (
                "closed",
                collapsible(
                    &CollapsibleProps {
                        id: s("c"),
                        trigger: s("More"),
                        is_closed: true,
                        chevron_position: s("start"),
                        signal: s("more"),
                        class: s("x"),
                        ..Default::default()
                    },
                    child(),
                ),
            ),
            (
                "disabled",
                collapsible(
                    &CollapsibleProps {
                        trigger: s("Nope"),
                        is_disabled: true,
                        ..Default::default()
                    },
                    child(),
                ),
            ),
            (
                "group",
                collapsible_group(&CollapsibleGroupProps {
                    value: ss(&["b", "a"]),
                    has_dividers: true,
                    items: vec![item("A", "a"), item("B", "b"), item("C", "")],
                    ..Default::default()
                }),
            ),
            (
                "group-multiple",
                collapsible_group(&CollapsibleGroupProps {
                    kind: s("multiple"),
                    value: ss(&["a", "b"]),
                    density: s("compact"),
                    chevron_position: s("start"),
                    items: vec![
                        item("A", "a"),
                        CollapsibleProps {
                            chevron_position: s("end"),
                            ..item("B", "b")
                        },
                    ],
                    ..Default::default()
                }),
            ),
        ],
    );
}

#[test]
fn metadata_list_fixtures() {
    let items = vec![
        MetadataListItem {
            label: s("Owner"),
            text: s("Ada"),
            ..Default::default()
        },
        MetadataListItem {
            label: s("Status"),
            value: Some(child()),
            icon: Some(child()),
            ..Default::default()
        },
        MetadataListItem {
            label: s("Size"),
            text: s("3 MB"),
            class: s("x"),
            ..Default::default()
        },
    ];
    let list = |p: MetadataListProps| metadata_list(&p);
    check(
        "metadata-list",
        vec![
            (
                "default",
                list(MetadataListProps {
                    items: items.clone(),
                    ..Default::default()
                }),
            ),
            (
                "max",
                list(MetadataListProps {
                    id: s("m"),
                    title: s("File"),
                    max_num_of_items: 2,
                    label_width: s("8rem"),
                    items: items.clone(),
                    ..Default::default()
                }),
            ),
            (
                "multi",
                list(MetadataListProps {
                    columns: s("multi"),
                    max_num_of_items: 1,
                    items: items.clone(),
                    ..Default::default()
                }),
            ),
            (
                "columns",
                list(MetadataListProps {
                    columns: s("3"),
                    label_position: s("start"),
                    items: items.clone(),
                    ..Default::default()
                }),
            ),
            (
                "columns-top",
                list(MetadataListProps {
                    columns: s("2"),
                    items: items[..1].to_vec(),
                    ..Default::default()
                }),
            ),
            (
                "horizontal",
                list(MetadataListProps {
                    orientation: s("horizontal"),
                    max_num_of_items: 1,
                    items: items[..2].to_vec(),
                    ..Default::default()
                }),
            ),
        ],
    );
}

#[test]
fn thumbnail_fixtures() {
    let open = || Some(vec![attr("data-on:click", "$open = 1")]);
    check(
        "thumbnail",
        vec![
            ("default", thumbnail(&ThumbnailProps::default())),
            (
                "image",
                thumbnail(&ThumbnailProps {
                    src: s("/a.png"),
                    alt: s("A cat"),
                    label: s("cat.png"),
                    tooltip_id: s("t"),
                    action: open(),
                    remove_attrs: vec![attr("data-on:click", "@delete('/a')")],
                    ..Default::default()
                }),
            ),
            (
                "loading",
                thumbnail(&ThumbnailProps {
                    is_loading: true,
                    ..Default::default()
                }),
            ),
            (
                "uploading",
                thumbnail(&ThumbnailProps {
                    src: s("/a.png"),
                    is_loading: true,
                    action: open(),
                    is_removable: true,
                    show_remove_on: s("always"),
                    ..Default::default()
                }),
            ),
            (
                "disabled",
                thumbnail(&ThumbnailProps {
                    src: s("/a.png"),
                    alt: s("A"),
                    is_disabled: true,
                    is_removable: true,
                    class: s("x"),
                    ..Default::default()
                }),
            ),
        ],
    );
}

#[test]
fn timestamp_fixtures() {
    // 2026-03-04T17:05:09.120+01:00
    let at = 1_772_640_309_120;
    let entry = |zone: &str, format: &str, label: &str| TimestampTooltipEntry {
        timezone_id: s(zone),
        format: s(format),
        label: s(label),
    };
    check(
        "timestamp",
        vec![
            (
                "default",
                timestamp(&TimestampProps {
                    value_ms: at,
                    ..Default::default()
                }),
            ),
            (
                "absolute",
                timestamp(&TimestampProps {
                    value_ms: at,
                    format: s("date"),
                    timezone_id: s("Asia/Seoul"),
                    is_timezone_shown: true,
                    kind: s("body"),
                    color: s("primary"),
                    ..Default::default()
                }),
            ),
            (
                "entries",
                timestamp(&TimestampProps {
                    id: s("ts"),
                    value_ms: at,
                    format: s("relative_short"),
                    is_live: true,
                    locale: s("ko-KR"),
                    auto_threshold: 60,
                    tooltip_entries: vec![entry("UTC", "date_time", "UTC"), entry("", "", "")],
                    ..Default::default()
                }),
            ),
            (
                "no-tooltip",
                timestamp(&TimestampProps {
                    value_ms: -84_600_000,
                    format: s("auto"),
                    auto_threshold: 60,
                    has_no_tooltip: true,
                    ..Default::default()
                }),
            ),
            (
                "absolute-tip",
                timestamp(&TimestampProps {
                    value_ms: at,
                    format: s("system_date"),
                    tooltip_entries: vec![entry("local", "full", "")],
                    ..Default::default()
                }),
            ),
        ],
    );
}

fn a(n: &str, v: &str) -> Attr {
    attr(n, v)
}

#[test]
fn text_area_fixtures() {
    check(
        "text-area",
        vec![
            (
                "default",
                text_area(&TextAreaProps {
                    label: s("Bio"),
                    ..Default::default()
                }),
            ),
            (
                "full",
                text_area(&TextAreaProps {
                    id: s("bio"),
                    label: s("Bio"),
                    value: s("\nHi \"you\""),
                    description: s("Short"),
                    is_required: true,
                    start_icon: s("info"),
                    status: Some(FieldStatus::new("warning", "Long")),
                    size: s("lg"),
                    is_loading: true,
                    placeholder: s("Tell us"),
                    rows: 5,
                    max_length: 4,
                    html_max_length: 2000,
                    has_auto_focus: true,
                    name: s("bio"),
                    has_spell_check: Some(false),
                    class: s("c"),
                    control_class: s("cc"),
                    attrs: vec![a("data-x", "1")],
                    input_attrs: vec![a("data-bind:bio", "")],
                    ..Default::default()
                }),
            ),
            (
                "disabled-messageless",
                text_area(&TextAreaProps {
                    id: s("n"),
                    label: s("Notes"),
                    is_disabled: true,
                    name: s("n"),
                    status: Some(FieldStatus::new("error", "")),
                    max_length: 1000,
                    value: s("abc"),
                    ..Default::default()
                }),
            ),
        ],
    );
}

#[test]
fn item_fixtures() {
    check(
        "item",
        vec![
            (
                "default",
                item(&ItemProps {
                    label: s("Row"),
                    ..Default::default()
                }),
            ),
            (
                "button",
                item(&ItemProps {
                    label: s("Row"),
                    description: s("More"),
                    action: Some(vec![a("data-on:click", "$n++")]),
                    start_content: Some(child()),
                    end_content: Some(child()),
                    align: s("first-line"),
                    density: s("compact"),
                    is_selected: true,
                    label_lines: 2,
                    description_lines: 1,
                    class: s("x"),
                    attrs: vec![a("data-y", "1")],
                    ..Default::default()
                }),
            ),
            (
                "link-disabled",
                item(&ItemProps {
                    as_: s("li"),
                    label: s("Docs"),
                    href: s("/docs"),
                    target: s("_blank"),
                    is_disabled: true,
                    layout: s("inline"),
                    description: s("inline"),
                    ..Default::default()
                }),
            ),
            (
                "link",
                item(&ItemProps {
                    label: s("Docs"),
                    href: s("/docs"),
                    target: s("_blank"),
                    is_highlighted: true,
                    ..Default::default()
                }),
            ),
            (
                "role",
                item(&ItemProps {
                    label: s("Opt"),
                    role: s("option"),
                    is_selected: true,
                    is_disabled: true,
                    label_content: Some(child()),
                    density: s("spacious"),
                    align: s("start"),
                    ..Default::default()
                }),
            ),
            (
                "disabled-button",
                item(&ItemProps {
                    label: s("Row"),
                    action: Some(vec![a("data-on:click", "$n++"), a("data-x", "1")]),
                    is_disabled: true,
                    ..Default::default()
                }),
            ),
        ],
    );
}

#[test]
fn checkbox_list_fixtures() {
    let cb = |label: &str| CheckboxListItem {
        label: s(label),
        ..Default::default()
    };
    check(
        "checkbox-list",
        vec![
            (
                "default",
                checkbox_list(&CheckboxListProps {
                    label: s("Toppings"),
                    items: vec![cb("Cheese"), cb("Ham")],
                    ..Default::default()
                }),
            ),
            (
                "full",
                checkbox_list(&CheckboxListProps {
                    id: s("t"),
                    label: s("Toppings"),
                    description: s("Pick"),
                    is_required: true,
                    status: Some(FieldStatus::new("error", "Pick one")),
                    name: s("top"),
                    density: s("compact"),
                    has_dividers: true,
                    width: s("300px"),
                    class: s("c"),
                    attrs: vec![a("data-x", "1")],
                    input_attrs: vec![a("data-bind:top", "")],
                    items: vec![
                        CheckboxListItem {
                            value: s("olives"),
                            is_disabled: true,
                            ..cb("Olives")
                        },
                        CheckboxListItem {
                            value: s("cheese"),
                            description: s("Mozzarella"),
                            end_content: Some(child()),
                            align: s("start"),
                            class: s("i"),
                            attrs: vec![a("data-r", "1")],
                            input_attrs: vec![a("data-i", "1")],
                            ..cb("Cheese")
                        },
                        CheckboxListItem {
                            label_content: Some(child()),
                            value: s("rich"),
                            is_indeterminate: true,
                            is_loading: true,
                            id: s("rich"),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }),
            ),
            (
                "read-only-checked",
                checkbox_list(&CheckboxListProps {
                    label: s("Read"),
                    is_read_only: true,
                    is_required: true,
                    is_disabled: true,
                    items: vec![CheckboxListItem {
                        is_checked: true,
                        aria_label: s("Alpha"),
                        ..cb("A")
                    }],
                    ..Default::default()
                }),
            ),
        ],
    );
}

#[test]
fn radio_list_fixtures() {
    let r = |label: &str, value: &str| RadioListItem {
        label: s(label),
        value: s(value),
        ..Default::default()
    };
    check(
        "radio-list",
        vec![
            (
                "default",
                radio_list(&RadioListProps {
                    label: s("Plan"),
                    items: vec![r("Free", "free"), r("Pro", "pro")],
                    ..Default::default()
                }),
            ),
            (
                "full",
                radio_list(&RadioListProps {
                    id: s("p"),
                    label: s("Plan"),
                    description: s("Billing"),
                    value: s("pro"),
                    orientation: s("horizontal"),
                    name: s("plan"),
                    is_required: true,
                    status: Some(FieldStatus::new("error", "Pick")),
                    size: s("sm"),
                    width: s("20rem"),
                    class: s("c"),
                    container_class: s("cc"),
                    attrs: vec![a("data-x", "1")],
                    input_attrs: vec![a("data-bind:plan", "")],
                    items: vec![
                        RadioListItem {
                            description: s("3 projects"),
                            start_content: Some(child()),
                            end_content: Some(child()),
                            class: s("i"),
                            attrs: vec![a("data-r", "1")],
                            input_attrs: vec![a("data-i", "1")],
                            ..r("Free", "free")
                        },
                        RadioListItem {
                            aria_label: s("Pro plan"),
                            id: s("pro"),
                            ..r("Pro", "pro")
                        },
                        RadioListItem {
                            is_disabled: true,
                            ..r("Enterprise", "ent")
                        },
                    ],
                    ..Default::default()
                }),
            ),
            (
                "disabled",
                radio_list(&RadioListProps {
                    label: s("Off"),
                    is_disabled: true,
                    is_required: true,
                    is_optional: true,
                    items: vec![r("A", "a")],
                    ..Default::default()
                }),
            ),
        ],
    );
}

#[test]
fn input_group_fixtures() {
    let price = InputGroupProps {
        id: s("price"),
        label: s("Price"),
        description: s("Per seat"),
        size: s("lg"),
        status: Some(FieldStatus::new("error", "Too low")),
        is_required: true,
        class: s("c"),
        attrs: vec![a("data-x", "1")],
        ..Default::default()
    };
    let off = InputGroupProps {
        label: s("Off"),
        is_disabled: true,
        ..Default::default()
    };
    let members = maud::html! {
        (input_group_text("$"))
        (number_input(&NumberInputProps { id: s("amt"), label: s("Amount"), value: Some(5.0), group: price.member(), units: s("USD"), ..Default::default() }))
        (text_input(&TextInputProps { id: s("note"), label: s("Note"), group: price.member(), status: Some(FieldStatus::new("error", "")), ..Default::default() }))
        (button(&ButtonProps { label: s("Go"), attrs: vec![input_group_item()], ..Default::default() }))
    };
    check(
        "input-group",
        vec![
            (
                "default",
                input_group(
                    &InputGroupProps {
                        label: s("Site"),
                        ..Default::default()
                    },
                    child(),
                ),
            ),
            ("members", input_group(&price, members)),
            (
                "disabled",
                input_group(
                    &off,
                    text_input(&TextInputProps {
                        label: s("Inner"),
                        group: off.member(),
                        ..Default::default()
                    }),
                ),
            ),
        ],
    );
}

#[test]
fn number_input_fixtures() {
    check(
        "number-input",
        vec![
            (
                "default",
                number_input(&NumberInputProps {
                    label: s("Quantity"),
                    ..Default::default()
                }),
            ),
            (
                "full",
                number_input(&NumberInputProps {
                    id: s("qty"),
                    label: s("Quantity"),
                    value: Some(1234.5),
                    description: s("How many"),
                    is_required: true,
                    start_icon: s("search"),
                    label_icon: s("info"),
                    status: Some(FieldStatus::new("warning", "High")),
                    size: s("lg"),
                    placeholder: s("0"),
                    width: s("200px"),
                    has_auto_focus: true,
                    name: s("qty"),
                    min: Some(-5.0),
                    max: Some(1e6),
                    step: 0.5,
                    format: s(r#"{"style":"currency","currency":"USD"}"#),
                    is_wheel_disabled: true,
                    has_number_steppers: true,
                    units: s("pcs"),
                    is_integer_only: true,
                    has_clear: true,
                    locale: s("de-DE"),
                    class: s("c"),
                    control_class: s("cc"),
                    attrs: vec![a("data-x", "1")],
                    input_attrs: vec![a("data-i", "1")],
                    value_attrs: vec![a("data-bind:qty", "")],
                    ..Default::default()
                }),
            ),
            (
                "error-message",
                number_input(&NumberInputProps {
                    label: s("Qty"),
                    status: Some(FieldStatus::new("error", "Bad")),
                    status_variant: s("detached"),
                    has_clear: true,
                    ..Default::default()
                }),
            ),
            (
                "disabled",
                number_input(&NumberInputProps {
                    label: s("Qty"),
                    value: Some(0.0),
                    is_disabled: true,
                    name: s("q"),
                    has_clear: true,
                    has_number_steppers: true,
                    is_read_only: true,
                    ..Default::default()
                }),
            ),
        ],
    );
}

#[test]
fn slider_fixtures() {
    let mark = |v: f64, l: &str| SliderMark {
        value: v,
        label: s(l),
    };
    check(
        "slider",
        vec![
            (
                "default",
                slider(&SliderProps {
                    label: s("Volume"),
                    ..Default::default()
                }),
            ),
            (
                "full",
                slider(&SliderProps {
                    id: s("vol"),
                    label: s("Volume"),
                    values: vec![150.0],
                    description: s("Loud"),
                    is_required: true,
                    status: Some(FieldStatus::new("error", "Too loud")),
                    width: s("300px"),
                    min: Some(-10.0),
                    max: Some(120.0),
                    step: 2.5,
                    format: s(r#"{"style":"percent"}"#),
                    locale: s("ko-KR"),
                    name: s("v"),
                    form: s("f"),
                    marks: vec![mark(-10.0, "Low"), mark(55.0, ""), mark(120.0, "High")],
                    class: s("c"),
                    container_class: s("cc"),
                    attrs: vec![a("data-x", "1")],
                    value_attrs: vec![a("data-bind:vol", "")],
                    ..Default::default()
                }),
            ),
            (
                "range",
                slider(&SliderProps {
                    label: s("Price"),
                    values: vec![20.0, 80.0],
                    min_steps_between_thumbs: 5,
                    value_display: s("text"),
                    orientation: s("vertical"),
                    marks: vec![mark(50.0, "Mid")],
                    is_disabled: true,
                    name: s("p"),
                    attrs: vec![a("data-x", "1")],
                    value_attrs: vec![a("data-bind:lo", "")],
                    end_value_attrs: vec![a("data-bind:hi", "")],
                    ..Default::default()
                }),
            ),
            (
                "none",
                slider(&SliderProps {
                    label: s("Quiet"),
                    values: vec![1.25],
                    min: Some(1.0),
                    max: Some(1.0),
                    value_display: s("none"),
                    ..Default::default()
                }),
            ),
        ],
    );
}

#[test]
fn file_input_fixtures() {
    check(
        "file-input",
        vec![
            (
                "default",
                file_input(&FileInputProps {
                    label: s("Avatar"),
                    ..Default::default()
                }),
            ),
            (
                "full",
                file_input(&FileInputProps {
                    id: s("docs"),
                    label: s("Documents"),
                    accept: s(".pdf,image/*"),
                    is_multiple: true,
                    max_size: 1048576,
                    max_files: 3,
                    is_required: true,
                    description: s("PDFs"),
                    placeholder: s("Pick"),
                    mode: s("dropzone"),
                    width: s("400px"),
                    name: s("docs"),
                    form: s("f"),
                    class: s("c"),
                    control_class: s("cc"),
                    attrs: vec![a("data-x", "1")],
                    input_attrs: vec![a("data-bind:docs", "")],
                    ..Default::default()
                }),
            ),
            (
                "status",
                file_input(&FileInputProps {
                    label: s("Avatar"),
                    status: Some(FieldStatus::new("success", "Uploaded")),
                    is_loading: true,
                    status_variant: s("detached"),
                    ..Default::default()
                }),
            ),
            (
                "disabled",
                file_input(&FileInputProps {
                    label: s("Avatar"),
                    is_disabled: true,
                    name: s("a"),
                    form: s("f"),
                    max_files: 2,
                    ..Default::default()
                }),
            ),
        ],
    );
}

#[test]
fn selector_fixtures() {
    let opts = vec![
        SelectorOption {
            description: s("Crisp"),
            icon: s("check"),
            ..SelectorOption::new("a", "Apple")
        },
        SelectorOption {
            value: s("b"),
            ..Default::default()
        },
        SelectorOption {
            is_disabled: true,
            ..SelectorOption::new("c", "Cherry")
        },
    ];
    let sections = vec![
        selector_section(
            "Fruit",
            vec![
                SelectorOption::new("a", "Apple"),
                SelectorOption::new("b", "Banana"),
            ],
        ),
        selector_divider(),
        selector_section("", vec![SelectorOption::new("x", "Other")]),
        SelectorOption::new("z", "Zed"),
    ];
    let g = InputGroupProps {
        id: s("grp"),
        label: s("Group"),
        size: s("sm"),
        ..Default::default()
    };
    check(
        "selector",
        vec![
            (
                "default",
                selector(&SelectorProps {
                    label: s("Fruit"),
                    options: opts.clone(),
                    ..Default::default()
                }),
            ),
            (
                "full",
                selector(&SelectorProps {
                    id: s("f"),
                    label: s("Fruit"),
                    options: opts.clone(),
                    value: s("a"),
                    has_clear: true,
                    description: s("Pick one"),
                    is_required: true,
                    status: Some(FieldStatus::new("error", "Bad")),
                    size: s("lg"),
                    placeholder: s("Choose"),
                    width: s("300px"),
                    indicator_position: s("start"),
                    placement: s("above"),
                    is_default_open: true,
                    is_loading: true,
                    name: s("fruit"),
                    form: s("frm"),
                    class: s("c"),
                    control_class: s("cc"),
                    attrs: vec![a("data-x", "1")],
                    trigger_attrs: vec![a("data-t", "1")],
                    value_attrs: vec![a("data-bind:fruit", "")],
                    ..Default::default()
                }),
            ),
            (
                "search-sections",
                selector(&SelectorProps {
                    id: s("s"),
                    label: s("Region"),
                    options: sections,
                    has_search: true,
                    start_icon: s("search"),
                    search_placeholder: s("Find"),
                    empty_text: s("Nothing"),
                    empty_search_text: s("No match"),
                    variant: s("secondary"),
                    status: Some(FieldStatus::new("warning", "Slow")),
                    is_optional: true,
                    is_required: true,
                    ..Default::default()
                }),
            ),
            (
                "read-only",
                selector(&SelectorProps {
                    id: s("r"),
                    label: s("Locked"),
                    options: opts.clone(),
                    value: s("b"),
                    is_read_only: true,
                    has_clear: true,
                    ..Default::default()
                }),
            ),
            (
                "disabled-empty",
                selector(&SelectorProps {
                    id: s("d"),
                    label: s("Off"),
                    is_disabled: true,
                    name: s("off"),
                    variant: s("ghost"),
                    status: Some(FieldStatus::new("error", "")),
                    ..Default::default()
                }),
            ),
            (
                "group",
                input_group(
                    &g,
                    selector(&SelectorProps {
                        id: s("gs"),
                        label: s("Unit"),
                        options: opts,
                        group: g.member(),
                        ..Default::default()
                    }),
                ),
            ),
        ],
    );
}
