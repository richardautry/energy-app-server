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
    devices::{RawDevice},
    error::{Error as TPLinkerError, Result as TPLinkerResult},
    capabilities::DeviceActions,
};
use std::{net::SocketAddr, time::SystemTime};
use tplinker::{
    discovery::discover,
    devices::Device,
};
use serde_json::json;
use tokio::time;
use std::error;
use chrono::{Utc, Date};
use chrono::DateTime;
use chrono::offset::TimeZone;
use chrono::serde::ts_seconds_option;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Create channel
    // let (tx, rx) = tokio::sync::channel();

    let app = Router::new()
        .route("/", get(root))
        .route("/users", post(create_user))
        .route("/devices", get(device_data))
        .route("/devices/turn_on", post(turn_on_device))
        .route("/devices/turn_off", post(turn_off_device))
        .route("/devices/set_timer", post(start_timer_device));

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

fn check_command_error(value: &serde_json::Value, pointer: &str) -> TPLinkerResult<()> {
    if let Some(err_code) = value.pointer(pointer) {
        if err_code == 0 {
            Ok(())
        } else {
            Err(TPLinkerError::from(format!("Invalid error code {}", err_code)))
        }
    } else {
        Err(TPLinkerError::from(format!("Invalid response format: {}", value)))
    }
}

async fn turn_on_device(
    Json(payload): Json<SimpleDeviceData>,
) -> (StatusCode, Json<bool>) {
    turn_on_off_device(payload, true).await
}

async fn turn_off_device(
    Json(payload): Json<SimpleDeviceData>,
) -> (StatusCode, Json<bool>) {
    turn_on_off_device(payload, false).await
}

async fn turn_on_off_device (
    payload: SimpleDeviceData,
    turn_on: bool
) -> (StatusCode, Json<bool>) {
    let mut result = false;
    let devices = get_devices().await;
    let state_int: u8 = match turn_on {
        true => 1,
        false => 0
    };
    let device = get_device(&payload.mac).await.unwrap();
    result = match device {
        Device::Unknown(device) => {
            let sys_info = match device.sysinfo() {
                Ok(sys_info) => sys_info,
                Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(false))
            };
            match sys_info.mac == payload.mac {
                true => {
                    let command = json!({
                        "system": {"set_relay_state": {"state": state_int}}
                    }).to_string();
                    let command_result = check_command_error(
                        &device.send(&command).unwrap(),
                        "/system/set_relay_state/err_code",
                    );
                    match command_result {
                        Ok(_) => true,
                        Err(_) => false,
                    }
                }
                false => false
            }
        },
        _ => false
    };

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

// #[derive(Serialize, Deserialize)]
// struct SetTimerData {
//     alias: String,
//     mac: String,
//     // #[serde(with = "ts_seconds_option")]
//     // start_date_time: Option<DateTime<Utc>>,
//     // end_date_time: Option<DateTime<Utc>>
//     start_date_time: String,
//     end_date_time: String
// }

#[derive(Serialize, Deserialize)]
struct SetTimerData {
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
        println!("tick");
    }
}

// async fn start_timer_device (
//     Json(payload): Json<SetTimerData>,
// ) -> (StatusCode, Json<bool>) {
//     let result = false;
//     // let device = get_device(&payload.mac).await;
//     // TODO: Handle unpacking these values
//     let start_date_time = match payload.start_date_time.parse::<DateTime<Utc>>() {
//         Ok(start_date_time) => start_date_time,
//         Err(_) => return (StatusCode::UNPROCESSABLE_ENTITY, Json(result))
//     };
//     let end_date_time =  match payload.start_date_time.parse::<DateTime<Utc>>() {
//         Ok(start_date_time) => start_date_time,
//         Err(_) => return (StatusCode::UNPROCESSABLE_ENTITY, Json(result))
//     };
//     let duration = end_date_time - start_date_time;
//     start_timer(duration.num_milliseconds().try_into().unwrap()).await;
//     // Issue turn off command
//     turn_off_device(Json(SimpleDeviceData {
//         alias: payload.alias,
//         mac: payload.mac
//     })).await   
//     // Return true
// }

async fn start_timer_device (
    Json(payload): Json<SetTimerData>,
) -> (StatusCode, Json<bool>) {
    let duration = payload.length_ms;
    start_timer(duration).await;
    turn_off_device(Json(SimpleDeviceData {
        alias: payload.alias,
        mac: payload.mac
    })).await   
}

#[derive(Debug, Clone)]
struct FindDeviceError;


async fn get_device(mac: &String) -> Result<Device, FindDeviceError> {
    let devices = get_devices().await;
    let ret_device: &Device = devices.iter().find_map(|d| match_device(d, &mac)).unwrap();
    
    Ok(ret_device.clone())
}

fn match_device<'device>(device: & 'device Device, mac: &String) -> Option<& 'device Device> {
    let found = match device {
        Device::Unknown(device) => {
            let sys_info = match device.sysinfo() {
                Ok(sys_info) => sys_info,
                // TODO: Return errors correctly
                Err(_) => return None
            };
            sys_info.mac == mac.clone()
        },
        _ => false
    };
    match found {
        true => Some(device),
        false => None
    }
}