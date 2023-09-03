mod timer;
mod device;
mod eia_client;
mod energy_demand;
mod sync;

use axum::{
    routing::{get, post},
    Router,
};
use timer::start_timer_device;
use device::{
    device_data,
    turn_on_device,
    turn_off_device,
};
use energy_demand::find_peak_hour_timeframe;
use sync::start_sync_with_energy_demand;

#[tokio::main]
async fn main() {
    // get_eia_data().await;
    find_peak_hour_timeframe().await;

    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(root))
        .route("/devices", get(device_data))
        .route("/devices/turn_on", post(turn_on_device))
        .route("/devices/turn_off", post(turn_off_device))
        .route("/devices/set_timer", post(start_timer_device))
        .route("/devices/sync_with_demand", post(start_sync_with_energy_demand));
        //.route("/sleep/:id", get(move |path| sleep_and_print(path, &tx)));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Welcome to EnergySync"
}