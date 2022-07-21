//! This crate is a set of commands used to modify the local data from either
//! remote endpoints or existing local data.
//!
//! These commands have to be used with the help of a crontab and need to be
//! executed regulary in order to keep the application data up to date.

use async_std::{
    fs::File,
    io::{copy, Cursor},
};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use dotenv::dotenv;
use error::CliError;
use ffb_structs::{api_token, bet, bookmaker, club, game, info, info::Model as Info, league, odd};
use scraper::{Html, Selector};
use std::process::{ExitCode, Termination};
use url::Url;

#[macro_use]
extern crate log;

#[macro_use]
extern crate derive_more;

/// Crate to handle common applicative errors.
pub(crate) mod error;

/// Cli arguments,
/// One getter is defined so far,
/// no options are available.
#[derive(Parser, Debug)]
struct Args {
    /// The getter is the list of models that
    /// can be fetched.
    #[clap(subcommand)]
    get: Getter,
}

/// The getter subcommand defines which model
/// or struct we will want to fetch or manipulate.
#[derive(Subcommand, Debug)]
enum Getter {
    /// The leagues used in the application.
    ///
    /// They are fetched from remote.
    Leagues {
        #[clap(arg_enum)]
        indexable: Indexable,
    },
    /// The clubs used in the application.
    ///
    /// Unlike other models, they are taken from the existing fixtures.
    Clubs {
        #[clap(arg_enum)]
        indexable: Indexable,
    },
    Fixtures {
        /// The day difference
        ///
        /// This fetches if nothing is passed or
        /// equals to 0, the games of the day
        /// will be fetched, if it is equals to
        /// -1, the days of yesterday will be fetched,
        /// and so on.
        #[clap(default_value = "0")]
        day_diff: i64,
    },
    /// Index the odds.
    ///
    /// Usually it has to be used after the command [Getter::Odds] has been
    /// called.
    IndexOdds,
    /// Validate the bets.
    ///
    /// This arg is meant give points of the betters if they predicted
    /// correctly.
    ValidateBets,
    /// Fetching the odds.
    ///
    /// This adds the probabilities to win of each teams
    /// to the fixtures, so that the users can bet easily.
    Odds {
        /// Day diff, similar to [Getter::Fixtures::day_diff]
        #[clap(default_value = "0")]
        day_diff: i64,
    },
    /// Fetch the bookmaker (or the source of the odds).
    Bookmakers,
    /// Add an api token to the list.
    ///
    /// The model of usage of the API tokens is to use always the one with the
    /// most remaining calls. Since the API I am using is a freemium, the goal
    /// is to use register several and use them all without going through the
    /// threshold.
    ApiToken {
        /// The token to add to the list of tokens.
        token: String,
    },
    /// Fetch the latest transfer news. The source is the soccer subreddit,
    /// the frequency can be from every 1 to every 5 minutes.
    News,
}

/// A fetchable struct is a remote structure from the API Provider.
///
/// A fetchable struct is  both containing a fetchable model (its data) and a
/// logo. Both have to be downloaded separatly.
#[derive(clap::ArgEnum, Debug, Clone)]
enum Fetchable {
    /// Fetch only the model (or data) associed to the super command.
    Model,
    /// Fetch only the logo asocied to the super command.
    Logo,
}

/// An indexable structure is a structure that has to be indexed on ES services.
///
/// This way it can be searchable with the criteria defined within the structure
/// .
#[derive(clap::ArgEnum, Debug, Clone)]
enum Indexable {
    /// Fetch only the model (or data) associed to the super command.
    Model,
    /// Fetch only the logo asocied to the super command.
    Logo,
    /// Index the structure within the research engine.
    Index,
}

