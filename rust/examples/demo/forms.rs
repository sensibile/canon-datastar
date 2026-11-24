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
                units: s("USD"), value_attrs: bind("amount"), ..Default::default() }))
            (number_input(&NumberInputProps { id: s("ratio"), label: s("Ratio (de-DE)"), value: Some(0.5), step: 0.1,
                locale: s("de-DE"), status: Some(FieldStatus::new("success", "Looks good")), ..Default::default() }))
        }
        p.demo-panel { "qty: " span data-text="$qty" {} " · " button type="button" data-on:click="$qty = 7" { "Set qty to 7" } " · amount: " span data-text="JSON.stringify($amount)" {} }
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

fn opt(value: &str, label: &str) -> SelectorOption {
    SelectorOption::new(value, label)
}

fn fruit_options() -> Vec<SelectorOption> {
    vec![
        SelectorOption {
            description: s("Crisp and sweet"),
            ..opt("apple", "Apple")
        },
        opt("banana", "Banana"),
        SelectorOption {
            icon: s("success"),
            ..opt("cherry", "Cherry")
        },
        SelectorOption {
            is_disabled: true,
            ..opt("durian", "Durian")
        },
        opt("elderberry", "Elderberry"),
        opt("fig", "Fig"),
        opt("grape", "Grape"),
    ]
}

fn region_options() -> Vec<SelectorOption> {
    vec![
        selector_section(
            "Americas",
            vec![
                opt("us-east", "US East"),
                opt("us-west", "US West"),
                opt("sa", "São Paulo"),
            ],
        ),
        selector_divider(),
        selector_section(
            "Asia Pacific",
            vec![
                opt("seoul", "Seoul"),
                opt("tokyo", "Tokyo"),
                opt("sydney", "Sydney"),
            ],
        ),
        selector_divider(),
        opt("global", "Global"),
    ]
}

fn selectors() -> Markup {
    html! {
        div.demo-grid {
            (selector(&SelectorProps { id: s("fruit"), label: s("Fruit"), options: fruit_options(), value: s("banana"), has_clear: true,
                description: s("Type a letter to jump"), value_attrs: bind("fruit"), ..Default::default() }))
            (selector(&SelectorProps { id: s("region"), label: s("Region"), options: region_options(), has_search: true,
                placeholder: s("Pick a region"), start_icon: s("search"), value_attrs: bind("region"), ..Default::default() }))
            (selector(&SelectorProps { id: s("sort"), label: s("Sort"), options: fruit_options(), variant: s("secondary"), size: s("sm"),
                indicator_position: s("start"), status: Some(FieldStatus::new("warning", "Detached for this variant")), ..Default::default() }))
            (selector(&SelectorProps { id: s("locked"), label: s("Read only"), options: fruit_options(), value: s("fig"), is_read_only: true,
                ..Default::default() }))
            (selector(&SelectorProps { id: s("off"), label: s("Disabled"), options: fruit_options(), is_disabled: true,
                status: Some(FieldStatus::new("error", "Pick a fruit")), ..Default::default() }))
        }
        p.demo-panel { "fruit: " span data-text="$fruit" {} " · region: " span data-text="$region" {} " · " button type="button" data-on:click="$fruit = 'grape'" { "Set fruit to grape" } }
    }
}

fn multi_selectors() -> Markup {
    let vals = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    html! {
        div.demo-grid {
            (multi_selector(&MultiSelectorProps { id: s("fruits"), label: s("Fruits"), options: fruit_options(), value: vals(&["apple", "fig"]),
                has_clear: true, has_select_all: true, trigger_display: s("labels"), description: s("The list stays open while you toggle"),
                value_attrs: bind("fruits"), ..Default::default() }))
            (multi_selector(&MultiSelectorProps { id: s("tags"), label: s("Regions"), options: region_options(), has_search: true,
                has_create: true, has_select_all: true, trigger_display: s("badges"), max_badges: 2, placeholder: s("Any region"),
                value_attrs: bind("tags"), ..Default::default() }))
            (multi_selector(&MultiSelectorProps { id: s("count"), label: s("Count"), options: fruit_options(), value: vals(&["banana", "cherry"]),
                variant: s("secondary"), size: s("sm"), indicator_position: s("end"), ..Default::default() }))
            (multi_selector(&MultiSelectorProps { id: s("mlocked"), label: s("Read only"), options: fruit_options(), value: vals(&["fig"]),
                is_read_only: true, ..Default::default() }))
        }
        p.demo-panel { "fruits: " span data-text="JSON.stringify($fruits)" {} " · tags: " span data-text="JSON.stringify($tags)" {} " · " button type="button" data-on:click="$fruits = ['grape', 'banana']" { "Set fruits" } }
    }
}

/// A static Typeahead list with descriptions and keywords.
fn people() -> Vec<TypeaheadItem> {
    let p = |id: &str, label: &str, desc: &str, kw: &[&str]| TypeaheadItem {
        description: s(desc),
        keywords: kw.iter().map(|k| s(k)).collect(),
        ..TypeaheadItem::new(id, label)
    };
    vec![
        p("ada", "Ada Lovelace", "Analytical Engine", &["math"]),
        p("alan", "Alan Turing", "Computability", &["enigma"]),
        p("grace", "Grace Hopper", "Compilers", &["cobol", "navy"]),
        p("edsger", "Edsger Dijkstra", "Shortest paths", &[]),
        p("barbara", "Barbara Liskov", "Substitution", &[]),
        p("donald", "Donald Knuth", "TeX", &["taocp"]),
        p(
            "margaret",
            "Margaret Hamilton",
            "Apollo software",
            &["nasa"],
        ),
        p("ken", "Ken Thompson", "Unix, Go", &[]),
    ]
}

