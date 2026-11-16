//! Thin async client for the intervals.icu REST API.

use std::time::Duration;

use anyhow::{bail, Context, Result};
use chrono::NaiveDate;
use reqwest::{Method, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{json, Value};

use crate::calendar::NewEvent;
use crate::models::{Activity, Wellness};

pub struct IntervalsClient {
    http: reqwest::Client,
    /// `https://intervals.icu/api/v1/athlete/{id}`
    base: String,
    api_key: String,
}

/// Turns a non-2xx response into a readable error.
async fn ok_or_bail(resp: Response, what: &str) -> Result<Response> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }
    let body = resp.text().await.unwrap_or_default();
    if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        bail!("intervals.icu returned {status} for {what}: check INTERVALS_API_KEY ({body})");
    }
    bail!("intervals.icu returned {status} for {what}: {body}");
}

impl IntervalsClient {
    /// `athlete_id` can be `"0"`, which means "the athlete that owns this API key".
    pub fn new(api_key: String, athlete_id: &str) -> Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .context("building HTTP client")?;
        Ok(Self {
            http,
            base: format!("https://intervals.icu/api/v1/athlete/{athlete_id}"),
            api_key,
        })
    }

    pub async fn wellness(&self, oldest: NaiveDate, newest: NaiveDate) -> Result<Vec<Wellness>> {
        self.get_range("wellness", oldest, newest).await
    }

    pub async fn activities(&self, oldest: NaiveDate, newest: NaiveDate) -> Result<Vec<Activity>> {
        self.get_range("activities", oldest, newest).await
    }

    /// Creates events, or updates them if one with the same `external_id`
    /// already exists. Returns the raw JSON response (an array of events).
    pub async fn upsert_events(&self, events: &[NewEvent]) -> Result<Value> {
        self.send_json(Method::POST, "events/bulk?upsert=true", events)
            .await
    }

    /// Deletes events by our own `external_id`. Ids that don't exist are ignored.
    pub async fn delete_events_by_external_id(&self, external_ids: &[&str]) -> Result<Value> {
        let body: Vec<Value> = external_ids
            .iter()
            .map(|id| json!({ "external_id": id }))
            .collect();
        self.send_json(Method::PUT, "events/bulk-delete", &body).await
    }

    async fn get_range<T: DeserializeOwned>(
        &self,
        path: &str,
        oldest: NaiveDate,
        newest: NaiveDate,
    ) -> Result<Vec<T>> {
        let url = format!("{}/{}", self.base, path);
        let resp = self
            .http
            .get(&url)
            // intervals.icu uses HTTP Basic auth: the username is the literal
            // string "API_KEY" and the password is your key.
            .basic_auth("API_KEY", Some(&self.api_key))
            .query(&[("oldest", oldest.to_string()), ("newest", newest.to_string())])
            .send()
            .await
            .with_context(|| format!("request to {url} failed"))?;

        ok_or_bail(resp, path)
            .await?
            .json::<Vec<T>>()
            .await
            .with_context(|| format!("unexpected JSON shape from {path}"))
    }

    /// Sends a JSON body and returns the JSON reply (or `Null` for an empty one).
    async fn send_json<B: Serialize + ?Sized>(
        &self,
        method: Method,
        path: &str,
        body: &B,
    ) -> Result<Value> {
        let url = format!("{}/{}", self.base, path);
        let resp = self
            .http
            .request(method, &url)
            .basic_auth("API_KEY", Some(&self.api_key))
            .json(body)
            .send()
            .await
            .with_context(|| format!("request to {url} failed"))?;

        let text = ok_or_bail(resp, path)
            .await?
            .text()
            .await
            .with_context(|| format!("reading reply from {path}"))?;
        if text.trim().is_empty() {
            return Ok(Value::Null);
        }
        serde_json::from_str(&text)
            .with_context(|| format!("unexpected reply from {path}: {text}"))
    }
}