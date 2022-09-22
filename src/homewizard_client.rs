use crate::model::Config;
use jarvis_lib::measurement_client::MeasurementClient;
use jarvis_lib::model::{EntityType, Measurement, MetricType, Sample, SampleType};

use chrono::Utc;
use log::{debug, info};
use mdns_sd::{ServiceDaemon, ServiceEvent};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
use std::error::Error;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::time::Duration;
use uuid::Uuid;

pub struct HomewizardClientConfig {
    timeout_seconds: u64,
}

impl HomewizardClientConfig {
    pub fn new(timeout_seconds: u64) -> Result<Self, Box<dyn Error>> {
        debug!(
            "HomewizardClientConfig::new(timeout_seconds: {})",
            timeout_seconds
        );
        Ok(Self { timeout_seconds })
    }

    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        let timeout_seconds: u64 = env::var("TIMEOUT_SECONDS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()?;

        Self::new(timeout_seconds)
    }
}

pub struct HomewizardClient {
    config: HomewizardClientConfig,
}

impl MeasurementClient<Config> for HomewizardClient {
    fn get_measurement(
        &self,
        config: Config,
        _last_measurement: Option<Measurement>,
    ) -> Result<Measurement, Box<dyn Error>> {
        info!("Reading measurement from homewizard devices...");

        let mut measurement = Measurement {
            id: Uuid::new_v4().to_string(),
            source: String::from("jarvis-homewizard-exporter"),
            location: config.location,
            samples: Vec::new(),
            measured_at_time: Utc::now(),
        };

        info!("Discovering devices...");
        let devices = self.discover_devices()?;

        for device in devices.iter() {
            match self.get_samples(device) {
                Ok(samples) => {
                    measurement.samples.append(&mut samples.clone());
                }
                Err(_) => continue,
            }
        }

        info!("Read measurement from Homewizard devices");

        Ok(measurement)
    }
}

impl HomewizardClient {
    pub fn new(config: HomewizardClientConfig) -> Self {
        Self { config }
    }

