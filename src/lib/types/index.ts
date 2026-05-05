// Tipos compartidos entre todos los componentes y rutas.
// Refleja exactamente las structs Rust de types.rs.

export interface TaskbarApp {
  id: string;
  title: string;
  icon_base64: string;
  is_active: boolean;
  is_pinned: boolean;
  hwnd: number;
  exec_path: string;
}

export interface MonitorRect {
  x: number;
  y: number;
  width: number;
  height: number;
  scale_factor: number;
}

export interface BatteryInfo {
  percentage: number;
  is_charging: boolean;
  battery_saver: boolean;
}

export interface VolumeInfo {
  volume: number;
  is_muted: boolean;
}

export interface NetworkStatus {
  is_online: boolean;
  connection_type: "wifi" | "ethernet" | "none" | "unknown";
  signal_strength: number;
}

export interface RadioStates {
  wifi: boolean;
  bluetooth: boolean;
}

export interface WifiNetwork {
  ssid: string;
  signal_bars: number;
}

export interface BluetoothDeviceInfo {
  name: string;
  is_connected: boolean;
}

export interface PowerProfile {
  guid: string;
  name: string;
  active: boolean;
}

export interface MediaInfo {
  title: string;
  artist: string;
  album: string;
  app_id: string;
  thumbnail_base64: string;
  is_playing: boolean;
  position_ms: number;
  duration_ms: number;
}

/** Estado de arrastre de iconos en la barra de tareas */
export interface DragOrigin {
  id: string;
  app: TaskbarApp;
  startX: number;
  startY: number;
}
