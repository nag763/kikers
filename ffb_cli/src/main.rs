pub(crate) mod error;

use clap::{Parser, Subcommand};
use dotenv::dotenv;
use error::CliError;
use serde_json::json;
use ffb_structs::{league, country, games};

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    get: Getter,
}

#[derive(Subcommand)]
enum Getter {
    Leagues,
    Countries,
    Fixtures {
        #[clap(default_value = "0")]
        day_diff: i64,
    },
}

#[tokio::main]
async fn main() {
    env_logger::init();
    std::process::exit(match run_main().await {
        Ok(_) => {
            println!("Process exited with success");
            0
        }
        Err(err) => {
            eprintln!("An error happened : {}", err);
            eprintln!("The application finished with return code 1");
            1
        }
    });
}

async fn run_main() -> Result<(), CliError> {
    dotenv().ok();

    let args = Args::parse();
    match args.get {
        Getter::Leagues => fetch_leagues().await?,
        Getter::Countries => fetch_countries().await?,
        Getter::Fixtures { day_diff } => fetch_fixtures(day_diff).await?,
    }
    Ok(())
}

async fn fetch_leagues() -> Result<(), CliError> {
    let res = call_api_endpoint("leagues".into()).await?;
    let response : String = res["response"].to_string();
    league::Entity::store(&response)?;
    Ok(())
}

async fn fetch_countries() -> Result<(), CliError> {
    let res = call_api_endpoint("countries".into()).await?;
    let response : String = res["response"].to_string();
    country::Entity::store(&response)?;
    Ok(())
}

async fn fetch_fixtures(day_diff: i64) -> Result<(), CliError> {
    let now = chrono::Utc::now();
    let mut date_diff = (now + chrono::Duration::days(day_diff)).to_rfc3339();
    date_diff.truncate(10);
    let res = call_api_endpoint(format!("fixtures?date={}", date_diff)).await?;
    let stored_fixture: serde_json::Value = json!(
        {
            "games": res["response"],
            "fetched_on": now.to_rfc2822(),
        }
    );
    games::Entity::store(&date_diff, &stored_fixture.to_string())?;
    Ok(())
}

async fn call_api_endpoint(endpoint: String) -> Result<serde_json::Value, CliError> {
    let client = reqwest::Client::builder().build()?;
    let value: serde_json::Value = client
        .get(std::env::var("API_PROVIDER")? + endpoint.as_str())
        .header("x-rapidapi-host", "api-football-v1.p.rapidapi.com")
        .header("x-rapidapi-key", std::env::var("API_TOKEN")?)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    Ok(value)
}
