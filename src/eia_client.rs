use serde::Deserialize;
use reqwest;
use std::fs::File;
use std::io::BufReader;
use std::env::current_dir;

#[derive(Deserialize, Debug)]
struct User {
    login: String,
    id: u32,
}

#[derive(Deserialize, Debug)]
struct EIAData {
    period: String,
    respondent: String,
    // TODO: Translate respondent-name to respondent_name before deserialization
    // Explore using custom serde Deserialization logic to translate this value before deserialization
    // Otherwise, will have to change values in the payload string in separate function
    #[serde(rename(deserialize = "respondent-name"))]
    respondent_name: String,
    r#type: String,
    #[serde(rename(deserialize = "type-name"))]
    type_name: String,
    value: u64,
    // TODO: Translate this from value-units to value_units
    #[serde(rename(deserialize = "value-units"))]
    value_units: String
}

#[derive(Deserialize, Debug)]
struct EIAResponse {
    total: u64,
    #[serde(rename(deserialize = "dateFormat"))]
    date_format: String,
    frequency: String,
    data: Vec<EIAData>,
    description: String
}

#[derive(Deserialize, Debug)]
struct EIARequest {
    command: String,
    params: EIARequestParams,
}

#[derive(Deserialize, Debug)]
struct EIARequestParams {
    api_key: String,
    frequency: String,
    data: Vec<String>,
    facets: EIARequestParamsFacets,
    start: String,
    end: String
}

#[derive(Deserialize, Debug)]
struct EIARequestParamsFacets {
    respondent: Vec<String>
}

#[derive(Deserialize, Debug)]
struct EIAJsonResult {
    response: EIAResponse,
    request: EIARequest,
    #[serde(rename(deserialize = "apiVersion"))]
    api_version: String
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
    

    let frequency = String::from("&frequency=local-hourly");
    let data = String::from("&data[0]=value");
    let facets = String::from("&facets[respondent][]=MIDA");
    let start_date = String::from("&start=2023-07-14T00:00:00-04:00");
    let end_date = String::from("&end=2023-07-17T00:00:00-04:00");

    let full_url = eia_url + &String::from("?api_key=") + &api_key + &frequency + &data + &facets + &start_date + &end_date;
    
    let response = reqwest::get(&full_url).await?;
    
    let ser_data = response.json::<EIAJsonResult>().await;

    match ser_data {
        Ok(result) => println!("{:?}", result),
        Err(e) => println!("{}", e)
    }

    println!("Full URL: {}", full_url);
    // println!("{}", response.status());
    // println!("{}", response.text().await?);
    // println!("{:?}", ser_data);

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