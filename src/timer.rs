use axum::{
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::{time::SystemTime};
use tokio::time;
use crate::device::{
    turn_off_device,
    SimpleDeviceData
};

#[derive(Serialize, Deserialize)]
pub struct SetTimerData {
    alias: String,
    mac: String,
    // #[serde(with = "ts_seconds_option")]
    // start_date_time: Option<DateTime<Utc>>,
    // end_date_time: Option<DateTime<Utc>>
    length_ms: u64
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
        turn_off_device(Json(SimpleDeviceData {
            alias: payload.alias,
            mac: payload.mac
        })).await;
    });
    (StatusCode::OK, Json(true))
}