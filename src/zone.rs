//! The farm's time zone (yaks-ac6a): the one clock behind every calendar date
//! yaks reads or writes.
//!
//! `timezone: America/New_York` in `.yaks/config.yaml` names an IANA zone, and
//! the tz database is compiled in (`chrono-tz`), so the same farm reads the
//! same dates on every machine. Without it the farm keeps the machine's zone.
//! Farm-wide only: a family spread over zones still works, because a datetime
//! `due:` keeps the offset it was given.
//!
//! A [`Zone`] is a small `Copy` value that callers pass in, so nothing here
//! reads the environment but [`Zone::Machine`]'s own offset lookup.

use chrono::{
    DateTime, Duration, FixedOffset, Local, LocalResult, NaiveDate, NaiveDateTime, NaiveTime,
    Offset, TimeZone, Utc,
};
use chrono_tz::Tz;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Zone {
    /// The machine's own zone (what yaks used before there was a `timezone:`).
    #[default]
    Machine,
    /// An IANA zone from the compiled-in database.
    Named(Tz),
    /// A fixed offset: no daylight saving. For tests and docshots.
    #[cfg_attr(not(test), allow(dead_code))]
    Fixed(FixedOffset),
}

impl Zone {
    /// An IANA name (`America/New_York`, `UTC`); `None` for one the database
    /// does not know.
    pub fn parse(name: &str) -> Option<Zone> {
        name.trim().parse::<Tz>().ok().map(Zone::Named)
    }

    /// The instant `t` as this zone's wall clock, with the offset in force then.
    pub fn at(self, t: DateTime<Utc>) -> DateTime<FixedOffset> {
        match self {
            Zone::Machine => t.with_timezone(&Local).fixed_offset(),
            Zone::Named(tz) => t.with_timezone(&tz).fixed_offset(),
            Zone::Fixed(off) => t.with_timezone(&off),
        }
    }

    /// The wall-clock time `local` in this zone, as an instant. A time that
    /// happens twice (clocks went back) is the first of them; one that never
    /// happens (clocks went forward) is moved on by the gap, as calendars do.
    /// `None` only past the end of chrono's range.
    pub fn resolve(self, local: NaiveDateTime) -> Option<DateTime<FixedOffset>> {
        match self {
            Zone::Machine => resolve_in(&Local, local),
            Zone::Named(tz) => resolve_in(&tz, local),
            Zone::Fixed(off) => off.from_local_datetime(&local).single(),
        }
    }

    /// The calendar day it is at `t` in this zone.
    pub fn date_of(self, t: DateTime<Utc>) -> NaiveDate {
        self.at(t).date_naive()
    }

    /// Today on the calendar a person reads, which is what `+2d` counts from.
    pub fn today(self) -> NaiveDate {
        self.date_of(Utc::now())
    }

    /// When `day` begins in this zone.
    pub fn midnight(self, day: NaiveDate) -> Option<DateTime<FixedOffset>> {
        self.resolve(day.and_time(NaiveTime::MIN))
    }
}

fn resolve_in<T: TimeZone>(tz: &T, local: NaiveDateTime) -> Option<DateTime<FixedOffset>> {
    match tz.from_local_datetime(&local) {
        LocalResult::Single(t) | LocalResult::Ambiguous(t, _) => Some(t.fixed_offset()),
        LocalResult::None => {
            // In a gap: read it with the offset in force before the gap, which
            // lands the same number of minutes past it.
            let before = tz
                .offset_from_utc_datetime(&local.checked_sub_signed(Duration::days(1))?)
                .fix();
            let utc =
                local.checked_sub_signed(Duration::seconds(before.local_minus_utc().into()))?;
            Some(tz.from_utc_datetime(&utc).fixed_offset())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ny() -> Zone {
        Zone::parse("America/New_York").unwrap()
    }

    fn local(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").unwrap()
    }

    #[test]
    fn names_are_iana_and_unknown_ones_refused() {
        assert!(Zone::parse("America/New_York").is_some());
        assert!(Zone::parse(" Europe/Berlin ").is_some());
        assert!(Zone::parse("UTC").is_some());
        for bad in ["", "Mars/Olympus", "new york"] {
            assert!(Zone::parse(bad).is_none(), "{bad}");
        }
    }

    #[test]
    fn the_offset_follows_daylight_saving() {
        let off = |s: &str| ny().resolve(local(s)).unwrap().offset().local_minus_utc();
        assert_eq!(off("2026-11-01T19:30:00"), -5 * 3600, "after the fall back");
        assert_eq!(off("2026-10-31T19:30:00"), -4 * 3600, "before it");
    }

    #[test]
    fn a_repeated_hour_is_the_first_and_a_skipped_one_moves_on() {
        // Nov 1 01:30 happens twice in New York: take the first (EDT).
        let t = ny().resolve(local("2026-11-01T01:30:00")).unwrap();
        assert_eq!(t.to_rfc3339(), "2026-11-01T01:30:00-04:00");
        // Mar 8 02:30 never happens: it becomes 03:30 EDT.
        let t = ny().resolve(local("2026-03-08T02:30:00")).unwrap();
        assert_eq!(t.to_rfc3339(), "2026-03-08T03:30:00-04:00");
    }

    #[test]
    fn a_day_is_the_zones_not_utcs() {
        // 23:30 EST on Nov 7 is already Nov 8 in UTC.
        let t = Utc.with_ymd_and_hms(2026, 11, 8, 4, 30, 0).unwrap();
        assert_eq!(ny().date_of(t).to_string(), "2026-11-07");
        assert_eq!(
            ny().midnight(t.date_naive()).unwrap().to_rfc3339(),
            "2026-11-08T00:00:00-05:00"
        );
    }
}
