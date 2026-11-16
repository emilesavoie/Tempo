//! User-editable settings, loaded from `tricoach.toml`.

use std::path::Path;

use anyhow::{ensure, Context, Result};
use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub race: Race,
    /// Weekly hours schedule; kept sorted by `from` after loading.
    pub hours: Vec<HoursEntry>,
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
        cfg.hours.sort_by_key(|h| h.from);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn the_shipped_example_file_is_valid() {
        let cfg = Config::parse(include_str!("../tricoach.example.toml")).unwrap();
        assert_eq!(cfg.race.date, d("2027-07-25"));
        assert_eq!(cfg.hours_for(d("2026-11-16")), 6.0);
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
}
