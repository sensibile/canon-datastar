//! canon-datastar gallery: `cargo run --example demo` (from rust/). Mirrors go/cmd/demo.

mod components;
mod content;
mod forms;
mod overlays;

use axum::{
    Router,
    extract::Query,
    response::{Sse, sse::Event},
    routing::{get, post},
};
use canon_datastar::{
    THEMES, ThemeMode, ThemeProps, ToastProps, ToastViewportProps, theme, toast, toast_viewport,
};
use datastar::{
    axum::ReadSignals,
    prelude::{ElementPatchMode, PatchElements, PatchSignals},
};
use futures_util::stream::{self, Stream, StreamExt};
use maud::{DOCTYPE, Markup, html};
use serde::Deserialize;
use std::path::Path;
use std::{convert::Infallible, time::Duration};
use tower_http::services::ServeDir;

/// Datastar expression shared by <html> and Theme: drop data-theme for "system".
const MODE_EXPR: &str = "$mode === 'system' ? null : $mode";

/// Keeps the chosen theme and mode across demo pages.
fn nav_expr(path: &str) -> String {
    format!("'{path}?theme=' + $theme + '&mode=' + $mode")
}

const COLOR_GROUPS: &[(&str, &[&str])] = &[
    (
        "Background",
        &[
            "--color-background-body",
            "--color-background-surface",
            "--color-background-card",
            "--color-background-muted",
            "--color-background-popover",
            "--color-background-inverted",
        ],
    ),
    (
        "Text",
        &[
            "--color-text-primary",
            "--color-text-secondary",
            "--color-text-disabled",
            "--color-text-accent",
            "--color-text-error",
            "--color-text-success",
            "--color-text-warning",
            "--color-text-info",
        ],
    ),
    (
        "Accent",
        &[
            "--color-accent",
            "--color-accent-muted",
            "--color-on-accent",
        ],
    ),
    (
        "Status",
        &[
            "--color-error",
            "--color-error-muted",
            "--color-success",
            "--color-success-muted",
            "--color-warning",
            "--color-warning-muted",
            "--color-info",
            "--color-info-muted",
        ],
    ),
    (
        "Border",
        &[
            "--color-border",
            "--color-border-emphasized",
            "--color-border-error",
            "--color-border-success",
            "--color-border-warning",
            "--color-border-info",
        ],
    ),
];
const TYPE_STYLES: &[&str] = &[
    "display-1",
    "display-2",
    "display-3",
    "heading-1",
    "heading-2",
    "heading-3",
    "heading-4",
    "heading-5",
    "heading-6",
    "large",
    "body",
    "label",
    "supporting",
    "code",
];
const SPACING: &[&str] = &[
    "0-5", "1", "1-5", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12",
];
const RADII: &[&str] = &["inner", "element", "container", "page", "full"];
const SHADOWS: &[&str] = &["low", "med", "high"];

#[derive(Deserialize)]
struct ThemeQuery {
    theme: Option<String>,
    mode: Option<String>,
}

pub(crate) fn layout(title: &str, name: &str, mode: ThemeMode, content: Markup) -> Markup {
    let signals = serde_json::json!({ "mode": format!("{mode:?}").to_lowercase(), "theme": name })
        .to_string();
    let attrs = vec![
        (
            "data-attr:data-astryx-theme".to_string(),
            "$theme".to_string(),
        ),
        ("data-attr:data-theme".to_string(), MODE_EXPR.to_string()),
    ];
    html! {
        (DOCTYPE)
        html lang="en" data-astryx-theme=(name) data-theme=[mode.data_theme()]
            data-attr:data-astryx-theme="$theme" data-attr:data-theme=(MODE_EXPR) {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) }
                link rel="stylesheet" href="/canon/css/astryx.css";
                @for t in THEMES { link rel="stylesheet" href={ "/canon/css/themes/" (t) ".css" }; }
                link rel="stylesheet" href="/demo/demo.css";
                script type="module" src="/canon/static/canon.js" {}
                script type="module" src="/canon/static/datastar.js" {}
            }
            body data-signals=(signals) {
                (theme(&ThemeProps { name: Some(name.into()), mode, attrs }, html! {
                    div.demo-shell {
                        header.demo-header {
                            nav {
                                strong { "canon-datastar" } " "
                                a href="/" data-attr:href=(nav_expr("/")) { "Tokens" } " "
                                a href="/components" data-attr:href=(nav_expr("/components")) { "Components" } " "
                                a href="/content" data-attr:href=(nav_expr("/content")) { "Content" } " "
                                a href="/forms" data-attr:href=(nav_expr("/forms")) { "Forms" } " "
                                a href="/overlays" data-attr:href=(nav_expr("/overlays")) { "Overlays" }
                            }
                            div.demo-controls {
                                label {
                                    "Theme "
                                    select data-bind:theme {
                                        @for t in THEMES { option value=(t) selected[t == name] { (t) } }
                                    }
                                }
                                label {
                                    "Mode "
                                    select data-bind:mode {
                                        option value="system" selected[mode == ThemeMode::System] { "System" }
                                        option value="light" selected[mode == ThemeMode::Light] { "Light" }
                                        option value="dark" selected[mode == ThemeMode::Dark] { "Dark" }
                                    }
                                }
                            }
                        }
                        main { (content) }
                    }
                    (toast_viewport(&ToastViewportProps::default(), html! {}))
                }))
            }
        }
    }
}

