//! Builds an element whose attribute list is partly dynamic. maud's `html!` needs
//! attribute names at compile time, so component roots that accept extra attributes
//! (Datastar bindings) are written with this builder instead.

use maud::{Escaper, Markup, PreEscaped};
use std::fmt::Write;

pub(crate) struct Tag {
    buf: String,
}

impl Tag {
    pub(crate) fn new(name: &str) -> Self {
        Tag {
            buf: format!("<{name}"),
        }
    }

    pub(crate) fn attr(mut self, name: &str, value: &str) -> Self {
        assert!(valid_name(name), "invalid HTML attribute name: {name:?}");
        write!(self.buf, " {name}=\"").unwrap();
        Escaper::new(&mut self.buf).write_str(value).unwrap();
        self.buf.push('"');
        self
    }

    pub(crate) fn attr_opt(self, name: &str, value: Option<&str>) -> Self {
        match value {
            Some(v) => self.attr(name, v),
            None => self,
        }
    }

    pub(crate) fn attrs<'a>(self, list: impl IntoIterator<Item = &'a crate::Attr>) -> Self {
        list.into_iter().fold(self, |t, (n, v)| t.attr(n, v))
    }

    /// Closes the start tag, writes `children` and the end tag.
    pub(crate) fn wrap(mut self, name: &str, children: Markup) -> Markup {
        write!(self.buf, ">{}</{name}>", children.into_string()).unwrap();
        PreEscaped(self.buf)
    }
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && !name.chars().any(|c| {
            c.is_whitespace() || c.is_control() || matches!(c, '"' | '\'' | '>' | '/' | '=' | '<')
        })
}
