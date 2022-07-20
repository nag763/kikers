pub(crate) mod error;

use async_std::{
    fs::File,
    io::{copy, Cursor},
};
use clap::{Parser, Subcommand};
use dotenv::dotenv;
use error::CliError;
use ffb_structs::{api_token, bet, bookmaker, club, game, info, info::Model as Info, league, odd};
use scraper::{Html, Selector};
use std::process::{ExitCode, Termination};
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
    Leagues {
        #[clap(arg_enum)]
        indexable: Indexable,
    },
    Clubs {
        #[clap(arg_enum)]
        indexable: Indexable,
    },
    Fixtures {
        #[clap(default_value = "0")]
        day_diff: i64,
    },
    IndexOdds,
    ValidateBets,
    Odds {
        #[clap(default_value = "0")]
        day_diff: i64,
    },
    Bookmakers,
    ApiToken {
        token: String,
    },
    News,
}

#[derive(clap::ArgEnum, Debug, Clone)]
enum Fetchable {
    Model,
    Logo,
}

#[derive(clap::ArgEnum, Debug, Clone)]
enum Indexable {
    Model,
    Logo,
    Index,
}

#[tokio::main]
async fn main() -> ExitCode {
    let now = std::time::Instant::now();
    env_logger::init();
    match run_main().await {
        Ok(_) => {
            println!(
                "Process exited in {} seconds with success",
                now.elapsed().as_secs()
            );
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("An error happened : {}", err);
            eprintln!("The application finished with return code {:?}", err);
            err.report()
        }
    }
}

async fn run_main() -> Result<(), CliError> {
    dotenv().ok();
    debug!("Environnement initialized");

    let args = Args::parse();
    debug!("Args parsed : {:#?}", args);
    match args.get {
        Getter::Leagues { indexable } => match indexable {
            Indexable::Model => fetch_leagues().await?,
            Indexable::Logo => fetch_leagues_logo().await?,
            Indexable::Index => league::Entity::index().await?,
        },
        Getter::Clubs { indexable } => match indexable {
            Indexable::Model => fetch_clubs().await?,
            Indexable::Logo => fetch_clubs_logo().await?,
            Indexable::Index => index_clubs().await?,
        },
        Getter::Fixtures { day_diff } => fetch_fixtures(day_diff).await?,
        Getter::Bookmakers => fetch_bookmakers().await?,
        Getter::ApiToken { token } => api_token::Entity::register(&token)?,
        Getter::Odds { day_diff } => fetch_odds(day_diff).await?,
        Getter::IndexOdds => odd::Entity::index().await?,
        Getter::News => fetch_news().await?,
        Getter::ValidateBets => bet::Entity::validate_bets().await?,
    }
    Ok(())
}

