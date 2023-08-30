use crate::eia_client::{
    get_eia_data,
    EIAData
};
use chrono::Local;
use chrono::Duration;
use reqwest;

// Use the following python function to calculate peak hours
// def find_peak_hour_timeframe(demand_data):
//     """
//     Given demand data from the EIA API
//     Find the consecutive hour blocks that constitute roughly 20% of daily energy demand
//     i.e. 2pm - 6pm for the East Coast
//     """
//     highest_energy_hour = max(demand_data, key=lambda data: data.value)
//     current_index = demand_data.index(highest_energy_hour)
//     left_index = current_index - 1
//     right_index = current_index + 1
//     peak_hours_percentage = highest_energy_hour.value / total_megawatt_hours
    
//     while peak_hours_percentage < 0.20 and left_index > -1 and right_index < len(demand_data):
//         left_value = demand_data[left_index].value
//         right_value = demand_data[right_index].value
//         if left_value > right_value:
//             peak_hours_percentage += (left_value / total_megawatt_hours)
//             left_index -= 1
//         else:
//             peak_hours_percentage += (right_value / total_megawatt_hours)
//             right_index += 1
//     return demand_data[left_index], demand_data[right_index], peak_hours_percentage

pub async fn find_peak_hour_timeframe() -> Result<Vec<EIAData>, reqwest::Error>{
    let mut result: Vec<EIAData> = Vec::new();
    let json_result = get_eia_data().await?;

    let response = json_result.response;
    let data = response.data;
    
    // Current using yesterday's data as current forecast is not available
    let local_datetime = Local::now() - Duration::days(1);
    let current_day_str = local_datetime.format("%Y-%m-%d").to_string();

    let current_day_data = data.iter().filter(
        |d| d.period.contains(&current_day_str) && d.r#type == "DF"
    );

    let high_hour_data = current_day_data.clone().max_by_key(|d| d.value).unwrap();
    let high_hour_index = current_day_data.clone().position(|d| d.value == high_hour_data.value).unwrap();
    let total_megawatt_hours = current_day_data.map(|d| d.value).reduce(|acc, v| acc + v).unwrap();
    let mut peak_hours_percentage = (high_hour_data.value / total_megawatt_hours) as f32;
    let mut left_index = high_hour_index - 1;
    let mut right_index = high_hour_index + 1;

    while peak_hours_percentage > 0.2 {

    }

    // TODO: Implement loop on data here to find peak hours

    println!("{:?}", high_hour_data);

    Ok(data)
}