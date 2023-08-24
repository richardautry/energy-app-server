use serde::Deserialize;
use reqwest;
use serde::__private::de::IdentifierDeserializer;
// use tokio::io::BufReader;
// use tokio::fs::File;
use std::fs::File;
use std::io::BufReader;
use std::env::current_dir;

#[derive(Deserialize, Debug)]
struct User {
    login: String,
    id: u32,
}

#[derive(Deserialize)]
struct Config {
    api_key: String,
}

// TODO: Add EIA structs and calls here
pub async fn get_eia_data() -> Result<(), reqwest::Error> {
    // let request_url = format!("https://api/github.com/repos/{owner}/{repo}/stargazers",
    //                             owner = "rust-lang-nursery",
    //                             repo = "rust-cookbook");
    // let response = reqwest::get(&request_url).await?;
    // let users: Vec<User> = response.json().await?;
    let api_key = get_config_json().await.expect("").api_key;
    let eia_url: String = String::from("https://api.eia.gov/v2/electricity/rto/region-data/data/");
    let full_url = eia_url + &String::from("?api_key=") + &api_key;
    let response = reqwest::get(&full_url).await?;
    println!("Full URL: {}", full_url);
    println!("{}", response.status());

    Ok(())
}

async fn get_config_json() -> Result<Config, serde_json::Error> {
    let mut file_path = current_dir().unwrap();
    file_path.push("config.json");

    let config_file = File::open(file_path).unwrap();
    let reader = BufReader::new(config_file);

    let config = serde_json::from_reader(reader);

    config
}