fn tokens_page() -> Markup {
    html! {
        h1 { "Tokens" }
        p.demo-lead { "Astryx design tokens and themes, rendered by Rust + maud. Theme and mode switch with Datastar signals." }
        section.demo-section {
            h2 { "Color" }
            @for (group, vars) in COLOR_GROUPS {
                h3 { (group) }
                div.demo-swatches {
                    @for v in *vars {
                        div.demo-swatch {
                            div.demo-chip style={ "background: var(" (v) ")" } {}
                            code { (v) }
                        }
                    }
                }
            }
        }
        section.demo-section {
            h2 { "Typography" }
            @for t in TYPE_STYLES {
                div.demo-type-row style={ "font-size: var(--text-" (t) "-size); font-weight: var(--text-" (t) "-weight); line-height: var(--text-" (t) "-leading)" } {
                    span { "The quick brown fox" } code { "--text-" (t) }
                }
            }
        }
        section.demo-section {
            h2 { "Spacing" }
            @for s in SPACING {
                div.demo-space-row {
                    code { "--spacing-" (s) }
                    div.demo-bar style={ "width: var(--spacing-" (s) ")" } {}
                }
            }
        }
        section.demo-section {
            h2 { "Radius and shadow" }
            div.demo-boxes {
                @for r in RADII { div.demo-box style={ "border-radius: var(--radius-" (r) ")" } { code { "--radius-" (r) } } }
                @for s in SHADOWS { div.demo-box.shadow style={ "box-shadow: var(--shadow-" (s) ")" } { code { "--shadow-" (s) } } }
            }
        }
    }
}

fn theme_from_query(q: &ThemeQuery) -> (&str, ThemeMode) {
    let name = q
        .theme
        .as_deref()
        .filter(|t| THEMES.contains(t))
        .unwrap_or("neutral");
    (name, ThemeMode::parse(q.mode.as_deref().unwrap_or("")))
}

async fn index(Query(q): Query<ThemeQuery>) -> Markup {
    let (name, mode) = theme_from_query(&q);
    layout("canon-datastar · Tokens", name, mode, tokens_page())
}

async fn components_index(Query(q): Query<ThemeQuery>) -> Markup {
    let (name, mode) = theme_from_query(&q);
    layout(
        "canon-datastar · Components",
        name,
        mode,
        components::page(),
    )
}

async fn content_index(Query(q): Query<ThemeQuery>) -> Markup {
    let (name, mode) = theme_from_query(&q);
    layout("canon-datastar · Content", name, mode, content::page())
}

async fn forms_index(Query(q): Query<ThemeQuery>) -> Markup {
    let (name, mode) = theme_from_query(&q);
    layout("canon-datastar · Forms", name, mode, forms::page())
}

/// Forms: echo the signals the page sent, to show what the bound controls hold.
async fn demo_forms(
    ReadSignals(signals): ReadSignals<serde_json::Value>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let json = serde_json::to_string_pretty(&signals).unwrap_or_default();
    Sse::new(stream::once(async move {
        Ok(PatchElements::new(forms::submitted(&json).into_string()).write_as_axum_sse_event())
    }))
}

