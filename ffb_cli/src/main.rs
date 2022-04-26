pub(crate) mod error;

use clap::{Parser, Subcommand};
use dotenv::dotenv;
use error::CliError;
use ffb_structs::{country, games, league};

#[macro_use]
extern crate log;

#[derive(Parser, Debug)]
struct Args {
    #[clap(subcommand)]
    get: Getter,
}

#[derive(Subcommand, Debug)]
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
    debug!("Environnement initialized");

    let args = Args::parse();
    debug!("Args parsed : {:#?}", args);
    match args.get {
        Getter::Leagues => fetch_leagues().await?,
        Getter::Countries => fetch_countries().await?,
        Getter::Fixtures { day_diff } => fetch_fixtures(day_diff).await?,
    }
    Ok(())
}

async fn fetch_leagues() -> Result<(), CliError> {
    debug!("Fetch leagues called");
    let res = call_api_endpoint("leagues".into()).await?;
    let response: String = res["response"].to_string();
    league::Entity::store(&response).await?;
    debug!("League entity stored");
    Ok(())
}

async fn fetch_countries() -> Result<(), CliError> {
    let res = call_api_endpoint("countries".into()).await?;
    let response: String = res["response"].to_string();
    country::Entity::store(&response).await?;
    debug!("Countries stored");
    Ok(())
}

async fn fetch_fixtures(day_diff: i64) -> Result<(), CliError> {
    let now = chrono::Utc::now();
    let mut date_to_fetch = (now + chrono::Duration::days(day_diff)).to_rfc3339();
    date_to_fetch.truncate(10);
    debug!("Date fetched : {}", date_to_fetch);
    let res = call_api_endpoint(format!("fixtures?date={}", &date_to_fetch)).await?;
    let response: String = res["response"].to_string();
    games::Entity::store(&date_to_fetch, &response).await?;
    debug!("Games stored");
    Ok(())
}

async fn call_api_endpoint(endpoint: String) -> Result<serde_json::Value, CliError> {
    let client = reqwest::Client::builder().build()?;
    info!("Endpoint called : {}", endpoint.as_str());
    let value: serde_json::Value = client
        .get(std::env::var("API_PROVIDER")? + endpoint.as_str())
        .header("x-rapidapi-host", "api-football-v1.p.rapidapi.com")
        .header("x-rapidapi-key", std::env::var("API_TOKEN")?)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    info!("Endpoint successfully reached");
    Ok(value)
}
