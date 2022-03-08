pub mod error;

use error::CliError;
use dotenv::dotenv;
use clap::{Parser, Subcommand};

extern crate redis;

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    get : Getter,
}

#[derive(Subcommand)]
enum Getter {
    Leagues,
    Fixtures

}

#[tokio::main]
async fn main() {
    env_logger::init();
    std::process::exit(match run_main().await {
        Ok(_) => {
            println!("Process exited with success");
            0
        },
        Err(err) => {
            eprintln!("An error happened : {}", err);
            eprintln!("The application finished with return code 1");
            1
        }
    });
}


async fn run_main() -> Result<(), CliError> {
    dotenv().ok();
    let mut date_now : String = chrono::Utc::now().to_rfc3339();
    date_now.truncate(10);
    let args = Args::parse();
    let client = redis::Client::open(std::env::var("REDIS_URL")?)?;
    let mut con = client.get_connection()?;
    let client = reqwest::Client::builder()
        .build()?;
    let map_endpoint_path : (String, String) = match args.get {
        Getter::Leagues => ("leagues?current=true".into(), "leagues".into()),
        Getter::Fixtures => (format!("fixtures?date={}", date_now), format!("fixtures-{}", date_now)),
    };
    let res = client
        .get(std::env::var("API_PROVIDER")? + map_endpoint_path.0.as_str())
        .header("x-rapidapi-host", "api-football-v1.p.rapidapi.com")
        .header("x-rapidapi-key", std::env::var("API_TOKEN")?)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    redis::cmd("SET").arg(map_endpoint_path.1).arg(res["response"].to_string()).query(&mut con)?;
    Ok(())
}

