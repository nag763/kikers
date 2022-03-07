pub mod error;

use error::CliError;
use dotenv::dotenv;

extern crate redis;

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
    let client = redis::Client::open(std::env::var("REDIS_URL")?)?;
    let mut con = client.get_connection()?;
    let client = reqwest::Client::builder()
        .build()?;
    let res = client
        .get(std::env::var("API_PROVIDER")? + "/leagues?current=true")
        .header("x-rapidapi-host", "api-football-v1.p.rapidapi.com")
        .header("x-rapidapi-key", std::env::var("API_TOKEN")?)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    redis::cmd("SET").arg("leagues").arg(res.to_string()).query(&mut con)?;
    Ok(())
}

