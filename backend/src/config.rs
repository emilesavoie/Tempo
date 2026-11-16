//! User-editable settings, loaded from `tricoach.toml`.

use std::path::Path;

use anyhow::{ensure, Context, Result};
use chrono::{NaiveDate, Weekday};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub race: Race,
    /// Weekly hours schedule; kept sorted by `from` after loading.
    pub hours: Vec<HoursEntry>,
    /// Weekly rhythm (gym days, swim sessions, long-session days); sorted by `from`.
    #[serde(default)]
    pub rhythm: Vec<RhythmEntry>,
    #[serde(default)]
    pub safety: Safety,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Race {
    pub name: String,
    pub date: NaiveDate,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HoursEntry {
    pub from: NaiveDate,
    pub hours: f64,
}

/// What a normal week looks like from `from` onwards. Days are written as
/// "mon", "tue", ... in the config file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RhythmEntry {
    pub from: NaiveDate,
    /// Days you lift weights. The planner avoids hard sessions on these days.
    #[serde(default)]
    pub gym_days: Vec<Weekday>,
    #[serde(default = "default_swim_sessions")]
    pub swim_sessions: u32,
    /// Days you'd like to swim on; other swims go on other free days.
    #[serde(default)]
    pub swim_preferred_days: Vec<Weekday>,
    #[serde(default = "default_long_bike_day")]
    pub long_bike_day: Weekday,
    #[serde(default = "default_long_run_day")]
    pub long_run_day: Weekday,
    /// Days with no training at all.
    #[serde(default)]
    pub rest_days: Vec<Weekday>,
}

fn default_swim_sessions() -> u32 {
    2
}
fn default_long_bike_day() -> Weekday {
    Weekday::Sat
}
fn default_long_run_day() -> Weekday {
    Weekday::Sun
}

