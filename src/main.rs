mod calendar;
mod client;
mod config;
mod models;
mod planner;
mod readiness;
mod summary;

use std::io::Write as _;
use std::path::PathBuf;

use anyhow::{ensure, Context, Result};
use chrono::{Duration, Local};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tricoach", about = "AI triathlon planner backed by intervals.icu")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Fetch recent data from intervals.icu and print a training summary
    Summary {
        /// How many days of weekly history to include
        #[arg(long, default_value_t = 28)]
        days: i64,
        /// Print JSON (the form we'll later hand to the LLM) instead of text
        #[arg(long)]
        json: bool,
    },
    /// Rules-based daily check: go, go easy, or rest
    Readiness {
        /// Print JSON instead of text
        #[arg(long)]
        json: bool,
    },
    /// Show the season skeleton: phase and target hours for every week until the race
    Plan {
        /// Path to the config file
        #[arg(long, default_value = "tricoach.toml")]
        config: PathBuf,
        /// Print JSON instead of text
        #[arg(long)]
        json: bool,
    },
    /// Check that writing to your calendar works, using one clearly labelled test workout
    WriteTest {
        /// Actually send it. Without this flag nothing is sent (dry run).
        #[arg(long)]
        send: bool,
        /// Leave the test workout on the calendar instead of deleting it afterwards
        #[arg(long, requires = "send")]
        keep: bool,
        /// Only delete a leftover test workout, then exit
        #[arg(long, conflicts_with_all = ["send", "keep"])]
        cleanup: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Summary { days, json } => run_summary(days, json).await,
        Command::Readiness { json } => run_readiness(json).await,
        Command::Plan { config, json } => run_plan(&config, json),
        Command::WriteTest {
            send,
            keep,
            cleanup,
        } => run_write_test(send, keep, cleanup).await,
    }
}

fn client_from_env() -> Result<client::IntervalsClient> {
    let api_key = std::env::var("INTERVALS_API_KEY")
        .context("set the INTERVALS_API_KEY environment variable to your intervals.icu API key")?;
    // "0" means "the athlete that owns this key".
    let athlete_id = std::env::var("INTERVALS_ATHLETE_ID").unwrap_or_else(|_| "0".into());
    client::IntervalsClient::new(api_key, &athlete_id)
}

async fn run_summary(days: i64, json: bool) -> Result<()> {
    ensure!(days >= 1, "--days must be at least 1");
    let client = client_from_env()?;

    let as_of = Local::now().date_naive();
    // Always fetch at least 28 days so the resting HR / sleep baselines work.
    let oldest = as_of - Duration::days(days.max(28) - 1);

    let (wellness, activities) = tokio::try_join!(
        client.wellness(oldest, as_of),
        client.activities(oldest, as_of)
    )?;

    let summary = summary::build(as_of, days, &wellness, &activities);
    if json {
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        print!("{summary}");
    }
    Ok(())
}

async fn run_readiness(json: bool) -> Result<()> {
    let client = client_from_env()?;

    let as_of = Local::now().date_naive();
    // 28 days of history for the resting HR baseline, plus today.
    let wellness = client.wellness(as_of - Duration::days(28), as_of).await?;

    let fitness = summary::latest_fitness(as_of, &wellness);
    let result = readiness::assess(
        as_of,
        &wellness,
        fitness.as_ref(),
        &readiness::Thresholds::default(),
    );

    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        print!("{result}");
    }
    Ok(())
}

/// Needs no network or API key: it only reads the config file.
fn run_plan(config_path: &std::path::Path, json: bool) -> Result<()> {
    let cfg = config::Config::load(config_path)?;
    let plan = planner::build_plan(&cfg, Local::now().date_naive());
    if json {
        println!("{}", serde_json::to_string_pretty(&plan)?);
    } else {
        print!("{plan}");
    }
    Ok(())
}

async fn run_write_test(send: bool, keep: bool, cleanup: bool) -> Result<()> {
    if cleanup {
        let client = client_from_env()?;
        let reply = client
            .delete_events_by_external_id(&[calendar::TEST_EXTERNAL_ID])
            .await?;
        println!("Delete reply from intervals.icu: {reply}");
        return Ok(());
    }

    let date = Local::now().date_naive() + Duration::days(1);
    let event = calendar::test_event(date);
    println!("Test workout:\n{}", serde_json::to_string_pretty(&event)?);

    if !send {
        println!("\nDry run: nothing was sent. Re-run with --send to create it.");
        return Ok(());
    }

    let client = client_from_env()?;
    let reply = client.upsert_events(std::slice::from_ref(&event)).await?;
    println!(
        "\nCreated. Event id(s): {:?}\nFull reply from intervals.icu:\n{}",
        calendar::event_ids(&reply),
        serde_json::to_string_pretty(&reply)?
    );

    if keep {
        println!("\nLeft on your calendar. Remove it later with: tricoach write-test --cleanup");
        return Ok(());
    }

    print!(
        "\nOpen your intervals.icu calendar and look for the test workout on {date}.\n\
         Press Enter to delete it (or Ctrl+C to keep it; --cleanup removes it later): "
    );
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut String::new())?;

    let deleted = client
        .delete_events_by_external_id(&[calendar::TEST_EXTERNAL_ID])
        .await?;
    println!("Deleted. Reply from intervals.icu: {deleted}");
    Ok(())
}