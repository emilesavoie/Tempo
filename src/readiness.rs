//! Rules-based daily readiness check: go, go easy, or rest.
//!
//! Deliberately simple and explainable. Each signal is rated Ok, Caution or
//! Red, and the ratings are added up as points (0 / 1 / 2):
//!
//! - 0-1 points -> GO        (a single yellow flag is shown but doesn't change the day)
//! - 2 points   -> EASY DAY  (one red flag, or two cautions)
//! - 3+ points  -> REST
//!
//! Signals: form (TSB), resting HR vs your own 28-day baseline, last night's
//! sleep. A signal with no data is skipped, not guessed. HRV is not used
//! because it isn't logged; it can become a fourth signal later.
//!
//! All thresholds live in [`Thresholds`] so they are easy to tune.

use std::fmt;

use chrono::{Duration, NaiveDate};
use serde::Serialize;

use crate::models::Wellness;
use crate::summary::{mean, parse_date, Fitness};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Ok,
    Caution,
    Red,
}

impl Level {
    fn points(self) -> u32 {
        match self {
            Level::Ok => 0,
            Level::Caution => 1,
            Level::Red => 2,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Level::Ok => "ok",
            Level::Caution => "caution",
            Level::Red => "red",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Go,
    Easy,
    Rest,
    NoData,
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Verdict::Go => "GO",
            Verdict::Easy => "EASY DAY",
            Verdict::Rest => "REST",
            Verdict::NoData => "NO DATA",
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Signal {
    pub name: &'static str,
    pub level: Level,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Readiness {
    pub as_of: NaiveDate,
    pub verdict: Verdict,
    pub signals: Vec<Signal>,
}

/// Tunable cut-offs. The defaults are generic starting points, not gospel:
/// adjust them once you see how they match how you actually feel.
#[derive(Debug, Clone)]
pub struct Thresholds {
    /// TSB below this is a caution / red flag (deep fatigue).
    pub tsb_caution_below: f64,
    pub tsb_red_below: f64,
    /// Resting HR this many bpm or more above your 28-day baseline.
    pub rhr_caution_bpm: f64,
    pub rhr_red_bpm: f64,
    /// Last night's sleep below this many hours.
    pub sleep_caution_h: f64,
    pub sleep_red_h: f64,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            tsb_caution_below: -20.0,
            tsb_red_below: -30.0,
            rhr_caution_bpm: 5.0,
            rhr_red_bpm: 8.0,
            sleep_caution_h: 6.0,
            sleep_red_h: 5.0,
        }
    }
}

/// For metrics where a LOW value is bad (TSB, sleep).
fn low_is_bad(value: f64, caution_below: f64, red_below: f64) -> Level {
    if value < red_below {
        Level::Red
    } else if value < caution_below {
        Level::Caution
    } else {
        Level::Ok
    }
}

/// For metrics where a HIGH value is bad (resting HR above baseline).
fn high_is_bad(value: f64, caution_at: f64, red_at: f64) -> Level {
    if value >= red_at {
        Level::Red
    } else if value >= caution_at {
        Level::Caution
    } else {
        Level::Ok
    }
}

/// Most recent positive reading from `as_of` or the day before. Watches often
/// sync late, so "today" may not exist yet; anything older is too stale to use.
fn latest_recent(
    as_of: NaiveDate,
    wellness: &[Wellness],
    pick: impl Fn(&Wellness) -> Option<f64>,
) -> Option<(NaiveDate, f64)> {
    wellness
        .iter()
        .filter_map(|w| {
            let date = parse_date(&w.id)?;
            if date > as_of || date < as_of - Duration::days(1) {
                return None;
            }
            let value = pick(w).filter(|v| *v > 0.0)?;
            Some((date, value))
        })
        .max_by_key(|(date, _)| *date)
}

fn resting_hr_signal(as_of: NaiveDate, wellness: &[Wellness], t: &Thresholds) -> Option<Signal> {
    let (date, today) = latest_recent(as_of, wellness, |w| w.resting_hr)?;
    // Baseline: the 28 days before the reading, so today doesn't skew its own baseline.
    let baseline = mean(
        wellness
            .iter()
            .filter(|w| {
                parse_date(&w.id).is_some_and(|d| d < date && d >= date - Duration::days(28))
            })
            .filter_map(|w| w.resting_hr)
            .filter(|v| *v > 0.0),
    )?;
    let delta = today - baseline;
    Some(Signal {
        name: "resting_hr",
        level: high_is_bad(delta, t.rhr_caution_bpm, t.rhr_red_bpm),
        detail: format!("{today:.0} bpm on {date}, {delta:+.1} vs 28-day baseline {baseline:.1}"),
    })
}

fn sleep_signal(as_of: NaiveDate, wellness: &[Wellness], t: &Thresholds) -> Option<Signal> {
    let (date, hours) = latest_recent(as_of, wellness, |w| w.sleep_secs.map(|s| s / 3600.0))?;
    Some(Signal {
        name: "sleep",
        level: low_is_bad(hours, t.sleep_caution_h, t.sleep_red_h),
        detail: format!("{hours:.1} h on {date}"),
    })
}

fn form_signal(fitness: &Fitness, t: &Thresholds) -> Signal {
    Signal {
        name: "form",
        level: low_is_bad(fitness.tsb, t.tsb_caution_below, t.tsb_red_below),
        detail: format!(
            "TSB {:+.1} (CTL {:.1}, ATL {:.1}) on {}",
            fitness.tsb, fitness.ctl, fitness.atl, fitness.date
        ),
    }
}

fn decide(signals: &[Signal]) -> Verdict {
    if signals.is_empty() {
        return Verdict::NoData;
    }
    match signals.iter().map(|s| s.level.points()).sum::<u32>() {
        0 | 1 => Verdict::Go,
        2 => Verdict::Easy,
        _ => Verdict::Rest,
    }
}

pub fn assess(
    as_of: NaiveDate,
    wellness: &[Wellness],
    fitness: Option<&Fitness>,
    t: &Thresholds,
) -> Readiness {
    let mut signals = Vec::new();
    if let Some(f) = fitness {
        signals.push(form_signal(f, t));
    }
    signals.extend(resting_hr_signal(as_of, wellness, t));
    signals.extend(sleep_signal(as_of, wellness, t));

    Readiness {
        as_of,
        verdict: decide(&signals),
        signals,
    }
}

impl fmt::Display for Readiness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Readiness for {}: {}", self.as_of, self.verdict)?;
        if self.signals.is_empty() {
            writeln!(f, "  No usable wellness data from the last two days.")?;
        }
        for s in &self.signals {
            writeln!(f, "  [{:<7}] {:<10} {}", s.level.label(), s.name, s.detail)?;
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

    fn day(date: NaiveDate, rhr: Option<f64>, sleep_h: Option<f64>) -> Wellness {
        Wellness {
            id: date.to_string(),
            ctl: None,
            atl: None,
            ramp_rate: None,
            hrv: None,
            resting_hr: rhr,
            sleep_secs: sleep_h.map(|h| h * 3600.0),
            sleep_score: None,
            readiness: None,
        }
    }

    fn fitness(tsb: f64) -> Fitness {
        Fitness {
            date: d("2026-11-15"),
            ctl: 50.0,
            atl: 50.0 - tsb,
            tsb,
        }
    }

    /// 28 days of baseline (resting HR 50, sleep 7.5 h) plus today's readings.
    fn history(as_of: NaiveDate, rhr: f64, sleep_h: f64) -> Vec<Wellness> {
        let mut v: Vec<Wellness> = (1..=28)
            .map(|i| day(as_of - Duration::days(i), Some(50.0), Some(7.5)))
            .collect();
        v.push(day(as_of, Some(rhr), Some(sleep_h)));
        v
    }

    fn run(tsb: f64, rhr: f64, sleep_h: f64) -> Readiness {
        let as_of = d("2026-11-15");
        assess(
            as_of,
            &history(as_of, rhr, sleep_h),
            Some(&fitness(tsb)),
            &Thresholds::default(),
        )
    }

    #[test]
    fn fresh_and_rested_is_go() {
        let r = run(8.3, 51.0, 8.0);
        assert_eq!(r.verdict, Verdict::Go);
        assert_eq!(r.signals.len(), 3);
        assert!(r.signals.iter().all(|s| s.level == Level::Ok));
    }

    #[test]
    fn a_single_caution_is_flagged_but_still_go() {
        let r = run(8.3, 50.0, 5.5); // 5.5 h sleep: caution, not red
        assert_eq!(r.verdict, Verdict::Go);
        assert!(r.signals.iter().any(|s| s.level == Level::Caution));
    }

    #[test]
    fn a_single_red_flag_means_easy_day() {
        assert_eq!(run(8.3, 50.0, 4.5).verdict, Verdict::Easy);
    }

    #[test]
    fn deep_fatigue_alone_means_easy_day() {
        assert_eq!(run(-35.0, 50.0, 8.0).verdict, Verdict::Easy);
    }

    #[test]
    fn red_plus_caution_means_rest() {
        // 4.5 h sleep (red) + resting HR 6 bpm above baseline (caution) = 3 points.
        assert_eq!(run(8.3, 56.0, 4.5).verdict, Verdict::Rest);
    }

    #[test]
    fn two_cautions_mean_easy_day() {
        // TSB -22 (caution) + resting HR +5 bpm (caution).
        assert_eq!(run(-22.0, 55.0, 8.0).verdict, Verdict::Easy);
    }

    #[test]
    fn no_data_is_reported_as_such() {
        let r = assess(d("2026-11-15"), &[], None, &Thresholds::default());
        assert_eq!(r.verdict, Verdict::NoData);
        assert!(r.signals.is_empty());
    }

    #[test]
    fn falls_back_to_yesterday_but_not_older() {
        let as_of = d("2026-11-15");
        let t = Thresholds::default();

        let yesterday = vec![day(as_of - Duration::days(1), None, Some(8.0))];
        assert_eq!(assess(as_of, &yesterday, None, &t).signals.len(), 1);

        let too_old = vec![day(as_of - Duration::days(2), None, Some(8.0))];
        assert!(assess(as_of, &too_old, None, &t).signals.is_empty());
    }

    #[test]
    fn resting_hr_needs_a_baseline() {
        // Today's reading but no history: can't say if it's elevated, so no signal.
        let as_of = d("2026-11-15");
        let w = vec![day(as_of, Some(60.0), None)];
        assert!(assess(as_of, &w, None, &Thresholds::default())
            .signals
            .is_empty());
    }
}