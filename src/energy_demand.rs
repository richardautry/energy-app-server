use crate::eia_client::{
    get_eia_data,
    EIAData
};
use chrono::Local;
use chrono::Duration;
use chrono::DateTime;
use chrono::LocalResult;
use reqwest;

// TODO: Change return to tuple of datetime
pub async fn find_peak_hour_timeframe() -> Result<(DateTime<Local>, DateTime<Local>), reqwest::Error>{
    let mut result: Vec<EIAData> = Vec::new();
    let json_result = get_eia_data().await?;

    let response = json_result.response;
    let data = response.data;
    
    // Current using yesterday's data as current forecast is not available
    let local_datetime = Local::now() - Duration::days(1);
    let current_day_str = local_datetime.format("%Y-%m-%d").to_string();

    let current_day_data: Vec<&EIAData> = data.iter().filter(
        |d| d.period.contains(&current_day_str) && d.r#type == "DF"
    ).collect();

    let high_hour_data = current_day_data.iter().max_by_key(|d| d.value).unwrap();
    let high_hour_index = current_day_data.iter().position(|d| d.value == high_hour_data.value).unwrap();
    let total_megawatt_hours = current_day_data.iter().map(|d| d.value).reduce(|acc, v| acc + v).unwrap();
    let mut peak_hours_percentage = high_hour_data.value as f32 / total_megawatt_hours as f32;

    let mut left_vec = Vec::from(&current_day_data[0..high_hour_index]);
    left_vec.reverse();
    let mut left_iter = left_vec.iter().peekable();
    let right_vec = Vec::from(&current_day_data[high_hour_index + 1..]);
    let mut right_iter = right_vec.iter().peekable();
    let mut left_data = left_iter.next().unwrap();
    let mut right_data = right_iter.next().unwrap();
    let mut left_index = high_hour_index - 1;
    let mut right_index = high_hour_index + 1;

    while peak_hours_percentage < 0.2 && left_iter.peek().is_some() && right_iter.peek().is_some() {
        // TODO: Should probably redo this loop logic to check for all states
        if left_data.value > right_data.value {
            peak_hours_percentage += left_data.value as f32 / total_megawatt_hours as f32;
            left_data = left_iter.next().unwrap();
            left_index -= 1;
        } else {
            peak_hours_percentage += right_data.value as f32 / total_megawatt_hours as f32;
            right_data = right_iter.next().unwrap();
            right_index += 1;
        }
    }

    println!("{:?}", high_hour_data);
    println!("peak_hours_percentage: {:.2}", peak_hours_percentage);
    println!("Left Index {}, Right Index {}", left_index, right_index);
    println!("{} to {}", &current_day_data[left_index].period, &current_day_data[right_index].period);

    let left_datetime_string = current_day_data[left_index].period.to_owned();
    let right_datetime_string = current_day_data[right_index].period.to_owned();
    let local_timezone = local_datetime.timezone();

    let left_datetime = get_datetime_from_string(left_datetime_string, local_timezone).await.unwrap();
    let right_datetime = get_datetime_from_string(right_datetime_string, local_timezone).await.unwrap();

    println!("{} to {}", left_datetime, right_datetime);

    Ok((left_datetime, right_datetime))
}

// TODO: Refactor out timedate parsing to get left and right datetimes
pub async fn get_datetime_from_string(datetime_string: String, timezone: Local) -> LocalResult<DateTime<Local>> {
    let mut datetime_split = datetime_string.split("T");
    let date_string = datetime_split.next().unwrap();
    let time_string = datetime_split.next().unwrap();
    let hour_string = &time_string[..2];
    let tz_string = &time_string[2..];  // TODO: Parse timezone from API response
    let date = Local::now().date_naive();

    date.and_hms_opt(
        hour_string.parse::<u32>().unwrap(), 0, 0
    ).unwrap().and_local_timezone(timezone)
}