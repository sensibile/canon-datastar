//! Card, Section, Center, Grid, GridSpan, AspectRatio. Ported from Astryx
//! (packages/core/src/Card, Section, Center, Grid, AspectRatio), MIT, Meta Platforms, Inc.
//! Sizes (`width`, `height`, `max_width`, `min_height`) are CSS lengths, e.g. "320px" or "100%".
//! Not ported: Card's layout container scope (header/footer bleed).

use crate::Attr;
use crate::html::{attrs, cls, decl, el, or, styles};
use crate::types::space_css;
use maud::Markup;

fn first<'a>(vals: &[&'a str]) -> &'a str {
    vals.iter().copied().find(|v| !v.is_empty()).unwrap_or("")
}

fn size_styles(width: &str, height: &str, max_width: &str, min_height: &str) -> Vec<String> {
    vec![
        decl("width", width),
        decl("height", height),
        decl("max-width", max_width),
        decl("min-height", min_height),
    ]
}

/// Props for [`card`].
#[derive(Clone, Debug, Default)]
pub struct CardProps {
    /// "default", "transparent", "muted", or a color: "blue", "cyan", "gray", "green", "orange",
    /// "pink", "purple", "red", "teal", "yellow".
    pub variant: String,
    /// "none" (default), "low", "med", "high".
    pub elevation: String,
    pub width: String,
    pub height: String,
    pub max_width: String,
    pub min_height: String,
    /// Spacing step overriding the theme's card padding.
    pub padding: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Renders a surface around `children`.
pub fn card(p: &CardProps, children: Markup) -> Markup {
    let mut decls = size_styles(&p.width, &p.height, &p.max_width, &p.min_height);
    decls.push(decl("--_card-padding", &space_css(&p.padding)));
    let a = attrs()
        .set("class", &cls("astryx-card", &p.class))
        .set("data-variant", or(&p.variant, "default"))
        .set("data-elevation", or(&p.elevation, "none"))
        .flag(
            "data-scrollable",
            !p.height.is_empty() && p.height != "auto",
        )
        .opt("style", &styles(&decls))
        .extra(&p.attrs);
    el("div", a, [Some(children)])
}

/// Props for [`section`]. Padding fields take spacing steps.
#[derive(Clone, Debug, Default)]
pub struct SectionProps {
    /// "section" (default), "transparent", "muted".
    pub variant: String,
    pub width: String,
    pub height: String,
    pub max_width: String,
    pub min_height: String,
    /// Borders on these sides: "top", "bottom", "start", "end".
    pub dividers: Vec<String>,
    pub padding: String,
    pub padding_inline: String,
    pub padding_inline_start: String,
    pub padding_inline_end: String,
    pub padding_block: String,
    pub padding_block_start: String,
    pub padding_block_end: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Renders a full-bleed band that cancels its container's padding.
pub fn section(p: &SectionProps, children: Markup) -> Markup {
    let has = |side: &str| p.dividers.iter().any(|d| d == side);
    let outer = attrs()
        .set("class", &cls("astryx-section-outer", &p.class))
        .opt(
            "style",
            &styles(&size_styles(
                &p.width,
                &p.height,
                &p.max_width,
                &p.min_height,
            )),
        )
        .extra(&p.attrs);
    let inner = attrs()
        .set("class", "astryx-section")
        .set("data-variant", or(&p.variant, "section"))
        .flag("data-divider-top", has("top"))
        .flag("data-divider-bottom", has("bottom"))
        .flag("data-divider-start", has("start"))
        .flag("data-divider-end", has("end"))
        .opt(
            "style",
            &styles(&[
                decl("--_section-padding", &space_css(&p.padding)),
                decl(
                    "--_section-pis",
                    &space_css(first(&[&p.padding_inline_start, &p.padding_inline])),
                ),
                decl(
                    "--_section-pie",
                    &space_css(first(&[&p.padding_inline_end, &p.padding_inline])),
                ),
                decl(
                    "--_section-pbs",
                    &space_css(first(&[&p.padding_block_start, &p.padding_block])),
                ),
                decl(
                    "--_section-pbe",
                    &space_css(first(&[&p.padding_block_end, &p.padding_block])),
                ),
            ]),
        );
    el("div", outer, [Some(el("div", inner, [Some(children)]))])
}

/// Props for [`center`]. Padding fields take spacing steps.
#[derive(Clone, Debug, Default)]
pub struct CenterProps {
    /// "both" (default), "horizontal", "vertical".
    pub axis: String,
    pub width: String,
    pub height: String,
    pub max_width: String,
    pub min_height: String,
    pub padding: String,
    pub padding_inline: String,
    pub padding_inline_start: String,
    pub padding_inline_end: String,
    pub padding_block: String,
    pub padding_block_start: String,
    pub padding_block_end: String,
    pub is_inline: bool,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Centers `children`.
pub fn center(p: &CenterProps, children: Markup) -> Markup {
    let mut decls = size_styles(&p.width, &p.height, &p.max_width, &p.min_height);
    let pad = |side: &str, step: &str, axis: &str| {
        decl(side, &space_css(first(&[step, axis, &p.padding])))
    };
    decls.push(pad(
        "padding-inline-start",
        &p.padding_inline_start,
        &p.padding_inline,
    ));
    decls.push(pad(
        "padding-inline-end",
        &p.padding_inline_end,
        &p.padding_inline,
    ));
    decls.push(pad(
        "padding-block-start",
        &p.padding_block_start,
        &p.padding_block,
    ));
    decls.push(pad(
        "padding-block-end",
        &p.padding_block_end,
        &p.padding_block,
    ));
    let a = attrs()
        .set("class", &cls("astryx-center", &p.class))
        .flag("data-inline", p.is_inline)
        .set("data-axis", or(&p.axis, "both"))
        .opt("style", &styles(&decls))
        .extra(&p.attrs);
    el("div", a, [Some(children)])
}

/// A responsive grid: as many columns of at least `min_width` px as fit.
#[derive(Clone, Debug, Default)]
pub struct GridAuto {
    pub min_width: u32,
    /// Caps the number of columns (0 = no cap).
    pub max: u32,
    /// "fill" (default, keeps empty tracks) or "fit" (collapses them).
    pub repeat: String,
}

/// Props for [`grid`]. Gaps take spacing steps.
#[derive(Clone, Debug, Default)]
pub struct GridProps {
    /// A fixed column count (0 = one column); `auto`, when set, wins.
    pub columns: u32,
    pub auto: Option<GridAuto>,
    pub width: String,
    pub height: String,
    pub max_width: String,
    pub min_height: String,
    pub gap: String,
    pub row_gap: String,
    pub column_gap: String,
    /// A fixed row height in px (0 = auto).
    pub row_height: u32,
    /// "start", "center", "end", "stretch".
    pub align: String,
    pub justify: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

fn grid_template(p: &GridProps) -> String {
    if let Some(auto) = &p.auto {
        let mode = if auto.repeat == "fit" {
            "auto-fit"
        } else {
            "auto-fill"
        };
        let min = format!("{}px", auto.min_width);
        if auto.max > 0 {
            let n = auto.max;
            let g = first(&[&p.column_gap, &p.gap]);
            let per = if g.is_empty() {
                format!("calc(100% / {n})")
            } else {
                format!("calc((100% - {} * {}) / {n})", n - 1, space_css(g))
            };
            return format!("repeat({mode}, minmax(min(100%, max({min}, {per})), 1fr))");
        }
        return format!("repeat({mode}, minmax(min(100%, {min}), 1fr))");
    }
    if p.columns > 0 {
        return format!("repeat({}, 1fr)", p.columns);
    }
    "1fr".into()
}

/// Lays out `children` in a CSS grid.
pub fn grid(p: &GridProps, children: Markup) -> Markup {
    let row_height = if p.row_height > 0 {
        format!("{}px", p.row_height)
    } else {
        String::new()
    };
    let mut decls = vec![
        decl("grid-template-columns", &grid_template(p)),
        decl("grid-auto-rows", &row_height),
        decl("gap", &space_css(&p.gap)),
        decl("row-gap", &space_css(&p.row_gap)),
        decl("column-gap", &space_css(&p.column_gap)),
        decl("align-items", &p.align),
        decl("justify-items", &p.justify),
    ];
    decls.extend(size_styles(
        &p.width,
        &p.height,
        &p.max_width,
        &p.min_height,
    ));
    let a = attrs()
        .set("class", &cls("astryx-grid", &p.class))
        .when(
            p.auto.is_none() && p.columns > 0,
            "data-columns",
            &p.columns.to_string(),
        )
        .opt("data-gap", &p.gap)
        .opt("data-align", &p.align)
        .opt("data-justify", &p.justify)
        .set("style", &styles(&decls))
        .extra(&p.attrs);
    el("div", a, [Some(children)])
}

/// Props for [`grid_span`].
#[derive(Clone, Debug, Default)]
pub struct GridSpanProps {
    /// Columns to span (0 = one); `is_full` spans the whole row.
    pub columns: u32,
    pub is_full: bool,
    /// Rows to span (0 = one).
    pub rows: u32,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Makes `children` span several grid tracks.
pub fn grid_span(p: &GridSpanProps, children: Markup) -> Markup {
    let column = if p.is_full {
        "1 / -1".to_string()
    } else if p.columns > 0 {
        format!("span {}", p.columns)
    } else {
        String::new()
    };
    let row = if p.rows > 0 {
        format!("span {}", p.rows)
    } else {
        String::new()
    };
    let a = attrs()
        .set("class", &cls("astryx-grid-span", &p.class))
        .opt(
            "style",
            &styles(&[decl("grid-column", &column), decl("grid-row", &row)]),
        )
        .extra(&p.attrs);
    el("div", a, [Some(children)])
}

/// Props for [`aspect_ratio`].
#[derive(Clone, Debug, Default)]
pub struct AspectRatioProps {
    /// Width divided by height, e.g. 16.0 / 9.0.
    pub ratio: f64,
    /// "rectangle" (default) or "ellipse".
    pub shape: String,
    /// How an img/video child fills the box: "cover", "contain", or "center" (any child).
    pub fit: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

/// Keeps `children` at a fixed width-to-height ratio.
pub fn aspect_ratio(p: &AspectRatioProps, children: Markup) -> Markup {
    let ratio = if p.ratio > 0.0 { p.ratio } else { 1.0 };
    let a = attrs()
        .set("class", &cls("astryx-aspect-ratio", &p.class))
        .set("data-shape", or(&p.shape, "rectangle"))
        .set("style", &format!("aspect-ratio: {ratio}"))
        .extra(&p.attrs);
    el(
        "div",
        a,
        [Some(el(
            "div",
            attrs()
                .set("class", "astryx-aspect-ratio-child")
                .opt("data-fit", &p.fit),
            [Some(children)],
        ))],
    )
}
