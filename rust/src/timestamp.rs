//! Port of Astryx `Timestamp` (packages/core/src/Timestamp), MIT, Meta Platforms, Inc.
//! The server renders a `<time datetime>` with a UTC date-and-time fallback; static/canon.js
//! formats it in the viewer's zone and locale with Intl (relative, absolute or "auto"), keeps live
//! relative times ticking and fills the details tooltip. Not ported: the hover card with copy
//! buttons, non-Gregorian calendars.

use crate::html::{attrs, cls, el, or, text};
use crate::misc::{VisuallyHiddenProps, visually_hidden};
use crate::text::{TextProps, text as text_el};
use crate::tooltip::{TooltipProps, tooltip};
use crate::{Attr, attr};
use maud::Markup;

/// One line of the details tooltip.
#[derive(Clone, Debug, Default)]
pub struct TimestampTooltipEntry {
    /// An IANA zone, or "local" / "" for the viewer's zone.
    pub timezone_id: String,
    /// "full" (default), "date", "date_long", "date_weekday", "date_time", "time",
    /// "system_date", "system_date_time", "system_time", "unix_seconds".
    pub format: String,
    pub label: String,
}

/// Props for [`timestamp`].
#[derive(Clone, Debug, Default)]
pub struct TimestampProps {
    /// Id of the details tooltip. Defaults to "timestamp-" + the Unix milliseconds.
    pub id: String,
    /// The instant as Unix milliseconds.
    pub value_ms: i64,
    /// "auto" (default: relative within `auto_threshold`, else date_time), "relative",
    /// "relative_short", or one of the absolute formats listed on [`TimestampTooltipEntry`].
    pub format: String,
    /// IANA zone for absolute formats (the viewer's zone by default).
    pub timezone_id: String,
    /// Seconds within which "auto" is relative (7 days when 0).
    pub auto_threshold: u32,
    pub has_no_tooltip: bool,
    pub tooltip_entries: Vec<TimestampTooltipEntry>,
    pub is_timezone_shown: bool,
    /// Keeps a relative time up to date.
    pub is_live: bool,
    /// A BCP 47 tag ("en-US" by default).
    pub locale: String,
    /// Text styling ("supporting", secondary by default).
    pub kind: String,
    pub size: String,
    pub color: String,
    pub weight: String,
    pub class: String,
    pub attrs: Vec<Attr>,
}

struct Civil {
    year: i64,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    milli: u32,
}

/// Splits Unix milliseconds into a UTC date and time (proleptic Gregorian).
fn civil(ms: i64) -> Civil {
    let secs = ms.div_euclid(1000);
    let milli = ms.rem_euclid(1000) as u32;
    let days = secs.div_euclid(86400);
    let sod = secs.rem_euclid(86400) as u32;
    // Howard Hinnant's civil_from_days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = yoe + era * 400 + i64::from(month <= 2);
    Civil {
        year,
        month,
        day,
        hour: sod / 3600,
        minute: sod / 60 % 60,
        second: sod % 60,
        milli,
    }
}

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

fn is_relative(f: &str) -> bool {
    f == "relative" || f == "relative_short"
}

/// Renders a date and time.
pub fn timestamp(p: &TimestampProps) -> Markup {
    let format = or(&p.format, "auto");
    let needs_clock = is_relative(format) || format == "auto";
    let effective = if needs_clock { "date_time" } else { format };
    let c = civil(p.value_ms);
    let threshold = if p.auto_threshold == 0 {
        7 * 86400
    } else {
        p.auto_threshold
    };
    let has_tip = !p.has_no_tooltip && (needs_clock || !p.tooltip_entries.is_empty());
    let iso = format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        c.year, c.month, c.day, c.hour, c.minute, c.second, c.milli
    );
    let hour12 = match c.hour % 12 {
        0 => 12,
        h => h,
    };
    let fallback = format!(
        "{} {}, {}, {}:{:02} {} UTC",
        MONTHS[c.month as usize - 1],
        c.day,
        c.year,
        hour12,
        c.minute,
        if c.hour < 12 { "AM" } else { "PM" }
    );
    let ta = attrs()
        .set("datetime", &iso)
        .set("class", "astryx-timestamp-time")
        .set("data-format", format)
        .set("data-locale", or(&p.locale, "en-US"))
        .opt("data-timezone", &p.timezone_id)
        .when(
            format == "auto" && p.auto_threshold != 0,
            "data-threshold",
            &threshold.to_string(),
        )
        .flag("data-timezone-shown", p.is_timezone_shown)
        .flag("data-live", p.is_live)
        .when(has_tip, "tabindex", "0");
    let time_el = el("time", ta, [Some(text(&fallback))]);

    let body = if has_tip {
        let default_entry = [TimestampTooltipEntry::default()];
        let entries: &[TimestampTooltipEntry] = if p.tooltip_entries.is_empty() {
            &default_entry
        } else {
            &p.tooltip_entries
        };
        let labelled = entries.iter().any(|e| !e.label.is_empty());
        let zone = || {
            visually_hidden(
                &VisuallyHiddenProps {
                    attrs: vec![attr("data-zone-name", "")],
                    ..Default::default()
                },
                text(""),
            )
        };
        let rows = entries.iter().map(|e| {
            let dt = if !labelled {
                el(
                    "dt",
                    attrs().set("class", "astryx-timestamp-sr-dt"),
                    [Some(zone())],
                )
            } else if !e.label.is_empty() {
                el(
                    "dt",
                    attrs().set("class", "astryx-timestamp-label"),
                    [Some(text(&e.label))],
                )
            } else {
                el(
                    "dt",
                    attrs().set("class", "astryx-timestamp-label"),
                    [Some(zone())],
                )
            };
            Some(el(
                "div",
                attrs()
                    .set("class", "astryx-timestamp-row")
                    .opt("data-timezone", &e.timezone_id)
                    .opt("data-format", &e.format),
                [
                    Some(dt),
                    Some(el("dd", attrs().set("class", "astryx-timestamp-value"), [])),
                ],
            ))
        });
        let details = el(
            "dl",
            attrs()
                .set("class", "astryx-timestamp-details")
                .flag("data-labelled", labelled)
                .flag("data-default", p.tooltip_entries.is_empty()),
            rows,
        );
        let id = if p.id.is_empty() {
            format!("timestamp-{}", p.value_ms)
        } else {
            p.id.clone()
        };
        tooltip(
            &TooltipProps {
                id,
                content_markup: Some(details),
                focus_trigger: "always".into(),
                hover_indication: "never".into(),
                ..Default::default()
            },
            time_el,
        )
    } else {
        time_el
    };
    let mut a = vec![attr("data-timestamp-format", effective)];
    a.extend(p.attrs.iter().cloned());
    text_el(
        &TextProps {
            kind: or(&p.kind, "supporting").into(),
            size: p.size.clone(),
            color: or(&p.color, "secondary").into(),
            weight: p.weight.clone(),
            class: cls("astryx-timestamp", &p.class),
            attrs: a,
            ..Default::default()
        },
        body,
    )
}
