//! The demo's Components page. Mirrors go/cmd/demo/components.templ.

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

fn btn(p: ButtonProps) -> Markup {
    button(&p)
}

pub(crate) fn page() -> Markup {
    let t = |kind: &str, body: &str| {
        text(
            &TextProps {
                kind: s(kind),
                ..Default::default()
            },
            html! { (body) },
        )
    };
    html! {
        h1 { "Components" }
        p.demo-lead { "Server-rendered with Rust; interactivity through Datastar signals and SSE." }
        div data-signals="{saving: false, saved: 0, name: '', agree: false, notify: true, size: 'md'}" {
            (section("Button", html! {
                (btn(ButtonProps { label: s("Primary"), variant: s("primary"), ..Default::default() }))
                (btn(ButtonProps { label: s("Secondary"), ..Default::default() }))
                (btn(ButtonProps { label: s("Ghost"), variant: s("ghost"), ..Default::default() }))
                (btn(ButtonProps { label: s("Destructive"), variant: s("destructive"), ..Default::default() }))
                (btn(ButtonProps {
                    label: s("With icon"),
                    icon: s("check"),
                    end_content: Some(badge(&BadgeProps { label: s("3"), ..Default::default() })),
                    ..Default::default()
                }))
                (icon_button(&ButtonProps { label: s("Close"), icon: s("close"), variant: s("ghost"), ..Default::default() }))
                (btn(ButtonProps { label: s("Disabled"), is_disabled: true, ..Default::default() }))
                (btn(ButtonProps { label: s("Loading"), is_loading: true, ..Default::default() }))
                (btn(ButtonProps { label: s("Link"), href: s("#"), elevation: s("low"), ..Default::default() }))
            }))
            (section("Button sizes", html! {
                @for (label, size) in [("Small", "sm"), ("Medium", ""), ("Large", "lg")] {
                    (btn(ButtonProps { label: s(label), size: s(size), variant: s("primary"), ..Default::default() }))
                }
            }))
            (section("Datastar: loading over SSE", html! {
                (btn(ButtonProps {
                    label: s("Save"),
                    variant: s("primary"),
                    icon: s("upload"),
                    attrs: vec![
                        attr("data-attr:aria-busy", "$saving && 'true'"),
                        attr("data-on:click", "$saving = true; @post('/demo/save')"),
                    ],
                    ..Default::default()
                }))
                (text(
                    &TextProps { kind: s("supporting"), attrs: vec![attr("data-text", "'Saved ' + $saved + ' times'")], ..Default::default() },
                    html! { "Saved 0 times" },
                ))
            }))
            (section("Badge", html! {
                @for v in ["neutral", "info", "success", "warning", "error", "blue", "green", "purple", "orange", "teal"] {
                    (badge(&BadgeProps { label: s(v), variant: s(v), ..Default::default() }))
                }
                (badge(&BadgeProps { label: s("Done"), variant: s("success"), icon: s("check"), ..Default::default() }))
            }))
            (section("Text and Heading", vstack(&StackProps { gap: s("2"), ..Default::default() }, html! {
                (heading(&HeadingProps { level: 2, ..Default::default() }, html! { "Heading level 2" }))
                (heading(&HeadingProps { level: 3, kind: s("display-3"), ..Default::default() }, html! { "Display 3" }))
                (t("large", "Large text"))
                (t("supporting", "Supporting text in secondary color"))
                (t("code", "code text"))
                (text(
                    &TextProps { max_lines: 1, attrs: vec![attr("style", "width: 240px")], ..Default::default() },
                    html! { "A long line that gets truncated with an ellipsis when it runs out of room" },
                ))
                (link(
                    &LinkProps { href: s("https://astryx.atmeta.com"), is_external_link: true, ..Default::default() },
                    html! { "Astryx docs" },
                ))
            })))
            (section("Icon", html! {
                @for n in ["check", "close", "chevronDown", "search", "calendar", "clock", "info", "warning", "error", "success", "externalLink", "copy"] {
                    (icon(&IconProps { size: s("lg"), label: s(n), ..IconProps::new(n) }))
                }
            }))
            (section("Spinner", html! {
                (spinner(&SpinnerProps { size: s("sm"), ..Default::default() }))
                (spinner(&SpinnerProps::default()))
                (spinner(&SpinnerProps { size: s("lg"), shade: s("subtle"), ..Default::default() }))
                (spinner(&SpinnerProps { size: s("xl"), label: s("Loading"), ..Default::default() }))
            }))
            (section("Divider", vstack(&StackProps { gap: s("4"), width: s("100%"), ..Default::default() }, html! {
                (divider(&DividerProps::default()))
                (divider(&DividerProps { label: s("or"), variant: s("strong"), ..Default::default() }))
            })))
            (section("Form controls (data-bind)", vstack(&StackProps { gap: s("4"), width: s("360px"), ..Default::default() }, html! {
                (text_input(&TextInputProps {
                    id: s("name"), label: s("Name"), description: s("Bound to $name"), placeholder: s("Type here"),
                    input_attrs: vec![attr("data-bind:name", "")], ..Default::default()
                }))
                (text_input(&TextInputProps {
                    id: s("email"), label: s("Email"), start_icon: s("search"), is_required: true,
                    status: Some(FieldStatus::new("error", "Enter a valid email")), ..Default::default()
                }))
                (text_input(&TextInputProps {
                    id: s("ro"), label: s("Read only"), value: s("Fixed value"), is_read_only: true, size: s("sm"), ..Default::default()
                }))
                (checkbox_input(&CheckboxInputProps {
                    id: s("agree"), label: s("I agree"), description: s("Bound to $agree"),
                    input_attrs: vec![attr("data-bind:agree", "")], ..Default::default()
                }))
                (checkbox_input(&CheckboxInputProps { id: s("some"), label: s("Some selected"), is_indeterminate: true, ..Default::default() }))
                (checkbox_input(&CheckboxInputProps { id: s("off"), label: s("Disabled"), is_disabled: true, checked: true, ..Default::default() }))
                (switch(&SwitchProps {
                    id: s("notify"), label: s("Notifications"), checked: true,
                    input_attrs: vec![attr("data-bind:notify", "")], ..Default::default()
                }))
                (switch(&SwitchProps {
                    id: s("spread"), label: s("Spread label"), label_position: s("start"), label_spacing: s("spread"), size: s("sm"),
                    ..Default::default()
                }))
                (text(
                    &TextProps {
                        kind: s("supporting"),
                        attrs: vec![attr("data-text", "JSON.stringify({name: $name, agree: $agree, notify: $notify})")],
                        ..Default::default()
                    },
                    html! {},
                ))
            })))
        }
    }
}