#[tokio::main]
async fn main() -> ExitCode {
    env_logger::init();
    let now = std::time::Instant::now();
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

/// Fetch the latest news from reddit.
///
/// This method uses the data from reddit.com/r/soccer and stores them.
///
/// Is called with [Getter::News].
async fn fetch_news() -> Result<(), CliError> {
    let res = reqwest::get("https://old.reddit.com/r/soccer/new/")
        .await?
        .text()
        .await?;
    debug!("News successfully fetched from old reddit");
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
                debug!("All infos successfully fetched from remote");
                break;
            }
        }
    }
    info::Entity::store(infos)?;
    debug!("Infos are now stored");
    Ok(())
}

/// Fetch the leagues from the API provider.
///
/// Has to be called with [Getter::Leagues] variant [Indexable::Model].
async fn fetch_leagues() -> Result<(), CliError> {
    debug!("Fetch leagues called");
    let res = call_api_endpoint("leagues".into()).await?;
    let mut storable: Vec<serde_json::Value> = Vec::new();
    for elt in res["response"].as_array().ok_or_else(|| {
        CliError::RequestError("Data received in the wrong format for the server".into())
    })? {
        // The response contains both the league and country information,
        // the country information is ignored by this process.
        storable.push(elt["league"].clone());
    }
    debug!("League entity successfully retrieved from response");
    league::Entity::store(&serde_json::to_string(&storable)?).await?;
    debug!("League entity stored");
    Ok(())
}

/// Index the clubs within the ES engine.
///
/// Has to be called with [Getter::Clubs] variant [Indexable::Index].
async fn index_clubs() -> Result<(), CliError> {
    debug!("Start of club indexing");
    club::Entity::index().await?;
    debug!("Club successfully indexed");
    Ok(())
}

/// Fetch the clubs stored on the current database.
///
/// This method is using the locally stored clubs on the database fixtures
/// to store them as a separate structure.
///
/// Has to be called with [Getter::Clubs].
async fn fetch_clubs() -> Result<(), CliError> {
    debug!("Fetch clubs called");
    club::Entity::store().await?;
    Ok(())
}

/// Fetch the leagues logo.
///
/// Has to be called with [Getter::Leagues] variant [Indexable::Logo].
async fn fetch_leagues_logo() -> Result<(), CliError> {
    debug!("Fetch logos called");
    let leagues_logos: Vec<String> = league::Entity::get_all_leagues_logo().await?;
    bulk_download_files(leagues_logos).await?;
    league::Entity::replace_all_league_logo().await?;
    Ok(())
}

/// Fetch the clubs logo.
///
/// Has to be called with [Getter::Clubs] variant [Indexable::Logo].
async fn fetch_clubs_logo() -> Result<(), CliError> {
    debug!("Fetch countries logo called");
    let clubs_logo: Vec<String> = club::Entity::get_logos().await?;
    bulk_download_files(clubs_logo).await?;
    club::Entity::replace_all_club_logo().await?;
    Ok(())
}

