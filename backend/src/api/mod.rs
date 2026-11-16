
use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::{header, HeaderValue, Method, StatusCode},
    routing::get,
    Json, Router,
};
use chrono::{Duration, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use tokio::try_join;
use tower_http::cors::CorsLayer;

use crate::{
    client::IntervalsClient,
    readiness::{self, Readiness, Thresholds},
    summary::{self, Summary},
};

struct AppState {
    client: IntervalsClient,
}

#[derive(Deserialize)]
struct DashboardQuery {
    days: Option<i64>,
}

#[derive(Serialize)]
struct DashboardResponse {
    as_of: NaiveDate,
    summary: Summary,
    readiness: Readiness,
}

pub fn router(client: IntervalsClient) -> Router {
    let state = Arc::new(AppState { client });

    let cors = CorsLayer::new()
        .allow_origin(HeaderValue::from_static("http://localhost:5173"))
        .allow_methods([Method::GET])
        .allow_headers([header::CONTENT_TYPE]);

    Router::new()
        .route("/api/dashboard", get(dashboard))
        .layer(cors)
        .with_state(state)
}

async fn dashboard(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DashboardQuery>,
) -> Result<Json<DashboardResponse>, (StatusCode, String)> {
    let days = params.days.unwrap_or(28);

    if !(1..=365).contains(&days) {
        return Err((
            StatusCode::BAD_REQUEST,
            "days must be between 1 and 365".into(),
        ));
    }

    let as_of = Local::now().date_naive();

    // Fetch at least 28 days for readiness and recovery baselines.
    let oldest = as_of - Duration::days(days.max(28) - 1);

    let (wellness, activities) = try_join!(
        state.client.wellness(oldest, as_of),
        state.client.activities(oldest, as_of),
    )
    .map_err(|err| {
        (
            StatusCode::BAD_GATEWAY,
            format!("Failed to fetch Intervals.icu data: {err}"),
        )
    })?;

    let summary = summary::build(as_of, days, &wellness, &activities);

    let readiness = readiness::assess(
        as_of,
        &wellness,
        summary.fitness.as_ref(),
        &Thresholds::default(),
    );

    Ok(Json(DashboardResponse {
        as_of,
        summary,
        readiness,
    }))
}