async fn fetch_news() -> Result<(), CliError> {
    let res = reqwest::get("https://old.reddit.com/r/soccer/new/")
        .await?
        .text()
        .await?;
    let fragment = Html::parse_fragment(&res);
    let selector = Selector::parse(r#"a.title.may-blank"#).unwrap();
    let input = fragment.select(&selector);
    let mut infos: Vec<Info> = Vec::new();
    for elt in input {
        if let Some(href) = elt.value().attr("href") {
            infos.push(Info {
                title: elt.inner_html(),
                href: href.to_string(),
            });
            if 10 < infos.len() {
                break;
            }
        }
    }
    info::Entity::store(infos)?;
    Ok(())
}

async fn fetch_leagues() -> Result<(), CliError> {
    debug!("Fetch leagues called");
    let res = call_api_endpoint("leagues".into()).await?;
    let mut storable: Vec<serde_json::Value> = Vec::new();
    for elt in res["response"].as_array().ok_or_else(|| {
        CliError::RequestError("Data received in the wrong format for the server".into())
    })? {
        storable.push(elt["league"].clone());
    }
    league::Entity::store(&serde_json::to_string(&storable)?).await?;
    debug!("League entity stored");
    Ok(())
}

async fn index_clubs() -> Result<(), CliError> {
    club::Entity::index().await?;
    Ok(())
}

async fn fetch_clubs() -> Result<(), CliError> {
    debug!("Fetch clubs called");
    club::Entity::store().await?;
    Ok(())
}

async fn fetch_leagues_logo() -> Result<(), CliError> {
    debug!("Fetch logos called");
    let leagues_logos: Vec<String> = league::Entity::get_all_leagues_logo().await?;
    bulk_download_files(leagues_logos).await?;
    league::Entity::replace_all_league_logo().await?;
    Ok(())
}

async fn fetch_clubs_logo() -> Result<(), CliError> {
    debug!("Fetch countries logo called");
    let clubs_logo: Vec<String> = club::Entity::get_logos().await?;
    bulk_download_files(clubs_logo).await?;
    club::Entity::replace_all_club_logo().await?;
    Ok(())
}

async fn bulk_download_files(files_uri: Vec<String>) -> Result<(), CliError> {
    let cooldown: u64 = std::env::var("BULK_DOWNLOAD_COOLDOWN")?.parse()?;
    let size: usize = std::env::var("BULK_DOWNLOAD_SIZE")?.parse()?;
    for (i, logo) in files_uri.into_iter().enumerate() {
        if i % size == 0 {
            debug!("Sleep requested");
            tokio::time::sleep(tokio::time::Duration::from_secs(cooldown)).await;
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
        })
        .await?;
    }
    Ok(())
}

async fn fetch_bookmakers() -> Result<(), CliError> {
    let res = call_api_endpoint("odds/bookmakers".into()).await?;
    let response: String = res["response"].to_string();
    bookmaker::Entity::store(&response).await?;
    debug!("Games stored");
    Ok(())
}

async fn fetch_odds(day_diff: i64) -> Result<(), CliError> {
    let now = chrono::Utc::now();
    let mut date_to_fetch = (now + chrono::Duration::days(day_diff)).to_rfc3339();
    date_to_fetch.truncate(10);
    let main_bookmaker_id: u32 = bookmaker::Entity::get_main_bookmaker_id()
        .await?
        .ok_or(CliError::NoMainBookmaker)?;
    let mut page: u64 = 1;
    loop {
        info!(
            "Page {} being called for bookmaker id {} and date {}",
            page, main_bookmaker_id, date_to_fetch
        );
        let res = call_api_endpoint(format!(
            "odds?date={}&bookmaker={}&bet=1&page={}",
            date_to_fetch, main_bookmaker_id, page
        ))
        .await?;
        let response: String = res["response"].to_string();
        let total_pages: Option<u64> = res["paging"]["total"].as_u64();
        odd::Entity::store(&response).await?;
        match total_pages {
            Some(v) if v != page => {
                info!("Page {}/{} successfully stored", page, v);
                page += 1;
            }
            _ => break,
        }
    }
    debug!("Odds stored");
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
    let token: String = api_token::Entity::get_token()?;
    info!("Endpoint called : {}", endpoint.as_str());
    let res = client
        .get(std::env::var("API_PROVIDER")? + endpoint.as_str())
        .header("x-rapidapi-host", "api-football-v1.p.rapidapi.com")
        .header("x-rapidapi-key", &token)
        .send()
        .await?;

    if let Some(rem) = res.headers().get("X-RateLimit-requests-Remaining") {
        let remaining_calls: i32 = rem.to_str().unwrap().parse()?;
        info!(
            "Number of calls remaining for token {} : {}",
            token, remaining_calls
        );
        api_token::Entity::update_threshold(&token, remaining_calls)?;
    }

    let value: serde_json::Value = res.json::<serde_json::Value>().await?;
    info!("Endpoint successfully reached");
    trace!("Response : {:#?}", value);
    Ok(value)
}