/// A static Tokenizer list.
fn languages() -> Vec<TypeaheadItem> {
    [
        ("go", "Go"),
        ("rust", "Rust"),
        ("ts", "TypeScript"),
        ("py", "Python"),
        ("zig", "Zig"),
        ("c", "C"),
        ("kt", "Kotlin"),
        ("swift", "Swift"),
        ("ex", "Elixir"),
        ("hs", "Haskell"),
    ]
    .iter()
    .map(|(id, l)| TypeaheadItem::new(id, l))
    .collect()
}

const CITY_NAMES: &[&str] = &[
    "Amsterdam",
    "Athens",
    "Bangkok",
    "Barcelona",
    "Berlin",
    "Bogotá",
    "Buenos Aires",
    "Busan",
    "Cairo",
    "Cape Town",
    "Chicago",
    "Copenhagen",
    "Daejeon",
    "Delhi",
    "Dublin",
    "Helsinki",
    "Istanbul",
    "Jakarta",
    "Lagos",
    "Lima",
    "Lisbon",
    "London",
    "Los Angeles",
    "Madrid",
    "Melbourne",
    "Mexico City",
    "Montréal",
    "Mumbai",
    "Nairobi",
    "New York",
    "Osaka",
    "Oslo",
    "Paris",
    "Prague",
    "Rome",
    "San Francisco",
    "Santiago",
    "São Paulo",
    "Seoul",
    "Singapore",
    "Stockholm",
    "Sydney",
    "Taipei",
    "Tokyo",
    "Toronto",
    "Vienna",
    "Warsaw",
    "Zürich",
];

/// The demo's server-side search: a case-insensitive substring match.
pub(crate) fn search_cities(query: &str) -> Vec<TypeaheadItem> {
    let q = query.trim().to_lowercase();
    CITY_NAMES
        .iter()
        .filter(|c| q.is_empty() || c.to_lowercase().contains(&q))
        .map(|c| TypeaheadItem::new(&c.to_lowercase().replace(' ', "-"), c))
        .collect()
}

/// Asks /demo/search for the results of the query in an "astryx-search" event.
fn search_expr(id: &str) -> Vec<Attr> {
    vec![attr(
        "data-on:astryx-search",
        &format!("@get('/demo/search?id={id}&q=' + encodeURIComponent(evt.detail.query))"),
    )]
}

fn searches() -> Markup {
    html! {
        div.demo-grid {
            (typeahead(&TypeaheadProps { id: s("person"), label: s("Person"), items: people(), value: s("ada"),
                has_entries_on_focus: true, description: s("Static list; matches keywords too (try “nasa”)"),
                value_attrs: bind("person"), ..Default::default() }))
            (typeahead(&TypeaheadProps { id: s("city"), label: s("City"), is_remote: true, start_icon: s("search"),
                description: s("Searched on the server"), placeholder: s("Search cities…"),
                attrs: search_expr("city"), value_attrs: bind("city"), ..Default::default() }))
            (tokenizer(&TokenizerProps { id: s("langs"), label: s("Languages"), items: languages(),
                value: vec![TypeaheadItem::new("go", "Go"), TypeaheadItem::new("rust", "Rust")], has_create: true, has_clear: true,
                max_entries: 4, has_entries_on_focus: true, description: s("Up to 4; type a new one to create it"),
                value_attrs: bind("langs"), ..Default::default() }))
            (tokenizer(&TokenizerProps { id: s("cities"), label: s("Cities"), is_remote: true, size: s("sm"),
                attrs: search_expr("cities"), value_attrs: bind("cities"), ..Default::default() }))
            (typeahead(&TypeaheadProps { id: s("tdisabled"), label: s("Disabled"), items: people(), value: s("grace"),
                is_disabled: true, ..Default::default() }))
        }
        p.demo-panel { "person: " span data-text="$person" {} " · city: " span data-text="$city" {} " · langs: " span data-text="JSON.stringify($langs)" {} " · cities: " span data-text="JSON.stringify($cities)" {} " · " button type="button" data-on:click="$person = 'grace'" { "Set person" } }
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
        div data-signals="{bio: 'Writes Go and Rust.', qty: 3, amount: '1234.5', price: 1200, volume: 40, low: 20, high: 80, plan: 'pro', fruit: 'banana', region: '', fruits: ['apple', 'fig'], tags: [], person: 'ada', city: '', langs: ['go', 'rust'], cities: [], agree: false}" {
            (section("TextArea", text_areas()))
            (section("NumberInput", number_inputs()))
            (section("InputGroup", input_groups()))
            (section("CheckboxList and RadioList", choice_lists()))
            (section("Slider", sliders()))
            (section("FileInput", file_inputs()))
            (section("Selector", selectors()))
            (section("MultiSelector", multi_selectors()))
            (section("Typeahead and Tokenizer", searches()))
            (section("Item", items()))
            (section("Submit", html! {
                (button(&ButtonProps { label: s("Send signals"), variant: s("primary"),
                    attrs: vec![attr("data-on:click", "@post('/demo/forms')")], ..Default::default() }))
                (submitted(""))
            }))
        }
    }
}
