use chrono::Local;
use crate::energy_demand::find_peak_hour_timeframe;
use crate::device::{
    turn_off_device,
    turn_on_device,
    SimpleDeviceData
};
use axum::{
    http::StatusCode,
    Json,
};
use serde::de;
use tokio::time;

pub async fn start_sync_with_energy_demand(
    Json(payload): Json<SimpleDeviceData>,
) -> (StatusCode, Json<bool>) {
    // TODO:
    // Create async function that spawns a task which will
    // check the for the current time every hour (or 5 minutes maybe)
    // If the time is inside the high demand window, turn off the device
    // If the time is outside the high demand window, turn on the device
    // Check state before issuing command to avoid unnecessary commands
    tokio::spawn(async move {
        let mut interval = time::interval(time::Duration::from_secs(5 * 60));
        let demand_times = find_peak_hour_timeframe().await.unwrap();
        let start_time = demand_times.0;
        let end_time = demand_times.1;
        let alias = &payload.alias;
        let mac = &payload.mac;
        loop {
            interval.tick().await;
            let local_time = Local::now();
            println!("tick: Five minutes elapsed {}", local_time);

            if local_time > start_time && local_time < end_time {
                println!("Time inside high demand window. Turning device {} off", alias);
                let simple_device_data = SimpleDeviceData {
                    alias: alias.to_owned(),
                    mac: mac.to_owned()
                };
                turn_off_device(Json(simple_device_data)).await;
            } else {
                println!("Time outside high demand window. Turning device {} on", alias);
                let simple_device_data = SimpleDeviceData {
                    alias: alias.to_owned(),
                    mac: mac.to_owned()
                };
                turn_on_device(Json(simple_device_data)).await;
            }
        }
    });

    (StatusCode::OK, Json(true))
}