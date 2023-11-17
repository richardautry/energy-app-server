mod timer;
mod device;
mod eia_client;
mod energy_demand;
mod sync;

use axum::{
    routing::{get, post},
    Router,
    body::Body,
    http::Request,
    routing::any
};
use axum::extract::Host;
use timer::start_timer_device;
use device::{
    device_data,
    turn_on_device,
    turn_off_device,
};
use energy_demand::find_peak_hour_timeframe;
use sync::start_sync_with_energy_demand;
use tower::ServiceExt;
use mdns_sd::{ServiceDaemon, ServiceInfo};
use std::collections::HashMap;
use tokio::task;
use tokio::signal;
use tokio::select;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() {
    // get_eia_data().await;
    find_peak_hour_timeframe().await;

    let cancel_token = CancellationToken::new();
    let cloned_cancel_token = cancel_token.clone();

    // register_service().await;
    tokio::spawn(register_service(cloned_cancel_token));

    tracing_subscriber::fmt::init();

    // TODO: Make this service discoverable on local wifi

    let app = Router::new()
        .route("/", get(root))
        .route("/devices", get(device_data))
        .route("/devices/turn_on", post(turn_on_device))
        .route("/devices/turn_off", post(turn_off_device))
        .route("/devices/set_timer", post(start_timer_device))
        .route("/devices/sync_with_demand", post(start_sync_with_energy_demand));
        //.route("/sleep/:id", get(move |path| sleep_and_print(path, &tx)));

    // TODO: Add graceful shutdown to deregister mdns service
    // https://tokio.rs/tokio/topics/shutdown
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Welcome to EnergySync"
}

async fn register_service(cancel_token: CancellationToken) {
    // TODO: Basic idea works standalone. Need to turn this into a background thread/service
    // https://tokio.rs/tokio/tutorial/spawning

    // Create a daemon
    let mdns = ServiceDaemon::new().expect("Failed to create daemon");

    // Create a service info.
    let service_type = "_rust._tcp.local.";
    let instance_name = "myinstance";
    let mut full_name: String = String::new();
    full_name.push_str(service_type);
    full_name.push_str(instance_name);
    // let host_ipv4 = "192.168.1.12";
    let host_ipv4 = "192.168.1.12";
    // let host_name = "192.168.1.12.local.";
    let host_name = "192.168.1.12.local.";
    let port = 5200;
    let properties = [("property_1", "test"), ("property_2", "1234")];

    let my_service = ServiceInfo::new(
        service_type,
        instance_name,
        host_name,
        host_ipv4,
        port,
        &properties[..],
    ).unwrap();

    // Register with the daemon, which publishes the service.
    mdns.register(my_service).expect("Failed to register our service");
    println!("Finished registering");

    while true {
        select! {
            _ = cancel_token.cancelled() => {
                mdns.unregister(&full_name).unwrap();
                mdns.shutdown().unwrap();
            }
            _ = std::thread::sleep(std::time::Duration::from_secs(360));
        }
    }
    
    
}