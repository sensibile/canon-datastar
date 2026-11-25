//! Calendar. Ported from Astryx `Calendar` (packages/core/src/Calendar), MIT, Meta Platforms, Inc.
//! A month grid for picking a date (mode "single") or a range (mode "range"). Dates are ISO
//! YYYY-MM-DD strings. The server renders the frame, the settings and the value; static/canon.js
//! draws the month grids, since today and the month and day names belong to the browser. Until
//! then the grid area keeps its space, hidden.
//! The value lives in hidden inputs: one for a single date, start and end for a range. Bind
//! signals with `value_attrs` / `start_value_attrs` / `end_value_attrs`, e.g.
//! `("data-bind:day", "")`. Moving the shown months fires a bubbling "astryx-focus-date" event
//! with detail {focusDate}.
//! Gregorian only. Not ported: the `calendar` prop, dateConstraints functions (use
//! `disabled_dates` and `disabled_days_of_week`), the locale provider (pass `locale`).

use crate::Attr;
use crate::button::{ButtonProps, button};
use crate::html::{attrs, cls, el, or, void};
use crate::misc::{VisuallyHiddenProps, visually_hidden};
use maud::Markup;

/// Props for [`calendar`].
#[derive(Clone, Debug, Default)]
pub struct CalendarProps {
    /// On the root element.
    pub id: String,
    /// "single" (default) or "range".
    pub mode: String,
    /// The selected date in single mode.
    pub value: String,
    /// The selected range in range mode (both or neither).
    pub start: String,
    pub end: String,
    /// Months side by side: 1 (default) or 2.
    pub number_of_months: u8,
    pub min: String,
    pub max: String,
    /// Dates that can't be picked.
    pub disabled_dates: Vec<String>,
    /// Days of the week (0 Sunday … 6 Saturday) that can't be picked.
    pub disabled_days_of_week: Vec<u8>,
    /// Bounds on a range's length in days, counting both ends; 0 means none.
    pub max_range_span: u32,
    pub min_range_span: u32,
    /// A day in the first month shown. Default: the value, else today clamped into min/max.
    pub focus_date: String,
    /// Hides days of adjacent months (shown dimmed by default).
    pub no_outside_days: bool,
    pub has_week_numbers: bool,
    pub has_variable_row_count: bool,
    /// 0 (Sunday, default) to 6.
    pub week_starts_on: u8,
    /// Default "en-US".
    pub locale: String,
    /// `name` (single), `start_name` and `end_name` (range) and `form` go on the hidden inputs.
    pub name: String,
    pub start_name: String,
    pub end_name: String,
    pub form: String,
    pub class: String,
    pub attrs: Vec<Attr>,
    pub value_attrs: Vec<Attr>,
    pub start_value_attrs: Vec<Attr>,
    pub end_value_attrs: Vec<Attr>,
}

fn hidden(class: &str, name: &str, form: &str, value: &str, extra: &[Attr]) -> Markup {
    void(
        "input",
        attrs()
            .set("type", "hidden")
            .set("class", class)
            .opt("name", name)
            .opt("form", form)
            .set("value", value)
            .extra(extra),
    )
}

fn nav(dir: &str, label: &str, icon: &str) -> Markup {
    button(&ButtonProps {
        label: label.into(),
        variant: "ghost".into(),
        icon: icon.into(),
        is_icon_only: true,
        is_disabled: true,
        class: "astryx-calendar-nav".into(),
        attrs: vec![("data-nav".into(), dir.into())],
        ..Default::default()
    })
}

/// Renders the calendar frame; canon.js draws the months.
pub fn calendar(p: &CalendarProps) -> Markup {
    let mode = if p.mode == "range" { "range" } else { "single" };
    let week = if p.week_starts_on > 6 {
        0
    } else {
        p.week_starts_on
    };
    let header = el(
        "div",
        attrs().set("class", "astryx-calendar-header"),
        [
            Some(nav("prev", "Previous month", "chevronLeft")),
            Some(el(
                "span",
                attrs().set("class", "astryx-calendar-label"),
                [None],
            )),
            Some(nav("next", "Next month", "chevronRight")),
        ],
    );
    let grids = el(
        "div",
        attrs()
            .set("class", "astryx-calendar-months")
            .flag("data-pending", true)
            .set("aria-hidden", "true")
            .flag("inert", true),
        [None],
    );
    let live = visually_hidden(
        &VisuallyHiddenProps::default(),
        el(
            "span",
            attrs()
                .set("class", "astryx-calendar-live")
                .set("aria-live", "polite")
                .set("aria-atomic", "true"),
            [None],
        ),
    );
    let mut children = vec![Some(header), Some(grids), Some(live)];
    if mode == "single" {
        children.push(Some(hidden(
            "astryx-calendar-value",
            &p.name,
            &p.form,
            &p.value,
            &p.value_attrs,
        )));
    } else {
        let both = !p.start.is_empty() && !p.end.is_empty();
        let (start, end) = if both {
            (p.start.as_str(), p.end.as_str())
        } else {
            ("", "")
        };
        children.push(Some(hidden(
            "astryx-calendar-start",
            &p.start_name,
            &p.form,
            start,
            &p.start_value_attrs,
        )));
        children.push(Some(hidden(
            "astryx-calendar-end",
            &p.end_name,
            &p.form,
            end,
            &p.end_value_attrs,
        )));
    }
    let days = p
        .disabled_days_of_week
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(" ");
    let a = attrs()
        .opt("id", &p.id)
        .set("class", &cls("astryx-calendar", &p.class))
        .set("data-mode", mode)
        .when(p.number_of_months == 2, "data-months", "2")
        .set("data-locale", or(&p.locale, "en-US"))
        .when(week != 0, "data-week-starts-on", &week.to_string())
        .opt("data-min", &p.min)
        .opt("data-max", &p.max)
        .opt("data-focus-date", &p.focus_date)
        .opt("data-disabled-dates", &p.disabled_dates.join(" "))
        .opt("data-disabled-days", &days)
        .when(
            p.max_range_span > 0,
            "data-max-span",
            &p.max_range_span.to_string(),
        )
        .when(
            p.min_range_span > 0,
            "data-min-span",
            &p.min_range_span.to_string(),
        )
        .flag("data-no-outside-days", p.no_outside_days)
        .flag("data-week-numbers", p.has_week_numbers)
        .flag("data-variable-rows", p.has_variable_row_count)
        .extra(&p.attrs);
    el("div", a, children)
}
