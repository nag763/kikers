pub(crate) mod error;

use async_std::{
    fs::File,
    io::{copy, Cursor},
};
use clap::{Parser, Subcommand};
use dotenv::dotenv;
use error::CliError;
use ffb_structs::{country, game, league};
use url::Url;

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
    FetchLogo,
    Countries,
    Fixtures {
        #[clap(default_value = "0")]
        day_diff: i64,
    },
}

#[tokio::main]
async fn main() {
    let now = std::time::Instant::now();
    env_logger::init();
    std::process::exit(match run_main().await {
        Ok(_) => {
            println!(
                "Process exited in {} seconds with success",
                now.elapsed().as_secs()
            );
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
        Getter::FetchLogo => fetch_logo().await?,
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

async fn fetch_logo() -> Result<(), CliError> {
    debug!("Fetch logos called");
    let leagues_logos: Vec<String> = league::Entity::get_all_leagues_logo().await?;
    for (i, logo) in leagues_logos.into_iter().enumerate() {
        if i % 20 == 0 {
            debug!("Sleep requested");
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
        async_std::task::spawn(async move {
            let assets_path: String = std::env::var("ASSETS_LOCAL_PATH")?;
            let url = Url::parse(&logo)?;
            let file_name = url.path();
            let resp = reqwest::get(logo).await?;
            if resp.status().is_success() {
                let mut content = Cursor::new(resp.bytes().await?);
                let mut out = File::create(format!("{}/{}", assets_path, file_name)).await?;
                copy(&mut content, &mut out).await?;
                debug!("File {} created with success", file_name);
                Ok(())
            } else {
                Err(CliError::RequestError(format!(
                    "Request terminated with error : {} => {}",
                    resp.status(),
                    resp.text().await?
                )))
            }
        }).await?;
    }
    league::Entity::replace_all_league_logo().await?;
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
    game::Entity::store(&date_to_fetch, &response).await?;
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