/// Internal method to download a bulk of files from a remote endpoint.
///
/// This method is using the environment variables `BULK_DOWNLOAD_COOLDOWN`,
/// `ASSETS_LOCAL_PATH`, and `BULK_DOWNLOAD_SIZE` to respectivivly know
///  * what should be the cooldown between two downloads.
///  * where should the downloaded assets be stored.
///  * how many items should be downloaded before cooling down.
///
///  This method is multithreaded to win the most time possible, hence why it is
///  using a cooldown feature, which would otherwise trigger a
///  `Too many requests` status on the API side.
///
///  # Arguments
///
///  - files_uri : the uri of files, they will be stored in what will correspond
///  to their relative remote URI path preceeded with the value of
///  `ASSETS_LOCAL_PATH`.
async fn bulk_download_files(files_uri: Vec<String>) -> Result<(), CliError> {
    let cooldown: u64 = std::env::var("BULK_DOWNLOAD_COOLDOWN")?.parse()?;
    let size: usize = std::env::var("BULK_DOWNLOAD_SIZE")?.parse()?;
    for (i, logo) in files_uri.into_iter().enumerate() {
        // When we reach the bulk_max_download upload, we request a sleep before making subsequent
        // calls
        if i % size == 0 {
            debug!("Sleep requested");
            tokio::time::sleep(tokio::time::Duration::from_secs(cooldown)).await;
        }
        debug!("All threads created, starting the fetching of the remote assets");
        async_std::task::spawn(async move {
            let assets_path: String = std::env::var("ASSETS_LOCAL_PATH")?;
            let url: Url = Url::parse(&logo)?;
            debug!("URL of remote file : {:?}", url);
            let file_name = url.path();
            let resp = reqwest::get(logo).await?;
            if resp.status().is_success() {
                let mut content = Cursor::new(resp.bytes().await?);
                let local_file_path: String = format!("{}/{}", assets_path, file_name);
                debug!("File is about to be stored at {}", &local_file_path);
                let mut out = File::create(&local_file_path).await?;
                copy(&mut content, &mut out).await?;
                debug!("File {} created with success", file_name);
                Ok(())
            } else {
                debug!("The following file prvocked an http error : {}", url);
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

/// Fetch the bookmakers.
/// Has to be called with [Getter::Bookmakers].
async fn fetch_bookmakers() -> Result<(), CliError> {
    let res = call_api_endpoint("odds/bookmakers".into()).await?;
    let response: String = res["response"].to_string();
    debug!("Response to bookmaker endpoint is succesful, entity will be stored.");
    bookmaker::Entity::store(&response).await?;
    debug!("Bookmakers stored");
    Ok(())
}

/// Fetch the odds associed with each fixtures for the given day_diff.
///
/// Has to be called with [Getter::Odds].
///
/// # Arguments
///
/// - day_diff : The day we want to fetch the odds of the fixtures, as a
/// difference of today (ie. yesteday = -1, tomorow =1, today =0, ...).
async fn fetch_odds(day_diff: i64) -> Result<(), CliError> {
    let now: DateTime<Utc> = Utc::now();
    let date_diff: DateTime<Utc> = now + chrono::Duration::days(day_diff);
    let mut date_to_fetch = date_diff.to_rfc3339();
    // RFC 3339 format has to be truncated of its last 10 chars to obtain
    // a date such `YYYY-MM-DD`
    date_to_fetch.truncate(10);
    let main_bookmaker_id: u32 = bookmaker::Entity::get_main_bookmaker_id()
        .await?
        .ok_or(CliError::NoMainBookmaker)?;
    // The result is most of the time paginated, so we need to ensure to get the
    // results from each page, the number of page is unknwon before making the
    // first call, so we need to loop until our page number equals the last one
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
        debug!(
            "Remoe end point called successfully for page number #{}",
            page
        );
        let response: String = res["response"].to_string();
        let total_pages: Option<u64> = res["paging"]["total"].as_u64();
        // We then store the odds, who are as String
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

/// Fetch the remote fixtures.
///
/// Has to be called with [Getter::Fixtures] variant [Fetchable::Model].
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

/// Calls the remote API endpoint.
///
/// Be aware that it is using the `API_PROVIDER` environment variable.
///
/// The final URL that will be called will then be `API_PROVIDER` + `endpoint`.
///
/// # Arguments :
/// * endpoint : The endpoint to call, the endpoint.
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

    // Part to know whether there are still calls to be made with this token
    if let Some(rem) = res.headers().get("X-RateLimit-requests-Remaining") {
        let remaining_calls: i32 = rem.to_str().unwrap().parse()?;
        info!(
            "Number of calls remaining for token {} : {}",
            &token, remaining_calls
        );
        api_token::Entity::update_threshold(&token, remaining_calls)?;
    } else {
        warn!(
            "The number of calls remaining for the token {} couldn't have been determined",
            &token
        );
    }

    let value: serde_json::Value = res.json::<serde_json::Value>().await?;
    info!("Endpoint successfully reached");
    trace!("Response : {:#?}", value);
    Ok(value)
}
