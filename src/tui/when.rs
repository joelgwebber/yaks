//! Human-friendly timestamps (yaks-ef82): "5m ago", "yesterday 14:05 UTC",
//! "Oct 5, 2026 22:15 +02:00". A pure formatter over an injected "now" and
//! timezone, so snapshots and docshots stay deterministic; [`Clock`] is the
//! injectable source of both. Reused by the detail Created/Updated fields and
//! (later) comment headers.

use chrono::{DateTime, Datelike, FixedOffset, Local, Offset, TimeZone, Utc};

/// Where "now" and the display timezone come from.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Clock {
    /// The real clock and the machine's local timezone (read on each call, so
    /// "5m ago" stays current and DST is honoured per timestamp).
    System,
    /// A fixed instant and offset, for tests and docshots.
    #[cfg_attr(not(test), allow(dead_code))]
    Fixed(DateTime<Utc>, FixedOffset),
}

impl Clock {
    /// The clock an [`App`](super::App) starts with: the system clock, except
    /// under `cargo test`, where it is pinned so frames never depend on today.
    pub(crate) fn app_default() -> Clock {
        #[cfg(test)]
        {
            Clock::test_default()
        }
        #[cfg(not(test))]
        {
            Clock::System
        }
    }

    /// 2026-09-10 12:00 UTC, displayed in UTC.
    #[cfg(test)]
    pub(crate) fn test_default() -> Clock {
        Clock::Fixed(
            Utc.with_ymd_and_hms(2026, 9, 10, 12, 0, 0).unwrap(),
            FixedOffset::east_opt(0).unwrap(),
        )
    }

    /// Render the RFC 3339 timestamp `iso` relative to this clock; input that
    /// does not parse passes through unchanged.
    pub(crate) fn format(&self, iso: &str) -> String {
        match *self {
            Clock::System => format_in(iso, Utc::now(), &Local),
            Clock::Fixed(now, off) => format_in(iso, now, &off),
        }
    }
}

/// Format `iso` as seen from `now` in timezone `tz`:
///
/// | age | form |
/// |---|---|
/// | under 1 min (or up to 1 min in the future) | `just now` |
/// | under 1 h | `Nm ago` |
/// | under 24 h | `Nh ago` |
/// | the previous calendar day | `yesterday HH:MM <zone>` |
/// | 2–6 calendar days back | `Weekday HH:MM <zone>` |
/// | anything older, or further in the future | `Oct 5, 2026 22:15 <zone>` |
///
/// `<zone>` is `UTC` or a `+HH:MM` offset. Calendar days are those of `tz`.
pub(crate) fn format_in<Tz: TimeZone>(iso: &str, now: DateTime<Utc>, tz: &Tz) -> String
where
    Tz::Offset: std::fmt::Display,
{
    let Ok(ts) = DateTime::parse_from_rfc3339(iso) else {
        return iso.to_string();
    };
    let ts = ts.with_timezone(&Utc);
    let secs = now.signed_duration_since(ts).num_seconds();
    if secs < 60 && secs > -60 {
        return "just now".into();
    }
    if (0..3600).contains(&secs) {
        return format!("{}m ago", secs / 60);
    }
    if (0..86_400).contains(&secs) {
        return format!("{}h ago", secs / 3600);
    }
    let local = ts.with_timezone(tz);
    let off = local.offset().fix();
    let zone = if off.local_minus_utc() == 0 {
        "UTC".to_string()
    } else {
        off.to_string()
    };
    let days_back = (now.with_timezone(tz).date_naive().num_days_from_ce()
        - local.date_naive().num_days_from_ce()) as i64;
    let clock = local.format("%H:%M");
    match days_back {
        1 if secs > 0 => format!("yesterday {clock} {zone}"),
        2..=6 if secs > 0 => format!("{} {clock} {zone}", local.format("%A")),
        _ => format!("{} {zone}", local.format("%b %-d, %Y %H:%M")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(h: i32) -> FixedOffset {
        FixedOffset::east_opt(h * 3600).unwrap()
    }

    // Thu 2026-10-15 12:00 UTC.
    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 15, 12, 0, 0).unwrap()
    }

    fn f(iso: &str) -> String {
        format_in(iso, now(), &at(0))
    }

    #[test]
    fn rules_table() {
        for (iso, want) in [
            ("2026-10-15T12:00:00Z", "just now"),
            ("2026-10-15T11:59:01Z", "just now"),
            ("2026-10-15T12:00:30Z", "just now"), // slight clock skew
            ("2026-10-15T11:59:00Z", "1m ago"),
            ("2026-10-15T11:15:00Z", "45m ago"),
            ("2026-10-15T11:00:01Z", "59m ago"),
            ("2026-10-15T11:00:00Z", "1h ago"),
            ("2026-10-15T00:30:00Z", "11h ago"),
            ("2026-10-14T12:00:01Z", "23h ago"), // under 24 h wins over "yesterday"
            ("2026-10-14T12:00:00Z", "yesterday 12:00 UTC"),
            ("2026-10-14T01:05:00Z", "yesterday 01:05 UTC"),
            ("2026-10-13T23:59:00Z", "Tuesday 23:59 UTC"),
            ("2026-10-09T08:00:00Z", "Friday 08:00 UTC"), // 6 days back
            ("2026-10-08T23:59:00Z", "Oct 8, 2026 23:59 UTC"), // 7 days back
            ("2025-12-31T19:00:00Z", "Dec 31, 2025 19:00 UTC"),
            ("2026-10-15T12:05:00Z", "Oct 15, 2026 12:05 UTC"), // future: absolute
            ("not-a-date", "not-a-date"),
            ("", ""),
        ] {
            assert_eq!(f(iso), want, "{iso}");
        }
    }

    #[test]
    fn calendar_days_follow_the_offset() {
        // 23:30 UTC on the 14th is 01:30 on the 15th at +02:00: today, but
        // 12:00Z is 14:00 local, so 12h30 ago reads as relative anyway.
        assert_eq!(format_in("2026-10-14T23:30:00Z", now(), &at(2)), "12h ago");
        // 2026-10-13T22:30Z is Oct 14 00:30 at +02:00: yesterday locally,
        // though only the 13th in UTC (Tuesday vs yesterday).
        assert_eq!(
            format_in("2026-10-13T22:30:00Z", now(), &at(2)),
            "yesterday 00:30 +02:00"
        );
        assert_eq!(f("2026-10-13T22:30:00Z"), "Tuesday 22:30 UTC");
        // Negative offsets label with their sign.
        assert_eq!(
            format_in("2026-09-01T10:00:00Z", now(), &at(-5)),
            "Sep 1, 2026 05:00 -05:00"
        );
        // An offset in the input is honoured, not re-labelled.
        assert_eq!(f("2026-10-14T14:00:00+02:00"), "yesterday 12:00 UTC");
    }

    #[test]
    fn clock_fixed_and_system() {
        let c = Clock::Fixed(now(), at(0));
        assert_eq!(c.format("2026-10-15T11:00:00Z"), "1h ago");
        assert_eq!(c.format("bogus"), "bogus");
        // The system clock is only smoke-tested: a timestamp from "now" is fresh.
        assert_eq!(Clock::System.format(&Utc::now().to_rfc3339()), "just now");
    }
}
