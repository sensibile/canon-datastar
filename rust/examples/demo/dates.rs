//! The demo's Dates page. Mirrors go/cmd/demo/dates.templ.

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

fn calendars() -> Markup {
    html! {
        div.demo-grid {
            (calendar(&CalendarProps { id: s("cal-day"), value: s("2026-03-14"), has_week_numbers: true,
                value_attrs: bind("day"), ..Default::default() }))
            (calendar(&CalendarProps { id: s("cal-range"), mode: s("range"), number_of_months: 2, start: s("2026-03-10"),
                end: s("2026-03-20"), start_value_attrs: bind("from"), end_value_attrs: bind("to"), ..Default::default() }))
        }
        p.demo-panel { "day: " span data-text="$day" {} " · range: " span data-text="$from + ' → ' + $to" {} " · " button type="button" data-on:click="$day = '2026-04-01'" { "Set day" } }
    }
}

fn bounds() -> Markup {
    html! {
        div.demo-grid {
            (calendar(&CalendarProps { id: s("cal-trip"), mode: s("range"), min: s("2026-03-05"), max: s("2026-05-20"),
                disabled_days_of_week: vec![0, 6], min_range_span: 3, max_range_span: 10, focus_date: s("2026-03-01"),
                start_value_attrs: bind("trip"), end_value_attrs: bind("trip-end"), ..Default::default() }))
            (calendar(&CalendarProps { id: s("cal-de"), locale: s("de-DE"), week_starts_on: 1, no_outside_days: true,
                has_variable_row_count: true, focus_date: s("2026-02-01"),
                disabled_dates: vec![s("2026-02-16"), s("2026-02-17")], ..Default::default() }))
            (calendar(&CalendarProps { id: s("cal-ko"), locale: s("ko-KR"), focus_date: s("2026-09-01"), ..Default::default() }))
        }
        p.demo-panel { "trip: " span data-text="$trip + ' → ' + $tripEnd" {} " (weekdays only, 3–10 days, Mar 5 – May 20)" }
    }
}

pub(crate) fn page() -> Markup {
    html! {
        h1 { "Dates" }
        p.demo-lead { "The server renders each calendar's frame and value; static/canon.js draws the months with the browser's clock and Intl names." }
        div data-signals="{day: '2026-03-14', from: '2026-03-10', to: '2026-03-20', trip: '', tripEnd: ''}" {
            (section("Calendar", calendars()))
            (section("Bounds and locales", bounds()))
        }
    }
}
