use axum::{
    routing::{get, post},
    http::StatusCode,
    response::IntoResponse,
    Json,
    Router,
    debug_handler
};
use serde::{Deserialize, Serialize};
use tplinker::{
    datatypes::{DeviceData, SysInfo},
    devices,
    error::{Error, Result},
    capabilities::DeviceActions
};
use std::net::SocketAddr;
use tplinker::{
    discovery::discover,
    devices::Device,
};
use serde_json::json;


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(root))
        .route("/users", post(create_user))
        .route("/devices", get(device_data))
        .route("/devices/turn_on", post(turn_on_device));

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

async fn get_devices() -> Vec<Device> {
    let mut devices: Vec<Device> = Vec::new();
    for (addr, data) in discover().unwrap() {
        let device = Device::from_data(addr, &data);
        devices.push(device)
    }
    devices
}

async fn device_data() -> Json<Vec<SimpleDeviceData>> {
    let mut devices_data: Vec<SimpleDeviceData> = Vec::new();
    for (addr, data) in discover().unwrap() {
        let device: Device = Device::from_data(addr, &data);
        let sysInfo: &SysInfo = data.sysinfo();
        devices_data.push(
            SimpleDeviceData { alias: sysInfo.alias.clone(), mac: sysInfo.mac.clone() }
        );
    }
    axum::Json(devices_data)
}

fn check_command_error(value: &serde_json::Value, pointer: &str) -> Result<()> {
    if let Some(err_code) = value.pointer(pointer) {
        if err_code == 0 {
            Ok(())
        } else {
            Err(Error::from(format!("Invalid error code {}", err_code)))
        }
    } else {
        Err(Error::from(format!("Invalid response format: {}", value)))
    }
}

#[debug_handler]
async fn turn_on_device(
    Json(payload): Json<SimpleDeviceData>,
) -> (StatusCode, Json<bool>) {
    let mut result = false;
    let devices = get_devices().await;
    for device in devices {
        result = match device {
            Device::Unknown(device) => {
                let sys_info = match device.sysinfo() {
                    Ok(sys_info) => sys_info,
                    Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(false))
                };
                let command = json!({
                    "system": {"set_relay_state": {"state": 1}}
                }).to_string();
                let command_result = check_command_error(
                    &device.send(&command).unwrap(),
                    "/system/set_relay_state/err_code",
                );
                match command_result {
                    Ok(_) => true,
                    Err(_) => false,
                }
            },
            _ => false
        }
    }

    (StatusCode::OK, Json(result))
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

#[derive(Serialize, Deserialize)]
struct SimpleDeviceData {
    alias: String,
    mac: String
}