//! Deterministic season skeleton: which phase each week falls in and how many
//! hours to aim for. No LLM involved. Later, the LLM fills in the actual
//! sessions *within* these constraints, so the overall structure never depends
//! on a model's mood.
//!
//! Weeks run Monday-Sunday. "Weeks to race" counts back from race week (0).
//!
//! | Phase | Weeks to race        | Purpose                                  |
//! |-------|----------------------|------------------------------------------|
//! | Base  | 17 and up            | aerobic foundation, technique, consistency |
//! | Build | 5 to 16 (12 weeks)   | race-specific work, longer brick sessions |
//! | Peak  | 2 to 4 (3 weeks)     | highest load, race-pace rehearsal         |
//! | Taper | 0 to 1 (2 weeks)     | shed fatigue, arrive fresh                |
//!
//! Outside peak and taper, every 4th week is a recovery week at 70% hours.
//! Taper weeks run at 60% (week before) and 40% (race week, excluding the race).

use std::fmt;

use chrono::{Datelike, Duration, NaiveDate};
use serde::Serialize;

use crate::config::Config;

const TAPER_WEEKS: i64 = 2;
const PEAK_WEEKS: i64 = 3;
const BUILD_WEEKS: i64 = 12;
const RECOVERY_EVERY: usize = 4;
const RECOVERY_FACTOR: f64 = 0.7;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Base,
    Build,
    Peak,
    Taper,
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Phase::Base => "base",
            Phase::Build => "build",
            Phase::Peak => "peak",
            Phase::Taper => "taper",
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PlannedWeek {
    /// Monday of the week.
    pub week_start: NaiveDate,
    /// 0 = race week.
    pub weeks_to_race: i64,
    pub phase: Phase,
    pub recovery: bool,
    pub target_hours: f64,
}

#[derive(Debug, Serialize)]
pub struct Plan {
    pub race: String,
    pub race_date: NaiveDate,
    pub weeks: Vec<PlannedWeek>,
}

pub fn phase_for(weeks_to_race: i64) -> Phase {
    if weeks_to_race < TAPER_WEEKS {
        Phase::Taper
    } else if weeks_to_race < TAPER_WEEKS + PEAK_WEEKS {
        Phase::Peak
    } else if weeks_to_race < TAPER_WEEKS + PEAK_WEEKS + BUILD_WEEKS {
        Phase::Build
    } else {
        Phase::Base
    }
}

fn round1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

/// The Monday on or before `date`.
fn monday_on_or_before(date: NaiveDate) -> NaiveDate {
    date - Duration::days(i64::from(date.weekday().num_days_from_monday()))
}

/// The Monday on or after `date`.
fn monday_on_or_after(date: NaiveDate) -> NaiveDate {
    let days_until = (7 - date.weekday().num_days_from_monday()) % 7;
    date + Duration::days(i64::from(days_until))
}

/// Plans every week from the next Monday (or today, if it is a Monday) up to
/// and including race week. Returns no weeks if the race has passed.
pub fn build_plan(cfg: &Config, today: NaiveDate) -> Plan {
    let race_week = monday_on_or_before(cfg.race.date);
    let mut week_start = monday_on_or_after(today);
    let mut weeks = Vec::new();

    let mut index = 0usize;
    while week_start <= race_week {
        let weeks_to_race = (race_week - week_start).num_days() / 7;
        let phase = phase_for(weeks_to_race);
        let recovery = matches!(phase, Phase::Base | Phase::Build)
            && index % RECOVERY_EVERY == RECOVERY_EVERY - 1;
        let factor = match (phase, weeks_to_race) {
            (Phase::Taper, 0) => 0.4,
            (Phase::Taper, _) => 0.6,
            _ if recovery => RECOVERY_FACTOR,
            _ => 1.0,
        };
        weeks.push(PlannedWeek {
            week_start,
            weeks_to_race,
            phase,
            recovery,
            target_hours: round1(cfg.hours_for(week_start) * factor),
        });
        week_start += Duration::days(7);
        index += 1;
    }

    Plan {
        race: cfg.race.name.clone(),
        race_date: cfg.race.date,
        weeks,
    }
}

