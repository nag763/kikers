pub mod error;

use clap::{Parser, Subcommand};
use dotenv::dotenv;
use error::CliError;

extern crate redis;

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    get: Getter,
}

#[derive(Subcommand)]
enum Getter {
    Leagues,
    Fixtures,
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
    let now = chrono::Utc::now();
    let mut date_now: String = now.to_rfc3339();
    date_now.truncate(10);
    let mut date_yesterday = (now - chrono::Duration::days(1)).to_rfc3339();
    date_yesterday.truncate(10);
    let mut date_tomorow = (now + chrono::Duration::days(1)).to_rfc3339();
    date_tomorow.truncate(10);

    let args = Args::parse();
    let client = redis::Client::open(std::env::var("REDIS_URL")?)?;
    let mut con = client.get_connection()?;
    let client = reqwest::Client::builder().build()?;
    let map_endpoint_path: Vec<(String, String)> = match args.get {
        Getter::Leagues => vec![("leagues?current=true".into(), "leagues".into())],
        Getter::Fixtures => vec![(
            format!("fixtures?date={}", date_now),
            format!("fixtures-{}", date_now),
        ),
        (
            format!("fixtures?date={}", date_yesterday),
            format!("fixtures-{}", date_yesterday),
        ),
        (
            format!("fixtures?date={}", date_tomorow),
            format!("fixtures-{}", date_tomorow),
        ),
        ],
    };
    for (endpoint, redis_path) in map_endpoint_path {
        let res = client
            .get(std::env::var("API_PROVIDER")? + endpoint.as_str())
            .header("x-rapidapi-host", "api-football-v1.p.rapidapi.com")
            .header("x-rapidapi-key", std::env::var("API_TOKEN")?)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        redis::cmd("SET")
            .arg(redis_path)
            .arg(res["response"].to_string())
            .query(&mut con)?;
    }
    Ok(())
}
