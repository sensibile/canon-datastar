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
