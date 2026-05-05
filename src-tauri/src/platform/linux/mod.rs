// Stubs de plataforma Linux para futura implementación.
//
// Cada función devuelve un valor por defecto seguro.
// Las implementaciones reales se añadirán cuando se porte a Linux.

use crate::types::{
    BatteryInfo, BluetoothDeviceInfo, NetworkStatus, PowerProfile, RadioStates, VolumeInfo,
    WifiNetwork,
};

pub fn get_brightness() -> u8 {
    // TODO: leer de /sys/class/backlight/*/brightness
    50
}

pub fn set_brightness(_value: u8) {
    // TODO: escribir en /sys/class/backlight/*/brightness
}

pub fn get_battery_status() -> BatteryInfo {
    // TODO: leer de /sys/class/power_supply/BAT*/
    for i in 0..2 {
        let cap_path = format!("/sys/class/power_supply/BAT{}/capacity", i);
        let stat_path = format!("/sys/class/power_supply/BAT{}/status", i);

        if let Ok(cap_str) = std::fs::read_to_string(&cap_path) {
            if let Ok(cap_val) = cap_str.trim().parse::<u8>() {
                let charging = std::fs::read_to_string(&stat_path)
                    .map(|s| s.trim() == "Charging" || s.trim() == "Full")
                    .unwrap_or(false);

                return BatteryInfo {
                    percentage: cap_val,
                    is_charging: charging,
                    battery_saver: false,
                };
            }
        }
    }

    BatteryInfo {
        percentage: 100,
        is_charging: true,
        battery_saver: false,
    }
}

pub fn get_volume_status() -> VolumeInfo {
    // TODO: PulseAudio / PipeWire
    VolumeInfo {
        volume: 0.5,
        is_muted: false,
    }
}

pub fn set_volume(_value: f32) {}
pub fn toggle_mute() {}

pub fn get_power_profiles() -> Vec<PowerProfile> {
    vec![]
}

pub fn set_power_profile(_guid: &str) {}

pub fn get_network_status() -> NetworkStatus {
    NetworkStatus {
        is_online: false,
        connection_type: "unknown".to_string(),
        signal_strength: 0,
    }
}

pub async fn get_radio_states() -> Result<RadioStates, String> {
    Ok(RadioStates {
        wifi: false,
        bluetooth: false,
    })
}

pub async fn toggle_radio(_kind: &str, _enable: bool) -> Result<(), String> {
    Ok(())
}

pub async fn get_wifi_networks() -> Result<Vec<WifiNetwork>, String> {
    Ok(vec![])
}

pub async fn get_bluetooth_devices() -> Result<Vec<BluetoothDeviceInfo>, String> {
    Ok(vec![])
}

pub fn media_play_pause() {
    // TODO: enigo / xdotool
    #[cfg(not(target_os = "windows"))]
    {
        use enigo::{Enigo, Key, KeyboardControllable};
        Enigo::new().key_click(Key::MediaPlayPause);
    }
}

pub fn media_next() {
    #[cfg(not(target_os = "windows"))]
    {
        use enigo::{Enigo, Key, KeyboardControllable};
        Enigo::new().key_click(Key::MediaNextTrack);
    }
}

pub fn media_prev() {
    #[cfg(not(target_os = "windows"))]
    {
        use enigo::{Enigo, Key, KeyboardControllable};
        Enigo::new().key_click(Key::MediaPrevTrack);
    }
}

pub fn volume_mute() {
    #[cfg(not(target_os = "windows"))]
    {
        use enigo::{Enigo, Key, KeyboardControllable};
        Enigo::new().key_click(Key::VolumeMute);
    }
}

pub fn volume_up() {
    #[cfg(not(target_os = "windows"))]
    {
        use enigo::{Enigo, Key, KeyboardControllable};
        Enigo::new().key_click(Key::VolumeUp);
    }
}

pub fn volume_down() {
    #[cfg(not(target_os = "windows"))]
    {
        use enigo::{Enigo, Key, KeyboardControllable};
        Enigo::new().key_click(Key::VolumeDown);
    }
}

pub fn open_start_menu() {
    #[cfg(not(target_os = "windows"))]
    {
        use enigo::{Enigo, Key, KeyboardControllable};
        Enigo::new().key_click(Key::Meta);
    }
}

pub fn get_monitor_rects() -> Vec<crate::types::MonitorRect> {
    vec![]
}

pub fn get_taskbar_apps() -> Vec<crate::types::TaskbarApp> {
    vec![]
}

pub fn reorder_apps(_ordered_ids: Vec<String>) {}

pub fn hide_windows_taskbar() {}
pub fn restore_windows_taskbar() {}

pub fn interact_app(_hwnd: isize, _exec_path: &str) {}
