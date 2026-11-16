//! Writing planned workouts to the intervals.icu calendar.
//!
//! Events are created with the bulk "upsert" endpoint, matched on our own
//! `external_id`. That makes writes idempotent: sending the same event twice
//! updates it instead of duplicating it, and we can delete exactly what we
//! created without touching anything else on your calendar.

use chrono::NaiveDate;
use serde::Serialize;
use serde_json::Value;

/// External id of the throw-away event used by `write-test`. `--cleanup`
/// deletes by this id, so it can never touch a real workout.
pub const TEST_EXTERNAL_ID: &str = "tricoach-write-test";

/// A calendar event to create.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct NewEvent {
    /// "WORKOUT" for a planned session.
    pub category: String,
    /// Local time, e.g. `2026-11-16T06:00:00`.
    pub start_date_local: String,
    /// intervals.icu sport type: "Run", "Ride", "Swim", ...
    #[serde(rename = "type")]
    pub kind: String,
    pub name: String,
    /// intervals.icu workout text, e.g. `- 10m 65%`.
    pub description: String,
    pub external_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moving_time: Option<u32>,
}

/// A tiny, obviously fake workout for checking that writing works.
pub fn test_event(date: NaiveDate) -> NewEvent {
    NewEvent {
        category: "WORKOUT".into(),
        start_date_local: format!("{date}T06:00:00"),
        kind: "Run".into(),
        name: "tricoach write test (safe to delete)".into(),
        description: "- 10m 65%".into(),
        external_id: TEST_EXTERNAL_ID.into(),
        // Left unset on purpose: we want to see whether intervals.icu works
        // out the duration from the description text, since that is what the
        // real planner will rely on.
        moving_time: None,
    }
}

/// Best-effort extraction of event ids from an upsert response (an array of
/// event objects, each with a numeric `id`).
pub fn event_ids(response: &Value) -> Vec<i64> {
    response
        .as_array()
        .map(|events| {
            events
                .iter()
                .filter_map(|e| e.get("id").and_then(Value::as_i64))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serialises_with_the_field_names_intervals_icu_expects() {
        let date = NaiveDate::from_ymd_opt(2026, 11, 16).unwrap();
        let v = serde_json::to_value(test_event(date)).unwrap();
        assert_eq!(
            v,
            json!({
                "category": "WORKOUT",
                "start_date_local": "2026-11-16T06:00:00",
                "type": "Run",
                "name": "tricoach write test (safe to delete)",
                "description": "- 10m 65%",
                "external_id": "tricoach-write-test"
            })
        );
    }

    #[test]
    fn moving_time_is_included_only_when_set() {
        let date = NaiveDate::from_ymd_opt(2026, 11, 16).unwrap();
        let mut e = test_event(date);
        e.moving_time = Some(600);
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["moving_time"], json!(600));
    }

    #[test]
    fn extracts_ids_from_an_upsert_response() {
        let resp = json!([{"id": 33375903, "category": "WORKOUT"}, {"id": 7}]);
        assert_eq!(event_ids(&resp), vec![33375903, 7]);
    }

    #[test]
    fn unexpected_response_shapes_yield_no_ids_instead_of_failing() {
        assert!(event_ids(&json!({"error": "nope"})).is_empty());
        assert!(event_ids(&Value::Null).is_empty());
    }
}
