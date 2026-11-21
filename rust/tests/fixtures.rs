//! Renders each case and compares it with fixtures/<component>/<case>.html,
//! the same golden files the Go package tests against.

use canon_datastar::{ThemeMode, ThemeProps, theme};
use maud::{Markup, PreEscaped};
use std::path::PathBuf;

/// templ writes &#34; and &#39;, maud writes &quot; and leaves ' alone; compare one spelling.
fn normalize(s: &str) -> String {
    s.replace("&#34;", "&quot;").replace("&#39;", "'")
}

fn check(component: &str, case: &str, got: Markup) {
    let path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "..",
        "fixtures",
        component,
        &format!("{case}.html"),
    ]
    .iter()
    .collect();
    let want = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    assert_eq!(
        normalize(&got.into_string()),
        want.trim_end_matches('\n'),
        "{}",
        path.display()
    );
}

fn child() -> Markup {
    PreEscaped("<p>Hello</p>".into())
}

fn attr(n: &str, v: &str) -> (String, String) {
    (n.into(), v.into())
}

#[test]
fn theme_fixtures() {
    check("theme", "default", theme(&ThemeProps::default(), child()));
    check(
        "theme",
        "dark",
        theme(
            &ThemeProps {
                name: Some("matcha".into()),
                mode: ThemeMode::Dark,
                ..Default::default()
            },
            child(),
        ),
    );
    check(
        "theme",
        "attrs",
        theme(
            &ThemeProps {
                name: Some("y2k".into()),
                mode: ThemeMode::Light,
                attrs: vec![
                    attr("data-attr:data-astryx-theme", "$theme"),
                    attr("title", r#"a "quoted" & <tagged> value"#),
                ],
            },
            child(),
        ),
    );
}
