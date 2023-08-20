use axum::{
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use tplinker::{
    datatypes::{SysInfo},
    error::{Error as TPLinkerError, Result as TPLinkerResult},
    capabilities::DeviceActions,
};
use tplinker::{
    discovery::discover,
    devices::Device,
};
use serde_json::json;

async fn get_devices() -> Vec<Device> {
    let mut devices: Vec<Device> = Vec::new();
    for (addr, data) in discover().unwrap() {
        let device = Device::from_data(addr, &data);
        devices.push(device)
    }
    devices
}

pub async fn device_data() -> Json<Vec<SimpleDeviceData>> {
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

pub async fn turn_on_device(
    Json(payload): Json<SimpleDeviceData>,
) -> (StatusCode, Json<bool>) {
    turn_on_off_device(payload, true).await
}

pub async fn turn_off_device(
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
pub struct SimpleDeviceData {
    pub alias: String,
    pub mac: String
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