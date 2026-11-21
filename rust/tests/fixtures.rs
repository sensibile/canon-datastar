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
