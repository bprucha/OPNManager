use crate::db::Database;
use crate::http_client::make_http_request;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{Manager, State};

const MAX_DATA_POINTS: usize = 120;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InterfaceTraffic {
    pub interfaces: HashMap<String, InterfaceData>,
    pub time: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InterfaceTopData {
    pub records: Vec<InterfaceTopRecord>,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InterfaceTopRecord {
    pub address: String,
    pub rate_bits_in: u64,
    pub rate_bits_out: u64,
    pub rate_bits: u64,
    pub cumulative_bytes_in: u64,
    pub cumulative_bytes_out: u64,
    pub cumulative_bytes: u64,
    pub tags: Vec<String>,
    pub rname: String,
    pub rate_in: String,
    pub rate_out: String,
    pub rate: String,
    pub cumulative_in: String,
    pub cumulative_out: String,
    pub cumulative: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InterfaceData {
    pub name: String,
    pub device: String,
    #[serde(rename = "bytes received")]
    pub bytes_received: String,
    #[serde(rename = "bytes transmitted")]
    pub bytes_transmitted: String,
    pub driver: String,
    pub index: Option<String>,
    pub flags: Option<String>,
    #[serde(rename = "promiscuous listeners")]
    pub promiscuous_listeners: Option<String>,
    #[serde(rename = "send queue length")]
    pub send_queue_length: Option<String>,
    #[serde(rename = "send queue max length")]
    pub send_queue_max_length: Option<String>,
    #[serde(rename = "send queue drops")]
    pub send_queue_drops: Option<String>,
    #[serde(rename = "type")]
    pub interface_type: Option<String>,
    #[serde(rename = "address length")]
    pub address_length: Option<String>,
    #[serde(rename = "header length")]
    pub header_length: Option<String>,
    #[serde(rename = "link state")]
    pub link_state: Option<String>,
    pub vhid: Option<String>,
    pub datalen: Option<String>,
    pub mtu: Option<String>,
    pub metric: Option<String>,
    #[serde(rename = "line rate")]
    pub line_rate: Option<String>,
    #[serde(rename = "packets received")]
    pub packets_received: Option<String>,
    #[serde(rename = "input errors")]
    pub input_errors: Option<String>,
    #[serde(rename = "packets transmitted")]
    pub packets_transmitted: Option<String>,
    #[serde(rename = "output errors")]
    pub output_errors: Option<String>,
    pub collisions: Option<String>,
    #[serde(rename = "multicasts received")]
    pub multicasts_received: Option<String>,
    #[serde(rename = "multicasts transmitted")]
    pub multicasts_transmitted: Option<String>,
    #[serde(rename = "input queue drops")]
    pub input_queue_drops: Option<String>,
    #[serde(rename = "packets for unknown protocol")]
    pub packets_unknown_protocol: Option<String>,
    #[serde(rename = "HW offload capabilities")]
    pub hw_offload_capabilities: Option<String>,
    #[serde(rename = "uptime at attach or stat reset")]
    pub uptime_at_attach: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrafficDataPoint {
    pub timestamp: f64,
    pub interface_name: String,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub bits_per_second_in: u64,
    pub bits_per_second_out: u64,
}

#[derive(Default)]
pub struct TrafficCache {
    data_points: Mutex<Vec<TrafficDataPoint>>,
    last_update: Mutex<Option<InterfaceTraffic>>,
}

impl TrafficCache {
    pub fn new() -> Self {
        Self {
            data_points: Mutex::new(Vec::new()),
            last_update: Mutex::new(None),
        }
    }

    pub fn add_data_point(&self, traffic: &InterfaceTraffic) {
        let mut data_points = self.data_points.lock().unwrap();
        let mut last_update = self.last_update.lock().unwrap();

        if let Some(previous) = last_update.as_ref() {
            let time_diff = traffic.time - previous.time;

            if time_diff <= 0.0 {
                return;
            }

            for (interface_key, current_data) in &traffic.interfaces {
                if let Some(previous_data) = previous.interfaces.get(interface_key) {
                    if current_data.name.is_empty() {
                        continue;
                    }

                    let current_in = current_data.bytes_received.parse::<u64>().unwrap_or(0);
                    let current_out = current_data.bytes_transmitted.parse::<u64>().unwrap_or(0);
                    let previous_in = previous_data.bytes_received.parse::<u64>().unwrap_or(0);
                    let previous_out = previous_data.bytes_transmitted.parse::<u64>().unwrap_or(0);
                    let bytes_diff_in = if current_in >= previous_in {
                        current_in - previous_in
                    } else {
                        current_in
                    };
                    let bytes_diff_out = if current_out >= previous_out {
                        current_out - previous_out
                    } else {
                        current_out
                    };

                    let bps_in = (bytes_diff_in as f64 * 8.0 / time_diff) as u64;
                    let bps_out = (bytes_diff_out as f64 * 8.0 / time_diff) as u64;

                    let data_point = TrafficDataPoint {
                        timestamp: traffic.time,
                        interface_name: current_data.name.clone(),
                        bytes_in: current_in,
                        bytes_out: current_out,
                        bits_per_second_in: bps_in,
                        bits_per_second_out: bps_out,
                    };

                    data_points.push(data_point);
                }
            }
        }

        if data_points.len() > MAX_DATA_POINTS {
            let excess = data_points.len() - MAX_DATA_POINTS;
            data_points.drain(0..excess);
        }

        *last_update = Some(traffic.clone());
    }

    pub fn get_data_points(&self) -> Vec<TrafficDataPoint> {
        self.data_points.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        let mut data_points = self.data_points.lock().unwrap();
        data_points.clear();

        let mut last_update = self.last_update.lock().unwrap();
        *last_update = None;
    }
}

#[tauri::command]
pub async fn get_interface_traffic(
    database: State<'_, Database>,
) -> Result<InterfaceTraffic, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!(
        "{}:{}/api/diagnostics/traffic/interface",
        api_info.api_url, api_info.port
    );

    let response = make_http_request(
        "GET",
        &url,
        None,
        None,
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    response
        .json::<InterfaceTraffic>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

#[tauri::command]
pub async fn get_interface_top_traffic(
    database: State<'_, Database>,
    interfaces: Vec<String>,
) -> Result<HashMap<String, InterfaceTopData>, String> {
    let api_info = database
        .get_default_api_info()
        .map_err(|e| format!("Failed to get API info: {}", e))?
        .ok_or_else(|| "API info not found".to_string())?;

    let url = format!(
        "{}:{}/api/diagnostics/traffic/top/{}",
        api_info.api_url, api_info.port, interfaces.join(",")
    );

    let response = make_http_request(
        "GET",
        &url,
        None,
        None,
        Some(30),
        Some(&api_info.api_key),
        Some(&api_info.api_secret),
    )
    .await?;

    response
        .json::<HashMap<String, InterfaceTopData>>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

#[tauri::command]
pub fn get_traffic_graph_data(
    traffic_cache: State<'_, TrafficCache>,
) -> Result<Vec<TrafficDataPoint>, String> {
    Ok(traffic_cache.get_data_points())
}

#[tauri::command]
pub async fn update_traffic_data(
    database: State<'_, Database>,
    traffic_cache: State<'_, TrafficCache>,
) -> Result<(), String> {
    let traffic = get_interface_traffic(database).await?;
    traffic_cache.add_data_point(&traffic);
    Ok(())
}

pub fn register_traffic_cache(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let traffic_cache = TrafficCache::new();
    app.manage(traffic_cache);
    Ok(())
}

#[tauri::command]
pub fn clear_traffic_cache(traffic_cache: State<'_, TrafficCache>) -> Result<(), String> {
    traffic_cache.clear();
    Ok(())
}
