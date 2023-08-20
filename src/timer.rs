use axum::{
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use tokio::time;
use crate::device::{
    turn_off_device,
    turn_on_device,
    SimpleDeviceData
};

#[derive(Serialize, Deserialize)]
enum TurnOnOrOff {
    #[serde(rename = "turn_on")]
    TurnOn,
    #[serde(rename = "turn_off")]
    TurnOff
}

#[derive(Serialize, Deserialize)]
pub struct SetTimerData {
    alias: String,
    mac: String,
    length_ms: u64,
    turn_on_or_off: TurnOnOrOff
}

async fn start_timer(length_ms: u64) {
    let duration: time::Duration = time::Duration::from_millis(length_ms);
    let now: SystemTime = SystemTime::now();
    let mut interval = time::interval(time::Duration::from_secs(1));

    while now.elapsed().expect("").as_secs() < duration.as_secs() {
        interval.tick().await;
        println!("tick {}", now.elapsed().unwrap().as_secs());
    }
}

pub async fn start_timer_device (
    Json(payload): Json<SetTimerData>,
) -> (StatusCode, Json<bool>) {
    let duration = payload.length_ms;
    tokio::spawn(async move {
        start_timer(duration).await;
        let simple_device_data = SimpleDeviceData {
            alias: payload.alias,
            mac: payload.mac
        };
        match payload.turn_on_or_off {
            TurnOnOrOff::TurnOff => turn_off_device(Json(simple_device_data)).await,
            TurnOnOrOff::TurnOn => turn_on_device(Json(simple_device_data)).await
        }
        
    });
    (StatusCode::OK, Json(true))
}