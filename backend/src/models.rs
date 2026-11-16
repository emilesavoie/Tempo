//! Raw shapes of what intervals.icu returns.
//!
//! Almost everything is `Option` on purpose: the API omits fields you don't
//! log (no HRV strap, no sleep tracker, a manually entered activity...), and
//! a missing field must not make the whole request fail. Unknown fields are
//! ignored by serde, so we only declare what we use.

// Some fields (readiness, sleep score, ...) are parsed now but only used by the
// upcoming readiness rules. Remove this once they are.
#![allow(dead_code)]

use serde::Deserialize;

/// One day of wellness data. `id` is the date as `YYYY-MM-DD`.
#[derive(Debug, Clone, Deserialize)]
pub struct Wellness {
    pub id: String,
    pub ctl: Option<f64>,
    pub atl: Option<f64>,
    #[serde(rename = "rampRate")]
    pub ramp_rate: Option<f64>,
    pub hrv: Option<f64>,
    #[serde(rename = "restingHR")]
    pub resting_hr: Option<f64>,
    #[serde(rename = "sleepSecs")]
    pub sleep_secs: Option<f64>,
    #[serde(rename = "sleepScore")]
    pub sleep_score: Option<f64>,
    pub readiness: Option<f64>,
}

/// A completed activity.
#[derive(Debug, Clone, Deserialize)]
pub struct Activity {
    /// Local start time, e.g. `2026-11-14T07:30:00`.
    pub start_date_local: Option<String>,
    pub name: Option<String>,
    /// intervals.icu sport type: "Ride", "Run", "Swim", "VirtualRide", ...
    #[serde(rename = "type")]
    pub kind: Option<String>,
    /// Seconds.
    pub moving_time: Option<f64>,
    /// Metres.
    pub distance: Option<f64>,
    pub icu_training_load: Option<f64>,
}