    fn get_samples(&self, device: &HomewizardDevice) -> Result<Vec<Sample>, Box<dyn Error>> {
        // get general device data to determine type and name
        let device_info_response = reqwest::blocking::get(format!(
            "http://{}/api",
            device.ip_addresses.iter().next().unwrap()
        ))?
        .json::<DeviceInfoResponse>()?;

        match HomewizardDeviceType::from_str(&device_info_response.product_type).unwrap() {
            HomewizardDeviceType::EnergySocket => {
                // get measurement data
                let data_response = reqwest::blocking::get(format!(
                    "http://{}/api/{}/data",
                    device.ip_addresses.iter().next().unwrap(),
                    device_info_response.api_version
                ))?
                .json::<EnergySocketDataResponse>()?;

                Ok(vec![
                    Sample {
                        entity_type: EntityType::Device,
                        entity_name: device_info_response.product_type.clone(),
                        sample_type: SampleType::ElectricityConsumption,
                        sample_name: device_info_response.product_name.clone(),
                        metric_type: MetricType::Counter,
                        value: data_response.total_power_import_t1_kwh * 1000.0 * 3600.0,
                    },
                    Sample {
                        entity_type: EntityType::Device,
                        entity_name: device_info_response.product_type.clone(),
                        sample_type: SampleType::ElectricityProduction,
                        sample_name: device_info_response.product_name.clone(),
                        metric_type: MetricType::Counter,
                        value: data_response.total_power_export_t1_kwh * 1000.0 * 3600.0,
                    },
                    Sample {
                        entity_type: EntityType::Device,
                        entity_name: device_info_response.product_type.clone(),
                        sample_type: SampleType::ElectricityConsumption,
                        sample_name: device_info_response.product_name.clone(),
                        metric_type: MetricType::Gauge,
                        value: data_response.active_power_w,
                    },
                ])
            }
            HomewizardDeviceType::SinglePhaseKwhMeter => {
                // get measurement data
                let data_response = reqwest::blocking::get(format!(
                    "http://{}/api/{}/data",
                    device.ip_addresses.iter().next().unwrap(),
                    device_info_response.api_version
                ))?
                .json::<SinglePhaseKwhMeterDataResponse>()?;

                Ok(vec![
                    Sample {
                        entity_type: EntityType::Device,
                        entity_name: device_info_response.product_type.clone(),
                        sample_type: SampleType::ElectricityConsumption,
                        sample_name: device_info_response.product_name.clone(),
                        metric_type: MetricType::Counter,
                        value: data_response.total_power_import_t1_kwh * 1000.0 * 3600.0,
                    },
                    Sample {
                        entity_type: EntityType::Device,
                        entity_name: device_info_response.product_type.clone(),
                        sample_type: SampleType::ElectricityProduction,
                        sample_name: device_info_response.product_name.clone(),
                        metric_type: MetricType::Counter,
                        value: data_response.total_power_export_t1_kwh * 1000.0 * 3600.0,
                    },
                    Sample {
                        entity_type: EntityType::Device,
                        entity_name: device_info_response.product_type.clone(),
                        sample_type: SampleType::ElectricityConsumption,
                        sample_name: device_info_response.product_name.clone(),
                        metric_type: MetricType::Gauge,
                        value: data_response.active_power_w,
                    },
                ])
            }
            HomewizardDeviceType::TriplePhaseKwhMeter => {
                // get measurement data
                let data_response = reqwest::blocking::get(format!(
                    "http://{}/api/{}/data",
                    device.ip_addresses.iter().next().unwrap(),
                    device_info_response.api_version
                ))?
                .json::<TriplePhaseKwhMeterDataResponse>()?;

                Ok(vec![
                    Sample {
                        entity_type: EntityType::Device,
                        entity_name: device_info_response.product_type.clone(),
                        sample_type: SampleType::ElectricityConsumption,
                        sample_name: device_info_response.product_name.clone(),
                        metric_type: MetricType::Counter,
                        value: data_response.total_power_import_t1_kwh * 1000.0 * 3600.0,
                    },
                    Sample {
                        entity_type: EntityType::Device,
                        entity_name: device_info_response.product_type.clone(),
                        sample_type: SampleType::ElectricityProduction,
                        sample_name: device_info_response.product_name.clone(),
                        metric_type: MetricType::Counter,
                        value: data_response.total_power_export_t1_kwh * 1000.0 * 3600.0,
                    },
                    Sample {
                        entity_type: EntityType::Device,
                        entity_name: device_info_response.product_type.clone(),
                        sample_type: SampleType::ElectricityConsumption,
                        sample_name: device_info_response.product_name.clone(),
                        metric_type: MetricType::Gauge,
                        value: data_response.active_power_w,
                    },
                ])
            }
            HomewizardDeviceType::WaterMeter => {
                // get measurement data
                let data_response = reqwest::blocking::get(format!(
                    "http://{}/api/{}/data",
                    device.ip_addresses.iter().next().unwrap(),
                    device_info_response.api_version
                ))?
                .json::<WaterMeterDataResponse>()?;

                Ok(vec![
                    Sample {
                        entity_type: EntityType::Device,
                        entity_name: device_info_response.product_type.clone(),
                        sample_type: SampleType::WaterConsumption,
                        sample_name: device_info_response.product_name.clone(),
                        metric_type: MetricType::Counter,
                        value: data_response.total_liter_m3,
                    },
                    Sample {
                        entity_type: EntityType::Device,
                        entity_name: device_info_response.product_type.clone(),
                        sample_type: SampleType::WaterConsumption,
                        sample_name: device_info_response.product_name.clone(),
                        metric_type: MetricType::Gauge,
                        value: data_response.active_liter_lpm,
                    },
                ])
            }
            HomewizardDeviceType::P1Meter => {
                // get measurement data
                let data_response = reqwest::blocking::get(format!(
                    "http://{}/api/{}/data",
                    device.ip_addresses.iter().next().unwrap(),
                    device_info_response.api_version
                ))?
                .json::<P1MeterDataResponse>()?;

                Ok(vec![
                    Sample {
                        entity_type: EntityType::Tariff,
                        entity_name: device_info_response.product_type.clone(),
                        sample_type: SampleType::ElectricityConsumption,
                        sample_name: "t1 import".into(),
                        metric_type: MetricType::Counter,
                        value: data_response.total_power_import_t1_kwh * 1000.0 * 3600.0,
                    },
                    Sample {
                        entity_type: EntityType::Tariff,
                        entity_name: device_info_response.product_type.clone(),
                        sample_type: SampleType::ElectricityProduction,
                        sample_name: "t1 export".into(),
                        metric_type: MetricType::Counter,
                        value: data_response.total_power_export_t1_kwh * 1000.0 * 3600.0,
                    },
                    Sample {
                        entity_type: EntityType::Tariff,
                        entity_name: device_info_response.product_type.clone(),
                        sample_type: SampleType::ElectricityConsumption,
                        sample_name: "t2 import".into(),
                        metric_type: MetricType::Counter,
                        value: data_response.total_power_import_t2_kwh * 1000.0 * 3600.0,
                    },
                    Sample {
                        entity_type: EntityType::Tariff,
                        entity_name: device_info_response.product_type.clone(),
                        sample_type: SampleType::ElectricityProduction,
                        sample_name: "t2 export".into(),
                        metric_type: MetricType::Counter,
                        value: data_response.total_power_export_t2_kwh * 1000.0 * 3600.0,
                    },
                    Sample {
                        entity_type: EntityType::Device,
                        entity_name: device_info_response.product_type.clone(),
                        sample_type: SampleType::ElectricityConsumption,
                        sample_name: device_info_response.product_name.clone(),
                        metric_type: MetricType::Gauge,
                        value: data_response.active_power_w,
                    },
                ])
            }
        }
    }

