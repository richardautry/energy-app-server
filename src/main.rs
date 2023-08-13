use axum::{
    routing::{get, post},
    http::StatusCode,
    response::IntoResponse,
    Json,
    Router
};
use serde::{Deserialize, Serialize};
use tplinker::{datatypes::DeviceData, devices};
use std::net::SocketAddr;
use tplinker::{
    discovery::discover,
    devices::Device,
    datatypes::SysInfo
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(root))
        .route("/users", post(create_user))
        .route("/devices", get(device_data));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn create_user(
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    let user = User {
        id: 1337,
        username: payload.username,
    };

    (StatusCode::CREATED, Json(user))
}

async fn device_data() -> Json<Vec<SysInfo>> {
    let mut devices_data: Vec<SysInfo> = Vec::new();
    for (addr, data) in discover().unwrap() {
        let device: Device = Device::from_data(addr, &data);
        let sysInfo: &SysInfo = data.sysinfo();
        devices_data.push(sysInfo.clone());
    }
    axum::Json(devices_data)
}

#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}

// #[derive(Serialize)]
// struct DeviceDataSerialized {
//     mac: String,
// }