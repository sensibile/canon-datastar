//! The demo's Content page. Mirrors go/cmd/demo/content.templ.

use canon_datastar::*;
use maud::{Markup, html};
use std::time::{SystemTime, UNIX_EPOCH};

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

fn para(t: &str) -> Markup {
    html! { p.demo-panel { (t) } }
}

fn click(expr: &str) -> Vec<Attr> {
    vec![attr("data-on:click", expr)]
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as i64)
}

/// The upload bar, re-rendered by the /demo/progress SSE stream.
pub(crate) fn progress(value: f64) -> Markup {
    progress_bar(&ProgressBarProps {
        id: s("upload"),
        label: s("Uploading report.pdf"),
        value,
        has_value_label: true,
        marks: vec![ProgressBarMark {
            value: 50.0,
            label: s("Halfway"),
        }],
        attrs: vec![attr("id", "upload-bar")],
        ..Default::default()
    })
}

fn named(name: &str) -> AvatarProps {
    AvatarProps {
        name: s(name),
        ..Default::default()
    }
}

fn item(label: &str, text: &str) -> MetadataListItem {
    MetadataListItem {
        label: s(label),
        text: s(text),
        ..Default::default()
    }
}

pub(crate) fn page() -> Markup {
    let now = now_ms();
    let card_ = |p: CardProps, body: Markup| card(&p, body);
    html! {
        h1 { "Layout and content" }
        p.demo-lead { "Server-rendered markup; static/canon.js adds collapsing, dismissing, image fallbacks, keyboard models and Intl timestamps." }
        div data-signals="{details: true, removed: 0, filter: ''}" {
            section.demo-section {
                h2 { "Card, Section, Grid, Center, AspectRatio" }
                (grid(&GridProps { auto: Some(GridAuto { min_width: 200, ..Default::default() }), gap: s("4"), ..Default::default() }, html! {
                    (card_(CardProps::default(), html! {
                        (heading(&HeadingProps { level: 4, ..Default::default() }, html! { "Default card" }))
                        (section_(&SectionProps { variant: s("muted"), dividers: vec![s("top"), s("bottom")], ..Default::default() }, html! { "A full-bleed section" }))
                        (text(&TextProps { kind: s("supporting"), ..Default::default() }, html! { "Padding comes from the theme." }))
                    }))
                    (card_(CardProps { variant: s("blue"), elevation: s("med"), padding: s("6"), ..Default::default() }, html! { "Blue, elevated, padding 6" }))
                    (card_(CardProps { variant: s("muted"), min_height: s("120px"), ..Default::default() },
                        center(&CenterProps { height: s("100%"), ..Default::default() }, html! { "Centered" })))
                    (grid_span(&GridSpanProps { is_full: true, ..Default::default() },
                        card_(CardProps { variant: s("transparent"), ..Default::default() },
                            aspect_ratio(&AspectRatioProps { ratio: 21.0 / 9.0, fit: s("cover"), ..Default::default() },
                                html! { img src="/demo/photo.svg" alt="Gradient landscape"; }))))
                }))
            }
            (section("Avatar", html! {
                (avatar(&AvatarProps { size: s("lg"), status: Some(AvatarStatusDotProps { label: s("Online"), ..Default::default() }), ..named("Ada Lovelace") }))
                (avatar(&AvatarProps { src: s("/demo/missing.png"), fallback_src: s("/demo/photo.svg"), size: s("lg"), ..named("Grace Hopper") }))
                (avatar(&AvatarProps { src: s("/demo/missing.png"), size: s("lg"), shape: s("rounded"), ..named("Broken Image") }))
                (avatar(&AvatarProps { size: s("lg"), status: Some(AvatarStatusDotProps { variant: s("neutral"), label: s("Away"), ..Default::default() }), ..Default::default() }))
                (avatar(&AvatarProps { size: s("xl"), shape: s("rounded"),
                    status: Some(AvatarStatusDotProps { variant: s("error"), label: s("Busy"), ..Default::default() }),
                    action: Some(click("canon.toast({body: 'Opened profile'})")), ..named("Linus") }))
                (avatar_group(&AvatarGroupProps {
                    id: s("team"),
                    avatars: vec![
                        AvatarProps { href: s("#ada"), has_no_tooltip: true, ..named("Ada Lovelace") },
                        AvatarProps { href: s("#grace"), has_no_tooltip: true, ..named("Grace Hopper") },
                        AvatarProps { href: s("#alan"), has_no_tooltip: true, ..named("Alan Turing") },
                    ],
                    overflow: Some(AvatarGroupOverflowProps { count: 12, action: Some(click("canon.toast({body: '12 more people'})")), ..Default::default() }),
                    ..Default::default()
                }))
            }))
            (section("Token", html! {
                (token(&TokenProps { label: s("Draft"), ..Default::default() }))
                (token(&TokenProps { label: s("Docs"), href: s("#docs"), color: s("blue"), ..Default::default() }))
                (token(&TokenProps { label: s("Filter: open"), color: s("green"), action: Some(click("$filter = 'open'")),
                    remove_attrs: click("$removed++"), ..Default::default() }))
                (token(&TokenProps { label: s("Disabled"), is_removable: true, is_disabled: true, ..Default::default() }))
                span.demo-note data-text="'filter: ' + $filter + ', removed: ' + $removed" {}
            }))
            section.demo-section {
                h2 { "Banner" }
                div.demo-stack {
                    (banner(&BannerProps { status: s("info"), title: s("A new version is available"), description: s("Reload to update."), is_dismissable: true, ..Default::default() }))
                    (banner(&BannerProps { id: s("quota"), status: s("warning"), title: s("Storage almost full"), signal: s("details"), is_open: true,
                        content: Some(para("You have used 9.1 GB of 10 GB. The body's open state is the $details signal.")), ..Default::default() }))
                    (banner(&BannerProps { status: s("error"), title: s("Deploy failed"), container: s("section"), elevation: s("low"),
                        end_content: Some(button(&ButtonProps { label: s("Retry"), size: s("sm"), ..Default::default() })), is_dismissable: true, ..Default::default() }))
                    span.demo-note data-text="'details open: ' + $details" {}
                }
            }
            section.demo-section {
                h2 { "Collapsible" }
                div.demo-stack {
                    (collapsible(&CollapsibleProps { trigger: s("Standalone, open by default"), ..Default::default() }, html! { "Content shows when open." }))
                    (collapsible_group(&CollapsibleGroupProps {
                        value: vec![s("a")],
                        has_dividers: true,
                        items: vec![
                            CollapsibleProps { trigger: s("Accordion item A"), value: s("a"), content: Some(para("Opening B closes A: this group is \"single\".")), ..Default::default() },
                            CollapsibleProps { trigger: s("Accordion item B"), value: s("b"), content: Some(para("Only one item is open at a time.")), ..Default::default() },
                            CollapsibleProps { trigger: s("Disabled"), value: s("c"), is_disabled: true, ..Default::default() },
                        ],
                        ..Default::default()
                    }))
                }
            }
            section.demo-section {
                h2 { "ProgressBar" }
                div.demo-stack {
                    (progress(20.0))
                    (progress_bar(&ProgressBarProps { label: s("Syncing"), is_indeterminate: true, variant: s("success"), ..Default::default() }))
                    div.demo-row {
                        (button(&ButtonProps { label: s("Upload (SSE)"), attrs: click("@post('/demo/progress')"), ..Default::default() }))
                    }
                }
            }
            section.demo-section {
                h2 { "MetadataList" }
                (metadata_list(&MetadataListProps {
                    id: s("file-meta"),
                    title: s("File details"),
                    max_num_of_items: 3,
                    items: vec![
                        item("Owner", "Ada Lovelace"),
                        MetadataListItem { label: s("Status"), value: Some(badge(&BadgeProps { label: s("Active"), variant: s("success"), ..Default::default() })), ..Default::default() },
                        item("Created", "March 4, 2026"),
                        item("Size", "3.2 MB"),
                        item("Region", "ap-northeast-2"),
                    ],
                    ..Default::default()
                }))
            }
            (section("Thumbnail", html! {
                (thumbnail(&ThumbnailProps { src: s("/demo/photo.svg"), alt: s("Gradient"), label: s("photo.svg"), tooltip_id: s("thumb-1"),
                    action: Some(click("canon.toast({body: 'Opened photo.svg'})")), remove_attrs: click("$removed++"), ..Default::default() }))
                (thumbnail(&ThumbnailProps { src: s("/demo/photo.svg"), is_loading: true, ..Default::default() }))
                (thumbnail(&ThumbnailProps { is_loading: true, ..Default::default() }))
                (thumbnail(&ThumbnailProps { src: s("/demo/missing.png"), label: s("missing.png"), tooltip_id: s("thumb-2"), ..Default::default() }))
            }))
            (section("Timestamp", html! {
                (timestamp(&TimestampProps { id: s("ts-1"), value_ms: now - 90_000, is_live: true, ..Default::default() }))
                (timestamp(&TimestampProps { id: s("ts-2"), value_ms: now - 26 * 3_600_000, ..Default::default() }))
                (timestamp(&TimestampProps { id: s("ts-3"), value_ms: 1_766_601_000_000, format: s("date_long"),
                    tooltip_entries: vec![
                        TimestampTooltipEntry { timezone_id: s("Asia/Seoul"), format: s("date_time"), label: s("Seoul") },
                        TimestampTooltipEntry { timezone_id: s("UTC"), format: s("system_date_time"), label: s("UTC") },
                    ],
                    ..Default::default() }))
                (timestamp(&TimestampProps { id: s("ts-4"), value_ms: now - 3 * 3_600_000, format: s("relative_short"), locale: s("ko-KR"), ..Default::default() }))
            }))
            (section("Kbd, Code, StatusDot, Skeleton", html! {
                (kbd(&KbdProps { keys: s("mod+k"), ..Default::default() }))
                (kbd(&KbdProps { keys: s("shift+enter"), ..Default::default() }))
                (code(&CodeProps::default(), html! { "cargo run --example demo" }))
                (status_dot(&StatusDotProps { variant: s("success"), label: s("Online"), tooltip: s("Online"), is_pulsing: true, ..Default::default() }))
                (status_dot(&StatusDotProps { variant: s("error"), label: s("Offline"), ..Default::default() }))
                (skeleton(&SkeletonProps { width: s("120px"), height: s("16px"), ..Default::default() }))
                (skeleton(&SkeletonProps { width: s("32px"), height: s("32px"), radius: s("rounded"), index: 1, ..Default::default() }))
            }))
            section.demo-section {
                h2 { "Blockquote and EmptyState" }
                (blockquote(&BlockquoteProps { cite: s("Ada Lovelace"), ..Default::default() },
                    html! { "The Analytical Engine weaves algebraic patterns just as the Jacquard loom weaves flowers and leaves." }))
                (card_(CardProps::default(), empty_state(&EmptyStateProps {
                    title: s("No projects yet"),
                    description: s("Create a project to get started."),
                    icon: Some(icon(&IconProps { name: s("search"), size: s("lg"), color: s("secondary"), ..Default::default() })),
                    actions: Some(button(&ButtonProps { label: s("New project"), variant: s("primary"), ..Default::default() })),
                    ..Default::default()
                })))
            }
        }
    }
}

fn section_(p: &SectionProps, children: Markup) -> Markup {
    canon_datastar::section(p, children)
}
