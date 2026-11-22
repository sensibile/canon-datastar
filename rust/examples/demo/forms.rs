//! The demo's Forms page. Mirrors go/cmd/demo/forms.templ.

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

fn bind(name: &str) -> Vec<Attr> {
    vec![attr(&format!("data-bind:{name}"), "")]
}

/// The echo of the signals the page sent, re-rendered by POST /demo/forms.
pub(crate) fn submitted(json: &str) -> Markup {
    html! { pre #form-result .demo-panel { (json) } }
}

fn cb(label: &str, value: &str) -> CheckboxListItem {
    CheckboxListItem {
        label: s(label),
        value: s(value),
        ..Default::default()
    }
}

fn radio(label: &str, value: &str) -> RadioListItem {
    RadioListItem {
        label: s(label),
        value: s(value),
        ..Default::default()
    }
}

fn mark(v: f64, l: &str) -> SliderMark {
    SliderMark {
        value: v,
        label: s(l),
    }
}

fn text_areas() -> Markup {
    html! {
        div.demo-grid {
            (text_area(&TextAreaProps { id: s("bio"), label: s("Bio"), value: s("Writes Go and Rust."), max_length: 40,
                description: s("Counts emoji as one character"), input_attrs: bind("bio"), ..Default::default() }))
            (text_area(&TextAreaProps { label: s("Notes"), start_icon: s("info"), is_loading: true, rows: 2,
                status: Some(FieldStatus::new("warning", "Saving is slow right now")), ..Default::default() }))
            (text_area(&TextAreaProps { label: s("Disabled notes"), value: s("Read only text"), is_disabled: true, ..Default::default() }))
        }
        p.demo-panel { "bio: " span data-text="$bio" {} }
    }
}

fn number_inputs() -> Markup {
    html! {
        div.demo-grid {
            (number_input(&NumberInputProps { id: s("qty"), label: s("Quantity"), value: Some(3.0), min: Some(0.0), max: Some(10.0),
                has_number_steppers: true, is_integer_only: true, value_attrs: bind("qty"), ..Default::default() }))
            (number_input(&NumberInputProps { id: s("amount"), label: s("Amount"), value: Some(1234.5), has_clear: true,
                format: s(r#"{"style":"currency","currency":"USD"}"#), description: s("Paste \"(12)\" or \"€5\""),
                units: s("USD"), ..Default::default() }))
            (number_input(&NumberInputProps { id: s("ratio"), label: s("Ratio (de-DE)"), value: Some(0.5), step: 0.1,
                locale: s("de-DE"), status: Some(FieldStatus::new("success", "Looks good")), ..Default::default() }))
        }
        p.demo-panel { "qty: " span data-text="$qty" {} " · " button type="button" data-on:click="$qty = 7" { "Set qty to 7" } }
    }
}

fn input_groups() -> Markup {
    let price = InputGroupProps {
        id: s("price"),
        label: s("Price"),
        description: s("Per seat, per month"),
        ..Default::default()
    };
    let site = InputGroupProps {
        id: s("site"),
        label: s("Website"),
        size: s("md"),
        ..Default::default()
    };
    html! {
        div.demo-grid {
            (input_group(&price, html! {
                (input_group_text("$"))
                (number_input(&NumberInputProps { id: s("price-amount"), label: s("Amount"), value: Some(1200.0),
                    group: price.member(), value_attrs: bind("price"), ..Default::default() }))
                (input_group_text("USD"))
            }))
            (input_group(&site, html! {
                (input_group_text("https://"))
                (text_input(&TextInputProps { id: s("site-host"), label: s("Host"), placeholder: s("example.com"),
                    group: site.member(), ..Default::default() }))
                (button(&ButtonProps { label: s("Check"), attrs: vec![input_group_item()], ..Default::default() }))
            }))
        }
    }
}

fn choice_lists() -> Markup {
    let toppings = vec![
        CheckboxListItem {
            description: s("Mozzarella"),
            ..cb("Cheese", "cheese")
        },
        cb("Mushrooms", "mushrooms"),
        CheckboxListItem {
            is_disabled: true,
            ..cb("Olives", "olives")
        },
        CheckboxListItem {
            is_loading: true,
            ..cb("Basil", "basil")
        },
    ];
    let plans = vec![
        RadioListItem {
            description: s("Up to 3 projects"),
            ..radio("Free", "free")
        },
        RadioListItem {
            description: s("Unlimited projects"),
            ..radio("Pro", "pro")
        },
        RadioListItem {
            is_disabled: true,
            ..radio("Enterprise", "enterprise")
        },
    ];
    html! {
        div.demo-grid {
            (checkbox_list(&CheckboxListProps { id: s("toppings"), label: s("Toppings"), name: s("toppings"), is_required: true,
                description: s("Pick at least one"), items: toppings, ..Default::default() }))
            (checkbox_list(&CheckboxListProps { id: s("compact"), label: s("Compact, dividers"), density: s("compact"),
                has_dividers: true, items: vec![
                    CheckboxListItem { is_checked: true, ..cb("Email", "") },
                    CheckboxListItem { is_indeterminate: true, ..cb("SMS", "") },
                    cb("Push", ""),
                ], ..Default::default() }))
            (radio_list(&RadioListProps { id: s("plan"), label: s("Plan"), value: s("pro"), items: plans,
                input_attrs: bind("plan"), ..Default::default() }))
            (radio_list(&RadioListProps { id: s("size"), label: s("Size (none checked)"), orientation: s("horizontal"),
                size: s("sm"), status: Some(FieldStatus::new("error", "Choose a size")),
                items: vec![radio("S", "s"), radio("M", "m"), radio("L", "l")], ..Default::default() }))
        }
        p.demo-panel { "plan: " span data-text="$plan" {} }
    }
}

fn sliders() -> Markup {
    html! {
        div.demo-grid {
            (slider(&SliderProps { id: s("volume"), label: s("Volume"), values: vec![40.0], step: 5.0,
                marks: vec![mark(0.0, "0"), mark(50.0, "50"), mark(100.0, "100")], value_attrs: bind("volume"),
                ..Default::default() }))
            (slider(&SliderProps { id: s("range"), label: s("Price range"), values: vec![20.0, 80.0],
                min_steps_between_thumbs: 10, value_display: s("text"),
                format: s(r#"{"style":"currency","currency":"USD","maximumFractionDigits":0}"#),
                value_attrs: bind("low"), end_value_attrs: bind("high"), ..Default::default() }))
            (slider(&SliderProps { label: s("Disabled"), values: vec![30.0], is_disabled: true, ..Default::default() }))
        }
        p.demo-panel { "volume: " span data-text="$volume" {} " · range: " span data-text="$low + '–' + $high" {} }
    }
}

fn file_inputs() -> Markup {
    html! {
        div.demo-grid {
            (file_input(&FileInputProps { id: s("avatar"), label: s("Avatar"), accept: s("image/*"), max_size: 1048576,
                description: s("An image up to 1 MB"), ..Default::default() }))
            (file_input(&FileInputProps { id: s("docs"), label: s("Documents"), mode: s("dropzone"), is_multiple: true,
                max_files: 2, accept: s(".pdf,.txt"), ..Default::default() }))
            (file_input(&FileInputProps { label: s("Uploading"), is_loading: true, ..Default::default() }))
        }
    }
}

fn items() -> Markup {
    let qty = || Some(vec![attr("data-on:click", "$qty++")]);
    html! {
        div.demo-grid {
            (item(&ItemProps { label: s("Plain row"), description: s("With a description"), ..Default::default() }))
            (item(&ItemProps { label: s("Button row"), description: s("Datastar click"), action: qty(),
                end_content: Some(badge(&BadgeProps { label: s("qty++"), ..Default::default() })), ..Default::default() }))
            (item(&ItemProps { label: s("Link row"), href: s("/content"), layout: s("inline"),
                description: s("inline description"), ..Default::default() }))
            (item(&ItemProps { label: s("Disabled row"), action: qty(), is_disabled: true, ..Default::default() }))
        }
    }
}

pub(crate) fn page() -> Markup {
    html! {
        h1 { "Forms" }
        p.demo-lead { "Server-rendered controls bound to Datastar signals; static/canon.js adds counters, drafts, stepping, dragging and file checks." }
        div data-signals="{bio: 'Writes Go and Rust.', qty: 3, price: 1200, volume: 40, low: 20, high: 80, plan: 'pro', agree: false}" {
            (section("TextArea", text_areas()))
            (section("NumberInput", number_inputs()))
            (section("InputGroup", input_groups()))
            (section("CheckboxList and RadioList", choice_lists()))
            (section("Slider", sliders()))
            (section("FileInput", file_inputs()))
            (section("Item", items()))
            (section("Submit", html! {
                (button(&ButtonProps { label: s("Send signals"), variant: s("primary"),
                    attrs: vec![attr("data-on:click", "@post('/demo/forms')")], ..Default::default() }))
                (submitted(""))
            }))
        }
    }
}
