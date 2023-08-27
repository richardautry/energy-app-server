mod timer;
mod device;
mod eia_client;

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
use eia_client::get_eia_data;

#[tokio::main]
async fn main() {
    get_eia_data().await;

    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(root))
        .route("/devices", get(device_data))
        .route("/devices/turn_on", post(turn_on_device))
        .route("/devices/turn_off", post(turn_off_device))
        .route("/devices/set_timer", post(start_timer_device));
        //.route("/sleep/:id", get(move |path| sleep_and_print(path, &tx)));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Welcome to EnergySync"
}