impl fmt::Display for Plan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{} on {}: {} planned weeks",
            self.race,
            self.race_date,
            self.weeks.len()
        )?;
        if self.weeks.is_empty() {
            return writeln!(f, "The race date has already passed.");
        }

        let count = |p: Phase| self.weeks.iter().filter(|w| w.phase == p).count();
        writeln!(
            f,
            "base {} | build {} | peak {} | taper {}",
            count(Phase::Base),
            count(Phase::Build),
            count(Phase::Peak),
            count(Phase::Taper)
        )?;
        writeln!(f)?;
        writeln!(f, "{:<11} {:>7}  {:<6} {:>5}", "Week of", "To race", "Phase", "Hours")?;
        for w in &self.weeks {
            writeln!(
                f,
                "{:<11} {:>7}  {:<6} {:>5.1}{}",
                w.week_start,
                w.weeks_to_race,
                w.phase,
                w.target_hours,
                if w.recovery { "  recovery" } else { "" }
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn cfg(extra_hours: &str) -> Config {
        Config::parse(&format!(
            "[race]\nname = \"Maine\"\ndate = \"2027-07-25\"\n\
             [[hours]]\nfrom = \"2026-11-16\"\nhours = 6.0\n{extra_hours}"
        ))
        .unwrap()
    }

    #[test]
    fn phase_boundaries() {
        assert_eq!(phase_for(0), Phase::Taper);
        assert_eq!(phase_for(1), Phase::Taper);
        assert_eq!(phase_for(2), Phase::Peak);
        assert_eq!(phase_for(4), Phase::Peak);
        assert_eq!(phase_for(5), Phase::Build);
        assert_eq!(phase_for(16), Phase::Build);
        assert_eq!(phase_for(17), Phase::Base);
    }

    #[test]
    fn plans_36_weeks_from_the_next_monday_to_race_week() {
        // 2026-11-15 is a Sunday, so planning starts Monday 2026-11-16.
        let plan = build_plan(&cfg(""), d("2026-11-15"));
        assert_eq!(plan.weeks.len(), 36);

        let first = &plan.weeks[0];
        assert_eq!(first.week_start, d("2026-11-16"));
        assert_eq!(first.weeks_to_race, 35);
        assert_eq!(first.phase, Phase::Base);

        let last = plan.weeks.last().unwrap();
        assert_eq!(last.week_start, d("2027-07-19")); // Monday of race week
        assert_eq!(last.weeks_to_race, 0);
        assert_eq!(last.phase, Phase::Taper);
        assert_eq!(last.target_hours, 2.4); // 40% of 6 h

        let count = |p: Phase| plan.weeks.iter().filter(|w| w.phase == p).count();
        assert_eq!(count(Phase::Base), 19);
        assert_eq!(count(Phase::Build), 12);
        assert_eq!(count(Phase::Peak), 3);
        assert_eq!(count(Phase::Taper), 2);
    }

    #[test]
    fn every_fourth_base_or_build_week_is_a_recovery_week() {
        let plan = build_plan(&cfg(""), d("2026-11-15"));
        let recovery: Vec<usize> = plan
            .weeks
            .iter()
            .enumerate()
            .filter(|(_, w)| w.recovery)
            .map(|(i, _)| i)
            .collect();
        assert!(recovery.iter().all(|i| i % 4 == 3));
        assert_eq!(plan.weeks[3].target_hours, 4.2); // 70% of 6 h
        assert_eq!(plan.weeks[0].target_hours, 6.0);
        // Never during peak or taper.
        assert!(plan
            .weeks
            .iter()
            .filter(|w| matches!(w.phase, Phase::Peak | Phase::Taper))
            .all(|w| !w.recovery));
    }

    #[test]
    fn taper_week_before_the_race_is_60_percent() {
        let plan = build_plan(&cfg(""), d("2026-11-15"));
        let w = plan.weeks.iter().find(|w| w.weeks_to_race == 1).unwrap();
        assert_eq!(w.target_hours, 3.6);
    }

    #[test]
    fn follows_the_hours_schedule() {
        let plan = build_plan(
            &cfg("[[hours]]\nfrom = \"2027-01-04\"\nhours = 8.0\n"),
            d("2026-11-15"),
        );
        let hours_on = |date: &str| {
            plan.weeks
                .iter()
                .find(|w| w.week_start == d(date))
                .unwrap()
                .target_hours
        };
        assert_eq!(hours_on("2026-12-28"), 6.0);
        assert_eq!(hours_on("2027-01-11"), 8.0);
    }

    #[test]
    fn a_monday_starts_that_same_day() {
        let plan = build_plan(&cfg(""), d("2026-11-16"));
        assert_eq!(plan.weeks[0].week_start, d("2026-11-16"));
    }

    #[test]
    fn a_past_race_yields_no_weeks() {
        let plan = build_plan(&cfg(""), d("2027-08-02"));
        assert!(plan.weeks.is_empty());
    }
}
