use serde::Deserialize;
use reqwest::Error;

#[derive(Deserialize, Debug)]
struct User {
    login: String,
    id: u32,
}

// TODO: Add EIA structs and calls here
async fn get_eia_data() -> Result<(), Error> {
    let request_url = format!("https://api/github.com/repos/{owner}/{repo}/stargazers",
                                owner = "rust-lang-nursery",
                                repo = "rust-cookbook");
    let response = reqwest::get(&request_url).await?;
    let users: Vec<User> = response.json().await?;
    Ok(())
}