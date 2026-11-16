//! Turns raw wellness + activity records into a compact, typed summary.
//!
//! This is the part that will later be serialised to JSON and handed to the
//! LLM, so it is deliberately small: a few numbers per week and per sport
//! rather than every record. It is pure (no I/O), which makes it easy to test.

use std::collections::BTreeMap;
use std::fmt;

use chrono::{Datelike, Duration, NaiveDate};
use serde::Serialize;

use crate::models::{Activity, Wellness};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Sport {
    Swim,
    Bike,
    Run,
    Other,
}

impl Sport {
    /// Maps an intervals.icu activity type to one of our three disciplines.
    pub fn from_type(t: &str) -> Sport {
        match t {
            "Swim" | "OpenWaterSwim" => Sport::Swim,
            "Ride" | "VirtualRide" | "GravelRide" | "MountainBikeRide" | "EBikeRide" => Sport::Bike,
            "Run" | "TrailRun" | "VirtualRun" => Sport::Run,
            _ => Sport::Other,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Sport::Swim => "swim",
            Sport::Bike => "bike",
            Sport::Run => "run",
            Sport::Other => "other",
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize)]
pub struct SportTotals {
    pub sessions: u32,
    pub hours: f64,
    pub load: f64,
    pub distance_km: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WeekSummary {
    /// Monday of the week.
    pub week_start: NaiveDate,
    /// True if the week is cut off by the start of the window or by today.
    pub partial: bool,
    pub by_sport: BTreeMap<Sport, SportTotals>,
    pub total_hours: f64,
    pub total_load: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Fitness {
    pub date: NaiveDate,
    /// Chronic training load ("fitness").
    pub ctl: f64,
    /// Acute training load ("fatigue").
    pub atl: f64,
    /// Training stress balance ("form") = CTL - ATL. Negative means fatigued.
    pub tsb: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Recovery {
    pub hrv_7d: Option<f64>,
    pub hrv_28d: Option<f64>,
    /// 7-day HRV vs 28-day baseline, in percent. Strongly negative = watch out.
    pub hrv_vs_baseline_pct: Option<f64>,
    pub resting_hr_7d: Option<f64>,
    pub resting_hr_28d: Option<f64>,
    pub sleep_hours_7d: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Summary {
    pub as_of: NaiveDate,
    pub fitness: Option<Fitness>,
    pub recovery: Recovery,
    pub weeks: Vec<WeekSummary>,
}

fn round1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

/// Some devices report 0 when they have no reading; treat that as missing.
fn positive(v: Option<f64>) -> Option<f64> {
    v.filter(|x| *x > 0.0)
}

/// Accepts `YYYY-MM-DD` or a longer ISO timestamp and returns the date part.
pub(crate) fn parse_date(s: &str) -> Option<NaiveDate> {
    s.get(..10)
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
}

pub(crate) fn mean(values: impl Iterator<Item = f64>) -> Option<f64> {
    let (sum, n) = values.fold((0.0, 0u32), |(s, n), v| (s + v, n + 1));
    (n > 0).then(|| sum / f64::from(n))
}

/// Latest day (not after `as_of`) that has both CTL and ATL.
pub fn latest_fitness(as_of: NaiveDate, wellness: &[Wellness]) -> Option<Fitness> {
    wellness
        .iter()
        .filter_map(|w| {
            let date = parse_date(&w.id)?;
            if date > as_of {
                return None;
            }
            Some((date, w.ctl?, w.atl?))
        })
        .max_by_key(|(date, _, _)| *date)
        .map(|(date, ctl, atl)| Fitness {
            date,
            ctl: round1(ctl),
            atl: round1(atl),
            tsb: round1(ctl - atl),
        })
}

/// Mean of `pick` over wellness records in the `days` ending on `as_of`.
fn window_mean(
    wellness: &[Wellness],
    as_of: NaiveDate,
    days: i64,
    pick: impl Fn(&Wellness) -> Option<f64>,
) -> Option<f64> {
    let start = as_of - Duration::days(days - 1);
    mean(
        wellness
            .iter()
            .filter(|w| parse_date(&w.id).is_some_and(|d| d >= start && d <= as_of))
            .filter_map(pick),
    )
}

/// Builds the summary. `days` is how many days of weekly history to include.
pub fn build(
    as_of: NaiveDate,
    days: i64,
    wellness: &[Wellness],
    activities: &[Activity],
) -> Summary {
    let fitness = latest_fitness(as_of, wellness);

    let hrv_7d = window_mean(wellness, as_of, 7, |w| positive(w.hrv));
    let hrv_28d = window_mean(wellness, as_of, 28, |w| positive(w.hrv));
    let hrv_vs_baseline_pct = match (hrv_7d, hrv_28d) {
        (Some(recent), Some(base)) if base > 0.0 => Some(round1((recent - base) / base * 100.0)),
        _ => None,
    };
    let recovery = Recovery {
        hrv_7d: hrv_7d.map(round1),
        hrv_28d: hrv_28d.map(round1),
        hrv_vs_baseline_pct,
        resting_hr_7d: window_mean(wellness, as_of, 7, |w| positive(w.resting_hr)).map(round1),
        resting_hr_28d: window_mean(wellness, as_of, 28, |w| positive(w.resting_hr)).map(round1),
        sleep_hours_7d: window_mean(wellness, as_of, 7, |w| {
            positive(w.sleep_secs).map(|s| s / 3600.0)
        })
        .map(round1),
    };

    // Group activities into Monday-start weeks, then by sport.
    let window_start = as_of - Duration::days(days - 1);
    let mut grouped: BTreeMap<NaiveDate, BTreeMap<Sport, SportTotals>> = BTreeMap::new();
    for a in activities {
        let Some(date) = a.start_date_local.as_deref().and_then(parse_date) else {
            continue;
        };
        if date < window_start || date > as_of {
            continue;
        }
        let week_start = date - Duration::days(i64::from(date.weekday().num_days_from_monday()));
        let sport = a.kind.as_deref().map(Sport::from_type).unwrap_or(Sport::Other);
        let totals = grouped
            .entry(week_start)
            .or_default()
            .entry(sport)
            .or_default();
        totals.sessions += 1;
        totals.hours += a.moving_time.unwrap_or(0.0) / 3600.0;
        totals.load += a.icu_training_load.unwrap_or(0.0);
        totals.distance_km += a.distance.unwrap_or(0.0) / 1000.0;
    }

    let weeks = grouped
        .into_iter()
        .map(|(week_start, mut by_sport)| {
            for t in by_sport.values_mut() {
                t.hours = round1(t.hours);
                t.load = round1(t.load);
                t.distance_km = round1(t.distance_km);
            }
            let total_hours = round1(by_sport.values().map(|t| t.hours).sum());
            let total_load = round1(by_sport.values().map(|t| t.load).sum());
            WeekSummary {
                week_start,
                partial: week_start < window_start || week_start + Duration::days(6) > as_of,
                by_sport,
                total_hours,
                total_load,
            }
        })
        .collect();

    Summary {
        as_of,
        fitness,
        recovery,
        weeks,
    }
}

fn opt(v: Option<f64>, unit: &str) -> String {
    v.map_or_else(|| "n/a".to_string(), |x| format!("{x:.1}{unit}"))
}

impl fmt::Display for Summary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Training summary as of {}", self.as_of)?;
        writeln!(f)?;

        match &self.fitness {
            Some(x) => writeln!(
                f,
                "Fitness (CTL) {:.1} | Fatigue (ATL) {:.1} | Form (TSB) {:+.1}  [{}]",
                x.ctl, x.atl, x.tsb, x.date
            )?,
            None => writeln!(f, "Fitness: no CTL/ATL data found")?,
        }

        let r = &self.recovery;
        writeln!(
            f,
            "HRV 7d {} vs 28d {} ({})",
            opt(r.hrv_7d, ""),
            opt(r.hrv_28d, ""),
            r.hrv_vs_baseline_pct
                .map_or_else(|| "n/a".to_string(), |p| format!("{p:+.1}%")),
        )?;
        writeln!(
            f,
            "Resting HR 7d {} vs 28d {}",
            opt(r.resting_hr_7d, ""),
            opt(r.resting_hr_28d, "")
        )?;
        writeln!(f, "Sleep 7d avg {}", opt(r.sleep_hours_7d, " h"))?;

        for w in &self.weeks {
            writeln!(f)?;
            writeln!(
                f,
                "Week of {}{}: {:.1} h, load {:.0}",
                w.week_start,
                if w.partial { " (partial)" } else { "" },
                w.total_hours,
                w.total_load
            )?;
            for (sport, t) in &w.by_sport {
                writeln!(
                    f,
                    "  {:<5} {} sessions, {:.1} h, {:.1} km, load {:.0}",
                    sport.label(),
                    t.sessions,
                    t.hours,
                    t.distance_km,
                    t.load
                )?;
            }
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

    fn wellness(date: NaiveDate, ctl: Option<f64>, atl: Option<f64>, hrv: Option<f64>) -> Wellness {
        Wellness {
            id: date.to_string(),
            ctl,
            atl,
            ramp_rate: None,
            hrv,
            resting_hr: None,
            sleep_secs: None,
            sleep_score: None,
            readiness: None,
        }
    }

    fn activity(date: &str, kind: &str, secs: f64, load: f64, metres: f64) -> Activity {
        Activity {
            start_date_local: Some(format!("{date}T07:30:00")),
            name: None,
            kind: Some(kind.to_string()),
            moving_time: Some(secs),
            distance: Some(metres),
            icu_training_load: Some(load),
        }
    }

    #[test]
    fn maps_activity_types_to_sports() {
        assert_eq!(Sport::from_type("Swim"), Sport::Swim);
        assert_eq!(Sport::from_type("OpenWaterSwim"), Sport::Swim);
        assert_eq!(Sport::from_type("VirtualRide"), Sport::Bike);
        assert_eq!(Sport::from_type("TrailRun"), Sport::Run);
        assert_eq!(Sport::from_type("WeightTraining"), Sport::Other);
    }

    #[test]
    fn groups_activities_by_monday_week_and_sport() {
        // 2026-11-15 is a Sunday; the 14-day window starts Monday 2026-11-02.
        let as_of = d("2026-11-15");
        let acts = vec![
            activity("2026-11-01", "Run", 3600.0, 40.0, 10_000.0), // before window
            activity("2026-11-03", "Run", 3600.0, 50.0, 10_000.0),
            activity("2026-11-03", "Ride", 7200.0, 100.0, 50_000.0),
            activity("2026-11-10", "Swim", 1800.0, 30.0, 1_500.0),
            activity("2026-11-16", "Run", 3600.0, 50.0, 10_000.0), // after as_of
        ];
        let s = build(as_of, 14, &[], &acts);

        assert_eq!(s.weeks.len(), 2);

        let w1 = &s.weeks[0];
        assert_eq!(w1.week_start, d("2026-11-02"));
        assert!(!w1.partial);
        assert_eq!(w1.total_hours, 3.0);
        assert_eq!(w1.total_load, 150.0);
        assert_eq!(w1.by_sport[&Sport::Run].sessions, 1);
        assert_eq!(w1.by_sport[&Sport::Bike].distance_km, 50.0);

        let w2 = &s.weeks[1];
        assert_eq!(w2.week_start, d("2026-11-09"));
        assert_eq!(w2.by_sport[&Sport::Swim].distance_km, 1.5);
    }

    #[test]
    fn flags_the_current_week_as_partial() {
        let as_of = d("2026-11-12"); // Thursday
        let acts = vec![activity("2026-11-10", "Run", 3600.0, 50.0, 10_000.0)];
        let s = build(as_of, 7, &[], &acts);
        assert!(s.weeks[0].partial);
    }

    #[test]
    fn computes_hrv_baseline_delta() {
        let as_of = d("2026-11-15");
        // Last 7 days at 54, the 21 days before that at 60.
        let w: Vec<Wellness> = (0..28)
            .map(|i| {
                let hrv = if i < 7 { 54.0 } else { 60.0 };
                wellness(as_of - Duration::days(i), None, None, Some(hrv))
            })
            .collect();
        let r = build(as_of, 28, &w, &[]).recovery;
        assert_eq!(r.hrv_7d, Some(54.0));
        assert_eq!(r.hrv_28d, Some(58.5));
        assert_eq!(r.hrv_vs_baseline_pct, Some(-7.7));
    }

    #[test]
    fn fitness_uses_latest_day_with_ctl_and_atl_and_ignores_future() {
        let as_of = d("2026-11-15");
        let w = vec![
            wellness(d("2026-11-13"), Some(48.0), Some(55.0), None),
            wellness(d("2026-11-14"), Some(50.0), Some(62.0), None),
            wellness(d("2026-11-15"), None, None, Some(55.0)), // no CTL/ATL yet
            wellness(d("2026-11-20"), Some(99.0), Some(99.0), None), // future
        ];
        let f = build(as_of, 28, &w, &[]).fitness.unwrap();
        assert_eq!(f.date, d("2026-11-14"));
        assert_eq!(f.tsb, -12.0);
    }

    #[test]
    fn missing_data_yields_none_not_a_panic() {
        let s = build(d("2026-11-15"), 28, &[], &[]);
        assert!(s.fitness.is_none());
        assert!(s.recovery.hrv_7d.is_none());
        assert!(s.weeks.is_empty());
    }

    #[test]
    fn zero_hrv_is_treated_as_missing() {
        let as_of = d("2026-11-15");
        let w = vec![wellness(as_of, None, None, Some(0.0))];
        assert!(build(as_of, 7, &w, &[]).recovery.hrv_7d.is_none());
    }
}
