# tricoach

A small Rust backend for an AI triathlon planner built on the
[intervals.icu](https://intervals.icu) API.

**Current step:** fetch your recent wellness data and activities and boil them
down to a compact summary (weekly load per swim/bike/run, CTL/ATL/TSB, HRV and
resting-HR trends, sleep). This summary is what we will later hand to the LLM.

## Setup

1. In intervals.icu go to **Settings -> Developer Settings** and create an API key.
2. Export it (keep it out of git, and never paste it into a chat):

   ```sh
   export INTERVALS_API_KEY="your-key-here"
   # optional; defaults to "0", meaning the athlete that owns the key
   export INTERVALS_ATHLETE_ID="i12345"
   ```

3. Run it:

   ```sh
   cargo run -- summary              # last 28 days, readable text
   cargo run -- summary --days 56    # longer history
   cargo run -- summary --json       # the JSON the LLM will receive
   ```

## Tests

```sh
cargo test
```

The summary logic is pure (no network), so the tests run offline.

## Layout

- `src/client.rs`  - HTTP client for intervals.icu (Basic auth, `API_KEY` as username)
- `src/models.rs`  - typed shapes of the API responses (all optional fields)
- `src/summary.rs` - turns raw records into the compact summary, plus tests
- `src/main.rs`    - CLI

## Roadmap

1. Fetch + summarise (this step)
2. Rules-based daily readiness check
3. Weekly plan generation with Gemini, pushed to your calendar after approval
4. Chat interface