impl RhythmEntry {
    /// Used when the config has no `[[rhythm]]` entries at all.
    pub fn fallback(from: NaiveDate) -> Self {
        Self {
            from,
            gym_days: Vec::new(),
            swim_sessions: default_swim_sessions(),
            swim_preferred_days: Vec::new(),
            long_bike_day: default_long_bike_day(),
            long_run_day: default_long_run_day(),
            rest_days: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Safety {
    /// A week's target is capped at this many percent above the last full week
    /// you actually trained. 0 turns the cap off.
    #[serde(default = "default_ramp")]
    pub max_weekly_ramp_pct: f64,
}

fn default_ramp() -> f64 {
    30.0
}

impl Default for Safety {
    fn default() -> Self {
        Self {
            max_weekly_ramp_pct: default_ramp(),
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path).with_context(|| {
            format!(
                "could not read {} (copy tricoach.example.toml to tricoach.toml to get started)",
                path.display()
            )
        })?;
        Self::parse(&text).with_context(|| format!("problem in {}", path.display()))
    }

    pub fn parse(text: &str) -> Result<Self> {
        let mut cfg: Config = toml::from_str(text).context("invalid config file")?;
        ensure!(
            !cfg.hours.is_empty(),
            "add at least one [[hours]] entry with `from` and `hours`"
        );
        for h in &cfg.hours {
            ensure!(
                h.hours > 0.0 && h.hours <= 40.0,
                "hours must be between 0 and 40 (got {} for {})",
                h.hours,
                h.from
            );
        }
        for r in &cfg.rhythm {
            ensure!(
                r.swim_sessions <= 7,
                "swim_sessions must be 7 or fewer (entry from {})",
                r.from
            );
            ensure!(
                !r.rest_days.contains(&r.long_bike_day) && !r.rest_days.contains(&r.long_run_day),
                "the long bike or long run day is also a rest day (entry from {})",
                r.from
            );
        }
        ensure!(
            cfg.safety.max_weekly_ramp_pct >= 0.0,
            "max_weekly_ramp_pct can't be negative (use 0 to turn the cap off)"
        );
        cfg.hours.sort_by_key(|h| h.from);
        cfg.rhythm.sort_by_key(|r| r.from);
        Ok(cfg)
    }

    /// Target hours for a normal week starting on `week_start`: the latest
    /// entry that has started by then, or the first entry if none has yet.
    pub fn hours_for(&self, week_start: NaiveDate) -> f64 {
        self.hours
            .iter()
            .rev()
            .find(|h| h.from <= week_start)
            .or_else(|| self.hours.first())
            .map_or(0.0, |h| h.hours)
    }

    /// The weekly rhythm in force for the week starting `week_start`.
    pub fn rhythm_for(&self, week_start: NaiveDate) -> RhythmEntry {
        self.rhythm
            .iter()
            .rev()
            .find(|r| r.from <= week_start)
            .or_else(|| self.rhythm.first())
            .cloned()
            .unwrap_or_else(|| RhythmEntry::fallback(week_start))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    const BASE: &str = "[race]\nname = \"r\"\ndate = \"2027-07-25\"\n\
                        [[hours]]\nfrom = \"2026-11-16\"\nhours = 6.0\n";

    #[test]
    fn the_shipped_example_file_is_valid() {
        let cfg = Config::parse(include_str!("../tricoach.example.toml")).unwrap();
        assert_eq!(cfg.race.date, d("2027-07-25"));
        assert_eq!(cfg.hours_for(d("2026-11-16")), 6.0);

        let nov = cfg.rhythm_for(d("2026-11-16"));
        assert_eq!(nov.swim_sessions, 0);
        assert_eq!(nov.gym_days, vec![Weekday::Mon, Weekday::Tue]);

        let jan = cfg.rhythm_for(d("2027-01-11"));
        assert_eq!(jan.swim_sessions, 3);
        assert_eq!(jan.swim_preferred_days, vec![Weekday::Wed]);
        assert!(jan.gym_days.is_empty());
    }

    #[test]
    fn picks_the_latest_started_entry_regardless_of_file_order() {
        let cfg = Config::parse(
            r#"
            [race]
            name = "r"
            date = "2027-07-25"
            [[hours]]
            from = "2027-01-04"
            hours = 8.0
            [[hours]]
            from = "2026-11-16"
            hours = 6.0
            "#,
        )
        .unwrap();
        assert_eq!(cfg.hours_for(d("2026-12-28")), 6.0);
        assert_eq!(cfg.hours_for(d("2027-01-04")), 8.0);
        assert_eq!(cfg.hours_for(d("2027-03-01")), 8.0);
        // Before the first entry, fall back to the first one.
        assert_eq!(cfg.hours_for(d("2026-01-01")), 6.0);
    }

    #[test]
    fn rejects_missing_or_silly_hours() {
        let no_hours = "[race]\nname = \"r\"\ndate = \"2027-07-25\"\n";
        assert!(Config::parse(no_hours).is_err());

        let negative = format!("{no_hours}[[hours]]\nfrom = \"2026-11-16\"\nhours = -2.0\n");
        assert!(Config::parse(&negative).is_err());

        let huge = format!("{no_hours}[[hours]]\nfrom = \"2026-11-16\"\nhours = 60.0\n");
        assert!(Config::parse(&huge).is_err());
    }

    #[test]
    fn rejects_unquoted_or_malformed_dates() {
        let bad = "[race]\nname = \"r\"\ndate = \"next July\"\n[[hours]]\nfrom = \"2026-11-16\"\nhours = 6.0\n";
        assert!(Config::parse(bad).is_err());
    }

    #[test]
    fn rhythm_days_accept_lowercase_and_missing_fields_use_defaults() {
        let cfg = Config::parse(&format!(
            "{BASE}[[rhythm]]\nfrom = \"2026-11-16\"\ngym_days = [\"mon\", \"Tue\"]\nswim_sessions = 0\n"
        ))
        .unwrap();
        let r = cfg.rhythm_for(d("2026-11-23"));
        assert_eq!(r.gym_days, vec![Weekday::Mon, Weekday::Tue]);
        assert_eq!(r.long_bike_day, Weekday::Sat);
        assert_eq!(r.long_run_day, Weekday::Sun);
        assert!(r.rest_days.is_empty());
    }

    #[test]
    fn no_rhythm_entries_means_sensible_defaults() {
        let cfg = Config::parse(BASE).unwrap();
        let r = cfg.rhythm_for(d("2026-11-16"));
        assert_eq!(r.swim_sessions, 2);
        assert_eq!(r.long_bike_day, Weekday::Sat);
        assert_eq!(cfg.safety.max_weekly_ramp_pct, 30.0);
    }

    #[test]
    fn rhythm_follows_the_latest_started_entry() {
        let cfg = Config::parse(&format!(
            "{BASE}[[rhythm]]\nfrom = \"2027-01-04\"\nswim_sessions = 3\n\
             [[rhythm]]\nfrom = \"2026-11-16\"\nswim_sessions = 0\n"
        ))
        .unwrap();
        assert_eq!(cfg.rhythm_for(d("2026-12-28")).swim_sessions, 0);
        assert_eq!(cfg.rhythm_for(d("2027-01-04")).swim_sessions, 3);
    }

    #[test]
    fn rejects_a_long_session_on_a_rest_day() {
        let bad = format!("{BASE}[[rhythm]]\nfrom = \"2026-11-16\"\nrest_days = [\"sat\"]\n");
        assert!(Config::parse(&bad).is_err());
    }

    #[test]
    fn rejects_a_negative_ramp_cap() {
        let bad = format!("{BASE}[safety]\nmax_weekly_ramp_pct = -5\n");
        assert!(Config::parse(&bad).is_err());
    }
}
