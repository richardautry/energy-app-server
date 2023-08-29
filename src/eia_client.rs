use serde::Deserialize;
use reqwest;
use std::fs::File;
use std::io::BufReader;
use std::env::current_dir;
use chrono::prelude::*;
use chrono::Duration;

#[derive(Deserialize, Debug)]
struct User {
    login: String,
    id: u32,
}

#[derive(Deserialize, Debug)]
pub struct EIAData {
    pub period: String,
    respondent: String,
    #[serde(rename(deserialize = "respondent-name"))]
    respondent_name: String,
    pub r#type: String,
    #[serde(rename(deserialize = "type-name"))]
    type_name: String,
    pub value: u64,
    #[serde(rename(deserialize = "value-units"))]
    value_units: String
}

#[derive(Deserialize, Debug)]
pub struct EIAResponse {
    total: u64,
    #[serde(rename(deserialize = "dateFormat"))]
    date_format: String,
    frequency: String,
    pub data: Vec<EIAData>,
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
pub struct EIAJsonResult {
    pub response: EIAResponse,
    request: EIARequest,
    #[serde(rename(deserialize = "apiVersion"))]
    api_version: String
}

#[derive(Deserialize)]
struct Config {
    api_key: String,
}

pub async fn get_eia_data() -> Result<EIAJsonResult, reqwest::Error> {
    let api_key = get_config_json().await.expect("").api_key;
    let eia_url: String = String::from("https://api.eia.gov/v2/electricity/rto/region-data/data/");
    
    let frequency = String::from("&frequency=local-hourly");
    let data = String::from("&data[0]=value");
    let facets = String::from("&facets[respondent][]=MIDA");
    
    // TODO: Breakout datetime stuff into separate function
    let datetime_format: &str = "%Y-%m-%dT%H:%M:%S-04:00";

    let local_datetime: DateTime<Local> = Local::now();
    let local_start_datetime = local_datetime - Duration::days(3);
    let local_datetime_str: String = local_datetime.format(datetime_format).to_string();
    let local_start_datetime_str = local_start_datetime.format(datetime_format).to_string();

    let end_date = String::from(format!("&end={}", local_datetime_str));
    let start_date = String::from(format!("&start={}", local_start_datetime_str));

    println!("Current Datetime: {}", local_datetime_str);

    let full_url = eia_url + &String::from("?api_key=") + &api_key + &frequency + &data + &facets + &start_date + &end_date;
    
    let response = reqwest::get(&full_url).await?;
    
    let ser_data = response.json::<EIAJsonResult>().await;

    // match &ser_data {
    //     Ok(result) => println!("{:?}", result),
    //     Err(e) => println!("{}", e)
    // }

    // println!("Full URL: {}", full_url);
    // println!("{}", response.status());
    // println!("{}", response.text().await?);
    // println!("{:?}", ser_data);

    ser_data
}

async fn get_config_json() -> Result<Config, serde_json::Error> {
    let mut file_path = current_dir().unwrap();
    file_path.push("config.json");

    let config_file = File::open(file_path).unwrap();
    let reader = BufReader::new(config_file);

    let config = serde_json::from_reader(reader);

    config
}