    fn discover_devices(&self) -> Result<Vec<HomewizardDevice>, Box<dyn Error>> {
        let mut devices: HashMap<String, HomewizardDevice> = HashMap::new();

        // Create a daemon
        let mdns = ServiceDaemon::new().expect("Failed to create daemon");

        // Browse for a service type.
        let service_type = "_hwenergy._tcp.local.";
        let receiver = mdns.browse(service_type).expect("Failed to browse");

        // while let Ok(event) = receiver.recv() {
        let start = std::time::Instant::now();
        let timeout = Duration::new(self.config.timeout_seconds, 0);

        while let Ok(event) = receiver.recv() {
            match event {
                ServiceEvent::ServiceResolved(info) => {
                    println!(
                        "At {:?}: Resolved a new service: {} IP: {:?}",
                        start.elapsed(),
                        info.get_fullname(),
                        info.get_addresses()
                    );

                    let fullname = info.get_fullname().to_string();
                    let ip_addresses = info.get_addresses().clone();

                    devices.insert(
                        fullname.clone(),
                        HomewizardDevice {
                            fullname,
                            ip_addresses,
                        },
                    );
                }
                other_event => {
                    println!(
                        "At {:?} : Received other event: {:?}",
                        start.elapsed(),
                        &other_event
                    );
                }
            }

            if start.elapsed() > timeout {
                break;
            }
        }

        Ok(devices.into_values().collect())
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum HomewizardDeviceType {
    P1Meter,
    SinglePhaseKwhMeter,
    TriplePhaseKwhMeter,
    EnergySocket,
    WaterMeter,
}

impl FromStr for HomewizardDeviceType {
    type Err = ();

    fn from_str(input: &str) -> Result<HomewizardDeviceType, Self::Err> {
        match input {
            "HWE-P1" => Ok(HomewizardDeviceType::P1Meter),
            "HWE-SKT" => Ok(HomewizardDeviceType::EnergySocket),
            "HWE-WTR" => Ok(HomewizardDeviceType::WaterMeter),
            "SDM230-wifi" => Ok(HomewizardDeviceType::SinglePhaseKwhMeter),
            "SDM630-wifi" => Ok(HomewizardDeviceType::TriplePhaseKwhMeter),
            _ => Err(()),
        }
    }
}

pub struct HomewizardDevice {
    pub fullname: String,
    pub ip_addresses: HashSet<Ipv4Addr>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct DeviceInfoResponse {
    product_type: String,
    product_name: String,
    serial: String,
    firmware_version: String,
    api_version: String,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
struct P1MeterDataResponse {
    pub smr_version: usize,
    pub meter_model: String,
    pub wifi_ssid: String,
    pub wifi_strength: usize,
    pub total_power_import_t1_kwh: f64,
    pub total_power_export_t1_kwh: f64,
    pub total_power_import_t2_kwh: f64,
    pub total_power_export_t2_kwh: f64,
    pub active_power_w: f64,
    pub active_power_l1_w: f64,
    pub active_power_l2_w: f64,
    pub active_power_l3_w: f64,
    pub total_gas_m3: f64,
    pub gas_timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
struct EnergySocketDataResponse {
    pub wifi_ssid: String,
    pub wifi_strength: usize,
    pub total_power_import_t1_kwh: f64,
    pub total_power_export_t1_kwh: f64,
    pub active_power_w: f64,
    pub active_power_l1_w: f64,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
struct SinglePhaseKwhMeterDataResponse {
    pub wifi_ssid: String,
    pub wifi_strength: usize,
    pub total_power_import_t1_kwh: f64,
    pub total_power_export_t1_kwh: f64,
    pub active_power_w: f64,
    pub active_power_l1_w: f64,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
struct TriplePhaseKwhMeterDataResponse {
    pub wifi_ssid: String,
    pub wifi_strength: usize,
    pub total_power_import_t1_kwh: f64,
    pub total_power_export_t1_kwh: f64,
    pub active_power_w: f64,
    pub active_power_l1_w: f64,
    pub active_power_l2_w: f64,
    pub active_power_l3_w: f64,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
struct WaterMeterDataResponse {
    pub wifi_ssid: String,
    pub wifi_strength: usize,
    total_liter_m3: f64,
    active_liter_lpm: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn discover_devices() {
        let homewizard_client = HomewizardClient::new(HomewizardClientConfig {
            timeout_seconds: 10,
        });

        // act
        let devices = homewizard_client
            .discover_devices()
            .expect("Failed retrieving devices");

        assert_eq!(devices.len(), 1);
        assert_eq!(
            devices[0].fullname,
            "watermeter-2D7A68._hwenergy._tcp.local."
        );
    }

    #[test]
    #[ignore]
    fn get_samples() {
        let homewizard_client =
            HomewizardClient::new(HomewizardClientConfig { timeout_seconds: 5 });
        let devices = homewizard_client
            .discover_devices()
            .expect("Failed retrieving devices");
        let mut samples: Vec<Sample> = vec![];

        // act
        for device in devices.iter() {
            match homewizard_client.get_samples(&device) {
                Ok(s) => {
                    samples.append(&mut s.clone());
                }
                Err(_) => continue,
            }
        }

        assert_eq!(samples.len(), 2);
        assert_eq!(samples[0].entity_type, EntityType::Device);
        assert_eq!(samples[0].entity_name, "HWE-WTR");
        assert_eq!(samples[0].sample_type, SampleType::WaterConsumption);
        assert_eq!(samples[0].sample_name, "Watermeter");
        assert_eq!(samples[0].metric_type, MetricType::Counter);
        // assert_eq!(samples[0].value, 0.0);
        assert_eq!(samples[1].entity_type, EntityType::Device);
        assert_eq!(samples[1].entity_name, "HWE-WTR");
        assert_eq!(samples[1].sample_type, SampleType::WaterConsumption);
        assert_eq!(samples[1].sample_name, "Watermeter");
        assert_eq!(samples[1].metric_type, MetricType::Gauge);
        // assert_eq!(samples[1].value, 0.0);
    }
}
