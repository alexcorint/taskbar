// Tipos de datos compartidos entre módulos y comandos Tauri.
// Separados aquí para evitar duplicaciones y facilitar el mantenimiento cross-platform.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Media
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct MediaInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub app_id: String,
    pub thumbnail_base64: String,
    pub is_playing: bool,
    pub position_ms: u64,
    pub duration_ms: u64,
}

// ---------------------------------------------------------------------------
// Batería / Energía
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct BatteryInfo {
    pub percentage: u8,
    pub is_charging: bool,
    pub battery_saver: bool,
}

// ---------------------------------------------------------------------------
// Audio / Volumen
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct VolumeInfo {
    pub volume: f32,
    pub is_muted: bool,
}

// ---------------------------------------------------------------------------
// Red
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct NetworkStatus {
    pub is_online: bool,
    pub connection_type: String,
    pub signal_strength: u8,
}

#[derive(Serialize)]
pub struct RadioStates {
    pub wifi: bool,
    pub bluetooth: bool,
}

#[derive(Serialize)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal_bars: u8,
}

#[derive(Serialize)]
pub struct BluetoothDeviceInfo {
    pub name: String,
    pub is_connected: bool,
}

// ---------------------------------------------------------------------------
// Barra de tareas
// ---------------------------------------------------------------------------

#[derive(Serialize, Clone, Debug)]
pub struct TaskbarApp {
    pub id: String,
    pub title: String,
    pub icon_base64: String,
    pub is_active: bool,
    pub is_pinned: bool,
    pub hwnd: isize,
    pub exec_path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(dead_code)]
pub struct PinnedApp {
    pub id: String,
    pub name: String,
    pub exec_path: String,
}

// ---------------------------------------------------------------------------
// Monitor
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct MonitorRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub scale_factor: f64,
}

// ---------------------------------------------------------------------------
// Acciones deserializables
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaControlAction {
    PlayPause,
    Next,
    Prev,
}

#[derive(Serialize, Deserialize)]
pub enum VolumeAction {
    Mute,
    Up,
    Down,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum WindowAction {
    Show,
    ShowAt { x: i32, y: i32 },
    Hide,
    Update { x: i32, y: i32, w: i32, h: i32 },
    UpdateLogical { x: f64, y: f64, w: f64, h: f64 },
}
