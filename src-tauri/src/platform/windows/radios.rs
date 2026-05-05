// Módulos de radios: WiFi y Bluetooth.

use crate::types::{BluetoothDeviceInfo, RadioStates, WifiNetwork};
use windows::Devices::Bluetooth::BluetoothDevice;
use windows::Devices::Enumeration::DeviceInformation;
use windows::Devices::Radios::{Radio, RadioKind, RadioState};
use windows::Devices::WiFi::WiFiAdapter;

pub async fn get_radio_states() -> Result<RadioStates, String> {
    let mut states = RadioStates {
        wifi: false,
        bluetooth: false,
    };

    let radios = Radio::GetRadiosAsync()
        .map_err(|e| e.to_string())?
        .get()
        .map_err(|e| e.to_string())?;

    for i in 0..radios.Size().unwrap_or(0) {
        if let Ok(radio) = radios.GetAt(i) {
            if let (Ok(kind), Ok(state)) = (radio.Kind(), radio.State()) {
                let is_on = state == RadioState::On;
                match kind {
                    RadioKind::WiFi => states.wifi = is_on,
                    RadioKind::Bluetooth => states.bluetooth = is_on,
                    _ => {}
                }
            }
        }
    }

    Ok(states)
}

pub async fn toggle_radio(kind: &str, enable: bool) -> Result<(), String> {
    let radios = Radio::GetRadiosAsync()
        .map_err(|e| e.to_string())?
        .get()
        .map_err(|e| e.to_string())?;

    let target_kind = match kind {
        "wifi" => RadioKind::WiFi,
        "bluetooth" => RadioKind::Bluetooth,
        _ => return Ok(()),
    };

    let target_state = if enable { RadioState::On } else { RadioState::Off };

    for i in 0..radios.Size().unwrap_or(0) {
        if let Ok(radio) = radios.GetAt(i) {
            if radio.Kind().ok() == Some(target_kind) {
                if let Ok(op) = radio.SetStateAsync(target_state) {
                    let _ = op.get();
                }
            }
        }
    }

    Ok(())
}

pub async fn get_wifi_networks() -> Result<Vec<WifiNetwork>, String> {
    let mut networks = Vec::new();

    let adapters = WiFiAdapter::FindAllAdaptersAsync()
        .map_err(|e| e.to_string())?
        .get()
        .map_err(|e| e.to_string())?;

    if let Ok(adapter) = adapters.GetAt(0) {
        if let Ok(report) = adapter.NetworkReport() {
            if let Ok(available) = report.AvailableNetworks() {
                for i in 0..available.Size().unwrap_or(0) {
                    if let Ok(net) = available.GetAt(i) {
                        if let Ok(ssid) = net.Ssid() {
                            let ssid_str = ssid.to_string();
                            let bars = net.SignalBars().unwrap_or(0);
                            if !ssid_str.is_empty()
                                && !networks.iter().any(|n: &WifiNetwork| n.ssid == ssid_str)
                            {
                                networks.push(WifiNetwork {
                                    ssid: ssid_str,
                                    signal_bars: bars,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    networks.sort_by(|a, b| b.signal_bars.cmp(&a.signal_bars));
    Ok(networks)
}

pub async fn get_bluetooth_devices() -> Result<Vec<BluetoothDeviceInfo>, String> {
    let mut devices = Vec::new();

    let selector = BluetoothDevice::GetDeviceSelector().map_err(|e| e.to_string())?;

    let collection = DeviceInformation::FindAllAsyncAqsFilter(&selector)
        .map_err(|e| e.to_string())?
        .get()
        .map_err(|e| e.to_string())?;

    for i in 0..collection.Size().unwrap_or(0) {
        if let Ok(dev) = collection.GetAt(i) {
            let name = dev.Name().unwrap_or_default().to_string();
            if name.is_empty() {
                continue;
            }

            let mut is_connected = false;
            if let Ok(id) = dev.Id() {
                if let Ok(op) = BluetoothDevice::FromIdAsync(&id) {
                    if let Ok(bt_dev) = op.get() {
                        if let Ok(status) = bt_dev.ConnectionStatus() {
                            is_connected = status
                                == windows::Devices::Bluetooth::BluetoothConnectionStatus::Connected;
                        }
                    }
                }
            }

            devices.push(BluetoothDeviceInfo { name, is_connected });
        }
    }

    Ok(devices)
}
