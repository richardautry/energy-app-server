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