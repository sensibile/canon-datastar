//! A small HTML builder whose output is byte-for-byte what the Go package's builder
//! (go/canon/html.go) writes: attributes in a fixed order, double-quoted, escaped like maud
//! (& < > "), boolean attributes as bare names, no whitespace between tags.

use crate::Attr;
use maud::{Markup, PreEscaped};

/// An ordered attribute list being built.
#[derive(Default)]
pub(crate) struct Attrs(String);

pub(crate) fn attrs() -> Attrs {
    Attrs::default()
}

impl Attrs {
    /// Writes name="value".
    pub(crate) fn set(mut self, name: &str, value: &str) -> Self {
        check_name(name);
        self.0.push(' ');
        self.0.push_str(name);
        self.0.push_str("=\"");
        escape_to(&mut self.0, value);
        self.0.push('"');
        self
    }

    /// Writes name="value" when value is not empty.
    pub(crate) fn opt(self, name: &str, value: &str) -> Self {
        if value.is_empty() {
            self
        } else {
            self.set(name, value)
        }
    }

    /// Writes name="value" when cond holds.
    pub(crate) fn when(self, cond: bool, name: &str, value: &str) -> Self {
        if cond { self.set(name, value) } else { self }
    }

    /// Writes a bare boolean attribute when cond holds.
    pub(crate) fn flag(mut self, name: &str, cond: bool) -> Self {
        if cond {
            check_name(name);
            self.0.push(' ');
            self.0.push_str(name);
        }
        self
    }

    /// Appends caller-supplied attributes in order.
    pub(crate) fn extra(self, list: &[Attr]) -> Self {
        list.iter().fold(self, |a, (n, v)| a.set(n, v))
    }
}

fn check_name(name: &str) {
    assert!(
        !name.is_empty() && !name.chars().any(|c| " \t\n\u{c}\r\"'>/=<".contains(c)),
        "canon-datastar: invalid HTML attribute name {name:?}"
    );
}

pub(crate) fn escape_to(out: &mut String, s: &str) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c => out.push(c),
        }
    }
}

/// Renders `<tag attrs>children</tag>`; `None` children are skipped.
pub(crate) fn el<I>(tag: &str, a: Attrs, children: I) -> Markup
where
    I: IntoIterator<Item = Option<Markup>>,
{
    let mut out = format!("<{tag}{}>", a.0);
    for c in children.into_iter().flatten() {
        out.push_str(&c.0);
    }
    out.push_str(&format!("</{tag}>"));
    PreEscaped(out)
}

/// Renders `<tag attrs>` with no end tag (input).
pub(crate) fn void(tag: &str, a: Attrs) -> Markup {
    PreEscaped(format!("<{tag}{}>", a.0))
}

/// Escaped text.
pub(crate) fn text(s: &str) -> Markup {
    let mut out = String::new();
    escape_to(&mut out, s);
    PreEscaped(out)
}

/// Trusted markup as is.
pub(crate) fn raw(s: &str) -> Markup {
    PreEscaped(s.to_string())
}

/// Concatenates parts; `None` entries are skipped.
pub(crate) fn group<I: IntoIterator<Item = Option<Markup>>>(parts: I) -> Markup {
    PreEscaped(parts.into_iter().flatten().map(|m| m.0).collect())
}

/// Joins a base class with an optional extra class.
pub(crate) fn cls(base: &str, extra: &str) -> String {
    if extra.is_empty() {
        base.to_string()
    } else {
        format!("{base} {extra}")
    }
}

/// Joins "prop: value" declarations into a style value ("" when none).
pub(crate) fn styles(decls: &[String]) -> String {
    decls
        .iter()
        .filter(|d| !d.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join("; ")
}

pub(crate) fn decl(prop: &str, value: &str) -> String {
    if value.is_empty() {
        String::new()
    } else {
        format!("{prop}: {value}")
    }
}

pub(crate) fn or<'a>(v: &'a str, fallback: &'a str) -> &'a str {
    if v.is_empty() { fallback } else { v }
}

/// Joins non-empty ids with spaces (for aria-describedby).
pub(crate) fn join_ids(ids: &[&str]) -> String {
    ids.iter()
        .filter(|s| !s.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Drops attributes whose name starts with `prefix` (e.g. "data-on:" handlers).
pub(crate) fn without_prefix(list: &[Attr], prefix: &str) -> Vec<Attr> {
    list.iter()
        .filter(|(n, _)| !n.starts_with(prefix))
        .cloned()
        .collect()
}

/// Quotes `s` as a single-quoted JavaScript string for Datastar expressions.
pub(crate) fn js_string(s: &str) -> String {
    let mut out = String::from("'");
    for c in s.chars() {
        match c {
            '\\' | '\'' => {
                out.push('\\');
                out.push(c);
            }
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            c => out.push(c),
        }
    }
    out.push('\'');
    out
}