#[derive(Deserialize)]
struct SearchQuery {
    id: String,
    #[serde(default)]
    q: String,
}

/// Typeahead/Tokenizer remote search: answers with the results for the query (after a short
/// delay, to show the loading state).
async fn demo_search(
    Query(q): Query<SearchQuery>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    Sse::new(stream::once(async move {
        tokio::time::sleep(Duration::from_millis(200)).await;
        let results = canon_datastar::typeahead_results(&canon_datastar::TypeaheadResultsProps {
            id: q.id,
            items: forms::search_cities(&q.q),
            query: q.q,
        });
        Ok(PatchElements::new(results.into_string()).write_as_axum_sse_event())
    }))
}

/// ProgressBar over SSE: patches the bar as the "upload" advances; Datastar morphs it in place.
async fn demo_progress() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    Sse::new(stream::iter(0..=10).then(|i| async move {
        if i > 0 {
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        let bar = content::progress(f64::from(i * 10));
        Ok(PatchElements::new(bar.into_string()).write_as_axum_sse_event())
    }))
}

async fn overlays_index(Query(q): Query<ThemeQuery>) -> Markup {
    let (name, mode) = theme_from_query(&q);
    layout("canon-datastar · Overlays", name, mode, overlays::page())
}

/// Appends a server-rendered toast to the viewport.
fn toast_event(p: &ToastProps) -> Event {
    PatchElements::new(toast(p).into_string())
        .selector("#astryx-toasts")
        .mode(ElementPatchMode::Append)
        .write_as_axum_sse_event()
}

#[derive(Deserialize)]
struct ToastQuery {
    body: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
}

/// Toast over SSE.
async fn demo_toast(
    Query(q): Query<ToastQuery>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let p = ToastProps {
        body: q.body.unwrap_or_default(),
        kind: q.kind.unwrap_or_default(),
        ..Default::default()
    };
    Sse::new(stream::once(async move { Ok(toast_event(&p)) }))
}

#[derive(Deserialize)]
struct DeleteSignals {
    deleted: i64,
}

/// Dialog over SSE: closes the confirm dialog through its signal and shows a toast.
async fn demo_delete(
    ReadSignals(s): ReadSignals<DeleteSignals>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let delay = stream::once(async { tokio::time::sleep(Duration::from_millis(800)).await });
    let events = delay.flat_map(move |_| {
        let patch =
            serde_json::json!({ "confirmOpen": false, "deleted": s.deleted + 1 }).to_string();
        stream::iter([
            Ok(PatchSignals::new(patch).write_as_axum_sse_event()),
            Ok(toast_event(&ToastProps {
                body: "Project deleted".into(),
                ..Default::default()
            })),
        ])
    });
    Sse::new(events)
}

#[derive(Deserialize)]
struct SaveSignals {
    saved: i64,
}

/// Datastar demo: the button sets $saving, posts its signals here, and this SSE response
/// patches $saving back to false (clearing aria-busy) after a short delay.
async fn save(
    ReadSignals(s): ReadSignals<SaveSignals>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    Sse::new(stream::once(async move {
        tokio::time::sleep(Duration::from_millis(1200)).await;
        let patch = serde_json::json!({ "saving": false, "saved": s.saved + 1 }).to_string();
        Ok(PatchSignals::new(patch).write_as_axum_sse_event())
    }))
}

#[tokio::main]
async fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let app = Router::new()
        .route("/", get(index))
        .route("/components", get(components_index))
        .route("/content", get(content_index))
        .route("/forms", get(forms_index))
        .route("/demo/forms", post(demo_forms))
        .route("/demo/search", get(demo_search))
        .route("/overlays", get(overlays_index))
        .route("/demo/progress", post(demo_progress))
        .route("/demo/save", post(save))
        .route("/demo/toast", post(demo_toast))
        .route("/demo/delete", post(demo_delete))
        .nest_service("/canon/css", ServeDir::new(root.join("css")))
        .nest_service("/canon/static", ServeDir::new(root.join("static")))
        .nest_service("/demo", ServeDir::new(root.join("demo")));
    let addr = std::env::var("ADDR").unwrap_or_else(|_| "localhost:8081".into());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("canon-datastar demo on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}
